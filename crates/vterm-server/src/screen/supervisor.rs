//! `ScreenServerSupervisor`: vterm-server's handle on the local
//! vterm-screen-server process (coordination spec §2/§4/§6).
//!
//! Modeled on [`crate::voice::AsrSupervisor`]: attach-first (a running
//! instance answering the ctl pipe wins over spawning), `kill_on_drop`
//! child as the dev/debug fallback, config push on (re)connect and on
//! config-watch changes. Health is the pipe itself: a broken pipe marks the
//! client dead and the next operation re-attaches or respawns, then
//! re-pushes the tool-path config (tokens are volatile by design, §8).
//!
//! Unlike the voice supervisor, the screen-server lifecycle **follows
//! `remote_desktop.enabled`**: server startup with the flag on (or a webui
//! toggle to on) brings the screen-server up, a toggle to off tears it down.
//! The `run()` loop owns that decision; direct operations (`ipc()`) still
//! bring the process up on demand for a client request.
//!
//! The Windows release packaging (VTermScreenSvc install/start) is a
//! follow-up; the child-process model covers dev and macOS/Linux today.

use std::net::IpAddr;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, Mutex};
use tracing::{debug, error, info, warn};
use crate::screen::ctl as ctl;

use crate::config::{ConfigManager, RemoteDesktopConfig};

use super::ipc::{IpcClient, SLOW_REQUEST_TIMEOUT};

/// How long to wait for a freshly spawned screen-server to accept its first
/// ctl connection.
const ATTACH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
/// Poll interval while waiting for attach.
const ATTACH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);
/// CoreSimulator accepts power-state changes asynchronously. Keep the client
/// request open until the requested state is visible, but never indefinitely.
const SIMULATOR_STATE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(90);
const SIMULATOR_STATE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);
/// Emulator launch may restart adb and briefly wait for the child to settle.
const ANDROID_EMULATOR_CTL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
/// Stream token ttl (spec §6: one token per stream start).
const TOKEN_TTL_MS: u64 = 60_000;
/// Frame rate pushed when the config holds no explicit value
/// (`RemoteDesktopConfig.screen_fps: None` = 60).
pub const DEFAULT_STREAM_FPS: u32 = 60;

/// The max resolution pushed when the config has no override
/// (`RemoteDesktopConfig.screen_max_resolution: None` = "4k").
pub const DEFAULT_MAX_RESOLUTION: &str = "4k";

/// The screen-server's encoder catalog + effective stream tunables (the ctl
/// `GetEncoders` answer, unpacked).
#[derive(Debug, Clone)]
pub struct EncoderCatalog {
    pub entries: Vec<ctl::EncoderBackendInfo>,
    /// Encoder backend id currently in effect.
    pub current: String,
    /// Effective frame rate (override when set, else the server default).
    pub fps: u32,
    pub fps_override: Option<u32>,
    /// Max-resolution override in effect (None = the "4k" default).
    pub max_resolution: Option<String>,
    /// Fixed-bitrate override in effect, kbit/s (None = auto area-scaled).
    pub bitrate_kbps: Option<u32>,
    /// Virtual-display override in effect (None = the server's startup
    /// default; macOS headless main-display capture).
    pub virtual_display: Option<bool>,
}

/// What a client needs for its UDP media handshake (spec §5.1).
#[derive(Debug, Clone)]
pub struct StreamStart {
    pub udp_port: u16,
    pub token: [u8; 16],
}

/// Windows: a Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` that owns
/// the supervised child. The job handle stays open for the supervisor's
/// lifetime, so when the vterm-server process dies — even a force-kill —
/// the kernel closes the handle and terminates every process in the job.
/// Without it, a force-killed vterm-server orphans the screen-server, which
/// then has to be force-killed later — and a hard-killed process can leave a
/// phantom UDP bind behind (its socket stays bound until the kernel reaps
/// the wedged process object, blocking 38517).
#[cfg(target_os = "windows")]
#[derive(Default)]
struct KillOnCloseJob(Option<windows_sys::Win32::Foundation::HANDLE>);

// SAFETY: the handle is only ever touched while `SupervisorInner` is held
// under its Mutex; Windows handles are plain integer-like values safe to
// move between threads.
#[cfg(target_os = "windows")]
unsafe impl Send for KillOnCloseJob {}

#[cfg(not(target_os = "windows"))]
#[derive(Default)]
struct KillOnCloseJob;

impl KillOnCloseJob {
    /// Assign `pid` to a fresh kill-on-close job (best-effort: nested-job
    /// restrictions or a racing exit are ignored — the graceful shutdown in
    /// [`ScreenServerSupervisor::stop`] remains the primary path).
    #[cfg(target_os = "windows")]
    fn assign(&mut self, pid: u32) {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };
        use windows_sys::Win32::System::Threading::{
            OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
        };
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return;
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            let process = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid);
            if !process.is_null() {
                // Best-effort: nested jobs / a racing exit may reject this.
                let _ = AssignProcessToJobObject(job, process);
                CloseHandle(process);
            }
            // Adopt the job handle: it must stay open for the child's whole
            // life (closing it fires KILL_ON_JOB_CLOSE immediately). It is
            // closed on Drop — process exit, or stop() after the reap.
            self.0 = Some(job);
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn assign(&mut self, _pid: u32) {}
}

#[cfg(target_os = "windows")]
impl Drop for KillOnCloseJob {
    fn drop(&mut self) {
        if let Some(job) = self.0.take() {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(job);
            }
        }
    }
}

// -- Windows service install helpers ----------------------------------------
// The webui's service buttons. The elevated `--install`/`--remove` helpers
// and the SCM status query used to run inside the screen-server (its ctl
// handlers); the install flow now stops every running instance first, so
// the supervisor runs the helper itself against the screen-server binary.

/// SCM service name the screen-server registers via `--install` (mirrors
/// `vterm_screen_server::service::DEFAULT_SERVICE_NAME` — that crate is not
/// a dependency here, only its binary and the ctl protocol are).
#[cfg(target_os = "windows")]
const SCREEN_SERVICE_NAME: &str = "VTermScreenSvc";

