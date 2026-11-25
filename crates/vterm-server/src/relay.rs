use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot, watch, RwLock};
use tokio_tungstenite::tungstenite::Message as WebSocketMessage;
use tokio_tungstenite::{client_async_tls_with_config, Connector, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use vterm_common::{
    secret_key_challenge_response, verify_secret_key_challenge_response, ErrorCode, KnownAppInfo,
    RecentProjectDirectory, RelayServerInfo, SessionId, SessionInfo, TabGroup, TabId,
    TabInfo as ProtocolTabInfo, TabSplitPlacement, TerminalCommandHistory, TerminalLayout,
    TerminalProfile, TerminalSize, MAX_MESSAGE_SIZE, PROTOCOL_VERSION, TAB_GROUP_ALL_ID,
    TAB_GROUP_UNGROUPED_ID,
};
use vterm_core::crypto::{MessageCrypto, MessageCryptoHandshake};
use vterm_core::protocol::{ConnectionKind, Message, ServerInfo, TransportBinaryFrameDecoder};

use crate::config::ConfigManager;
use crate::connection::ConnectionRegistry;
use crate::local_network::{self, LocalNetworkBlockTracker};
use crate::port_forward::PortForwardManager;
use crate::relay_udp::RelayUdpAllocator;
use crate::session::{ConnectionTx, SessionError, SessionManager};

/// Delay before the first retry of a failed upstream relay connection.
/// Doubles after each failed attempt up to RELAY_RECONNECT_MAX_INTERVAL.
const RELAY_RECONNECT_INITIAL_INTERVAL: Duration = Duration::from_secs(5);
/// Upper bound for the upstream reconnect backoff.
const RELAY_RECONNECT_MAX_INTERVAL: Duration = Duration::from_secs(60);
/// A connection that stayed up at least this long counts as healthy and
/// resets the backoff to RELAY_RECONNECT_INITIAL_INTERVAL.
const RELAY_RECONNECT_RESET_AFTER: Duration = Duration::from_secs(60);
const RELAY_SYNC_INTERVAL: Duration = Duration::from_secs(3);
/// Interval at which the upstream relay connection sends a WebSocket ping.
/// Both peers run tokio-tungstenite, which answers pings automatically.
const RELAY_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// Reconnect when the upstream relay produced no inbound frames — including
/// heartbeat pongs — for this long; a half-open relay link otherwise goes
/// unnoticed and leaves this server's viewer registrations on remote
/// sessions alive.
const RELAY_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
/// While a macOS Local Network permission block is suspected, retry at this
/// fixed interval instead of the exponential backoff: the block clears the
/// moment the user re-toggles the permission, and the reconnect should feel
/// immediate.
const RELAY_LOCAL_NETWORK_RETRY_INTERVAL: Duration = Duration::from_secs(5);
const RELAY_TERMINAL_CAPABILITY_PREFIX: &str = "rtc1";
const RELAY_REQUEST_CAPABILITY_PREFIX: &str = "rrc1";
const RELAY_TERMINAL_CAPABILITY_LIFETIME_SECS: u64 = 300;
const RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS: u64 = 12 * 60 * 60;

fn relay_capability_time_is_valid(expires_at: u64, now: u64) -> bool {
    let issued_at = expires_at.saturating_sub(RELAY_TERMINAL_CAPABILITY_LIFETIME_SECS);
    now.abs_diff(issued_at) <= RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS
}

fn unix_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn session_id_hex(session_id: SessionId) -> String {
    let mut encoded = String::with_capacity(32);
    for byte in session_id.as_bytes() {
        use std::fmt::Write as _;
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn create_relay_terminal_capability(
    secret_key: &str,
    session_id: SessionId,
    expires_at: u64,
    nonce: &str,
) -> String {
    let signed_payload = format!(
        "vterm-relay-terminal-v1:{}:{expires_at}:{nonce}",
        session_id_hex(session_id)
    );
    let signature = secret_key_challenge_response(secret_key, &signed_payload);
    format!("{RELAY_TERMINAL_CAPABILITY_PREFIX}:{expires_at}:{nonce}:{signature}")
}

fn verify_relay_terminal_capability(
    secret_key: &str,
    session_id: SessionId,
    capability: &str,
    now: u64,
) -> Option<u64> {
    let mut parts = capability.split(':');
    if parts.next()? != RELAY_TERMINAL_CAPABILITY_PREFIX {
        return None;
    }
    let expires_at = parts.next()?.parse::<u64>().ok()?;
    let nonce = parts.next()?;
    let signature = parts.next()?;
    if parts.next().is_some()
        || nonce.is_empty()
        || !relay_capability_time_is_valid(expires_at, now)
    {
        return None;
    }
    let signed_payload = format!(
        "vterm-relay-terminal-v1:{}:{expires_at}:{nonce}",
        session_id_hex(session_id)
    );
    let expected = secret_key_challenge_response(secret_key, &signed_payload);
    verify_secret_key_challenge_response(&expected, Some(signature)).then_some(expires_at)
}

fn create_relay_request_capability(
    secret_key: &str,
    server_id: &str,
    expires_at: u64,
    nonce: &str,
) -> String {
    let signed_payload = format!("vterm-relay-request-v1:{server_id}:{expires_at}:{nonce}");
    let signature = secret_key_challenge_response(secret_key, &signed_payload);
    format!("{RELAY_REQUEST_CAPABILITY_PREFIX}:{expires_at}:{nonce}:{server_id}:{signature}")
}

fn verify_relay_request_capability(
    secret_key: &str,
    expected_server_id: &str,
    capability: &str,
    now: u64,
) -> Option<u64> {
    let mut parts = capability.split(':');
    if parts.next()? != RELAY_REQUEST_CAPABILITY_PREFIX {
        return None;
    }
    let expires_at = parts.next()?.parse::<u64>().ok()?;
    let nonce = parts.next()?;
    let server_id = parts.next()?;
    let signature = parts.next()?;
    if parts.next().is_some()
        || nonce.is_empty()
        || server_id != expected_server_id
        || !relay_capability_time_is_valid(expires_at, now)
    {
        return None;
    }
    let signed_payload = format!("vterm-relay-request-v1:{server_id}:{expires_at}:{nonce}");
    let expected = secret_key_challenge_response(secret_key, &signed_payload);
    verify_secret_key_challenge_response(&expected, Some(signature)).then_some(expires_at)
}

fn unique_tab_ids(tab_ids: impl Iterator<Item = TabId>) -> Vec<TabId> {
    let mut tab_ids = tab_ids
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    tab_ids.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    tab_ids
}

fn merge_global_all_group(groups: Vec<TabGroup>) -> Vec<TabGroup> {
    let all_tab_ids = unique_tab_ids(
        groups
            .iter()
            .filter(|group| group.id == TAB_GROUP_ALL_ID)
            .flat_map(|group| group.tab_ids.iter().copied()),
    );

    let mut merged = Vec::with_capacity(groups.len().saturating_add(1));
    merged.push(TabGroup {
        server_id: None,
        id: TAB_GROUP_ALL_ID,
        name: "All".to_string(),
        builtin: true,
        tab_ids: all_tab_ids,
    });
    merged.extend(
        groups
            .into_iter()
            .filter(|group| group.id != TAB_GROUP_ALL_ID),
    );
    merged
}

struct SharedServerState {
    id: String,
    name: String,
    request_url: Option<String>,
    terminal_tunnel_key: String,
    primary_tx: ConnectionTx,
    /// The shared server's reachable IP address (from the relay's TCP
    /// connection), used as the target host for UDP and TCP port forwarding.
    peer_ip: std::net::IpAddr,
    sessions: HashMap<SessionId, SessionInfo>,
    /// Authoritative tab layouts reported by the shared server (tab_id →
    /// layout). Kept separate from `sessions` so a layout change that does not
    /// alter the session list still propagates to clients (e.g. merging a
    /// shared tab into another keeps the split layout).
    tab_layouts: HashMap<TabId, TerminalLayout>,
    tab_groups: Vec<TabGroup>,
    profiles: Vec<TerminalProfile>,
    recent_project_directories: Vec<RecentProjectDirectory>,
    terminal_command_history: Vec<TerminalCommandHistory>,
    known_apps: Vec<KnownAppInfo>,
    server_info: Option<ServerInfo>,
    pending_creates: VecDeque<ConnectionTx>,
    /// Relay-role UDP pairings allocated on behalf of this shared server
    /// (screen streaming, spec §7). Reaped in bulk on unregister.
    udp_pairings: HashSet<String>,
}

struct RelayState {
    clients: Vec<ConnectionTx>,
    servers: HashMap<String, SharedServerState>,
    remote_session_index: HashMap<SessionId, String>,
    consumed_terminal_capabilities: HashMap<String, u64>,
    upstream_terminal_tunnel_key: Option<String>,
    /// Shared-role: sender toward the upstream relay's RelayPrimary channel
    /// while `run_single_upstream_connection` is alive.
    upstream_relay_tx: Option<ConnectionTx>,
    /// Shared-role: host portion of the configured upstream relay address,
    /// advertised to clients as the media endpoint host for relay pairings.
    upstream_relay_host: Option<String>,
}

/// Outcome of a `RelayAllocateUdpPairing` request: (pairing, host, port).
type UdpPairingOutcome = Result<(String, String, u16), String>;

/// Outcome of a `RelayScreenTokenRequest`: (token_hex, screen_udp_port).
type ScreenTokenOutcome = Result<(String, u16), String>;

/// Outcome of a `RelayScreenSourcesRequest`.
type ScreenSourcesOutcome =
    Result<(Option<u64>, Vec<vterm_core::protocol::ScreenSourceInfo>), String>;

fn sorted_shared_servers(state: &RelayState) -> Vec<&SharedServerState> {
    let mut servers = state.servers.values().collect::<Vec<_>>();
    servers.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.id.cmp(&right.id))
    });
    servers
}

fn sort_session_infos(sessions: &mut [SessionInfo]) {
    sessions.sort_by(|left, right| {
        left.sequence
            .cmp(&right.sequence)
            .then_with(|| left.id.as_bytes().cmp(right.id.as_bytes()))
    });
}

fn shared_server_tab_infos(server: &SharedServerState) -> Vec<ProtocolTabInfo> {
    let mut tab_sessions: HashMap<TabId, Vec<SessionInfo>> = HashMap::new();
    for session in server.sessions.values() {
        tab_sessions
            .entry(session.tab_id)
            .or_default()
            .push(session.clone());
    }

    let mut tabs = tab_sessions
        .into_iter()
        .map(|(tab_id, mut sessions)| {
            sort_session_infos(&mut sessions);
            let primary_id = sessions
                .first()
                .map(|session| session.id)
                .unwrap_or_default();
            // Prefer the shared server's authoritative layout for this tab;
            // fall back to a plain terminal (its primary session).
            let layout =
                server
                    .tab_layouts
                    .get(&tab_id)
                    .cloned()
                    .unwrap_or(TerminalLayout::Terminal {
                        session_id: primary_id,
                    });
            ProtocolTabInfo {
                server_id: Some(server.id.clone()),
                tab_id,
                sessions,
                layout,
            }
        })
        .collect::<Vec<_>>();
    tabs.sort_by(|left, right| {
        let left_sequence = left
            .sessions
            .first()
            .map(|session| session.sequence)
            .unwrap_or(u64::MAX);
        let right_sequence = right
            .sessions
            .first()
            .map(|session| session.sequence)
            .unwrap_or(u64::MAX);
        left_sequence
            .cmp(&right_sequence)
            .then_with(|| left.tab_id.as_bytes().cmp(right.tab_id.as_bytes()))
    });
    tabs
}

pub struct RelayManager {
    config_manager: Arc<ConfigManager>,
    session_manager: Arc<SessionManager>,
    screen_supervisor: Arc<crate::screen::ScreenServerSupervisor>,
    port_forward_manager: Arc<PortForwardManager>,
    connection_registry: Arc<ConnectionRegistry>,
    state: RwLock<RelayState>,
    local_state_revision: watch::Sender<u64>,
    /// Relay-role UDP pairing allocator, built lazily from the configured
    /// port range on the first allocation (screen streaming, spec §7).
    udp_allocator: tokio::sync::OnceCell<RelayUdpAllocator>,
    /// Shared-role pending `RelayAllocateUdpPairing` requests, keyed by the
    /// protocol `request_id` and completed by `RelayUdpPairingAllocated`.
    pending_udp_allocations: std::sync::Mutex<HashMap<u64, oneshot::Sender<UdpPairingOutcome>>>,
    next_udp_request_id: std::sync::atomic::AtomicU64,
    /// Shared-role pairings grouped by client label (connection peer
    /// address) for bulk release on connection teardown.
    udp_pairings_by_label: std::sync::Mutex<HashMap<String, HashSet<String>>>,
    /// Relay-role pending `RelayScreenTokenRequest` results, keyed by
    /// `request_id`, completed by `RelayScreenTokenResponse` arriving from
    /// a shared server.
    pending_screen_tokens: std::sync::Mutex<HashMap<u64, oneshot::Sender<ScreenTokenOutcome>>>,
    /// Relay-role pending `RelayScreenSourcesRequest` results.
    pending_screen_sources: std::sync::Mutex<HashMap<u64, oneshot::Sender<ScreenSourcesOutcome>>>,
    next_screen_request_id: std::sync::atomic::AtomicU64,
    /// Shared-role outer requests waiting for the final relay proxy port.
    pending_shared_port_forwards:
        std::sync::Mutex<HashMap<(SessionId, u16), oneshot::Sender<Result<u16, String>>>>,
    /// Relay-role inner requests waiting for each shared server's proxy port.
    pending_relay_port_forwards: std::sync::Mutex<HashMap<(String, u16), VecDeque<SessionId>>>,
}

