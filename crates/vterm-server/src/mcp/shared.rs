//! Cross-service helpers shared by all MCP services.

use std::sync::Arc;

use rmcp::model::ErrorData;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use vterm_common::SessionId;

/// Build the `LocalSessionManager` every service serves with, rmcp's default
/// 5-minute idle timeout disabled: sessions live until the client explicitly
/// disconnects (HTTP DELETE) or the server restarts. Interactive code agents
/// routinely idle far longer than 5 minutes between tool calls — the default
/// killed their sessions mid-conversation, surfacing in clients as "Maximum
/// reconnection attempts exceeded" once the SSE stream dropped. Sessions
/// orphaned by crashed clients are a few small worker tasks each, acceptable
/// for a local single-user server.
pub fn local_session_manager() -> Arc<LocalSessionManager> {
    let mut mgr = LocalSessionManager::default();
    mgr.session_config.keep_alive = None;
    Arc::new(mgr)
}

/// Parse a string terminal/device id into a [`SessionId`].
///
/// Used by tools that accept a UUID id. Errors map to MCP `invalid_params`.
pub fn parse_session_id(s: &str) -> Result<SessionId, ErrorData> {
    uuid::Uuid::parse_str(s)
        .map(SessionId)
        .map_err(|_| ErrorData::invalid_params(format!("invalid terminal_id: {s}"), None))
}

/// Cheap timestamp string (`unix:<secs>`) for tool/resource payloads.
///
/// Intentionally not ISO-8601 to avoid pulling a datetime crate; the format is
/// only meant for coarse ordering and human scan.
pub fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}
