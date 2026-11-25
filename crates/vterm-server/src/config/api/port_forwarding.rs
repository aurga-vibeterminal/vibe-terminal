use serde::{Deserialize, Serialize};
use vterm_common::PortForwardInfo;

use crate::config::types::{ConfigError, TerminalServerConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebCreatePortForwardRequest {
    pub(super) target_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebPortForward {
    target_port: u16,
    proxy_port: u16,
    active_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebPortForwardShare {
    #[serde(skip_serializing_if = "Option::is_none")]
    share_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ticket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebPortForwardingConfig {
    pub(super) port_range_start: u16,
    pub(super) port_range_end: u16,
    /// Optional in PUT bodies so older Web UI clients preserve the current
    /// runtime policy instead of resetting it while editing the port range.
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    idle_timeout_minutes: Option<u32>,
    /// Only meaningful in GET responses. PUT bodies may omit it: the live
    /// forward list is runtime state owned by the manager, never written
    /// through this config endpoint.
    #[serde(default)]
    forwards: Vec<WebPortForward>,
}

pub(super) fn port_forwarding_config_to_web(
    config: &TerminalServerConfig,
    forwards: &[PortForwardInfo],
) -> WebPortForwardingConfig {
    WebPortForwardingConfig {
        port_range_start: config.server.proxy_port_start,
        port_range_end: config.server.proxy_port_end,
        enabled: Some(config.server.proxy_enabled),
        idle_timeout_minutes: Some(config.server.proxy_idle_timeout_minutes),
        forwards: forwards.iter().map(web_port_forward).collect(),
    }
}

pub(super) fn validate_create_request(
    payload: &WebCreatePortForwardRequest,
) -> Result<(), ConfigError> {
    if payload.target_port == 0 {
        return Err(ConfigError::BadRequest(
            "target port must be greater than zero".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn apply_web_port_forwarding_config(
    config: &mut TerminalServerConfig,
    payload: WebPortForwardingConfig,
) {
    config.server.proxy_port_start = payload.port_range_start;
    config.server.proxy_port_end = payload.port_range_end;
    if let Some(enabled) = payload.enabled {
        config.server.proxy_enabled = enabled;
    }
    if let Some(idle_timeout_minutes) = payload.idle_timeout_minutes {
        config.server.proxy_idle_timeout_minutes = idle_timeout_minutes;
    }
}

pub(super) fn web_port_forward(info: &PortForwardInfo) -> WebPortForward {
    WebPortForward {
        target_port: info.target_port,
        proxy_port: info.proxy_port,
        active_connections: info.active_connection_count,
    }
}

pub(super) fn web_port_forward_share(
    info: &PortForwardInfo,
    config: &TerminalServerConfig,
    share_token: Option<String>,
) -> WebPortForwardShare {
    let ticket = (info.target_port == config.server.config_port)
        .then(|| crate::http_auth::issue_settings_ticket("web-port-forward"));
    WebPortForwardShare {
        share_token,
        ticket,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::ServerConfig;

    #[test]
    fn fresh_server_config_disables_idle_timeout() {
        let config: ServerConfig = serde_json::from_str("{}").unwrap();

        // Desktop (`ui`) builds ship forwarding on; headless builds off.
        assert_eq!(config.proxy_enabled, cfg!(feature = "ui"));
        assert_eq!(
            config.proxy_idle_timeout_minutes,
            crate::config::types::DEFAULT_PROXY_IDLE_TIMEOUT_MINUTES
        );
        assert_eq!(crate::config::types::DEFAULT_PROXY_IDLE_TIMEOUT_MINUTES, 0);
    }

    #[test]
    fn web_update_preserves_omitted_policy_and_serializes_current_values() {
        let mut config = TerminalServerConfig::default();
        config.server.proxy_enabled = false;
        config.server.proxy_idle_timeout_minutes = 30;
        let legacy_payload = serde_json::from_value(serde_json::json!({
            "portRangeStart": 62000,
            "portRangeEnd": 62999
        }))
        .unwrap();

        apply_web_port_forwarding_config(&mut config, legacy_payload);

        assert!(!config.server.proxy_enabled);
        assert_eq!(config.server.proxy_idle_timeout_minutes, 30);
        let web = serde_json::to_value(port_forwarding_config_to_web(&config, &[])).unwrap();
        assert_eq!(web["enabled"], false);
        assert_eq!(web["idleTimeoutMinutes"], 30);

        let update = serde_json::from_value(serde_json::json!({
            "portRangeStart": 62000,
            "portRangeEnd": 62999,
            "enabled": true,
            "idleTimeoutMinutes": 0
        }))
        .unwrap();
        apply_web_port_forwarding_config(&mut config, update);
        assert!(config.server.proxy_enabled);
        assert_eq!(config.server.proxy_idle_timeout_minutes, 0);
    }

    #[test]
    fn static_http_share_reuses_its_token() {
        let config = TerminalServerConfig::default();
        let info = PortForwardInfo {
            target_port: config.server.config_port + 1,
            proxy_port: 62000,
            active_connection_count: 0,
        };
        let share = web_port_forward_share(&info, &config, Some("share-secret".into()));
        assert_eq!(share.share_token.as_deref(), Some("share-secret"));
        assert_eq!(share.ticket, None);
        assert_eq!(
            serde_json::to_value(share).unwrap(),
            serde_json::json!({ "shareToken": "share-secret" })
        );
    }

    #[test]
    fn settings_share_mints_a_one_time_ticket() {
        let config = TerminalServerConfig::default();
        let info = PortForwardInfo {
            target_port: config.server.config_port,
            proxy_port: 62000,
            active_connection_count: 0,
        };
        let share = web_port_forward_share(&info, &config, None);
        let ticket = share.ticket.expect("settings ticket");
        assert!(crate::http_auth::settings_access()
            .exchange_grant(&ticket)
            .is_some());
        assert!(crate::http_auth::settings_access()
            .exchange_grant(&ticket)
            .is_none());
    }

    #[test]
    fn ordinary_forward_has_no_builtin_credential() {
        let config = TerminalServerConfig::default();
        let info = PortForwardInfo {
            target_port: config.server.config_port + 1,
            proxy_port: 62000,
            active_connection_count: 0,
        };
        let share = web_port_forward_share(&info, &config, None);
        assert_eq!(serde_json::to_value(share).unwrap(), serde_json::json!({}));
    }
}
