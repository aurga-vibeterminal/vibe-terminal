//! WebSocket live-transcription endpoint (`/api/voice/transcribe/live`).
//!
//! A WS-to-WS bridge to the supervised asr-server's
//! `/v1/audio/transcriptions/live`: the browser's binary audio frames (f32 LE
//! PCM, 16 kHz mono) are forwarded upstream verbatim and the asr-server's
//! partial/final/error JSON text frames flow back. The two sides speak the
//! exact same protocol, so the bridge is a pure relay — no translation.
//!
//! The asr-server's auth middleware accepts the bearer token on WS upgrades
//! via the `Authorization` header, which tokio-tungstenite (unlike a browser)
//! can attach. Its TLS identity is an embedded self-signed certificate, so
//! the upstream connector accepts any server certificate — mirroring the
//! supervisor's reqwest client (`danger_accept_invalid_certs(true)`).
//!
//! Failures surface to the browser as one JSON `error` frame followed by a
//! close, the same shape the asr-server itself uses (e.g. "Model not
//! loaded"), so the webui's live tab displays them uniformly.

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use sha1::{Digest, Sha1};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::Role;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{
    connect_async_tls_with_config, Connector, MaybeTlsStream, WebSocketStream,
};
use tracing::warn;

use super::super::server::HttpRequest;
use crate::voice::AsrSupervisor;

const WS_MAGIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Handle a WebSocket upgrade request, then relay the session to the
/// asr-server's live endpoint.
pub async fn handle_upgrade(
    stream: TcpStream,
    request: HttpRequest,
    voice_supervisor: Arc<AsrSupervisor>,
) {
    let key = match request.headers.get("sec-websocket-key") {
        Some(k) => k.clone(),
        None => {
            warn!("WebSocket upgrade missing Sec-WebSocket-Key");
            return;
        }
    };

    let accept = compute_accept_key(&key);

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\
         \r\n",
        accept
    );

    let mut stream = stream;
    if let Err(e) = stream.write_all(response.as_bytes()).await {
        warn!("Failed to write WebSocket upgrade response: {}", e);
        return;
    }
    if let Err(e) = stream.flush().await {
        warn!("Failed to flush WebSocket upgrade response: {}", e);
        return;
    }

    // Only pass bytes after the HTTP headers (request.body). The browser may
    // have already queued audio frames in the same TCP segment.
    let ws = WebSocketStream::from_partially_read(stream, request.body, Role::Server, None).await;

    bridge_session(ws, voice_supervisor).await;
}

/// Relay one browser live-transcription session to the asr-server. Exits when
/// either side closes or errors; dropping the streams tears down the sockets.
async fn bridge_session(browser: WebSocketStream<TcpStream>, voice_supervisor: Arc<AsrSupervisor>) {
    let mut upstream = match connect_upstream(&voice_supervisor).await {
        Ok(ws) => ws,
        Err(error) => {
            warn!("Live transcription bridge could not reach the asr-server: {error}");
            send_error_and_close(
                browser,
                &format!("Live transcription is unavailable: {error}"),
            )
            .await;
            return;
        }
    };

    let (mut browser_tx, mut browser_rx) = browser.split();
    let (mut upstream_tx, mut upstream_rx) = upstream.split();

    loop {
        tokio::select! {
            from_browser = browser_rx.next() => {
                match from_browser {
                    // Audio frames (and the app-level "ping" keepalive) are
                    // forwarded upstream untouched.
                    Some(Ok(Message::Binary(data))) => {
                        if upstream_tx.send(Message::Binary(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Text(text))) => {
                        if upstream_tx.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        let _ = upstream_tx.send(Message::Close(None)).await;
                        break;
                    }
                    // Ping/Pong are answered by the protocol layer itself.
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        warn!("Browser live-transcription stream error: {e}");
                        let _ = upstream_tx.send(Message::Close(None)).await;
                        break;
                    }
                }
            }
            from_upstream = upstream_rx.next() => {
                match from_upstream {
                    // partial/final/error JSON frames pass through verbatim.
                    Some(Ok(Message::Text(json))) => {
                        if browser_tx.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        let _ = browser_tx.send(Message::Close(None)).await;
                        break;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        warn!("asr-server live-transcription stream error: {e}");
                        let error = serde_json::json!({
                            "type": "error",
                            "message": format!("ASR server stream error: {e}"),
                        });
                        let _ = browser_tx.send(Message::Text(error.to_string().into())).await;
                        let _ = browser_tx.send(Message::Close(None)).await;
                        break;
                    }
                }
            }
        }
    }
}

/// Dial the supervised asr-server's live endpoint. The supervisor's
/// `url_for` yields an `https://` base (loopback self-signed identity), so
/// the URL is mapped to `wss://` and the connector accepts any certificate —
/// the same trust posture as the supervisor's own reqwest client.
async fn connect_upstream(
    voice_supervisor: &Arc<AsrSupervisor>,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, String> {
    let base = voice_supervisor
        .url_for("/v1/audio/transcriptions/live")
        .await;
    let ws_url = if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        base
    };

    let mut request = ws_url
        .as_str()
        .into_client_request()
        .map_err(|e| e.to_string())?;
    if let Some(token) = voice_supervisor.bearer_token().await {
        let value = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|e| format!("invalid bearer token header: {e}"))?;
        request.headers_mut().insert(AUTHORIZATION, value);
    }

    let connector = vterm_common::tls::insecure_client_tls_config()
        .ok()
        .map(Connector::Rustls);
    let (ws, _response) = connect_async_tls_with_config(request, None, true, connector)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ws)
}

/// Tell the browser why the session cannot start, then close cleanly. Drains
/// incoming frames until the client notices the close so teardown is orderly.
async fn send_error_and_close(mut ws: WebSocketStream<TcpStream>, message: &str) {
    let msg = serde_json::json!({
        "type": "error",
        "message": message,
    });
    let _ = ws.send(Message::Text(msg.to_string().into())).await;
    let _ = ws.close(None).await;

    // Drain whatever the client sends before noticing the close.
    while let Some(msg) = ws.next().await {
        if matches!(msg, Ok(Message::Close(_)) | Err(_)) {
            break;
        }
    }
}

fn compute_accept_key(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(WS_MAGIC.as_bytes());
    let result = hasher.finalize();
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result)
}
