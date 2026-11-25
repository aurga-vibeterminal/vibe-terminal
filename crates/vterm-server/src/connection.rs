//! WebSocket connection handling: the wire protocol only.
//!
//! This module owns everything transport-level - the WebSocket upgrade, the
//! Hello/AuthChallenge handshake, optional message encryption, frame
//! encode/decode, heartbeats and idle timeouts, the persistent request
//! connection multiplexer, and per-connection teardown. Decoded
//! [`Message`]s are routed to per-feature handlers in [`crate::features`];
//! this file deliberately contains no feature logic.

use std::collections::{HashMap, HashSet, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Context;
use futures_util::{SinkExt, Stream, StreamExt};
use if_addrs::IfAddr;
use tokio::sync::{mpsc, watch, OwnedSemaphorePermit, Semaphore};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{self, Message as WebSocketMessage};
use tokio_tungstenite::WebSocketStream;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use tokio::io::{AsyncRead, AsyncWrite};

use vterm_common::{
    hash_secret_key, secret_key_challenge_response_from_hash, verify_secret_key_challenge_response,
    ErrorCode, SessionId, MAX_MESSAGE_SIZE, PROTOCOL_VERSION,
};
use vterm_core::{
    crypto::{
        is_encrypted_message_frame, MessageCrypto, MessageCryptoHandshake, TRANSPORT_TAG_BINARY,
        TRANSPORT_TAG_TEXT,
    },
    protocol::{ConnectionKind, Message, TransportBinaryFrameDecoder},
};
use vterm_transport::{
    decode_received_message, decrypt_request_frame, encode_message, encrypt_request_frame,
    FrameKind as RequestFrameKind, InboundEvent as RequestInboundEvent, InboundMultiplexer,
    OutboundCommand as RequestOutboundCommand, OutboundMultiplexer, OutboundRequest,
    ReceivedMessage, RequestId, TransportLimits,
};

use crate::addon::AddonHost;
use crate::config::{stored_server_secret_key_hash, ConfigManager};
use crate::features::screen::ScreenStreamGrant;
use crate::fs_watcher::FsWatcherManager;
use crate::port_forward::PortForwardManager;
use crate::relay::RelayManager;
use crate::session::{ConnectionTx, FolderUploadSink, SessionManager};

const WEBSOCKET_OUTPUT_BATCH_MAX_BYTES: usize = 256 * 1024;
static REQUEST_EXECUTOR_LIMIT: OnceLock<Arc<Semaphore>> = OnceLock::new();
/// Caps the number of connections simultaneously in their pre-auth
/// handshake phase (WS upgrade + challenge/response + Noise exchange).
/// Complements the per-frame timeouts: a slowloris flood may hold each
/// connection up to [`HANDSHAKE_FRAME_TIMEOUT`], but only this many at
/// once — beyond that, peers queue until a slot frees. Post-auth
/// connections hold no slot (released in [`Connection::handle`]) and are
/// bounded by heartbeats and the idle timeout instead.
static HANDSHAKE_CONCURRENCY_LIMIT: OnceLock<Arc<Semaphore>> = OnceLock::new();
const HANDSHAKE_CONCURRENCY: usize = 256;

/// The server-wide pre-auth handshake concurrency limiter.
pub(crate) fn handshake_concurrency_limiter() -> Arc<Semaphore> {
    HANDSHAKE_CONCURRENCY_LIMIT
        .get_or_init(|| Arc::new(Semaphore::new(HANDSHAKE_CONCURRENCY)))
        .clone()
}

/// Slowloris guard (security review §1.1): a peer must finish the
/// WebSocket upgrade this fast after the TCP/TLS connection was accepted.
const WEBSOCKET_UPGRADE_TIMEOUT: Duration = Duration::from_secs(10);
/// Slowloris guard (§1.1): every pre-authentication handshake frame — the
/// Hello, the AuthResponse and each Noise exchange frame — must arrive
/// within this window. Post-authentication traffic is bounded by
/// [`CONNECTION_IDLE_TIMEOUT`] instead.
const HANDSHAKE_FRAME_TIMEOUT: Duration = Duration::from_secs(15);
/// Auth-failure rate limiting (§1.2): this many failures from one IP inside
/// [`AUTH_FAILURE_WINDOW`] locks that IP out for [`AUTH_FAILURE_LOCKOUT`].
/// The lock is not extended by further attempts, so a NAT-shared IP hit by
/// one attacker self-heals after the lockout.
const AUTH_FAILURE_WINDOW: Duration = Duration::from_secs(300);
const AUTH_FAILURE_MAX: usize = 5;
const AUTH_FAILURE_LOCKOUT: Duration = Duration::from_secs(60);
/// Uniform response for every handshake-stage authentication failure (§1.3):
/// identical code and message for all failure stages — probers get no
/// oracle about which check failed; the distinguishing detail goes to the
/// server log only.
const AUTH_FAILED_MESSAGE: &str = "Authentication failed";
/// Cap on tracked auth-failure IPs; beyond it stale entries are swept so a
/// connection flood cannot balloon the limiter's memory.
const AUTH_FAILURE_MAP_SWEEP_THRESHOLD: usize = 4096;

/// Interval at which every primary/terminal connection receives a WebSocket
/// ping from the server. Peers built on tokio-tungstenite answer pings
/// automatically (RFC 6455), so a healthy client always produces inbound
/// frames.
const CONNECTION_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// Close a connection that produced no inbound frames — including heartbeat
/// pongs — for this long. This reaps dead or half-open peers (crashed client,
/// dropped network, sleeping laptop) whose FIN never arrived; without it a
/// stale terminal connection keeps its registered viewers alive forever and
/// pins session PTY sizes.
const CONNECTION_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

fn coalesce_queued_output(
    message: &mut Message,
    rx: &mut mpsc::UnboundedReceiver<Message>,
) -> Option<Message> {
    let Message::Output {
        session_id,
        start_offset,
        data,
    } = message
    else {
        return None;
    };

    while data.len() < WEBSOCKET_OUTPUT_BATCH_MAX_BYTES {
        let next = match rx.try_recv() {
            Ok(next) => next,
            Err(_) => break,
        };
        match next {
            Message::Output {
                session_id: next_session_id,
                start_offset: next_start_offset,
                data: next_data,
            } if next_session_id == *session_id
                && next_start_offset == start_offset.saturating_add(data.len() as u64)
                && data.len() + next_data.len() <= WEBSOCKET_OUTPUT_BATCH_MAX_BYTES =>
            {
                data.extend_from_slice(&next_data);
            }
            next => return Some(next),
        }
    }
    None
}

#[derive(Debug, Default)]
pub struct ConnectionRegistry {
    primaries: Mutex<HashMap<String, (IpAddr, watch::Sender<bool>)>>,
    /// Per-IP handshake auth-failure history (review §1.2). Keyed by IP, not
    /// IP:port, so reconnect floods cannot bypass the window.
    auth_failures: Mutex<HashMap<IpAddr, AuthFailureRecord>>,
}

/// Sliding-window failure history plus the lockout it triggers.
#[derive(Debug, Default)]
struct AuthFailureRecord {
    failures: VecDeque<Instant>,
    /// Last activity (failure or probe) — eviction ordering when the map
    /// hits its sweep threshold.
    last_seen: Option<Instant>,
    locked_until: Option<Instant>,
}

impl AuthFailureRecord {
    /// Drop failures that fell out of the window, measured from `now`
    /// (injected so tests can time-travel deterministically).
    fn prune(&mut self, now: Instant) {
        let Some(cutoff) = now.checked_sub(AUTH_FAILURE_WINDOW) else {
            // Clock too early for a full window to have elapsed; nothing
            // can be stale yet.
            return;
        };
        while self
            .failures
            .front()
            .is_some_and(|failure| *failure < cutoff)
        {
            self.failures.pop_front();
        }
    }

    /// A record with no in-window failures and no active lockout can be
    /// dropped from the map entirely.
    fn is_stale(&self, now: Instant) -> bool {
        self.failures.is_empty() && !self.locked_until.is_some_and(|locked| locked > now)
    }
}

impl ConnectionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn register_primary(&self, token: String, peer_ip: IpAddr) {
        let (tx, _rx) = watch::channel(true);
        if let Ok(mut primaries) = self.primaries.lock() {
            primaries.insert(token, (peer_ip, tx));
        }
    }

    fn unregister_primary(&self, token: &str) {
        let sender = self
            .primaries
            .lock()
            .ok()
            .and_then(|mut primaries| primaries.remove(token));
        if let Some((_, sender)) = sender {
            let _ = sender.send(false);
        }
    }

    fn validate_child(&self, token: &str) -> Option<watch::Receiver<bool>> {
        self.primaries.lock().ok().and_then(|primaries| {
            primaries.get(token).and_then(|(_, sender)| {
                if *sender.borrow() {
                    Some(sender.subscribe())
                } else {
                    None
                }
            })
        })
    }

    /// Whether any connected primary client comes from a LAN address —
    /// proof that the local network path toward this server currently works.
    /// Used by the macOS Local Network block detection in
    /// [`crate::local_network`].
    pub(crate) fn has_active_lan_peer(&self) -> bool {
        self.primaries
            .lock()
            .map(|primaries| {
                primaries
                    .values()
                    .any(|(ip, _)| crate::local_network::is_lan_ip(ip))
            })
            .unwrap_or(false)
    }

    /// Count one handshake authentication failure from `ip`; crossing
    /// [`AUTH_FAILURE_MAX`] within the window locks the IP for
    /// [`AUTH_FAILURE_LOCKOUT`] (review §1.2).
    pub fn record_auth_failure(&self, ip: IpAddr) {
        self.record_auth_failure_at(ip, Instant::now());
    }

    fn record_auth_failure_at(&self, ip: IpAddr, now: Instant) {
        let Ok(mut failures) = self.auth_failures.lock() else {
            return;
        };
        if failures.len() >= AUTH_FAILURE_MAP_SWEEP_THRESHOLD {
            failures.retain(|_, record| {
                record.prune(now);
                !record.is_stale(now)
            });
            while failures.len() >= AUTH_FAILURE_MAP_SWEEP_THRESHOLD {
                let Some(oldest) = failures
                    .iter()
                    .min_by_key(|(_, record)| record.last_seen)
                    .map(|(ip, _)| *ip)
                else {
                    break;
                };
                failures.remove(&oldest);
            }
        }
        let record = failures.entry(ip).or_default();
        record.prune(now);
        record.last_seen = Some(now);
        record.failures.push_back(now);
        if record.failures.len() >= AUTH_FAILURE_MAX {
            record.locked_until = Some(now + AUTH_FAILURE_LOCKOUT);
        }
    }

    pub fn record_auth_success(&self, ip: IpAddr) {
        if let Ok(mut failures) = self.auth_failures.lock() {
            failures.remove(&ip);
        }
    }

    /// Whether `ip` is currently locked out by [`Self::record_auth_failure`].
    pub fn is_auth_limited(&self, ip: IpAddr) -> bool {
        self.is_auth_limited_at(ip, Instant::now())
    }

    fn is_auth_limited_at(&self, ip: IpAddr, now: Instant) -> bool {
        let Ok(failures) = self.auth_failures.lock() else {
            return false;
        };
        failures
            .get(&ip)
            .is_some_and(|record| record.locked_until.is_some_and(|locked| locked > now))
    }
}

