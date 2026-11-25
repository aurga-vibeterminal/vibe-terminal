use rustls::ServerConfig;
use socket2::{Domain, Protocol, Socket, Type};
use std::io::BufReader;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_rustls::TlsAcceptor as RustlsTlsAcceptor;
use tracing::{error, info, trace, warn};

mod addon;
mod autostart;
mod cli;
mod config;
mod connection;
mod ddns_client;
mod debug;
mod desktop_env;
mod diagnostics;
mod features;
mod fs_watcher;
mod http_auth;
mod local_network;
mod mcp;
mod notification;
mod port_forward;
mod private_ca;
mod relay;
mod relay_udp;
mod screen;
mod session;
mod session_unlock;
/// Linux-only IPC endpoint serving one-time settings tickets to the
/// `vterm-server --open` CLI (see `settings_ipc.rs`).
#[cfg(target_os = "linux")]
mod settings_ipc;
mod static_http;
mod trayicon;
mod volume_watch;

mod voice;
use addon::AddonHost;
use config::{ConfigHttpServer, ConfigManager};
use connection::{Connection, ConnectionRegistry};
use ddns_client::DdnsManager;
use fs_watcher::FsWatcherManager;
use notification::{NotificationManager, NotificationPayload};
use port_forward::PortForwardManager;
use relay::RelayManager;
use session::{
    RecentProjectScanState, SessionManager, TerminalObserverEvent, TerminalObserverManager,
};
use static_http::StaticHttpManager;
use vterm_core::protocol::Message;

/// Environment variable for certificate file path
const CERT_FILE_ENV: &str = "TLS_CERT_FILE";
/// Environment variable for private key file path
const KEY_FILE_ENV: &str = "TLS_KEY_FILE";

const DEFAULT_LISTEN_BACKLOG: i32 = 1024;
/// Slowloris guard (security review §1.1): a peer must finish the TLS
/// handshake this fast after the TCP connection was accepted.
const TLS_ACCEPT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

// Every inbound TLS listener on every platform uses rustls. native-tls was
// previously kept for Windows (SChannel + a managed PFX identity), but the
// rustls clients could not interoperate with it reliably, and native-tls
// delegates to Security.framework on macOS (Keychain prompts) and links the
// system OpenSSL on Linux, which breaks older distros that only ship OpenSSL
// 1.1 (e.g. Ubuntu 18.04). Managed identities are PEM pairs generated
// in-process via vterm_common (rcgen/ring).
pub(crate) type ServerTlsAcceptor = RustlsTlsAcceptor;

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct GlibcMallocTuning {
    arena_max: std::os::raw::c_int,
    mmap_threshold: std::os::raw::c_int,
    trim_threshold: std::os::raw::c_int,
}

/// Load the rustls acceptor from the first usable identity: the
/// TLS_CERT_FILE / TLS_KEY_FILE env pair, then server.cert_path /
/// server.key_path from the config, and otherwise the managed identity under
/// `~/.vibeterminal/certs/`, which is loaded when present and valid and
/// regenerated (rcgen, in-process) when missing or near expiry. The
/// cwd-relative `server.crt`/`server.key` pair is deliberately NOT consulted
/// — it was the old "embedded" dev identity and shadowed the managed one.
fn load_rustls_tls_acceptor(
    config_cert_path: &str,
    config_key_path: &str,
    extra_sans: &[String],
) -> anyhow::Result<(RustlsTlsAcceptor, Arc<private_ca::PrivateCaManager>)> {
    let private_ca = Arc::new(private_ca::PrivateCaManager::new(
        &private_ca::managed_tls_directory()?,
    ));
    let candidates = [
        resolve_tls_pair_from_env(),
        resolve_tls_pair_from_config(config_cert_path, config_key_path),
    ];

    for candidate in candidates {
        let Some((cert, key)) = candidate else {
            continue;
        };
        match load_rustls_pkcs8_tls_acceptor(&cert, &key) {
            Ok(acceptor) => return Ok((acceptor, private_ca)),
            Err(error) => warn!(
                "Failed to load rustls TLS files {} and {}: {}. Trying next TLS identity.",
                cert, key, error
            ),
        }
    }

    info!("Loading managed CA-signed TLS identity");
    private_ca.ensure_certificate(extra_sans)?;
    Ok((private_ca.tls_acceptor()?, private_ca))
}

fn load_rustls_pkcs8_tls_acceptor(
    cert_path: &str,
    key_path: &str,
) -> anyhow::Result<RustlsTlsAcceptor> {
    let cert_file = std::fs::File::open(cert_path).map_err(|error| {
        anyhow::anyhow!("Failed to open TLS certificate {}: {}", cert_path, error)
    })?;
    let key_file = std::fs::File::open(key_path).map_err(|error| {
        anyhow::anyhow!("Failed to open TLS private key {}: {}", key_path, error)
    })?;
    let certs = rustls_pemfile::certs(&mut BufReader::new(cert_file))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            anyhow::anyhow!("Failed to parse TLS certificate {}: {}", cert_path, error)
        })?;
    let key = rustls_pemfile::private_key(&mut BufReader::new(key_file))
        .map_err(|error| {
            anyhow::anyhow!("Failed to parse TLS private key {}: {}", key_path, error)
        })?
        .ok_or_else(|| anyhow::anyhow!("No private key found in {}", key_path))?;
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|error| anyhow::anyhow!("Failed to build rustls server config: {}", error))?;
    Ok(RustlsTlsAcceptor::from(Arc::new(config)))
}

fn resolve_tls_pair_from_env() -> Option<(String, String)> {
    resolve_tls_pair(
        std::env::var(CERT_FILE_ENV)
            .ok()
            .and_then(|path| normalize_tls_path(&path)),
        std::env::var(KEY_FILE_ENV)
            .ok()
            .and_then(|path| normalize_tls_path(&path)),
        &format!("{} and {}", CERT_FILE_ENV, KEY_FILE_ENV),
    )
}

