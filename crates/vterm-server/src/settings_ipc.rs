//! Linux-only local IPC for the headless CLI (`vterm-server --open`).
//!
//! The settings Web UI requires token-to-cookie authentication
//! (`docs/settings-static-http-authentication.md`): a browser must present a
//! one-time ticket that the running server exchanges for a session cookie.
//! Tickets live only in the server process's memory, so a second process
//! cannot mint one itself. On Linux — where the headless server has no tray
//! icon and no first-run browser path — the server therefore listens on a
//! small abstract-namespace unix socket: the CLI connects, asks for a
//! ticket, and prints/opens the URL with `?ticket=...` attached.
//!
//! Security: abstract sockets carry no file permissions, so the server
//! verifies `SO_PEERCRED` and only serves its own effective UID. That user
//! can already read the config file holding the secret key — a settings
//! ticket grants nothing beyond what they hold. The socket name embeds the
//! UID (and the dev/prod flavor) so parallel users or flavors never cross.

use std::io::{Read, Write};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};
use std::sync::Once;
use std::time::Duration;

use tracing::{info, warn};

/// Single-line request the CLI sends. One request per connection.
const REQUEST: &str = "GET-TICKET";
/// Upper bound for a request line — anything longer is garbage and the
/// connection is dropped instead of buffered.
const MAX_REQUEST_LINE: usize = 128;
/// Both sides are local; anything beyond a couple of seconds means the peer
/// is misbehaving or hung.
const IO_TIMEOUT: Duration = Duration::from_secs(2);

/// Abstract socket name shared by server and CLI. Derived from the
/// single-instance lock base (see `app.rs::singleton_instance_name`) plus
/// the effective UID: same user + same flavor finds its server, other
/// users/flavors get their own endpoint.
pub fn socket_name() -> String {
    let base = if cfg!(DEVELOPMENT) {
        "vibe-terminal-vterm-server-dev"
    } else {
        "vibe-terminal-vterm-server"
    };
    format!("{base}-settings-{}", effective_uid())
}

fn effective_uid() -> u32 {
    unsafe { libc::geteuid() }
}

/// Bind the abstract-namespace listener and spawn its accept thread once per
/// process. The listener lives exactly as long as the process — abstract
/// sockets vanish when the last fd closes, so there is no stale-endpoint
/// cleanup to do.
pub fn spawn_listener() {
    static SPAWNED: Once = Once::new();
    SPAWNED.call_once(|| {
        let name = socket_name();
        match bind(&name) {
            Ok(listener) => {
                info!("Settings ticket IPC listening on abstract socket @{name}");
                if let Err(error) = std::thread::Builder::new()
                    .name("settings-ipc".to_string())
                    .spawn(move || accept_loop(listener))
                {
                    // Only the --open token path is lost; the config server
                    // itself is unaffected.
                    warn!("Settings IPC accept thread failed to start: {error}");
                }
            }
            // Refuses nothing today: the name is UID-scoped and the
            // single-instance lock already prevents a same-flavor duplicate,
            // so a bind failure is logged and skipped rather than fatal.
            Err(error) => warn!("Could not bind settings IPC socket @{name}: {error}"),
        }
    });
}

fn bind(name: &str) -> std::io::Result<UnixListener> {
    UnixListener::bind_addr(&SocketAddr::from_abstract_name(name)?)
}