pub struct Connection {
    pub(crate) addr: SocketAddr,
    pub(crate) config_manager: Arc<ConfigManager>,
    pub(crate) session_manager: Arc<SessionManager>,
    pub(crate) addon_host: Arc<AddonHost>,
    pub(crate) relay_manager: Arc<RelayManager>,
    pub(crate) port_forward_manager: Arc<PortForwardManager>,
    pub(crate) fs_watcher_manager: Arc<FsWatcherManager>,
    pub(crate) static_http_manager: Arc<crate::static_http::StaticHttpManager>,
    pub(crate) rx: Option<mpsc::UnboundedReceiver<Message>>,
    pub(crate) tx: ConnectionTx,
    pub(crate) streamed_session_ids: HashSet<SessionId>,
    pub(crate) fs_watched_session_ids: HashSet<SessionId>,
    pub(crate) shared_server_id: Option<String>,
    pub(crate) shared_server_connection_kind: Option<ConnectionKind>,
    pub(crate) auth_token: Option<String>,
    pub(crate) connection_kind: ConnectionKind,
    pub(crate) parent_alive: Option<watch::Receiver<bool>>,
    pub(crate) authorized_session_id: Option<SessionId>,
    pub(crate) connection_registry: Arc<ConnectionRegistry>,
    pub(crate) supports_binary_output: bool,
    pub(crate) voice_supervisor: Arc<crate::voice::AsrSupervisor>,
    pub(crate) screen_supervisor: Arc<crate::screen::ScreenServerSupervisor>,
    /// This connection's active screen streams: source spec → grant
    /// (for StopScreenStream revocation and cleanup accounting).
    pub(crate) screen_streams: HashMap<String, ScreenStreamGrant>,
    /// Lazily spawned source/Android-operation/stats forwarder for this
    /// client; aborted in [`Connection::cleanup_connection`].
    pub(crate) screen_event_forwarder: Option<tokio::task::JoinHandle<()>>,
    /// `true` when this connection arrived through a relay request tunnel
    /// (relayed client). Screen-stream endpoints for such connections must
    /// go through the relay's UDP pairings — the client cannot reach this
    /// host's screen-server directly (spec §7).
    pub(crate) via_relay: bool,
    handshake_permit: Option<OwnedSemaphorePermit>,
}
enum HandshakeOutcome {
    Local(Option<Arc<Mutex<MessageCrypto>>>),
    Proxied,
}