impl RelayManager {
    pub fn new(
        config_manager: Arc<ConfigManager>,
        session_manager: Arc<SessionManager>,
        screen_supervisor: Arc<crate::screen::ScreenServerSupervisor>,
        port_forward_manager: Arc<PortForwardManager>,
        connection_registry: Arc<ConnectionRegistry>,
    ) -> Self {
        let (local_state_revision, _) = watch::channel(0);
        Self {
            config_manager,
            session_manager,
            screen_supervisor,
            port_forward_manager,
            connection_registry,
            state: RwLock::new(RelayState {
                clients: Vec::new(),
                servers: HashMap::new(),
                remote_session_index: HashMap::new(),
                consumed_terminal_capabilities: HashMap::new(),
                upstream_terminal_tunnel_key: None,
                upstream_relay_tx: None,
                upstream_relay_host: None,
            }),
            local_state_revision,
            udp_allocator: tokio::sync::OnceCell::new(),
            pending_udp_allocations: std::sync::Mutex::new(HashMap::new()),
            next_udp_request_id: std::sync::atomic::AtomicU64::new(1),
            udp_pairings_by_label: std::sync::Mutex::new(HashMap::new()),
            pending_screen_tokens: std::sync::Mutex::new(HashMap::new()),
            pending_screen_sources: std::sync::Mutex::new(HashMap::new()),
            next_screen_request_id: std::sync::atomic::AtomicU64::new(1),
            pending_shared_port_forwards: std::sync::Mutex::new(HashMap::new()),
            pending_relay_port_forwards: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Relay role: the UDP pairing allocator, initialized from the
    /// configured `server.relay_udp_port_start/end` range on first use.
    pub async fn udp_allocator(&self) -> &RelayUdpAllocator {
        self.udp_allocator
            .get_or_init(|| async {
                let config = self.config_manager.snapshot().await;
                RelayUdpAllocator::new(
                    config.server.relay_udp_port_start,
                    config.server.relay_udp_port_end,
                )
            })
            .await
    }

    /// Shared role: ask the upstream relay to allocate a UDP pairing toward
    /// the local screen-server's `target_port`. Resolves with
    /// `(pairing, host, port)`; `host` is whatever the relay reported — the
    /// caller should prefer [`Self::upstream_relay_host`] when set.
    pub async fn allocate_upstream_udp_pairing(
        &self,
        target_port: u16,
    ) -> anyhow::Result<UdpPairingOutcome> {
        let relay_tx = {
            let state = self.state.read().await;
            state.upstream_relay_tx.clone()
        }
        .ok_or_else(|| anyhow::anyhow!("no upstream relay connection"))?;
        let request_id = self
            .next_udp_request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending_udp_allocations
            .lock()
            .unwrap()
            .insert(request_id, tx);
        if relay_tx
            .send(Message::RelayAllocateUdpPairing {
                request_id,
                target_port,
            })
            .is_err()
        {
            self.pending_udp_allocations
                .lock()
                .unwrap()
                .remove(&request_id);
            anyhow::bail!("upstream relay connection closed");
        }
        match tokio::time::timeout(Duration::from_secs(10), rx).await {
            Ok(Ok(outcome)) => Ok(outcome),
            Ok(Err(_closed)) => anyhow::bail!("upstream relay connection closed"),
            Err(_) => {
                self.pending_udp_allocations
                    .lock()
                    .unwrap()
                    .remove(&request_id);
                anyhow::bail!("timed out waiting for relay UDP pairing allocation")
            }
        }
    }

    /// Shared role: release a pairing previously allocated from the
    /// upstream relay. Fire-and-forget; the relay also reaps idle pairings.
    pub async fn release_upstream_udp_pairing(&self, pairing: String) {
        let relay_tx = {
            let state = self.state.read().await;
            state.upstream_relay_tx.clone()
        };
        if let Some(relay_tx) = relay_tx {
            let _ = relay_tx.send(Message::RelayReleaseUdpPairing { pairing });
        }
    }

    /// Shared role: host portion of the upstream relay address clients
    /// should target for relayed media UDP.
    pub async fn upstream_relay_host(&self) -> Option<String> {
        let state = self.state.read().await;
        state.upstream_relay_host.clone()
    }

    /// Check if this server is running as a shared server (has an upstream relay connection).
    pub async fn is_shared_server(&self) -> bool {
        let state = self.state.read().await;
        state.upstream_relay_tx.is_some()
    }

    /// Shared role: ask the upstream relay to coordinate a two-stage forward
    /// for a static HTTP server and return the final relay proxy port.
    pub async fn request_shared_port_forward_and_wait(
        &self,
        session_id: SessionId,
        target_port: u16,
    ) -> anyhow::Result<u16> {
        let relay_tx = {
            let state = self.state.read().await;
            state.upstream_relay_tx.clone()
        }
        .ok_or_else(|| anyhow::anyhow!("no upstream relay connection"))?;

        let key = (session_id, target_port);
        let (tx, rx) = oneshot::channel();
        self.pending_shared_port_forwards
            .lock()
            .unwrap()
            .insert(key, tx);

        if relay_tx
            .send(Message::RequestSharedPortForward {
                session_id,
                target_port,
                auth_token: None,
            })
            .is_err()
        {
            self.pending_shared_port_forwards
                .lock()
                .unwrap()
                .remove(&key);
            anyhow::bail!("upstream relay connection closed");
        }

        info!(
            "Requested shared port forward for session {:?} target_port={}, waiting for relay response",
            session_id, target_port
        );

        match tokio::time::timeout(Duration::from_secs(10), rx).await {
            Ok(Ok(result)) => result.map_err(|e| anyhow::anyhow!(e)),
            Ok(Err(_)) => {
                self.pending_shared_port_forwards
                    .lock()
                    .unwrap()
                    .remove(&key);
                anyhow::bail!("relay connection closed while waiting for port forward response")
            }
            Err(_) => {
                self.pending_shared_port_forwards
                    .lock()
                    .unwrap()
                    .remove(&key);
                anyhow::bail!("timed out waiting for shared port forward response")
            }
        }
    }

    fn complete_shared_port_forward(
        &self,
        session_id: SessionId,
        target_port: u16,
        result: Result<u16, String>,
    ) {
        if let Some(tx) = self
            .pending_shared_port_forwards
            .lock()
            .unwrap()
            .remove(&(session_id, target_port))
        {
            let _ = tx.send(result);
        } else {
            warn!(
                "Received SharedPortForwardReady for unknown session {:?} target_port {}",
                session_id, target_port
            );
        }
    }

    fn add_pending_relay_port_forward(
        &self,
        server_id: &str,
        target_port: u16,
        session_id: SessionId,
    ) {
        self.pending_relay_port_forwards
            .lock()
            .unwrap()
            .entry((server_id.to_string(), target_port))
            .or_default()
            .push_back(session_id);
    }

    fn take_pending_relay_port_forward(
        &self,
        server_id: &str,
        target_port: u16,
    ) -> Option<SessionId> {
        let key = (server_id.to_string(), target_port);
        let mut pending = self.pending_relay_port_forwards.lock().unwrap();
        let session_id = pending.get_mut(&key)?.pop_front();
        if pending.get(&key).is_some_and(VecDeque::is_empty) {
            pending.remove(&key);
        }
        session_id
    }

    fn remove_pending_relay_port_forward(
        &self,
        server_id: &str,
        target_port: u16,
        session_id: SessionId,
    ) -> bool {
        let key = (server_id.to_string(), target_port);
        let mut pending = self.pending_relay_port_forwards.lock().unwrap();
        let Some(queue) = pending.get_mut(&key) else {
            return false;
        };
        let original_len = queue.len();
        queue.retain(|pending_session_id| *pending_session_id != session_id);
        let removed = queue.len() != original_len;
        if queue.is_empty() {
            pending.remove(&key);
        }
        removed
    }

    /// Complete a pending shared-role allocation request. Called from
    /// `handle_upstream_command` on `RelayUdpPairingAllocated`.
    fn complete_udp_pairing_allocation(&self, request_id: u64, outcome: UdpPairingOutcome) {
        let tx = self
            .pending_udp_allocations
            .lock()
            .unwrap()
            .remove(&request_id);
        if let Some(tx) = tx {
            let _ = tx.send(outcome);
        } else {
            warn!(
                "Relay UDP pairing allocation response for unknown request {}",
                request_id
            );
        }
    }

    /// Shared role: associate an allocated pairing with a client label (the
    /// screen-stream grant label, i.e. the connection's peer address).
    /// Request messages are handled by short-lived child connections whose
    /// per-connection stream maps do not outlive the request, so pairing
    /// teardown is label-based — mirroring the token model's bulk
    /// `revoke_client(label)` (spec §6/§7.4).
    pub fn track_upstream_udp_pairing(&self, label: &str, pairing: String) {
        self.udp_pairings_by_label
            .lock()
            .unwrap()
            .entry(label.to_string())
            .or_default()
            .insert(pairing);
    }

    /// Shared role: stop tracking a single pairing (explicit StopScreenStream
    /// release); teardown of the underlying relay pairing is the caller's
    /// job via [`Self::release_upstream_udp_pairing`].
    pub fn untrack_upstream_udp_pairing(&self, label: &str, pairing: &str) {
        let mut map = self.udp_pairings_by_label.lock().unwrap();
        if let Some(pairings) = map.get_mut(label) {
            pairings.remove(pairing);
            if pairings.is_empty() {
                map.remove(label);
            }
        }
    }

    /// Shared role: release every pairing tracked for a client label
    /// (connection teardown). No-op when the label has none.
    pub async fn release_upstream_udp_pairings_for_label(&self, label: &str) {
        let pairings = self
            .udp_pairings_by_label
            .lock()
            .unwrap()
            .remove(label)
            .unwrap_or_default();
        for pairing in pairings {
            self.release_upstream_udp_pairing(pairing).await;
        }
    }

    // ── Screen-stream relay coordination (relay-role) ───────────────────
    //
    // The relay (localhost server) coordinates screen streaming for shared
    // servers: it allocates a UDP port locally and asks the shared server
    // only for a token + the screen-server's UDP port. This avoids
    // transparent-frame tunnels for screen requests.

    /// Resolve a shared server's reachable IP (for UDP forwarding target).
    pub async fn shared_server_peer_ip(&self, server_id: &str) -> Option<std::net::IpAddr> {
        let state = self.state.read().await;
        state.servers.get(server_id).map(|s| s.peer_ip)
    }

    /// Ask a shared server for a screen-stream token + screen-server UDP port.
    /// Returns `(token_hex, screen_udp_port)` on success.
    pub async fn request_screen_token(
        &self,
        server_id: &str,
        source: &str,
        client_label: &str,
    ) -> anyhow::Result<ScreenTokenOutcome> {
        let server_tx = {
            let state = self.state.read().await;
            let known_ids: Vec<&str> = state.servers.keys().map(|s| s.as_str()).collect();
            match state.servers.get(server_id) {
                Some(server) => {
                    info!(
                        "request_screen_token: found shared server {} (peer_ip={}), known servers: {:?}",
                        server_id, server.peer_ip, known_ids
                    );
                    Some(server.primary_tx.clone())
                }
                None => {
                    warn!(
                        "request_screen_token: unknown shared server {}, known servers: {:?}",
                        server_id, known_ids
                    );
                    None
                }
            }
        }
        .ok_or_else(|| anyhow::anyhow!("unknown shared server {server_id}"))?;

        let request_id = self
            .next_screen_request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending_screen_tokens
            .lock()
            .unwrap()
            .insert(request_id, tx);
        info!(
            "request_screen_token: sending RelayScreenTokenRequest request_id={} to shared server {}",
            request_id, server_id
        );
        if server_tx
            .send(Message::RelayScreenTokenRequest {
                request_id,
                source: source.to_string(),
                client_label: client_label.to_string(),
            })
            .is_err()
        {
            self.pending_screen_tokens
                .lock()
                .unwrap()
                .remove(&request_id);
            anyhow::bail!("shared server connection closed");
        }
        match tokio::time::timeout(Duration::from_secs(10), rx).await {
            Ok(Ok(outcome)) => Ok(outcome),
            Ok(Err(_)) => anyhow::bail!("shared server connection closed"),
            Err(_) => {
                self.pending_screen_tokens
                    .lock()
                    .unwrap()
                    .remove(&request_id);
                anyhow::bail!("timed out waiting for screen token from shared server")
            }
        }
    }

    /// Ask a shared server for its streamable source list.
    pub async fn request_screen_sources(
        &self,
        server_id: &str,
    ) -> anyhow::Result<ScreenSourcesOutcome> {
        let server_tx = {
            let state = self.state.read().await;
            state.servers.get(server_id).map(|s| s.primary_tx.clone())
        }
        .ok_or_else(|| anyhow::anyhow!("unknown shared server {server_id}"))?;

        let request_id = self
            .next_screen_request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending_screen_sources
            .lock()
            .unwrap()
            .insert(request_id, tx);
        if server_tx
            .send(Message::RelayScreenSourcesRequest { request_id })
            .is_err()
        {
            self.pending_screen_sources
                .lock()
                .unwrap()
                .remove(&request_id);
            anyhow::bail!("shared server connection closed");
        }
        match tokio::time::timeout(Duration::from_secs(10), rx).await {
            Ok(Ok(outcome)) => Ok(outcome),
            Ok(Err(_)) => anyhow::bail!("shared server connection closed"),
            Err(_) => {
                self.pending_screen_sources
                    .lock()
                    .unwrap()
                    .remove(&request_id);
                anyhow::bail!("timed out waiting for screen sources from shared server")
            }
        }
    }

    /// Tell a shared server to revoke a screen-stream token (fire-and-forget).
    pub async fn revoke_screen_token(&self, server_id: &str, token: &str) {
        let server_tx = {
            let state = self.state.read().await;
            state.servers.get(server_id).map(|s| s.primary_tx.clone())
        };
        if let Some(tx) = server_tx {
            let _ = tx.send(Message::RelayScreenTokenRevoke {
                token: token.to_string(),
            });
        }
    }

    /// Complete a pending screen-token request (called from
    /// `handle_shared_server_message` when `RelayScreenTokenResponse`
    /// arrives).
    fn complete_screen_token_request(&self, request_id: u64, outcome: ScreenTokenOutcome) {
        if let Some(tx) = self
            .pending_screen_tokens
            .lock()
            .unwrap()
            .remove(&request_id)
        {
            let _ = tx.send(outcome);
        }
    }

    /// Complete a pending screen-sources request.
    fn complete_screen_sources_request(&self, request_id: u64, outcome: ScreenSourcesOutcome) {
        if let Some(tx) = self
            .pending_screen_sources
            .lock()
            .unwrap()
            .remove(&request_id)
        {
            let _ = tx.send(outcome);
        }
    }

    pub async fn run_upstream_connector(self: Arc<Self>) {
        let mut config_changes = self.config_manager.subscribe();
        let mut reconnect_interval = RELAY_RECONNECT_INITIAL_INTERVAL;
        let mut local_network = LocalNetworkBlockTracker::new();
        loop {
            let config = self.config_manager.snapshot().await;
            let relay_address = config.server.relay_address.trim().to_string();
            let relay_secret_key = config.server.relay_secret_key.trim().to_string();
            let relay_strict_tls = config.server.relay_strict_tls;
            let relay_ca_pem = config.server.relay_ca_pem.trim().to_string();

            let mut wait_interval = reconnect_interval;
            if !relay_address.is_empty() {
                let server_name = if config.server.name.trim().is_empty() {
                    "Shared Server".to_string()
                } else {
                    config.server.name.trim().to_string()
                };

                let attempt_started = tokio::time::Instant::now();
                match self
                    .clone()
                    .run_single_upstream_connection(
                        &relay_address,
                        &relay_secret_key,
                        relay_strict_tls,
                        &relay_ca_pem,
                        &server_name,
                        config.server.relay_port,
                    )
                    .await
                {
                    Ok(()) => {
                        warn!("Upstream relay connection to {} closed", relay_address);
                        local_network.record_success();
                    }
                    Err(error) => {
                        warn!(
                            "Upstream relay connection to {} failed: {}",
                            relay_address, error
                        );
                        local_network.record_failure(
                            &error,
                            &relay_address,
                            attempt_started.elapsed(),
                            self.connection_registry.has_active_lan_peer(),
                        );
                    }
                }
                if attempt_started.elapsed() >= RELAY_RECONNECT_RESET_AFTER {
                    reconnect_interval = RELAY_RECONNECT_INITIAL_INTERVAL;
                }

                // The connection loop exits as soon as the configured relay
                // connection settings change; apply the new settings right away
                // instead of waiting out the reconnect interval.
                let current = self.config_manager.snapshot().await;
                if current.server.relay_address.trim() != relay_address
                    || current.server.relay_secret_key.trim() != relay_secret_key
                    || current.server.relay_strict_tls != relay_strict_tls
                    || current.server.relay_ca_pem.trim() != relay_ca_pem
                {
                    reconnect_interval = RELAY_RECONNECT_INITIAL_INTERVAL;
                    continue;
                }

                if local_network.is_active() {
                    wait_interval = RELAY_LOCAL_NETWORK_RETRY_INTERVAL;
                }
                info!(
                    "Retrying upstream relay connection to {} in {}s",
                    relay_address,
                    wait_interval.as_secs()
                );
            }

            // Wait for the next reconnect attempt, but wake early when the
            // relay connection settings change (covers empty → configured too).
            let wait_started = tokio::time::Instant::now();
            let mut woke_for_config_change = false;
            loop {
                let remaining = wait_interval.saturating_sub(wait_started.elapsed());
                if remaining.is_zero() {
                    break;
                }
                tokio::select! {
                    _ = tokio::time::sleep(remaining) => break,
                    changed = config_changes.changed() => {
                        if changed.is_err() {
                            return;
                        }
                        let current = config_changes.borrow();
                        if current.server.relay_address.trim() != relay_address
                            || current.server.relay_secret_key.trim() != relay_secret_key
                            || current.server.relay_strict_tls != relay_strict_tls
                            || current.server.relay_ca_pem.trim() != relay_ca_pem
                        {
                            woke_for_config_change = true;
                            break;
                        }
                    }
                }
            }
            if woke_for_config_change {
                reconnect_interval = RELAY_RECONNECT_INITIAL_INTERVAL;
                continue;
            }
            reconnect_interval = (reconnect_interval * 2).min(RELAY_RECONNECT_MAX_INTERVAL);
        }
    }

    pub async fn register_client(&self, tx: ConnectionTx) {
        let shared_server_info = {
            let mut state = self.state.write().await;
            if !state
                .clients
                .iter()
                .any(|existing| existing.same_channel(&tx))
            {
                state.clients.push(tx.clone());
            }
            state
                .servers
                .values()
                .filter_map(|server| {
                    server
                        .server_info
                        .clone()
                        .map(|info| (server.id.clone(), info))
                })
                .collect::<Vec<_>>()
        };
        for (server_id, info) in shared_server_info {
            let _ = tx.send(Message::ServerInfo {
                server_id: Some(server_id),
                info,
            });
        }
    }

    pub async fn unregister_client(&self, tx: &ConnectionTx) {
        let mut state = self.state.write().await;
        state.clients.retain(|existing| !existing.same_channel(tx));
    }

    pub async fn register_shared_server(
        &self,
        name: String,
        request_port: Option<u16>,
        peer_addr: std::net::SocketAddr,
        tx: ConnectionTx,
    ) -> (String, String) {
        let server_id = Uuid::new_v4().to_string();
        let terminal_tunnel_key = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let request_url = request_port
            .map(|port| format!("wss://{}:{}", format_socket_host(peer_addr.ip()), port));
        let request_url_for_log = request_url.clone();
        let replaced_servers = {
            let mut state = self.state.write().await;
            let replaced_ids = if request_url.is_some() {
                state
                    .servers
                    .values()
                    .filter(|server| {
                        server.peer_ip == peer_addr.ip()
                            && server.request_url.as_deref() == request_url.as_deref()
                    })
                    .map(|server| server.id.clone())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let mut replaced_servers = Vec::with_capacity(replaced_ids.len());
            for replaced_id in replaced_ids {
                if let Some(server) =
                    Self::remove_shared_server_from_state(&mut state, &replaced_id)
                {
                    replaced_servers.push(server);
                }
            }
            state.servers.insert(
                server_id.clone(),
                SharedServerState {
                    id: server_id.clone(),
                    name: normalize_server_name(&name),
                    request_url,
                    terminal_tunnel_key: terminal_tunnel_key.clone(),
                    primary_tx: tx,
                    peer_ip: peer_addr.ip(),
                    sessions: HashMap::new(),
                    tab_layouts: HashMap::new(),
                    tab_groups: Vec::new(),
                    profiles: Vec::new(),
                    recent_project_directories: Vec::new(),
                    terminal_command_history: Vec::new(),
                    known_apps: Vec::new(),
                    server_info: None,
                    pending_creates: VecDeque::new(),
                    udp_pairings: HashSet::new(),
                },
            );
            replaced_servers
        };
        for server in &replaced_servers {
            info!(
                "Shared server '{}' ({}) from {} was replaced by a new registration for the same terminal endpoint",
                server.name, server.id, server.peer_ip
            );
            self.release_shared_server_udp_pairings(server);
            self.broadcast_removed_shared_server_metadata(&server.id)
                .await;
        }
        info!(
            "Shared server '{}' registered from {} as {} (terminal_endpoint={})",
            normalize_server_name(&name),
            peer_addr,
            server_id,
            request_url_for_log.as_deref().unwrap_or("unavailable")
        );
        self.broadcast_session_list().await;
        self.broadcast_relay_server_list().await;
        if !replaced_servers.is_empty() {
            self.broadcast_terminal_profile_list().await;
        }
        (server_id, terminal_tunnel_key)
    }

    fn remove_shared_server_from_state(
        state: &mut RelayState,
        server_id: &str,
    ) -> Option<SharedServerState> {
        let server = state.servers.remove(server_id)?;
        for session_id in server.sessions.keys() {
            if state
                .remote_session_index
                .get(session_id)
                .is_some_and(|owner| owner == server_id)
            {
                state.remote_session_index.remove(session_id);
            }
        }
        Some(server)
    }

    fn release_shared_server_udp_pairings(&self, server: &SharedServerState) {
        if !server.udp_pairings.is_empty() {
            if let Some(allocator) = self.udp_allocator.get() {
                allocator.release_all(server.udp_pairings.clone());
            }
        }
    }

    async fn broadcast_removed_shared_server_metadata(&self, server_id: &str) {
        let server_id = server_id.to_string();
        self.broadcast_client_message(Message::RecentProjectDirectoryList {
            server_id: Some(server_id.clone()),
            directories: Vec::new(),
        })
        .await;
        self.broadcast_client_message(Message::TerminalCommandHistoryList {
            server_id: Some(server_id.clone()),
            commands: Vec::new(),
        })
        .await;
        self.broadcast_client_message(Message::KnownAppList {
            server_id: Some(server_id),
            apps: Vec::new(),
        })
        .await;
    }

    pub async fn unregister_shared_server(&self, server_id: &str) {
        let removed = {
            let mut state = self.state.write().await;
            Self::remove_shared_server_from_state(&mut state, server_id)
        };
        match &removed {
            Some(server) => info!(
                "Shared server '{}' ({}) from {} unregistered; dropped {} terminal session(s)",
                server.name,
                server_id,
                server.peer_ip,
                server.sessions.len()
            ),
            None => debug!(
                "unregister_shared_server: no shared server with id {} was registered",
                server_id
            ),
        }
        if let Some(server) = &removed {
            self.release_shared_server_udp_pairings(server);
        }
        self.broadcast_session_list().await;
        self.broadcast_terminal_profile_list().await;
        self.broadcast_relay_server_list().await;
        if removed.is_some() {
            self.broadcast_removed_shared_server_metadata(server_id)
                .await;
        }
    }

    pub async fn handle_shared_server_message(
        self: &Arc<Self>,
        server_id: &str,
        message: Message,
    ) -> anyhow::Result<()> {
        match message {
            Message::RequestSharedPortForward {
                session_id,
                target_port,
                ..
            } => {
                let shared_tx = {
                    let state = self.state.read().await;
                    state
                        .servers
                        .get(server_id)
                        .map(|server| server.primary_tx.clone())
                };
                let Some(shared_tx) = shared_tx else {
                    warn!(
                        "Cannot forward static port {} for unknown shared server {}",
                        target_port, server_id
                    );
                    return Ok(());
                };

                self.add_pending_relay_port_forward(server_id, target_port, session_id);
                if shared_tx
                    .send(Message::RequestReverseProxy {
                        target_port,
                        auth_token: None,
                        purpose: None,
                    })
                    .is_err()
                {
                    self.remove_pending_relay_port_forward(server_id, target_port, session_id);
                    let _ = shared_tx.send(Message::SharedPortForwardReady {
                        session_id,
                        target_port,
                        relay_port: 0,
                        error: Some("Shared server connection closed".to_string()),
                    });
                    return Ok(());
                }
                info!(
                    "Requested shared server {} to forward static port {} for session {:?}",
                    server_id, target_port, session_id
                );
            }
            Message::ReverseProxyReady {
                target_port,
                proxy_port,
                error,
                ..
            } => {
                let Some(session_id) = self.take_pending_relay_port_forward(server_id, target_port)
                else {
                    warn!(
                        "Received shared reverse proxy response for unknown server {} target port {}",
                        server_id, target_port
                    );
                    return Ok(());
                };

                let result = match error {
                    Some(error) => Err(error),
                    None => match self.shared_server_peer_ip(server_id).await {
                        Some(shared_ip) => {
                            let upstream_addr = std::net::SocketAddr::new(shared_ip, proxy_port);
                            self.port_forward_manager
                                .ensure_relayed_forward(upstream_addr)
                                .await
                                .map(|lease| lease.proxy_port)
                                .map_err(|error| format!("{error:#}"))
                        }
                        None => Err(format!("Shared server {server_id} is not registered")),
                    },
                };

                let (relay_port, error) = match result {
                    Ok(relay_port) => {
                        info!(
                            "Created relay port forward for session {:?}: shared_target={} relay_port={}",
                            session_id, proxy_port, relay_port
                        );
                        (relay_port, None)
                    }
                    Err(error) => {
                        warn!(
                            "Failed to create relay port forward for session {:?}: {}",
                            session_id, error
                        );
                        (0, Some(error))
                    }
                };
                let state = self.state.read().await;
                if let Some(server) = state.servers.get(server_id) {
                    let _ = server.primary_tx.send(Message::SharedPortForwardReady {
                        session_id,
                        target_port,
                        relay_port,
                        error,
                    });
                }
            }
            Message::StartStaticHttpServerResult {
                session_id,
                path,
                port,
                url,
                share_token,
                port_forwarded,
                error,
            } => {
                // The shared server has already received the final relay proxy
                // port. Forward the result to the client unchanged.
                self.broadcast_client_message(Message::StartStaticHttpServerResult {
                    session_id,
                    path,
                    port,
                    url,
                    share_token,
                    port_forwarded,
                    error,
                })
                .await;
            }
            Message::SessionList { sessions } => {
                self.update_shared_sessions(server_id, sessions).await;
            }
            Message::TabInfoList { tabs, .. } => {
                // Keep the shared server's authoritative layouts (tab_id →
                // TerminalLayout) so a merge/split layout change that does not
                // alter the session list still reaches clients. Layouts come
                // from the real shared server, so their recursion depth is
                // bounded by actual UI structure.
                let layout_count = {
                    let mut state = self.state.write().await;
                    if let Some(server) = state.servers.get_mut(server_id) {
                        server.tab_layouts.clear();
                        server
                            .tab_layouts
                            .extend(tabs.into_iter().map(|tab| (tab.tab_id, tab.layout)));
                    }
                    state
                        .servers
                        .get(server_id)
                        .map(|server| server.tab_layouts.len())
                        .unwrap_or(0)
                };
                trace!(
                    "Shared server {} synchronized {} terminal tab descriptor(s)",
                    server_id,
                    layout_count
                );
                // Broadcast the aggregated layout/tab state to all connected
                // clients so they pick up split/merge changes without waiting
                // for a session-list refresh.
                self.spawn_shared_metadata_broadcast();
            }
            Message::TabGroupList { groups } => {
                self.update_shared_tab_groups(server_id, groups).await;
            }
            Message::TerminalProfileList { profiles } => {
                self.update_shared_profiles(server_id, profiles).await;
            }
            Message::RecentProjectDirectoryList { directories, .. } => {
                self.update_shared_recent_project_directories(server_id, directories)
                    .await;
            }
            Message::TerminalCommandHistoryList { commands, .. } => {
                self.update_shared_terminal_command_history(server_id, commands)
                    .await;
            }
            Message::KnownAppList { apps, .. } => {
                self.update_shared_known_apps(server_id, apps).await;
            }
            Message::ServerInfo { info, .. } => {
                self.update_shared_server_info(server_id, info).await;
            }
            Message::SessionCreated { session_id } => {
                let requester = self.take_pending_create(server_id).await;
                info!(
                    "Shared server {} created terminal session {} (requester_waiting={})",
                    server_id,
                    session_id,
                    requester.is_some()
                );
                if let Some(requester) = requester {
                    let _ = requester.send(Message::SessionCreated { session_id });
                }
            }
            Message::PathContentList {
                session_id,
                path,
                items,
                ..
            } => {
                warn!(
                    "No pending requester for shared server path content response session={} path={}",
                    session_id, path
                );
                drop(items);
            }
            Message::FindFilesResult {
                session_id,
                query,
                chunk_index,
                is_final,
                results,
                error,
                ..
            } => {
                warn!(
                    "No pending requester for shared server file search response session={} query_bytes={} chunk={} final={} files={} error={}",
                    session_id,
                    query.len(),
                    chunk_index,
                    is_final,
                    results.len(),
                    error.is_some()
                );
            }
            Message::SearchPathNamesResult {
                session_id,
                query,
                is_final,
                results,
                error,
                ..
            } => {
                warn!(
                    "No pending requester for shared server path name search response session={} query_bytes={} final={} matches={} error={}",
                    session_id,
                    query.len(),
                    is_final,
                    results.len(),
                    error.is_some()
                );
            }
            Message::FileContent {
                session_id,
                path,
                data,
                diff_ranges: _,
                error,
            } => {
                warn!(
                    "No pending requester for shared server file response session={} path={} bytes={} error={}",
                    session_id,
                    path,
                    data.len(),
                    error.is_some()
                );
            }
            Message::AiConversationList {
                session_id,
                conversations,
            } => {
                self.broadcast_client_message(Message::AiConversationList {
                    session_id,
                    conversations,
                })
                .await;
            }
            Message::AiConversationMessages {
                session_id,
                conversation_id,
                messages,
            } => {
                warn!(
                    "No pending requester for shared server AI conversation response session={} conversation={} messages={}",
                    session_id,
                    conversation_id,
                    messages.len()
                );
            }
            Message::AiConversationMessagesUpdate {
                session_id,
                conversation_id,
                start_index,
                messages,
            } => {
                self.broadcast_client_message(Message::AiConversationMessagesUpdate {
                    session_id,
                    conversation_id,
                    start_index,
                    messages,
                })
                .await;
            }
            Message::GitChangedFilesList {
                session_id,
                commit,
                items,
                ..
            } => {
                warn!(
                    "No pending requester for shared server git changed files response session={} commit={} count={}",
                    session_id,
                    commit.as_deref().unwrap_or("<worktree>"),
                    items.len()
                );
            }
            Message::GitBranchesResult {
                session_id,
                current,
                branches,
                remote_tracking_hash,
                tags,
                ..
            } => {
                warn!(
                    "No pending requester for shared server git branches response session={} count={} tags={} current={}",
                    session_id,
                    branches.len(),
                    tags.len(),
                    current.as_deref().unwrap_or("<none>")
                );
                drop(remote_tracking_hash);
            }
            Message::GitLogList {
                session_id,
                commit,
                branch,
                commits,
                ..
            } => {
                warn!(
                    "No pending requester for shared server git log response session={} commit={} branch={} count={}",
                    session_id,
                    commit.as_deref().unwrap_or("HEAD"),
                    branch.as_deref().unwrap_or("<default>"),
                    commits.len()
                );
            }
            Message::GitDiffResult {
                session_id,
                path,
                diff,
            } => {
                warn!(
                    "No pending requester for shared server git diff response session={} path={} bytes={}",
                    session_id,
                    path,
                    diff.len()
                );
            }
            Message::SessionBufferInfo {
                session_id,
                start_offset,
                end_offset,
            } => {
                warn!(
                    "Ignoring unexpected shared server primary SessionBufferInfo session={} start={} end={}",
                    session_id, start_offset, end_offset
                );
            }
            Message::HistoryRawChunk {
                session_id,
                start_offset,
                end_offset,
                data,
            } => {
                warn!(
                    "Ignoring unexpected shared server primary HistoryRawChunk session={} start={} end={} bytes={}",
                    session_id,
                    start_offset,
                    end_offset,
                    data.len()
                );
            }
            Message::Output {
                session_id,
                start_offset,
                data,
            } => {
                warn!(
                    "Ignoring unexpected shared server primary Output session={} start={} bytes={}",
                    session_id,
                    start_offset,
                    data.len()
                );
            }
            Message::SessionClosed { session_id } => {
                self.broadcast_client_message(Message::SessionClosed { session_id })
                    .await;
                self.spawn_shared_metadata_broadcast();
            }
            Message::TerminalAttentionRequired { attention } => {
                self.broadcast_client_message(Message::TerminalAttentionRequired { attention })
                    .await;
            }
            Message::TerminalAttentionResolved { session_id } => {
                self.broadcast_client_message(Message::TerminalAttentionResolved { session_id })
                    .await;
            }
            Message::TerminalTaskCompleted { completion } => {
                self.broadcast_client_message(Message::TerminalTaskCompleted { completion })
                    .await;
            }
            Message::Error { code, message } => {
                if let Some(requester) = self.take_pending_create(server_id).await {
                    let _ = requester.send(Message::Error {
                        code,
                        message: message.clone(),
                    });
                }
                self.broadcast_error_to_clients(code, &message).await;
            }
            Message::RelayAllocateUdpPairing {
                request_id,
                target_port,
            } => {
                // Resolve the shared server's reachable IP so the relay
                // forwards media datagrams to the correct host (not
                // 127.0.0.1 — the relay and screen-server may be on
                // different machines).
                let (primary_tx, target_host) = {
                    let state = self.state.read().await;
                    match state.servers.get(server_id) {
                        Some(server) => (server.primary_tx.clone(), server.peer_ip.to_string()),
                        None => {
                            // Should not happen (shared server gone between
                            // the request and here), but fall back safely.
                            warn!("RelayAllocateUdpPairing from unknown shared server {server_id}");
                            return Ok(());
                        }
                    }
                };
                let allocator = self.udp_allocator().await;
                let response = match allocator.allocate(&target_host, target_port).await {
                    Ok(allocated) => {
                        {
                            let mut state = self.state.write().await;
                            if let Some(server) = state.servers.get_mut(server_id) {
                                server.udp_pairings.insert(allocated.pairing.clone());
                            }
                        }
                        info!(
                            "Allocated relay UDP pairing {} on port {} for shared server {} (target_port={})",
                            allocated.pairing, allocated.port, server_id, target_port
                        );
                        // The relay cannot know how clients reach it; the
                        // shared server substitutes its configured
                        // relay_address host before answering the client.
                        Message::RelayUdpPairingAllocated {
                            request_id,
                            pairing: allocated.pairing,
                            host: String::new(),
                            port: allocated.port,
                            error: None,
                        }
                    }
                    Err(error) => Message::RelayUdpPairingAllocated {
                        request_id,
                        pairing: String::new(),
                        host: String::new(),
                        port: 0,
                        error: Some(format!("{error:#}")),
                    },
                };
                let _ = primary_tx.send(response);
            }
            Message::RelayReleaseUdpPairing { pairing } => {
                if let Some(allocator) = self.udp_allocator.get() {
                    allocator.release(&pairing);
                }
                let mut state = self.state.write().await;
                if let Some(server) = state.servers.get_mut(server_id) {
                    server.udp_pairings.remove(&pairing);
                }
            }
            // Screen-stream relay coordination: complete pending oneshots.
            Message::RelayScreenTokenResponse {
                request_id,
                token,
                screen_udp_port,
                error,
            } => {
                info!(
                    "RelayScreenTokenResponse request_id={}: screen_udp_port={}, error={:?}",
                    request_id, screen_udp_port, error
                );
                let outcome = match error {
                    Some(error) => Err(error),
                    None => Ok((token, screen_udp_port)),
                };
                self.complete_screen_token_request(request_id, outcome);
            }
            Message::RelayScreenSourcesResponse {
                request_id,
                sources,
                revision,
                error,
            } => {
                info!(
                    "RelayScreenSourcesResponse request_id={}: {} sources, error={:?}",
                    request_id,
                    sources.len(),
                    error
                );
                let outcome = match error {
                    Some(error) => Err(error),
                    None => Ok((revision, sources)),
                };
                self.complete_screen_sources_request(request_id, outcome);
            }
            Message::Pong | Message::Ping => {}
            other => {
                // Avoid formatting the full message (`{:?}`) here — shared-server
                // responses like SearchPathNamesResult can carry large result
                // lists whose Debug dump stalls the relay loop and trips client
                // timeouts. Log only the message type.
                warn!(
                    "Ignoring unexpected shared server message from {}: type={}",
                    server_id,
                    other.msg_name()
                );
            }
        }

        Ok(())
    }

    pub async fn list_sessions(&self) -> anyhow::Result<Vec<SessionInfo>> {
        let sessions = self.session_manager.list_sessions().await?;
        Ok(self.merge_shared_server_sessions(sessions).await)
    }

    async fn merge_shared_server_sessions(
        &self,
        mut sessions: Vec<SessionInfo>,
    ) -> Vec<SessionInfo> {
        let state = self.state.read().await;
        for server in sorted_shared_servers(&state) {
            let mut remote_sessions = server.sessions.values().cloned().collect::<Vec<_>>();
            sort_session_infos(&mut remote_sessions);
            sessions.extend(remote_sessions);
        }
        sessions
    }

    pub async fn list_terminal_profiles(&self) -> Vec<TerminalProfile> {
        let mut profiles = self.session_manager.list_terminal_profiles().await;
        let state = self.state.read().await;
        for server in state.servers.values() {
            profiles.extend(server.profiles.iter().cloned());
        }
        profiles
    }

    pub async fn list_tab_groups(&self) -> anyhow::Result<Vec<TabGroup>> {
        let groups = self.session_manager.list_tab_groups().await;
        Ok(self.merge_shared_server_tab_groups(groups).await)
    }

    async fn merge_shared_server_tab_groups(&self, mut groups: Vec<TabGroup>) -> Vec<TabGroup> {
        let state = self.state.read().await;
        for server in sorted_shared_servers(&state) {
            if !server.tab_groups.is_empty() {
                groups.extend(server.tab_groups.iter().cloned());
                continue;
            }
            let sessions = server.sessions.values().cloned().collect::<Vec<_>>();
            groups.push(TabGroup {
                server_id: Some(server.id.clone()),
                id: TAB_GROUP_ALL_ID,
                name: "All".to_string(),
                builtin: true,
                tab_ids: unique_tab_ids(sessions.iter().map(|session| session.tab_id.clone())),
            });
            groups.push(TabGroup {
                server_id: Some(server.id.clone()),
                id: TAB_GROUP_UNGROUPED_ID,
                name: "Ungrouped".to_string(),
                builtin: true,
                tab_ids: unique_tab_ids(
                    sessions
                        .iter()
                        .filter(|session| session.tab_group_id == TAB_GROUP_UNGROUPED_ID)
                        .map(|session| session.tab_id.clone()),
                ),
            });
        }
        merge_global_all_group(groups)
    }

    /// Aggregate TabInfo from local and all remote servers.
    pub async fn list_all_tab_infos(&self) -> Vec<ProtocolTabInfo> {
        let tabs = self.session_manager.list_tab_infos().await;
        self.merge_shared_server_tab_infos(tabs).await
    }

    async fn merge_shared_server_tab_infos(
        &self,
        mut tabs: Vec<ProtocolTabInfo>,
    ) -> Vec<ProtocolTabInfo> {
        let state = self.state.read().await;
        for server in sorted_shared_servers(&state) {
            tabs.extend(shared_server_tab_infos(server));
        }
        tabs
    }

    pub async fn recent_project_directory_lists(
        &self,
    ) -> Vec<(String, Vec<RecentProjectDirectory>)> {
        let config = self.config_manager.snapshot().await;
        let mut lists = vec![(
            config.server.server_id,
            self.session_manager.recent_project_directories(),
        )];
        let state = self.state.read().await;
        for server in state.servers.values() {
            lists.push((server.id.clone(), server.recent_project_directories.clone()));
        }
        lists
    }

    pub async fn terminal_command_history_lists(
        &self,
    ) -> Vec<(String, Vec<TerminalCommandHistory>)> {
        let config = self.config_manager.snapshot().await;
        let mut lists = vec![(
            config.server.server_id,
            self.session_manager.recent_terminal_commands(),
        )];
        let state = self.state.read().await;
        for server in state.servers.values() {
            lists.push((server.id.clone(), server.terminal_command_history.clone()));
        }
        lists
    }

    pub async fn known_app_lists(&self) -> Vec<(String, Vec<KnownAppInfo>)> {
        let config = self.config_manager.snapshot().await;
        let mut lists = vec![(
            config.server.server_id,
            self.session_manager.known_apps().await,
        )];
        let state = self.state.read().await;
        for server in state.servers.values() {
            lists.push((server.id.clone(), server.known_apps.clone()));
        }
        lists
    }

    pub async fn broadcast_known_app_lists(&self) {
        for (server_id, apps) in self.known_app_lists().await {
            self.broadcast_client_message(Message::KnownAppList {
                server_id: Some(server_id),
                apps,
            })
            .await;
        }
    }

    pub async fn relay_server_list(&self) -> Vec<RelayServerInfo> {
        let state = self.state.read().await;
        let mut servers = state
            .servers
            .values()
            .map(|server| RelayServerInfo {
                id: server.id.clone(),
                name: server.name.clone(),
                session_count: server.sessions.len() as u32,
            })
            .collect::<Vec<_>>();
        servers.sort_by(|left, right| left.name.cmp(&right.name));
        servers
    }

    /// Consume a short-lived capability issued by the upstream relay for one
    /// transparent terminal connection. Capabilities are single-use so a
    /// captured relay-to-shared handshake cannot be replayed.
    pub async fn consume_terminal_capability(
        &self,
        session_id: SessionId,
        capability: &str,
    ) -> bool {
        let now = unix_time_secs();
        let terminal_tunnel_key = {
            let state = self.state.read().await;
            state.upstream_terminal_tunnel_key.clone()
        };
        let Some(terminal_tunnel_key) = terminal_tunnel_key else {
            return false;
        };
        let Some(expires_at) =
            verify_relay_terminal_capability(&terminal_tunnel_key, session_id, capability, now)
        else {
            return false;
        };

        let mut state = self.state.write().await;
        state
            .consumed_terminal_capabilities
            .retain(|_, expiry| relay_capability_time_is_valid(*expiry, now));
        if state
            .consumed_terminal_capabilities
            .contains_key(capability)
        {
            return false;
        }
        state
            .consumed_terminal_capabilities
            .insert(capability.to_string(), expires_at);
        true
    }

    /// Consume a single-use capability authorizing one transparent request
    /// tunnel from the upstream relay.
    pub async fn consume_request_capability(&self, server_id: &str, capability: &str) -> bool {
        let now = unix_time_secs();
        let tunnel_key = {
            let state = self.state.read().await;
            state.upstream_terminal_tunnel_key.clone()
        };
        let Some(tunnel_key) = tunnel_key else {
            return false;
        };
        let Some(expires_at) =
            verify_relay_request_capability(&tunnel_key, server_id, capability, now)
        else {
            return false;
        };
        let mut state = self.state.write().await;
        state
            .consumed_terminal_capabilities
            .retain(|_, expiry| relay_capability_time_is_valid(*expiry, now));
        if state
            .consumed_terminal_capabilities
            .contains_key(capability)
        {
            return false;
        }
        state
            .consumed_terminal_capabilities
            .insert(capability.to_string(), expires_at);
        true
    }

    async fn set_upstream_terminal_tunnel_key(&self, terminal_tunnel_key: String) {
        let mut state = self.state.write().await;
        state.upstream_terminal_tunnel_key = Some(terminal_tunnel_key);
        state.consumed_terminal_capabilities.clear();
    }

    /// Route a terminal WebSocket to its owning shared server. After the
    /// synthetic upstream Hello is sent, every WebSocket frame (including the
    /// Noise handshake) is copied without decoding or re-encoding its payload.
    pub async fn proxy_remote_terminal<S>(
        &self,
        client_websocket: &mut WebSocketStream<S>,
        session_id: SessionId,
        protocol_version: u32,
        client_version: String,
        supports_binary_output: bool,
        supports_message_encryption: bool,
        mut parent_alive: watch::Receiver<bool>,
    ) -> anyhow::Result<bool>
    where
        S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    {
        let (request_url, terminal_tunnel_key) = {
            let state = self.state.read().await;
            let Some(server_id) = state.remote_session_index.get(&session_id) else {
                return Ok(false);
            };
            let Some(server) = state.servers.get(server_id) else {
                return Ok(false);
            };
            let Some(request_url) = server.request_url.clone() else {
                anyhow::bail!("Shared server {} has no terminal endpoint", server_id);
            };
            (request_url, server.terminal_tunnel_key.clone())
        };

        if !supports_message_encryption {
            anyhow::bail!("Transparent relayed terminals require message encryption");
        }

        let expires_at = unix_time_secs().saturating_add(RELAY_TERMINAL_CAPABILITY_LIFETIME_SECS);
        let capability = create_relay_terminal_capability(
            &terminal_tunnel_key,
            session_id,
            expires_at,
            &Uuid::new_v4().simple().to_string(),
        );

        let connector = tls_connector_for_url(&request_url)?;
        let mut shared_websocket = connect_relay_websocket(request_url.as_str(), connector).await?;
        let upstream_hello = Message::Hello {
            protocol_version,
            client_version,
            secret_key_hash: None,
            supports_binary_output,
            supports_message_encryption,
            connection_kind: Some(ConnectionKind::Terminal),
            auth_token: None,
            target_session_id: Some(session_id),
            relay_terminal_capability: Some(capability),
            target_server_id: None,
            relay_request_capability: None,
        };
        shared_websocket
            .send(WebSocketMessage::Text(upstream_hello.to_text()?.into()))
            .await
            .map_err(|error| anyhow::anyhow!("Failed to open shared terminal tunnel: {error}"))?;

        info!(
            "Transparent terminal tunnel established for session {} via {}",
            session_id, request_url
        );

        // Keep the two directions independent. In particular, a desktop client
        // that is slow to consume terminal output must not block its keyboard
        // frames from reaching the shared server.
        let (mut client_write, mut client_read) = client_websocket.split();
        let (mut shared_write, mut shared_read) = shared_websocket.split();
        let client_to_shared = async {
            while let Some(frame) = client_read.next().await {
                let frame = frame
                    .map_err(|error| anyhow::anyhow!("Client terminal tunnel failed: {error}"))?;
                let closed = matches!(frame, WebSocketMessage::Close(_));
                shared_write.send(frame).await.map_err(|error| {
                    anyhow::anyhow!("Failed to forward terminal frame to shared server: {error}")
                })?;
                if closed {
                    break;
                }
            }
            anyhow::Ok(())
        };
        let shared_to_client = async {
            while let Some(frame) = shared_read.next().await {
                let frame = frame
                    .map_err(|error| anyhow::anyhow!("Shared terminal tunnel failed: {error}"))?;
                let closed = matches!(frame, WebSocketMessage::Close(_));
                client_write.send(frame).await.map_err(|error| {
                    anyhow::anyhow!("Failed to forward terminal frame to client: {error}")
                })?;
                if closed {
                    break;
                }
            }
            anyhow::Ok(())
        };
        tokio::select! {
            result = client_to_shared => result?,
            result = shared_to_client => result?,
            changed = parent_alive.changed() => {
                if changed.is_ok() && *parent_alive.borrow() {
                    anyhow::bail!("Primary terminal tunnel state changed unexpectedly");
                }
            }
        }

        Ok(true)
    }

    /// Establish a one-to-one request tunnel to a registered shared server.
    /// The route Hello is created by the relay; all subsequent authentication
    /// and Noise frames are forwarded without application decoding.
    pub async fn proxy_remote_request<S>(
        &self,
        client_websocket: &mut WebSocketStream<S>,
        server_id: &str,
        protocol_version: u32,
        client_version: String,
        supports_binary_output: bool,
        supports_message_encryption: bool,
        mut parent_alive: watch::Receiver<bool>,
    ) -> anyhow::Result<bool>
    where
        S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    {
        let (request_url, tunnel_key) = {
            let state = self.state.read().await;
            let Some(server) = state.servers.get(server_id) else {
                return Ok(false);
            };
            let Some(request_url) = server.request_url.clone() else {
                anyhow::bail!("Shared server {} has no request endpoint", server_id);
            };
            (request_url, server.terminal_tunnel_key.clone())
        };
        if !supports_message_encryption {
            anyhow::bail!("Transparent relayed requests require message encryption");
        }

        let expires_at = unix_time_secs().saturating_add(RELAY_TERMINAL_CAPABILITY_LIFETIME_SECS);
        let capability = create_relay_request_capability(
            &tunnel_key,
            server_id,
            expires_at,
            &Uuid::new_v4().simple().to_string(),
        );
        let connector = tls_connector_for_url(&request_url)?;
        let mut shared_websocket = connect_relay_websocket(request_url.as_str(), connector).await?;
        shared_websocket
            .send(WebSocketMessage::Text(
                Message::Hello {
                    protocol_version,
                    client_version,
                    secret_key_hash: None,
                    supports_binary_output,
                    supports_message_encryption,
                    connection_kind: Some(ConnectionKind::RelayRequest),
                    auth_token: None,
                    target_session_id: None,
                    relay_terminal_capability: None,
                    target_server_id: Some(server_id.to_string()),
                    relay_request_capability: Some(capability),
                }
                .to_text()?
                .into(),
            ))
            .await
            .map_err(|error| anyhow::anyhow!("Failed to open shared request tunnel: {error}"))?;

        info!(
            "Transparent request tunnel established for shared server {} via {}",
            server_id, request_url
        );
        // Each direction owns its own read/write pump. A slow upstream write
        // applies one-frame backpressure only to that direction and cannot
        // stall acknowledgements or cancellation flowing the other way.
        let (mut client_write, mut client_read) = client_websocket.split();
        let (mut shared_write, mut shared_read) = shared_websocket.split();
        let client_to_shared = async {
            while let Some(frame) = client_read.next().await {
                let frame = frame
                    .map_err(|error| anyhow::anyhow!("Client request tunnel failed: {error}"))?;
                let closed = matches!(frame, WebSocketMessage::Close(_));
                shared_write.send(frame).await.map_err(|error| {
                    anyhow::anyhow!("Failed to forward request frame to shared server: {error}")
                })?;
                if closed {
                    break;
                }
            }
            anyhow::Ok(())
        };
        let shared_to_client = async {
            while let Some(frame) = shared_read.next().await {
                let frame = frame
                    .map_err(|error| anyhow::anyhow!("Shared request tunnel failed: {error}"))?;
                let closed = matches!(frame, WebSocketMessage::Close(_));
                client_write.send(frame).await.map_err(|error| {
                    anyhow::anyhow!("Failed to forward request frame to client: {error}")
                })?;
                if closed {
                    break;
                }
            }
            anyhow::Ok(())
        };
        tokio::select! {
            result = client_to_shared => result?,
            result = shared_to_client => result?,
            changed = parent_alive.changed() => {
                if changed.is_ok() && *parent_alive.borrow() {
                    anyhow::bail!("Primary request tunnel state changed unexpectedly");
                }
            }
        }
        Ok(true)
    }

    pub async fn create_remote_session(
        &self,
        server_id: &str,
        shell: String,
        profile_id: Option<String>,
        size: TerminalSize,
        env: HashMap<String, String>,
        cwd: Option<String>,
        sibling_session_id: Option<SessionId>,
        duplicate_session_id: Option<SessionId>,
        tab_group_id: Option<u32>,
        title: Option<String>,
        app: Option<String>,
        background: Option<[u8; 3]>,
        foreground: Option<[u8; 3]>,
        requester: ConnectionTx,
    ) -> anyhow::Result<bool> {
        let server_tx = {
            let mut state = self.state.write().await;
            let Some(server) = state.servers.get_mut(server_id) else {
                return Ok(false);
            };
            server.pending_creates.push_back(requester);
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::CreateSession {
                shell,
                profile_id,
                size,
                env,
                cwd,
                title,
                sibling_session_id,
                server_id: None,
                tab_group_id,
                duplicate_session_id,
                app,
                background,
                foreground,
            })
            .map_err(|_| anyhow::anyhow!("Failed to forward create session request"))?;
        Ok(true)
    }

    /// Forward a split request for a session owned by a shared (relay) server
    /// to that server. The shared server creates the new split session and
    /// replies with `SessionCreated` (or `Error`), which `handle_shared_server_message`
    /// routes back to `requester` via the same pending-create queue used by
    /// `create_remote_session`.
    pub async fn split_remote_terminal(
        &self,
        target_session_id: SessionId,
        placement: TabSplitPlacement,
        size: TerminalSize,
        profile_id: Option<String>,
        requester: ConnectionTx,
    ) -> anyhow::Result<bool> {
        let server_tx = {
            let mut state = self.state.write().await;
            let Some(server_id) = state.remote_session_index.get(&target_session_id).cloned()
            else {
                return Ok(false);
            };
            let Some(server) = state.servers.get_mut(&server_id) else {
                return Ok(false);
            };
            server.pending_creates.push_back(requester);
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::SplitTerminal {
                target_session_id,
                placement,
                size,
                profile_id,
            })
            .map_err(|_| anyhow::anyhow!("Failed to forward split terminal request"))?;
        Ok(true)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn close_remote_session(&self, session_id: SessionId) -> anyhow::Result<bool> {
        let server_tx = {
            let state = self.state.read().await;
            let Some(server_id) = state.remote_session_index.get(&session_id) else {
                return Ok(false);
            };
            let Some(server) = state.servers.get(server_id) else {
                return Ok(false);
            };
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::CloseSession { session_id })
            .map_err(|_| anyhow::anyhow!("Failed to forward close session request"))?;
        Ok(true)
    }

    pub async fn detach_remote_session(&self, session_id: SessionId) -> anyhow::Result<bool> {
        let server_tx = {
            let state = self.state.read().await;
            let Some(server_id) = state.remote_session_index.get(&session_id) else {
                return Ok(false);
            };
            let Some(server) = state.servers.get(server_id) else {
                return Ok(false);
            };
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::DetachSession { session_id })
            .map_err(|_| anyhow::anyhow!("Failed to forward detach session request"))?;
        Ok(true)
    }

    pub async fn rename_remote_session(
        &self,
        session_id: SessionId,
        session_name: Option<String>,
    ) -> anyhow::Result<bool> {
        let server_tx = {
            let state = self.state.read().await;
            let Some(server_id) = state.remote_session_index.get(&session_id) else {
                return Ok(false);
            };
            let Some(server) = state.servers.get(server_id) else {
                return Ok(false);
            };
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::RenameSession {
                session_id,
                session_name,
            })
            .map_err(|_| anyhow::anyhow!("Failed to forward rename session request"))?;
        Ok(true)
    }

    pub async fn create_remote_tab_group(
        &self,
        server_id: &str,
        name: String,
        initial_session_id: Option<SessionId>,
    ) -> anyhow::Result<bool> {
        let server_tx = {
            let state = self.state.read().await;
            let Some(server) = state.servers.get(server_id) else {
                return Ok(false);
            };
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::CreateTabGroup {
                server_id: None,
                name,
                initial_session_id,
            })
            .map_err(|_| anyhow::anyhow!("Failed to forward create terminal group request"))?;
        Ok(true)
    }

    pub async fn delete_remote_tab_group(
        &self,
        server_id: &str,
        group_id: u32,
    ) -> anyhow::Result<bool> {
        let server_tx = {
            let state = self.state.read().await;
            let Some(server) = state.servers.get(server_id) else {
                return Ok(false);
            };
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::DeleteTabGroup {
                server_id: None,
                group_id,
            })
            .map_err(|_| anyhow::anyhow!("Failed to forward delete terminal group request"))?;
        Ok(true)
    }

    pub async fn rename_remote_tab_group(
        &self,
        server_id: &str,
        group_id: u32,
        name: String,
    ) -> anyhow::Result<bool> {
        let server_tx = {
            let state = self.state.read().await;
            let Some(server) = state.servers.get(server_id) else {
                return Ok(false);
            };
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::RenameTabGroup {
                server_id: None,
                group_id,
                name,
            })
            .map_err(|_| anyhow::anyhow!("Failed to forward rename terminal group request"))?;
        Ok(true)
    }

    pub async fn move_remote_terminal_to_group(
        &self,
        session_id: SessionId,
        target_server_id: &str,
        group_id: u32,
    ) -> anyhow::Result<bool> {
        let server_tx = {
            let state = self.state.read().await;
            let Some(session_server_id) = state.remote_session_index.get(&session_id) else {
                return Ok(false);
            };
            if session_server_id != target_server_id {
                return Ok(false);
            }
            let Some(server) = state.servers.get(session_server_id) else {
                return Ok(false);
            };
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::MoveTabToGroup {
                session_id,
                server_id: None,
                group_id,
            })
            .map_err(|_| anyhow::anyhow!("Failed to forward move terminal group request"))?;
        Ok(true)
    }

    /// Forward a tab close for a tab owned by a shared (relay) server. The
    /// tab's id is the root session id; the owning server is looked up in the
    /// remote session index.
    pub async fn close_remote_tab(&self, tab_id: SessionId) -> anyhow::Result<bool> {
        let server_tx = {
            let state = self.state.read().await;
            let Some(server_id) = state.remote_session_index.get(&tab_id) else {
                return Ok(false);
            };
            let Some(server) = state.servers.get(server_id) else {
                return Ok(false);
            };
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::CloseTab { tab_id })
            .map_err(|_| anyhow::anyhow!("Failed to forward close tab request"))?;
        Ok(true)
    }

    /// Move a session into a target tab when both belong to the same shared
    /// (relay) server. The request is forwarded to that server, which then
    /// owns the move and broadcasts the resulting session/tab state.
    pub async fn move_remote_session_to_tab(
        &self,
        target_server_id: &str,
        session_id: SessionId,
        tab_id: SessionId,
    ) -> anyhow::Result<bool> {
        let server_tx = {
            let state = self.state.read().await;
            let Some(session_server_id) = state.remote_session_index.get(&session_id) else {
                return Ok(false);
            };
            let Some(tab_server_id) = state.remote_session_index.get(&tab_id) else {
                return Ok(false);
            };
            // Both the moving session and the target tab must live on the
            // intended server (mirrors move_remote_terminal_to_group).
            if session_server_id != target_server_id || tab_server_id != target_server_id {
                return Ok(false);
            }
            let Some(server) = state.servers.get(target_server_id) else {
                return Ok(false);
            };
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::MoveSessionToTab {
                server_id: None,
                session_id,
                tab_id,
            })
            .map_err(|_| anyhow::anyhow!("Failed to forward move session to tab request"))?;
        Ok(true)
    }

    /// Update a tab's layout on the shared server that owns the tab.
    pub async fn update_remote_tab_layout(
        &self,
        tab_id: SessionId,
        layout: TerminalLayout,
    ) -> anyhow::Result<bool> {
        let server_tx = {
            let state = self.state.read().await;
            let Some(server_id) = state.remote_session_index.get(&tab_id) else {
                return Ok(false);
            };
            let Some(server) = state.servers.get(server_id) else {
                return Ok(false);
            };
            server.primary_tx.clone()
        };

        server_tx
            .send(Message::UpdateTerminalLayout { tab_id, layout })
            .map_err(|_| anyhow::anyhow!("Failed to forward terminal layout update"))?;
        Ok(true)
    }

    pub async fn is_remote_session(&self, session_id: SessionId) -> bool {
        let state = self.state.read().await;
        state.remote_session_index.contains_key(&session_id)
    }

    /// Return the shared-server id that owns `session_id`, if any.
    pub async fn remote_server_for_session(&self, session_id: SessionId) -> Option<String> {
        let state = self.state.read().await;
        state.remote_session_index.get(&session_id).cloned()
    }

    /// Send a message to a shared server by its server id.
    pub async fn send_to_shared_server(
        &self,
        server_id: &str,
        message: Message,
    ) -> anyhow::Result<()> {
        let state = self.state.read().await;
        let server = state
            .servers
            .get(server_id)
            .ok_or_else(|| anyhow::anyhow!("unknown shared server {}", server_id))?;
        server
            .primary_tx
            .send(message)
            .map_err(|_| anyhow::anyhow!("shared server connection closed"))?;
        Ok(())
    }

    /// Broadcast SessionList to all connected clients.
    pub async fn broadcast_session_list(&self) {
        // A single session-lock sweep feeds all three lists; the previous
        // per-list collection made every broadcast three O(N) sweeps.
        let (local_sessions, local_groups, local_tabs) =
            self.session_manager.session_lists_snapshot().await;

        let sessions = self.merge_shared_server_sessions(local_sessions).await;
        self.broadcast_message(Message::SessionList { sessions })
            .await;

        let groups = self.merge_shared_server_tab_groups(local_groups).await;
        self.broadcast_message(Message::TabGroupList { groups })
            .await;

        let tabs = self.merge_shared_server_tab_infos(local_tabs).await;
        self.broadcast_message(Message::TabInfoList {
            server_id: None,
            tabs,
        })
        .await;
    }

    /// Wake the upstream RelayPrimary state synchronizer after the local
    /// SessionManager changes. This must not be called for snapshots received
    /// from a downstream shared server, otherwise chained relay roles can
    /// feed state notifications back into each other.
    pub fn notify_local_state_changed(&self) {
        self.local_state_revision.send_modify(|revision| {
            *revision = (*revision).wrapping_add(1);
        });
    }

    /// Broadcast RelayServerList to all connected clients.
    pub async fn broadcast_relay_server_list(&self) {
        let servers = self.relay_server_list().await;
        self.broadcast_message(Message::RelayServerList { servers })
            .await;
    }

    /// Broadcast the aggregated terminal profile list to all connected clients.
    pub async fn broadcast_terminal_profile_list(&self) {
        let profiles = self.list_terminal_profiles().await;
        self.broadcast_message(Message::TerminalProfileList { profiles })
            .await;
    }

    /// Send a message to all connected clients and prune any that have disconnected.
    async fn broadcast_message(&self, message: Message) {
        let clients = {
            let state = self.state.read().await;
            state.clients.clone()
        };

        let mut dead_clients = Vec::new();
        for client in clients {
            if client.send(message.clone()).is_err() {
                dead_clients.push(client);
            }
        }

        if !dead_clients.is_empty() {
            let mut state = self.state.write().await;
            state.clients.retain(|client| {
                !dead_clients
                    .iter()
                    .any(|dead_client| dead_client.same_channel(client))
            });
        }
    }

    pub async fn broadcast_client_message(&self, message: Message) {
        self.broadcast_message(message).await;
    }

    async fn run_single_upstream_connection(
        self: Arc<Self>,
        relay_address: &str,
        relay_secret_key: &str,
        relay_strict_tls: bool,
        relay_ca_pem: &str,
        server_name: &str,
        request_port: u16,
    ) -> anyhow::Result<()> {
        let websocket_url = normalize_websocket_url(relay_address);
        let connector =
            upstream_tls_connector_for_url(&websocket_url, relay_strict_tls, relay_ca_pem)?;
        let mut websocket = connect_relay_websocket(websocket_url.as_str(), connector).await?;

        let crypto = perform_handshake(
            &mut websocket,
            &websocket_url,
            relay_secret_key,
            ConnectionKind::RelayPrimary,
            None,
        )
        .await?;

        let registration_frames = encode_relay_websocket_messages(
            Message::RelayRegisterServer {
                name: server_name.to_string(),
                request_port: Some(request_port),
            },
            Some(&crypto),
        )?;
        for frame in registration_frames {
            websocket
                .send(frame)
                .await
                .map_err(|error| anyhow::anyhow!("Failed to register shared server: {}", error))?;
        }

        let mut registration_decoder = TransportBinaryFrameDecoder::default();
        let registration = next_relay_protocol_message(
            &mut websocket,
            Some(&crypto),
            &mut registration_decoder,
            "Relay server closed before registration completed",
        )
        .await?;
        let Message::RelayServerRegistered {
            server_id,
            terminal_tunnel_key,
        } = registration
        else {
            anyhow::bail!(
                "Expected relay registration acknowledgement, received {:?}",
                registration
            );
        };
        let terminal_tunnel_key = terminal_tunnel_key.ok_or_else(|| {
            anyhow::anyhow!("Relay did not provide a transparent terminal tunnel key")
        })?;
        self.set_upstream_terminal_tunnel_key(terminal_tunnel_key)
            .await;

        info!(
            "Registered shared server '{}' with upstream relay {} as {}",
            server_name, websocket_url, server_id
        );

        let (mut write_half, mut read_half) = websocket.split();
        let (relay_tx, mut relay_rx) = mpsc::unbounded_channel::<Message>();
        // Publish the upstream sender + relay host so the coordinator
        // (StartScreenStream on relayed request connections) can request UDP
        // pairings and advertise the relay's media endpoint (spec §7).
        {
            let mut state = self.state.write().await;
            state.upstream_relay_tx = Some(relay_tx.clone());
            state.upstream_relay_host = Some(websocket_url_host(&websocket_url));
        }
        let mut streamed_session_ids = HashSet::new();
        let mut last_sessions_snapshot = None;
        let mut last_tab_groups_snapshot = None;
        let mut last_profiles_snapshot = None;
        let mut last_recent_projects_snapshot = None;
        let mut last_command_history_snapshot = None;
        let mut last_known_apps_snapshot = None;
        let mut last_tab_infos_snapshot = None;
        let mut last_server_info_snapshot = None;
        let mut config_changes = self.config_manager.subscribe();
        let mut local_state_changes = self.local_state_revision.subscribe();
        let mut sync_interval = tokio::time::interval(RELAY_SYNC_INTERVAL);
        let mut transport_binary_decoder = TransportBinaryFrameDecoder::default();
        let mut heartbeat = tokio::time::interval(RELAY_HEARTBEAT_INTERVAL);
        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut last_frame_at = tokio::time::Instant::now();

        send_local_state_if_changed(
            &self.session_manager,
            &relay_tx,
            &mut last_sessions_snapshot,
            &mut last_tab_groups_snapshot,
            &mut last_profiles_snapshot,
            &mut last_recent_projects_snapshot,
            &mut last_command_history_snapshot,
            &mut last_known_apps_snapshot,
            &mut last_tab_infos_snapshot,
            &mut last_server_info_snapshot,
            &self.config_manager,
        )
        .await?;

        loop {
            tokio::select! {
                maybe_message = relay_rx.recv() => {
                    let Some(message) = maybe_message else {
                        break;
                    };
                    let websocket_messages =
                        encode_relay_websocket_messages(message, Some(&crypto))?;
                    for websocket_message in websocket_messages {
                        write_half
                            .send(websocket_message)
                            .await
                            .map_err(|error| anyhow::anyhow!("Failed to write to upstream relay: {}", error))?;
                    }
                }
                frame = read_half.next() => {
                    last_frame_at = tokio::time::Instant::now();
                    let Some(frame) = frame else {
                        break;
                    };
                    let Some(message) = decode_transport_websocket_frame(
                        frame,
                        &mut transport_binary_decoder,
                        Some(&crypto),
                    )? else {
                        continue;
                    };
                    handle_upstream_command(
                        &self,
                        &self.session_manager,
                        &relay_tx,
                        &mut streamed_session_ids,
                        message,
                    )
                    .await?;
                    send_local_state_if_changed(
                        &self.session_manager,
                        &relay_tx,
                        &mut last_sessions_snapshot,
                        &mut last_tab_groups_snapshot,
                        &mut last_profiles_snapshot,
                        &mut last_recent_projects_snapshot,
                        &mut last_command_history_snapshot,
                        &mut last_known_apps_snapshot,
                        &mut last_tab_infos_snapshot,
                        &mut last_server_info_snapshot,
                        &self.config_manager,
                    )
                    .await?;
                }
                _ = config_changes.changed() => {
                    // New relay connection settings only take effect on a fresh
                    // connection, so drop this one and let the connector loop
                    // reconnect with the updated settings immediately.
                    let relay_settings_changed = {
                        let updated = config_changes.borrow();
                        updated.server.relay_address.trim() != relay_address
                            || updated.server.relay_secret_key.trim() != relay_secret_key
                            || updated.server.relay_strict_tls != relay_strict_tls
                            || updated.server.relay_ca_pem.trim() != relay_ca_pem
                    };
                    if relay_settings_changed {
                        info!(
                            "Upstream relay configuration changed; reconnecting to apply new settings"
                        );
                        break;
                    }
                    send_local_state_if_changed(
                        &self.session_manager,
                        &relay_tx,
                        &mut last_sessions_snapshot,
                        &mut last_tab_groups_snapshot,
                        &mut last_profiles_snapshot,
                        &mut last_recent_projects_snapshot,
                        &mut last_command_history_snapshot,
                        &mut last_known_apps_snapshot,
                        &mut last_tab_infos_snapshot,
                        &mut last_server_info_snapshot,
                        &self.config_manager,
                    )
                    .await?;
                }
                changed = local_state_changes.changed() => {
                    if changed.is_err() {
                        break;
                    }
                    send_local_state_if_changed(
                        &self.session_manager,
                        &relay_tx,
                        &mut last_sessions_snapshot,
                        &mut last_tab_groups_snapshot,
                        &mut last_profiles_snapshot,
                        &mut last_recent_projects_snapshot,
                        &mut last_command_history_snapshot,
                        &mut last_known_apps_snapshot,
                        &mut last_tab_infos_snapshot,
                        &mut last_server_info_snapshot,
                        &self.config_manager,
                    )
                    .await?;
                }
                _ = sync_interval.tick() => {
                    send_local_state_if_changed(
                        &self.session_manager,
                        &relay_tx,
                        &mut last_sessions_snapshot,
                        &mut last_tab_groups_snapshot,
                        &mut last_profiles_snapshot,
                        &mut last_recent_projects_snapshot,
                        &mut last_command_history_snapshot,
                        &mut last_known_apps_snapshot,
                        &mut last_tab_infos_snapshot,
                        &mut last_server_info_snapshot,
                        &self.config_manager,
                    )
                    .await?;
                }
                _ = heartbeat.tick() => {
                    // Keep-alive probe: the upstream peer answers pings
                    // automatically, so a dead or half-open link stops
                    // producing inbound frames and trips the idle timeout.
                    write_half
                        .send(WebSocketMessage::Ping(Vec::new()))
                        .await
                        .map_err(|error| anyhow::anyhow!("Failed to ping upstream relay: {}", error))?;
                }
                _ = tokio::time::sleep_until(last_frame_at + RELAY_IDLE_TIMEOUT) => {
                    warn!(
                        "Upstream relay connection idle for {:?} without heartbeat; reconnecting",
                        RELAY_IDLE_TIMEOUT
                    );
                    break;
                }
            }
        }

        for session_id in streamed_session_ids.drain() {
            let _ = self
                .session_manager
                .remove_viewer(session_id, &relay_tx)
                .await;
        }

        {
            let mut state = self.state.write().await;
            state.upstream_relay_tx = None;
            state.upstream_relay_host = None;
        }
        // Fail every pending UDP pairing request so coordinators do not wait
        // out the 10 s timeout on a dead upstream.
        {
            let mut pending = self.pending_udp_allocations.lock().unwrap();
            for (_, tx) in pending.drain() {
                let _ = tx.send(Err("upstream relay connection closed".to_string()));
            }
        }

        Ok(())
    }

    fn spawn_shared_metadata_broadcast(self: &Arc<Self>) {
        let relay_manager = Arc::clone(self);
        tokio::spawn(async move {
            relay_manager.broadcast_session_list().await;
            relay_manager.broadcast_terminal_profile_list().await;
            relay_manager.broadcast_relay_server_list().await;
        });
    }

    async fn update_shared_sessions(self: &Arc<Self>, server_id: &str, sessions: Vec<SessionInfo>) {
        let session_count = sessions.len();
        {
            let mut state = self.state.write().await;
            let Some((server_identity, existing_session_ids)) =
                state.servers.get(server_id).map(|server| {
                    (
                        (server.id.clone(), server.name.clone()),
                        server.sessions.keys().copied().collect::<Vec<_>>(),
                    )
                })
            else {
                return;
            };

            for session_id in existing_session_ids {
                if state
                    .remote_session_index
                    .get(&session_id)
                    .is_some_and(|owner| owner == server_id)
                {
                    state.remote_session_index.remove(&session_id);
                }
            }

            let mut updated_sessions = HashMap::new();
            for mut session in sessions {
                session.server_id = Some(server_identity.0.clone());
                session.server_name = Some(server_identity.1.clone());
                state
                    .remote_session_index
                    .insert(session.id, server_identity.0.clone());
                updated_sessions.insert(session.id, session);
            }

            if let Some(server) = state.servers.get_mut(server_id) {
                server.sessions = updated_sessions;
            }
        }

        trace!(
            "Shared server {} synchronized {} terminal session(s)",
            server_id,
            session_count
        );
        // Do not await the aggregated broadcasts from the RelayPrimary receive
        // stack. That call chain is already deep on Windows and can exhaust a
        // Tokio worker stack while collecting/cloning the local config.
        self.spawn_shared_metadata_broadcast();
    }

    async fn update_shared_tab_groups(self: &Arc<Self>, server_id: &str, groups: Vec<TabGroup>) {
        {
            let mut state = self.state.write().await;
            let Some(server) = state.servers.get_mut(server_id) else {
                return;
            };

            server.tab_groups = groups
                .into_iter()
                .map(|mut group| {
                    group.server_id = Some(server.id.clone());
                    group
                })
                .collect();
        }

        let relay_manager = Arc::clone(self);
        tokio::spawn(async move {
            match relay_manager.list_tab_groups().await {
                Ok(groups) => {
                    relay_manager
                        .broadcast_client_message(Message::TabGroupList { groups })
                        .await;
                }
                Err(error) => {
                    error!("Failed to build aggregated terminal group list: {}", error)
                }
            }
        });
    }

    async fn update_shared_profiles(
        self: &Arc<Self>,
        server_id: &str,
        profiles: Vec<TerminalProfile>,
    ) {
        let profile_count = profiles.len();
        {
            let mut state = self.state.write().await;
            let Some(server) = state.servers.get_mut(server_id) else {
                return;
            };

            server.profiles = profiles
                .into_iter()
                .map(|mut profile| {
                    profile.server_id = Some(server.id.clone());
                    profile.server_name = Some(server.name.clone());
                    profile
                })
                .collect();
        }

        trace!(
            "Shared server {} synchronized {} terminal profile(s)",
            server_id,
            profile_count
        );
        let relay_manager = Arc::clone(self);
        tokio::spawn(async move {
            relay_manager.broadcast_terminal_profile_list().await;
        });
    }

    async fn update_shared_recent_project_directories(
        &self,
        server_id: &str,
        directories: Vec<RecentProjectDirectory>,
    ) {
        {
            let mut state = self.state.write().await;
            let Some(server) = state.servers.get_mut(server_id) else {
                return;
            };
            server.recent_project_directories = directories.clone();
        }

        self.broadcast_client_message(Message::RecentProjectDirectoryList {
            server_id: Some(server_id.to_string()),
            directories,
        })
        .await;
    }

    async fn update_shared_terminal_command_history(
        &self,
        server_id: &str,
        commands: Vec<TerminalCommandHistory>,
    ) {
        {
            let mut state = self.state.write().await;
            let Some(server) = state.servers.get_mut(server_id) else {
                return;
            };
            server.terminal_command_history = commands.clone();
        }

        self.broadcast_client_message(Message::TerminalCommandHistoryList {
            server_id: Some(server_id.to_string()),
            commands,
        })
        .await;
    }

    async fn update_shared_known_apps(&self, server_id: &str, apps: Vec<KnownAppInfo>) {
        {
            let mut state = self.state.write().await;
            let Some(server) = state.servers.get_mut(server_id) else {
                return;
            };
            server.known_apps = apps.clone();
        }

        self.broadcast_client_message(Message::KnownAppList {
            server_id: Some(server_id.to_string()),
            apps,
        })
        .await;
    }

    async fn update_shared_server_info(&self, server_id: &str, info: ServerInfo) {
        {
            let mut state = self.state.write().await;
            let Some(server) = state.servers.get_mut(server_id) else {
                return;
            };
            server.server_info = Some(info.clone());
        }
        self.broadcast_client_message(Message::ServerInfo {
            server_id: Some(server_id.to_string()),
            info,
        })
        .await;
    }

    async fn take_pending_create(&self, server_id: &str) -> Option<ConnectionTx> {
        let mut state = self.state.write().await;
        state
            .servers
            .get_mut(server_id)
            .and_then(|server| server.pending_creates.pop_front())
    }

    async fn broadcast_error_to_clients(&self, code: ErrorCode, message: &str) {
        let clients = {
            let state = self.state.read().await;
            state.clients.clone()
        };

        for client in clients {
            let _ = client.send(Message::Error {
                code: code.clone(),
                message: message.to_string(),
            });
        }
    }
}

async fn handle_upstream_command(
    relay_manager: &Arc<RelayManager>,
    session_manager: &Arc<SessionManager>,
    relay_tx: &ConnectionTx,
    streamed_session_ids: &mut HashSet<SessionId>,
    message: Message,
) -> anyhow::Result<()> {
    match message {
        Message::RelayUdpPairingAllocated {
            request_id,
            pairing,
            host,
            port,
            error,
        } => {
            let outcome = match error {
                Some(error) => Err(error),
                None => Ok((pairing, host, port)),
            };
            relay_manager.complete_udp_pairing_allocation(request_id, outcome);
        }
        Message::SharedPortForwardReady {
            session_id,
            target_port,
            relay_port,
            error,
        } => {
            if let Some(error) = error {
                warn!(
                    "Shared port forward failed for session {:?} target_port={}: {}",
                    session_id, target_port, error
                );
                relay_manager.complete_shared_port_forward(session_id, target_port, Err(error));
            } else {
                info!(
                    "Shared port forward ready for session {:?} target_port={}, relay_port={}",
                    session_id, target_port, relay_port
                );
                relay_manager.complete_shared_port_forward(session_id, target_port, Ok(relay_port));
            }
        }
        Message::RequestReverseProxy { target_port, .. } => {
            info!(
                "Relay requested shared port forward for target port {}",
                target_port
            );
            let response = match relay_manager
                .port_forward_manager
                .ensure_forward(target_port)
                .await
            {
                Ok(lease) => {
                    info!(
                        "Shared port forward ready: target_port={} proxy_port={} reused={}",
                        lease.target_port, lease.proxy_port, lease.reused_existing
                    );
                    Message::ReverseProxyReady {
                        target_port: lease.target_port,
                        proxy_port: lease.proxy_port,
                        reused_existing: lease.reused_existing,
                        settings_ticket: None,
                        error: None,
                    }
                }
                Err(error) => {
                    warn!(
                        "Failed to create shared port forward for target port {}: {}",
                        target_port, error
                    );
                    Message::ReverseProxyReady {
                        target_port,
                        proxy_port: 0,
                        reused_existing: false,
                        settings_ticket: None,
                        error: Some(format!("{error:#}")),
                    }
                }
            };
            relay_tx
                .send(response)
                .map_err(|_| anyhow::anyhow!("Failed to send shared port forward response"))?;
        }
        Message::CreateSession {
            shell,
            profile_id,
            size,
            env,
            cwd,
            title,
            sibling_session_id,
            duplicate_session_id,
            tab_group_id,
            app,
            background,
            foreground,
            ..
        } => {
            match session_manager
                .create_session_with_theme_colors(
                    size,
                    shell,
                    profile_id,
                    env,
                    cwd,
                    sibling_session_id,
                    duplicate_session_id,
                    tab_group_id,
                    title,
                    app,
                    background,
                    foreground,
                )
                .await
            {
                Ok(session_id) => {
                    relay_tx
                        .send(Message::SessionCreated { session_id })
                        .map_err(|_| {
                            anyhow::anyhow!("Failed to send remote session created event")
                        })?;
                }
                Err(error) => send_upstream_session_error(relay_tx, error)?,
            }
        }
        Message::SplitTerminal {
            target_session_id,
            placement,
            size,
            profile_id,
        } => match session_manager
            .split_terminal(target_session_id, placement, size, profile_id)
            .await
        {
            Ok(session_id) => {
                relay_tx
                    .send(Message::SessionCreated { session_id })
                    .map_err(|_| anyhow::anyhow!("Failed to send remote session created event"))?;
            }
            Err(error) => send_upstream_session_error(relay_tx, error)?,
        },
        Message::CloseSession { session_id } => {
            match session_manager.close_session(session_id).await {
                Ok(()) => {
                    streamed_session_ids.remove(&session_id);
                    if let Some(fs_watcher_manager) = session_manager.fs_watcher_manager().await {
                        fs_watcher_manager.remove_session(session_id);
                    }
                }
                Err(error) => send_upstream_session_error(relay_tx, error)?,
            }
        }
        Message::CloseTab { tab_id } => match session_manager.close_tab(tab_id).await {
            Ok(closed_ids) => {
                if let Some(fs_watcher_manager) = session_manager.fs_watcher_manager().await {
                    for session_id in &closed_ids {
                        fs_watcher_manager.remove_session(*session_id);
                    }
                }
                for session_id in &closed_ids {
                    streamed_session_ids.remove(session_id);
                }
            }
            Err(error) => send_upstream_session_error(relay_tx, error)?,
        },
        Message::RenameSession {
            session_id,
            session_name,
        } => {
            if let Err(error) = session_manager
                .rename_session(session_id, session_name)
                .await
            {
                send_upstream_session_error(relay_tx, error)?;
            }
        }
        Message::SetSessionEncoding {
            session_id,
            encoding,
        } => {
            let error = session_manager
                .set_session_encoding(session_id, encoding)
                .await
                .err()
                .map(|error| error.to_string());
            relay_tx
                .send(Message::SetSessionEncodingResult { session_id, error })
                .map_err(|_| anyhow::anyhow!("Failed to send UART encoding result"))?;
        }
        Message::SetTerminalThemeColors {
            session_id,
            background,
            foreground,
        } => {
            let _ = session_manager
                .apply_terminal_theme_colors(session_id, background, foreground)
                .await;
        }
        Message::ClearSessionScreen { session_id } => {
            let error = session_manager
                .clear_session_screen(session_id)
                .await
                .err()
                .map(|error| error.to_string());
            relay_tx
                .send(Message::ClearSessionScreenResult { session_id, error })
                .map_err(|_| anyhow::anyhow!("Failed to send UART screen-clear result"))?;
        }
        Message::ListSessions => {
            relay_tx
                .send(Message::SessionList {
                    sessions: session_manager.list_sessions().await?,
                })
                .map_err(|_| anyhow::anyhow!("Failed to send local session list"))?;
        }
        Message::ListTabGroups => {
            relay_tx
                .send(Message::TabGroupList {
                    groups: session_manager.list_tab_groups().await,
                })
                .map_err(|_| anyhow::anyhow!("Failed to send local terminal group list"))?;
        }
        Message::GetTabInfoList { server_id: _ } => {
            relay_tx
                .send(Message::TabInfoList {
                    server_id: None,
                    tabs: session_manager.list_tab_infos().await,
                })
                .map_err(|_| anyhow::anyhow!("Failed to send local tab info list"))?;
        }
        Message::UpdateTerminalLayout { tab_id, layout } => {
            if let Err(error) = session_manager.update_tab_layout(tab_id, layout).await {
                send_upstream_session_error(relay_tx, error)?;
            } else {
                relay_tx
                    .send(Message::TabInfoList {
                        server_id: None,
                        tabs: session_manager.list_tab_infos().await,
                    })
                    .map_err(|_| anyhow::anyhow!("Failed to send local tab info list"))?;
            }
        }
        Message::CreateTabGroup {
            server_id: _,
            name,
            initial_session_id,
        } => {
            if let Err(error) = session_manager
                .create_tab_group(name, initial_session_id)
                .await
            {
                send_upstream_session_error(relay_tx, error)?;
            }
        }
        Message::DeleteTabGroup {
            server_id: _,
            group_id,
        } => {
            if let Err(error) = session_manager.delete_tab_group(group_id).await {
                send_upstream_session_error(relay_tx, error)?;
            }
        }
        Message::RenameTabGroup {
            server_id: _,
            group_id,
            name,
        } => {
            if let Err(error) = session_manager.rename_tab_group(group_id, name).await {
                send_upstream_session_error(relay_tx, error)?;
            }
        }
        Message::MoveTabToGroup {
            session_id,
            server_id: _,
            group_id,
        } => {
            if let Err(error) = session_manager
                .move_terminal_to_group(session_id, group_id)
                .await
            {
                send_upstream_session_error(relay_tx, error)?;
            }
        }
        Message::MoveSessionToTab {
            session_id,
            server_id: _,
            tab_id,
        } => {
            if let Err(error) = session_manager
                .move_session_to_tab(session_id, tab_id)
                .await
            {
                send_upstream_session_error(relay_tx, error)?;
            }
        }
        Message::ListTerminalProfiles => {
            relay_tx
                .send(Message::TerminalProfileList {
                    profiles: session_manager.list_terminal_profiles().await,
                })
                .map_err(|_| anyhow::anyhow!("Failed to send terminal profile list"))?;
        }
        Message::KnownAppList { .. } | Message::ServerInfo { .. } => {}
        Message::ListRelayServers => {
            relay_tx
                .send(Message::RelayServerList {
                    servers: Vec::new(),
                })
                .map_err(|_| anyhow::anyhow!("Failed to send relay server list"))?;
        }
        Message::Ping => {
            relay_tx
                .send(Message::Pong)
                .map_err(|_| anyhow::anyhow!("Failed to send pong"))?;
        }
        // ── Screen-stream relay coordination (shared-server role) ──────
        // The relay asks us for a token + screen-server UDP port; we
        // consult our local screen-server via IPC and reply.
        Message::RelayScreenTokenRequest {
            request_id,
            source,
            client_label,
        } => {
            info!("Relay requested screen token for source={source} client={client_label}");
            let response = match relay_manager
                .screen_supervisor
                .start_stream(&source, &client_label, None)
                .await
            {
                Ok(start) => {
                    info!(
                        "Screen token granted for source={source}: udp_port={}",
                        start.udp_port
                    );
                    Message::RelayScreenTokenResponse {
                        request_id,
                        token: crate::screen::ctl::token_to_hex(&start.token),
                        screen_udp_port: start.udp_port,
                        error: None,
                    }
                }
                Err(error) => {
                    warn!("Screen token request for source={source} failed: {error:#}");
                    Message::RelayScreenTokenResponse {
                        request_id,
                        token: String::new(),
                        screen_udp_port: 0,
                        error: Some(format!("{error:#}")),
                    }
                }
            };
            relay_tx
                .send(response)
                .map_err(|_| anyhow::anyhow!("Failed to send screen token response"))?;
        }
        Message::RelayScreenTokenRevoke { token } => {
            info!("Relay requested screen token revocation");
            if let Some(arr) = crate::screen::ctl::token_from_hex(&token) {
                let _ = relay_manager.screen_supervisor.stop_stream(arr).await;
            }
        }
        Message::RelayScreenSourcesRequest { request_id } => {
            info!("Relay requested screen sources");
            let response = match relay_manager
                .screen_supervisor
                .list_source_snapshot()
                .await
            {
                Ok((revision, entries)) => Message::RelayScreenSourcesResponse {
                    request_id,
                    sources: entries
                        .iter()
                        .map(crate::screen::source_entry_to_proto)
                        .collect(),
                    revision: Some(revision),
                    error: None,
                },
                Err(error) => Message::RelayScreenSourcesResponse {
                    request_id,
                    sources: Vec::new(),
                    revision: None,
                    error: Some(format!("{error:#}")),
                },
            };
            relay_tx
                .send(response)
                .map_err(|_| anyhow::anyhow!("Failed to send screen sources response"))?;
        }
        Message::Pong
        | Message::SessionCreated { .. }
        | Message::SessionBufferInfo { .. }
        | Message::HistoryRawChunk { .. }
        | Message::HistorySnapshot { .. }
        | Message::ExportSessionRawHistoryResult { .. }
        | Message::Output { .. }
        | Message::PathContentList { .. }
        | Message::FindFilesResult { .. }
        | Message::SearchPathNamesResult { .. }
        | Message::RecentProjectDirectoryList { .. }
        | Message::TerminalCommandHistoryList { .. }
        | Message::AiConversationList { .. }
        | Message::AiConversationMessages { .. }
        | Message::AiConversationMessagesUpdate { .. }
        | Message::GitChangedFilesList { .. }
        | Message::GitLogList { .. }
        | Message::GitDiffResult { .. }
        | Message::SessionClosed { .. }
        | Message::SessionList { .. }
        | Message::TabGroupList { .. }
        | Message::TabInfoList { .. }
        | Message::TerminalProfileList { .. }
        | Message::Error { .. }
        | Message::RelayServerList { .. }
        | Message::RelayRegisterServer { .. }
        | Message::RelayServerRegistered { .. }
        | Message::RelayAllocateUdpPairing { .. }
        | Message::RelayReleaseUdpPairing { .. }
        | Message::RelayScreenTokenResponse { .. }
        | Message::RelayScreenSourcesResponse { .. }
        | Message::Hello { .. }
        | Message::AuthChallenge { .. }
        | Message::AuthResponse { .. }
        | Message::HelloAck { .. }
        | Message::ReverseProxyReady { .. }
        | Message::ListReverseProxies { .. }
        | Message::ReverseProxyList { .. }
        | Message::ViewerJoined { .. }
        | Message::ViewerLeft { .. }
        | Message::TerminalAttentionRequired { .. }
        | Message::TerminalAttentionResolved { .. }
        | Message::TerminalTaskCompleted { .. }
        | Message::DetachSession { .. }
        | Message::FileContent { .. }
        | Message::SaveFileResult { .. }
        | Message::CreatePathResult { .. }
        | Message::DeletePathsResult { .. }
        | Message::PastePathsResult { .. }
        | Message::RenamePathResult { .. }
        | Message::GenerateVtermConfigResult { .. }
        | Message::StartStaticHttpServerResult { .. }
        | Message::GitStageResult { .. }
        | Message::GitUnstageResult { .. }
        | Message::GitDiscardResult { .. }
        | Message::GitCommitResult { .. }
        | Message::GitFetchResult { .. }
        | Message::GitPullResult { .. }
        | Message::GitPushResult { .. }
        | Message::GitRevertResult { .. }
        | Message::GitGenerateCommitMessageAccepted { .. }
        | Message::GitGenerateCommitMessageResult { .. }
        | Message::GitAnalyzeDiffResult { .. }
        | Message::GitRebaseAiResolveResult { .. }
        | Message::DebugLaunchList { .. }
        | Message::DebugLaunchListResult { .. }
        | Message::DebugStart { .. }
        | Message::DebugStartResult { .. }
        | Message::DebugOutput { .. }
        | Message::DebugDap { .. }
        | Message::QuickCommandList { .. }
        | Message::QuickCommandListResult { .. }
        | Message::RunQuickCommand { .. }
        | Message::RunQuickCommandResult { .. }
        | Message::TranscribeAudio { .. }
        | Message::TranscribeAudioResult { .. }
        | Message::PolishVoiceInputText { .. }
        | Message::PolishVoiceInputTextResult { .. }
        | Message::TranslateMessageResult { .. }
        | Message::StreamStart { .. }
        | Message::StreamEnd { .. }
        | Message::FetchHistoryRaw { .. }
        | Message::ExportSessionRawHistory { .. }
        | Message::ApplyEnvironment { .. }
        | Message::ApplyEnvironmentResult { .. }
        | Message::SetNotificationEnabled { .. }
        | Message::SetNotificationEnabledResult { .. }
        | Message::SetAutoConfirmEnabled { .. }
        | Message::SetAutoConfirmEnabledResult { .. }
        | Message::SetSessionEncodingResult { .. }
        | Message::ClearSessionScreenResult { .. }
        | Message::Input { .. }
        | Message::ConfirmSession { .. }
        | Message::SetupMcpApp { .. }
        | Message::SetupMcpAppResult { .. }
        | Message::ConfirmSessionResult { .. }
        | Message::Resize { .. }
        | Message::RemoveRecentProjectDirectory { .. }
        | Message::SetFsWatchers { .. }
        | Message::ListPathContent { .. }
        | Message::FindFiles { .. }
        | Message::SearchPathNames { .. }
        | Message::ListAiConversations { .. }
        | Message::StopWatchingAiConversation { .. }
        | Message::ReadAiConversation { .. }
        | Message::FetchFile { .. }
        | Message::SaveFile { .. }
        | Message::CreatePath { .. }
        | Message::DeletePaths { .. }
        | Message::PastePaths { .. }
        | Message::RenamePath { .. }
        | Message::GenerateVtermConfig { .. }
        | Message::StartStaticHttpServer { .. }
        | Message::GitChangedFiles { .. }
        | Message::GitBranches { .. }
        | Message::GitBranchesResult { .. }
        | Message::GitLog { .. }
        | Message::GitDiff { .. }
        | Message::GitStage { .. }
        | Message::GitUnstage { .. }
        | Message::GitDiscard { .. }
        | Message::GitRevert { .. }
        | Message::GitCommit { .. }
        | Message::GitFetch { .. }
        | Message::GitPull { .. }
        | Message::GitPush { .. }
        | Message::GitGenerateCommitMessageAgent { .. }
        | Message::TranslateMessage { .. }
        | Message::GitAnalyzeDiff { .. }
        | Message::GitStashList { .. }
        | Message::GitStashListResult { .. }
        | Message::GitStashPush { .. }
        | Message::GitStashApply { .. }
        | Message::GitStashPop { .. }
        | Message::GitStashDrop { .. }
        | Message::GitStashResult { .. }
        | Message::GitSubmoduleList { .. }
        | Message::GitSubmoduleListResult { .. }
        | Message::GitCheckout { .. }
        | Message::GitCheckoutResult { .. }
        | Message::GitCreateBranch { .. }
        | Message::GitCreateBranchResult { .. }
        | Message::GitCreateWorktree { .. }
        | Message::GitCreateWorktreeResult { .. }
        | Message::GitDeleteBranch { .. }
        | Message::GitDeleteBranchResult { .. }
        | Message::GitMerge { .. }
        | Message::GitMergeResult { .. }
        | Message::GitRebaseStart { .. }
        | Message::GitRebaseStartResult { .. }
        | Message::GitRebaseApply { .. }
        | Message::GitRebaseApplyResult { .. }
        | Message::GitRebaseConflictContent { .. }
        | Message::GitRebaseConflictContentResult { .. }
        | Message::GitRebaseResolve { .. }
        | Message::GitRebaseResolveResult { .. }
        | Message::GitRebaseContinue { .. }
        | Message::GitRebaseContinueResult { .. }
        | Message::GitRebaseSkip { .. }
        | Message::GitRebaseSkipResult { .. }
        | Message::GitRebaseAbort { .. }
        | Message::GitRebaseAbortResult { .. }
        | Message::GitRebaseAiResolve { .. }
        | Message::GitRebaseAiResolveCancel { .. } => {}
        // Screen-streaming coordination messages: these are forwarded
        // transparently through proxy_remote_request tunnels (the relay
        // never parses them on the upstream state-sync channel). The
        // `server_id` on simulator/emulator lifecycle requests drives
        // client-side relay routing (relay_route_server_id); once the request
        // arrives at the shared server via the transparent tunnel, it is
        // handled locally there. StartScreenStream/ListScreenSources/
        // StopScreenStream instead use relay coordination so a Relay UDP
        // endpoint can be returned.
        Message::ListScreenSources { .. }
        | Message::ScreenSourcesResult { .. }
        | Message::ScreenSourcesChanged { .. }
        | Message::StartScreenStream { .. }
        | Message::ScreenStreamReady { .. }
        | Message::StopScreenStream { .. }
        | Message::ScreenStreamStopped { .. }
        | Message::ScreenStreamNotify { .. }
        | Message::ScreenStreamStats { .. }
        | Message::BootSimulator { .. }
        | Message::BootSimulatorResult { .. }
        | Message::ShutdownSimulator { .. }
        | Message::ShutdownSimulatorResult { .. }
        | Message::CopySimulatorClipboard { .. }
        | Message::CopySimulatorClipboardResult { .. }
        | Message::BootAndroidEmulator { .. }
        | Message::BootAndroidEmulatorResult { .. }
        | Message::AndroidEmulatorBooted { .. }
        | Message::AndroidEmulatorShutdown { .. }
        | Message::AndroidEmulatorOperationFailed { .. }
        | Message::ShutdownAndroidEmulator { .. }
        | Message::ShutdownAndroidEmulatorResult { .. }
        | Message::RotateAndroidDevice { .. }
        | Message::RotateAndroidDeviceResult { .. }
        // File transfers ride opaque RelayRequest tunnels and never reach
        // this handler; listed to keep the match exhaustive.
        | Message::TempFileUploadStart { .. }
        | Message::TempFileUploadResult { .. }
        | Message::UploadStart { .. }
        | Message::UploadResult { .. }
        | Message::DownloadStart { .. }
        | Message::DownloadResult { .. }
        // Shared port forward messages are handled in handle_shared_server_message.
        | Message::RequestSharedPortForward { .. }
        | Message::SharedPortForwardReady { .. } => {}
    }

    Ok(())
}

fn send_upstream_session_error(relay_tx: &ConnectionTx, error: SessionError) -> anyhow::Result<()> {
    let code = match error {
        SessionError::SessionNotFound(_) => ErrorCode::UnknownSession,
        SessionError::ResizeNotAllowed(_)
        | SessionError::PtyError(_)
        | SessionError::FileSystemError(_)
        | SessionError::InvalidState(_)
        | SessionError::Unsupported(_) => ErrorCode::ServerError,
    };

    relay_tx
        .send(Message::Error {
            code,
            message: error.to_string(),
        })
        .map_err(|_| anyhow::anyhow!("Failed to send upstream session error"))
}

async fn send_local_state_if_changed(
    session_manager: &Arc<SessionManager>,
    relay_tx: &ConnectionTx,
    last_sessions_snapshot: &mut Option<Vec<u8>>,
    last_tab_groups_snapshot: &mut Option<Vec<u8>>,
    last_profiles_snapshot: &mut Option<Vec<u8>>,
    last_recent_projects_snapshot: &mut Option<Vec<u8>>,
    last_command_history_snapshot: &mut Option<Vec<u8>>,
    last_known_apps_snapshot: &mut Option<Vec<u8>>,
    last_tab_infos_snapshot: &mut Option<Vec<u8>>,
    last_server_info_snapshot: &mut Option<Vec<u8>>,
    config_manager: &Arc<ConfigManager>,
) -> anyhow::Result<()> {
    // One session-lock sweep feeds sessions, tab groups and tab infos;
    // collecting each list separately tripled the sweep cost on every call.
    let (sessions, groups, tabs) = session_manager.session_lists_snapshot().await;
    let sessions_snapshot = serde_json::to_vec(&sessions)?;
    if last_sessions_snapshot.as_ref() != Some(&sessions_snapshot) {
        info!(
            "Synchronizing {} local terminal session(s) with upstream relay",
            sessions.len()
        );
        relay_tx
            .send(Message::SessionList { sessions })
            .map_err(|_| anyhow::anyhow!("Failed to send session snapshot"))?;
        *last_sessions_snapshot = Some(sessions_snapshot);
    }

    let groups_snapshot = serde_json::to_vec(&groups)?;
    if last_tab_groups_snapshot.as_ref() != Some(&groups_snapshot) {
        relay_tx
            .send(Message::TabGroupList { groups })
            .map_err(|_| anyhow::anyhow!("Failed to send terminal group snapshot"))?;
        *last_tab_groups_snapshot = Some(groups_snapshot);
    }

    let profiles = session_manager.list_terminal_profiles().await;
    let profiles_snapshot = serde_json::to_vec(&profiles)?;
    if last_profiles_snapshot.as_ref() != Some(&profiles_snapshot) {
        info!(
            "Synchronizing {} local terminal profile(s) with upstream relay",
            profiles.len()
        );
        relay_tx
            .send(Message::TerminalProfileList { profiles })
            .map_err(|_| anyhow::anyhow!("Failed to send terminal profile snapshot"))?;
        *last_profiles_snapshot = Some(profiles_snapshot);
    }

    let directories = session_manager.recent_project_directories();
    let directories_snapshot = serde_json::to_vec(&directories)?;
    if last_recent_projects_snapshot.as_ref() != Some(&directories_snapshot) {
        relay_tx
            .send(Message::RecentProjectDirectoryList {
                directories,
                server_id: None,
            })
            .map_err(|_| anyhow::anyhow!("Failed to send recent project directories"))?;
        *last_recent_projects_snapshot = Some(directories_snapshot);
    }

    let commands = session_manager.recent_terminal_commands();
    let commands_snapshot = serde_json::to_vec(&commands)?;
    if last_command_history_snapshot.as_ref() != Some(&commands_snapshot) {
        relay_tx
            .send(Message::TerminalCommandHistoryList {
                commands,
                server_id: None,
            })
            .map_err(|_| anyhow::anyhow!("Failed to send terminal command history"))?;
        *last_command_history_snapshot = Some(commands_snapshot);
    }

    let apps = session_manager.known_apps().await;
    let apps_snapshot = serde_json::to_vec(&apps)?;
    if last_known_apps_snapshot.as_ref() != Some(&apps_snapshot) {
        relay_tx
            .send(Message::KnownAppList {
                apps,
                server_id: None,
            })
            .map_err(|_| anyhow::anyhow!("Failed to send known app list"))?;
        *last_known_apps_snapshot = Some(apps_snapshot);
    }

    let info = crate::config::public_server_info(&config_manager.snapshot().await);
    let info_snapshot = serde_json::to_vec(&info)?;
    if last_server_info_snapshot.as_ref() != Some(&info_snapshot) {
        relay_tx
            .send(Message::ServerInfo {
                server_id: None,
                info,
            })
            .map_err(|_| anyhow::anyhow!("Failed to send server info"))?;
        *last_server_info_snapshot = Some(info_snapshot);
    }

    // Tab layouts (split/merge structure) — sent whenever they change so the
    // upstream relay can propagate the updated layout to clients.
    let tabs_snapshot = serde_json::to_vec(&tabs)?;
    if last_tab_infos_snapshot.as_ref() != Some(&tabs_snapshot) {
        relay_tx
            .send(Message::TabInfoList {
                server_id: None,
                tabs,
            })
            .map_err(|_| anyhow::anyhow!("Failed to send tab info snapshot"))?;
        *last_tab_infos_snapshot = Some(tabs_snapshot);
    }

    Ok(())
}

async fn perform_handshake(
    websocket: &mut RelayWebSocket,
    websocket_url: &str,
    relay_secret_key: &str,
    connection_kind: ConnectionKind,
    auth_token: Option<String>,
) -> anyhow::Result<Arc<Mutex<MessageCrypto>>> {
    let hello = Message::Hello {
        protocol_version: PROTOCOL_VERSION,
        client_version: env!("CARGO_PKG_VERSION").to_string(),
        secret_key_hash: None,
        supports_binary_output: true,
        // Security review §1.7: the shared-server→relay uplink negotiates
        // message encryption like every other connection. The relay is the
        // Noise endpoint here — it decrypts the control plane itself — so
        // nothing changes functionally for the relay, but a middleman on
        // the path can no longer read session titles / tab lists /
        // command history.
        supports_message_encryption: true,
        connection_kind: Some(connection_kind),
        auth_token,
        target_session_id: None,
        relay_terminal_capability: None,
        target_server_id: None,
        relay_request_capability: None,
    };

    websocket
        .send(WebSocketMessage::Text(hello.to_text()?.into()))
        .await
        .map_err(|error| anyhow::anyhow!("Failed to send relay handshake: {}", error))?;

    let mut transport_binary_decoder = TransportBinaryFrameDecoder::default();
    let authentication_challenge = next_relay_protocol_message(
        websocket,
        None,
        &mut transport_binary_decoder,
        &format!(
            "Upstream relay {} closed before authentication challenge",
            websocket_url
        ),
    )
    .await?;

    let mut message_crypto: Option<Arc<Mutex<MessageCrypto>>> = None;
    match authentication_challenge {
        Message::AuthChallenge {
            nonce,
            message_encryption,
        } => {
            if let Some(nonce) = nonce {
                let proof = configured_secret_key(relay_secret_key)
                    .map(|secret_key| secret_key_challenge_response(&secret_key, &nonce));
                websocket
                    .send(WebSocketMessage::Text(
                        Message::AuthResponse { proof }.to_text()?.into(),
                    ))
                    .await
                    .map_err(|error| {
                        anyhow::anyhow!("Failed to send relay authentication response: {}", error)
                    })?;
            }
            if message_encryption {
                // Noise initiator — mirrors vterm-cli's client handshake.
                // The relay-side responder is the generic one in
                // connection.rs; frames before this point (Hello, the
                // challenge, the proof) travelled in the clear inside TLS.
                let mut handshake = MessageCryptoHandshake::new_initiator()?;
                websocket
                    .send(WebSocketMessage::Binary(handshake.write_frame()?.into()))
                    .await
                    .map_err(|error| {
                        anyhow::anyhow!("Failed to send encryption handshake: {}", error)
                    })?;
                match websocket.next().await {
                    Some(Ok(WebSocketMessage::Binary(data))) => handshake.read_frame(&data)?,
                    Some(Ok(other)) => anyhow::bail!(
                        "Unexpected upstream encryption handshake frame: {:?}",
                        other
                    ),
                    Some(Err(error)) => {
                        anyhow::bail!("Failed to read upstream encryption handshake: {}", error)
                    }
                    None => anyhow::bail!("Upstream relay closed during encryption handshake"),
                }
                websocket
                    .send(WebSocketMessage::Binary(handshake.write_frame()?.into()))
                    .await
                    .map_err(|error| {
                        anyhow::anyhow!("Failed to finish encryption handshake: {}", error)
                    })?;
                message_crypto = Some(Arc::new(Mutex::new(handshake.into_transport()?)));
            }
        }
        Message::Error { code, message } => {
            anyhow::bail!("Upstream relay rejected handshake: {}: {}", code, message)
        }
        other => anyhow::bail!("Unexpected relay authentication challenge: {:?}", other),
    }

    // Fail-closed (§1.7): a relay that did not negotiate message encryption
    // is either pre-hardening or tampered with — refuse the uplink either
    // way. Same-repo relays follow the client's flag, so lockstep releases
    // keep working; only older relay builds are rejected.
    let Some(crypto) = message_crypto else {
        anyhow::bail!(
            "Upstream relay {} did not negotiate message encryption; refusing plaintext uplink",
            websocket_url
        );
    };

    // The HelloAck arrives encrypted once the Noise handshake completed.
    let handshake_response = next_relay_protocol_message(
        websocket,
        Some(&crypto),
        &mut transport_binary_decoder,
        &format!(
            "Upstream relay {} closed before completing handshake",
            websocket_url
        ),
    )
    .await?;

    match handshake_response {
        Message::HelloAck {
            protocol_version,
            server_version,
            ..
        } => {
            if protocol_version != PROTOCOL_VERSION {
                warn!(
                    "Relay protocol mismatch. Local server uses {}, upstream uses {} ({}); continuing anyway",
                    PROTOCOL_VERSION, protocol_version, server_version
                );
            }
        }
        Message::Error { code, message } => {
            anyhow::bail!("Upstream relay rejected handshake: {}: {}", code, message)
        }
        other => anyhow::bail!("Unexpected relay handshake response: {:?}", other),
    }

    Ok(crypto)
}

async fn next_relay_protocol_message<T>(
    websocket: &mut T,
    crypto: Option<&Arc<Mutex<MessageCrypto>>>,
    transport_binary_decoder: &mut TransportBinaryFrameDecoder,
    closed_message: &str,
) -> anyhow::Result<Message>
where
    T: futures_util::Stream<Item = Result<WebSocketMessage, tokio_tungstenite::tungstenite::Error>>
        + Unpin,
{
    loop {
        let Some(frame) = websocket.next().await else {
            anyhow::bail!(closed_message.to_string());
        };
        match frame {
            Ok(WebSocketMessage::Text(text)) => {
                if crypto.is_some() {
                    anyhow::bail!(
                        "Unencrypted text frame from relay after encryption was negotiated"
                    );
                }
                return Message::from_text(text.as_ref()).map_err(|error| {
                    anyhow::anyhow!("Failed to decode relay text message: {}", error)
                });
            }
            Ok(WebSocketMessage::Binary(data)) => {
                match crate::connection::decode_websocket_binary_frame(
                    &data,
                    crypto,
                    transport_binary_decoder,
                ) {
                    Ok(Some(message)) => return Ok(message),
                    // Fragment of a larger message — more chunks to come.
                    Ok(None) => continue,
                    Err(error) => anyhow::bail!("Failed to decode relay binary message: {}", error),
                }
            }
            Ok(WebSocketMessage::Close(_)) => anyhow::bail!("Relay connection closed"),
            Ok(WebSocketMessage::Ping(_)) | Ok(WebSocketMessage::Pong(_)) => continue,
            Ok(_) => anyhow::bail!("Unexpected relay WebSocket frame"),
            Err(error) => return Err(anyhow::anyhow!("Relay transport error: {}", error)),
        }
    }
}

fn decode_websocket_frame(
    frame: Result<WebSocketMessage, tokio_tungstenite::tungstenite::Error>,
) -> anyhow::Result<Option<Message>> {
    match frame {
        Ok(WebSocketMessage::Text(text)) => Message::from_text(text.as_ref())
            .map(Some)
            .map_err(|error| anyhow::anyhow!("Failed to decode relay text message: {}", error)),
        Ok(WebSocketMessage::Binary(data)) => Message::from_transport_binary(&data)
            .map(Some)
            .map_err(|error| anyhow::anyhow!("Failed to decode relay binary message: {}", error)),
        Ok(WebSocketMessage::Close(_)) => anyhow::bail!("Relay connection closed"),
        Ok(WebSocketMessage::Ping(_)) | Ok(WebSocketMessage::Pong(_)) => Ok(None),
        Ok(_) => anyhow::bail!("Unexpected relay WebSocket frame"),
        Err(error) => Err(anyhow::anyhow!("Relay transport error: {}", error)),
    }
}

fn decode_transport_websocket_frame(
    frame: Result<WebSocketMessage, tokio_tungstenite::tungstenite::Error>,
    transport_binary_decoder: &mut TransportBinaryFrameDecoder,
    crypto: Option<&Arc<Mutex<MessageCrypto>>>,
) -> anyhow::Result<Option<Message>> {
    match frame {
        Ok(WebSocketMessage::Text(text)) => {
            if crypto.is_some() {
                anyhow::bail!("Unencrypted text frame from relay after encryption was negotiated");
            }
            Message::from_text(text.as_ref())
                .map(Some)
                .map_err(|error| anyhow::anyhow!("Failed to decode relay text message: {}", error))
        }
        Ok(WebSocketMessage::Binary(data)) => crate::connection::decode_websocket_binary_frame(
            &data,
            crypto,
            transport_binary_decoder,
        )
        .map_err(|error| anyhow::anyhow!("Failed to decode relay binary message: {}", error)),
        Ok(WebSocketMessage::Close(_)) => anyhow::bail!("Relay connection closed"),
        Ok(WebSocketMessage::Ping(_)) | Ok(WebSocketMessage::Pong(_)) => Ok(Some(Message::Pong)),
        Ok(_) => anyhow::bail!("Unexpected relay WebSocket frame"),
        Err(error) => Err(anyhow::anyhow!("Relay transport error: {}", error)),
    }
}

fn encode_relay_websocket_messages(
    message: Message,
    crypto: Option<&Arc<Mutex<MessageCrypto>>>,
) -> anyhow::Result<Vec<WebSocketMessage>> {
    crate::connection::encode_websocket_messages(message, crypto, true)
}

trait RelayTransportIo: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T> RelayTransportIo for T where T: AsyncRead + AsyncWrite + Send + Unpin {}
type RelayTransport = Box<dyn RelayTransportIo>;
type RelayWebSocket = WebSocketStream<MaybeTlsStream<RelayTransport>>;

fn websocket_tcp_target(websocket_url: &str) -> anyhow::Result<(String, u16)> {
    let uri = websocket_url
        .parse::<tokio_tungstenite::tungstenite::http::Uri>()
        .map_err(|error| anyhow::anyhow!("Invalid relay WebSocket URL: {error}"))?;
    let host = uri
        .host()
        .ok_or_else(|| anyhow::anyhow!("Relay WebSocket URL has no host"))?
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_string();
    let port = match (uri.port_u16(), uri.scheme_str()) {
        (Some(port), _) => port,
        (None, Some("ws")) => 80,
        (None, Some("wss")) => 443,
        (None, _) => anyhow::bail!("Relay WebSocket URL has no port"),
    };
    Ok((host, port))
}

async fn connect_direct_websocket(
    websocket_url: &str,
    connector: Option<Connector>,
) -> anyhow::Result<RelayWebSocket> {
    let (host, port) = websocket_tcp_target(websocket_url)?;
    let host = if host.contains(':') {
        format!("[{host}]")
    } else {
        host
    };
    let stream = TcpStream::connect((host.as_str(), port)).await?;
    let stream: RelayTransport = Box::new(stream);
    let (websocket, _) =
        client_async_tls_with_config(websocket_url, stream, None, connector).await?;
    Ok(websocket)
}

async fn connect_relay_websocket(
    websocket_url: &str,
    connector: Option<Connector>,
) -> anyhow::Result<RelayWebSocket> {
    match connect_direct_websocket(websocket_url, connector.clone()).await {
        Ok(websocket) => Ok(websocket),
        Err(direct_error) => {
            #[cfg(target_os = "macos")]
            if local_network::is_local_network_block_error(&direct_error)
                && local_network::relay_target_is_lan(websocket_url)
            {
                warn!(
                    "Direct relay connection failed with a possible macOS Local Network block; retrying through a disclaimed network process"
                );
                let recovery = async {
                    let (host, port) = websocket_tcp_target(websocket_url)?;
                    let stream = local_network::recovery_tcp_stream(&host, port).await?;
                    let stream: RelayTransport = Box::new(stream);
                    let (websocket, _) =
                        client_async_tls_with_config(websocket_url, stream, None, connector)
                            .await?;
                    anyhow::Ok(websocket)
                }
                .await;
                match recovery {
                    Ok(websocket) => {
                        info!("Relay WebSocket connected through Local Network recovery process");
                        return Ok(websocket);
                    }
                    Err(recovery_error) => {
                        warn!(
                            "Local Network recovery connection failed: {recovery_error}; preserving the direct connection error for retry classification"
                        );
                    }
                }
            }
            Err(direct_error)
        }
    }
}

fn normalize_websocket_url(addr: &str) -> String {
    if addr.starts_with("ws://") || addr.starts_with("wss://") {
        addr.to_string()
    } else if let Some(rest) = addr.strip_prefix("ws:/") {
        format!("ws://{}", rest.trim_start_matches('/'))
    } else if let Some(rest) = addr.strip_prefix("wss:/") {
        format!("wss://{}", rest.trim_start_matches('/'))
    } else if let Some(rest) = addr.strip_prefix("http://") {
        format!("ws://{}", rest)
    } else if let Some(rest) = addr.strip_prefix("https://") {
        format!("wss://{}", rest)
    } else if let Some(rest) = addr.strip_prefix("http:/") {
        format!("ws://{}", rest.trim_start_matches('/'))
    } else if let Some(rest) = addr.strip_prefix("https:/") {
        format!("wss://{}", rest.trim_start_matches('/'))
    } else {
        format!("wss://{}", addr)
    }
}

fn format_socket_host(ip: std::net::IpAddr) -> String {
    match ip {
        std::net::IpAddr::V4(ip) => ip.to_string(),
        std::net::IpAddr::V6(ip) => format!("[{}]", ip),
    }
}

/// Extract the host portion (without port or path) from a websocket URL,
/// e.g. `wss://relay.example.com:7878/ws` → `relay.example.com`.
fn websocket_url_host(url: &str) -> String {
    let without_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let authority = without_scheme.split('/').next().unwrap_or(without_scheme);
    if authority.starts_with('[') {
        // Bracketed IPv6 literal: keep through the closing bracket.
        if let Some(end) = authority.find(']') {
            return authority[..=end].to_string();
        }
        return authority.to_string();
    }
    authority.split(':').next().unwrap_or(authority).to_string()
}

fn upstream_tls_connector_for_url(
    websocket_url: &str,
    strict_tls: bool,
    ca_pem: &str,
) -> anyhow::Result<Option<Connector>> {
    if !strict_tls {
        return tls_connector_for_url(websocket_url);
    }
    if !websocket_url.starts_with("wss://") {
        anyhow::bail!("Strict CA verification requires a wss:// relay URL");
    }

    let config = vterm_common::tls::build_strict_rustls_client_config(websocket_url, ca_pem)
        .map_err(|error| anyhow::anyhow!("Failed to build relay TLS connector: {}", error))?;
    Ok(Some(Connector::Rustls(config)))
}

/// Terminal and request callbacks retain the existing certificate policy.
fn tls_connector_for_url(websocket_url: &str) -> anyhow::Result<Option<Connector>> {
    if !websocket_url.starts_with("wss://") {
        return Ok(None);
    }

    let config = vterm_common::tls::build_rustls_client_config(websocket_url)
        .map_err(|error| anyhow::anyhow!("Failed to build relay TLS connector: {}", error))?;
    Ok(Some(Connector::Rustls(config)))
}

fn configured_secret_key(relay_secret_key: &str) -> Option<String> {
    let relay_secret_key = relay_secret_key.trim();
    (!relay_secret_key.is_empty()).then(|| relay_secret_key.to_string())
}

fn normalize_server_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        "Shared Server".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod local_network_recovery_tests {
    use super::{connect_direct_websocket, websocket_tcp_target};
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::Message;

    #[test]
    fn websocket_tcp_targets_include_default_ports_and_ipv6_literals() {
        assert_eq!(
            websocket_tcp_target("wss://192.168.1.155:7878/ws").unwrap(),
            ("192.168.1.155".to_string(), 7878)
        );
        assert_eq!(
            websocket_tcp_target("ws://relay.example.com/ws").unwrap(),
            ("relay.example.com".to_string(), 80)
        );
        assert_eq!(
            websocket_tcp_target("wss://[fd00::5]/ws").unwrap(),
            ("fd00::5".to_string(), 443)
        );
    }

    #[tokio::test]
    async fn direct_websocket_uses_the_relay_transport() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut websocket = tokio_tungstenite::accept_async(stream).await.unwrap();
            let message = websocket.next().await.unwrap().unwrap();
            websocket.send(message).await.unwrap();
        });

        let mut websocket = connect_direct_websocket(&format!("ws://{address}"), None)
            .await
            .unwrap();
        websocket
            .send(Message::Text("relay-transport-test".into()))
            .await
            .unwrap();
        let response = websocket.next().await.unwrap().unwrap();
        assert_eq!(response, Message::Text("relay-transport-test".into()));
        server.await.unwrap();
    }
}

#[cfg(test)]
mod transparent_terminal_tests {
    use super::{
        create_relay_request_capability, create_relay_terminal_capability, handle_upstream_command,
        merge_global_all_group, next_relay_protocol_message, verify_relay_request_capability,
        verify_relay_terminal_capability, RelayManager, RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS,
    };
    use crate::config::ConfigManager;
    use crate::connection::ConnectionRegistry;
    use crate::port_forward::PortForwardManager;
    use crate::screen::ScreenServerSupervisor;
    use crate::session::SessionManager;
    use std::collections::{HashMap, HashSet};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::mpsc;
    use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message as WebSocketMessage};
    use vterm_common::{
        SessionId, TabGroup, TabId, TerminalSize, TAB_GROUP_ALL_ID, TAB_GROUP_UNGROUPED_ID,
    };
    use vterm_core::emulator::Color;
    use vterm_core::protocol::Message;
    use vterm_core::protocol::TransportBinaryFrameDecoder;

