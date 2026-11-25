//! UDP media relay allocator (screen-streaming spec §7).
//!
//! The relay side of a screen-streaming session: each pairing owns one
//! public UDP port (from the configured range) and one dedicated outbound
//! socket toward the screen-server's UDP port on the shared server's host
//! (`target_host:target_port`). The dedicated outbound socket gives every
//! pairing its own source port at the screen-server, which keys its media
//! sessions by source address — so per-client session isolation survives
//! relaying without any wire-format change.
//!
//! The allocator never parses the forwarded datagrams; the screen-streaming
//! token travels inside the client's handshake packet untouched.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::net::UdpSocket;
use tokio::task::JoinHandle;
use tracing::{info, warn};
use uuid::Uuid;

/// Pairings idle out on the same 4-second silence rule the screen-server
/// applies to its UDP sessions, so dead clients free relay ports without
/// extra signaling (spec §7.4). Applies only once the pairing is pinned to
/// a client; before the first datagram [`RELAY_UDP_FIRST_PACKET_TIMEOUT`]
/// governs.
pub const RELAY_UDP_IDLE_TIMEOUT: Duration = Duration::from_secs(4);

/// Grace period for a pairing no client datagram has reached yet. The 4s
/// idle rule must not run before the first packet: between
/// `ScreenStreamReady` and the first media datagram the client may
/// legitimately take seconds (control-plane handshake, a stalled UI turn),
/// and reaping then kills the stream before it ever starts — observed as a
/// guaranteed first-attempt handshake timeout whenever the client needed
/// >4s to connect. 60s matches the screen-server's token TTL.
const RELAY_UDP_FIRST_PACKET_TIMEOUT: Duration = Duration::from_secs(60);

const UDP_BUFFER_SIZE: usize = 64 * 1024;

/// A live pairing: one public port forwarding to one screen-server target.
#[derive(Debug)]
pub struct AllocatedPairing {
    /// Opaque pairing id echoed in `ScreenStreamReady.endpoint` for logging.
    pub pairing: String,
    /// The public UDP port clients send their media datagrams to.
    pub port: u16,
}

struct PairingEntry {
    last_active: Arc<Mutex<Instant>>,
    task: JoinHandle<()>,
}

/// Allocates and reaps per-pairing UDP forwarding loops. Cheap to clone;
/// all clones share the same pairing table.
#[derive(Clone)]
pub struct RelayUdpAllocator {
    port_start: u16,
    port_end: u16,
    idle_timeout: Duration,
    first_packet_timeout: Duration,
    pairings: Arc<Mutex<HashMap<String, PairingEntry>>>,
}

