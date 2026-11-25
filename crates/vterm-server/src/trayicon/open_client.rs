//! Tray "Open": bring an already-running client to the foreground, or
//! launch one.
//!
//! The client side of this channel lives in `crates/vterm/src/ipc.rs` and
//! `cli_forward.rs`: every running client owns a PID-named endpoint speaking
//! newline-delimited JSON (see the endpoint scheme there). This module
//! mirrors just enough of that machinery on the server side to
//! 1. discover running client instances of the same build flavor,
//! 2. query them and send an `activate` request — restore/raise their
//!    windows — to the best candidate, and
//! 3. when no instance answers, spawn the sibling `vterm` binary detached
//!    (the client reciprocally spawns us via `vterm/src/local_server.rs`).
//!
//! The wire types below intentionally duplicate `vterm/src/ipc.rs`; both
//! sides handshake on `IPC_PROTOCOL_VERSION`, so a mismatched build fails
//! loudly instead of silently.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Must match `vterm/src/ipc.rs` — the Hello handshake enforces it.
const IPC_PROTOCOL_VERSION: u32 = 1;

/// Per-instance probe budget (connect + hello + query).
const PROBE_TIMEOUT: Duration = Duration::from_millis(700);
/// Budget for one `activate` round trip.
const ACTIVATE_TIMEOUT: Duration = Duration::from_secs(2);
/// Total "Open" budget before falling back to launching a new client.
const OPEN_BUDGET: Duration = Duration::from_secs(3);

/// ── Wire protocol (mirror of `vterm/src/ipc.rs`) ────────────────────

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    Hello { proto: u32 },
    Query,
    Activate,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Reply {
    HelloOk {
        proto: u32,
        pid: u32,
    },
    State {
        connected: bool,
        is_local: bool,
        windows: usize,
    },
    Ok,
    Err {
        reason: String,
    },
}

/// Flavor-scoped endpoint base; dev and production clients never talk to
/// each other (see `vterm/src/ipc.rs::endpoint_base`).
fn endpoint_base() -> String {
    if cfg!(DEVELOPMENT) {
        "vibe-terminal-client-dev"
    } else {
        "vibe-terminal-client"
    }
    .to_string()
}

enum Endpoint {
    /// Windows named pipe, full `\\.\pipe\...` path.
    #[cfg(target_os = "windows")]
    NamedPipe(String),
    /// Linux abstract-namespace unix socket name (without leading NUL).
    #[cfg(target_os = "linux")]
    AbstractUnix(String),
    /// macOS filesystem unix socket path.
    #[cfg(target_os = "macos")]
    UnixPath(PathBuf),
}

fn endpoint_for_pid(pid: u32) -> Endpoint {
    let base = endpoint_base();
    #[cfg(target_os = "windows")]
    {
        Endpoint::NamedPipe(format!(r"\\.\pipe\{base}-{pid}"))
    }
    #[cfg(target_os = "linux")]
    {
        Endpoint::AbstractUnix(format!("{base}-{pid}"))
    }
    #[cfg(target_os = "macos")]
    {
        let dir = dirs::home_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join(if cfg!(DEVELOPMENT) {
                ".vibeterminal-dev"
            } else {
                ".vibeterminal"
            })
            .join("instances");
        Endpoint::UnixPath(dir.join(format!("{base}-{pid}.sock")))
    }
}

/// ── Transport ──────────────────────────────────────────────────────

enum IpcStream {
    #[cfg(target_os = "windows")]
    NamedPipe(tokio::net::windows::named_pipe::NamedPipeClient),
    #[cfg(not(target_os = "windows"))]
    Unix(tokio::net::UnixStream),
}