impl Connection {
    pub fn new(
        addr: SocketAddr,
        config_manager: Arc<ConfigManager>,
        session_manager: Arc<SessionManager>,
        addon_host: Arc<AddonHost>,
        relay_manager: Arc<RelayManager>,
        port_forward_manager: Arc<PortForwardManager>,
        fs_watcher_manager: Arc<FsWatcherManager>,
        static_http_manager: Arc<crate::static_http::StaticHttpManager>,
        connection_registry: Arc<ConnectionRegistry>,
        voice_supervisor: Arc<crate::voice::AsrSupervisor>,
        screen_supervisor: Arc<crate::screen::ScreenServerSupervisor>,
        handshake_permit: OwnedSemaphorePermit,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            addr,
            config_manager,
            session_manager,
            relay_manager,
            port_forward_manager,
            fs_watcher_manager,
            static_http_manager,
            addon_host,
            rx: Some(rx),
            tx,
            streamed_session_ids: HashSet::new(),
            fs_watched_session_ids: HashSet::new(),
            shared_server_id: None,
            shared_server_connection_kind: None,
            auth_token: None,
            connection_kind: ConnectionKind::Primary,
            parent_alive: None,
            authorized_session_id: None,
            connection_registry,
            supports_binary_output: false,
            voice_supervisor,
            screen_supervisor,
            screen_streams: HashMap::new(),
            screen_event_forwarder: None,
            via_relay: false,
            handshake_permit: Some(handshake_permit),
        }
    }

    pub async fn handle<S>(mut self, stream: S) -> anyhow::Result<()>
    where
        S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    {
        let addr = self.addr;
        // info!("Client connected: {}", addr);

        let mut websocket = tokio::time::timeout(WEBSOCKET_UPGRADE_TIMEOUT, accept_async(stream))
            .await
            .map_err(|_| anyhow::anyhow!("WebSocket upgrade timed out for {}", addr))?
            .with_context(|| format!("WebSocket upgrade failed for {}", addr))?;
        let message_crypto = match self
            .perform_handshake(&mut websocket)
            .await
            .with_context(|| format!("Handshake failed for {}", addr))?
        {
            HandshakeOutcome::Local(message_crypto) => message_crypto,
            HandshakeOutcome::Proxied => return Ok(()),
        };
        // The handshake concurrency slot only needs to be held while
        // pre-auth work is in flight (WS upgrade, challenge/response, the
        // Noise exchange). Release it here: failure paths never reach this
        // point — the dropped `Connection` releases its permit — and the
        // Proxied arm above returned by dropping the whole connection.
        // Post-auth connections are bounded by heartbeats and the idle
        // timeout instead.
        drop(self.handshake_permit.take());
        // info!(
        //     "Established {:?} connection from {} (encrypted={}, binary_output={})",
        //     self.connection_kind,
        //     addr,
        //     message_crypto.is_some(),
        //     self.supports_binary_output
        // );

        if self.connection_kind == ConnectionKind::Request {
            let result = self
                .handle_persistent_request_connection(websocket, message_crypto)
                .await;
            self.cleanup_connection().await;
            // info!("Client disconnected: {}", addr);
            return result.with_context(|| format!("Request connection {} failed", addr));
        }

        let (mut write_half, mut read_half) = websocket.split();
        let mut rx = self.rx.take().expect("rx should exist");
        let supports_binary_output = self.supports_binary_output;
        let crypto_writer = message_crypto.clone();
        let connection_kind = self.connection_kind;

        let writer_task = tokio::spawn(async move {
            let mut pending = None;
            let mut heartbeat = tokio::time::interval(CONNECTION_HEARTBEAT_INTERVAL);
            heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                let mut message = if let Some(pending_message) = pending.take() {
                    pending_message
                } else {
                    tokio::select! {
                        received = rx.recv() => match received {
                            Some(message) => message,
                            None => break,
                        },
                        _ = heartbeat.tick() => {
                            // Keep-alive probe: peers on tokio-tungstenite
                            // answer pings automatically. A dead or half-open
                            // peer never answers, so the reader's idle
                            // timeout reaps the connection and its viewers.
                            if write_half
                                .send(WebSocketMessage::Ping(Vec::new()))
                                .await
                                .is_err()
                            {
                                error!("Failed to send heartbeat ping to {}", addr);
                                return;
                            }
                            continue;
                        }
                    }
                };
                pending = coalesce_queued_output(&mut message, &mut rx);
                let output_stats = match &message {
                    Message::Output {
                        session_id, data, ..
                    } => Some((*session_id, data.len())),
                    _ => None,
                };
                let write_started = std::time::Instant::now();
                //log_outgoing_connection_message(connection_kind, addr, &message);
                let websocket_messages = match encode_websocket_messages(
                    message,
                    crypto_writer.as_ref(),
                    supports_binary_output,
                ) {
                    Ok(messages) => messages,
                    Err(error) => {
                        error!("Failed to encode message: {}", error);
                        continue;
                    }
                };

                for websocket_message in websocket_messages {
                    if let Err(e) = write_half.send(websocket_message).await {
                        error!("Failed to write to stream: {}", e);
                        return;
                    }
                }
                if let Some((session_id, bytes)) = output_stats {
                    // Echo-path filter: only output following a recent
                    // keystroke is sampled; background streams stay quiet.
                    if vterm_common::latency::echo_ms_since_input(session_id).is_some() {
                        let encode_write_ms = write_started.elapsed().as_secs_f64() * 1000.0;
                        if vterm_common::latency::sample(
                            "server:output-socket-write",
                            session_id,
                            encode_write_ms,
                            10.0,
                            4.0,
                        ) {
                            vterm_common::latency_log!(
                                stage = "server:output-socket-write",
                                session = %session_id,
                                bytes,
                                encode_write_ms,
                                "[latency] echo output encode+websocket write was slow"
                            );
                        }
                    }
                }
            }

            debug!("Writer task ended for {}", addr);
        });

        if self.connection_kind == ConnectionKind::Primary {
            self.relay_manager.register_client(self.tx.clone()).await;
            let config = self.config_manager.snapshot().await;
            self.send_message(Message::ServerInfo {
                server_id: Some(config.server.server_id.clone()),
                info: crate::config::public_server_info(&config),
            })
            .await?;
            self.send_initial_snapshots().await?;
            info!(
                "Primary connection {} registered and initial snapshots sent",
                addr
            );
        }

        let result = if let Some(parent_alive) = self.parent_alive.take() {
            trace!("Terminal connection {} registered as primary child", addr);
            self.handle_terminal_incoming(&mut read_half, message_crypto.as_ref(), parent_alive)
                .await
        } else {
            self.handle_incoming(&mut read_half, message_crypto.as_ref())
                .await
        };

        self.cleanup_connection().await;
        drop(self.tx);
        let _ = writer_task.await;

        if !matches!(
            connection_kind,
            ConnectionKind::Request | ConnectionKind::RelayRequest
        ) {
            // Normal lifecycle event (happens on every tab close / client
            // exit) — routine, below the default level.
            trace!("Client disconnected: {} ({:?})", addr, connection_kind);
        }
        result.with_context(|| {
            format!(
                "{:?} connection {} failed while processing incoming messages",
                connection_kind, addr
            )
        })
    }

    async fn send_initial_snapshots(&self) -> anyhow::Result<()> {
        let profiles = self.relay_manager.list_terminal_profiles().await;
        self.send_message(Message::TerminalProfileList { profiles })
            .await?;
        let servers = self.relay_manager.relay_server_list().await;
        self.send_message(Message::RelayServerList { servers })
            .await?;
        let sessions = self.relay_manager.list_sessions().await?;
        self.send_message(Message::SessionList { sessions }).await?;
        let groups = self.relay_manager.list_tab_groups().await?;
        self.send_message(Message::TabGroupList { groups }).await?;
        let tabs = self.relay_manager.list_all_tab_infos().await;
        self.send_message(Message::TabInfoList {
            server_id: None,
            tabs,
        })
        .await?;
        for (server_id, directories) in self.relay_manager.recent_project_directory_lists().await {
            self.send_message(Message::RecentProjectDirectoryList {
                server_id: Some(server_id),
                directories,
            })
            .await?;
        }
        for (server_id, commands) in self.relay_manager.terminal_command_history_lists().await {
            self.send_message(Message::TerminalCommandHistoryList {
                server_id: Some(server_id),
                commands,
            })
            .await?;
        }
        self.relay_manager.broadcast_known_app_lists().await;
        Ok(())
    }

    async fn cleanup_connection(&mut self) {
        for session_id in self.fs_watched_session_ids.drain() {
            // Only remove watches this connection still owns; a newer
            // terminal connection for the same session may already have
            // taken over.
            self.fs_watcher_manager
                .remove_session_if_owner(session_id, &self.tx);
        }
        for session_id in self.streamed_session_ids.drain() {
            let _ = self
                .session_manager
                .remove_viewer(session_id, &self.tx)
                .await;
        }
        if let Some(server_id) = self.shared_server_id.take() {
            if self.shared_server_connection_kind.take() == Some(ConnectionKind::RelayPrimary) {
                self.relay_manager
                    .unregister_shared_server(&server_id)
                    .await;
            }
        } else if self.connection_kind == ConnectionKind::Primary {
            self.relay_manager.unregister_client(&self.tx).await;
        }
        if self.connection_kind == ConnectionKind::Primary {
            if let Some(auth_token) = self.auth_token.take() {
                crate::http_auth::settings_access().revoke_owner(&auth_token);
                self.connection_registry.unregister_primary(&auth_token);
            }
        }
        // Revoke this client's screen-stream token grants (spec §6 teardown
        // symmetry): the UDP sessions themselves time out on their own.
        //
        // Relay pairings are tracked by label on the relay manager, not in
        // this connection's map: request messages (StartScreenStream) are
        // handled by short-lived child connections whose grant maps do not
        // survive the request, so teardown must be label-based like the
        // token revocation below.
        self.relay_manager
            .release_upstream_udp_pairings_for_label(&self.addr.to_string())
            .await;
        if !self.screen_streams.is_empty() {
            let label = self.addr.to_string();
            if let Err(e) = self.screen_supervisor.revoke_client(&label).await {
                warn!("screen-stream grant revocation for {label} failed: {e:#}");
            }
            self.screen_streams.clear();
        }
        if let Some(forwarder) = self.screen_event_forwarder.take() {
            forwarder.abort();
        }
    }
    /// Perform the application handshake after the WebSocket upgrade succeeds.
    async fn perform_handshake<S>(
        &mut self,
        websocket: &mut WebSocketStream<S>,
    ) -> anyhow::Result<HandshakeOutcome>
    where
        S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    {
        // Rate-limit lockout (review §1.2): peers that exhausted the
        // auth-failure budget are rejected before any challenge work is
        // spent — and before their HMAC guesses reach the verifier.
        if self.connection_registry.is_auth_limited(self.addr.ip()) {
            self.send_handshake_error(
                websocket,
                ErrorCode::RateLimited,
                "Too many authentication failures; retry later",
            )
            .await?;
            anyhow::bail!("Auth failure lockout active for {}", self.addr);
        }

        // Slowloris guard (review §1.1): the Hello must arrive promptly.
        let first_frame = tokio::time::timeout(HANDSHAKE_FRAME_TIMEOUT, websocket.next())
            .await
            .map_err(|_| anyhow::anyhow!("Handshake first frame timed out for {}", self.addr))?;
        let Some(frame) = first_frame else {
            anyhow::bail!("Client disconnected before handshake completed");
        };

        let message = match frame {
            Ok(WebSocketMessage::Text(text)) => {
                Message::from_text(text.as_ref()).with_context(|| {
                    format!("Failed to decode plaintext handshake from {}", self.addr)
                })?
            }
            Ok(WebSocketMessage::Binary(data)) => {
                Message::deserialize(&data).with_context(|| {
                    format!(
                        "Failed to decode binary handshake from {} ({} bytes)",
                        self.addr,
                        data.len()
                    )
                })?
            }
            Ok(WebSocketMessage::Close(_)) => {
                anyhow::bail!("Client closed the connection during handshake");
            }
            Ok(_) => {
                anyhow::bail!("Unexpected WebSocket frame during handshake");
            }
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "Failed to read handshake frame from {}: {} ({:?})",
                    self.addr,
                    error,
                    error
                ));
            }
        };

        match message {
            Message::Hello {
                protocol_version,
                client_version,
                secret_key_hash: _,
                supports_binary_output,
                supports_message_encryption,
                connection_kind,
                auth_token,
                target_session_id,
                relay_terminal_capability,
                target_server_id,
                relay_request_capability,
            } => {
                let connection_kind = connection_kind.unwrap_or(ConnectionKind::Primary);
                // info!(
                //     "Handshake from {} using protocol {} and client version {} as {:?}",
                //     self.addr, protocol_version, client_version, connection_kind
                // );

                if protocol_version != PROTOCOL_VERSION {
                    warn!(
                        "Client {} uses protocol version {} but server requires {}; continuing anyway",
                        self.addr, protocol_version, PROTOCOL_VERSION
                    );
                }

                // Fail-closed encryption policy (review §1.7): non-local
                // peers must negotiate message encryption. A plaintext
                // Hello can only come from a legacy client or a middleman
                // stripping the capability bit (the §4-C downgrade); either
                // way the handshake now fails loudly instead of silently
                // producing a plaintext session. Checked before any
                // capability consumption or challenge work. Loopback /
                // own-interface peers are exempt when local trust is on
                // (a MITM there means the machine is already compromised).
                let server_config = self.config_manager.snapshot().await.server;
                let local_peer =
                    server_config.trust_local_peers && is_local_machine(self.addr.ip());
                if !supports_message_encryption && !local_peer {
                    self.send_handshake_error(
                        websocket,
                        ErrorCode::InvalidMessage,
                        "Message encryption is required",
                    )
                    .await?;
                    anyhow::bail!("Rejected plaintext handshake from {}", self.addr);
                }

                let relayed_terminal_session = if connection_kind == ConnectionKind::Terminal {
                    if let (Some(session_id), Some(capability)) =
                        (target_session_id, relay_terminal_capability.as_deref())
                    {
                        if !self
                            .relay_manager
                            .consume_terminal_capability(session_id, capability)
                            .await
                        {
                            return Err(self
                                .reject_auth(
                                    websocket,
                                    &format!(
                                        "invalid relayed terminal capability for session {session_id}"
                                    ),
                                )
                                .await);
                        }
                        self.authorized_session_id = Some(session_id);
                        Some(session_id)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let relayed_request = if connection_kind == ConnectionKind::RelayRequest {
                    if let (Some(server_id), Some(capability)) = (
                        target_server_id.as_deref(),
                        relay_request_capability.as_deref(),
                    ) {
                        if !self
                            .relay_manager
                            .consume_request_capability(server_id, capability)
                            .await
                        {
                            return Err(self
                                .reject_auth(
                                    websocket,
                                    &format!(
                                        "invalid relayed request capability for server {server_id}"
                                    ),
                                )
                                .await);
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                // A routed terminal is authenticated by its live primary token
                // before the relay opens the shared-server tunnel. From the
                // shared server's challenge onward, including the complete
                // Noise handshake, frames are forwarded without inspection.
                if connection_kind == ConnectionKind::Terminal
                    && relayed_terminal_session.is_none()
                    && relay_terminal_capability.is_none()
                {
                    if let Some(session_id) = target_session_id {
                        if self.relay_manager.is_remote_session(session_id).await {
                            let Some(auth_token) = auth_token.as_deref() else {
                                return Err(self
                                    .reject_auth(
                                        websocket,
                                        "remote terminal connection omitted its primary token",
                                    )
                                    .await);
                            };
                            let Some(parent_alive) =
                                self.connection_registry.validate_child(auth_token)
                            else {
                                return Err(self
                                    .reject_auth(
                                        websocket,
                                        "remote terminal connection used an invalid primary token",
                                    )
                                    .await);
                            };
                            if self
                                .relay_manager
                                .proxy_remote_terminal(
                                    websocket,
                                    session_id,
                                    protocol_version,
                                    client_version.clone(),
                                    supports_binary_output,
                                    supports_message_encryption,
                                    parent_alive,
                                )
                                .await?
                            {
                                return Ok(HandshakeOutcome::Proxied);
                            }
                        }
                    }
                }

                // A client selects the shared server in the plaintext route
                // Hello. The relay validates the live Primary token, opens a
                // capability-bound upstream connection, then stops inspecting
                // frames before the shared server challenge and Noise setup.
                if connection_kind == ConnectionKind::RelayRequest && !relayed_request {
                    let Some(server_id) = target_server_id.as_deref() else {
                        self.send_handshake_error(
                            websocket,
                            ErrorCode::InvalidMessage,
                            "Missing shared server route",
                        )
                        .await?;
                        anyhow::bail!("RelayRequest omitted target_server_id");
                    };
                    let Some(auth_token) = auth_token.as_deref() else {
                        return Err(self
                            .reject_auth(websocket, "RelayRequest omitted primary token")
                            .await);
                    };
                    let Some(parent_alive) = self.connection_registry.validate_child(auth_token)
                    else {
                        return Err(self
                            .reject_auth(websocket, "RelayRequest used an invalid primary token")
                            .await);
                    };
                    if self
                        .relay_manager
                        .proxy_remote_request(
                            websocket,
                            server_id,
                            protocol_version,
                            client_version.clone(),
                            supports_binary_output,
                            supports_message_encryption,
                            parent_alive,
                        )
                        .await?
                    {
                        return Ok(HandshakeOutcome::Proxied);
                    }
                    self.send_handshake_error(
                        websocket,
                        ErrorCode::InvalidMessage,
                        "Shared server is unavailable",
                    )
                    .await?;
                    anyhow::bail!("Unknown or unavailable shared server {}", server_id);
                }

                let secret_key = server_config.secret_key.clone();
                let challenge_key_hash =
                    match if relayed_terminal_session.is_some() || relayed_request {
                        Ok(None)
                    } else {
                        challenge_key_hash_for_peer(local_peer, &secret_key)
                    } {
                        Ok(challenge_key_hash) => challenge_key_hash,
                        Err(error) => {
                            // Server-side fault (secret key unconfigured or
                            // undecryptable): answer generically and log
                            // loudly, but do NOT count toward the per-IP
                            // failure budget — a broken config would
                            // otherwise lock out every retrying client.
                            error!("Refusing handshake from {}: {error:#}", self.addr);
                            self.send_handshake_error(
                                websocket,
                                ErrorCode::AuthenticationFailed,
                                AUTH_FAILED_MESSAGE,
                            )
                            .await?;
                            return Err(error);
                        }
                    };
                let challenge_nonce = challenge_key_hash
                    .as_ref()
                    .map(|_| generate_auth_challenge_nonce());

                let challenge = Message::AuthChallenge {
                    nonce: challenge_nonce.clone(),
                    message_encryption: supports_message_encryption,
                };
                websocket
                    .send(WebSocketMessage::Text(challenge.to_text()?.into()))
                    .await
                    .map_err(|error| {
                        anyhow::anyhow!("Failed to send authentication challenge: {}", error)
                    })?;

                if let (Some(key_hash), Some(nonce)) =
                    (challenge_key_hash.as_deref(), challenge_nonce.as_deref())
                {
                    let response = read_plain_handshake_message(websocket).await?;
                    let proof = match response {
                        Message::AuthResponse { proof } => proof,
                        other => {
                            return Err(self
                                .reject_auth(
                                    websocket,
                                    &format!(
                                        "expected authentication response, received {:?}",
                                        other.msg_name()
                                    ),
                                )
                                .await);
                        }
                    };
                    let expected = secret_key_challenge_response_from_hash(key_hash, nonce);
                    if !verify_secret_key_challenge_response(&expected, proof.as_deref()) {
                        // The brute-force case (review §1.2): each wrong
                        // password guess counts toward the lockout.
                        return Err(self
                            .reject_auth(websocket, "secret key proof mismatch")
                            .await);
                    }
                }

                let acknowledged_auth_token = if relayed_terminal_session.is_some()
                    || relayed_request
                {
                    None
                } else {
                    match connection_kind {
                        ConnectionKind::Primary => {
                            let auth_token = generate_auth_token();
                            self.connection_registry
                                .register_primary(auth_token.clone(), self.addr.ip());
                            self.auth_token = Some(auth_token.clone());
                            info!(
                                "Primary connection from {} issued auth token {}",
                                self.addr,
                                token_log_id(&auth_token)
                            );
                            Some(auth_token)
                        }
                        ConnectionKind::Terminal
                        | ConnectionKind::Request
                        | ConnectionKind::Debug
                        | ConnectionKind::RelayRequest => {
                            let Some(auth_token) = auth_token else {
                                return Err(self
                                        .reject_auth(
                                            websocket,
                                            &format!(
                                                "{connection_kind:?} connection did not provide a primary auth token"
                                            ),
                                        )
                                        .await);
                            };
                            let Some(parent_alive) =
                                self.connection_registry.validate_child(&auth_token)
                            else {
                                return Err(self
                                        .reject_auth(
                                            websocket,
                                            &format!(
                                                "{connection_kind:?} connection provided an invalid primary auth token"
                                            ),
                                        )
                                        .await);
                            };
                            self.auth_token = Some(auth_token.clone());
                            self.parent_alive = Some(parent_alive);
                            // info!(
                            //     "{:?} connection from {} authenticated with primary token {}",
                            //     connection_kind,
                            //     self.addr,
                            //     token_log_id(&auth_token)
                            // );
                            Some(auth_token)
                        }
                        ConnectionKind::RelayPrimary => None,
                    }
                };
                self.connection_registry.record_auth_success(self.addr.ip());
                self.connection_kind = if relayed_request {
                    ConnectionKind::Request
                } else {
                    connection_kind
                };
                // Relayed clients cannot reach this host's screen-server UDP
                // port; their streams must be allocated through the relay.
                self.via_relay = relayed_request;
                self.supports_binary_output = supports_binary_output;
                let message_crypto = if supports_message_encryption {
                    Some(Arc::new(Mutex::new(
                        perform_message_encryption_handshake_responder(websocket).await?,
                    )))
                } else {
                    None
                };

                let ack = Message::HelloAck {
                    protocol_version: PROTOCOL_VERSION,
                    server_version: env!("CARGO_PKG_VERSION").to_string(),
                    auth_token: acknowledged_auth_token,
                    server_id: Some(server_config.server_id),
                    message_encryption: supports_message_encryption,
                };

                let response = encode_websocket_messages(ack, message_crypto.as_ref(), false)?
                    .into_iter()
                    .next()
                    .ok_or_else(|| {
                        anyhow::anyhow!("Handshake response encoding produced no frames")
                    })?;
                websocket.send(response).await.map_err(|error| {
                    anyhow::anyhow!("Failed to send handshake response: {}", error)
                })?;

                Ok(HandshakeOutcome::Local(message_crypto))
            }
            other => anyhow::bail!("Expected hello message, received {:?}", other),
        }
    }

    /// Handle incoming application messages from the WebSocket stream.
    async fn handle_incoming<T>(
        &mut self,
        read_half: &mut T,
        crypto: Option<&Arc<Mutex<MessageCrypto>>>,
    ) -> anyhow::Result<()>
    where
        T: Stream<Item = Result<WebSocketMessage, tungstenite::Error>> + Unpin,
    {
        let mut transport_binary_decoder = TransportBinaryFrameDecoder::default();
        let mut last_frame_at = tokio::time::Instant::now();
        loop {
            let frame_opt = tokio::select! {
                // Close connections that go silent: a dead or half-open peer
                // stops answering heartbeat pings, which would otherwise keep
                // its registered viewers alive forever.
                _ = tokio::time::sleep_until(last_frame_at + CONNECTION_IDLE_TIMEOUT) => {
                    warn!(
                        "Connection {} idle for {:?} without heartbeat; closing",
                        self.addr, CONNECTION_IDLE_TIMEOUT
                    );
                    break;
                }
                frame = read_half.next() => {
                    last_frame_at = tokio::time::Instant::now();
                    frame
                }
            };
            let Some(frame) = frame_opt else {
                debug!("Connection closed by peer");
                break;
            };

            let message = match frame {
                Ok(WebSocketMessage::Text(text)) => {
                    if crypto.is_some() {
                        error!(
                            "Received unencrypted text frame from {} after message encryption was negotiated",
                            self.addr
                        );
                        self.send_error(
                            ErrorCode::InvalidMessage,
                            "Unencrypted message after encryption negotiation",
                        )
                        .await;
                        continue;
                    }

                    if text.len() > MAX_MESSAGE_SIZE {
                        error!("Message too large: {} bytes", text.len());
                        break;
                    }
                    match Message::from_text(text.as_ref()) {
                        Ok(message) => message,
                        Err(e) => {
                            error!(
                                "Failed to deserialize text message from {} ({} bytes): {:#}",
                                self.addr,
                                text.len(),
                                e
                            );
                            self.send_error(ErrorCode::InvalidMessage, "Invalid message format")
                                .await;
                            continue;
                        }
                    }
                }
                Ok(WebSocketMessage::Binary(data)) => {
                    if data.len() > MAX_MESSAGE_SIZE {
                        error!("Message too large: {} bytes", data.len());
                        break;
                    }
                    let decoded = match decode_websocket_binary_frame(
                        &data,
                        crypto,
                        &mut transport_binary_decoder,
                    ) {
                        Ok(Some(message)) => Ok(Some(message)),
                        Ok(None) => Ok(None),
                        Err(error) => Err(error),
                    };
                    match decoded {
                        Ok(Some(message)) => message,
                        Ok(None) => continue,
                        Err(e) => {
                            error!(
                                "Failed to deserialize binary message from {} ({} bytes): {:#}",
                                self.addr,
                                data.len(),
                                e
                            );
                            self.send_error(ErrorCode::InvalidMessage, "Invalid message format")
                                .await;
                            continue;
                        }
                    }
                }
                Ok(WebSocketMessage::Close(_)) => {
                    info!("Client {} closed the WebSocket connection", self.addr);
                    break;
                }
                Ok(WebSocketMessage::Ping(_)) | Ok(WebSocketMessage::Pong(_)) => {
                    continue;
                }
                Ok(_) => {
                    continue;
                }
                Err(e) => {
                    if is_disconnect_error(&e) {
                        //                         info!("Client {} disconnected while reading message", self.addr);
                    } else {
                        error!("Failed to read message from {}: {} ({:?})", self.addr, e, e);
                    }
                    break;
                }
            };

            if let Err(e) = self.handle_message(message).await {
                if is_send_failure(&e) {
                    info!(
                        "Stopping connection {} after writer channel closed",
                        self.addr
                    );
                    break;
                }
                error!("Error handling message: {}", e);
            }
        }

        Ok(())
    }

    async fn handle_terminal_incoming<T>(
        &mut self,
        read_half: &mut T,
        crypto: Option<&Arc<Mutex<MessageCrypto>>>,
        mut parent_alive: watch::Receiver<bool>,
    ) -> anyhow::Result<()>
    where
        T: Stream<Item = Result<WebSocketMessage, tungstenite::Error>> + Unpin,
    {
        let mut transport_binary_decoder = TransportBinaryFrameDecoder::default();
        let mut last_frame_at = tokio::time::Instant::now();
        loop {
            let frame_opt = tokio::select! {
                changed = parent_alive.changed() => {
                    if changed.is_err() || !*parent_alive.borrow() {
                        info!("Closing terminal connection {} because its primary connection closed", self.addr);
                        self.send_error(
                            ErrorCode::AuthenticationFailed,
                            "Primary connection closed",
                        )
                        .await;
                        break;
                    }
                    continue;
                }
                // Close connections that go silent: a dead or half-open peer
                // stops answering heartbeat pings, which would otherwise keep
                // its registered viewers alive forever.
                _ = tokio::time::sleep_until(last_frame_at + CONNECTION_IDLE_TIMEOUT) => {
                    warn!(
                        "Terminal connection {} idle for {:?} without heartbeat; closing",
                        self.addr, CONNECTION_IDLE_TIMEOUT
                    );
                    break;
                }
                frame = read_half.next() => {
                    last_frame_at = tokio::time::Instant::now();
                    frame
                }
            };
            let Some(frame) = frame_opt else {
                debug!("Terminal connection closed by peer");
                break;
            };

            let Some(message) = self
                .decode_incoming_frame(frame, crypto, &mut transport_binary_decoder)
                .await?
            else {
                continue;
            };

            if let Err(e) = self.handle_message(message).await {
                if is_send_failure(&e) {
                    info!(
                        "Stopping terminal connection {} after writer channel closed",
                        self.addr
                    );
                    break;
                }
                error!(
                    "Error handling terminal message from {}: {:#}",
                    self.addr, e
                );
            }
        }

        Ok(())
    }

    async fn decode_incoming_frame(
        &self,
        frame: Result<WebSocketMessage, tungstenite::Error>,
        crypto: Option<&Arc<Mutex<MessageCrypto>>>,
        transport_binary_decoder: &mut TransportBinaryFrameDecoder,
    ) -> anyhow::Result<Option<Message>> {
        match frame {
            Ok(WebSocketMessage::Text(text)) => {
                if crypto.is_some() {
                    error!(
                        "Received unencrypted text frame from {} after message encryption was negotiated",
                        self.addr
                    );
                    self.send_error(
                        ErrorCode::InvalidMessage,
                        "Unencrypted message after encryption negotiation",
                    )
                    .await;
                    return Ok(None);
                }

                if text.len() > MAX_MESSAGE_SIZE {
                    error!("Message too large: {} bytes", text.len());
                    anyhow::bail!("Message too large");
                }
                match Message::from_text(text.as_ref()) {
                    Ok(message) => Ok(Some(message)),
                    Err(e) => {
                        error!(
                            "Failed to deserialize text message from {} ({} bytes): {:#}",
                            self.addr,
                            text.len(),
                            e
                        );
                        self.send_error(ErrorCode::InvalidMessage, "Invalid message format")
                            .await;
                        Ok(None)
                    }
                }
            }
            Ok(WebSocketMessage::Binary(data)) => {
                if data.len() > MAX_MESSAGE_SIZE {
                    error!("Message too large: {} bytes", data.len());
                    anyhow::bail!("Message too large");
                }
                match decode_websocket_binary_frame(&data, crypto, transport_binary_decoder) {
                    Ok(Some(message)) => Ok(Some(message)),
                    Ok(None) => Ok(None),
                    Err(e) => {
                        error!(
                            "Failed to deserialize binary message from {} ({} bytes): {:#}",
                            self.addr,
                            data.len(),
                            e
                        );
                        self.send_error(ErrorCode::InvalidMessage, "Invalid message format")
                            .await;
                        Ok(None)
                    }
                }
            }
            Ok(WebSocketMessage::Close(_)) => {
                info!("Client {} closed the WebSocket connection", self.addr);
                Ok(None)
            }
            Ok(WebSocketMessage::Ping(_)) | Ok(WebSocketMessage::Pong(_)) | Ok(_) => Ok(None),
            Err(e) => {
                if is_disconnect_error(&e) {
                    //                     info!("Client {} disconnected while reading message", self.addr);
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!(
                        "Failed to read message from {}: {} ({:?})",
                        self.addr,
                        e,
                        e
                    ))
                }
            }
        }
    }

    async fn handle_persistent_request_connection<S>(
        &mut self,
        websocket: WebSocketStream<S>,
        crypto: Option<Arc<Mutex<MessageCrypto>>>,
    ) -> anyhow::Result<()>
    where
        S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    {
        let crypto = crypto.ok_or_else(|| {
            anyhow::anyhow!("Persistent request connections require message encryption")
        })?;
        let (mut write_half, mut read_half) = websocket.split();
        let limits = TransportLimits::default();
        let (mut outbound, outbound_tx) = OutboundMultiplexer::new(limits.clone())?;
        let writer_crypto = crypto.clone();
        let (writer_status_tx, mut writer_status_rx) = mpsc::channel::<anyhow::Result<()>>(1);
        let writer_task = tokio::spawn(async move {
            let result = async {
                while let Some(frame) = outbound.next_frame().await? {
                    for encrypted in encrypt_request_frame(&frame, &writer_crypto)? {
                        write_half
                            .send(WebSocketMessage::Binary(encrypted.into()))
                            .await?;
                    }
                }
                anyhow::Ok(())
            }
            .await;
            let _ = writer_status_tx.send(result).await;
        });

        let mut inbound = InboundMultiplexer::new(limits)?;
        // Uploads stream their body chunk-by-chunk (files are written
        // as bytes arrive) instead of spooling the whole body — this lifts
        // the max_body_size cap and makes progress visible on disk.
        inbound.set_stream_classifier(|message| matches!(message, Message::UploadStart { .. }));
        let mut received_requests: HashMap<RequestId, Vec<ReceivedMessage>> = HashMap::new();
        let mut handler_tasks: HashMap<RequestId, tokio::task::AbortHandle> = HashMap::new();
        let mut streaming_uploads: HashMap<RequestId, FolderUploadSink> = HashMap::new();
        let mut streaming_requests: HashSet<RequestId> = HashSet::new();
        let per_connection_limit = Arc::new(Semaphore::new(32));
        let global_limit = REQUEST_EXECUTOR_LIMIT
            .get_or_init(|| Arc::new(Semaphore::new(256)))
            .clone();
        let mutation_lanes = Arc::new(tokio::sync::Mutex::new(HashMap::<
            SessionId,
            Arc<tokio::sync::Mutex<()>>,
        >::new()));
        let mut parent_alive = self.parent_alive.clone();

        info!("Persistent request connection {} ready", self.addr);
        let result = 'connection: loop {
            tokio::select! {
                status = writer_status_rx.recv() => {
                    match status {
                        Some(Ok(())) => break Err(anyhow::anyhow!("Request writer stopped")),
                        Some(Err(error)) => break Err(error.context("Request writer failed")),
                        None => break Err(anyhow::anyhow!("Request writer status channel closed")),
                    }
                }
                changed = async {
                    match parent_alive.as_mut() {
                        Some(parent) => parent.changed().await,
                        None => std::future::pending().await,
                    }
                } => {
                    if changed.is_err()
                        || parent_alive.as_ref().is_some_and(|parent| !*parent.borrow())
                    {
                        break Err(anyhow::anyhow!("Primary connection closed"));
                    }
                }
                websocket_frame = read_half.next() => {
                    let Some(websocket_frame) = websocket_frame else {
                        break Err(anyhow::anyhow!("Request peer closed the connection"));
                    };
                    let request_frame = match websocket_frame? {
                        WebSocketMessage::Binary(data) => {
                            let Some(frame) = decrypt_request_frame(&data, &crypto)? else {
                                continue;
                            };
                            frame
                        }
                        WebSocketMessage::Close(reason) => {
                            break Err(anyhow::anyhow!("Request peer closed: {reason:?}"));
                        }
                        WebSocketMessage::Ping(_) | WebSocketMessage::Pong(_) => continue,
                        other => {
                            break Err(anyhow::anyhow!(
                                "Unexpected persistent request frame: {other:?}"
                            ));
                        }
                    };

                    if request_frame.header.kind == RequestFrameKind::WindowUpdate {
                        let credits = InboundMultiplexer::decode_window_update(&request_frame)?;
                        outbound_tx
                            .send(RequestOutboundCommand::AddCredit {
                                request_id: request_frame.header.request_id,
                                message_sequence: request_frame.header.message_sequence,
                                credits,
                            })
                            .await
                            .map_err(|_| anyhow::anyhow!("Request writer stopped"))?;
                        continue;
                    }

                    for event in inbound.accept(request_frame).await? {
                        match event {
                            RequestInboundEvent::BodyProgress { .. } => {}
                            RequestInboundEvent::RequestStarted { request_id, response } => {
                                if response
                                    || received_requests.contains_key(&request_id)
                                    || handler_tasks.contains_key(&request_id)
                                {
                                    break 'connection Err(anyhow::anyhow!(
                                        "Unexpected request start for {request_id}"
                                    ));
                                }
                                received_requests.insert(request_id, Vec::new());
                            }
                            RequestInboundEvent::Message { request_id, message, .. } => {
                                let Some(messages) = received_requests.get_mut(&request_id) else {
                                    break 'connection Err(anyhow::anyhow!(
                                        "Message for unknown request {request_id}"
                                    ));
                                };
                                messages.push(message);
                            }
                            // Upload streaming: the body is consumed
                            // incrementally (files land as bytes arrive).
                            RequestInboundEvent::StreamStarted {
                                request_id, metadata, ..
                            } => {
                                if let Message::UploadStart {
                                    task_id,
                                    session_id,
                                    dst_dir,
                                } = metadata
                                {
                                    info!(
                                        %task_id,
                                        ?session_id,
                                        %dst_dir,
                                        "upload request received (streaming)"
                                    );
                                    streaming_requests.insert(request_id);
                                    let sink = if self
                                        .ensure_session_access(session_id)
                                        .await?
                                    {
                                        self.session_manager
                                            .upload_sink(session_id, task_id, dst_dir)
                                            .await
                                    } else {
                                        FolderUploadSink::failed(
                                            task_id,
                                            session_id,
                                            "Session access denied".to_string(),
                                        )
                                    };
                                    streaming_uploads.insert(request_id, sink);
                                }
                            }
                            RequestInboundEvent::StreamChunk { request_id, payload, .. } => {
                                if let Some(sink) = streaming_uploads.get_mut(&request_id) {
                                    sink.push(&payload).await;
                                }
                            }
                            RequestInboundEvent::StreamEnded { request_id, .. } => {
                                if let Some(sink) = streaming_uploads.remove(&request_id) {
                                    let result = sink.finish();
                                    outbound_tx
                                        .send(RequestOutboundCommand::Start(OutboundRequest {
                                            request_id,
                                            response: true,
                                        }))
                                        .await
                                        .map_err(|_| anyhow::anyhow!("Request writer stopped"))?;
                                    send_correlated_response_message(
                                        &outbound_tx,
                                        request_id,
                                        0,
                                        result,
                                    )
                                    .await?;
                                    outbound_tx
                                        .send(RequestOutboundCommand::Finish(request_id))
                                        .await
                                        .map_err(|_| anyhow::anyhow!("Request writer stopped"))?;
                                }
                            }
                            RequestInboundEvent::RequestFinished(request_id) => {
                                // Streaming uploads have no assembled
                                // Message; their result was already sent at
                                // StreamEnded (or is pending below).
                                if streaming_requests.remove(&request_id) {
                                    received_requests.remove(&request_id);
                                    continue;
                                }
                                let messages = received_requests
                                    .remove(&request_id)
                                    .ok_or_else(|| anyhow::anyhow!("Unknown request {request_id}"))?;
                                if messages.len() != 1 {
                                    queue_correlated_error(
                                        &outbound_tx,
                                        request_id,
                                        format!("Expected one request Message, got {}", messages.len()),
                                    )
                                    .await;
                                    continue;
                                }
                                let request = messages.into_iter().next().unwrap();
                                if !request.metadata.is_request_message() {
                                    queue_correlated_error(
                                        &outbound_tx,
                                        request_id,
                                        format!("{} is not a Request message", request.metadata.msg_name()),
                                    )
                                    .await;
                                    continue;
                                }
                                let child = self.request_child();
                                let response_tx = outbound_tx.clone();
                                let per_connection_limit = per_connection_limit.clone();
                                let global_limit = global_limit.clone();
                                let mutation_lanes = mutation_lanes.clone();
                                let task = tokio::spawn(async move {
                                    let _connection_permit = match per_connection_limit.acquire_owned().await {
                                        Ok(permit) => permit,
                                        Err(_) => return,
                                    };
                                    let _global_permit = match global_limit.acquire_owned().await {
                                        Ok(permit) => permit,
                                        Err(_) => return,
                                    };
                                    let mutation_lane = if is_mutating_request(&request.metadata) {
                                        if let Some(session_id) = request.metadata.session_id() {
                                            Some({
                                                let mut lanes = mutation_lanes.lock().await;
                                                lanes
                                                    .entry(session_id)
                                                    .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                                                    .clone()
                                            })
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };
                                    let _mutation_guard = match mutation_lane.as_ref() {
                                        Some(lane) => Some(lane.lock().await),
                                        None => None,
                                    };
                                    if let Err(error) = run_correlated_request_handler(
                                        child,
                                        request_id,
                                        request,
                                        response_tx.clone(),
                                    )
                                    .await
                                    {
                                        queue_correlated_error(
                                            &response_tx,
                                            request_id,
                                            error.to_string(),
                                        )
                                        .await;
                                    }
                                });
                                handler_tasks.insert(request_id, task.abort_handle());
                            }
                            RequestInboundEvent::Cancelled { request_id, .. } => {
                                received_requests.remove(&request_id);
                                streaming_requests.remove(&request_id);
                                if streaming_uploads.remove(&request_id).is_some() {
                                    info!(%request_id, "upload cancelled mid-stream");
                                }
                                if let Some(task) = handler_tasks.remove(&request_id) {
                                    task.abort();
                                }
                                let _ = outbound_tx
                                    .send(RequestOutboundCommand::Cancel {
                                        request_id,
                                        reason: "request cancelled by peer".into(),
                                    })
                                    .await;
                            }
                            RequestInboundEvent::Error { request_id, message } => {
                                warn!("Request peer reported error for {}: {}", request_id, message);
                                received_requests.remove(&request_id);
                                streaming_requests.remove(&request_id);
                                streaming_uploads.remove(&request_id);
                                if let Some(task) = handler_tasks.remove(&request_id) {
                                    task.abort();
                                }
                            }
                            RequestInboundEvent::WindowUpdate(update) => {
                                outbound_tx
                                    .send(RequestOutboundCommand::Control(update))
                                    .await
                                    .map_err(|_| anyhow::anyhow!("Request writer stopped"))?;
                            }
                            // Upload-side body progress: only clients surface
                            // transfer bars; the server has nothing to update.
                            RequestInboundEvent::BodyProgress { .. } => {}
                        }
                    }
                    handler_tasks.retain(|_, task| !task.is_finished());
                }
            }
        };

        writer_task.abort();
        for (_, task) in handler_tasks {
            task.abort();
        }
        if !streaming_uploads.is_empty() {
            warn!(
                "Request connection closed with {} streaming upload(s) in flight",
                streaming_uploads.len()
            );
            streaming_uploads.clear();
        }
        inbound.abort_all();
        result
    }

    fn request_child(&self) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            addr: self.addr,
            config_manager: self.config_manager.clone(),
            session_manager: self.session_manager.clone(),
            addon_host: self.addon_host.clone(),
            relay_manager: self.relay_manager.clone(),
            port_forward_manager: self.port_forward_manager.clone(),
            fs_watcher_manager: self.fs_watcher_manager.clone(),
            static_http_manager: self.static_http_manager.clone(),
            rx: Some(rx),
            tx,
            streamed_session_ids: HashSet::new(),
            fs_watched_session_ids: HashSet::new(),
            shared_server_id: None,
            shared_server_connection_kind: None,
            auth_token: self.auth_token.clone(),
            connection_kind: ConnectionKind::Request,
            parent_alive: None,
            authorized_session_id: self.authorized_session_id,
            connection_registry: self.connection_registry.clone(),
            supports_binary_output: true,
            voice_supervisor: self.voice_supervisor.clone(),
            screen_supervisor: self.screen_supervisor.clone(),
            // A request-scoped child connection inherits no stream grants;
            // the parent's cleanup revokes by label (same addr).
            screen_streams: HashMap::new(),
            screen_event_forwarder: None,
            // Request children never perform a handshake of their own, so
            // they hold no handshake concurrency slot.
            handshake_permit: None,
            // Request messages (StartScreenStream among them) are handled by
            // these children, so the relay marker must carry over for
            // endpoint selection to work on relayed clients.
            via_relay: self.via_relay,
        }
    }

    async fn handle_message(&mut self, message: Message) -> anyhow::Result<()> {
        // debug!("Received message from {}: {:?}", self.addr, message);

        if let Some(authorized_session_id) = self.authorized_session_id {
            if let Some(message_session_id) = message.session_id() {
                if message_session_id != authorized_session_id {
                    warn!(
                        "Rejected relayed terminal message from {} for unauthorized session {} (authorized={})",
                        self.addr, message_session_id, authorized_session_id
                    );
                    self.send_error(
                        ErrorCode::AuthenticationFailed,
                        "Relayed terminal is not authorized for this session",
                    )
                    .await;
                    return Ok(());
                }
            }
        }

        if let Some(server_id) = self.shared_server_id.clone() {
            self.relay_manager
                .handle_shared_server_message(&server_id, message)
                .await?;
            return Ok(());
        }

        if !self.message_allowed_for_role(&message) {
            warn!(
                "Rejected {:?} connection message from {}: {:?}",
                self.connection_kind, self.addr, message
            );
            self.send_error(
                ErrorCode::InvalidMessage,
                &format!(
                    "Message is not allowed on {:?} connection",
                    self.connection_kind
                ),
            )
            .await;
            return Ok(());
        }

        match message {
            Message::Hello { .. }
            | Message::AuthChallenge { .. }
            | Message::AuthResponse { .. }
            | Message::HelloAck { .. }
            | Message::ServerInfo { .. } => {}
            Message::Ping => {
                info!("Client {} sent protocol ping", self.addr);
                self.send_message(Message::Pong).await?;
            }
            // Everything below is feature handling, delegated to the
            // per-feature modules in `crate::features`. This match is only
            // the routing table from message variants to feature handlers.
            Message::RelayRegisterServer { .. } | Message::ListRelayServers => {
                self.handle_relay_admin_message(message).await?;
            }
            Message::CreateSession { .. }
            | Message::SplitTerminal { .. }
            | Message::FetchHistoryRaw { .. }
            | Message::ExportSessionRawHistory { .. }
            | Message::CloseSession { .. }
            | Message::CloseTab { .. }
            | Message::DetachSession { .. }
            | Message::RenameSession { .. }
            | Message::ApplyEnvironment { .. }
            | Message::SetNotificationEnabled { .. }
            | Message::SetAutoConfirmEnabled { .. }
            | Message::SetSessionEncoding { .. }
            | Message::SetTerminalThemeColors { .. }
            | Message::ClearSessionScreen { .. }
            | Message::Input { .. }
            | Message::ConfirmSession { .. }
            | Message::Resize { .. }
            | Message::ListSessions
            | Message::ListTabGroups
            | Message::GetTabInfoList { .. }
            | Message::UpdateTerminalLayout { .. }
            | Message::CreateTabGroup { .. }
            | Message::DeleteTabGroup { .. }
            | Message::RenameTabGroup { .. }
            | Message::MoveTabToGroup { .. }
            | Message::MoveSessionToTab { .. }
            | Message::ListTerminalProfiles
            | Message::RemoveRecentProjectDirectory { .. }
            | Message::StreamStart { .. }
            | Message::StreamEnd { .. } => {
                self.handle_session_message(message).await?;
            }
            Message::SetFsWatchers { .. }
            | Message::ListPathContent { .. }
            | Message::FindFiles { .. }
            | Message::SearchPathNames { .. }
            | Message::FetchFile { .. }
            | Message::SaveFile { .. }
            | Message::CreatePath { .. }
            | Message::DeletePaths { .. }
            | Message::PastePaths { .. }
            | Message::RenamePath { .. }
            | Message::StartStaticHttpServer { .. } => {
                self.handle_file_message(message).await?;
            }
            Message::TranslateMessage { .. } => {
                self.handle_translation_message(message).await?;
            }
            Message::ListAiConversations { .. }
            | Message::StopWatchingAiConversation { .. }
            | Message::ReadAiConversation { .. } => {
                self.handle_ai_conversation_message(message).await?;
            }
            Message::ListScreenSources { .. }
            | Message::StartScreenStream { .. }
            | Message::StopScreenStream { .. } => {
                self.handle_screen_message(message).await?;
            }
            Message::BootSimulator { .. }
            | Message::ShutdownSimulator { .. }
            | Message::CopySimulatorClipboard { .. }
            | Message::BootAndroidEmulator { .. }
            | Message::ShutdownAndroidEmulator { .. }
            | Message::RotateAndroidDevice { .. } => {
                self.handle_simulator_message(message).await?;
            }
            Message::DebugLaunchList { .. }
            | Message::DebugStart { .. }
            | Message::QuickCommandList { .. }
            | Message::RunQuickCommand { .. }
            | Message::SetupMcpApp { .. }
            | Message::DebugDap { .. }
            | Message::GenerateVtermConfig { .. } => {
                self.handle_debug_tool_message(message).await?;
            }
            Message::GitChangedFiles { .. }
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
            | Message::GitAnalyzeDiff { .. }
            | Message::GitBranches { .. }
            | Message::GitStashList { .. }
            | Message::GitStashPush { .. }
            | Message::GitStashApply { .. }
            | Message::GitStashPop { .. }
            | Message::GitStashDrop { .. }
            | Message::GitSubmoduleList { .. }
            | Message::GitCheckout { .. }
            | Message::GitCreateBranch { .. }
            | Message::GitDeleteBranch { .. }
            | Message::GitMerge { .. }
            | Message::GitRebaseStart { .. }
            | Message::GitRebaseApply { .. }
            | Message::GitRebaseConflictContent { .. }
            | Message::GitRebaseResolve { .. }
            | Message::GitRebaseContinue { .. }
            | Message::GitRebaseSkip { .. }
            | Message::GitRebaseAbort { .. }
            | Message::GitRebaseAiResolve { .. }
            | Message::GitRebaseAiResolveCancel { .. }
            | Message::GitCreateWorktree { .. } => {
                self.handle_git_message(message).await?;
            }
            Message::RequestReverseProxy { .. } | Message::ListReverseProxies { .. } => {
                self.handle_port_forward_message(message).await?;
            }
            Message::TranscribeAudio { .. } | Message::PolishVoiceInputText { .. } => {
                self.handle_voice_message(message).await?;
            }
            _ => {
                warn!("Unhandled message: {}", message.msg_name());
            }
        }

        Ok(())
    }

    fn message_allowed_for_role(&self, message: &Message) -> bool {
        match self.connection_kind {
            ConnectionKind::Primary => {
                !message.is_terminal_client_message() && !message.is_request_message()
            }
            ConnectionKind::Terminal => message.is_terminal_client_message(),
            ConnectionKind::Debug => matches!(
                message,
                Message::StreamStart { .. } | Message::StreamEnd { .. } | Message::DebugDap { .. }
            ),
            ConnectionKind::Request | ConnectionKind::RelayRequest => message.is_request_message(),
            ConnectionKind::RelayPrimary => {
                !message.is_terminal_client_message() && !message.is_request_message()
            }
        }
    }

    fn can_access_session(&self, session_id: SessionId) -> bool {
        match self.connection_kind {
            ConnectionKind::Primary
            | ConnectionKind::Request
            | ConnectionKind::RelayPrimary
            | ConnectionKind::RelayRequest => true,
            ConnectionKind::Terminal | ConnectionKind::Debug => {
                self.streamed_session_ids.contains(&session_id)
            }
        }
    }

    pub(crate) async fn ensure_session_access(
        &mut self,
        session_id: SessionId,
    ) -> anyhow::Result<bool> {
        if self.connection_kind == ConnectionKind::Terminal
            && !self.streamed_session_ids.contains(&session_id)
        {
            info!(
                "Terminal connection {} implicitly starting stream for session {} before terminal operation",
                self.addr, session_id
            );
            self.start_session_stream(session_id, None).await?;
        }

        Ok(self.can_access_session(session_id))
    }

    pub(crate) async fn send_message(&self, message: Message) -> anyhow::Result<()> {
        self.tx
            .send(message)
            .map_err(|_| anyhow::anyhow!("Failed to send message"))
    }

    pub(crate) async fn send_error(&self, code: ErrorCode, message: &str) {
        let error_msg = Message::Error {
            code,
            message: message.to_string(),
        };
        let _ = self.tx.send(error_msg);
    }

    async fn send_handshake_error<S>(
        &self,
        websocket: &mut WebSocketStream<S>,
        code: ErrorCode,
        message: &str,
    ) -> anyhow::Result<()>
    where
        S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    {
        let payload = Message::Error {
            code,
            message: message.to_string(),
        }
        .to_text()?;
        websocket
            .send(WebSocketMessage::Text(payload.into()))
            .await
            .map_err(|error| anyhow::anyhow!("Failed to send handshake error: {}", error))
    }

    /// Uniform handshake authentication rejection (reviews §1.2 + §1.3):
    /// counts the failure against the peer's IP, logs the distinguishing
    /// detail server-side only, and answers every failure stage with the
    /// same generic message so probers get no oracle. Returns the error the
    /// caller should propagate.
    async fn reject_auth<S>(
        &self,
        websocket: &mut WebSocketStream<S>,
        detail: &str,
    ) -> anyhow::Error
    where
        S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    {
        self.connection_registry.record_auth_failure(self.addr.ip());
        warn!(
            "Handshake authentication failed for {}: {}",
            self.addr, detail
        );
        let _ = self
            .send_handshake_error(
                websocket,
                ErrorCode::AuthenticationFailed,
                AUTH_FAILED_MESSAGE,
            )
            .await;
        anyhow::anyhow!("Authentication failed for {}: {}", self.addr, detail)
    }

    pub(crate) fn has_valid_auth_token(&self, provided_token: Option<&str>) -> bool {
        self.auth_token.as_deref() == provided_token
    }
}

