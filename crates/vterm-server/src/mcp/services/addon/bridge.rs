//! WebSocket bridge between MCP addons and their out-of-process daemons
//! (e.g. the EasyEDA connector extension running inside the EDA sandbox,
//! which cannot listen on a socket and therefore dials out to us).
//!
//! One active connection per addon id. JS running in the QuickJS executor
//! calls `addon.bridge_request(payload)` → [`AddonBridgeHub::request`] → a
//! `{type:"request", id:"req-N", …}` frame to the daemon; the daemon's
//! `{type:"response", id, ok, result|error}` frame completes the parked
//! oneshot. `{type:"ping"}` frames get a protocol-level `{type:"pong"}`
//! reply — the EDA sandbox WebSocket has no close event, so daemons use the
//! pong to detect a dead connection.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, warn};

/// Process-wide hub. The WS handler is mounted once by `AddonMcpService`
/// and executors reach it through here, so no state has to travel through
/// axum extractors.
pub fn global_hub() -> &'static AddonBridgeHub {
    static HUB: OnceLock<AddonBridgeHub> = OnceLock::new();
    HUB.get_or_init(AddonBridgeHub::default)
}

struct PendingEntry {
    addon_id: String,
    sender: oneshot::Sender<Value>,
}

#[derive(Default)]
pub struct AddonBridgeHub {
    /// addon id → frame sender of the currently connected daemon. A new
    /// connection replaces the old one (the dropped receiver closes the old
    /// writer task).
    connections: Mutex<HashMap<String, mpsc::UnboundedSender<String>>>,
    /// request id → parked caller.
    pending: Mutex<HashMap<String, PendingEntry>>,
    next_id: AtomicU64,
}

impl AddonBridgeHub {
    /// Bind a daemon connection to an addon id.
    pub fn register(&self, addon_id: &str, tx: mpsc::UnboundedSender<String>) {
        self.connections().insert(addon_id.to_string(), tx);
        info!("addon bridge: daemon connected for '{addon_id}'");
    }

    /// Whether a daemon is currently connected for `addon_id`.
    pub fn is_connected(&self, addon_id: &str) -> bool {
        self.connections().contains_key(addon_id)
    }

    /// Drop the connection for `addon_id` and fail every request still
    /// waiting on it.
    pub fn unregister(&self, addon_id: &str) {
        self.connections().remove(addon_id);
        let mut pending = self.pending.lock().expect("bridge pending lock poisoned");
        let orphaned: Vec<String> = pending
            .iter()
            .filter(|(_, entry)| entry.addon_id == addon_id)
            .map(|(id, _)| id.clone())
            .collect();
        for id in orphaned {
            if let Some(entry) = pending.remove(&id) {
                let _ = entry.sender.send(Value::String(format!(
                    "daemon disconnected for addon '{addon_id}'"
                )));
            }
        }
        info!("addon bridge: daemon disconnected for '{addon_id}'");
    }