fn resolve_tls_pair_from_config(
    config_cert_path: &str,
    config_key_path: &str,
) -> Option<(String, String)> {
    resolve_tls_pair(
        normalize_tls_path(config_cert_path),
        normalize_tls_path(config_key_path),
        "server.cert_path/server.key_path",
    )
}

fn resolve_tls_pair(
    cert_path: Option<String>,
    key_path: Option<String>,
    source: &str,
) -> Option<(String, String)> {
    match (cert_path, key_path) {
        (Some(cert_path), Some(key_path)) => Some((cert_path, key_path)),
        (None, None) => None,
        _ => {
            warn!(
                "Ignoring TLS configuration from {} because both certificate and private key paths are required.",
                source
            );
            None
        }
    }
}

fn normalize_tls_path(path: &str) -> Option<String> {
    let normalized = path.trim();
    if normalized.is_empty() {
        return None;
    }

    Some(normalized.to_string())
}

struct ServerListeners {
    listeners: Vec<TcpListener>,
    addrs: Vec<SocketAddr>,
}

impl ServerListeners {
    async fn accept(&self) -> std::io::Result<(TcpStream, SocketAddr)> {
        match self.listeners.as_slice() {
            [listener] => listener.accept().await,
            [first, second] => {
                tokio::select! {
                    result = first.accept() => result,
                    result = second.accept() => result,
                }
            }
            _ => unreachable!("server listeners must contain one or two listeners"),
        }
    }

    fn addrs(&self) -> &[SocketAddr] {
        &self.addrs
    }
}

fn bind_server_listeners(port: u16) -> anyhow::Result<ServerListeners> {
    let ipv6_addr = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0));
    let ipv4_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port));

    let mut listeners = Vec::new();
    let mut addrs = Vec::new();
    let mut bind_errors = Vec::new();

    for (addr, only_v6) in [(ipv6_addr, true), (ipv4_addr, false)] {
        match bind_listener(addr, only_v6) {
            Ok(listener) => {
                addrs.push(addr);
                listeners.push(listener);
            }
            Err(error) => {
                warn!("Failed to bind {}: {}", addr, error);
                bind_errors.push(format!("{} ({})", addr, error));
            }
        }
    }

    if listeners.is_empty() {
        anyhow::bail!(
            "Failed to bind terminal relay server listener: {}",
            bind_errors.join("; ")
        );
    }

    Ok(ServerListeners { listeners, addrs })
}

fn bind_listener(addr: SocketAddr, only_v6: bool) -> anyhow::Result<TcpListener> {
    let domain = match addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;

    if matches!(addr, SocketAddr::V6(_)) {
        socket.set_only_v6(only_v6)?;
    }

    socket.set_nonblocking(true)?;
    // See vterm-asr-server: SO_REUSEADDR on Windows permits duplicate binds of
    // the same port by another process; only enable it on Unix (TIME_WAIT).
    #[cfg(not(windows))]
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    socket.listen(DEFAULT_LISTEN_BACKLOG)?;

    let listener = std::net::TcpListener::from(socket);
    Ok(TcpListener::from_std(listener)?)
}

/// Attach to the parent process's console (Windows release/GUI-subsystem
/// builds only). Console-subsystem builds inherit the console already;
/// non-Windows builds always have one from the launching shell.
/// Returns true when a parent console was attached.
fn adopt_parent_console() -> bool {
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    unsafe {
        return windows_sys::Win32::System::Console::AttachConsole(
            windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS,
        ) != 0;
    }
    #[allow(unreachable_code)]
    false
}

