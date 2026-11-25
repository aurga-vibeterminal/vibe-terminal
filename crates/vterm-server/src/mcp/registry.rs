//! The MCP service contract + compile-time registry.
//!
//! Each capability set (terminal, addon, …) implements [`McpService`] and is
//! registered in [`all_services`]. The host ([`crate::mcp::McpServer`]) iterates
//! the registry to build the merged axum router; it knows nothing about
//! specific services.

use std::sync::Arc;

use axum::Router;

use crate::addon::AddonHost;
use crate::config::ConfigManager;
use crate::session::SessionManager;

/// Shared context handed to every service when building its route. New service
/// needs extend this struct.
pub struct McpContext {
    pub session_manager: Arc<SessionManager>,
    pub config_manager: Arc<ConfigManager>,
    pub addon_host: Arc<AddonHost>,
}

/// One MCP service's contract.
///
/// Each service is served at `/{path}/mcp` as an independent rmcp server
/// (its own `initialize` handshake, its own `tools/list`). Implementations
/// must NOT add their own auth — the host applies the shared bearer layer.
pub trait McpService: Send + Sync {
    /// URL path segment. The service is served at `/{path}/mcp`.
    fn path(&self) -> &'static str;
    /// Server name reported in the rmcp `initialize` response (`serverInfo.name`).
    fn name(&self) -> &'static str;
    /// Build this service's axum route (a `nest_service` of its `StreamableHttpService`).
    fn build_route(&self, ctx: &McpContext) -> Router;
}

/// The compile-time registry of all services the host serves.
///
/// Add a service by appending one line. Services are served in registration
/// order; path uniqueness is enforced by the `registry_paths_unique` test.
pub fn all_services() -> Vec<Box<dyn McpService>> {
    vec![
        Box::new(super::services::terminal::TerminalService),
        Box::new(super::services::addon::AddonMcpService),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_terminal_service() {
        let services = all_services();
        assert!(
            services.iter().any(|s| s.path() == "terminal"),
            "terminal service must be registered"
        );
    }

    #[test]
    fn registry_has_addon_service() {
        let services = all_services();
        assert!(
            services.iter().any(|s| s.path() == "addon"),
            "addon service must be registered"
        );
    }

    #[test]
    fn registry_paths_are_unique() {
        let services = all_services();
        let mut paths: Vec<&str> = services.iter().map(|s| s.path()).collect();
        paths.sort();
        let before = paths.len();
        paths.dedup();
        assert_eq!(paths.len(), before, "duplicate service path in registry");
    }

    #[test]
    fn registered_services_have_nonempty_metadata() {
        for svc in all_services() {
            assert!(!svc.path().is_empty(), "empty path");
            assert!(!svc.name().is_empty(), "empty name");
        }
    }
}