async fn run_correlated_request_handler(
    mut connection: Connection,
    request_id: RequestId,
    request: ReceivedMessage,
    outbound_tx: mpsc::Sender<RequestOutboundCommand>,
) -> anyhow::Result<()> {
    outbound_tx
        .send(RequestOutboundCommand::Start(OutboundRequest {
            request_id,
            response: true,
        }))
        .await
        .map_err(|_| anyhow::anyhow!("Persistent request writer stopped"))?;

    let request_name = request.metadata.msg_name();
    // A StartScreenStream request is long-lived: the handler answers with a
    // single ScreenStreamReady and then holds the response channel open for
    // the whole stream (frames travel over the separate media connection).
    // Its idle-timeout firing — usually after the client closed its window
    // and stopped the stream — is normal end-of-life, not a failure.
    let is_screen_stream_request = matches!(request.metadata, Message::StartScreenStream { .. });
    let is_android_boot_request = matches!(request.metadata, Message::BootAndroidEmulator { .. });
    let is_static_http_request = matches!(request.metadata, Message::StartStaticHttpServer { .. });
    if let Some(session_id) = request.metadata.session_id() {
        if connection.relay_manager.is_remote_session(session_id).await {
            // Static HTTP server requests need to be forwarded to the shared server
            // so it can start the server and establish a port forward tunnel.
            if !is_static_http_request {
                send_correlated_response_message(
                    &outbound_tx,
                    request_id,
                    0,
                    Message::Error {
                        code: ErrorCode::InvalidMessage,
                        message: "Remote session request must use RelayRequestConnection"
                            .to_string(),
                    },
                )
                .await?;
                outbound_tx
                    .send(RequestOutboundCommand::Finish(request_id))
                    .await
                    .map_err(|_| anyhow::anyhow!("Persistent request writer stopped"))?;
                return Ok(());
            }
        }
    }
    if let Message::FetchFile {
        session_id,
        path,
        max_size,
    } = &request.metadata
    {
        if crate::features::files::handle_fetch_file_request(
            &mut connection,
            &outbound_tx,
            request_id,
            *session_id,
            path.clone(),
            *max_size,
        )
        .await?
        {
            return Ok(());
        }
    }
    // Download: the response body is a framed entry stream produced
    // by a lazy server-side walk (EncodedMessage with a streaming reader).
    if let Message::DownloadStart {
        task_id,
        session_id,
        paths,
    } = &request.metadata
    {
        if crate::features::files::handle_download_request(
            &mut connection,
            &outbound_tx,
            request_id,
            task_id.clone(),
            *session_id,
            paths.clone(),
        )
        .await?
        {
            return Ok(());
        }
    }
    let mut rx = connection.rx.take().expect("request child rx must exist");
    let handle_result = match request {
        ReceivedMessage {
            metadata:
                Message::TempFileUploadStart {
                    task_id,
                    session_id,
                    file_name,
                    content_type,
                    usage,
                },
            body,
        } if !connection.relay_manager.is_remote_session(session_id).await => {
            crate::features::files::handle_temp_file_upload_body_request(
                &mut connection,
                session_id,
                task_id,
                file_name,
                content_type,
                usage,
                body,
            )
            .await
        }
        ReceivedMessage {
            metadata:
                Message::UploadStart {
                    task_id,
                    session_id,
                    dst_dir,
                },
            body,
        } if !connection.relay_manager.is_remote_session(session_id).await => {
            crate::features::files::handle_upload_body_request(
                &mut connection,
                session_id,
                task_id,
                dst_dir,
                body,
            )
            .await
        }
        ReceivedMessage {
            metadata:
                Message::SaveFile {
                    session_id,
                    path,
                    data,
                },
            body,
        } if data.is_empty() && !connection.relay_manager.is_remote_session(session_id).await => {
            crate::features::files::handle_save_file_body_request(
                &mut connection,
                session_id,
                path,
                body,
            )
            .await
        }
        request => {
            let request = decode_received_message(request).await?;
            connection.handle_message(request).await
        }
    };
    if let Err(error) = handle_result {
        send_correlated_response_message(
            &outbound_tx,
            request_id,
            0,
            Message::Error {
                code: ErrorCode::ServerError,
                message: error.to_string(),
            },
        )
        .await?;
        outbound_tx
            .send(RequestOutboundCommand::Finish(request_id))
            .await
            .map_err(|_| anyhow::anyhow!("Persistent request writer stopped"))?;
        return Ok(());
    }

    // The child connection owns a private response channel. Dropping its
    // original sender after dispatch makes producer lifetime the explicit
    // completion signal: synchronous handlers close immediately, while
    // spawned streaming handlers keep their cloned sender until they finish.
    drop(connection);
    let mut message_sequence = 0u32;
    loop {
        let received = match tokio::time::timeout(Duration::from_secs(60), rx.recv()).await {
            Ok(received) => received,
            Err(_) => {
                // Screen streams stay open after ScreenStreamReady. Android
                // boot likewise keeps its channel open for the later Booted
                // push after the immediate BootAndroidEmulatorResult.
                if is_screen_stream_request || (is_android_boot_request && message_sequence > 0) {
                    tracing::debug!(
                        "long-lived request {} went idle; finishing without error",
                        request_name
                    );
                    break;
                }
                send_correlated_response_message(
                    &outbound_tx,
                    request_id,
                    message_sequence,
                    Message::Error {
                        code: ErrorCode::ServerError,
                        message: format!("Timed out waiting for response to {request_name}"),
                    },
                )
                .await?;
                break;
            }
        };
        match received {
            Some(response) => {
                send_correlated_response_message(
                    &outbound_tx,
                    request_id,
                    message_sequence,
                    response,
                )
                .await?;
                message_sequence = message_sequence.wrapping_add(1);
            }
            None => {
                if message_sequence == 0 {
                    send_correlated_response_message(
                        &outbound_tx,
                        request_id,
                        0,
                        Message::Error {
                            code: ErrorCode::ServerError,
                            message: format!(
                                "Request handler for {request_name} completed without a response"
                            ),
                        },
                    )
                    .await?;
                }
                break;
            }
        }
    }

    outbound_tx
        .send(RequestOutboundCommand::Finish(request_id))
        .await
        .map_err(|_| anyhow::anyhow!("Persistent request writer stopped"))?;
    Ok(())
}

