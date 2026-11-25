use std::collections::BTreeMap;
use std::net::{Ipv4Addr, Ipv6Addr};

use if_addrs::IfAddr;
use serde::{Deserialize, Serialize};

use crate::config::types::TerminalServerConfig;

use super::ddns::ddns_domain_from_config;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebServerConfig {
    pub(super) server_id: String,
    pub(super) port: u16,
    pub(super) config_port: u16,
    pub(super) static_http_port_start: u16,
    pub(super) static_http_port_end: u16,
    pub(super) static_http_bind_address: String,
    pub(super) name: String,
    pub(super) secret_key: String,
    pub(super) relay_address: String,
    pub(super) relay_secret_key: String,
    #[serde(default)]
    pub(super) relay_strict_tls: bool,
    #[serde(default)]
    pub(super) relay_ca_pem: String,
    pub(super) cert_path: String,
    pub(super) key_path: String,
    pub(super) log_to_file: bool,
    /// Persisted toggle; applied to the OS by the PUT handler.
    /// `#[serde(default)]` keeps PUTs from older webui builds working.
    #[serde(default)]
    pub(super) autostart: bool,
    /// Output-only: whether the server host can manage login autostart.
    #[serde(default)]
    pub(super) autostart_supported: bool,
    /// Output-only: host operating system (`std::env::consts::OS`), so the
    /// webui can show platform-specific hints (e.g. the Linux autologin note).
    #[serde(default)]
    pub(super) platform: String,
    pub(super) ddns_domain: String,
    pub(super) network_interfaces: Vec<WebNetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WebNetworkInterface {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) ipv4_addresses: Vec<String>,
    pub(crate) ipv6_addresses: Vec<String>,
}

pub(super) fn server_config_to_web(config: &TerminalServerConfig) -> WebServerConfig {
    WebServerConfig {
        server_id: config.server.server_id.clone(),
        port: config.server.relay_port,
        config_port: config.server.config_port,
        static_http_port_start: config.server.static_http_port_start,
        static_http_port_end: config.server.static_http_port_end,
        static_http_bind_address: match config.server.static_http_bind_address {
            crate::config::types::StaticHttpBindAddress::Local => "local".to_string(),
            crate::config::types::StaticHttpBindAddress::Any => "any".to_string(),
        },
        name: config.server.name.clone(),
        secret_key: config.server.secret_key.clone(),
        relay_address: config.server.relay_address.clone(),
        relay_secret_key: config.server.relay_secret_key.clone(),
        relay_strict_tls: config.server.relay_strict_tls,
        relay_ca_pem: config.server.relay_ca_pem.clone(),
        cert_path: config.server.cert_path.clone(),
        key_path: config.server.key_path.clone(),
        log_to_file: config.server.log_to_file,
        autostart: config.server.autostart,
        autostart_supported: crate::autostart::is_supported(),
        platform: std::env::consts::OS.to_string(),
        ddns_domain: server_ddns_domain(config),
        network_interfaces: detect_network_interfaces(),
    }
}

pub(super) fn apply_web_server_config(config: &mut TerminalServerConfig, payload: WebServerConfig) {
    if !payload.server_id.trim().is_empty() {
        config.server.server_id = payload.server_id.trim().to_string();
    }
    config.server.relay_port = payload.port;
    config.server.config_port = payload.config_port;
    config.server.static_http_port_start = payload.static_http_port_start;
    config.server.static_http_port_end = payload.static_http_port_end;
    config.server.static_http_bind_address = if payload.static_http_bind_address == "any" {
        crate::config::types::StaticHttpBindAddress::Any
    } else {
        crate::config::types::StaticHttpBindAddress::Local
    };
    config.server.name = payload.name.trim().to_string();
    config.server.secret_key = payload.secret_key.trim().to_string();
    config.server.relay_address = payload.relay_address.trim().to_string();
    config.server.relay_secret_key = payload.relay_secret_key.trim().to_string();
    config.server.relay_strict_tls = payload.relay_strict_tls;
    config.server.relay_ca_pem = payload.relay_ca_pem.trim().to_string();
    config.server.cert_path = payload.cert_path.trim().to_string();
    config.server.key_path = payload.key_path.trim().to_string();
    config.server.log_to_file = payload.log_to_file;
    config.server.autostart = payload.autostart;
}

