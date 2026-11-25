use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{
    connect_async_tls_with_config, Connector, MaybeTlsStream, WebSocketStream,
};
use vterm_common::{
    secret_key_challenge_response, tls, SessionId, TerminalProfile, TerminalSize, MAX_MESSAGE_SIZE,
};
use vterm_core::crypto::{
    is_encrypted_message_frame, MessageCrypto, MessageCryptoHandshake, TRANSPORT_TAG_BINARY,
    TRANSPORT_TAG_TEXT,
};
use vterm_core::protocol::{ConnectionKind, Message, TransportBinaryFrameDecoder};

/// WebSocket client for vterm-server
pub struct VTermClient {
    server_url: String,
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    /// The primary connection, kept open while a Terminal connection is in
    /// use. The server revokes child connections when their primary closes
    /// and reaps silent primaries after a 60s heartbeat timeout, so the
    /// keep-alive task continuously reads the primary (which also answers
    /// the server's WebSocket pings) for as long as we stream.
    primary_keepalive: Option<tokio::task::JoinHandle<()>>,
    transport_binary_decoder: TransportBinaryFrameDecoder,
    /// Auth token issued by the server on the primary connection.
    auth_token: Option<String>,
    /// Secret key used to answer the handshake challenge. When empty, the
    /// `TERMINAL_GUI_SECRET_KEY` environment variable is used.
    secret_key: Option<String>,
    /// Noise message encryption negotiated during the handshake. Required
    /// for attaching to shared-server (relayed) terminals, whose transparent
    /// proxy refuses plaintext connections.
    message_crypto: Option<MessageCrypto>,
}

impl VTermClient {
    /// Connect to vterm-server
    pub async fn connect(url: &str) -> Result<Self> {
        let url = normalize_websocket_url(url);
        let connector = tls_connector_for_url(&url)?;

        let (websocket, _) = connect_async_tls_with_config(&url, None, false, connector)
            .await
            .context(format!("Failed to connect to {}", url))?;

        Ok(Self {
            server_url: url,
            websocket,
            primary_keepalive: None,
            transport_binary_decoder: TransportBinaryFrameDecoder::default(),
            auth_token: None,
            secret_key: None,
            message_crypto: None,
        })
    }

    /// Set the secret key used to answer the server's handshake challenge.
    pub fn set_secret_key(&mut self, secret_key: Option<String>) {
        self.secret_key = secret_key;
    }

    /// Perform handshake with server
    pub async fn handshake(&mut self) -> Result<()> {
        self.handshake_as(ConnectionKind::Primary, None, None).await
    }

    /// Open a dedicated Terminal connection for streaming `session_id` and
    /// swap this client's transport onto it. Terminal messages (StreamStart,
    /// Input, Resize, ...) are rejected on Primary connections by the
    /// server's role-based routing, so terminal I/O must use a Terminal-kind
    /// connection authenticated with the primary connection's auth token.
    pub async fn upgrade_to_terminal(&mut self, session_id: SessionId) -> Result<()> {
        let auth_token = self.auth_token.clone().ok_or_else(|| {
            anyhow::anyhow!("Server did not issue an auth token; cannot open a terminal connection")
        })?;

        let mut terminal = VTermClient::connect(&self.server_url).await?;
        terminal.secret_key = self.secret_key.clone();
        terminal
            .handshake_as(ConnectionKind::Terminal, Some(auth_token), Some(session_id))
            .await?;

        // Swap the transport over to the terminal connection. The primary
        // must stay open (the server tears down Terminal child connections
        // as soon as their primary closes) and responsive (it reaps silent
        // primaries after its 60s heartbeat timeout), so keep reading it in
        // the background; polling it also flushes the automatic Pong frames.
        let mut primary_websocket = std::mem::replace(&mut self.websocket, terminal.websocket);
        self.message_crypto = terminal.message_crypto;
        self.primary_keepalive = Some(tokio::spawn(async move {
            use futures_util::StreamExt;
            // Server-to-client messages on the primary (ServerInfo updates,
            // relay notices, ...) are irrelevant to vtermc; discard them.
            while primary_websocket.next().await.is_some() {}
        }));
        self.transport_binary_decoder = TransportBinaryFrameDecoder::default();
        tracing::debug!("Upgraded to Terminal connection for session {}", session_id);
        Ok(())
    }