/// Allocate a hidden console for the daemon (Windows release builds only).
/// GUI-subsystem processes launched from Explorer / the Run key / the client
/// have no console, so every console-subsystem descendant — git, cmd, bash,
/// and anything those spawn in turn — would allocate a fresh console window
/// that flashes on screen. One hidden console is inherited by the whole
/// process tree instead.
#[cfg(all(target_os = "windows", not(debug_assertions)))]
fn allocate_hidden_console() {
    use windows_sys::Win32::System::Console::{AllocConsole, GetConsoleWindow};
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
    unsafe {
        if AllocConsole() != 0 {
            let hwnd = GetConsoleWindow();
            if !hwnd.is_null() {
                ShowWindow(hwnd, SW_HIDE);
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    if let Some(result) = local_network::run_recovery_process_if_requested() {
        return result;
    }

    // Release Windows builds are GUI-subsystem (no console): adopt the
    // parent console when there is one so headless CLI output
    // (--show-secret-key, --share, …) and the tracing mirror stay visible
    // in a shell. Launches from Explorer / the Run key have no parent
    // console; the call is a no-op there and std silently discards writes
    // to the null standard handles.
    let _console_adopted = adopt_parent_console();

    // Headless commands (--show-secret-key, --set-secret-key, --share, …)
    // run before logging and the instance lock so they stay quiet and work
    // even while a server instance is already running.
    let cli = <cli::Cli as clap::Parser>::parse();
    if let Some(result) = cli::run(&cli) {
        return result;
    }

    // Daemon mode with no parent console: give the process tree a hidden
    // console so console-subsystem descendants never flash a window.
    // Skipped when adopted (a shell owns that console and its lifetime).
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    if !_console_adopted {
        allocate_hidden_console();
    }

    #[cfg(target_os = "linux")]
    let malloc_tuning = tune_glibc_malloc_for_large_transient_buffers();

    // Tracing init precedes ConfigManager::load(), so the file-logging
    // setting is probed straight from the config file on disk.
    let (log_dir, tracing_log) = diagnostics::init(config::log_to_file_enabled());
    info!(
        pid = std::process::id(),
        version = env!("CARGO_PKG_VERSION"),
        commit = env!("VTERM_COMMIT_HASH"),
        commit_time = env!("VTERM_COMMIT_TIME"),
        ?log_dir,
        ?tracing_log,
        "Vibe Terminal Server process started"
    );
    #[cfg(target_os = "windows")]
    match vterm_common::windows_shutdown::set_process_shutdown_last() {
        Ok(()) => info!("Windows shutdown priority set to the last application range"),
        Err(error) => warn!("Failed to defer Windows shutdown notification: {error}"),
    }
    #[cfg(target_os = "linux")]
    info!(
        arena_max = malloc_tuning.arena_max,
        mmap_threshold = malloc_tuning.mmap_threshold,
        trim_threshold = malloc_tuning.trim_threshold,
        "glibc malloc tuned for large transient buffers"
    );

    let (config_port_tx, config_port_rx) = std::sync::mpsc::channel::<u16>();

    #[cfg(feature = "ui")]
    let voice_state = Arc::new(std::sync::RwLock::new(trayicon::TrayVoiceState::default()));
    #[cfg(feature = "ui")]
    let (action_tx, action_rx) = std::sync::mpsc::channel::<trayicon::TrayAction>();

    #[cfg(feature = "ui")]
    let (server_voice_state, server_action_rx) = (Some(voice_state.clone()), Some(action_rx));
    #[cfg(not(feature = "ui"))]
    let (server_voice_state, server_action_rx) = (None, None);

    // First start (no config file yet)? ConfigManager::load() creates the
    // file, so this must be checked before the server thread runs. The
    // non-headless startup then opens the WebUI on the help page directly
    // instead of waiting for the user to find the tray icon.
    #[cfg(feature = "ui")]
    let first_run = !config::config_file_exists();

    std::thread::spawn(move || {
        // Enforce single-instance semantics per build flavor (dev and
        // production builds use different lock names, ports, and config
        // dirs, so one of each can run side by side). A duplicate is
        // reported by `Ok(None)` — `Err` only for genuine lock-creation
        // failures. A spawner that health-probes before launching us — e.g.
        // vterm's startup path handling — races only while the first
        // instance is still binding; reaching this branch then exits cleanly
        // (code 0) instead of surfacing a confusing bind error, and the
        // first instance keeps serving. The lock guard lives for the whole
        // process.
        let instance_name = singleton_instance_name();
        let _instance_lock = match acquire_instance_lock(&instance_name) {
            Ok(Some(guard)) => guard,
            Ok(None) => {
                error!(
                    "Another vterm-server instance is already running (lock={}); exiting.",
                    instance_name
                );
                diagnostics::flush_and_exit(0);
            }
            Err(error) => {
                error!(
                    "Failed to acquire the single-instance lock {}: {}",
                    instance_name, error
                );
                diagnostics::flush_and_exit(1);
            }
        };
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        let outcome = rt.block_on(run_server_inner(
            Some(config_port_tx),
            server_voice_state,
            server_action_rx,
        ));

        // Bound the wait for unrelated spawn_blocking work during shutdown.
        info!(
            pid = std::process::id(),
            "Stopping remaining local server runtime tasks"
        );
        rt.shutdown_timeout(std::time::Duration::from_secs(5));
        info!(
            pid = std::process::id(),
            "Local server runtime stopped; all in-process listeners are closed"
        );

        if let Err(error) = outcome {
            error!("Server error: {}", error);
            diagnostics::flush_and_exit(1);
        }
    });

    #[cfg(feature = "ui")]
    let config_port = match config_port_rx.recv() {
        Ok(config_port) => config_port,
        Err(_) => {
            // The server thread already logged why it exited (duplicate
            // instance, bind failure, panic) and is exiting the process;
            // never panic here on top of it.
            diagnostics::flush_and_exit(1);
        }
    };

    #[cfg(not(feature = "ui"))]
    let _config_port = match config_port_rx.recv() {
        Ok(config_port) => config_port,
        Err(_) => diagnostics::flush_and_exit(1),
    };

    #[cfg(feature = "ui")]
    if first_run {
        let lang = system_language_tag()
            .map(|tag| format!("&lang={tag}"))
            .unwrap_or_default();
        let ticket = crate::http_auth::issue_settings_ticket("local-first-run");
        let url = format!("http://localhost:{config_port}/?section=help{lang}&ticket={ticket}");
        match open::that(&url) {
            Ok(()) => info!("First start: opened the Web UI help page on port {config_port}"),
            Err(error) => {
                warn!("First start: failed to open the Web UI on port {config_port}: {error}")
            }
        }
    }

    #[cfg(feature = "ui")]
    run_ui_app(config_port, voice_state, action_tx);

    #[cfg(not(feature = "ui"))]
    {
        info!("Headless build: no UI/tray; server runs on its background thread");
        loop {
            std::thread::park();
        }
    }

    Ok(())
}

fn singleton_instance_name() -> String {
    // single-instance uses this string as a named mutex on Windows and an
    // abstract unix socket on Linux, but as a FILE PATH on macOS — resolve an
    // absolute path under the flavor's config dir there so the lock never
    // depends on the server's current working directory.
    #[cfg(target_os = "macos")]
    {
        let dir = dirs::home_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join(if cfg!(DEVELOPMENT) {
                ".vibeterminal-dev"
            } else {
                ".vibeterminal"
            });
        let _ = std::fs::create_dir_all(&dir);
        dir.join("vterm-server.lock").to_string_lossy().into_owned()
    }
    #[cfg(not(target_os = "macos"))]
    {
        if cfg!(DEVELOPMENT) {
            "vibe-terminal-vterm-server-dev".to_string()
        } else {
            "vibe-terminal-vterm-server".to_string()
        }
    }
}

/// Guard holding the single-instance lock; lives for the whole process.
#[cfg(target_os = "linux")]
struct InstanceLockGuard {
    fd: i32,
}

#[cfg(target_os = "linux")]
impl Drop for InstanceLockGuard {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

/// Acquire the single-instance lock: `Ok(Some(guard))` when this is the only
/// instance, `Ok(None)` when another is already running, `Err` on genuine
/// lock-creation failure.
///
/// Linux binds the abstract unix socket itself instead of using the
/// single-instance crate: that crate creates the socket WITHOUT
/// `SOCK_CLOEXEC`, so the lock fd leaks into every child process we spawn —
/// an `adb fork-server` that outlived a crashed server kept the lock held and
/// blocked every later start with a bogus "already running".
#[cfg(target_os = "linux")]
fn acquire_instance_lock(name: &str) -> std::io::Result<Option<InstanceLockGuard>> {
    use std::io::Error;

    let bytes = name.as_bytes();
    // sun_path is 108 bytes; the abstract name gets a leading NUL.
    if bytes.len() + 1 > 108 {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "instance lock name too long",
        ));
    }
    unsafe {
        let fd = libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0);
        if fd < 0 {
            return Err(Error::last_os_error());
        }
        let mut addr: libc::sockaddr_un = std::mem::zeroed();
        addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr().cast::<libc::c_char>(),
            addr.sun_path.as_mut_ptr().add(1),
            bytes.len(),
        );
        let len = (std::mem::offset_of!(libc::sockaddr_un, sun_path) + 1 + bytes.len())
            as libc::socklen_t;
        if libc::bind(fd, &addr as *const _ as *const libc::sockaddr, len) == 0 {
            return Ok(Some(InstanceLockGuard { fd }));
        }
        let error = Error::last_os_error();
        libc::close(fd);
        if error.raw_os_error() == Some(libc::EADDRINUSE) {
            Ok(None)
        } else {
            Err(error)
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn acquire_instance_lock(name: &str) -> Result<Option<single_instance::SingleInstance>, String> {
    single_instance::SingleInstance::new(name)
        .map(|guard| guard.is_single().then_some(guard))
        .map_err(|error| error.to_string())
}

/// True when another vterm-server instance currently holds the
/// single-instance lock. Used by the headless CLI to warn that a running
/// server will not see config changes made from the command line.
pub(crate) fn server_instance_running() -> bool {
    // Probing releases the lock again immediately (guard drops here).
    acquire_instance_lock(&singleton_instance_name())
        .map(|guard| guard.is_none())
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn tune_glibc_malloc_for_large_transient_buffers() -> GlibcMallocTuning {
    const M_TRIM_THRESHOLD: std::os::raw::c_int = -1;
    const M_MMAP_THRESHOLD: std::os::raw::c_int = -3;
    const M_ARENA_MAX: std::os::raw::c_int = -8;

    const ARENA_MAX: std::os::raw::c_int = 2;
    const MMAP_THRESHOLD: std::os::raw::c_int = 128 * 1024;
    const TRIM_THRESHOLD: std::os::raw::c_int = 128 * 1024;

    unsafe {
        GlibcMallocTuning {
            arena_max: mallopt(M_ARENA_MAX, ARENA_MAX),
            mmap_threshold: mallopt(M_MMAP_THRESHOLD, MMAP_THRESHOLD),
            trim_threshold: mallopt(M_TRIM_THRESHOLD, TRIM_THRESHOLD),
        }
    }
}

#[cfg(target_os = "linux")]
extern "C" {
    fn mallopt(param: std::os::raw::c_int, value: std::os::raw::c_int) -> std::os::raw::c_int;
}

#[cfg(feature = "ui")]
/// BCP-47 tag of the operating-system display language (e.g. "zh-CN",
/// "en-US"), passed to the WebUI as `?lang=` on first start. The WebUI maps
/// unknown tags to English, so no filtering happens here.
fn system_language_tag() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Globalization::GetUserDefaultLocaleName;
        // LOCALE_NAME_MAX_LENGTH
        let mut buffer = [0u16; 85];
        let len = unsafe { GetUserDefaultLocaleName(buffer.as_mut_ptr(), buffer.len() as i32) };
        if len <= 0 {
            return None;
        }
        let tag = String::from_utf16_lossy(&buffer[..(len as usize - 1)]);
        if tag.is_empty() {
            None
        } else {
            Some(tag)
        }
    }
    #[cfg(target_os = "macos")]
    {
        // NSLocale.currentLocale.localeIdentifier → "zh_CN" → BCP-47 "zh-CN".
        use objc::{class, msg_send, sel, sel_impl};
        unsafe {
            let locale: *mut objc::runtime::Object = msg_send![class!(NSLocale), currentLocale];
            if locale.is_null() {
                return None;
            }
            let ident: *mut objc::runtime::Object = msg_send![locale, localeIdentifier];
            if ident.is_null() {
                return None;
            }
            let utf8: *const std::os::raw::c_char = msg_send![ident, UTF8String];
            if utf8.is_null() {
                return None;
            }
            let tag = std::ffi::CStr::from_ptr(utf8)
                .to_string_lossy()
                .replace('_', "-");
            if tag.is_empty() {
                None
            } else {
                Some(tag)
            }
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            let Ok(value) = std::env::var(var) else {
                continue;
            };
            let value = value.trim();
            if value.is_empty() || value == "C" || value == "POSIX" {
                continue;
            }
            // "zh_CN.UTF-8@modifier" → "zh-CN"
            let base = value.split(['.', '@']).next().unwrap_or(value);
            if !base.is_empty() {
                return Some(base.replace('_', "-"));
            }
        }
        None
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    None
}

#[cfg(feature = "ui")]
fn run_ui_app(
    config_port: u16,
    voice_state: Arc<std::sync::RwLock<trayicon::TrayVoiceState>>,
    action_tx: std::sync::mpsc::Sender<trayicon::TrayAction>,
) {
    #[cfg(target_os = "linux")]
    {
        // Autostart via lingering can start the server at boot, before the
        // graphical login: the process environment then has no display and
        // the one-shot `has_desktop` check would suppress the tray icon for
        // the server's whole lifetime. Wait (with backoff) until a
        // graphical session appears instead.
        if !trayicon::has_desktop() {
            info!("No desktop environment detected yet; waiting for a graphical session before creating the tray icon");
            let mut attempt: u32 = 0;
            while !desktop_env::import_desktop_environment() {
                attempt += 1;
                let wait_secs = if attempt <= 30 { 2 } else { 15 };
                std::thread::sleep(std::time::Duration::from_secs(wait_secs));
            }
            info!("Graphical session detected; creating tray icon");
        }
    }

    if !trayicon::has_desktop() {
        info!("No desktop environment detected, skipping tray icon");
        loop {
            std::thread::park();
        }
    }

    let app = vterm_ui::App::new_accessory();
    let mut tray_icon = match trayicon::TrayIcon::new(config_port, voice_state, action_tx) {
        Ok(tray_icon) => Some(tray_icon),
        Err(error) => {
            warn!("Tray icon error (non-fatal): {}", error);
            None
        }
    };

    app.run(|| {
        if let Some(tray_icon) = tray_icon.as_mut() {
            tray_icon.poll();
        }
    });
}

async fn run_server_inner(
    config_port_tx: Option<std::sync::mpsc::Sender<u16>>,
    voice_state: Option<Arc<std::sync::RwLock<trayicon::TrayVoiceState>>>,
    action_rx: Option<std::sync::mpsc::Receiver<trayicon::TrayAction>>,
) -> anyhow::Result<()> {
    // Tracing is initialized by diagnostics::init() in main() (log file +
    // console tee); the subscriber is global, so nothing to do here.
    let config_manager = Arc::new(ConfigManager::load().await?);
    // Reconcile the server-managed "Vibe Terminal MCP Bearer Token"
    // environment leaf with the persisted MCP config (created/refreshed
    // while MCP is enabled, removed when disabled). Runs at every startup so
    // the leaf exists for upgraded installs without a settings round-trip.
    {
        let mut config = config_manager.snapshot().await;
        if config::sync_mcp_token_environment(&mut config) {
            config_manager.replace(config).await?;
        }
    }
    let startup_config = config_manager.snapshot().await;

    let port = read_env_u16("PORT").unwrap_or(startup_config.server.relay_port);
    let config_port = read_env_u16("CONFIG_PORT").unwrap_or(startup_config.server.config_port);
    let listeners = bind_server_listeners(port)?;

    info!("Starting terminal relay server on port {}", port);

    // The managed identity uses a shared certificate resolver so every cloned
    // acceptor sees reissued leaf certificates on subsequent handshakes.
    let tls_acceptor_result = load_rustls_tls_acceptor(
        &startup_config.server.cert_path,
        &startup_config.server.key_path,
        &startup_config.server.tls_extra_sans,
    );
    let (tls_acceptor, private_ca) = match tls_acceptor_result {
        Ok(identity) => {
            info!("TLS configuration loaded successfully");
            identity
        }
        Err(e) => {
            error!("Failed to load TLS configuration: {}", e);
            diagnostics::flush_and_exit(1);
        }
    };

    let session_manager = Arc::new(SessionManager::new(Arc::clone(&config_manager)));
    session_manager.start_recent_project_scan();
    // A volume mounted after startup (encrypted volume unlocked after login,
    // USB drive, …) makes projects visible that the startup scan missed.
    {
        let session_manager = Arc::clone(&session_manager);
        volume_watch::spawn(move || {
            info!("volume change settled; rescanning recent projects");
            session_manager.start_recent_project_scan();
        });
    }
    session_manager.start_recent_command_scan();
    // Port-forwarding listeners use the same in-memory TLS configuration as the
    // terminal listener. In particular, this prevents macOS proxy connections
    // from creating a native-tls/Security.framework identity per connection.
    let port_forward_tls_acceptor = tls_acceptor.clone();
    let port_forward_manager = Arc::new(PortForwardManager::with_port_range(
        startup_config.server.proxy_port_start,
        startup_config.server.proxy_port_end,
        port_forward_tls_acceptor,
    ));
    port_forward_manager.update_settings(
        startup_config.server.proxy_enabled,
        startup_config.server.proxy_idle_timeout_minutes,
    );

    // The screen-streaming supervisor: owns the local vterm-screen-server
    // process (attach-first, spawn when `remote_desktop.enabled` is on or a
    // stream request arrives) and mints the one-time stream tokens for
    // client media connections. Created here (before RelayManager) so the
    // relay can coordinate screen streams for shared servers.
    let screen_supervisor = Arc::new(screen::ScreenServerSupervisor::new(Arc::clone(
        &config_manager,
    )));
    {
        let supervisor = Arc::clone(&screen_supervisor);
        tokio::spawn(async move {
            supervisor.run().await;
        });
    }

    let connection_registry = Arc::new(ConnectionRegistry::new());
    let relay_manager = Arc::new(RelayManager::new(
        Arc::clone(&config_manager),
        Arc::clone(&session_manager),
        Arc::clone(&screen_supervisor),
        Arc::clone(&port_forward_manager),
        Arc::clone(&connection_registry),
    ));
    // Server settings are an independent client-facing state. Broadcast local
    // updates immediately; a shared server also forwards its own update via
    // its upstream relay connection.
    {
        let mut config_changes = config_manager.subscribe();
        let config_manager = Arc::clone(&config_manager);
        let relay_manager = Arc::clone(&relay_manager);
        tokio::spawn(async move {
            while config_changes.changed().await.is_ok() {
                let config = config_manager.snapshot().await;
                relay_manager
                    .broadcast_client_message(Message::ServerInfo {
                        server_id: Some(config.server.server_id.clone()),
                        info: crate::config::public_server_info(&config),
                    })
                    .await;
            }
        });
    }
    let static_http_manager = Arc::new(StaticHttpManager::new(
        Arc::clone(&config_manager),
        // Same in-memory identity as the terminal listener: hosted static
        // services accept https (TLS sniffed per connection).
        tls_acceptor.clone(),
    ));
    // Hot-restart hosted static HTTP services when their settings (bind
    // address / port range) change, so a webui change applies to
    // already-running services immediately. Mirrors the MCP/ASR watchers.
    {
        let static_http_manager = Arc::clone(&static_http_manager);
        tokio::spawn(async move {
            static_http_manager.run().await;
        });
    }

    // Initialize the ASR supervisor. This owns the locally-managed
    // vterm-asr-server process: it spawns / health-attaches / restarts on
    // config changes and exposes the live HTTP endpoint + bearer token to
    // the voice API handlers.
    let voice_supervisor = Arc::new(voice::AsrSupervisor::new(Arc::clone(&config_manager)));
    // Drive the supervisor's config-watch loop in the background. It will
    // bring the asr-server up if `asr.enabled` is true, hot-restart on
    // endpoint config changes, and tear down on shutdown.
    {
        let supervisor = Arc::clone(&voice_supervisor);
        tokio::spawn(async move {
            supervisor.run().await;
        });
    }

    // Initialize tray voice state if tray is present.
    if let Some(ref vs) = voice_state {
        // "is_local" now means "the locally-supervised vterm-voice process is
        // up and reachable" — the condition under which the tray's Voice Input
        // menu can actually drive transcription. It's decoupled from the ASR
        // provider setting (there is no longer a "local" provider option).
        let is_local = voice_supervisor.endpoint().await.is_some();
        {
            let mut guard = vs.write().unwrap();
            guard.voice_enabled = startup_config.asr.enabled;
            guard.is_local = is_local;
            // No in-process state anymore; the supervisor proxies status from
            // the asr-server on demand. Leave the tray on NotLoaded for now;
            // the periodic sync task below will refresh it over HTTP.
            guard.service_state = voice::VoiceServiceState::NotLoaded;
        }

        // Periodically sync voice service state to tray. State is now read
        // from the asr-server over HTTP via the supervisor.
        let vs_clone = vs.clone();
        let vs_for_sync = Arc::clone(&voice_supervisor);
        let cm_for_sync = config_manager.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let config = cm_for_sync.snapshot().await;
                let is_local = vs_for_sync.endpoint().await.is_some();
                let state = vs_for_sync.model_status().await;
                if let Ok(mut guard) = vs_clone.write() {
                    guard.voice_enabled = config.asr.enabled;
                    guard.is_local = is_local;
                    guard.service_state = state;
                }
            }
        });
    }

    // Handle tray actions (e.g., voice start/stop).
    // Use an async channel bridge: spawn_blocking reads the sync mpsc receiver,
    // then forwards actions to an async mpsc channel consumed by a tokio task.
    if let Some(rx) = action_rx {
        let (async_tx, mut async_rx) =
            tokio::sync::mpsc::unbounded_channel::<trayicon::TrayAction>();
        tokio::task::spawn_blocking(move || {
            while let Ok(action) = rx.recv() {
                if async_tx.send(action).is_err() {
                    break;
                }
            }
        });
        let cm_for_action = config_manager.clone();
        let vs_for_action = Arc::clone(&voice_supervisor);
        let static_http_for_action = Arc::clone(&static_http_manager);
        tokio::spawn(async move {
            while let Some(action) = async_rx.recv().await {
                match action {
                    trayicon::TrayAction::StartVoice => {
                        let config = cm_for_action.snapshot().await;
                        tracing::info!("Tray: starting vterm-voice (supervised asr-server)");
                        // Bring the supervised asr-server process up. The
                        // process auto-loads the configured model on its own
                        // startup (its main.rs kicks off the load right after
                        // binding the HTTP port), so we don't issue a separate
                        // /v1/model/load here. This mirrors the WebUI's Start
                        // Server button.
                        if let Err(e) = vs_for_action.ensure_running(&config.asr).await {
                            tracing::warn!("Failed to start vterm-voice: {e:?}");
                        }
                    }
                    trayicon::TrayAction::StopVoice => {
                        tracing::info!("Tray: stopping vterm-voice (supervised asr-server)");
                        // Tear down the supervised process entirely. This
                        // matches the WebUI's Stop Server button and the
                        // independence of vterm-voice from the Enable Voice
                        // Recognition toggle.
                        vs_for_action.stop().await;
                    }
                    trayicon::TrayAction::StopStaticHttp { root } => {
                        static_http_for_action.stop_directory(&root).await;
                    }
                }
            }
        });
    }

    let (relay_state_tx, mut relay_state_rx) = mpsc::unbounded_channel();
    session_manager
        .set_state_change_notifier(relay_state_tx)
        .await;
    let relay_manager_for_state_changes = Arc::clone(&relay_manager);
    tokio::spawn(async move {
        // Trailing-edge debounce with a max-delay cap. Every broadcast
        // costs O(all sessions) lock sweeps, and with N sessions generating
        // CWD/app-status events the un-coalesced path turned into a
        // quadratic feedback loop that stalled the echo path. Coalesce a
        // burst into one broadcast, but never defer longer than the cap.
        const STATE_BROADCAST_DEBOUNCE: Duration = Duration::from_millis(100);
        const STATE_BROADCAST_MAX_DELAY: Duration = Duration::from_millis(500);
        while relay_state_rx.recv().await.is_some() {
            let deadline = tokio::time::Instant::now() + STATE_BROADCAST_MAX_DELAY;
            loop {
                let now = tokio::time::Instant::now();
                if now >= deadline {
                    break;
                }
                let window = std::cmp::min(deadline, now + STATE_BROADCAST_DEBOUNCE);
                match tokio::time::timeout_at(window, relay_state_rx.recv()).await {
                    Ok(Some(())) => continue,
                    Ok(None) => return,
                    Err(_) => break,
                }
            }
            relay_manager_for_state_changes
                .broadcast_session_list()
                .await;
            relay_manager_for_state_changes.notify_local_state_changed();
        }
    });
    let mut recent_project_scan_rx = session_manager.subscribe_recent_project_scan();
    let relay_manager_for_recent_projects = Arc::clone(&relay_manager);
    let config_manager_for_recent_projects = Arc::clone(&config_manager);
    tokio::spawn(async move {
        let mut first = true;
        let mut last_broadcast_paths: Option<Vec<String>> = None;
        loop {
            if first {
                first = false;
            } else if recent_project_scan_rx.changed().await.is_err() {
                break;
            }
            let scan_state = { recent_project_scan_rx.borrow().clone() };
            if let RecentProjectScanState::Ready(directories) = scan_state {
                let paths = directories
                    .iter()
                    .map(|directory| directory.path.clone())
                    .collect::<Vec<_>>();
                if last_broadcast_paths.as_ref() == Some(&paths) {
                    continue;
                }
                last_broadcast_paths = Some(paths);
                let server_id = config_manager_for_recent_projects
                    .snapshot()
                    .await
                    .server
                    .server_id;
                trace!(
                    "Broadcasting {} recent project directory/directories after scan completion",
                    directories.len()
                );
                relay_manager_for_recent_projects
                    .broadcast_client_message(Message::RecentProjectDirectoryList {
                        directories,
                        server_id: Some(server_id),
                    })
                    .await;
            }
        }
    });
    let addon_host = Arc::new(AddonHost::new());
    let ddns_manager = Arc::new(DdnsManager::new(
        Arc::clone(&config_manager),
        Arc::clone(&addon_host),
    ));

    let config_server = ConfigHttpServer::new(
        config_port,
        Arc::clone(&config_manager),
        Arc::clone(&private_ca),
        Arc::clone(&relay_manager),
        Arc::clone(&ddns_manager),
        Arc::clone(&addon_host),
        Arc::clone(&voice_supervisor),
        Arc::clone(&screen_supervisor),
        Arc::clone(&static_http_manager),
        Arc::clone(&port_forward_manager),
    );
    let config_server_task = tokio::spawn(async move {
        if let Err(error) = config_server.run().await {
            error!("Config server exited: {}", error);
        }
    });

    {
        let private_ca = Arc::clone(&private_ca);
        let config_manager = Arc::clone(&config_manager);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                if !private_ca.managed_identity_active() {
                    continue;
                }
                let extra_sans = config_manager.snapshot().await.server.tls_extra_sans;
                let manager = Arc::clone(&private_ca);
                let _ =
                    tokio::task::spawn_blocking(move || manager.ensure_certificate(&extra_sans))
                        .await;
            }
        });
    }

    // MCP host (in-process, exposes terminal and addon services to Code Agents
    // via Streamable HTTP). Hot-reloads on config changes; see mcp::McpServer.
    let mcp_server = crate::mcp::McpServer::new(
        Arc::clone(&session_manager),
        Arc::clone(&config_manager),
        Arc::clone(&addon_host),
        tls_acceptor.clone(),
    );
    let mcp_server_task = tokio::spawn(async move {
        if let Err(error) = mcp_server.run().await {
            error!("MCP host exited: {}", error);
        }
    });

    tokio::spawn(Arc::clone(&ddns_manager).run());
    tokio::spawn(Arc::clone(&relay_manager).run_upstream_connector());

    let fs_watcher_manager = Arc::new(FsWatcherManager::new()?);
    session_manager
        .set_fs_watcher_manager(Arc::clone(&fs_watcher_manager))
        .await;
    session_manager.start_ai_conversation_global_watch(Arc::clone(&fs_watcher_manager));
    tokio::spawn(Arc::clone(&fs_watcher_manager).run(Arc::clone(&session_manager)));

    let notification_manager = Arc::new(NotificationManager::new(
        Arc::clone(&config_manager),
        startup_config.notification.queue_capacity,
    ));
    tokio::spawn(Arc::clone(&notification_manager).run());

    let (observer_event_tx, mut observer_event_rx) = mpsc::unbounded_channel();
    let terminal_observer = Arc::new(TerminalObserverManager::new(
        Arc::clone(&config_manager),
        Arc::clone(&addon_host),
        observer_event_tx,
    ));
    session_manager
        .set_terminal_observer(Arc::clone(&terminal_observer))
        .await;

    let relay_manager_for_observer = Arc::clone(&relay_manager);
    let notification_manager_for_observer = Arc::clone(&notification_manager);
    let config_manager_for_observer = Arc::clone(&config_manager);
    let session_manager_for_observer = Arc::clone(&session_manager);
    tokio::spawn(async move {
        while let Some(event) = observer_event_rx.recv().await {
            match event {
                TerminalObserverEvent::SessionStateChanged {
                    session_id,
                    app,
                    app_status,
                    app_status_summary,
                } => {
                    if let Err(error) = session_manager_for_observer
                        .update_session_observation(session_id, app, app_status, app_status_summary)
                        .await
                    {
                        warn!(
                            "Failed to update observed terminal state for session {}: {}",
                            session_id, error
                        );
                    }
                }
                TerminalObserverEvent::AttentionRequired(attention) => {
                    relay_manager_for_observer
                        .broadcast_client_message(Message::TerminalAttentionRequired {
                            attention: attention.clone(),
                        })
                        .await;

                    let mut payload = NotificationPayload::new(
                        format!("{} needs input", attention.title),
                        attention.matched_line.clone(),
                    );
                    payload
                        .data
                        .insert("session_id".to_string(), attention.session_id.to_string());
                    payload.data.insert(
                        "kind".to_string(),
                        format!("{:?}", attention.kind).to_ascii_lowercase(),
                    );
                    payload
                        .data
                        .insert("summary".to_string(), attention.summary.clone());

                    let _ = notification_manager_for_observer
                        .enqueue_event("terminal_input_required", payload)
                        .await;
                }
                TerminalObserverEvent::TaskCompleted(completion) => {
                    relay_manager_for_observer
                        .broadcast_client_message(Message::TerminalTaskCompleted {
                            completion: completion.clone(),
                        })
                        .await;

                    if config_manager_for_observer
                        .snapshot()
                        .await
                        .terminal
                        .notify_on_completion
                    {
                        let mut payload = NotificationPayload::new(
                            format!("{} completed", completion.title),
                            completion.matched_line.clone(),
                        );
                        payload
                            .data
                            .insert("session_id".to_string(), completion.session_id.to_string());
                        payload
                            .data
                            .insert("summary".to_string(), completion.summary.clone());
                        let _ = notification_manager_for_observer
                            .enqueue_event("terminal_task_completed", payload)
                            .await;
                    }
                }
                TerminalObserverEvent::AutoConfirmRequested { session_id } => {
                    if let Err(error) = session_manager_for_observer
                        .forward_input(session_id, None, b"\r".to_vec())
                        .await
                    {
                        warn!(
                            "Failed to auto-confirm terminal prompt for session {}: {}",
                            session_id, error
                        );
                    }
                }
            }
        }
    });

    if let Some(tx) = config_port_tx {
        tx.send(config_port)
            .expect("Failed to send config_port to main thread");
    }

    for addr in listeners.addrs() {
        info!("Server listening on {} with TLS enabled", addr);
    }

    loop {
        let accepted = listeners.accept().await;
        match accepted {
            Ok((stream, peer_addr)) => {
                let config_manager = Arc::clone(&config_manager);
                let session_manager = Arc::clone(&session_manager);
                let relay_manager = Arc::clone(&relay_manager);
                let port_forward_manager = Arc::clone(&port_forward_manager);
                let fs_watcher_manager = Arc::clone(&fs_watcher_manager);
                let static_http_manager = Arc::clone(&static_http_manager);
                let addon_host_clone = Arc::clone(&addon_host);
                let connection_registry = Arc::clone(&connection_registry);
                let voice_supervisor = Arc::clone(&voice_supervisor);
                let screen_supervisor = Arc::clone(&screen_supervisor);
                let tls_acceptor = tls_acceptor.clone();

                tokio::spawn(async move {
                    if let Err(error) = stream.set_nodelay(true) {
                        warn!("Failed to enable TCP_NODELAY for {}: {}", peer_addr, error);
                    }

                    // Auth-failure lockout (security review §1.2): drop
                    // before spending a TLS handshake on the peer.
                    if connection_registry.is_auth_limited(peer_addr.ip()) {
                        trace!(
                            "Dropping connection from {} (auth failure lockout)",
                            peer_addr
                        );
                        return;
                    }

                    // Accept TLS connection (bounded — slowloris guard, §1.1)
                    let tls_stream =
                        match tokio::time::timeout(TLS_ACCEPT_TIMEOUT, tls_acceptor.accept(stream))
                            .await
                        {
                            Ok(Ok(tls_stream)) => tls_stream,
                            Ok(Err(e)) => {
                                // EOF/aborted/reset mid-handshake is routine:
                                // port probes, health checks and clients hanging
                                // up early all look like this. Any other failure
                                // is worth surfacing at warn.
                                if matches!(
                                    e.kind(),
                                    std::io::ErrorKind::UnexpectedEof
                                        | std::io::ErrorKind::ConnectionAborted
                                        | std::io::ErrorKind::ConnectionReset
                                ) {
                                    trace!(
                                        "TLS handshake ended early for {}: {} ({:?})",
                                        peer_addr,
                                        e,
                                        e
                                    );
                                } else {
                                    warn!(
                                        "TLS handshake failed for {}: {} ({:?})",
                                        peer_addr, e, e
                                    );
                                }
                                return;
                            }
                            Err(_) => {
                                trace!("TLS handshake timed out for {}", peer_addr);
                                return;
                            }
                        };
                    // Pre-auth handshake concurrency slot (security review
                    // §1.1 complement): bounds how many connections may be
                    // in their WS/auth/Noise handshake phase at once. The
                    // permit rides on the Connection and is released as
                    // soon as its handshake completes.
                    let handshake_permit = match crate::connection::handshake_concurrency_limiter()
                        .acquire_owned()
                        .await
                    {
                        Ok(permit) => permit,
                        Err(_) => return, // limiter closed; shutting down
                    };
                    {
                        let connection = Connection::new(
                            peer_addr,
                            config_manager,
                            session_manager,
                            addon_host_clone,
                            relay_manager,
                            port_forward_manager,
                            fs_watcher_manager,
                            static_http_manager,
                            connection_registry,
                            voice_supervisor,
                            screen_supervisor,
                            handshake_permit,
                        );

                        if let Err(error) = connection.handle(tls_stream).await {
                            // Mostly benign disconnect noise (e.g. the
                            // peer closed without a TLS close_notify);
                            // useful when troubleshooting but far too
                            // frequent for the default level.
                            trace!(
                                "Connection error debug details from {}: {:?}",
                                peer_addr,
                                error
                            );
                        }
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn read_env_u16(name: &str) -> Option<u16> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
}
