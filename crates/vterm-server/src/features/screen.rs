//! Screen sharing: source discovery and H.264 screen streams, either
//! locally via the supervised screen-server or relay-coordinated for
//! shared-server sources (spec §5/§7).

use tracing::{info, warn};
use vterm_core::protocol::Message;

use crate::connection::Connection;

/// A screen-stream grant held by a connection: the screen-server token plus,
/// for relayed clients, the relay UDP pairing to release on teardown.
pub(crate) struct ScreenStreamGrant {
    token: [u8; 16],
    pairing: Option<String>,
    /// When this grant is for a shared server's screen: the shared server's
    /// id and the hex-encoded token, used to revoke on the shared server
    /// via the relay coordination channel.
    shared_server_id: Option<String>,
    shared_token_hex: Option<String>,
}

impl Connection {
    pub(crate) async fn handle_screen_message(&mut self, message: Message) -> anyhow::Result<()> {
        self.ensure_screen_event_forwarder().await;
        match message {
            // Screen streaming (coordinated via vterm-screen-server, spec §5).
            // The relay (localhost server) is the sole coordinator: for local
            // sources it uses its own screen-server; for shared-server sources
            // it allocates a relay UDP port and asks the shared server only
            // for a token + screen-server UDP port.
            Message::ListScreenSources { server_id, .. } => {
                let local_server_id = self.config_manager.snapshot().await.server.server_id;
                let is_shared = server_id
                    .as_deref()
                    .map_or(false, |id| id != local_server_id);
                info!(
                    "ListScreenSources from {}: server_id={:?}, local={}, is_shared={}",
                    self.addr, server_id, local_server_id, is_shared
                );
                let message = if is_shared {
                    let sid = server_id.unwrap();
                    info!(
                        "Client {} listing screen sources from shared server {}",
                        self.addr, sid
                    );
                    match self.relay_manager.request_screen_sources(&sid).await {
                        Ok(Ok((revision, sources))) => Message::ScreenSourcesResult {
                            sources,
                            error: None,
                            server_id: Some(sid.clone()),
                            revision,
                        },
                        Ok(Err(e)) => Message::ScreenSourcesResult {
                            sources: Vec::new(),
                            error: Some(e),
                            server_id: Some(sid.clone()),
                            revision: None,
                        },
                        Err(e) => Message::ScreenSourcesResult {
                            sources: Vec::new(),
                            error: Some(format!("{e:#}")),
                            server_id: Some(sid),
                            revision: None,
                        },
                    }
                } else {
                    // Echo the request's server_id: a relayed client names
                    // this host explicitly (server_id == local id), and its
                    // per-server source cache keys on the echoed id — a flat
                    // `None` would be cached under the wrong key and the
                    // remote picker would show an empty list.
                    match self.screen_supervisor.list_source_snapshot().await {
                        Ok((revision, entries)) => Message::ScreenSourcesResult {
                            sources: entries
                                .iter()
                                .map(crate::screen::source_entry_to_proto)
                                .collect(),
                            error: None,
                            server_id: server_id.clone(),
                            revision: Some(revision),
                        },
                        Err(e) => Message::ScreenSourcesResult {
                            sources: Vec::new(),
                            error: Some(format!("{e:#}")),
                            server_id: server_id.clone(),
                            revision: None,
                        },
                    }
                };
                self.send_message(message).await?;
            }
            Message::StartScreenStream {
                source, server_id, ..
            } => {
                info!(
                    "Client {} requested screen stream source={}",
                    self.addr, source
                );
                let label = self.addr.to_string();
                let local_server_id = self.config_manager.snapshot().await.server.server_id;
                let response_server_id = server_id.clone();
                let is_shared = server_id
                    .as_deref()
                    .map_or(false, |id| id != local_server_id);
                let message = if is_shared {
                    let sid = server_id.unwrap();
                    info!(
                        "Relay-coordinated screen stream for shared server {} source={}",
                        sid, source
                    );
                    // 1. Ask the shared server for a token + screen-server UDP port.
                    let (token_hex, screen_udp_port) = match self
                        .relay_manager
                        .request_screen_token(&sid, &source, &label)
                        .await
                    {
                        Ok(Ok((token_hex, port))) => (token_hex, port),
                        Ok(Err(e)) => {
                            self.send_message(Message::ScreenStreamReady {
                                source,
                                server_id: response_server_id.clone(),
                                endpoint: vterm_core::protocol::StreamEndpoint::Direct {
                                    host: String::new(),
                                    port: 0,
                                },
                                token: String::new(),
                                codec: String::new(),
                                format: String::new(),
                                error: Some(e),
                            })
                            .await?;
                            return Ok(());
                        }
                        Err(e) => {
                            self.send_message(Message::ScreenStreamReady {
                                source,
                                server_id: response_server_id.clone(),
                                endpoint: vterm_core::protocol::StreamEndpoint::Direct {
                                    host: String::new(),
                                    port: 0,
                                },
                                token: String::new(),
                                codec: String::new(),
                                format: String::new(),
                                error: Some(format!("{e:#}")),
                            })
                            .await?;
                            return Ok(());
                        }
                    };
                    // 2. Allocate a relay UDP port forwarding to the shared
                    //    server's screen-server.
                    let peer_ip = match self.relay_manager.shared_server_peer_ip(&sid).await {
                        Some(ip) => ip.to_string(),
                        None => {
                            self.relay_manager
                                .revoke_screen_token(&sid, &token_hex)
                                .await;
                            self.send_message(Message::ScreenStreamReady {
                                source,
                                server_id: response_server_id.clone(),
                                endpoint: vterm_core::protocol::StreamEndpoint::Direct {
                                    host: String::new(),
                                    port: 0,
                                },
                                token: String::new(),
                                codec: String::new(),
                                format: String::new(),
                                error: Some("shared server gone".to_string()),
                            })
                            .await?;
                            return Ok(());
                        }
                    };
                    let allocator = self.relay_manager.udp_allocator().await;
                    match allocator.allocate(&peer_ip, screen_udp_port).await {
                        Ok(allocated) => {
                            let host = self
                                .relay_manager
                                .upstream_relay_host()
                                .await
                                .unwrap_or_default();
                            info!(
                                "Relay screen stream: allocated UDP port {} → {}:{} (pairing {}) for source={}",
                                allocated.port, peer_ip, screen_udp_port, allocated.pairing, source
                            );
                            self.relay_manager
                                .track_upstream_udp_pairing(&label, allocated.pairing.clone());
                            self.screen_streams.insert(
                                source.clone(),
                                ScreenStreamGrant {
                                    token: crate::screen::ctl::token_from_hex(&token_hex)
                                        .unwrap_or([0u8; 16]),
                                    pairing: Some(allocated.pairing.clone()),
                                    shared_server_id: Some(sid),
                                    shared_token_hex: Some(token_hex.clone()),
                                },
                            );
                            Message::ScreenStreamReady {
                                source,
                                server_id: response_server_id.clone(),
                                endpoint: vterm_core::protocol::StreamEndpoint::Relay {
                                    host,
                                    port: allocated.port,
                                    pairing: allocated.pairing,
                                },
                                token: token_hex,
                                codec: "h264".to_string(),
                                format: "nv12".to_string(),
                                error: None,
                            }
                        }
                        Err(error) => {
                            self.relay_manager
                                .revoke_screen_token(&sid, &token_hex)
                                .await;
                            Message::ScreenStreamReady {
                                source,
                                server_id: response_server_id.clone(),
                                endpoint: vterm_core::protocol::StreamEndpoint::Direct {
                                    host: String::new(),
                                    port: 0,
                                },
                                token: String::new(),
                                codec: String::new(),
                                format: String::new(),
                                error: Some(format!("relay UDP allocation failed: {error:#}")),
                            }
                        }
                    }
                } else {
                    // Local screen-server: direct path. Mutter forbids screen
                    // casting while the lock screen is up, so unlock the
                    // local session before the capture starts — connecting
                    // remotely then means "unlock this machine".
                    #[cfg(target_os = "linux")]
                    match crate::session_unlock::ensure_unlocked().await {
                        Ok(true) => {}
                        Ok(false) => info!(
                            "session still locked; the screen stream may stay black \
                             until the host unlocks"
                        ),
                        Err(e) => warn!("unlock-on-connect failed: {e:#}"),
                    }
                    match self
                        .screen_supervisor
                        .start_stream(&source, &label, Some(self.addr.ip()))
                        .await
                    {
                        Ok(start) => {
                            self.screen_streams.insert(
                                source.clone(),
                                ScreenStreamGrant {
                                    token: start.token,
                                    pairing: None,
                                    shared_server_id: None,
                                    shared_token_hex: None,
                                },
                            );
                            Message::ScreenStreamReady {
                                source,
                                server_id: response_server_id.clone(),
                                endpoint: vterm_core::protocol::StreamEndpoint::Direct {
                                    host: String::new(),
                                    port: start.udp_port,
                                },
                                token: crate::screen::ctl::token_to_hex(&start.token),
                                codec: "h264".to_string(),
                                format: "nv12".to_string(),
                                error: None,
                            }
                        }
                        Err(e) => Message::ScreenStreamReady {
                            source,
                            server_id: response_server_id.clone(),
                            endpoint: vterm_core::protocol::StreamEndpoint::Direct {
                                host: String::new(),
                                port: 0,
                            },
                            token: String::new(),
                            codec: String::new(),
                            format: String::new(),
                            error: Some(format!("{e:#}")),
                        },
                    }
                };
                self.send_message(message).await?;
            }
            Message::StopScreenStream { source, server_id } => {
                let error = if let Some(grant) = self.screen_streams.remove(&source) {
                    // For relayed (shared server) streams: release the relay
                    // UDP pairing and tell the shared server to revoke.
                    if let (Some(pairing), Some(sid), Some(token_hex)) = (
                        &grant.pairing,
                        &grant.shared_server_id,
                        &grant.shared_token_hex,
                    ) {
                        let label = self.addr.to_string();
                        self.relay_manager
                            .untrack_upstream_udp_pairing(&label, pairing);
                        self.relay_manager
                            .release_upstream_udp_pairing(pairing.clone())
                            .await;
                        self.relay_manager.revoke_screen_token(sid, token_hex).await;
                        None
                    } else {
                        // Local stream: revoke at local screen-server.
                        match self.screen_supervisor.stop_stream(grant.token).await {
                            Ok(()) => None,
                            Err(e) => Some(format!("{e:#}")),
                        }
                    }
                } else {
                    None
                };
                self.send_message(Message::ScreenStreamStopped {
                    source,
                    server_id,
                    error,
                })
                .await?;
            }
            other => {
                warn!("Unhandled screen message: {}", other.msg_name());
            }
        }

        Ok(())
    }

