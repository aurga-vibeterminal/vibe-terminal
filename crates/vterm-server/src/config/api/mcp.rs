use serde::{Deserialize, Serialize};

use crate::addon::AddonHost;
use crate::config::types::{ConfigError, McpConfig, TerminalServerConfig};
use crate::mcp::registry::all_services;
use crate::mcp::services::addon::bridge::global_hub;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebMcpConfig {
    pub enabled: bool,
    pub port: u16,
    pub bind_address: String,
    pub access_token: String,
    pub stream_throttle_ms: u32,
}

/// One MCP endpoint currently served by the MCP host, as shown in the
/// webui's MCP section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebMcpEndpoint {
    /// Service path segment (builtins) or addon id; the endpoint is served
    /// at `/{id}/mcp`.
    pub id: String,
    /// Human-readable label (server name for builtins, addon name for addons).
    pub name: String,
    /// `builtin` for the compiled-in services, `addon` for `type:"mcp"` addons.
    pub kind: &'static str,
    /// Addon endpoints only: whether the addon's daemon is currently
    /// connected to the `/addon/bridge` WebSocket hub.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daemon_connected: Option<bool>,
}

/// List every MCP endpoint the host serves: the compiled-in services from
/// the registry plus one endpoint per live `type:"mcp"` addon. The webui
/// prefixes these paths with the configured port to build full URLs.
pub(super) fn list_mcp_endpoints(addon_host: &AddonHost) -> Vec<WebMcpEndpoint> {
    let mut endpoints: Vec<WebMcpEndpoint> = all_services()
        .iter()
        .filter(|service| service.path() != "addon")
        .map(|service| WebMcpEndpoint {
            id: service.path().to_string(),
            name: service.name().to_string(),
            kind: "builtin",
            daemon_connected: None,
        })
        .collect();

    let hub = global_hub();
    for instance in addon_host.instances() {
        if instance.manifest.addon_type != "mcp" {
            continue;
        }
        endpoints.push(WebMcpEndpoint {
            id: instance.manifest.id.clone(),
            name: instance.manifest.name.clone(),
            kind: "addon",
            daemon_connected: Some(hub.is_connected(&instance.manifest.id)),
        });
    }
    endpoints
}

pub(super) fn mcp_config_to_web(config: &McpConfig) -> WebMcpConfig {
    WebMcpConfig {
        enabled: config.enabled,
        port: config.port,
        bind_address: config.bind_address.clone(),
        access_token: config.access_token.clone(),
        stream_throttle_ms: config.stream_throttle_ms,
    }
}

pub(super) fn apply_web_mcp_config(
    config: &mut TerminalServerConfig,
    payload: WebMcpConfig,
) -> Result<(), ConfigError> {
    // The token is mandatory: an empty one would lock remote agents out and
    // contradicts the random-token default (McpConfig::default). Trim so a
    // whitespace-only value is rejected too.
    let access_token = payload.access_token.trim().to_string();
    if access_token.is_empty() {
        return Err(ConfigError::Validation(
            "mcp.access_token must be set".to_string(),
        ));
    }

    config.mcp.enabled = payload.enabled;
    config.mcp.port = payload.port;
    config.mcp.bind_address = payload.bind_address;
    config.mcp.access_token = access_token;
    config.mcp.stream_throttle_ms = payload.stream_throttle_ms;

    // Keep the server-managed "Vibe Terminal MCP Bearer Token" environment
    // leaf in step with the new settings (created/refreshed while enabled,
    // removed when disabled) so terminals can apply it before running agent
    // setup commands.
    super::environment::sync_mcp_token_leaf(config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{EnvironmentNode, EnvironmentVariable};

    fn payload(access_token: &str) -> WebMcpConfig {
        WebMcpConfig {
            enabled: true,
            port: 7880,
            bind_address: "127.0.0.1".to_string(),
            access_token: access_token.to_string(),
            stream_throttle_ms: 100,
        }
    }

    #[test]
    fn apply_rejects_empty_access_token() {
        let mut config = TerminalServerConfig::default();
        let error = apply_web_mcp_config(&mut config, payload("")).unwrap_err();
        assert!(error.to_string().contains("mcp.access_token must be set"));
    }

    #[test]
    fn apply_rejects_whitespace_only_access_token() {
        let mut config = TerminalServerConfig::default();
        let error = apply_web_mcp_config(&mut config, payload("   ")).unwrap_err();
        assert!(error.to_string().contains("mcp.access_token must be set"));
    }

    #[test]
    fn apply_stores_trimmed_access_token() {
        let mut config = TerminalServerConfig::default();
        apply_web_mcp_config(&mut config, payload("  secret  ")).unwrap();
        assert_eq!(config.mcp.access_token, "secret");
    }

    /// Saving MCP settings reconciles the server-managed environment leaf:
    /// enabled → present with the current token; disabled → gone.
    #[test]
    fn apply_syncs_mcp_token_environment_leaf() {
        let mut config = TerminalServerConfig::default();
        config
            .environment
            .root
            .children
            .push(EnvironmentNode::Leaf {
                id: "1".to_string(),
                name: "user".to_string(),
                variables: Vec::new(),
            });

        let mut p = payload("secret");
        p.enabled = false;
        apply_web_mcp_config(&mut config, p).unwrap();
        assert!(config.environment.root.find_node("mcp-token").is_none());

        let mut p = payload("secret");
        p.enabled = true;
        apply_web_mcp_config(&mut config, p).unwrap();
        let leaf = config
            .environment
            .root
            .find_node("mcp-token")
            .expect("managed leaf created");
        assert_eq!(
            leaf.as_leaf_variables().unwrap(),
            &[EnvironmentVariable {
                key: "VTERM_MCP_TOKEN".to_string(),
                value: "secret".to_string(),
            }][..]
        );
        // User nodes survive.
        assert!(config.environment.root.find_node("1").is_some());
    }
}
