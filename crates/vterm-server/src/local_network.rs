//! Detection and mitigation for the macOS "Local Network" privacy block.
//!
//! When the NECP privacy session for this app gets corrupted (commonly after
//! reinstalling the .app over itself), the kernel synchronously drops every
//! outbound LAN connection from this process with EPERM (macOS 14) or
//! EHOSTUNREACH (macOS 15), while inbound LAN connections and internet access
//! keep working. The relay retries its TCP transport through a child process
//! that disclaims the parent's responsibility session, while keeping TLS,
//! WebSocket state, and terminal sessions in the server process. The tracker
//! also pokes the system into re-evaluating the permission via a short Bonjour
//! browse, notifies the user periodically, and lets the caller retry fast.

use std::net::IpAddr;
use std::time::{Duration, Instant};

use tracing::{info, warn};

/// Consecutive qualifying failures before the block is declared active.
const FAILURE_THRESHOLD: u32 = 3;
/// A qualifying connect failure must happen within this window. A genuine
/// LAN outage fails slowly (ARP resolution, TCP retransmits); a NECP drop is
/// returned synchronously by the kernel, typically in under a millisecond.
const MAX_BLOCK_FAILURE_LATENCY: Duration = Duration::from_millis(250);
/// How often the user notification repeats while the block persists.
const NOTIFICATION_INTERVAL: Duration = Duration::from_secs(30 * 60);

/// Whether `ip` belongs to a local network segment: RFC1918, link-local,
/// or IPv6 unique-local / link-local. Loopback is deliberately excluded.
pub fn is_lan_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => {
            let first = v6.segments()[0];
            // fc00::/7 unique-local, fe80::/10 link-local
            (first & 0xfe00) == 0xfc00 || (first & 0xffc0) == 0xfe80
        }
    }
}

/// Extract the host portion from a relay address — bare `host`, `host:port`,
/// or a ws(s):// URL — and report whether it is a LAN address literal.
/// Hostnames (including mDNS `.local` names) are not resolved and report
/// false.
pub fn relay_target_is_lan(address: &str) -> bool {
    let trimmed = address.trim();
    let without_scheme = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed);
    let authority = without_scheme.split('/').next().unwrap_or("");
    let host = if let Some(stripped) = authority.strip_prefix('[') {
        stripped.split(']').next().unwrap_or("")
    } else {
        authority.split(':').next().unwrap_or("")
    };
    host.parse::<IpAddr>()
        .map(|ip| is_lan_ip(&ip))
        .unwrap_or(false)
}

/// Whether the error chain indicates a macOS Local Network NECP block: the
/// kernel drops outbound LAN connections from the blocked process
/// synchronously with EPERM (1) or EHOSTUNREACH (65).
pub fn is_local_network_block_error(error: &anyhow::Error) -> bool {
    if !cfg!(target_os = "macos") {
        return false;
    }
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
        .any(|io| matches!(io.raw_os_error(), Some(1) | Some(65)))
}

/// Evidence that the LAN path itself works right now, used to confirm a
/// suspected block before notifying the user: either a LAN client currently
/// holds a primary connection to this server, or the relay host still has a
/// resolved ARP entry (it was alive on this LAN moments ago).
fn lan_path_evidence(relay_address: &str, has_inbound_lan_client: bool) -> bool {
    if has_inbound_lan_client {
        return true;
    }
    arp_entry_resolves(relay_address)
}

#[cfg(target_os = "macos")]
fn arp_entry_resolves(relay_address: &str) -> bool {
    use std::process::{Command, Stdio};

    let trimmed = relay_address.trim();
    let without_scheme = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed);
    let authority = without_scheme.split('/').next().unwrap_or("");
    let host = authority
        .strip_prefix('[')
        .and_then(|s| s.split(']').next())
        .unwrap_or_else(|| authority.split(':').next().unwrap_or(""));
    if host.is_empty() {
        return false;
    }
    let needle = format!("({host}) at ");
    let output = Command::new("/usr/sbin/arp")
        .arg("-an")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output();
    match output {
        Ok(output) => String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.contains(&needle) && !line.contains("incomplete")),
        Err(_) => false,
    }
}

#[cfg(not(target_os = "macos"))]
fn arp_entry_resolves(_relay_address: &str) -> bool {
    false
}