impl IpcStream {
    async fn connect(pid: u32) -> anyhow::Result<Self> {
        match endpoint_for_pid(pid) {
            #[cfg(target_os = "windows")]
            Endpoint::NamedPipe(name) => {
                use tokio::net::windows::named_pipe::ClientOptions;
                // ERROR_PIPE_BUSY is transient (instance between pipes);
                // retry briefly.
                let mut last_error = None;
                for _ in 0..10 {
                    match ClientOptions::new().open(&name) {
                        Ok(client) => return Ok(IpcStream::NamedPipe(client)),
                        Err(error) if error.raw_os_error() == Some(231) => {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                        Err(error) => {
                            last_error = Some(error);
                            break;
                        }
                    }
                }
                Err(anyhow::anyhow!(
                    "connect {}: {:?}",
                    name,
                    last_error.unwrap()
                ))
            }
            #[cfg(target_os = "linux")]
            Endpoint::AbstractUnix(name) => {
                // tokio's `connect` treats a leading NUL in the path as a
                // Linux abstract-namespace name.
                Ok(IpcStream::Unix(
                    tokio::net::UnixStream::connect(format!("\0{name}")).await?,
                ))
            }
            #[cfg(target_os = "macos")]
            Endpoint::UnixPath(path) => Ok(IpcStream::Unix(
                tokio::net::UnixStream::connect(&path).await?,
            )),
        }
    }

    /// One JSON-lines request → reply round trip.
    async fn request(&mut self, request: &Request) -> anyhow::Result<Reply> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let mut line = serde_json::to_string(request)?;
        line.push('\n');
        self.write_all(line.as_bytes()).await?;
        self.flush().await?;
        let mut reader = BufReader::new(&mut *self);
        let mut reply = String::new();
        reader.read_line(&mut reply).await?;
        if reply.is_empty() {
            anyhow::bail!("connection closed");
        }
        Ok(serde_json::from_str(reply.trim_end())?)
    }
}

impl tokio::io::AsyncRead for IpcStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self {
            #[cfg(target_os = "windows")]
            IpcStream::NamedPipe(client) => std::pin::Pin::new(client).poll_read(cx, buf),
            #[cfg(not(target_os = "windows"))]
            IpcStream::Unix(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl tokio::io::AsyncWrite for IpcStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match &mut *self {
            #[cfg(target_os = "windows")]
            IpcStream::NamedPipe(client) => std::pin::Pin::new(client).poll_write(cx, buf),
            #[cfg(not(target_os = "windows"))]
            IpcStream::Unix(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
        }
    }
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self {
            #[cfg(target_os = "windows")]
            IpcStream::NamedPipe(client) => std::pin::Pin::new(client).poll_flush(cx),
            #[cfg(not(target_os = "windows"))]
            IpcStream::Unix(stream) => std::pin::Pin::new(stream).poll_flush(cx),
        }
    }
    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self {
            #[cfg(target_os = "windows")]
            IpcStream::NamedPipe(client) => std::pin::Pin::new(client).poll_shutdown(cx),
            #[cfg(not(target_os = "windows"))]
            IpcStream::Unix(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
        }
    }
}

/// ── Instance discovery / activation ────────────────────────────────

#[derive(Debug, Clone, Copy)]
struct InstanceState {
    connected: bool,
    is_local: bool,
    windows: usize,
}

/// Connect, handshake, and query one instance. `None` on any failure — a
/// connect failure is also the definitive liveness check.
async fn probe_instance(pid: u32) -> Option<InstanceState> {
    let result = tokio::time::timeout(PROBE_TIMEOUT, async {
        let mut stream = IpcStream::connect(pid).await?;
        match stream
            .request(&Request::Hello {
                proto: IPC_PROTOCOL_VERSION,
            })
            .await?
        {
            Reply::HelloOk {
                pid: server_pid, ..
            } if server_pid == pid => {}
            _ => anyhow::bail!("unexpected handshake reply"),
        }
        match stream.request(&Request::Query).await? {
            Reply::State {
                connected,
                is_local,
                windows,
            } => Ok(InstanceState {
                connected,
                is_local,
                windows,
            }),
            _ => anyhow::bail!("unexpected query reply"),
        }
    })
    .await;
    match result {
        Ok(Ok(state)) => Some(state),
        Ok(Err(error)) => {
            tracing::debug!(pid, %error, "Tray Open: probe failed");
            None
        }
        Err(_) => {
            tracing::debug!(pid, "Tray Open: probe timed out");
            None
        }
    }
}