pub(crate) async fn send_correlated_response_message(
    outbound_tx: &mpsc::Sender<RequestOutboundCommand>,
    request_id: RequestId,
    message_sequence: u32,
    message: Message,
) -> anyhow::Result<()> {
    outbound_tx
        .send(RequestOutboundCommand::Message {
            request_id,
            message_sequence,
            message: encode_message(message)?,
        })
        .await
        .map_err(|_| anyhow::anyhow!("Persistent request writer stopped"))
}

async fn queue_correlated_error(
    outbound_tx: &mpsc::Sender<RequestOutboundCommand>,
    request_id: RequestId,
    message: String,
) {
    if outbound_tx
        .send(RequestOutboundCommand::Start(OutboundRequest {
            request_id,
            response: true,
        }))
        .await
        .is_err()
    {
        return;
    }
    let _ = send_correlated_response_message(
        outbound_tx,
        request_id,
        0,
        Message::Error {
            code: ErrorCode::InvalidMessage,
            message,
        },
    )
    .await;
    let _ = outbound_tx
        .send(RequestOutboundCommand::Finish(request_id))
        .await;
}

fn is_mutating_request(message: &Message) -> bool {
    matches!(
        message,
        Message::SaveFile { .. }
            | Message::TempFileUploadStart { .. }
            | Message::UploadStart { .. }
            | Message::CreatePath { .. }
            | Message::DeletePaths { .. }
            | Message::PastePaths { .. }
            | Message::RenamePath { .. }
            | Message::GitStage { .. }
            | Message::GitUnstage { .. }
            | Message::GitDiscard { .. }
            | Message::GitCommit { .. }
            | Message::GitPull { .. }
            | Message::GitPush { .. }
            | Message::GitRevert { .. }
            | Message::GitCheckout { .. }
            | Message::GitCreateBranch { .. }
            | Message::GitCreateWorktree { .. }
            | Message::GitDeleteBranch { .. }
            | Message::GitMerge { .. }
            | Message::GitRebaseStart { .. }
            | Message::GitRebaseApply { .. }
            | Message::GitRebaseResolve { .. }
            | Message::GitRebaseContinue { .. }
            | Message::GitRebaseSkip { .. }
            | Message::GitRebaseAbort { .. }
    )
}