#[cfg(target_os = "macos")]
const RECOVERY_PROCESS_FLAG: &str = "--local-network-recovery";
#[cfg(target_os = "macos")]
const RECOVERY_READY: &str = "VTERM_LOCAL_NETWORK_READY";

#[cfg(target_os = "macos")]
pub fn run_recovery_process_if_requested() -> Option<anyhow::Result<()>> {
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpStream};
    use std::os::fd::FromRawFd;

    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().map(|arg| arg.as_os_str()) != Some(RECOVERY_PROCESS_FLAG.as_ref()) {
        return None;
    }
    Some((|| {
        if args.len() != 3 {
            anyhow::bail!("Local Network recovery requires a host and port");
        }
        let host = args[1]
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Local Network recovery host is not UTF-8"))?;
        let port = args[2]
            .to_str()
            .and_then(|value| value.parse::<u16>().ok())
            .ok_or_else(|| anyhow::anyhow!("Local Network recovery port is invalid"))?;
        let stream = TcpStream::connect((host, port))?;
        writeln!(std::io::stderr().lock(), "{RECOVERY_READY}")?;
        let mut input_stream = stream.try_clone()?;
        let input = std::thread::spawn(move || -> std::io::Result<()> {
            let stdin = std::io::stdin();
            let mut stdin = stdin.lock();
            let _ = std::io::copy(&mut stdin, &mut input_stream)?;
            input_stream.shutdown(Shutdown::Write)
        });
        let mut stream = stream;
        // StdoutLock is line-buffered and stalls binary TLS records that contain no newline.
        let mut stdout = unsafe { std::fs::File::from_raw_fd(libc::STDOUT_FILENO) };
        let output_result = std::io::copy(&mut stream, &mut stdout).map(|_| ());
        let _ = stream.shutdown(Shutdown::Both);
        let input_result = input.join();
        output_result?;
        input_result
            .map_err(|_| anyhow::anyhow!("Local Network recovery input thread panicked"))??;
        Ok(())
    })())
}

#[cfg(target_os = "macos")]
struct RecoveryChild(libc::pid_t);