impl RelayUdpAllocator {
    pub fn new(port_start: u16, port_end: u16) -> Self {
        Self {
            port_start,
            port_end,
            idle_timeout: RELAY_UDP_IDLE_TIMEOUT,
            first_packet_timeout: RELAY_UDP_FIRST_PACKET_TIMEOUT,
            pairings: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Test constructor with a custom idle timeout; the first-packet grace
    /// is pinned to the same value so idle-reaper tests stay fast.
    #[cfg(test)]
    fn with_idle_timeout(port_start: u16, port_end: u16, idle_timeout: Duration) -> Self {
        Self {
            port_start,
            port_end,
            idle_timeout,
            first_packet_timeout: idle_timeout,
            pairings: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Number of live pairings (tests and diagnostics).
    pub fn len(&self) -> usize {
        self.pairings.lock().unwrap().len()
    }

    /// Bind a public port in the configured range and start the forwarding
    /// loop toward `target_host:target_port`.
    pub async fn allocate(
        &self,
        target_host: &str,
        target_port: u16,
    ) -> anyhow::Result<AllocatedPairing> {
        // `target_host` is the shared server's TCP peer address, which may
        // be IPv6 — including IPv4-mapped IPv6 (`::ffff:a.b.c.d`) when the
        // relay's listener is dual-stack. Silently falling back to
        // 127.0.0.1 here would forward media into the void, so fail loudly
        // on unparseable input and normalize mapped addresses to IPv4.
        let target_ip: std::net::IpAddr = target_host
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid screen-server target IP {target_host:?}"))?;
        let target_ip = match target_ip {
            std::net::IpAddr::V6(v6) => v6
                .to_ipv4_mapped()
                .map(std::net::IpAddr::V4)
                .unwrap_or(std::net::IpAddr::V6(v6)),
            v4 => v4,
        };
        let target = SocketAddr::new(target_ip, target_port);
        let mut last_error = None;
        for port in self.port_start..=self.port_end {
            // The public socket must be dual-stack so IPv4 (`127.0.0.1`) and
            // IPv6 (`::1`) clients can both reach it — `localhost` may
            // resolve to either family on Windows.
            match bind_tuned_udp_socket_dualstack(port) {
                Ok(public) => {
                    // The outbound socket's family must match the target:
                    // an IPv4-only socket cannot send to an IPv6
                    // screen-server. The dual-stack bind (V6ONLY=false)
                    // reaches both families, so use it for IPv6 targets.
                    let outbound = match target_ip {
                        std::net::IpAddr::V4(_) => {
                            bind_tuned_udp_socket(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?
                        }
                        std::net::IpAddr::V6(_) => bind_tuned_udp_socket_dualstack(0)?,
                    };
                    let pairing = Uuid::new_v4().to_string();
                    let last_active = Arc::new(Mutex::new(Instant::now()));
                    let task = tokio::spawn(run_pairing(
                        pairing.clone(),
                        public,
                        outbound,
                        target,
                        Arc::clone(&last_active),
                        self.idle_timeout,
                        self.first_packet_timeout,
                        Arc::clone(&self.pairings),
                    ));
                    self.pairings
                        .lock()
                        .unwrap()
                        .insert(pairing.clone(), PairingEntry { last_active, task });
                    info!(
                        "Relay UDP pairing {} allocated on port {} toward {}",
                        pairing, port, target
                    );
                    return Ok(AllocatedPairing { pairing, port });
                }
                Err(error) => {
                    last_error = Some(error);
                }
            }
        }
        Err(anyhow::anyhow!(
            "no free UDP port in relay range {}-{}: {:?}",
            self.port_start,
            self.port_end,
            last_error
        ))
    }

    /// Release one pairing (explicit teardown). Returns `true` when it
    /// existed. Idempotent with the idle reaper.
    pub fn release(&self, pairing: &str) -> bool {
        let entry = self.pairings.lock().unwrap().remove(pairing);
        if let Some(entry) = entry {
            entry.task.abort();
            info!("Relay UDP pairing {} released", pairing);
            true
        } else {
            false
        }
    }

    /// Release a batch of pairings (shared-server unregister).
    pub fn release_all(&self, pairings: impl IntoIterator<Item = String>) {
        for pairing in pairings {
            self.release(&pairing);
        }
    }
}

/// Bind a UDP socket with the same buffer tuning as the screen-server's
/// media sockets (`vterm-screen-common/src/server.rs`): no SO_REUSEADDR,
/// enlarged recv/send buffers.
fn bind_tuned_udp_socket(addr: SocketAddrV4) -> std::io::Result<UdpSocket> {
    let socket = socket2::Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;
    socket.bind(&addr.into())?;
    let _ = socket.set_recv_buffer_size(1024 * 1024);
    let _ = socket.set_send_buffer_size(4 * 1024 * 1024);
    socket.set_nonblocking(true)?;
    UdpSocket::from_std(socket.into())
}

/// Bind a **dual-stack** UDP socket (IPv6 with `IPV6_V6ONLY=false`) so it
/// accepts datagrams from both IPv4 (`127.0.0.1`) and IPv6 (`::1`) clients.
/// On Windows an IPv4-only bind cannot receive IPv6 loopback traffic, and
/// `localhost` may resolve to either family.
fn bind_tuned_udp_socket_dualstack(port: u16) -> std::io::Result<UdpSocket> {
    let socket = socket2::Socket::new(
        socket2::Domain::IPV6,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;
    // Allow IPv4-mapped addresses (dual-stack).
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let v6only: std::ffi::c_int = 0;
        unsafe {
            libc::setsockopt(
                socket.as_raw_fd(),
                libc::IPPROTO_IPV6,
                libc::IPV6_V6ONLY,
                &v6only as *const _ as *const _,
                std::mem::size_of_val(&v6only) as u32,
            );
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawSocket;
        let v6only: u32 = 0;
        unsafe {
            windows_sys::Win32::Networking::WinSock::setsockopt(
                socket.as_raw_socket() as usize,
                windows_sys::Win32::Networking::WinSock::IPPROTO_IPV6 as i32,
                windows_sys::Win32::Networking::WinSock::IPV6_V6ONLY as i32,
                &v6only as *const _ as *const _,
                std::mem::size_of_val(&v6only) as i32,
            );
        }
    }
    let addr = std::net::SocketAddrV6::new(std::net::Ipv6Addr::UNSPECIFIED, port, 0, 0);
    socket.bind(&addr.into())?;
    let _ = socket.set_recv_buffer_size(1024 * 1024);
    let _ = socket.set_send_buffer_size(4 * 1024 * 1024);
    socket.set_nonblocking(true)?;
    UdpSocket::from_std(socket.into())
}

/// Forward datagrams in both directions until both have been silent for
/// `idle_timeout`. The first client datagram pins the pairing to that
/// source address; datagrams from any other address are dropped. Before
/// the pairing is pinned, `first_packet_timeout` applies instead of the
/// short idle rule — the client may still be seconds away from its first
/// datagram right after allocation.
async fn run_pairing(
    pairing: String,
    public: UdpSocket,
    outbound: UdpSocket,
    target: SocketAddr,
    last_active: Arc<Mutex<Instant>>,
    idle_timeout: Duration,
    first_packet_timeout: Duration,
    pairings: Arc<Mutex<HashMap<String, PairingEntry>>>,
) {
    let mut client_addr: Option<SocketAddr> = None;
    let mut in_buf = vec![0u8; UDP_BUFFER_SIZE];
    let mut out_buf = vec![0u8; UDP_BUFFER_SIZE];
    loop {
        let deadline = *last_active.lock().unwrap()
            + if client_addr.is_some() {
                idle_timeout
            } else {
                first_packet_timeout
            };
        tokio::select! {
            result = public.recv_from(&mut in_buf) => {
                match result {
                    Ok((len, src)) => {
                        match client_addr {
                            None => {
                                client_addr = Some(src);
                                info!(
                                    "Relay UDP pairing {} pinned to client {}",
                                    pairing, src
                                );
                            }
                            Some(pinned) if pinned != src => continue,
                            _ => {}
                        }
                        *last_active.lock().unwrap() = Instant::now();
                        if let Err(error) = outbound.send_to(&in_buf[..len], target).await {
                            warn!("Relay UDP pairing {} forward to {} failed: {}", pairing, target, error);
                        }
                    }
                    Err(error) => {
                        warn!("Relay UDP pairing {} public recv failed: {}", pairing, error);
                        break;
                    }
                }
            }
            result = outbound.recv_from(&mut out_buf) => {
                match result {
                    Ok((len, _src)) => {
                        let Some(client) = client_addr else { continue };
                        *last_active.lock().unwrap() = Instant::now();
                        if let Err(error) = public.send_to(&out_buf[..len], client).await {
                            warn!("Relay UDP pairing {} forward to client {} failed: {}", pairing, client, error);
                        }
                    }
                    Err(error) => {
                        warn!("Relay UDP pairing {} outbound recv failed: {}", pairing, error);
                        break;
                    }
                }
            }
            _ = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => {
                if client_addr.is_some() {
                    info!("Relay UDP pairing {} reaped after idle timeout", pairing);
                } else {
                    info!(
                        "Relay UDP pairing {} reaped: no client datagram within {:?}",
                        pairing, first_packet_timeout
                    );
                }
                break;
            }
        }
    }
    pairings.lock().unwrap().remove(&pairing);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UdpSocket as TokioUdpSocket;

    /// A fake screen-server: echoes every datagram back to its sender.
    async fn spawn_echo_server() -> (u16, JoinHandle<()>) {
        let socket = TokioUdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = socket.local_addr().unwrap().port();
        let task = tokio::spawn(async move {
            let mut buf = vec![0u8; UDP_BUFFER_SIZE];
            while let Ok((len, src)) = socket.recv_from(&mut buf).await {
                let _ = socket.send_to(&buf[..len], src).await;
            }
        });
        (port, task)
    }

    #[tokio::test]
    async fn allocate_binds_ports_in_range() {
        let allocator = RelayUdpAllocator::new(46100, 46109);
        let first = allocator.allocate("127.0.0.1", 36459).await.unwrap();
        let second = allocator.allocate("127.0.0.1", 36459).await.unwrap();
        assert!((46100..=46109).contains(&first.port));
        assert!((46100..=46109).contains(&second.port));
        assert_ne!(first.port, second.port);
        assert_ne!(first.pairing, second.pairing);
        assert_eq!(allocator.len(), 2);
        allocator.release(&first.pairing);
        allocator.release(&second.pairing);
    }

    #[tokio::test]
    async fn exhausted_range_errors() {
        let allocator = RelayUdpAllocator::new(46110, 46110);
        let pairing = allocator.allocate("127.0.0.1", 36459).await.unwrap();
        let error = allocator.allocate("127.0.0.1", 36459).await.unwrap_err();
        assert!(
            error.to_string().contains("no free UDP port"),
            "unexpected error: {error}"
        );
        allocator.release(&pairing.pairing);
    }

    #[tokio::test]
    async fn datagrams_flow_both_ways_with_relay_source() {
        let (echo_port, echo_task) = spawn_echo_server().await;
        let allocator = RelayUdpAllocator::new(46120, 46129);
        let pairing = allocator.allocate("127.0.0.1", echo_port).await.unwrap();

        let client = TokioUdpSocket::bind("127.0.0.1:0").await.unwrap();
        let relay_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, pairing.port));
        client.send_to(b"hello", relay_addr).await.unwrap();

        let mut buf = [0u8; 64];
        let (len, src) = tokio::time::timeout(Duration::from_secs(2), client.recv_from(&mut buf))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(&buf[..len], b"hello");
        // The client-side address filter requires replies to come from the
        // exact relay host:pairing_port the client sent to.
        assert_eq!(src, relay_addr);

        allocator.release(&pairing.pairing);
        echo_task.abort();
    }

    #[tokio::test]
    async fn foreign_client_datagrams_are_dropped() {
        let (echo_port, echo_task) = spawn_echo_server().await;
        let allocator = RelayUdpAllocator::new(46130, 46139);
        let pairing = allocator.allocate("127.0.0.1", echo_port).await.unwrap();
        let relay_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, pairing.port));

        let pinned = TokioUdpSocket::bind("127.0.0.1:0").await.unwrap();
        pinned.send_to(b"pinned", relay_addr).await.unwrap();
        let mut buf = [0u8; 64];
        tokio::time::timeout(Duration::from_secs(2), pinned.recv_from(&mut buf))
            .await
            .unwrap()
            .unwrap();

        // A different source port must not receive the echo.
        let foreign = TokioUdpSocket::bind("127.0.0.1:0").await.unwrap();
        foreign.send_to(b"foreign", relay_addr).await.unwrap();
        let result =
            tokio::time::timeout(Duration::from_millis(300), foreign.recv_from(&mut buf)).await;
        assert!(
            result.is_err(),
            "foreign client received a relayed datagram"
        );

        allocator.release(&pairing.pairing);
        echo_task.abort();
    }

    /// A dual-stack TCP listener reports IPv4 peers as IPv4-mapped IPv6
    /// (`::ffff:127.0.0.1`); the pairing must normalize them instead of
    /// falling back to localhost.
    #[tokio::test]
    async fn ipv4_mapped_target_is_normalized() {
        let (echo_port, echo_task) = spawn_echo_server().await;
        let allocator = RelayUdpAllocator::new(46180, 46189);
        let pairing = allocator
            .allocate("::ffff:127.0.0.1", echo_port)
            .await
            .unwrap();

        let client = TokioUdpSocket::bind("127.0.0.1:0").await.unwrap();
        let relay_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, pairing.port));
        client.send_to(b"mapped", relay_addr).await.unwrap();

        let mut buf = [0u8; 64];
        let (len, _) = tokio::time::timeout(Duration::from_secs(2), client.recv_from(&mut buf))
            .await
            .expect("mapped target must reach the IPv4 echo server")
            .unwrap();
        assert_eq!(&buf[..len], b"mapped");

        allocator.release(&pairing.pairing);
        echo_task.abort();
    }