fn challenge_key_hash_for_peer(
    local_peer: bool,
    configured_secret_key: &str,
) -> anyhow::Result<Option<String>> {
    if local_peer {
        return Ok(None);
    }

    let configured_secret_key = configured_secret_key.trim();
    if configured_secret_key.is_empty() {
        // Defense in depth (review §1.5): config validation and the
        // store's auto-generation keep healthy setups from reaching this,
        // but an unconfigured secret must never silently degrade into an
        // unauthenticated public server.
        anyhow::bail!(
            "Server secret key is not configured; refusing unauthenticated non-local peer"
        );
    }

    if let Some(stored_hash) = stored_server_secret_key_hash(configured_secret_key) {
        return Ok(Some(stored_hash.to_string()));
    }

    if configured_secret_key.starts_with("enc-v") {
        anyhow::bail!("Server secret key is protected but could not be decrypted");
    }

    Ok(Some(hash_secret_key(configured_secret_key)))
}

async fn read_plain_handshake_message<S>(
    websocket: &mut WebSocketStream<S>,
) -> anyhow::Result<Message>
where
    S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
{
    // Slowloris guard (review §1.1): the AuthResponse must arrive promptly.
    let Some(frame) = tokio::time::timeout(HANDSHAKE_FRAME_TIMEOUT, websocket.next())
        .await
        .map_err(|_| anyhow::anyhow!("Authentication response timed out"))?
    else {
        anyhow::bail!("Peer disconnected during handshake");
    };

    match frame {
        Ok(WebSocketMessage::Text(text)) => Ok(Message::from_text(text.as_ref())?),
        Ok(WebSocketMessage::Binary(data)) => Ok(Message::deserialize(&data)?),
        Ok(WebSocketMessage::Close(_)) => anyhow::bail!("Peer closed during handshake"),
        Ok(_) => anyhow::bail!("Unexpected WebSocket frame during handshake"),
        Err(error) => anyhow::bail!("Failed to read handshake frame: {}", error),
    }
}