/// Ask instance `pid` to bring its windows to the foreground. `Some(true)`
/// = accepted.
async fn activate_instance(pid: u32) -> Option<bool> {
    let result = tokio::time::timeout(ACTIVATE_TIMEOUT, async {
        let mut stream = IpcStream::connect(pid).await?;
        match stream
            .request(&Request::Hello {
                proto: IPC_PROTOCOL_VERSION,
            })
            .await?
        {
            Reply::HelloOk {
                pid: server_pid, ..
            } if server_pid == pid => {}
            _ => anyhow::bail!("unexpected handshake reply"),
        }
        match stream.request(&Request::Activate).await? {
            Reply::Ok => Ok(true),
            Reply::Err { reason } => {
                tracing::debug!(pid, %reason, "Tray Open: activate rejected");
                Ok(false)
            }
            _ => anyhow::bail!("unexpected activate reply"),
        }
    })
    .await;
    match result {
        Ok(Ok(accepted)) => Some(accepted),
        Ok(Err(error)) => {
            tracing::debug!(pid, %error, "Tray Open: activate failed");
            None
        }
        Err(_) => {
            tracing::debug!(pid, "Tray Open: activate timed out");
            None
        }
    }
}

/// Probe every running instance and activate the best candidate. Instances
/// on the local server are preferred (most likely the user's main window —
/// same heuristic as the client's CLI forwarding); any live instance is a
/// valid target. Returns the activated (or merely alive) pid, or `None`
/// when no instance exists at all.
fn activate_running_instance() -> Option<u32> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;
    runtime.block_on(async {
        let deadline = std::time::Instant::now() + OPEN_BUDGET;

        let mut pids = enumerate_client_pids();
        pids.sort_unstable();
        tracing::debug!(candidates = ?pids, "Tray Open: probing running clients");

        let mut locals: Vec<(u32, usize)> = Vec::new(); // (pid, windows)
        let mut others: Vec<u32> = Vec::new();
        for pid in pids {
            if std::time::Instant::now() > deadline {
                break;
            }
            match probe_instance(pid).await {
                Some(state) if state.connected && state.is_local => {
                    locals.push((pid, state.windows));
                }
                Some(_) => others.push(pid),
                None => {}
            }
        }

        // Prefer the instance with the most windows (most likely the one
        // the user is looking at), lowest pid as a stable tie-break.
        locals.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let mut candidates: Vec<u32> = locals.into_iter().map(|(pid, _)| pid).collect();
        candidates.extend(others);
        let first_live = candidates.first().copied();

        for pid in candidates {
            if std::time::Instant::now() > deadline {
                break;
            }
            if activate_instance(pid).await == Some(true) {
                return Some(pid);
            }
        }
        // A live instance exists but none accepted (mixed versions, stuck
        // UI). Report it anyway: launching a duplicate client would be worse
        // than leaving the first unfocused.
        first_live
    })
}

/// Client binary name for this build flavor (`vterm` / `vterm-dev`).
fn client_exe_name() -> &'static str {
    if cfg!(DEVELOPMENT) {
        "vterm-dev"
    } else {
        "vterm"
    }
}

/// ── Instance discovery (process enumeration) ───────────────────────
///
/// Endpoints are named by PID, so discovery = "which PIDs run the client
/// binary?". Matching is by executable file name, which also separates
/// build flavors; mismatches are harmless — connecting to a same-named
/// foreign process fails instantly and is skipped. (The server's own exe
/// is vterm-server[-dev] and never matches.)