    /// Send a request frame to the daemon of `addon_id` and await its
    /// response. `payload` must be a JSON object (e.g. `{"action":"invoke",…}`)
    /// produced by the addon's JS; the hub merges `type`/`id` into it.
    /// Returns the response's `result` value, or an error string (no daemon,
    /// malformed payload, daemon-reported error, timeout).
    pub async fn request(
        &self,
        addon_id: &str,
        payload: &str,
        timeout: Duration,
    ) -> Result<Value, String> {
        let mut frame: Value =
            serde_json::from_str(payload).map_err(|e| format!("invalid bridge payload: {e}"))?;
        let Some(obj) = frame.as_object_mut() else {
            return Err("bridge payload must be a JSON object".to_string());
        };

        let id = format!("req-{}", self.next_id.fetch_add(1, Ordering::Relaxed));
        obj.insert("type".to_string(), Value::String("request".to_string()));
        obj.insert("id".to_string(), Value::String(id.clone()));

        let tx = self
            .connections()
            .get(addon_id)
            .cloned()
            .ok_or_else(|| format!("no daemon connected for addon '{addon_id}'"))?;

        let (resp_tx, resp_rx) = oneshot::channel();
        self.pending
            .lock()
            .expect("bridge pending lock poisoned")
            .insert(
                id.clone(),
                PendingEntry {
                    addon_id: addon_id.to_string(),
                    sender: resp_tx,
                },
            );

        debug!(
            "addon bridge: request {id} action={:?} to '{addon_id}'",
            obj.get("action")
        );
        if tx.send(frame.to_string()).is_err() {
            self.pending
                .lock()
                .expect("bridge pending lock poisoned")
                .remove(&id);
            return Err(format!("daemon connection closed for addon '{addon_id}'"));
        }

        let response = match tokio::time::timeout(timeout, resp_rx).await {
            Ok(Ok(value)) => value,
            Ok(Err(_)) => {
                return Err(format!("daemon disconnected for addon '{addon_id}'"));
            }
            Err(_) => {
                self.pending
                    .lock()
                    .expect("bridge pending lock poisoned")
                    .remove(&id);
                return Err(format!(
                    "daemon request timed out after {}s for addon '{addon_id}'",
                    timeout.as_secs()
                ));
            }
        };

        // `response` is the full response frame for WS replies, or a bare
        // error string planted by `unregister`.
        if response.is_string() {
            return Err(response.as_str().unwrap_or_default().to_string());
        }
        match response.get("ok").and_then(Value::as_bool) {
            Some(true) => Ok(response.get("result").cloned().unwrap_or(Value::Null)),
            _ => {
                let message = response
                    .get("error")
                    .and_then(|e| {
                        e.get("message")
                            .and_then(Value::as_str)
                            .or_else(|| e.as_str())
                    })
                    .unwrap_or("daemon request failed");
                Err(message.to_string())
            }
        }
    }

    /// Complete a pending request from an inbound response frame. Ignores
    /// frames with unknown ids (late responses after a timeout).
    pub fn handle_response(&self, frame: &Value) {
        let Some(id) = frame.get("id").and_then(Value::as_str) else {
            return;
        };
        let entry = self
            .pending
            .lock()
            .expect("bridge pending lock poisoned")
            .remove(id);
        if let Some(entry) = entry {
            let _ = entry.sender.send(frame.clone());
        }
    }

    fn connections(
        &self,
    ) -> std::sync::MutexGuard<'_, HashMap<String, mpsc::UnboundedSender<String>>> {
        self.connections
            .lock()
            .expect("bridge connections lock poisoned")
    }
}