async fn perform_message_encryption_handshake_responder<S>(
    websocket: &mut WebSocketStream<S>,
) -> anyhow::Result<MessageCrypto>
where
    S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
{
    let mut handshake = MessageCryptoHandshake::new_responder()?;

    // Slowloris guard (review §1.1): every Noise exchange frame must arrive
    // within the handshake window.
    let frame = read_encryption_handshake_frame(websocket, "encryption handshake").await?;
    handshake.read_frame(&frame)?;

    websocket
        .send(WebSocketMessage::Binary(handshake.write_frame()?.into()))
        .await
        .map_err(|error| anyhow::anyhow!("Failed to send encryption handshake: {}", error))?;

    let frame = read_encryption_handshake_frame(websocket, "final encryption handshake").await?;
    handshake.read_frame(&frame)?;

    handshake.into_transport()
}

/// One binary frame of the Noise exchange, bounded by
/// [`HANDSHAKE_FRAME_TIMEOUT`] (slowloris guard, review §1.1).
async fn read_encryption_handshake_frame<S>(
    websocket: &mut WebSocketStream<S>,
    context: &str,
) -> anyhow::Result<Vec<u8>>
where
    S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
{
    let Some(frame) = tokio::time::timeout(HANDSHAKE_FRAME_TIMEOUT, websocket.next())
        .await
        .map_err(|_| anyhow::anyhow!("{context} timed out"))?
    else {
        anyhow::bail!("Client closed during encryption handshake");
    };
    match frame {
        Ok(WebSocketMessage::Binary(data)) => Ok(data),
        Ok(other) => anyhow::bail!("Unexpected encryption handshake frame: {:?}", other),
        Err(error) => anyhow::bail!("Failed to read encryption handshake: {}", error),
    }
}