#[cfg(target_os = "windows")]
fn enumerate_client_pids() -> Vec<u32> {
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    let own_name = format!("{}.exe", client_exe_name()).to_lowercase();
    let mut pids = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == std::ptr::null_mut() {
            return pids;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let exe = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();
                if exe == own_name {
                    pids.push(entry.th32ProcessID);
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        windows_sys::Win32::Foundation::CloseHandle(snapshot);
    }
    pids
}

#[cfg(target_os = "linux")]
fn enumerate_client_pids() -> Vec<u32> {
    let own_name = client_exe_name();
    let mut pids = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return pids;
    };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        // `comm` is the kernel's 15-char executable name — enough for
        // "vterm"/"vterm-dev".
        let Ok(comm) = std::fs::read_to_string(format!("/proc/{pid}/comm")) else {
            continue;
        };
        if comm.trim() == own_name {
            pids.push(pid);
        }
    }
    pids
}

#[cfg(target_os = "macos")]
fn enumerate_client_pids() -> Vec<u32> {
    let own_name = client_exe_name();
    let mut pids = Vec::new();
    unsafe {
        // proc_listallpids + proc_name: proc_name returns the executable
        // name (MAXCOMLEN=16, fits "vterm"/"vterm-dev").
        let mut size = libc::proc_listallpids(std::ptr::null_mut(), 0);
        if size <= 0 {
            return pids;
        }
        // Room for processes started between the two calls.
        size += size / 4 + 8;
        let mut buffer = vec![0u32; size as usize / std::mem::size_of::<u32>()];
        let written = libc::proc_listallpids(
            buffer.as_mut_ptr().cast(),
            (buffer.len() * std::mem::size_of::<u32>()) as libc::c_int,
        );
        if written <= 0 {
            return pids;
        }
        for &pid in &buffer[..(written as usize / std::mem::size_of::<u32>()).min(buffer.len())] {
            if pid == 0 {
                continue;
            }
            let mut comm = [0u8; 64];
            let len = libc::proc_name(
                pid as libc::c_int,
                comm.as_mut_ptr().cast(),
                comm.len() as u32,
            );
            if len > 0 && &comm[..len as usize] == own_name.as_bytes() {
                pids.push(pid);
            }
        }
    }
    pids
}

/// ── Client launch fallback ─────────────────────────────────────────