    /// Perform a handshake declaring a specific connection kind.
    async fn handshake_as(
        &mut self,
        connection_kind: ConnectionKind,
        auth_token: Option<String>,
        target_session_id: Option<SessionId>,
    ) -> Result<()> {
        let hello = Message::Hello {
            protocol_version: vterm_common::PROTOCOL_VERSION,
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            secret_key_hash: None,
            supports_binary_output: true,
            supports_message_encryption: true,
            connection_kind: Some(connection_kind),
            auth_token,
            target_session_id,
            relay_terminal_capability: None,
            target_server_id: None,
            relay_request_capability: None,
        };

        self.send_message(hello).await?;

        let negotiate_encryption;
        match self.receive_message().await {
            Some(Message::AuthChallenge {
                nonce,
                message_encryption,
            }) => {
                negotiate_encryption = message_encryption;
                if let Some(nonce) = nonce {
                    let proof = self
                        .secret_key
                        .clone()
                        .map(|secret_key| secret_key.trim().to_string())
                        .filter(|secret_key| !secret_key.is_empty())
                        .or_else(|| {
                            std::env::var("TERMINAL_GUI_SECRET_KEY")
                                .ok()
                                .map(|secret_key| secret_key.trim().to_string())
                                .filter(|secret_key| !secret_key.is_empty())
                        })
                        .map(|secret_key| secret_key_challenge_response(&secret_key, &nonce));
                    self.send_message(Message::AuthResponse { proof }).await?;
                }
            }
            Some(Message::Error { code, message }) => {
                return Err(anyhow::anyhow!("Handshake failed: {} - {}", code, message));
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Server closed the connection during handshake \
                     (see the ERROR log above for the transport cause; attaching to \
                     shared-server terminals requires message encryption and a \
                     reachable shared server)"
                ));
            }
        }

        // The server reads the optional AuthResponse first, then immediately
        // runs the Noise responder handshake (XX pattern), then sends the
        // HelloAck encrypted. Shared-server (relayed) terminals require this:
        // their transparent proxy refuses plaintext connections.
        if negotiate_encryption {
            let mut handshake = MessageCryptoHandshake::new_initiator()?;
            self.websocket
                .send(WsMessage::Binary(handshake.write_frame()?.into()))
                .await
                .context("Failed to send encryption handshake")?;
            match self.websocket.next().await {
                Some(Ok(WsMessage::Binary(data))) => handshake.read_frame(&data)?,
                Some(Ok(WsMessage::Close(_))) => {
                    return Err(anyhow::anyhow!("Server closed during encryption handshake"));
                }
                Some(Ok(other)) => {
                    return Err(anyhow::anyhow!(
                        "Unexpected encryption handshake frame: {:?}",
                        other
                    ));
                }
                Some(Err(e)) => {
                    return Err(anyhow::anyhow!(
                        "Failed to read encryption handshake: {}",
                        e
                    ));
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "Connection closed during encryption handshake"
                    ));
                }
            }
            self.websocket
                .send(WsMessage::Binary(handshake.write_frame()?.into()))
                .await
                .context("Failed to finish encryption handshake")?;
            self.message_crypto = Some(handshake.into_transport()?);
            tracing::debug!("Message encryption negotiated");
        }

        match self.receive_message().await {
            Some(Message::HelloAck {
                auth_token: ack_token,
                ..
            }) => {
                if self.auth_token.is_none() {
                    self.auth_token = ack_token;
                }
                tracing::debug!(
                    "Handshake successful, auth_token: {:?}",
                    self.auth_token.is_some()
                );
                Ok(())
            }
            Some(Message::Error { code, message }) => {
                Err(anyhow::anyhow!("Handshake failed: {} - {}", code, message))
            }
            _ => Err(anyhow::anyhow!(
                "Server closed the connection before completing the handshake"
            )),
        }
    }

    /// Wait for the session list the server pushes to a fresh primary
    /// connection as part of its initial snapshots, skipping the other
    /// snapshot messages.
    pub async fn wait_for_sessions(&mut self) -> Result<Vec<vterm_common::SessionInfo>> {
        loop {
            match self.receive_message().await {
                Some(Message::SessionList { sessions }) => return Ok(sessions),
                Some(Message::Error { code, message }) => {
                    return Err(anyhow::anyhow!(
                        "Failed to list sessions: {} - {}",
                        code,
                        message
                    ))
                }
                Some(other) => {
                    tracing::debug!(
                        "Skipping unrelated message while waiting for SessionList: {:?}",
                        std::mem::discriminant(&other)
                    );
                    continue;
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "No response when listing sessions (connection closed)"
                    ))
                }
            }
        }
    }

    /// Fetch terminal profiles from server
    pub async fn list_profiles(&mut self) -> Result<Vec<TerminalProfile>> {
        self.send_message(Message::ListTerminalProfiles).await?;

        loop {
            match self.receive_message().await {
                Some(Message::TerminalProfileList { profiles }) => return Ok(profiles),
                Some(Message::Error { code, message }) => {
                    return Err(anyhow::anyhow!(
                        "Failed to list profiles: {} - {}",
                        code,
                        message
                    ))
                }
                Some(other) => {
                    tracing::debug!(
                        "Skipping unrelated message while waiting for TerminalProfileList: {:?}",
                        std::mem::discriminant(&other)
                    );
                    continue;
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "No response when listing profiles (connection closed)"
                    ))
                }
            }
        }
    }

    /// Create a new session
    pub async fn create_session(
        &mut self,
        profile_id: Option<String>,
        server_id: Option<String>,
        size: TerminalSize,
        env: HashMap<String, String>,
        cwd: String,
        sibling_session_id: Option<SessionId>,
    ) -> Result<SessionId> {
        let msg = Message::CreateSession {
            shell: String::new(),
            profile_id,
            size,
            env,
            cwd: Some(cwd),
            title: None,
            sibling_session_id,
            server_id,
            duplicate_session_id: None,
            tab_group_id: None,
            app: None,
            background: None,
            foreground: None,
        };

        self.send_message(msg).await?;

        // Wait for SessionCreated or Error, skipping unrelated messages
        loop {
            match self.receive_message().await {
                Some(Message::SessionCreated { session_id }) => return Ok(session_id),
                Some(Message::Error { code, message }) => {
                    return Err(anyhow::anyhow!(
                        "Failed to create session: {} - {}",
                        code,
                        message
                    ))
                }
                Some(other) => {
                    // Log and skip unrelated messages (e.g., RelayServerList)
                    tracing::debug!(
                        "Skipping unrelated message while waiting for SessionCreated: {:?}",
                        std::mem::discriminant(&other)
                    );
                    continue;
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "No response when creating session (connection closed)"
                    ))
                }
            }
        }
    }

    /// Start streaming output for a session
    pub async fn start_stream(&mut self, session_id: SessionId) -> Result<()> {
        self.send_message(Message::StreamStart {
            session_id,
            initial_size: None,
            background: None,
            foreground: None,
        })
        .await?;
        Ok(())
    }

    /// Send input to a session
    pub async fn send_input(&mut self, session_id: SessionId, data: Vec<u8>) -> Result<()> {
        let msg = Message::Input {
            session_id,
            data,
            cursor_from_end: None,
        };
        self.send_message(msg).await
    }

    /// Send resize to a session
    pub async fn resize(&mut self, session_id: SessionId, size: TerminalSize) -> Result<()> {
        let msg = Message::Resize { session_id, size };
        self.send_message(msg).await
    }

    /// Send a message to the server
    async fn send_message(&mut self, message: Message) -> Result<()> {
        let websocket_messages = if let Some(crypto) = self.message_crypto.as_mut() {
            // Encrypted connections wrap every payload (binary transport
            // frame or JSON text) in a Noise frame tagged with its
            // transport type.
            let mut frames = Vec::new();
            let payloads: Vec<Vec<u8>> = message.to_transport_binary_frames().unwrap_or_default();
            if payloads.is_empty() {
                let mut plaintext = Vec::with_capacity(1 + 64);
                plaintext.push(TRANSPORT_TAG_TEXT);
                plaintext.extend_from_slice(message.to_text()?.as_bytes());
                encrypt_into_frames(crypto, &plaintext, &mut frames)?;
            } else {
                for payload in payloads {
                    let mut plaintext = Vec::with_capacity(1 + payload.len());
                    plaintext.push(TRANSPORT_TAG_BINARY);
                    plaintext.extend_from_slice(&payload);
                    encrypt_into_frames(crypto, &plaintext, &mut frames)?;
                }
            }
            frames
        } else if let Some(payloads) = message.to_transport_binary_frames() {
            let mut frames = Vec::with_capacity(payloads.len());
            for payload in payloads {
                if payload.len() > MAX_MESSAGE_SIZE {
                    anyhow::bail!("Message too large: {} bytes", payload.len());
                }
                frames.push(WsMessage::Binary(payload.into()));
            }
            frames
        } else {
            let json = serde_json::to_string(&message).context("Failed to serialize message")?;
            if json.len() > MAX_MESSAGE_SIZE {
                anyhow::bail!("Message too large: {} bytes", json.len());
            }
            vec![WsMessage::Text(json.into())]
        };

        for websocket_message in websocket_messages {
            self.websocket
                .send(websocket_message)
                .await
                .context("Failed to send message")?;
        }

        Ok(())
    }

    /// Receive a message from the server
    pub async fn receive_message(&mut self) -> Option<Message> {
        loop {
            match self.websocket.next().await {
                Some(Ok(WsMessage::Text(text))) => match serde_json::from_str::<Message>(&text) {
                    Ok(msg) => return Some(msg),
                    Err(e) => {
                        tracing::error!("Failed to parse message: {} - {}", e, text);
                        continue;
                    }
                },
                Some(Ok(WsMessage::Binary(data))) => {
                    // Noise-encrypted frames carry a magic prefix and a
                    // transport tag; plaintext binary frames are plain
                    // transport frames.
                    let decoded =
                        if self.message_crypto.is_some() && is_encrypted_message_frame(&data) {
                            self.decrypt_frame(&data)
                        } else {
                            self.transport_binary_decoder.decode_message_frame(&data)
                        };
                    match decoded {
                        Ok(Some(msg)) => return Some(msg),
                        Ok(None) => continue,
                        Err(e) => {
                            tracing::warn!("Failed to parse binary frame: {}", e);
                            continue;
                        }
                    }
                }
                Some(Ok(WsMessage::Close(_))) => {
                    tracing::debug!("WebSocket closed");
                    return None;
                }
                Some(Ok(_)) => continue, // Ping/Pong
                Some(Err(e)) => {
                    tracing::error!("WebSocket error: {}", e);
                    return None;
                }
                None => {
                    tracing::debug!("WebSocket stream ended");
                    return None;
                }
            }
        }
    }

    /// Decrypt one Noise frame into a protocol message.
    fn decrypt_frame(&mut self, data: &[u8]) -> anyhow::Result<Option<Message>> {
        let Some(crypto) = self.message_crypto.as_mut() else {
            anyhow::bail!("Received encrypted frame before encryption was negotiated");
        };
        // `None` = a fragment of a larger message; more chunks to come.
        let Some(plaintext) = crypto.decrypt_frame(data)? else {
            return Ok(None);
        };
        let Some((&tag, payload)) = plaintext.split_first() else {
            anyhow::bail!("Encrypted frame payload is empty");
        };
        match tag {
            TRANSPORT_TAG_TEXT => Message::from_text(std::str::from_utf8(payload)?).map(Some),
            TRANSPORT_TAG_BINARY => self.transport_binary_decoder.decode_message_frame(payload),
            other => anyhow::bail!("Unknown encrypted transport tag {}", other),
        }
    }

    /// Close the connection gracefully
    pub async fn close(&mut self) -> Result<()> {
        if let Some(keepalive) = self.primary_keepalive.take() {
            keepalive.abort();
        }
        self.websocket.close(None).await.ok();
        Ok(())
    }
}