    /// Lazily forward source snapshots, Android operation outcomes and stream
    /// stats to this client.
    pub(crate) async fn ensure_screen_event_forwarder(&mut self) {
        if self.screen_event_forwarder.is_some() {
            return;
        }
        let server_id = if self.via_relay {
            Some(self.config_manager.snapshot().await.server.server_id)
        } else {
            None
        };
        let mut events = self.screen_supervisor.subscribe_events();
        let supervisor = self.screen_supervisor.clone();
        let tx = self.tx.clone();
        self.screen_event_forwarder = Some(tokio::spawn(async move {
            loop {
                let message = match events.recv().await {
                    Ok(crate::screen::ctl::Event::Stats(stats)) => Some(Message::ScreenStreamStats {
                        frames: stats.frames,
                        avg_pickup_us: stats.avg_pickup_us,
                        avg_encode_ms: stats.avg_encode_ms,
                        delivered: stats.delivered,
                        dropped: stats.dropped,
                        avg_net_ms: stats.avg_net_ms,
                    }),
                    Ok(crate::screen::ctl::Event::SourcesChanged { revision, entries }) => {
                        info!(
                            ?server_id,
                            revision,
                            sources = entries.len(),
                            "screen ctl: broadcasting source snapshot"
                        );
                        Some(Message::ScreenSourcesChanged {
                            sources: entries
                                .iter()
                                .map(crate::screen::source_entry_to_proto)
                                .collect(),
                            server_id: server_id.clone(),
                            revision,
                        })
                    }
                    Ok(crate::screen::ctl::Event::AndroidOperationCompleted {
                        operation_id,
                        operation,
                        target,
                        serial,
                        error,
                    }) => {
                        info!(
                            ?server_id,
                            operation_id,
                            ?operation,
                            target,
                            ?serial,
                            ?error,
                            "android ctl: forwarding operation completion"
                        );
                        let operation_kind = match operation {
                            crate::screen::ctl::AndroidOperation::Boot => {
                                vterm_core::protocol::AndroidOperationKind::Boot
                            }
                            crate::screen::ctl::AndroidOperation::Shutdown => {
                                vterm_core::protocol::AndroidOperationKind::Shutdown
                            }
                        };
                        if let Some(error) = error {
                            Some(Message::AndroidEmulatorOperationFailed {
                                operation: operation_kind,
                                target,
                                server_id: server_id.clone(),
                                operation_id: Some(operation_id),
                                error,
                            })
                        } else {
                            match operation {
                                crate::screen::ctl::AndroidOperation::Boot => {
                                    serial.map(|serial| Message::AndroidEmulatorBooted {
                                        avd: target,
                                        serial,
                                        server_id: server_id.clone(),
                                        operation_id: Some(operation_id),
                                    })
                                }
                                crate::screen::ctl::AndroidOperation::Shutdown => {
                                    Some(Message::AndroidEmulatorShutdown {
                                        serial: target,
                                        server_id: server_id.clone(),
                                        operation_id: Some(operation_id),
                                    })
                                }
                            }
                        }
                    }
                    Ok(_) => None,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        match supervisor.list_source_snapshot().await {
                            Ok((revision, entries)) => Some(Message::ScreenSourcesChanged {
                                sources: entries
                                    .iter()
                                    .map(crate::screen::source_entry_to_proto)
                                    .collect(),
                                server_id: server_id.clone(),
                                revision,
                            }),
                            Err(_) => None,
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                };
                if let Some(message) = message {
                    if tx.send(message).is_err() {
                        break;
                    }
                }
            }
        }));
    }
}