fn accept_loop(listener: UnixListener) {
    loop {
        match listener.accept() {
            Ok((stream, _)) => handle_connection(stream),
            // Transient errors (fd pressure, EINTR leftovers): back off a
            // beat instead of spinning; fatal ones surface repeatedly here.
            Err(error) => {
                warn!("Settings IPC accept failed: {error}");
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn handle_connection(mut stream: UnixStream) {
    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
    // Abstract sockets are connectable by any local user; the credential
    // check is the entire access control for this endpoint.
    if !peer_is_same_user(stream.as_raw_fd()) {
        return;
    }
    let Some(request) = read_request_line(&mut stream) else {
        return; // timed out or oversized; dropping closes the connection
    };
    let reply = if request == REQUEST {
        format!(
            "OK {}\n",
            crate::http_auth::issue_settings_ticket("local-cli")
        )
    } else {
        "ERR bad-request\n".to_string()
    };
    let _ = stream.write_all(reply.as_bytes());
}

/// `SO_PEERCRED` reports the connecting process's uid at connect time;
/// compare against our own effective uid (the server may run elevated, so
/// the real uid is not necessarily ours to compare with).
fn peer_is_same_user(fd: i32) -> bool {
    let mut cred: libc::ucred = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let rc = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            std::ptr::addr_of_mut!(cred).cast::<libc::c_void>(),
            &mut len,
        )
    };
    rc == 0 && cred.uid == effective_uid()
}

/// Read one `\n`-terminated line, bounded by [`MAX_REQUEST_LINE`]. Byte-wise
/// on purpose: requests are ~10 bytes and this avoids buffering beyond the
/// bound. `None` on EOF before any byte, timeout, or oversized input.
fn read_request_line(stream: &mut UnixStream) -> Option<String> {
    let mut line = Vec::with_capacity(REQUEST.len());
    let mut byte = [0u8; 1];
    loop {
        match stream.read(&mut byte) {
            Ok(0) => break,
            Ok(_) => {
                if byte[0] == b'\n' {
                    break;
                }
                line.push(byte[0]);
                if line.len() > MAX_REQUEST_LINE {
                    return None;
                }
            }
            Err(_) => return None,
        }
    }
    if line.is_empty() {
        return None;
    }
    Some(String::from_utf8_lossy(&line).trim().to_string())
}

/// Ask the running server for a one-time settings ticket. `None` when no
/// server answers (not running, different flavor or uid) or the handshake
/// fails — callers fall back to printing the plain, unauthenticated URL.
pub fn request_settings_ticket() -> Option<String> {
    request_settings_ticket_via(&socket_name())
}

fn request_settings_ticket_via(name: &str) -> Option<String> {
    let addr = SocketAddr::from_abstract_name(name).ok()?;
    let mut stream = UnixStream::connect_addr(&addr).ok()?;
    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
    stream.write_all(format!("{REQUEST}\n").as_bytes()).ok()?;
    let reply = read_request_line(&mut stream)?;
    let ticket = reply.strip_prefix("OK ")?;
    (!ticket.is_empty() && ticket.bytes().all(|b| b.is_ascii_alphanumeric()))
        .then(|| ticket.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_name_fits_abstract_namespace() {
        // sun_path is 108 bytes including the abstract leading NUL.
        assert!(socket_name().len() + 1 <= 108);
    }

    #[test]
    fn round_trip_mints_a_single_use_ticket() {
        // Unique per-test-run abstract name so parallel test binaries never
        // collide; abstract sockets also disappear with the process.
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let name = format!("vterm-test-settings-{}-{nonce}", std::process::id());
        let listener = bind(&name).expect("bind test settings socket");
        std::thread::spawn(move || accept_loop(listener));

        let ticket = request_settings_ticket_via(&name).expect("ticket from IPC");
        // The minted ticket must be a real one-time settings grant: the
        // first exchange yields a session, a replay must fail.
        let access = crate::http_auth::settings_access();
        assert!(access.exchange_grant(&ticket).is_some());
        assert!(access.exchange_grant(&ticket).is_none());
    }

    #[test]
    fn bad_request_line_is_rejected() {
        let name = format!("vterm-test-settings-{}-bad", std::process::id());
        let listener = bind(&name).expect("bind test settings socket");
        std::thread::spawn(move || accept_loop(listener));

        let addr = SocketAddr::from_abstract_name(&name).unwrap();
        let mut stream = UnixStream::connect_addr(&addr).unwrap();
        stream.write_all(b"GIMME-TOKEN\n").unwrap();
        let mut reply = String::new();
        let _ = stream.read_to_string(&mut reply);
        assert_eq!(reply, "ERR bad-request\n");
        // And the rejected path never turned into a ticket.
        assert!(request_settings_ticket_via(&name).is_some());
    }
}