    fn test_port_forward_manager() -> Arc<PortForwardManager> {
        test_port_forward_manager_with_range(62000, 62999)
    }

    fn test_port_forward_manager_with_range(start: u16, end: u16) -> Arc<PortForwardManager> {
        let certified_key = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(
            certified_key.signing_key.serialize_der().into(),
        );
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certified_key.cert.der().clone()], key)
            .unwrap();
        let manager = PortForwardManager::with_port_range(
            start,
            end,
            tokio_rustls::TlsAcceptor::from(Arc::new(config)),
        );
        // The policy default is off in headless builds; tests using this
        // helper exercise the forwarding path, not the default.
        manager.update_settings(true, 0);
        Arc::new(manager)
    }

    fn available_test_port() -> u16 {
        std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    async fn relay_manager_for_test() -> (Arc<RelayManager>, Arc<SessionManager>) {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let config_path =
            std::env::temp_dir().join(format!("relay-registration-tests-{unique_suffix}.json"));
        let config_manager = Arc::new(ConfigManager::load_from_path(config_path).await.unwrap());
        let session_manager = Arc::new(SessionManager::new(Arc::clone(&config_manager)));
        let relay_manager = Arc::new(RelayManager::new(
            Arc::clone(&config_manager),
            Arc::clone(&session_manager),
            Arc::new(ScreenServerSupervisor::new(config_manager)),
            test_port_forward_manager(),
            Arc::new(ConnectionRegistry::new()),
        ));
        (relay_manager, session_manager)
    }

    async fn session_info_template(
        session_manager: &Arc<SessionManager>,
    ) -> vterm_common::SessionInfo {
        session_manager
            .create_session(
                TerminalSize::new(24, 80),
                String::new(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        session_manager.list_sessions().await.unwrap().remove(0)
    }

    #[test]
    fn relay_primary_strict_tls_validates_connection_options() {
        let error = super::upstream_tls_connector_for_url("ws://relay.example.com", true, "")
            .err()
            .unwrap();
        assert!(error.to_string().contains("requires a wss://"));

        let error = super::upstream_tls_connector_for_url("wss://relay.example.com", true, "")
            .err()
            .unwrap();
        assert!(error.to_string().contains("requires a Root CA PEM"));

        let identity =
            rcgen::generate_simple_self_signed(vec!["relay.example.com".into()]).unwrap();
        assert!(super::upstream_tls_connector_for_url(
            "wss://relay.example.com",
            true,
            &identity.cert.pem(),
        )
        .unwrap()
        .is_some());

        assert!(
            super::upstream_tls_connector_for_url("ws://relay.example.com", false, "")
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn relay_protocol_message_skips_websocket_heartbeats() {
        let registration = Message::RelayServerRegistered {
            server_id: "shared-server".to_string(),
            terminal_tunnel_key: Some("tunnel-key".to_string()),
        };
        let mut frames = futures_util::stream::iter(vec![
            Ok::<_, WebSocketError>(WebSocketMessage::Ping(Vec::new())),
            Ok(WebSocketMessage::Pong(Vec::new())),
            Ok(WebSocketMessage::Text(
                registration.to_text().unwrap().into(),
            )),
        ]);

        let message = next_relay_protocol_message(
            &mut frames,
            None,
            &mut TransportBinaryFrameDecoder::default(),
            "closed",
        )
        .await
        .unwrap();

        assert!(matches!(
            message,
            Message::RelayServerRegistered {
                server_id,
                terminal_tunnel_key: Some(terminal_tunnel_key),
            } if server_id == "shared-server" && terminal_tunnel_key == "tunnel-key"
        ));
    }

    #[test]
    fn capability_is_bound_to_secret_and_session() {
        let now = 1_000_000;
        let expires_at = now + 60;
        let session_id = SessionId::new();
        let capability =
            create_relay_terminal_capability("relay-secret", session_id, expires_at, "nonce-1");

        assert_eq!(
            verify_relay_terminal_capability("relay-secret", session_id, &capability, now),
            Some(expires_at)
        );
        assert!(
            verify_relay_terminal_capability("different-secret", session_id, &capability, now)
                .is_none()
        );
        assert!(verify_relay_terminal_capability(
            "relay-secret",
            SessionId::new(),
            &capability,
            now
        )
        .is_none());
    }

    #[test]
    fn capability_enforces_symmetric_twelve_hour_clock_skew() {
        let issued_at = 1_000_000;
        let expires_at = issued_at + super::RELAY_TERMINAL_CAPABILITY_LIFETIME_SECS;
        let session_id = SessionId::new();
        let capability =
            create_relay_terminal_capability("relay-secret", session_id, expires_at, "nonce-2");

        for now in [
            issued_at - RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS,
            issued_at + RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS,
        ] {
            assert_eq!(
                verify_relay_terminal_capability("relay-secret", session_id, &capability, now),
                Some(expires_at)
            );
        }
        for now in [
            issued_at - RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS - 1,
            issued_at + RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS + 1,
        ] {
            assert!(
                verify_relay_terminal_capability("relay-secret", session_id, &capability, now)
                    .is_none()
            );
        }
        assert!(verify_relay_terminal_capability(
            "relay-secret",
            session_id,
            "not-a-capability",
            issued_at
        )
        .is_none());
    }

    #[test]
    fn request_capability_is_bound_to_secret_server_and_clock_skew() {
        let issued_at = 2_000_000;
        let expires_at = issued_at + super::RELAY_TERMINAL_CAPABILITY_LIFETIME_SECS;
        let capability = create_relay_request_capability(
            "relay-secret",
            "shared-server-a",
            expires_at,
            "request-nonce",
        );
        assert_eq!(
            verify_relay_request_capability(
                "relay-secret",
                "shared-server-a",
                &capability,
                issued_at,
            ),
            Some(expires_at)
        );
        assert!(verify_relay_request_capability(
            "different-secret",
            "shared-server-a",
            &capability,
            issued_at,
        )
        .is_none());
        assert!(verify_relay_request_capability(
            "relay-secret",
            "shared-server-b",
            &capability,
            issued_at,
        )
        .is_none());
        for now in [
            issued_at - RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS,
            issued_at + RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS,
        ] {
            assert_eq!(
                verify_relay_request_capability(
                    "relay-secret",
                    "shared-server-a",
                    &capability,
                    now,
                ),
                Some(expires_at)
            );
        }
        for now in [
            issued_at - RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS - 1,
            issued_at + RELAY_CAPABILITY_MAX_CLOCK_SKEW_SECS + 1,
        ] {
            assert!(verify_relay_request_capability(
                "relay-secret",
                "shared-server-a",
                &capability,
                now,
            )
            .is_none());
        }
    }

    #[test]
    fn tab_groups_have_one_global_all_and_server_scoped_ungrouped_groups() {
        let local_tab = TabId::new();
        let shared_tab = TabId::new();
        let groups = merge_global_all_group(vec![
            TabGroup {
                server_id: Some("local-server".to_string()),
                id: TAB_GROUP_ALL_ID,
                name: "All".to_string(),
                builtin: true,
                tab_ids: vec![local_tab],
            },
            TabGroup {
                server_id: Some("local-server".to_string()),
                id: TAB_GROUP_UNGROUPED_ID,
                name: "Ungrouped".to_string(),
                builtin: true,
                tab_ids: vec![local_tab],
            },
            TabGroup {
                server_id: Some("shared-server".to_string()),
                id: TAB_GROUP_ALL_ID,
                name: "All".to_string(),
                builtin: true,
                tab_ids: vec![shared_tab],
            },
            TabGroup {
                server_id: Some("shared-server".to_string()),
                id: TAB_GROUP_UNGROUPED_ID,
                name: "Ungrouped".to_string(),
                builtin: true,
                tab_ids: vec![shared_tab],
            },
        ]);

        let all_groups = groups
            .iter()
            .filter(|group| group.id == TAB_GROUP_ALL_ID)
            .collect::<Vec<_>>();
        assert_eq!(all_groups.len(), 1);
        assert_eq!(all_groups[0].server_id, None);
        assert_eq!(all_groups[0].tab_ids.len(), 2);
        assert!(all_groups[0].tab_ids.contains(&local_tab));
        assert!(all_groups[0].tab_ids.contains(&shared_tab));

        let ungrouped_server_ids = groups
            .iter()
            .filter(|group| group.id == TAB_GROUP_UNGROUPED_ID)
            .map(|group| group.server_id.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(
            ungrouped_server_ids,
            vec![Some("local-server"), Some("shared-server")]
        );
    }

    #[tokio::test]
    async fn registering_same_terminal_endpoint_replaces_stale_registration() {
        let (relay_manager, session_manager) = relay_manager_for_test().await;
        let mut session = session_info_template(&session_manager).await;
        let local_session_id = session.id;
        session.id = SessionId::new();
        session.tab_id = session.id;
        let peer_ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let (old_tx, _old_rx) = mpsc::unbounded_channel();
        let (old_server_id, _) = relay_manager
            .register_shared_server(
                "Shared".to_string(),
                Some(7878),
                SocketAddr::new(peer_ip, 12345),
                old_tx,
            )
            .await;
        relay_manager
            .update_shared_sessions(&old_server_id, vec![session.clone()])
            .await;

        let (new_tx, _new_rx) = mpsc::unbounded_channel();
        let (new_server_id, _) = relay_manager
            .register_shared_server(
                "Shared".to_string(),
                Some(7878),
                SocketAddr::new(peer_ip, 12346),
                new_tx,
            )
            .await;

        assert_ne!(old_server_id, new_server_id);
        assert_eq!(relay_manager.relay_server_list().await.len(), 1);
        assert_eq!(
            relay_manager.remote_server_for_session(session.id).await,
            None
        );

        relay_manager
            .update_shared_sessions(&new_server_id, vec![session.clone()])
            .await;
        relay_manager.unregister_shared_server(&old_server_id).await;
        assert_eq!(
            relay_manager.remote_server_for_session(session.id).await,
            Some(new_server_id)
        );
        session_manager
            .close_session(local_session_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn stale_session_updates_and_unregister_preserve_new_owner() {
        let (relay_manager, session_manager) = relay_manager_for_test().await;
        let template = session_info_template(&session_manager).await;
        let mut updated_session = template.clone();
        updated_session.id = SessionId::new();
        updated_session.tab_id = updated_session.id;
        let mut unregistered_session = template.clone();
        unregistered_session.id = SessionId::new();
        unregistered_session.tab_id = unregistered_session.id;
        let peer_ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let (old_tx, _old_rx) = mpsc::unbounded_channel();
        let (old_server_id, _) = relay_manager
            .register_shared_server(
                "Old endpoint".to_string(),
                Some(7878),
                SocketAddr::new(peer_ip, 12345),
                old_tx,
            )
            .await;
        let (new_tx, _new_rx) = mpsc::unbounded_channel();
        let (new_server_id, _) = relay_manager
            .register_shared_server(
                "New endpoint".to_string(),
                Some(7879),
                SocketAddr::new(peer_ip, 12346),
                new_tx,
            )
            .await;
        relay_manager
            .update_shared_sessions(
                &old_server_id,
                vec![updated_session.clone(), unregistered_session.clone()],
            )
            .await;
        relay_manager
            .update_shared_sessions(
                &new_server_id,
                vec![updated_session.clone(), unregistered_session.clone()],
            )
            .await;

        relay_manager
            .update_shared_sessions(&old_server_id, vec![unregistered_session.clone()])
            .await;
        assert_eq!(
            relay_manager
                .remote_server_for_session(updated_session.id)
                .await,
            Some(new_server_id.clone())
        );

        relay_manager
            .update_shared_sessions(&new_server_id, vec![unregistered_session.clone()])
            .await;
        relay_manager.unregister_shared_server(&old_server_id).await;
        assert_eq!(
            relay_manager
                .remote_server_for_session(unregistered_session.id)
                .await,
            Some(new_server_id)
        );
        session_manager.close_session(template.id).await.unwrap();
    }

    #[tokio::test]
    async fn shared_tab_order_survives_session_metadata_updates() {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let config_path =
            std::env::temp_dir().join(format!("relay-tab-order-tests-{unique_suffix}.json"));
        let config_manager = Arc::new(
            ConfigManager::load_from_path(PathBuf::from(config_path))
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(Arc::clone(&config_manager)));
        session_manager
            .create_session(
                TerminalSize::new(24, 80),
                String::new(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let template = session_manager.list_sessions().await.unwrap().remove(0);

        let mut first = template.clone();
        first.id = SessionId::new();
        first.tab_id = first.id;
        first.sequence = 10;
        let mut second = template.clone();
        second.id = SessionId::new();
        second.tab_id = second.id;
        second.sequence = 20;
        let mut third = template;
        third.id = SessionId::new();
        third.tab_id = third.id;
        third.sequence = 30;
        let expected_order = vec![first.tab_id, second.tab_id, third.tab_id];

        let relay_manager = Arc::new(RelayManager::new(
            config_manager.clone(),
            session_manager,
            Arc::new(ScreenServerSupervisor::new(config_manager)),
            test_port_forward_manager(),
            Arc::new(ConnectionRegistry::new()),
        ));
        let (shared_tx, _shared_rx) = mpsc::unbounded_channel();
        let (server_id, _) = relay_manager
            .register_shared_server(
                "Ordered Shared".to_string(),
                Some(7878),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 12346),
                shared_tx,
            )
            .await;

        relay_manager
            .update_shared_sessions(
                &server_id,
                vec![third.clone(), first.clone(), second.clone()],
            )
            .await;
        let initial_order = relay_manager
            .list_all_tab_infos()
            .await
            .into_iter()
            .filter(|tab| tab.server_id.as_deref() == Some(server_id.as_str()))
            .map(|tab| tab.tab_id)
            .collect::<Vec<_>>();

        first.current_working_path = Some("updated/cwd".to_string());
        second.viewer_count = second.viewer_count.saturating_add(1);
        relay_manager
            .update_shared_sessions(&server_id, vec![second, third, first])
            .await;
        let updated_order = relay_manager
            .list_all_tab_infos()
            .await
            .into_iter()
            .filter(|tab| tab.server_id.as_deref() == Some(server_id.as_str()))
            .map(|tab| tab.tab_id)
            .collect::<Vec<_>>();

        assert_eq!(initial_order, expected_order);
        assert_eq!(updated_order, expected_order);
    }

    #[tokio::test]
    async fn upstream_terminal_theme_colors_reach_shared_session() {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let config_path =
            std::env::temp_dir().join(format!("relay-theme-tests-{unique_suffix}.json"));
        let config_manager = Arc::new(
            ConfigManager::load_from_path(PathBuf::from(config_path))
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(Arc::clone(&config_manager)));
        let relay_manager = Arc::new(RelayManager::new(
            config_manager.clone(),
            Arc::clone(&session_manager),
            Arc::new(ScreenServerSupervisor::new(config_manager)),
            test_port_forward_manager(),
            Arc::new(ConnectionRegistry::new()),
        ));
        let (upstream_tx, mut upstream_rx) = mpsc::unbounded_channel();
        let mut streamed_session_ids = HashSet::new();

        handle_upstream_command(
            &relay_manager,
            &session_manager,
            &upstream_tx,
            &mut streamed_session_ids,
            Message::CreateSession {
                shell: String::new(),
                profile_id: None,
                size: TerminalSize::new(24, 80),
                env: HashMap::new(),
                cwd: None,
                title: None,
                sibling_session_id: None,
                server_id: None,
                tab_group_id: None,
                duplicate_session_id: None,
                app: None,
                background: Some([0x10, 0x11, 0x12]),
                foreground: Some([0x20, 0x21, 0x22]),
            },
        )
        .await
        .unwrap();
        let session_id = match upstream_rx.recv().await {
            Some(Message::SessionCreated { session_id }) => session_id,
            other => panic!("unexpected create response: {other:?}"),
        };
        let session = session_manager.get_session(session_id).await.unwrap();
        assert_eq!(
            session.lock().await.terminal.theme_default_background(),
            Some(Color::Rgb(0x10, 0x11, 0x12))
        );
        assert_eq!(
            session.lock().await.terminal.theme_default_foreground(),
            Some(Color::Rgb(0x20, 0x21, 0x22))
        );

        session
            .lock()
            .await
            .terminal
            .process(b"\x1b]10;rgb:aa/bb/cc\x1b\\\x1b]11;rgb:dd/ee/ff\x1b\\");

        handle_upstream_command(
            &relay_manager,
            &session_manager,
            &upstream_tx,
            &mut streamed_session_ids,
            Message::SetTerminalThemeColors {
                session_id,
                background: Some([0x30, 0x31, 0x32]),
                foreground: Some([0x40, 0x41, 0x42]),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            session.lock().await.terminal.theme_default_background(),
            Some(Color::Rgb(0x30, 0x31, 0x32))
        );
        assert_eq!(
            session.lock().await.terminal.theme_default_foreground(),
            Some(Color::Rgb(0x40, 0x41, 0x42))
        );
        assert_eq!(
            session.lock().await.terminal.osc_default_colors(),
            (None, None)
        );
        let query_response = {
            let mut session = session.lock().await;
            session.terminal.process(b"\x1b]11;?\x1b\\");
            session.terminal.take_pending_query_responses()
        };
        assert_eq!(
            query_response, b"\x1b]11;rgb:3030/3131/3232\x1b\\",
            "OSC queries after a theme change must report the new Host color"
        );
        session_manager.close_session(session_id).await.unwrap();
    }

    #[tokio::test]
    async fn empty_shared_state_snapshots_are_serializable() {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let config_path =
            std::env::temp_dir().join(format!("relay-state-tests-{unique_suffix}.json"));
        let config_manager = Arc::new(
            ConfigManager::load_from_path(PathBuf::from(config_path))
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(Arc::clone(&config_manager)));
        session_manager
            .create_session(
                TerminalSize::new(24, 80),
                String::new(),
                None,
                HashMap::new(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let relay_manager = Arc::new(RelayManager::new(
            config_manager.clone(),
            session_manager,
            Arc::new(ScreenServerSupervisor::new(config_manager)),
            test_port_forward_manager(),
            Arc::new(ConnectionRegistry::new()),
        ));
        let (client_tx, mut client_rx) = mpsc::unbounded_channel();
        relay_manager.register_client(client_tx).await;
        let (shared_tx, _shared_rx) = mpsc::unbounded_channel();
        let (server_id, _) = relay_manager
            .register_shared_server(
                "Test Shared".to_string(),
                Some(7878),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 12345),
                shared_tx,
            )
            .await;

        relay_manager
            .handle_shared_server_message(
                &server_id,
                Message::SessionList {
                    sessions: Vec::new(),
                },
            )
            .await
            .unwrap();
        relay_manager
            .handle_shared_server_message(
                &server_id,
                Message::TabInfoList {
                    server_id: None,
                    tabs: Vec::new(),
                },
            )
            .await
            .unwrap();
        relay_manager
            .handle_shared_server_message(
                &server_id,
                Message::TerminalProfileList {
                    profiles: Vec::new(),
                },
            )
            .await
            .unwrap();

        let mut serialized_messages = 0;
        while let Ok(message) = client_rx.try_recv() {
            message.to_text().unwrap();
            serialized_messages += 1;
        }
        assert!(serialized_messages > 0);
    }

    #[tokio::test]
    async fn shared_static_port_forward_uses_shared_proxy_then_relay_proxy() {
        let target_listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let static_port = target_listener.local_addr().unwrap().port();
        let target_task = tokio::spawn(async move {
            let (validation_stream, _) = target_listener.accept().await.unwrap();
            drop(validation_stream);
            let (mut target_stream, _) = target_listener.accept().await.unwrap();
            let mut request = [0u8; 4];
            target_stream.read_exact(&mut request).await.unwrap();
            assert_eq!(&request, b"ping");
            target_stream.write_all(b"pong").await.unwrap();
        });

        let shared_proxy_port = available_test_port();
        let relay_proxy_port = loop {
            let candidate = available_test_port();
            if candidate != shared_proxy_port {
                break candidate;
            }
        };
        let shared_port_forward_manager =
            test_port_forward_manager_with_range(shared_proxy_port, shared_proxy_port);
        let relay_port_forward_manager =
            test_port_forward_manager_with_range(relay_proxy_port, relay_proxy_port);

        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let relay_config_manager = Arc::new(
            ConfigManager::load_from_path(PathBuf::from(
                std::env::temp_dir()
                    .join(format!("relay-forward-tests-{unique_suffix}-relay.json")),
            ))
            .await
            .unwrap(),
        );
        let shared_config_manager = Arc::new(
            ConfigManager::load_from_path(PathBuf::from(
                std::env::temp_dir()
                    .join(format!("relay-forward-tests-{unique_suffix}-shared.json")),
            ))
            .await
            .unwrap(),
        );
        let relay_manager = Arc::new(RelayManager::new(
            Arc::clone(&relay_config_manager),
            Arc::new(SessionManager::new(Arc::clone(&relay_config_manager))),
            Arc::new(ScreenServerSupervisor::new(relay_config_manager)),
            Arc::clone(&relay_port_forward_manager),
            Arc::new(ConnectionRegistry::new()),
        ));
        let shared_manager = Arc::new(RelayManager::new(
            Arc::clone(&shared_config_manager),
            Arc::new(SessionManager::new(Arc::clone(&shared_config_manager))),
            Arc::new(ScreenServerSupervisor::new(shared_config_manager)),
            Arc::clone(&shared_port_forward_manager),
            Arc::new(ConnectionRegistry::new()),
        ));

        let (shared_tx, mut shared_rx) = mpsc::unbounded_channel();
        let (upstream_tx, mut upstream_rx) = mpsc::unbounded_channel();
        let (server_id, _) = relay_manager
            .register_shared_server(
                "Forward Test Shared".to_string(),
                Some(7878),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 12345),
                shared_tx,
            )
            .await;
        let session_id = SessionId::new();
        relay_manager
            .handle_shared_server_message(
                &server_id,
                Message::RequestSharedPortForward {
                    session_id,
                    target_port: static_port,
                    auth_token: None,
                },
            )
            .await
            .unwrap();

        let inner_request = shared_rx.recv().await.unwrap();
        assert!(matches!(
            &inner_request,
            Message::RequestReverseProxy { target_port, .. } if *target_port == static_port
        ));
        let mut streamed_session_ids = HashSet::new();
        handle_upstream_command(
            &shared_manager,
            &shared_manager.session_manager,
            &upstream_tx,
            &mut streamed_session_ids,
            inner_request,
        )
        .await
        .unwrap();
        let shared_forwards = shared_port_forward_manager.list_forwards().await;
        assert_eq!(shared_forwards.len(), 1);
        assert_eq!(shared_forwards[0].target_port, static_port);
        assert_eq!(shared_forwards[0].proxy_port, shared_proxy_port);

        let inner_response = upstream_rx.recv().await.unwrap();
        assert!(matches!(
            &inner_response,
            Message::ReverseProxyReady {
                target_port,
                proxy_port,
                error: None,
                ..
            } if *target_port == static_port && *proxy_port == shared_proxy_port
        ));
        relay_manager
            .handle_shared_server_message(&server_id, inner_response)
            .await
            .unwrap();
        match shared_rx.recv().await.unwrap() {
            Message::SharedPortForwardReady {
                session_id: response_session_id,
                target_port,
                relay_port,
                error,
            } => {
                assert_eq!(response_session_id, session_id);
                assert_eq!(target_port, static_port);
                assert_eq!(relay_port, relay_proxy_port);
                assert!(error.is_none());
            }
            other => panic!(
                "expected shared port forward ready, got {}",
                other.msg_name()
            ),
        }
        let relay_forwards = relay_port_forward_manager.list_forwards().await;
        assert_eq!(relay_forwards.len(), 1);
        assert_eq!(relay_forwards[0].target_port, shared_proxy_port);
        assert_eq!(relay_forwards[0].proxy_port, relay_proxy_port);

        let connector = tokio_rustls::TlsConnector::from(
            vterm_common::tls::insecure_client_tls_config().unwrap(),
        );
        let relay_stream = tokio::net::TcpStream::connect((Ipv4Addr::LOCALHOST, relay_proxy_port))
            .await
            .unwrap();
        let server_name = rustls::pki_types::ServerName::try_from("localhost".to_string()).unwrap();
        let mut relay_stream = connector.connect(server_name, relay_stream).await.unwrap();
        relay_stream.write_all(b"ping").await.unwrap();
        let mut response = [0u8; 4];
        relay_stream.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"pong");
        target_task.await.unwrap();

        let failed_session_id = SessionId::new();
        relay_manager
            .handle_shared_server_message(
                &server_id,
                Message::RequestSharedPortForward {
                    session_id: failed_session_id,
                    target_port: 0,
                    auth_token: None,
                },
            )
            .await
            .unwrap();
        let failed_request = shared_rx.recv().await.unwrap();
        handle_upstream_command(
            &shared_manager,
            &shared_manager.session_manager,
            &upstream_tx,
            &mut streamed_session_ids,
            failed_request,
        )
        .await
        .unwrap();
        let failed_response = upstream_rx.recv().await.unwrap();
        relay_manager
            .handle_shared_server_message(&server_id, failed_response)
            .await
            .unwrap();
        match shared_rx.recv().await.unwrap() {
            Message::SharedPortForwardReady {
                session_id: response_session_id,
                relay_port,
                error,
                ..
            } => {
                assert_eq!(response_session_id, failed_session_id);
                assert_eq!(relay_port, 0);
                assert!(error.is_some());
            }
            other => panic!(
                "expected shared port forward error, got {}",
                other.msg_name()
            ),
        }

        assert_eq!(relay_port_forward_manager.list_forwards().await.len(), 1);
        assert_eq!(shared_port_forward_manager.list_forwards().await.len(), 1);
        relay_port_forward_manager.shutdown_all().await;
        shared_port_forward_manager.shutdown_all().await;
    }
}