/// Locate the `vterm` binary, mirroring the lookup the client uses for us
/// (see `vterm/src/local_server.rs::server_binary_path`):
///
/// 1. Sibling of the running server executable — the shared workspace
///    `target/<profile>/` in dev, the install directory in packaged builds.
/// 2. Reverse of the packaged macOS layout: the server lives inside the
///    main app bundle (`Contents/Library/AURGA Vibe Terminal.app`), so the
///    client is in the enclosing app's `Contents/MacOS/`.
/// 3. `$HOME/.vibeterminal[-dev]/bin/` for deployments that ship the
///    client separately.
fn client_binary_path() -> Option<PathBuf> {
    fn candidate(dir: &Path) -> Option<PathBuf> {
        let mut path = dir.join(client_exe_name());
        if cfg!(target_os = "windows") {
            path.set_extension("exe");
        }
        path.is_file().then_some(path)
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(path) = candidate(parent) {
                return Some(path);
            }
            // VibeTerminal.app/Contents/Library/AURGA Vibe Terminal.app/
            // Contents/MacOS/vterm-server -> the enclosing app's
            // Contents/MacOS (4 levels up from the server's MacOS dir).
            #[cfg(target_os = "macos")]
            if !cfg!(DEVELOPMENT) {
                if let Some(outer_contents) = parent.ancestors().nth(4) {
                    if let Some(path) = candidate(&outer_contents.join("MacOS")) {
                        return Some(path);
                    }
                }
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let dir = if cfg!(DEVELOPMENT) {
            ".vibeterminal-dev"
        } else {
            ".vibeterminal"
        };
        if let Some(path) = candidate(&home.join(dir).join("bin")) {
            return Some(path);
        }
    }

    None
}

/// Spawn the client detached: it must survive the server and never receive
/// the server's console Ctrl+C. No launch flag is passed: a launch without a
/// path argument opens just the client, which attaches to the sessions this
/// server already hosts (and only creates a default terminal when the local
/// server has none of its own). The cwd is set to the user's home as a sane
/// neutral default.
fn spawn_client(binary: &Path) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // A plain spawn inherits this process's token. When the server runs
        // elevated (e.g. after the TF-flash "restart as administrator" —
        // it stays elevated until logout), the client would be elevated too,
        // and shell integration from an elevated client is flaky (UIPI):
        // "Reveal in File Explorer" silently does nothing and VS Code starts
        // as administrator. Launch the client with the interactive shell's
        // token instead, so it always gets the logged-in user's normal
        // privileges.
        if crate::config::process_is_elevated() {
            match spawn_client_as_shell_user(binary) {
                Ok(()) => {
                    info!(
                        binary = %binary.display(),
                        "Tray Open: launched client de-elevated (server is elevated)"
                    );
                    return Ok(());
                }
                Err(error) => warn!(
                    "Tray Open: de-elevated launch failed ({error}); falling back to a plain spawn"
                ),
            }
        }
    }

    let mut command = std::process::Command::new(binary);
    if let Some(home) = dirs::home_dir() {
        command.current_dir(home);
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        if !cfg!(DEVELOPMENT) {
            command
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Detach from the server's process group so signals aimed at it
        // (terminal hangup) cannot take the client down.
        command.process_group(0);
        if !cfg!(DEVELOPMENT) {
            command
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
        }
    }
    let mut child = command.spawn()?;
    // `Child`'s Drop never waits: the server is long-lived and launches a
    // client per tray "Open", so without an explicit reaper each exited
    // client would linger as a <defunct> zombie until the server dies.
    if let Err(err) = std::thread::Builder::new()
        .name("client-exit-reaper".to_string())
        .spawn(move || {
            let _ = child.wait();
        })
    {
        warn!("Tray Open: failed to spawn client reaper thread: {err}");
    }
    Ok(())
}

/// ── De-elevated client launch (Windows) ────────────────────────────

/// PID of the interactive shell (explorer.exe): its primary token carries
/// the logged-in user's medium-IL identity, which is exactly what the
/// client should run with.
#[cfg(target_os = "windows")]
fn shell_process_id() -> Option<u32> {
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_null() {
            return None;
        }
        let mut result = None;
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let exe = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();
                if exe == "explorer.exe" {
                    result = Some(entry.th32ProcessID);
                    break;
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        windows_sys::Win32::Foundation::CloseHandle(snapshot);
        result
    }
}