#[cfg(target_os = "macos")]
impl Drop for RecoveryChild {
    fn drop(&mut self) {
        unsafe {
            libc::kill(self.0, libc::SIGKILL);
            loop {
                if libc::waitpid(self.0, std::ptr::null_mut(), 0) >= 0 {
                    break;
                }
                if std::io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
                    break;
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
struct SpawnAttr(libc::posix_spawnattr_t);

#[cfg(target_os = "macos")]
impl SpawnAttr {
    fn new() -> anyhow::Result<Self> {
        let mut attr = std::mem::MaybeUninit::uninit();
        let result = unsafe { libc::posix_spawnattr_init(attr.as_mut_ptr()) };
        if result != 0 {
            return Err(std::io::Error::from_raw_os_error(result).into());
        }
        Ok(Self(unsafe { attr.assume_init() }))
    }
}

#[cfg(target_os = "macos")]
impl Drop for SpawnAttr {
    fn drop(&mut self) {
        unsafe {
            libc::posix_spawnattr_destroy(&mut self.0);
        }
    }
}

#[cfg(target_os = "macos")]
struct SpawnFileActions(libc::posix_spawn_file_actions_t);

#[cfg(target_os = "macos")]
impl SpawnFileActions {
    fn new() -> anyhow::Result<Self> {
        let mut actions = std::mem::MaybeUninit::uninit();
        let result = unsafe { libc::posix_spawn_file_actions_init(actions.as_mut_ptr()) };
        if result != 0 {
            return Err(std::io::Error::from_raw_os_error(result).into());
        }
        Ok(Self(unsafe { actions.assume_init() }))
    }

    fn check(operation: &str, result: libc::c_int) -> anyhow::Result<()> {
        if result == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "{operation}: {}",
                std::io::Error::from_raw_os_error(result)
            ))
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for SpawnFileActions {
    fn drop(&mut self) {
        unsafe {
            libc::posix_spawn_file_actions_destroy(&mut self.0);
        }
    }
}

#[cfg(target_os = "macos")]
fn owned_pipe() -> anyhow::Result<[std::os::fd::OwnedFd; 2]> {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

    let mut fds = [-1; 2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let pipes = unsafe { [OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])] };
    for fd in &pipes {
        if unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    Ok(pipes)
}

#[cfg(target_os = "macos")]
fn spawn_disclaimed(
    executable: &std::ffi::OsStr,
    args: &[std::ffi::OsString],
) -> anyhow::Result<(
    RecoveryChild,
    tokio::net::unix::pipe::Sender,
    tokio::net::unix::pipe::Receiver,
    tokio::net::unix::pipe::Receiver,
)> {
    use std::ffi::CString;
    use std::os::fd::AsRawFd;
    use std::os::unix::ffi::OsStrExt;

    unsafe extern "C" {
        fn responsibility_spawnattrs_setdisclaim(
            attr: *mut libc::posix_spawnattr_t,
            disclaim: libc::c_int,
        ) -> libc::c_int;
    }

    let [stdin_read, stdin_write] = owned_pipe()?;
    let [stdout_read, stdout_write] = owned_pipe()?;
    let [stderr_read, stderr_write] = owned_pipe()?;
    let mut attr = SpawnAttr::new()?;
    let mut actions = SpawnFileActions::new()?;
    SpawnFileActions::check(
        "posix_spawnattr_setdisclaim",
        unsafe { responsibility_spawnattrs_setdisclaim(&mut attr.0, 1) },
    )?;
    for (source, destination) in [
        (stdin_read.as_raw_fd(), libc::STDIN_FILENO),
        (stdout_write.as_raw_fd(), libc::STDOUT_FILENO),
        (stderr_write.as_raw_fd(), libc::STDERR_FILENO),
    ] {
        SpawnFileActions::check(
            "posix_spawn_file_actions_adddup2",
            unsafe { libc::posix_spawn_file_actions_adddup2(&mut actions.0, source, destination) },
        )?;
    }
    for fd in [
        &stdin_read,
        &stdin_write,
        &stdout_read,
        &stdout_write,
        &stderr_read,
        &stderr_write,
    ] {
        SpawnFileActions::check(
            "posix_spawn_file_actions_addclose",
            unsafe { libc::posix_spawn_file_actions_addclose(&mut actions.0, fd.as_raw_fd()) },
        )?;
    }

    let executable = CString::new(executable.as_bytes())?;
    let args = args
        .iter()
        .map(|arg| CString::new(arg.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;
    let argv = std::iter::once(executable.as_ptr() as *mut libc::c_char)
        .chain(args.iter().map(|arg| arg.as_ptr() as *mut libc::c_char))
        .chain(std::iter::once(std::ptr::null_mut()))
        .collect::<Vec<_>>();
    let environment = std::env::vars_os()
        .map(|(key, value)| {
            let mut entry = key.as_bytes().to_vec();
            entry.push(b'=');
            entry.extend_from_slice(value.as_bytes());
            CString::new(entry)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let envp = environment
        .iter()
        .map(|entry| entry.as_ptr() as *mut libc::c_char)
        .chain(std::iter::once(std::ptr::null_mut()))
        .collect::<Vec<_>>();

    let mut pid = -1;
    let result = unsafe {
        libc::posix_spawn(
            &mut pid,
            executable.as_ptr(),
            &actions.0,
            &attr.0,
            argv.as_ptr(),
            envp.as_ptr(),
        )
    };
    if result != 0 {
        return Err(anyhow::anyhow!(
            "posix_spawn: {}",
            std::io::Error::from_raw_os_error(result)
        ));
    }
    drop(stdin_read);
    drop(stdout_write);
    drop(stderr_write);
    let child = RecoveryChild(pid);
    Ok((
        child,
        tokio::net::unix::pipe::Sender::from_owned_fd(stdin_write)?,
        tokio::net::unix::pipe::Receiver::from_owned_fd(stdout_read)?,
        tokio::net::unix::pipe::Receiver::from_owned_fd(stderr_read)?,
    ))
}

#[cfg(target_os = "macos")]
async fn recovery_stream_from_disclaimed_program(
    executable: &std::ffi::OsStr,
    args: &[std::ffi::OsString],
) -> anyhow::Result<tokio::io::DuplexStream> {
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

    let (child, child_stdin, child_stdout, child_stderr) = spawn_disclaimed(executable, args)
        .map_err(|error| anyhow::anyhow!("Failed to start local network recovery process: {error}"))?;
    let mut child_stderr = BufReader::new(child_stderr);
    let mut diagnostics = String::new();
    let connected = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let mut status = String::new();
            let read = child_stderr.read_line(&mut status).await?;
            if read == 0 {
                return std::io::Result::Ok(false);
            }
            if status.trim_end() == RECOVERY_READY {
                return Ok(true);
            }
            diagnostics.push_str(&status);
        }
    })
    .await
    .map_err(|_| anyhow::anyhow!("Local Network recovery process timed out during TCP connect"))??;
    if !connected {
        let _ = child_stderr.read_to_string(&mut diagnostics).await;
        let diagnostics = diagnostics.trim();
        anyhow::bail!(
            "Local Network recovery process failed before TCP connect: {}",
            if diagnostics.is_empty() {
                "process exited without an error message"
            } else {
                diagnostics
            }
        );
    }

    let (stream, proxy) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let (mut proxy_reader, mut proxy_writer) = tokio::io::split(proxy);
        let mut child_stdout = child_stdout;
        let mut child_stdin = child_stdin;
        let copy_result = tokio::select! {
            result = tokio::io::copy(&mut proxy_reader, &mut child_stdin) => result.map(|_| ()),
            result = tokio::io::copy(&mut child_stdout, &mut proxy_writer) => result.map(|_| ()),
        };
        drop(child);
        let mut trailing_diagnostics = String::new();
        let _ = child_stderr.read_to_string(&mut trailing_diagnostics).await;
        diagnostics.push_str(&trailing_diagnostics);
        if let Err(error) = copy_result {
            warn!("Local Network recovery forwarding failed: {error}");
        }
        let diagnostics = diagnostics.trim();
        if !diagnostics.is_empty() {
            warn!("Local Network recovery process diagnostics: {diagnostics}");
        }
    });
    Ok(stream)
}

#[cfg(target_os = "macos")]
pub async fn recovery_tcp_stream(
    host: &str,
    port: u16,
) -> anyhow::Result<tokio::io::DuplexStream> {
    let executable = std::env::current_exe()?;
    let args = vec![
        std::ffi::OsString::from(RECOVERY_PROCESS_FLAG),
        std::ffi::OsString::from(host),
        std::ffi::OsString::from(port.to_string()),
    ];
    recovery_stream_from_disclaimed_program(executable.as_os_str(), &args).await
}

/// A short-lived Bonjour browse forces macOS to re-evaluate this app's Local
/// Network permission session (nehelper). That can heal a stuck NECP state
/// outright, or surface the permission prompt if the system wants to re-ask.
#[cfg(target_os = "macos")]
fn trigger_bonjour_probe() {
    use std::process::{Command, Stdio};

    info!("Triggering Bonjour browse to re-evaluate Local Network permission");
    if let Ok(mut child) = Command::new("/usr/bin/dns-sd")
        .args(["-B", "_services._dns-sd._udp", "local."])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(5));
            let _ = child.kill();
            let _ = child.wait();
        });
    }
}