fn server_ddns_domain(config: &TerminalServerConfig) -> String {
    ddns_domain_from_config(config)
}

pub(crate) fn detect_network_interfaces() -> Vec<WebNetworkInterface> {
    let Ok(interfaces) = if_addrs::get_if_addrs() else {
        return Vec::new();
    };

    let mut entries = BTreeMap::<String, WebNetworkInterface>::new();
    for interface in interfaces {
        if interface.is_loopback() {
            continue;
        }

        let entry = entries
            .entry(interface.name.clone())
            .or_insert_with(|| WebNetworkInterface {
                id: interface.name.clone(),
                name: interface.name.clone(),
                ipv4_addresses: Vec::new(),
                ipv6_addresses: Vec::new(),
            });

        match interface.addr {
            IfAddr::V4(address) => push_ipv4_address(&mut entry.ipv4_addresses, address.ip),
            IfAddr::V6(address) => push_ipv6_address(&mut entry.ipv6_addresses, address.ip),
        }
    }

    entries
        .into_values()
        .filter(|entry| !entry.ipv4_addresses.is_empty() || !entry.ipv6_addresses.is_empty())
        .collect()
}

fn push_ipv4_address(addresses: &mut Vec<String>, address: Ipv4Addr) {
    if address.is_loopback() || address.is_unspecified() {
        return;
    }

    let address = address.to_string();
    if !addresses.contains(&address) {
        addresses.push(address);
    }
}

fn push_ipv6_address(addresses: &mut Vec<String>, address: Ipv6Addr) {
    if address.is_loopback() || address.is_unspecified() {
        return;
    }

    let address = address.to_string();
    if !addresses.contains(&address) {
        addresses.push(address);
    }
}

#[cfg(test)]
mod tests {
    use super::server_config_to_web;
    use crate::config::types::TerminalServerConfig;

    #[test]
    fn server_config_exposes_enabled_ddns_domain() {
        let mut config = TerminalServerConfig::default();
        config.server.name = "Vibe Terminal".to_string();
        config.server.secret_key = "abcdefghijkl".to_string();
        config.server.relay_address = "wss://relay.example.com:7878".to_string();
        config.server.relay_secret_key = "relay-secret12".to_string();
        config.server.relay_strict_tls = true;
        config.server.relay_ca_pem = "relay-root-ca".to_string();
        config.ddns.enabled = true;
        config.ddns.provider = Some("cloudflare-ddns".to_string());
        config
            .ddns
            .user_config
            .insert("domain".to_string(), "example.com".to_string());
        config
            .ddns
            .user_config
            .insert("subdomain".to_string(), "remote".to_string());

        let web = server_config_to_web(&config);

        assert_eq!(web.name, "Vibe Terminal");
        assert_eq!(web.secret_key, "abcdefghijkl");
        assert_eq!(web.relay_address, "wss://relay.example.com:7878");
        assert_eq!(web.relay_secret_key, "relay-secret12");
        assert!(web.relay_strict_tls);
        assert_eq!(web.relay_ca_pem, "relay-root-ca");
        assert_eq!(web.ddns_domain, "remote.example.com");
    }

    #[test]
    fn server_config_hides_ddns_domain_when_ddns_is_disabled() {
        let mut config = TerminalServerConfig::default();
        config.ddns.enabled = false;
        config.ddns.provider = Some("cloudflare-ddns".to_string());

        let web = server_config_to_web(&config);

        assert!(web.ddns_domain.is_empty());
    }
}