/// Launch the client with a token duplicated from explorer.exe, i.e. with
/// the logged-in user's normal (medium-IL) privileges, on the interactive
/// desktop. Used when the server itself is elevated so the client — and
/// everything it spawns (Explorer reveals, VS Code, terminals) — behaves
/// like a manually started instance. Requires the caller to be elevated
/// (SeImpersonatePrivilege), which is exactly the case this path covers.
#[cfg(target_os = "windows")]
fn spawn_client_as_shell_user(binary: &Path) -> anyhow::Result<()> {
    use anyhow::anyhow;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        DuplicateTokenEx, SecurityImpersonation, TokenPrimary, TOKEN_ALL_ACCESS, TOKEN_DUPLICATE,
        TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, OpenProcessToken, PROCESS_QUERY_INFORMATION,
    };

    let shell_pid = shell_process_id()
        .ok_or_else(|| anyhow!("explorer.exe not found (no interactive shell?)"))?;

    unsafe {
        let shell_process = OpenProcess(PROCESS_QUERY_INFORMATION, 0, shell_pid);
        if shell_process.is_null() {
            return Err(anyhow!(
                "OpenProcess(explorer.exe, {shell_pid}) failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        let mut shell_token: HANDLE = std::ptr::null_mut();
        let opened = OpenProcessToken(
            shell_process,
            TOKEN_DUPLICATE | TOKEN_QUERY,
            &mut shell_token,
        );
        CloseHandle(shell_process);
        if opened == 0 {
            return Err(anyhow!(
                "OpenProcessToken(explorer.exe) failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        let mut primary_token: HANDLE = std::ptr::null_mut();
        let duplicated = DuplicateTokenEx(
            shell_token,
            TOKEN_ALL_ACCESS,
            std::ptr::null(),
            SecurityImpersonation,
            TokenPrimary,
            &mut primary_token,
        );
        CloseHandle(shell_token);
        if duplicated == 0 {
            return Err(anyhow!(
                "DuplicateTokenEx failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        let result = create_process_with_token(primary_token, binary);
        CloseHandle(primary_token);
        result
    }
}

/// `CreateProcessWithTokenW` wrapper: quoted command line (the API does no
/// path resolution for quoted exes), home cwd, no console — the process
/// lands on the token's interactive session/desktop.
#[cfg(target_os = "windows")]
unsafe fn create_process_with_token(
    token: windows_sys::Win32::Foundation::HANDLE,
    binary: &Path,
) -> anyhow::Result<()> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        CreateProcessWithTokenW, LOGON_WITH_PROFILE, PROCESS_INFORMATION, STARTUPINFOW,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let mut command_line = wide(&format!("\"{}\"", binary.display()));
    let cwd = dirs::home_dir().map(|home| wide(&home.to_string_lossy()));
    let mut startup_info: STARTUPINFOW = std::mem::zeroed();
    startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    let mut process_info: PROCESS_INFORMATION = std::mem::zeroed();

    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

    let created = CreateProcessWithTokenW(
        token,
        LOGON_WITH_PROFILE,
        std::ptr::null(),
        command_line.as_mut_ptr(),
        DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
        std::ptr::null(),
        cwd.as_ref().map_or(std::ptr::null(), |cwd| cwd.as_ptr()),
        &startup_info,
        &mut process_info,
    );
    if created == 0 {
        return Err(anyhow::anyhow!(
            "CreateProcessWithTokenW failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    CloseHandle(process_info.hProcess);
    CloseHandle(process_info.hThread);
    Ok(())
}

/// ── Entry point ────────────────────────────────────────────────────

/// Tray "Open" action: activate a running client, else launch one. Runs
/// everything on a background thread — tray callbacks execute on message/UI
/// threads that must never block on sockets.
pub fn open_client() {
    let spawned = std::thread::Builder::new()
        .name("tray-open-client".to_string())
        .spawn(attempt_open)
        .is_ok();
    if !spawned {
        warn!("Tray Open: failed to spawn worker thread");
    }
}

fn attempt_open() {
    // The user just acted on this process's tray, so we currently hold the
    // Windows foreground right. Hand it to any process so the activated
    // client's SetForegroundWindow succeeds (background processes are
    // normally forbidden from stealing focus). ASFW_ANY = u32::MAX.
    #[cfg(target_os = "windows")]
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(u32::MAX);
    }

    match activate_running_instance() {
        Some(pid) => {
            info!(
                pid,
                "Tray Open: brought the running client to the foreground"
            );
        }
        None => match client_binary_path() {
            Some(binary) => match spawn_client(&binary) {
                Ok(()) => info!(binary = %binary.display(), "Tray Open: launched client"),
                Err(error) => {
                    warn!(
                        "Tray Open: failed to launch {}: {}",
                        binary.display(),
                        error
                    )
                }
            },
            None => warn!(
                "Tray Open: no running client and no {} binary found next to the server",
                client_exe_name()
            ),
        },
    }
}
