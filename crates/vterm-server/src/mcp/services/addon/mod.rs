//! The addon MCP service — mounts one rmcp `StreamableHttpService` per live
//! `type:"mcp"` addon at `/{addon_id}/mcp`, plus the shared WebSocket bridge
//! at `/addon/bridge` that addon daemons (e.g. the EasyEDA connector) dial.
//!
//! Routes are built once when the MCP host starts: an MCP addon uploaded
//! *after* startup is served only after the host restarts (config change or
//! server reboot). Builtins such as `easyeda` are always present. Editing an
//! addon's JS source takes effect immediately regardless, because the
//! executor re-reads the source from [`AddonHost`] on every call.

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};

use crate::addon::AddonHost;
use crate::config::ConfigManager;
use crate::mcp::registry::{McpContext, McpService};

pub mod bridge;
pub mod handler;

pub struct AddonMcpService;

impl McpService for AddonMcpService {
    fn path(&self) -> &'static str {
        "addon"
    }
    fn name(&self) -> &'static str {
        "vterm-addon-mcp"
    }
    fn build_route(&self, ctx: &McpContext) -> Router {
        let addon_host: Arc<AddonHost> = Arc::clone(&ctx.addon_host);
        let config_manager: Arc<ConfigManager> = Arc::clone(&ctx.config_manager);

        let mut router = Router::new();
        for instance in ctx.addon_host.instances() {
            if instance.manifest.addon_type != "mcp" {
                continue;
            }
            let addon_id = instance.manifest.id.clone();

            // Broaden rmcp's default loopback-only `allowed_hosts`, same as the
            // terminal services.
            let mut http_cfg = StreamableHttpServerConfig::default();
            http_cfg.allowed_hosts = vec![
                "localhost".into(),
                "127.0.0.1".into(),
                "::1".into(),
                "0.0.0.0".into(),
            ];

            let session_mgr = crate::mcp::shared::local_session_manager();
            let service = StreamableHttpService::new(
                {
                    let addon_host = Arc::clone(&addon_host);
                    let config_manager = Arc::clone(&config_manager);
                    let addon_id = addon_id.clone();
                    move || {
                        Ok(handler::AddonMcpHandler {
                            addon_host: Arc::clone(&addon_host),
                            config_manager: Arc::clone(&config_manager),
                            addon_id: addon_id.clone(),
                        })
                    }
                },
                session_mgr,
                http_cfg,
            );
            router = router.nest_service(&format!("/{addon_id}/mcp"), service);
        }

        router.route("/addon/bridge", get(bridge::addon_bridge_ws))
    }
}