/// `GET /addon/bridge` — WebSocket upgrade. Mounted inside
/// `AddonMcpService::build_route`, so it shares the host's bearer-auth layer
/// (loopback exempt).
pub async fn addon_bridge_ws(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // First frame must be the hello binding this connection to an addon id;
    // the server replies with a welcome frame (peer identity proof).
    let addon_id = match socket.recv().await {
        Some(Ok(Message::Text(text))) => match serde_json::from_str::<Value>(&text) {
            Ok(frame)
                if frame.get("type").and_then(Value::as_str) == Some("hello")
                    && frame.get("addon").and_then(Value::as_str).is_some() =>
            {
                frame["addon"].as_str().unwrap_or_default().to_string()
            }
            _ => {
                warn!("addon bridge: first frame was not a valid hello, closing");
                let _ = socket.close().await;
                return;
            }
        },
        _ => {
            let _ = socket.close().await;
            return;
        }
    };

    let hub = global_hub();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();
    // Keep a second sender for protocol-level replies (pong) that bypass the hub.
    let pong_tx = out_tx.clone();
    hub.register(&addon_id, out_tx);

    // Answer the hello: the extension only treats the connection as live
    // after this frame, which proves the peer is vterm-server and not some
    // other service that happens to sit on the port.
    let welcome = serde_json::json!({
        "type": "welcome",
        "service": "vterm-addon-bridge",
        "addon": addon_id,
        "server": concat!("vterm-server ", env!("CARGO_PKG_VERSION")),
    });
    let _ = pong_tx.send(welcome.to_string());

    // Writer task: hub → socket.
    let (mut sink, mut stream) = socket.split();
    let writer = tokio::spawn(async move {
        while let Some(frame) = out_rx.recv().await {
            if sink.send(Message::Text(frame.into())).await.is_err() {
                break;
            }
        }
    });

    // Reader loop: socket → hub.
    while let Some(message) = stream.next().await {
        match message {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<Value>(&text) {
                    Ok(frame) if frame.get("type").and_then(Value::as_str) == Some("response") => {
                        hub.handle_response(&frame);
                    }
                    // Protocol-level keepalive: the EDA sandbox's sys_WebSocket
                    // has no close event, so daemons detect a dead connection by
                    // pinging and timing out the pong.
                    Ok(frame) if frame.get("type").and_then(Value::as_str) == Some("ping") => {
                        let pong = serde_json::json!({
                            "type": "pong",
                            "id": frame.get("id").cloned().unwrap_or(Value::Null),
                        });
                        let _ = pong_tx.send(pong.to_string());
                    }
                    Ok(frame) => {
                        debug!(
                            "addon bridge: ignoring non-response frame from '{addon_id}': {frame}"
                        );
                    }
                    Err(e) => {
                        warn!("addon bridge: invalid JSON from '{addon_id}': {e}");
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    hub.unregister(&addon_id);
    writer.abort();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_round_trip() {
        let hub = std::sync::Arc::new(AddonBridgeHub::default());
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        hub.register("easyeda", tx);

        let hub_for_req = std::sync::Arc::clone(&hub);
        let request = tokio::spawn(async move {
            hub_for_req
                .request("easyeda", r#"{"action":"ping"}"#, Duration::from_secs(5))
                .await
        });

        let frame: Value = serde_json::from_str(&rx.recv().await.unwrap()).unwrap();
        let id = frame["id"].as_str().unwrap();
        hub.handle_response(&serde_json::json!({
            "type": "response",
            "id": id,
            "ok": true,
            "result": {"pong": true}
        }));

        let result = request.await.unwrap().expect("request ok");
        assert_eq!(result["pong"], true);
    }

    #[tokio::test]
    async fn request_without_daemon_fails_immediately() {
        let hub = AddonBridgeHub::default();
        let err = hub
            .request("ghost", r#"{"action":"ping"}"#, Duration::from_secs(1))
            .await
            .unwrap_err();
        assert!(err.contains("no daemon connected"), "got: {err}");
    }

    #[tokio::test]
    async fn unregister_fails_pending_requests() {
        let hub = std::sync::Arc::new(AddonBridgeHub::default());
        let (tx, _rx) = mpsc::unbounded_channel::<String>();
        hub.register("easyeda", tx);

        let hub_for_req = std::sync::Arc::clone(&hub);
        let request = tokio::spawn(async move {
            hub_for_req
                .request("easyeda", r#"{"action":"ping"}"#, Duration::from_secs(30))
                .await
        });
        // Let the request park, then drop the connection.
        tokio::time::sleep(Duration::from_millis(50)).await;
        hub.unregister("easyeda");

        let err = request.await.unwrap().unwrap_err();
        assert!(err.contains("daemon disconnected"), "got: {err}");
    }

    #[tokio::test]
    async fn daemon_error_response_maps_to_err() {
        let hub = std::sync::Arc::new(AddonBridgeHub::default());
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        hub.register("easyeda", tx);

        let hub_for_req = std::sync::Arc::clone(&hub);
        let request = tokio::spawn(async move {
            hub_for_req
                .request(
                    "easyeda",
                    r#"{"action":"invoke","path":"nope"}"#,
                    Duration::from_secs(5),
                )
                .await
        });
        let frame: Value = serde_json::from_str(&rx.recv().await.unwrap()).unwrap();
        hub.handle_response(&serde_json::json!({
            "type": "response",
            "id": frame["id"],
            "ok": false,
            "error": {"code": "no_such_path", "message": "eda.nope is undefined"}
        }));
        let err = request.await.unwrap().unwrap_err();
        assert_eq!(err, "eda.nope is undefined");
    }

    #[tokio::test]
    async fn malformed_payload_is_rejected() {
        let hub = AddonBridgeHub::default();
        assert!(hub
            .request("x", "not json", Duration::from_secs(1))
            .await
            .is_err());
        assert!(hub
            .request("x", "[1,2]", Duration::from_secs(1))
            .await
            .is_err());
    }
}