/// Run `<exe> <args>` elevated (UAC prompt on the interactive desktop) and
/// wait for its exit — the supervisor-side twin of the screen-server's
/// `service::run_elevated`. `Ok(())` on exit code 0, a user-presentable
/// message otherwise (UAC declined, elevated run failed).
#[cfg(target_os = "windows")]
fn run_elevated_helper(exe: &std::path::Path, args: &str) -> Result<(), String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject};
    use windows_sys::Win32::UI::Shell::{
        ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

    /// Mirrors the screen-server helper's bound: SCM work is fast, the limit
    /// only stops a wedged helper from pinning the HTTP request.
    const ELEVATION_TIMEOUT_MS: u32 = 60_000;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain([0]).collect()
    }

    let file = wide(&exe.to_string_lossy());
    let params = wide(args);
    let verb = wide("runas");
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: verb.as_ptr(),
        lpFile: file.as_ptr(),
        lpParameters: params.as_ptr(),
        nShow: SW_HIDE,
        ..Default::default()
    };
    unsafe {
        if ShellExecuteExW(&mut info) == 0 {
            return Err("elevation was declined (UAC)".to_string());
        }
        if !info.hProcess.is_null() {
            let _ = WaitForSingleObject(info.hProcess, ELEVATION_TIMEOUT_MS);
            let mut code: u32 = 0;
            let ok = GetExitCodeProcess(info.hProcess, &mut code) != 0 && code == 0;
            let _ = CloseHandle(info.hProcess);
            if !ok {
                return Err("the elevated helper failed (see the server log)".to_string());
            }
        }
    }
    Ok(())
}

/// Whether `VTermScreenSvc` is registered with the SCM / currently running
/// (enumerate + query only; no elevation needed).
#[cfg(target_os = "windows")]
fn query_screen_service() -> (bool, bool) {
    use windows_sys::Win32::System::Services::{
        CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatus,
        SC_MANAGER_ENUMERATE_SERVICE, SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_STATUS,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain([0]).collect()
    }

    unsafe {
        let scm = OpenSCManagerW(
            std::ptr::null(),
            std::ptr::null(),
            SC_MANAGER_ENUMERATE_SERVICE,
        );
        if scm.is_null() {
            return (false, false);
        }
        let service = OpenServiceW(
            scm,
            wide(SCREEN_SERVICE_NAME).as_ptr(),
            SERVICE_QUERY_STATUS,
        );
        // A null handle (not registered / access denied) = not installed as
        // far as we can tell.
        let result = if service.is_null() {
            (false, false)
        } else {
            let mut status = SERVICE_STATUS::default();
            let running = QueryServiceStatus(service, &mut status) != 0
                && status.dwCurrentState == SERVICE_RUNNING;
            let _ = CloseServiceHandle(service);
            (true, running)
        };
        let _ = CloseServiceHandle(scm);
        result
    }
}

/// Whether a known Virtual Display Driver device node is present (any state —
/// a disabled device still counts). The webui's service card hides the VDD
/// download instructions when this is true. Mirrors the name-matching probe
/// in vterm-screen-server's `windows::vdd` (`VDD_NEEDLES`); duplicated here
/// because this crate cannot depend on the screen-server, and the status
/// endpoint deliberately answers without spawning one.
#[cfg(target_os = "windows")]
pub(crate) fn vdd_installed() -> bool {
    use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiGetDeviceRegistryPropertyW, DIGCF_ALLCLASSES, DIGCF_PRESENT, HDEVINFO,
        SETUP_DI_REGISTRY_PROPERTY, SPDRP_DEVICEDESC, SPDRP_FRIENDLYNAME, SP_DEVINFO_DATA,
    };

    /// Same needles as vterm-screen-server's `windows::vdd::VDD_NEEDLES` —
    /// keep the two lists in sync.
    const VDD_NEEDLES: &[&str] = &["virtual display driver", "virtual desktop monitor"];

    unsafe fn device_string_property(
        set: HDEVINFO,
        data: *const SP_DEVINFO_DATA,
        prop: SETUP_DI_REGISTRY_PROPERTY,
    ) -> Option<String> {
        let mut size = 0u32;
        let _ = SetupDiGetDeviceRegistryPropertyW(
            set,
            data,
            prop,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            &mut size,
        );
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        if SetupDiGetDeviceRegistryPropertyW(
            set,
            data,
            prop,
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            size,
            &mut size,
        ) == 0
        {
            return None;
        }
        let u16s: Vec<u16> = buf
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        Some(
            String::from_utf16_lossy(&u16s)
                .trim_end_matches('\0')
                .to_string(),
        )
    }

    unsafe {
        let set = SetupDiGetClassDevsW(
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null_mut(),
            DIGCF_ALLCLASSES | DIGCF_PRESENT,
        );
        // Failure returns INVALID_HANDLE_VALUE (-1), not null.
        if set == 0 || set == -1 {
            return false;
        }
        let mut data = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };
        let mut i = 0;
        let mut found = false;
        while SetupDiEnumDeviceInfo(set, i, &mut data) != 0 {
            let name = device_string_property(set, &data, SPDRP_FRIENDLYNAME)
                .or_else(|| device_string_property(set, &data, SPDRP_DEVICEDESC))
                .unwrap_or_default()
                .to_lowercase();
            if VDD_NEEDLES.iter().any(|needle| name.contains(needle)) {
                found = true;
                break;
            }
            data.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;
            i += 1;
        }
        let _ = SetupDiDestroyDeviceInfoList(set);
        found
    }
}

struct SupervisorInner {
    /// The supervised child process, if we spawned one. `None` when attached
    /// to an external instance or when stopped.
    child: Option<Child>,
    /// The child's stdin pipe write-end, held open for the supervisor's
    /// lifetime. Unix anti-orphan: when the vterm-server process dies —
    /// even a force-kill — every fd closes, the child sees stdin EOF, and
    /// its parent-pipe watch shuts it down gracefully (Windows uses the
    /// kill-on-close job instead). Never written to.
    child_stdin: Option<tokio::process::ChildStdin>,
    ipc: Option<Arc<IpcClient>>,
    /// Windows kill-on-close job owning `child` (anti-orphan).
    job: KillOnCloseJob,
}