#[cfg(not(target_os = "macos"))]
fn trigger_bonjour_probe() {}

#[cfg(target_os = "macos")]
fn send_system_notification() {
    use std::process::{Command, Stdio};

    let script = concat!(
        "display notification \"macOS is blocking local network access for VibeTerminal. ",
        "Open System Settings > Privacy & Security > Local Network and toggle VibeTerminal ",
        "off and on; the connection will resume automatically.\" ",
        "with title \"Vibe Terminal\" subtitle \"Cannot reach the local relay server\" ",
        "sound name \"Glass\""
    );
    let _ = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

#[cfg(not(target_os = "macos"))]
fn send_system_notification() {}

/// Tracks consecutive upstream-relay connect failures and decides when they
/// look like a macOS Local Network privacy block rather than a genuine
/// outage.
pub struct LocalNetworkBlockTracker {
    consecutive_failures: u32,
    active: bool,
    probe_attempted: bool,
    last_notification: Option<Instant>,
}

impl LocalNetworkBlockTracker {
    pub fn new() -> Self {
        Self {
            consecutive_failures: 0,
            active: false,
            probe_attempted: false,
            last_notification: None,
        }
    }

    /// Record a failed upstream connection attempt. `latency` is how long the
    /// attempt took; `has_inbound_lan_client` should reflect whether any LAN
    /// client currently holds a primary connection to this server.
    pub fn record_failure(
        &mut self,
        error: &anyhow::Error,
        relay_address: &str,
        latency: Duration,
        has_inbound_lan_client: bool,
    ) {
        let qualifies = is_local_network_block_error(error)
            && relay_target_is_lan(relay_address)
            && latency <= MAX_BLOCK_FAILURE_LATENCY;
        if !qualifies {
            self.consecutive_failures = 0;
            return;
        }
        self.consecutive_failures += 1;
        if self.active {
            self.notify_if_due(relay_address, has_inbound_lan_client);
            return;
        }
        if self.consecutive_failures >= FAILURE_THRESHOLD {
            self.active = true;
            warn!(
                "Upstream relay failures look like a macOS Local Network permission \
                 block (target {}, {} consecutive fast EHOSTUNREACH/EPERM); retrying \
                 faster and prompting permission re-evaluation",
                relay_address, self.consecutive_failures
            );
            if !self.probe_attempted {
                self.probe_attempted = true;
                trigger_bonjour_probe();
            }
            self.notify_if_due(relay_address, has_inbound_lan_client);
        }
    }

    /// The upstream connection came up: clear all state so a future block is
    /// detected (and notified) from scratch.
    pub fn record_success(&mut self) {
        if self.active {
            info!("Local Network block cleared; upstream relay is reachable again");
        }
        *self = Self::new();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Notify on activation and then once per NOTIFICATION_INTERVAL while the
    /// block persists — but only once there is evidence the LAN itself works
    /// (otherwise the relay host may simply be down, and the notification
    /// would cry wolf).
    fn notify_if_due(&mut self, relay_address: &str, has_inbound_lan_client: bool) {
        let due = self
            .last_notification
            .map_or(true, |when| when.elapsed() >= NOTIFICATION_INTERVAL);
        if due && lan_path_evidence(relay_address, has_inbound_lan_client) {
            self.last_notification = Some(Instant::now());
            send_system_notification();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lan_ip_classification() {
        assert!(is_lan_ip(&"192.168.1.155".parse().unwrap()));
        assert!(is_lan_ip(&"10.0.0.2".parse().unwrap()));
        assert!(is_lan_ip(&"172.16.3.4".parse().unwrap()));
        assert!(is_lan_ip(&"169.254.10.1".parse().unwrap()));
        assert!(is_lan_ip(&"fd00::1".parse().unwrap()));
        assert!(is_lan_ip(&"fe80::1".parse().unwrap()));
        assert!(!is_lan_ip(&"127.0.0.1".parse().unwrap()));
        assert!(!is_lan_ip(&"8.8.8.8".parse().unwrap()));
        assert!(!is_lan_ip(&"::1".parse().unwrap()));
        assert!(!is_lan_ip(&"240e:381:724:2b00::1".parse().unwrap()));
    }

    #[test]
    fn relay_address_parsing() {
        assert!(relay_target_is_lan("192.168.1.155"));
        assert!(relay_target_is_lan("192.168.1.155:7878"));
        assert!(relay_target_is_lan("wss://192.168.1.155:7878"));
        assert!(relay_target_is_lan("ws://10.1.2.3"));
        assert!(relay_target_is_lan("wss://[fd00::5]:7878/ws"));
        assert!(!relay_target_is_lan("relay.example.com"));
        assert!(!relay_target_is_lan("wss://8.8.8.8:7878"));
        assert!(!relay_target_is_lan("127.0.0.1:7878"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn block_error_detection() {
        let blocked = anyhow::Error::new(std::io::Error::from_raw_os_error(65));
        assert!(is_local_network_block_error(&blocked));
        let refused = anyhow::Error::new(std::io::Error::from_raw_os_error(61));
        assert!(!is_local_network_block_error(&refused));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn block_error_detection_is_macos_only() {
        let error = anyhow::Error::new(std::io::Error::from_raw_os_error(65));
        assert!(!is_local_network_block_error(&error));
    }

    #[cfg(target_os = "macos")]
    async fn test_recovery_stream(port: u16) -> tokio::io::DuplexStream {
        let args = vec![
            std::ffi::OsString::from("-c"),
            std::ffi::OsString::from(format!(
                "printf '%s\\n' '{RECOVERY_READY}' >&2; exec /usr/bin/nc 127.0.0.1 {port}"
            )),
        ];
        recovery_stream_from_disclaimed_program(std::ffi::OsStr::new("/bin/sh"), &args)
            .await
            .unwrap()
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn recovery_process_forwards_bidirectional_tcp() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let peer = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0; 4];
            stream.read_exact(&mut request).await.unwrap();
            assert_eq!(&request, b"ping");
            stream.write_all(b"pong").await.unwrap();
        });

        let mut stream = test_recovery_stream(address.port()).await;
        stream.write_all(b"ping").await.unwrap();
        stream.flush().await.unwrap();
        let mut response = [0; 4];
        tokio::time::timeout(
            Duration::from_secs(5),
            stream.read_exact(&mut response),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(&response, b"pong");
        peer.await.unwrap();
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn recovery_process_completes_websocket_handshake() {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::Message;

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let peer = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut websocket = tokio_tungstenite::accept_async(stream).await.unwrap();
            let message = websocket.next().await.unwrap().unwrap();
            websocket.send(message).await.unwrap();
        });

        let stream = test_recovery_stream(address.port()).await;
        let (mut websocket, _) = tokio_tungstenite::client_async_tls_with_config(
            format!("ws://{address}"),
            stream,
            None,
            None,
        )
        .await
        .unwrap();
        websocket
            .send(Message::Text("recovery-websocket-test".into()))
            .await
            .unwrap();
        let response = websocket.next().await.unwrap().unwrap();
        assert_eq!(response, Message::Text("recovery-websocket-test".into()));
        peer.await.unwrap();
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn recovery_process_reports_connect_errors() {
        let args = vec![
            std::ffi::OsString::from("-c"),
            std::ffi::OsString::from("echo 'Error: No route to host (os error 65)' >&2; exit 1"),
        ];
        let error = match recovery_stream_from_disclaimed_program(
            std::ffi::OsStr::new("/bin/sh"),
            &args,
        )
        .await
        {
            Ok(_) => panic!("recovery process unexpectedly connected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("No route to host (os error 65)"));
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn recovery_process_does_not_wait_for_newlines() {
        use tokio::io::AsyncReadExt;

        let args = vec![
            std::ffi::OsString::from("-c"),
            std::ffi::OsString::from(format!(
                "printf '%s\\n' '{RECOVERY_READY}' >&2; printf '\\001\\002\\003'; sleep 5"
            )),
        ];
        let mut stream = recovery_stream_from_disclaimed_program(
            std::ffi::OsStr::new("/bin/sh"),
            &args,
        )
        .await
        .unwrap();
        let mut bytes = [0; 3];
        tokio::time::timeout(Duration::from_secs(1), stream.read_exact(&mut bytes))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(bytes, [1, 2, 3]);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tracker_activates_after_threshold_and_resets_on_success() {
        let mut tracker = LocalNetworkBlockTracker::new();
        let error = anyhow::Error::new(std::io::Error::from_raw_os_error(65));
        for _ in 0..FAILURE_THRESHOLD - 1 {
            tracker.record_failure(&error, "192.168.1.155", Duration::from_millis(1), false);
            assert!(!tracker.is_active());
        }
        tracker.record_failure(&error, "192.168.1.155", Duration::from_millis(1), false);
        assert!(tracker.is_active());
        tracker.record_success();
        assert!(!tracker.is_active());
        assert_eq!(tracker.consecutive_failures, 0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tracker_ignores_slow_and_non_lan_failures() {
        let mut tracker = LocalNetworkBlockTracker::new();
        let error = anyhow::Error::new(std::io::Error::from_raw_os_error(65));
        for _ in 0..10 {
            // Slow failure: genuine outage, not a NECP drop.
            tracker.record_failure(&error, "192.168.1.155", Duration::from_secs(3), false);
            // Fast failure against a public target.
            tracker.record_failure(&error, "203.0.113.10", Duration::from_millis(1), false);
        }
        assert!(!tracker.is_active());
    }
}