pub(crate) fn encode_websocket_messages(
    message: Message,
    crypto: Option<&Arc<Mutex<MessageCrypto>>>,
    prefer_binary: bool,
) -> anyhow::Result<Vec<WebSocketMessage>> {
    if let Some(crypto) = crypto {
        if prefer_binary {
            if let Some(payloads) = message.to_transport_binary_frames() {
                let mut frames = Vec::with_capacity(payloads.len());
                for payload in payloads {
                    let mut plaintext = Vec::with_capacity(1 + payload.len());
                    plaintext.push(TRANSPORT_TAG_BINARY);
                    plaintext.extend_from_slice(&payload);
                    for encrypted in crypto
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Message crypto lock poisoned"))?
                        .encrypt_frame(&plaintext)?
                    {
                        if encrypted.len() > MAX_MESSAGE_SIZE {
                            anyhow::bail!(
                                "Message too large for transport: {} bytes",
                                encrypted.len()
                            );
                        }
                        frames.push(WebSocketMessage::Binary(encrypted.into()));
                    }
                }
                return Ok(frames);
            }
        }

        let mut plaintext = Vec::new();
        plaintext.push(TRANSPORT_TAG_TEXT);
        plaintext.extend_from_slice(message.to_text()?.as_bytes());
        let mut frames = Vec::new();
        for encrypted in crypto
            .lock()
            .map_err(|_| anyhow::anyhow!("Message crypto lock poisoned"))?
            .encrypt_frame(&plaintext)?
        {
            if encrypted.len() > MAX_MESSAGE_SIZE {
                anyhow::bail!("Message too large for transport: {} bytes", encrypted.len());
            }
            frames.push(WebSocketMessage::Binary(encrypted.into()));
        }
        return Ok(frames);
    }

    if prefer_binary {
        if let Some(payloads) = message.to_transport_binary_frames() {
            let mut frames = Vec::with_capacity(payloads.len());
            for payload in payloads {
                if payload.len() > MAX_MESSAGE_SIZE {
                    anyhow::bail!("Message too large for transport: {} bytes", payload.len());
                }
                frames.push(WebSocketMessage::Binary(payload.into()));
            }
            return Ok(frames);
        }
    }

    let payload = message.to_text()?;
    if payload.len() > MAX_MESSAGE_SIZE {
        anyhow::bail!("Message too large for transport: {} bytes", payload.len());
    }
    Ok(vec![WebSocketMessage::Text(payload.into())])
}

pub(crate) fn decode_websocket_binary_frame(
    data: &[u8],
    crypto: Option<&Arc<Mutex<MessageCrypto>>>,
    transport_binary_decoder: &mut TransportBinaryFrameDecoder,
) -> anyhow::Result<Option<Message>> {
    if is_encrypted_message_frame(data) {
        let crypto = crypto.ok_or_else(|| {
            anyhow::anyhow!("Received encrypted frame before encryption was negotiated")
        })?;
        // `None` = a fragment of a larger message; more chunks to come.
        let Some(plaintext) = crypto
            .lock()
            .map_err(|_| anyhow::anyhow!("Message crypto lock poisoned"))?
            .decrypt_frame(data)?
        else {
            return Ok(None);
        };
        let Some((&tag, payload)) = plaintext.split_first() else {
            anyhow::bail!("Encrypted frame payload is empty");
        };
        return match tag {
            TRANSPORT_TAG_TEXT => Message::from_text(std::str::from_utf8(payload)?).map(Some),
            TRANSPORT_TAG_BINARY => transport_binary_decoder.decode_message_frame(payload),
            other => anyhow::bail!("Unknown encrypted transport tag {}", other),
        };
    }

    if crypto.is_some() {
        anyhow::bail!("Received unencrypted binary frame after encryption was negotiated");
    }

    transport_binary_decoder.decode_message_frame(data)
}

/// Format client-sent terminal input into a compact escaped preview for logging.
#[allow(dead_code)]
fn format_input_preview(data: &[u8]) -> String {
    const MAX_PREVIEW_CHARS: usize = 80;

    let mut preview = String::new();
    for byte in data.iter().copied().take(MAX_PREVIEW_CHARS) {
        match byte {
            b'\r' => preview.push_str("\\r"),
            b'\n' => preview.push_str("\\n"),
            b'\t' => preview.push_str("\\t"),
            0x1B => preview.push_str("\\x1b"),
            0x20..=0x7E => preview.push(byte as char),
            _ => {
                let _ = std::fmt::Write::write_fmt(&mut preview, format_args!("\\x{byte:02x}"));
            }
        }
    }

    if data.len() > MAX_PREVIEW_CHARS {
        preview.push_str("...");
    }

    preview
}

fn is_local_machine(ip: IpAddr) -> bool {
    let ip = ip.to_canonical();
    if ip.is_loopback() {
        return true;
    }

    if let Ok(interfaces) = if_addrs::get_if_addrs() {
        return interfaces
            .into_iter()
            .any(|interface| match interface.addr {
                IfAddr::V4(v4) => IpAddr::V4(v4.ip) == ip,
                IfAddr::V6(v6) => IpAddr::V6(v6.ip) == ip,
            });
    }

    false
}

fn generate_auth_token() -> String {
    Uuid::new_v4().to_string()
}

fn token_log_id(token: &str) -> String {
    token.chars().take(8).collect()
}

fn log_outgoing_connection_message(
    connection_kind: ConnectionKind,
    addr: SocketAddr,
    message: &Message,
) {
    match message {
        Message::Output {
            session_id,
            start_offset,
            data,
        } => info!(
            "{:?} connection {} writing terminal output session={} start_offset={} bytes={}",
            connection_kind,
            addr,
            session_id,
            start_offset,
            data.len()
        ),
        Message::SessionBufferInfo {
            session_id,
            start_offset,
            end_offset,
        } => info!(
            "{:?} connection {} writing buffer info session={} start_offset={} end_offset={}",
            connection_kind, addr, session_id, start_offset, end_offset
        ),
        _ => {}
    }
}

fn generate_auth_challenge_nonce() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

/// Classify expected disconnect conditions from the WebSocket layer.
fn is_disconnect_error(error: &tungstenite::Error) -> bool {
    matches!(
        error,
        tungstenite::Error::ConnectionClosed
            | tungstenite::Error::AlreadyClosed
            | tungstenite::Error::Protocol(_)
            | tungstenite::Error::Io(_)
    )
}

fn is_send_failure(error: &anyhow::Error) -> bool {
    error.to_string().contains("Failed to send message")
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

    use super::{
        challenge_key_hash_for_peer, coalesce_queued_output, generate_auth_token,
        ConnectionRegistry, AUTH_FAILURE_LOCKOUT, AUTH_FAILURE_MAX, AUTH_FAILURE_WINDOW,
    };
    use std::time::{Duration, Instant};
    use tokio::sync::mpsc;
    use vterm_common::{hash_secret_key, SessionId};
    use vterm_core::protocol::Message;
    #[test]
    fn coalesces_contiguous_output_for_the_same_session() {
        let session_id = SessionId::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        tx.send(Message::Output {
            session_id,
            start_offset: 3,
            data: b"def".to_vec(),
        })
        .unwrap();
        let mut message = Message::Output {
            session_id,
            start_offset: 0,
            data: b"abc".to_vec(),
        };

        assert!(coalesce_queued_output(&mut message, &mut rx).is_none());
        match message {
            Message::Output {
                start_offset, data, ..
            } => {
                assert_eq!(start_offset, 0);
                assert_eq!(data, b"abcdef");
            }
            _ => panic!("expected terminal output"),
        }
    }

    #[test]
    fn preserves_non_contiguous_output_as_the_next_message() {
        let session_id = SessionId::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        tx.send(Message::Output {
            session_id,
            start_offset: 4,
            data: b"later".to_vec(),
        })
        .unwrap();
        let mut message = Message::Output {
            session_id,
            start_offset: 0,
            data: b"abc".to_vec(),
        };

        let pending = coalesce_queued_output(&mut message, &mut rx)
            .expect("non-contiguous output must stay queued");
        assert!(matches!(
            pending,
            Message::Output {
                start_offset: 4,
                ..
            }
        ));
    }

    #[test]
    fn remote_handshake_requires_challenge_key_hash() {
        let secret_key = "abcdefghijkl";

        assert_eq!(
            challenge_key_hash_for_peer(false, secret_key)
                .unwrap()
                .as_deref(),
            Some(hash_secret_key(secret_key).as_str())
        );
    }

    #[test]
    fn local_handshake_bypasses_challenge() {
        assert!(challenge_key_hash_for_peer(true, "abcdefghijkl")
            .unwrap()
            .is_none());
    }

    #[test]
    fn empty_secret_key_rejects_non_local_peers() {
        // Defense in depth (review §1.5): an unconfigured secret must never
        // silently turn into an unauthenticated public server.
        assert!(challenge_key_hash_for_peer(false, "").is_err());
        // Local peers keep the zero-config companion flow.
        assert!(challenge_key_hash_for_peer(true, "").unwrap().is_none());
    }

    #[test]
    fn auth_failure_lockout_engages_after_budget_and_expires() {
        let registry = ConnectionRegistry::new();
        let ip = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 9));
        let now = Instant::now();

        for _ in 0..AUTH_FAILURE_MAX {
            assert!(!registry.is_auth_limited_at(ip, now));
            registry.record_auth_failure_at(ip, now);
        }
        assert!(registry.is_auth_limited_at(ip, now));
        assert!(!registry.is_auth_limited_at(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 4)), now));

        let expired = now + AUTH_FAILURE_LOCKOUT + Duration::from_millis(1);
        assert!(!registry.is_auth_limited_at(ip, expired));
        registry.record_auth_failure_at(ip, expired);
        assert!(registry.is_auth_limited_at(ip, expired));
    }

    #[test]
    fn auth_failures_age_out_of_the_window() {
        let registry = ConnectionRegistry::new();
        let ip = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 10));
        let now = Instant::now();

        for _ in 0..AUTH_FAILURE_MAX - 1 {
            registry.record_auth_failure_at(ip, now);
        }
        let next_window = now + AUTH_FAILURE_WINDOW + Duration::from_millis(1);
        registry.record_auth_failure_at(ip, next_window);
        assert!(!registry.is_auth_limited_at(ip, next_window));
    }

    #[test]
    fn auth_success_clears_failures_and_lockout() {
        let registry = ConnectionRegistry::new();
        let ip = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 11));
        let now = Instant::now();
        for _ in 0..AUTH_FAILURE_MAX {
            registry.record_auth_failure_at(ip, now);
        }
        assert!(registry.is_auth_limited_at(ip, now));

        registry.record_auth_success(ip);
        assert!(!registry.is_auth_limited_at(ip, now));
        registry.record_auth_failure_at(ip, now);
        assert!(!registry.is_auth_limited_at(ip, now));
    }

    #[test]
    fn generated_auth_tokens_are_unique() {
        let first = generate_auth_token();
        let second = generate_auth_token();

        assert_ne!(first, second);
        assert!(!first.is_empty());
        assert!(!second.is_empty());
    }

    #[test]
    fn registry_validates_and_invalidates_primary_tokens() {
        let registry = ConnectionRegistry::new();
        let token = "token-123".to_string();

        assert!(registry.validate_child(&token).is_none());
        assert!(!registry.has_active_lan_peer());
        registry.register_primary(token.clone(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert!(!registry.has_active_lan_peer());
        registry.register_primary(
            "lan-token".to_string(),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 155)),
        );
        assert!(registry.has_active_lan_peer());
        let child = registry
            .validate_child(&token)
            .expect("registered primary token should validate");
        assert!(*child.borrow());

        registry.unregister_primary(&token);
        assert!(registry.validate_child(&token).is_none());
        assert!(!*child.borrow());
    }
}