    #[tokio::test]
    async fn invalid_target_host_errors() {
        let allocator = RelayUdpAllocator::new(46190, 46199);
        let error = allocator.allocate("not-an-ip", 36459).await.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("invalid screen-server target IP"),
            "unexpected error: {error}"
        );
    }

    #[tokio::test]
    async fn release_stops_forwarding() {
        let (echo_port, echo_task) = spawn_echo_server().await;
        let allocator = RelayUdpAllocator::new(46140, 46149);
        let pairing = allocator.allocate("127.0.0.1", echo_port).await.unwrap();
        assert!(allocator.release(&pairing.pairing));
        assert!(!allocator.release(&pairing.pairing));
        assert_eq!(allocator.len(), 0);
        // abort() only schedules cancellation; give the runtime a tick to
        // actually drop the task (and close its sockets) before probing.
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = TokioUdpSocket::bind("127.0.0.1:0").await.unwrap();
        let relay_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, pairing.port));
        client.send_to(b"late", relay_addr).await.unwrap();
        let mut buf = [0u8; 64];
        let result =
            tokio::time::timeout(Duration::from_millis(300), client.recv_from(&mut buf)).await;
        // Both "timed out" and "ICMP port unreachable surfaced as a recv
        // error" (Windows turns it into ConnectionReset) prove nothing was
        // forwarded; only an actual datagram means a leak.
        if let Ok(Ok((len, _))) = result {
            panic!("released pairing still forwards ({} bytes)", len);
        }
        echo_task.abort();
    }

    #[tokio::test]
    async fn idle_pairing_is_reaped() {
        let (echo_port, echo_task) = spawn_echo_server().await;
        let allocator =
            RelayUdpAllocator::with_idle_timeout(46150, 46159, Duration::from_millis(200));
        let pairing = allocator.allocate("127.0.0.1", echo_port).await.unwrap();
        assert_eq!(allocator.len(), 1);

        // Keep polling until the reaper removes the pairing.
        let deadline = Instant::now() + Duration::from_secs(3);
        while allocator.len() > 0 {
            assert!(Instant::now() < deadline, "pairing was not reaped");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(!allocator.release(&pairing.pairing));
        echo_task.abort();
    }

    #[tokio::test]
    async fn release_all_drains_pairings() {
        let allocator = RelayUdpAllocator::new(46160, 46169);
        let first = allocator.allocate("127.0.0.1", 36459).await.unwrap();
        let second = allocator.allocate("127.0.0.1", 36459).await.unwrap();
        allocator.release_all(vec![first.pairing, second.pairing]);
        assert_eq!(allocator.len(), 0);
    }

}