pub struct ScreenServerSupervisor {
    config_manager: Arc<ConfigManager>,
    inner: Arc<Mutex<SupervisorInner>>,
    events_tx: broadcast::Sender<ctl::Event>,
}

impl ScreenServerSupervisor {
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        let (events_tx, _) = broadcast::channel(64);
        Self {
            config_manager,
            inner: Arc::new(Mutex::new(SupervisorInner {
                child: None,
                child_stdin: None,
                ipc: None,
                job: KillOnCloseJob::default(),
            })),
            events_tx,
        }
    }

    /// Stream/capture lifecycle events from the screen-server
    /// (StreamStarted/Stopped, ClientConnected, Stats, ...).
    pub fn subscribe_events(&self) -> broadcast::Receiver<ctl::Event> {
        self.events_tx.subscribe()
    }

    /// Drive the supervisor: start the screen-server when remote desktop is
    /// enabled (initial bring-up, and on a webui toggle to on), stop it when
    /// the toggle goes off, and push the tool-path/stream config whenever it
    /// changes and a screen-server is connected. Runs forever; spawn on a
    /// background task. Exits (stopping the child) when the config watch
    /// sender drops (shutdown).
    ///
    /// The lifecycle follows `remote_desktop.enabled`: server startup with
    /// the flag on (or a toggle to on) brings the screen-server up, a toggle
    /// to off tears it down; unrelated config changes (paths/encoder/fps)
    /// are pushed over the ctl pipe instead of restarting.
    pub async fn run(self: Arc<Self>) {
        let mut cfg_rx = self.config_manager.subscribe();
        // Initial bring-up: honor the persisted `remote_desktop.enabled` so a
        // server configured for remote desktop starts its screen-server
        // without waiting for the first stream request.
        let mut enabled = cfg_rx.borrow().remote_desktop.enabled;
        if enabled {
            if let Err(e) = self.ipc().await {
                error!("initial screen-server bring-up failed: {e:?}");
            }
        }

        loop {
            if cfg_rx.changed().await.is_err() {
                self.stop().await;
                return;
            }
            let cfg = cfg_rx.borrow().remote_desktop.clone();
            if cfg.enabled != enabled {
                enabled = cfg.enabled;
                if enabled {
                    // Webui toggle to on: bring the screen-server up.
                    info!("remote desktop enabled; starting screen-server");
                    if let Err(e) = self.ipc().await {
                        error!("screen-server bring-up after enable failed: {e:?}");
                    }
                } else {
                    // Webui toggle to off: tear the screen-server down.
                    info!("remote desktop disabled; stopping screen-server");
                    self.stop().await;
                    continue;
                }
            }
            let ipc = self.inner.lock().await.ipc.clone();
            if let Some(ipc) = ipc {
                if !ipc.is_dead() {
                    if let Err(e) = push_paths(&ipc, &cfg).await {
                        warn!("screen-server config push failed: {e:#}");
                    }
                    if let Err(e) = push_stream_config(&ipc, &cfg).await {
                        warn!("screen-server stream-config push failed: {e:#}");
                    }
                }
            }
        }
    }

    /// Stop the supervised child (if any) and drop the IPC connection.
    ///
    /// Graceful first: a ctl `Shutdown` makes the screen-server close its UDP
    /// sockets and exit on its own. A force-killed child can leave a phantom
    /// UDP bind on Windows — the process is gone but its socket stays bound
    /// until the kernel reaps the (wedged) process object, blocking 38517.
    /// The force-kill below is only the fallback when the child ignores the
    /// request; the child is always reaped either way.
    pub async fn stop(&self) {
        // Best-effort graceful shutdown over the ctl pipe (before dropping
        // the connection).
        let ipc = self.inner.lock().await.ipc.clone();
        if let Some(ipc) = ipc {
            if !ipc.is_dead() {
                if let Err(e) = ipc.request(ctl::Cmd::Shutdown).await {
                    debug!("graceful screen-server shutdown request failed: {e:#}");
                }
            }
        }
        let mut inner = self.inner.lock().await;
        inner.ipc = None;
        // Close the lifetime pipe first: EOF tells the child's parent-pipe
        // watch to exit even if everything below fails.
        inner.child_stdin = None;
        if let Some(mut child) = inner.child.take() {
            info!("stopping managed vterm-screen-server child");
            // Give the child a short window to exit cleanly after Shutdown,
            // then force-kill + reap it (a zombie child holds its sockets).
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break, // exited cleanly
                    Ok(None) if std::time::Instant::now() < deadline => {
                        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                    }
                    _ => {
                        debug!("screen-server did not exit after Shutdown; force-killing");
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                        break;
                    }
                }
            }
        }
    }

    /// The live IPC client, connecting (attach-first, else spawn) when
    /// needed. This is the bring-up path for every operation.
    ///
    /// Bring-up honors `remote_desktop.enabled`: when the flag is off, no
    /// connection is made and every operation fails with the actionable
    /// "remote desktop is disabled" error instead of resurrecting the
    /// process — a webui paths/encoder query must not start the screen-server
    /// behind a disabled toggle, nor re-start it right after a toggle-off.
    async fn ipc(&self) -> Result<Arc<IpcClient>> {
        // Windows: while the VTermScreenSvc service runs, its worker owns
        // the ctl endpoint — get our standby standalone out of its way.
        #[cfg(target_os = "windows")]
        self.yield_to_service_worker().await;
        {
            let inner = self.inner.lock().await;
            if let Some(ipc) = &inner.ipc {
                if !ipc.is_dead() {
                    return Ok(Arc::clone(ipc));
                }
            }
        }
        let enabled = self.config_manager.snapshot().await.remote_desktop.enabled;
        if !enabled {
            bail!("remote desktop is disabled: enable it in Settings → Remote Desktop to start the screen-server");
        }
        self.ensure_connected().await
    }

    /// When the `VTermScreenSvc` service is running, its worker owns the
    /// ctl endpoint. A standby standalone of ours (lazily spawned before the
    /// service came up) would keep doing the capture/encoding itself AND
    /// permanently block the worker from creating the pipe — the endpoint is
    /// single-instance, so exactly one process can hold it. Stop our standby
    /// in that case; `ensure_connected` then attaches to the worker (the
    /// service watchdog retries the worker spawn every ~5s). Externally
    /// launched standalones are not touched (the service-install flow
    /// already stops those).
    #[cfg(target_os = "windows")]
    async fn yield_to_service_worker(&self) {
        let owns_standby = self.inner.lock().await.child.is_some();
        if owns_standby && query_screen_service().1 {
            info!(
                "VTermScreenSvc is running; stopping our standby vterm-screen-server \
                 so the service worker takes over the ctl endpoint"
            );
            self.stop().await;
        }
    }

    /// Attach-first connect: a live endpoint answering with a Hello wins;
    /// otherwise spawn the child and wait for its endpoint. After any fresh
    /// connection the persisted tool-path config is (re)pushed.
    async fn ensure_connected(&self) -> Result<Arc<IpcClient>> {
        let mut inner = self.inner.lock().await;
        if let Some(ipc) = &inner.ipc {
            if !ipc.is_dead() {
                return Ok(Arc::clone(ipc));
            }
        }
        inner.ipc = None;

        // While the service runs (Windows), never spawn a competing
        // standalone — its worker owns the ctl endpoint, and a standalone
        // would lock the worker out permanently. Keep retrying the attach
        // instead; the watchdog brings the worker up within ~5s of the pipe
        // becoming free.
        #[cfg(target_os = "windows")]
        let service_deadline = if query_screen_service().1 {
            Some(std::time::Instant::now() + ATTACH_TIMEOUT)
        } else {
            None
        };
        #[cfg(not(target_os = "windows"))]
        let service_deadline: Option<std::time::Instant> = None;

        loop {
            match try_attach(&self.events_tx).await {
                Ok(ipc) => {
                    info!(
                        "attached to a running vterm-screen-server (session {})",
                        ipc.hello.session_id
                    );
                    // Best-effort config push: a push failure must not fail the
                    // attach (the run() task re-pushes on the next config change).
                    let cfg = self.config_manager.snapshot().await.remote_desktop;
                    if let Err(e) = push_paths(&ipc, &cfg).await {
                        warn!("screen-server config push after attach failed: {e:#}");
                    }
                    if let Err(e) = push_stream_config(&ipc, &cfg).await {
                        warn!("screen-server stream-config push after attach failed: {e:#}");
                    }
                    inner.ipc = Some(Arc::clone(&ipc));
                    return Ok(ipc);
                }
                Err(e) => match service_deadline {
                    None => {
                        debug!("no attachable screen-server ({e:#}); spawning");
                        break; // fall through to the spawn path
                    }
                    Some(deadline) if std::time::Instant::now() < deadline => {
                        debug!(
                            "VTermScreenSvc is running; waiting for its worker's ctl \
                             endpoint ({e:#})"
                        );
                        tokio::time::sleep(ATTACH_INTERVAL).await;
                    }
                    Some(_) => {
                        bail!(
                            "the VTermScreenSvc worker did not answer its ctl endpoint \
                             within {}s (see the service log at {}): {e:#}",
                            ATTACH_TIMEOUT.as_secs(),
                            r"C:\ProgramData\VibeTerminal"
                        );
                    }
                },
            }
        }

        // Spawn the child (dev/child-process model).
        let bin = screen_server_binary_path().with_context(|| {
            "vterm-screen-server binary not found: build it with `cargo build -p vterm-screen-server` \
             and place it next to vterm-server (or under ~/.vibeterminal/bin/)"
        })?;
        let mut cmd = Command::new(&bin);
        cmd.kill_on_drop(true)
            .arg("--standalone")
            // Piped so the child's parent-pipe watch can detect our death
            // (EOF); the write-end is held in `SupervisorInner::child_stdin`.
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        // Same CREATE_NO_WINDOW rationale as vterm_common::command_no_window,
        // inline because this is a tokio Command.
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x0800_0000);
        info!("spawning vterm-screen-server: {:?}", cmd.as_std());
        let mut child = cmd
            .spawn()
            .with_context(|| format!("failed to spawn {}", bin.display()))?;
        // Hold the pipe write-end for the child's whole life (Unix
        // anti-orphan; see `SupervisorInner::child_stdin`).
        inner.child_stdin = child.stdin.take();
        // Kill-on-close job: even a force-killed vterm-server takes the
        // screen-server down with it (no orphan → no later hard kill → no
        // phantom UDP bind).
        if let Some(pid) = child.id() {
            inner.job.assign(pid);
        }
        //         if let Some(stdout) = child.stdout.take() {
        //             tokio::spawn(async move {
        //                 use tokio::io::{AsyncBufReadExt, BufReader};
        //                 let mut lines = BufReader::new(stdout).lines();
        //                 while let Ok(Some(line)) = lines.next_line().await {
        //                     debug!("[screen-server] {line}");
        //                 }
        //             });
        //         }
        //         if let Some(stderr) = child.stderr.take() {
        //             tokio::spawn(async move {
        //                 use tokio::io::{AsyncBufReadExt, BufReader};
        //                 let mut lines = BufReader::new(stderr).lines();
        //                 while let Ok(Some(line)) = lines.next_line().await {
        //                     debug!("[screen-server:err] {line}");
        //                 }
        //             });
        //         }

        // Wait for its endpoint to accept us.
        let deadline = std::time::Instant::now() + ATTACH_TIMEOUT;
        let ipc = loop {
            match try_attach(&self.events_tx).await {
                Ok(ipc) => break ipc,
                Err(e) => {
                    if std::time::Instant::now() >= deadline {
                        let _ = child.kill().await;
                        inner.child_stdin = None;
                        return Err(anyhow!("vterm-screen-server never came up: {e:#}"));
                    }
                    tokio::time::sleep(ATTACH_INTERVAL).await;
                }
            }
        };
        info!(
            "vterm-screen-server is up (session {}, udp {})",
            ipc.hello.session_id, ipc.hello.udp_port
        );
        // Adopt the child BEFORE the config pushes: the Child is
        // kill_on_drop, so a push failure must not SIGKILL a freshly spawned,
        // healthy screen-server. Pushes are best-effort below.
        if let Some(mut prev) = inner.child.take() {
            let _ = prev.kill().await;
        }
        inner.child = Some(child);
        inner.ipc = Some(Arc::clone(&ipc));
        {
            let cfg = self.config_manager.snapshot().await.remote_desktop;
            if let Err(e) = push_paths(&ipc, &cfg).await {
                warn!("screen-server config push after spawn failed: {e:#}");
            }
            if let Err(e) = push_stream_config(&ipc, &cfg).await {
                warn!("screen-server stream-config push after spawn failed: {e:#}");
            }
        }
        Ok(ipc)
    }

    // -- ctl request wrappers ------------------------------------------------

    async fn call(&self, cmd: ctl::Cmd) -> Result<ctl::ResponsePayload> {
        // One transparent retry: the ctl socket can drop mid-request (the
        // screen-server restarted, or a transient local break) even on
        // loopback, and every request wrapper here is idempotent enough to
        // re-send. `ipc()` reconnects (attach or respawn) when the previous
        // client was marked dead by its pump.
        let ipc = self.ipc().await?;
        match ipc.request(cmd.clone()).await {
            Ok(payload) => Ok(payload),
            Err(first) if first.to_string().contains("connection") => {
                warn!(
                    "screen-server ctl request failed ({first:#}); reconnecting and retrying once"
                );
                let ipc = self.ipc().await?;
                ipc.request(cmd).await
            }
            Err(first) => Err(first),
        }
    }

    pub async fn get_paths(&self) -> Result<ctl::Paths> {
        match self.call(ctl::Cmd::GetPaths).await? {
            ctl::ResponsePayload::Paths(paths) => Ok(paths),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected GetPaths response: {other:?}"),
        }
    }

    /// The uinput probe for the webui's Linux input-setup card: whether the
    /// consent-free injection route (`/dev/uinput`) exists and is usable by
    /// the screen-server. The webui shows its one-time setup script when it
    /// is not.
    pub async fn get_input_status(&self) -> Result<ctl::InputStatusInfo> {
        match self.call(ctl::Cmd::GetInputStatus).await? {
            ctl::ResponsePayload::InputStatus(info) => Ok(info),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected GetInputStatus response: {other:?}"),
        }
    }

    /// Forget the persisted portal ScreenCast consent (restore tokens) so
    /// the next stream start pops the source picker again — the webui's
    /// "re-select display" action.
    pub async fn reset_screen_consent(&self) -> Result<u32> {
        match self.call(ctl::Cmd::ResetScreenConsent).await? {
            ctl::ResponsePayload::ScreenConsentReset { cleared } => Ok(cleared),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected ResetScreenConsent response: {other:?}"),
        }
    }

    /// Windows service management for the webui: the SCM status /
    /// (un)install answers. Install and remove go through an elevated
    /// helper (a UAC prompt appears in the user session), so the HTTP
    /// request can block for a while.
    #[cfg(not(target_os = "windows"))]
    pub async fn service_status(&self) -> Result<ctl::ResponsePayload> {
        self.call(ctl::Cmd::ServiceStatus).await
    }

    /// SCM status read directly — no screen-server instance is involved
    /// (the ctl-routed variant would lazily spawn one just to answer a
    /// status poll).
    #[cfg(target_os = "windows")]
    pub async fn service_status(&self) -> Result<ctl::ResponsePayload> {
        let (installed, running) = query_screen_service();
        Ok(ctl::ResponsePayload::Service {
            installed,
            running,
            message: String::new(),
        })
    }

    /// Windows: install the `VTermScreenSvc` service. Any running
    /// vterm-screen-server (detected via the ctl named pipe — our supervised
    /// child or an externally launched one) is stopped FIRST: the service's
    /// worker takes over that pipe when it starts, and Windows allows only
    /// one server on it. With nothing left running to route the ctl
    /// command, the elevated `--install` helper runs from here directly.
    #[cfg(target_os = "windows")]
    pub async fn service_install(&self) -> Result<ctl::ResponsePayload> {
        let enabled = self.config_manager.snapshot().await.remote_desktop.enabled;
        if !enabled {
            bail!("remote desktop is disabled: enable it in Settings → Remote Desktop first");
        }
        self.stop_running_instance_for_install().await?;

        // Bake the persisted tunables into the service command line (the
        // previous ctl-side handler baked its live values; the coordinator
        // re-pushes the full config on the worker's first attach anyway).
        let rd = self.config_manager.snapshot().await.remote_desktop;
        let fps = rd.screen_fps.unwrap_or(DEFAULT_STREAM_FPS);
        let mut args = format!("--install --fps {fps}");
        if let Some(mbps) = rd.screen_bitrate_mbps.filter(|&m| m > 0) {
            args.push_str(&format!(" --bitrate {}", u32::from(mbps) * 1000));
        }
        let exe = screen_server_binary_path().with_context(|| {
            "vterm-screen-server binary not found: build it with `cargo build -p vterm-screen-server` \
             and place it next to vterm-server (or under ~/.vibeterminal/bin/)"
        })?;
        let message = match run_elevated_helper(&exe, &args) {
            Ok(()) => {
                "service installed and started (the worker takes over the ctl endpoint)".to_string()
            }
            Err(e) => e,
        };
        let (installed, running) = query_screen_service();
        Ok(ctl::ResponsePayload::Service {
            installed,
            running,
            message,
        })
    }

    #[cfg(not(target_os = "windows"))]
    pub async fn service_install(&self) -> Result<ctl::ResponsePayload> {
        self.call(ctl::Cmd::ServiceInstall).await
    }

    /// Stop whatever currently serves the ctl pipe, then wait until the
    /// endpoint is really gone. Attach-only probing (never `ipc()`, which
    /// would respawn) so externally launched instances are caught too; the
    /// wait turns an instance that ignored the graceful shutdown into an
    /// actionable install error instead of a broken service later.
    #[cfg(target_os = "windows")]
    async fn stop_running_instance_for_install(&self) -> Result<()> {
        {
            let mut inner = self.inner.lock().await;
            if inner.ipc.as_ref().map_or(true, |ipc| ipc.is_dead()) {
                inner.ipc = None;
                match try_attach(&self.events_tx).await {
                    Ok(ipc) => {
                        info!(
                            "running vterm-screen-server detected (ctl pipe answered); \
                             stopping it before the service install"
                        );
                        inner.ipc = Some(ipc);
                    }
                    Err(_) => return Ok(()), // nothing is serving the pipe
                }
            }
        }
        self.stop().await;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while try_attach(&self.events_tx).await.is_ok() {
            if std::time::Instant::now() >= deadline {
                bail!(
                    "a vterm-screen-server is still serving the ctl endpoint after a shutdown \
                     request; stop it manually and retry the install"
                );
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
        Ok(())
    }

    /// Windows: run the elevated `--remove` helper directly. Routing the
    /// remove through the ctl endpoint (the old flow) is inherently
    /// self-destructive — the helper stops the service's own worker mid-
    /// request, the connection dies, and the retry path would even respawn
    /// a standalone just to run the helper again.
    #[cfg(target_os = "windows")]
    pub async fn service_remove(&self) -> Result<ctl::ResponsePayload> {
        let exe = screen_server_binary_path().with_context(|| {
            "vterm-screen-server binary not found: build it with `cargo build -p vterm-screen-server` \
             and place it next to vterm-server (or under ~/.vibeterminal/bin/)"
        })?;
        let message = match run_elevated_helper(&exe, "--remove") {
            Ok(()) => "service removed".to_string(),
            Err(e) => e,
        };
        // The helper waits out STOP_PENDING before deleting, but a straggler
        // handle (the watchdog winding down) can keep the marked-for-delete
        // entry visible for a moment — poll briefly so the webui's answer
        // reflects the removal, not the transient SCM state.
        let mut state = query_screen_service();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while state.0 && std::time::Instant::now() < deadline {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            state = query_screen_service();
        }
        let (installed, running) = state;
        Ok(ctl::ResponsePayload::Service {
            installed,
            running,
            message,
        })
    }

    #[cfg(not(target_os = "windows"))]
    pub async fn service_remove(&self) -> Result<ctl::ResponsePayload> {
        self.call(ctl::Cmd::ServiceRemove).await
    }

    /// Apply tool-path overrides (spec §4): `Some` sets, `clear_*` resets to
    /// auto-detection. Validation errors come back verbatim.
    pub async fn set_paths(
        &self,
        sdk_path: Option<String>,
        agent_dir: Option<String>,
        clear_sdk: bool,
        clear_agent: bool,
    ) -> Result<ctl::Paths> {
        match self
            .call(ctl::Cmd::SetConfig {
                sdk_path,
                adb_path: None,
                agent_dir,
                clear_sdk,
                clear_adb: true,
                clear_agent,
            })
            .await?
        {
            ctl::ResponsePayload::ConfigApplied { paths } => Ok(paths),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected SetConfig response: {other:?}"),
        }
    }

    /// The encoder availability catalog + effective stream tunables.
    pub async fn get_encoders(&self) -> Result<EncoderCatalog> {
        match self.call(ctl::Cmd::GetEncoders).await? {
            ctl::ResponsePayload::Encoders {
                entries,
                current,
                fps,
                fps_override,
                max_resolution,
                bitrate_kbps,
                virtual_display,
            } => Ok(EncoderCatalog {
                entries,
                current,
                fps,
                fps_override,
                max_resolution,
                bitrate_kbps,
                virtual_display,
            }),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected GetEncoders response: {other:?}"),
        }
    }

    /// Apply the stream tunables on the screen-server; echoes the effective
    /// (encoder id, fps override, max-resolution override, bitrate override
    /// in kbit/s, virtual-display override). Backend/fps/resolution/virtual
    /// display affect streams started after the call; the bitrate also
    /// reaches running streams.
    pub async fn set_stream_config(
        &self,
        encoder: Option<String>,
        fps: Option<u32>,
        max_resolution: Option<String>,
        bitrate_kbps: Option<u32>,
        virtual_display: Option<bool>,
    ) -> Result<(
        String,
        Option<u32>,
        Option<String>,
        Option<u32>,
        Option<bool>,
    )> {
        match self
            .call(ctl::Cmd::SetStreamConfig {
                encoder,
                fps,
                max_resolution,
                bitrate_kbps,
                virtual_display,
            })
            .await?
        {
            ctl::ResponsePayload::StreamConfig {
                encoder,
                fps,
                max_resolution,
                bitrate_kbps,
                virtual_display,
            } => Ok((encoder, fps, max_resolution, bitrate_kbps, virtual_display)),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected SetStreamConfig response: {other:?}"),
        }
    }

    /// Ask the screen-server to bring the desktop input backend up — the
    /// webui "request permission" action. Opens the OS consent dialog
    /// (xdg-desktop-portal RemoteDesktop on Linux, Accessibility on macOS)
    /// for every live desktop stream, or a standby injector when none
    /// runs, so permission can be granted before the first client sends
    /// input. Uses the slow ctl timeout: the answer arrives after the user
    /// interacts with the dialog.
    pub async fn request_input_permission(&self) -> Result<(bool, String)> {
        let ipc = self.ipc().await?;
        let payload = ipc
            .request_with_timeout(ctl::Cmd::WarmupInput, SLOW_REQUEST_TIMEOUT)
            .await?;
        match payload {
            ctl::ResponsePayload::WarmupInput { granted, message } => Ok((granted, message)),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected WarmupInput response: {other:?}"),
        }
    }

    /// Ask the screen-server to grant screen-capture consent interactively
    /// — the webui "request permission" action on Linux. The portal's
    /// ScreenCast source picker pops NOW (not at the next stream start) and
    /// the answered selection is persisted as the generic screen source's
    /// restore token. Uses the slow ctl timeout: the answer arrives after
    /// the user picks a display.
    pub async fn request_screen_consent(&self) -> Result<(bool, String)> {
        let ipc = self.ipc().await?;
        let payload = ipc
            .request_with_timeout(ctl::Cmd::GrantScreenConsent, SLOW_REQUEST_TIMEOUT)
            .await?;
        match payload {
            ctl::ResponsePayload::ScreenConsentGrant { granted, message } => Ok((granted, message)),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected GrantScreenConsent response: {other:?}"),
        }
    }

    pub async fn list_sources(&self) -> Result<Vec<ctl::SourceEntry>> {
        self.list_source_snapshot()
            .await
            .map(|(_, entries)| entries)
    }

    pub async fn list_source_snapshot(&self) -> Result<(u64, Vec<ctl::SourceEntry>)> {
        match self.call(ctl::Cmd::ListSources).await? {
            ctl::ResponsePayload::Sources { entries, revision } => Ok((revision, entries)),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected ListSources response: {other:?}"),
        }
    }

    pub async fn status(&self) -> Result<ctl::Status> {
        match self.call(ctl::Cmd::GetStatus).await? {
            ctl::ResponsePayload::Status(status) => Ok(status),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected GetStatus response: {other:?}"),
        }
    }

    /// Mint a one-time token for one source (spec §6). The client gets the
    /// token + UDP port and connects directly. `client_ip`, when known
    /// (direct streams: the WSS peer address), address-binds the grant so a
    /// sniffed token cannot be replayed from elsewhere within the TTL
    /// (security review §3.2). Relay-requested streams pass `None`.
    pub async fn start_stream(
        &self,
        source: &str,
        client_label: &str,
        client_ip: Option<IpAddr>,
    ) -> Result<StreamStart> {
        let ipc = self.ipc().await?;
        let token: [u8; 16] = rand::random();
        let resp = ipc
            .request(ctl::Cmd::GrantToken {
                token: ctl::token_to_hex(&token),
                sources: vec![source.to_string()],
                client_label: client_label.to_string(),
                ttl_ms: TOKEN_TTL_MS,
                client_addr: client_ip.map(|ip| ip.to_string()),
            })
            .await?;
        match resp {
            ctl::ResponsePayload::GrantResult { granted: true } => Ok(StreamStart {
                udp_port: ipc.hello.udp_port,
                token,
            }),
            ctl::ResponsePayload::GrantResult { .. } => {
                bail!("screen-server refused the token grant")
            }
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected GrantToken response: {other:?}"),
        }
    }

    /// Revoke one stream's token (client closed its window).
    pub async fn stop_stream(&self, token: [u8; 16]) -> Result<()> {
        match self
            .call(ctl::Cmd::RevokeToken {
                token: ctl::token_to_hex(&token),
            })
            .await?
        {
            ctl::ResponsePayload::Ok => Ok(()),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected RevokeToken response: {other:?}"),
        }
    }

    /// Revoke every grant of one client (its WSS connection dropped).
    pub async fn revoke_client(&self, client_label: &str) -> Result<()> {
        match self
            .call(ctl::Cmd::RevokeTokens {
                client_label: client_label.to_string(),
            })
            .await?
        {
            ctl::ResponsePayload::Ok => Ok(()),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected RevokeTokens response: {other:?}"),
        }
    }

    /// Boot an iOS simulator on the screen-server host (macOS only).
    pub async fn boot_simulator(&self, udid: &str) -> Result<()> {
        match self
            .call(ctl::Cmd::BootSimulator {
                udid: udid.to_string(),
            })
            .await?
        {
            ctl::ResponsePayload::Ok => self.wait_for_simulator_state(udid, true).await,
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected BootSimulator response: {other:?}"),
        }
    }

    /// Shut an iOS simulator down on the screen-server host (macOS only).
    pub async fn shutdown_simulator(&self, udid: &str) -> Result<()> {
        match self
            .call(ctl::Cmd::ShutdownSimulator {
                udid: udid.to_string(),
            })
            .await?
        {
            ctl::ResponsePayload::Ok => self.wait_for_simulator_state(udid, false).await,
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected ShutdownSimulator response: {other:?}"),
        }
    }

    async fn wait_for_simulator_state(&self, udid: &str, booted: bool) -> Result<()> {
        let spec = format!("sim:{udid}");
        let deadline = tokio::time::Instant::now() + SIMULATOR_STATE_TIMEOUT;
        let mut last_error = None;
        loop {
            match self.list_sources().await {
                Ok(entries) => {
                    last_error = None;
                    if let Some(source) = entries.iter().find(|source| source.spec == spec) {
                        let reached = if booted {
                            matches!(
                                source.status,
                                ctl::SourceStatus::Available | ctl::SourceStatus::Streaming
                            )
                        } else {
                            matches!(&source.status, ctl::SourceStatus::Unavailable { reason } if reason == "simulator is Shutdown")
                        };
                        if reached {
                            return Ok(());
                        }
                    }
                }
                Err(error) => last_error = Some(format!("{error:#}")),
            }
            if tokio::time::Instant::now() >= deadline {
                let target = if booted { "Booted" } else { "Shutdown" };
                if let Some(error) = last_error {
                    bail!("timed out waiting for simulator {udid} to become {target}: {error}");
                }
                bail!("timed out waiting for simulator {udid} to become {target}");
            }
            tokio::time::sleep(SIMULATOR_STATE_INTERVAL).await;
        }
    }

    /// Launch an Android emulator (AVD) on the screen-server host (any host).
    pub async fn boot_android_emulator(&self, avd: &str) -> Result<Option<u64>> {
        let ipc = self.ipc().await?;
        let payload = ipc
            .request_with_timeout(
                ctl::Cmd::BootAndroidEmulator {
                    avd: avd.to_string(),
                },
                ANDROID_EMULATOR_CTL_TIMEOUT,
            )
            .await?;
        match payload {
            ctl::ResponsePayload::OperationAccepted { operation_id } => Ok(Some(operation_id)),
            ctl::ResponsePayload::Ok => Ok(None),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected BootAndroidEmulator response: {other:?}"),
        }
    }

    /// Shut a running Android emulator down on the screen-server host
    /// (`adb emu kill`; emulator serials only).
    pub async fn shutdown_android_emulator(&self, serial: &str) -> Result<Option<u64>> {
        let ipc = self.ipc().await?;
        let payload = ipc
            .request_with_timeout(
                ctl::Cmd::ShutdownAndroidEmulator {
                    serial: serial.to_string(),
                },
                ANDROID_EMULATOR_CTL_TIMEOUT,
            )
            .await?;
        match payload {
            ctl::ResponsePayload::OperationAccepted { operation_id } => Ok(Some(operation_id)),
            ctl::ResponsePayload::Ok => Ok(None),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected ShutdownAndroidEmulator response: {other:?}"),
        }
    }

    /// Read an iOS simulator's pasteboard (device copy relay). macOS only.
    pub async fn get_simulator_clipboard(&self, udid: &str) -> Result<String> {
        match self
            .call(ctl::Cmd::GetClipboard {
                udid: udid.to_string(),
            })
            .await?
        {
            ctl::ResponsePayload::Clipboard { text } => Ok(text),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected GetClipboard response: {other:?}"),
        }
    }

    /// Rotate an Android device/emulator display by 90° on the screen-server
    /// host (`adb` user_rotation cycle).
    pub async fn rotate_android_device(&self, serial: &str) -> Result<()> {
        match self
            .call(ctl::Cmd::RotateAndroidDevice {
                serial: serial.to_string(),
            })
            .await?
        {
            ctl::ResponsePayload::Ok => Ok(()),
            ctl::ResponsePayload::Error { message } => bail!(message),
            other => bail!("unexpected RotateAndroidDevice response: {other:?}"),
        }
    }
}

/// Push the persisted tool-path config into the connected screen-server
/// (spec §4.2). A `None` field pushes `clear_*` so the screen-server always
/// mirrors the persisted state exactly.
async fn push_paths(ipc: &IpcClient, cfg: &RemoteDesktopConfig) -> Result<()> {
    let resp = ipc
        .request(ctl::Cmd::SetConfig {
            sdk_path: configured_sdk_path(cfg),
            adb_path: cfg.adb_path.clone(),
            agent_dir: cfg.android_studio_path.clone(),
            clear_sdk: cfg.android_sdk_path.is_none() && cfg.adb_path.is_none(),
            clear_adb: cfg.adb_path.is_none(),
            clear_agent: cfg.android_studio_path.is_none(),
        })
        .await?;
    match resp {
        ctl::ResponsePayload::ConfigApplied { .. } => Ok(()),
        ctl::ResponsePayload::Error { message } => Err(anyhow!(message)),
        other => Err(anyhow!("unexpected SetConfig response: {other:?}")),
    }
}

fn configured_sdk_path(cfg: &RemoteDesktopConfig) -> Option<String> {
    cfg.android_sdk_path.clone().or_else(|| {
        let adb = std::path::Path::new(cfg.adb_path.as_deref()?);
        adb.parent()?
            .parent()
            .map(|root| root.to_string_lossy().into_owned())
    })
}

/// Push the persisted stream tunables (encoder backend + frame rate + max
/// resolution + bitrate + virtual display) into the connected screen-server.
/// `None` config values push the defaults ("auto" / 60 fps / "4k" /
/// bitrate-auto / virtual display off) so the screen-server always mirrors
/// the persisted state exactly. Backend/fps/resolution/virtual display apply
/// to streams started after the push; the bitrate also reaches running
/// streams. Non-macOS hosts ignore the virtual-display flag.
async fn push_stream_config(ipc: &IpcClient, cfg: &RemoteDesktopConfig) -> Result<()> {
    let resp = ipc
        .request(ctl::Cmd::SetStreamConfig {
            encoder: Some(
                cfg.screen_encoder
                    .clone()
                    .unwrap_or_else(|| "auto".to_string()),
            ),
            fps: Some(cfg.screen_fps.unwrap_or(DEFAULT_STREAM_FPS)),
            max_resolution: Some(
                cfg.screen_max_resolution
                    .clone()
                    .unwrap_or_else(|| DEFAULT_MAX_RESOLUTION.to_string()),
            ),
            // Some(0) = clear back to auto (area-scaled).
            bitrate_kbps: Some(u32::from(cfg.screen_bitrate_mbps.unwrap_or(0)) * 1000),
            virtual_display: Some(cfg.screen_virtual_display),
        })
        .await?;
    match resp {
        ctl::ResponsePayload::StreamConfig { .. } => Ok(()),
        ctl::ResponsePayload::Error { message } => Err(anyhow!(message)),
        other => Err(anyhow!("unexpected SetStreamConfig response: {other:?}")),
    }
}

/// Try to attach to an already-running screen-server (blocking pipe
/// connect, on a blocking thread).
async fn try_attach(events_tx: &broadcast::Sender<ctl::Event>) -> Result<Arc<IpcClient>> {
    let endpoint = ctl::endpoint();
    let events = events_tx.clone();
    tokio::task::spawn_blocking(move || IpcClient::connect(&endpoint, events))
        .await
        .map_err(|e| anyhow!("connect task join: {e}"))?
}

/// Locate the `vterm-screen-server` binary: next to the current executable
/// first (dev target dir and release layout), then `~/.vibeterminal/bin/`.
fn screen_server_binary_path() -> Option<std::path::PathBuf> {
    let exe_name = if cfg!(windows) {
        "vterm-screen-server.exe"
    } else {
        "vterm-screen-server"
    };
    if let Ok(current) = std::env::current_exe() {
        if let Some(dir) = current.parent() {
            let candidate = dir.join(exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        for base in [home.join(".vibeterminal"), home.join(".vibeterminaldev")] {
            let candidate = base.join("bin").join(exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
