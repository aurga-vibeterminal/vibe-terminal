//! The terminal MCP service — exposes terminal sessions to Code Agents.
//! Implements [`crate::mcp::registry::McpService`].

use std::sync::Arc;

use axum::Router;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};

use crate::mcp::registry::{McpContext, McpService};
use crate::session::SessionManager as VtSessionManager;

pub mod handler;
pub mod resources;
pub mod screen;
pub mod tools;

pub struct TerminalService;

impl McpService for TerminalService {
    fn path(&self) -> &'static str {
        "terminal"
    }
    fn name(&self) -> &'static str {
        "vterm-terminal-mcp"
    }
    fn build_route(&self, ctx: &McpContext) -> Router {
        let session_manager: Arc<VtSessionManager> = Arc::clone(&ctx.session_manager);

        // Broaden rmcp's default loopback-only `allowed_hosts` (DNS-rebinding
        // protection). The user has explicitly enabled MCP and set a token;
        // network exposure is controlled by the TCP bind in the host, not here.
        let mut http_cfg = StreamableHttpServerConfig::default();
        http_cfg.allowed_hosts = vec![
            "localhost".into(),
            "127.0.0.1".into(),
            "::1".into(),
            "0.0.0.0".into(),
        ];

        let session_mgr = crate::mcp::shared::local_session_manager();
        let service = StreamableHttpService::new(
            move || {
                // NOTE: ServerHandler is implemented directly on TerminalTools
                // (via #[tool_handler] in handler.rs), so we construct it here
                // directly — there is no separate TerminalHandler wrapper.
                let handler = tools::TerminalTools {
                    session_manager: Arc::clone(&session_manager),
                };
                Ok(handler)
            },
            session_mgr,
            http_cfg,
        );
        Router::new().nest_service("/terminal/mcp", service)
    }
}