/// Normalize URL to WebSocket format
fn normalize_websocket_url(url: &str) -> String {
    if url.starts_with("ws://") || url.starts_with("wss://") {
        url.to_string()
    } else if let Some(rest) = url.strip_prefix("ws:/") {
        format!("ws://{}", rest.trim_start_matches('/'))
    } else if let Some(rest) = url.strip_prefix("wss:/") {
        format!("wss://{}", rest.trim_start_matches('/'))
    } else if url.starts_with("http://") {
        url.replacen("http://", "ws://", 1)
    } else if url.starts_with("https://") {
        url.replacen("https://", "wss://", 1)
    } else if let Some(rest) = url.strip_prefix("http:/") {
        format!("ws://{}", rest.trim_start_matches('/'))
    } else if let Some(rest) = url.strip_prefix("https:/") {
        format!("wss://{}", rest.trim_start_matches('/'))
    } else {
        // Assume wss:// for bare addresses
        format!("wss://{}", url)
    }
}

/// Build TLS connector for WebSocket URL (rustls-based, see vterm_common::tls)
fn tls_connector_for_url(websocket_url: &str) -> anyhow::Result<Option<Connector>> {
    if !websocket_url.starts_with("wss://") {
        return Ok(None);
    }

    let config = tls::build_rustls_client_config(websocket_url)
        .map_err(|e| anyhow::anyhow!("Failed to build TLS connector: {}", e))?;
    Ok(Some(Connector::Rustls(config)))
}

/// Encrypt one tagged plaintext into Noise frame(s) (a large message is
/// fragmented by the crypto layer into several WebSocket frames).
fn encrypt_into_frames(
    crypto: &mut MessageCrypto,
    plaintext: &[u8],
    frames: &mut Vec<WsMessage>,
) -> anyhow::Result<()> {
    for encrypted in crypto.encrypt_frame(plaintext)? {
        if encrypted.len() > MAX_MESSAGE_SIZE {
            anyhow::bail!("Message too large: {} bytes", encrypted.len());
        }
        frames.push(WsMessage::Binary(encrypted.into()));
    }
    Ok(())
}
