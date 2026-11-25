use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::addon::AddonHost;
use crate::config::types::ConfigError;
use crate::config::types::TerminalServerConfig;
use crate::config::{decrypt_addon_secret, encrypt_addon_secret};

use super::server;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct WebDdnsSyncStatus {
    last_sync: Option<String>,
    last_ip: Option<String>,
    next_sync: Option<String>,
    syncing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebDdnsConfig {
    pub(super) enabled: bool,
    pub(super) provider: String,
    #[serde(default)]
    pub(super) config: HashMap<String, String>,
    pub(super) polling_interval_minutes: u32,
    pub(super) ip_family: String,
    #[serde(default)]
    pub(super) network_interface: Option<String>,
    sync_status: WebDdnsSyncStatus,
    pub(super) ddns_domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebDdnsSyncResult {
    pub(super) success: bool,
    pub(super) last_ip: Option<String>,
    pub(super) error: Option<String>,
}

pub(super) fn ddns_config_to_web(
    config: &TerminalServerConfig,
    addon_host: &Arc<AddonHost>,
    last_ip: Option<String>,
) -> WebDdnsConfig {
    let provider = config.ddns.provider.clone().unwrap_or_default();

    WebDdnsConfig {
        enabled: config.ddns.enabled,
        config: selected_addon_config(config, addon_host, &provider),
        provider,
        polling_interval_minutes: config.ddns.polling_interval_minutes,
        ip_family: config.ddns.ip_family.clone(),
        network_interface: config.ddns.network_interface.clone(),
        sync_status: WebDdnsSyncStatus {
            last_sync: None,
            last_ip,
            next_sync: None,
            syncing: false,
        },
        ddns_domain: ddns_domain_from_config(config),
    }
}

/// Compute the DDNS full domain from the addon user_config.
/// For cloudflare-ddns: use the first semicolon-separated subdomain, or the root domain if unset.
pub(crate) fn ddns_domain_from_config(config: &TerminalServerConfig) -> String {
    if !config.ddns.enabled {
        return String::new();
    }
    let value = |key: &str| {
        config
            .ddns
            .user_config
            .get(key)
            .map(|value| decrypt_addon_secret(value).unwrap_or_else(|_| value.clone()))
            .unwrap_or_default()
    };
    let domain = match value("domain") {
        d if !d.is_empty() => d,
        _ => return String::new(),
    };
    value("subdomain")
        .split(';')
        .map(str::trim)
        .find(|subdomain| !subdomain.is_empty())
        .map(|subdomain| format!("{}.{}", subdomain, domain))
        .unwrap_or(domain)
}

pub(super) fn apply_web_ddns_config(
    config: &mut TerminalServerConfig,
    addon_host: &Arc<AddonHost>,
    payload: WebDdnsConfig,
) -> Result<(), ConfigError> {
    config.ddns.enabled = payload.enabled;
    let provider = payload.provider.trim().to_string();
    config.ddns.provider = Some(provider.clone()).filter(|value| !value.is_empty());
    config.ddns.polling_interval_minutes = payload.polling_interval_minutes;
    let valid = ["ipv4", "ipv6", "dual"];
    if valid.contains(&payload.ip_family.as_str()) {
        config.ddns.ip_family = payload.ip_family;
    }
    config.ddns.network_interface = payload.network_interface.filter(|v| !v.is_empty());

    if !provider.is_empty() {
        let instance = addon_host
            .get_instance(&provider)
            .ok_or_else(|| ConfigError::BadRequest(format!("addon not found: {}", provider)))?;

        // Required fields only gate an enabled service: while disabled the
        // addon never runs, so a partially filled provider config may be
        // saved and completed later.
        if payload.enabled {
            for field in &instance.manifest.config_fields {
                if field.required {
                    let value = payload
                        .config
                        .get(&field.key)
                        .map(|v| v.trim())
                        .filter(|v| !v.is_empty());
                    let has_default = field
                        .default_value
                        .as_deref()
                        .map(|v| !v.trim().is_empty())
                        .unwrap_or(false);
                    if value.is_none() && !has_default {
                        return Err(ConfigError::Validation(format!(
                            "required field '{}' ({}) is empty",
                            field.key, field.label
                        )));
                    }
                }
            }
        }

        let mut user_config = HashMap::new();
        for field in &instance.manifest.config_fields {
            let value = payload
                .config
                .get(&field.key)
                .cloned()
                .or_else(|| field.default_value.clone())
                .unwrap_or_default();

            if !value.is_empty() && !value.starts_with("enc-v2:") {
                let encrypted = encrypt_addon_secret(&value).map_err(|error| {
                    ConfigError::Validation(format!("failed to encrypt addon config: {}", error))
                })?;
                user_config.insert(field.key.clone(), encrypted);
            } else {
                user_config.insert(field.key.clone(), value);
            }
        }

        config.ddns.user_config = user_config;
    }

    Ok(())
}

pub(super) fn ddns_network_interfaces() -> Vec<server::WebNetworkInterface> {
    server::detect_network_interfaces()
}

fn selected_addon_config(
    config: &TerminalServerConfig,
    addon_host: &Arc<AddonHost>,
    provider: &str,
) -> HashMap<String, String> {
    let Some(instance) = addon_host.get_instance(provider) else {
        return HashMap::new();
    };

    let stored = &config.ddns.user_config;
    let mut user_config = HashMap::new();

    for field in &instance.manifest.config_fields {
        let value = stored
            .get(&field.key)
            .cloned()
            .or_else(|| field.default_value.clone())
            .unwrap_or_default();

        let value = if value.starts_with("enc-v2:") {
            decrypt_addon_secret(&value).unwrap_or(value)
        } else {
            value
        };

        user_config.insert(field.key.clone(), value);
    }

    user_config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_ddns_config_persists_selected_addon_config() {
        let addon_host = Arc::new(AddonHost::new());
        let mut config = TerminalServerConfig::default();

        let mut addon_config = HashMap::new();
        addon_config.insert("email".to_string(), "user@example.com".to_string());
        addon_config.insert("api_key".to_string(), "secret-token".to_string());
        addon_config.insert("domain".to_string(), "term.example.com".to_string());
        addon_config.insert("subdomain".to_string(), "term".to_string());

        apply_web_ddns_config(
            &mut config,
            &addon_host,
            WebDdnsConfig {
                enabled: true,
                provider: "cloudflare-ddns".to_string(),
                config: addon_config,
                polling_interval_minutes: 5,
                ip_family: "ipv4".to_string(),
                network_interface: None,
                sync_status: WebDdnsSyncStatus {
                    last_sync: None,
                    last_ip: None,
                    next_sync: None,
                    syncing: false,
                },
                ddns_domain: String::new(),
            },
        )
        .unwrap();

        assert!(config
            .ddns
            .user_config
            .get("email")
            .unwrap()
            .starts_with("enc-v2:"));
        assert!(config
            .ddns
            .user_config
            .get("api_key")
            .unwrap()
            .starts_with("enc-v2:"));

        let web = ddns_config_to_web(&config, &addon_host, None);
        assert_eq!(web.provider, "cloudflare-ddns");
        assert_eq!(web.config.get("api_key").unwrap(), "secret-token");
        assert_eq!(web.config.get("domain").unwrap(), "term.example.com");
    }

    #[test]
    fn ddns_domain_uses_first_semicolon_separated_subdomain() {
        let mut config = TerminalServerConfig::default();
        config.ddns.enabled = true;
        config
            .ddns
            .user_config
            .insert("domain".to_string(), "example.com".to_string());
        config.ddns.user_config.insert(
            "subdomain".to_string(),
            " ; remote ; api ".to_string(),
        );

        assert_eq!(ddns_domain_from_config(&config), "remote.example.com");
    }

    #[test]
    fn disabled_ddns_config_saves_without_required_fields() {
        let addon_host = Arc::new(AddonHost::new());
        let mut config = TerminalServerConfig::default();

        // The cloudflare addon requires domain/api_key — while disabled an
        // empty config still saves; the same payload must fail when enabled.
        let payload = |enabled| WebDdnsConfig {
            enabled,
            provider: "cloudflare-ddns".to_string(),
            config: HashMap::new(),
            polling_interval_minutes: 5,
            ip_family: "ipv4".to_string(),
            network_interface: None,
            sync_status: WebDdnsSyncStatus {
                last_sync: None,
                last_ip: None,
                next_sync: None,
                syncing: false,
            },
            ddns_domain: String::new(),
        };

        apply_web_ddns_config(&mut config, &addon_host, payload(false)).unwrap();
        assert!(!config.ddns.enabled);
        assert_eq!(config.ddns.provider.as_deref(), Some("cloudflare-ddns"));

        let mut enabled_config = TerminalServerConfig::default();
        assert!(apply_web_ddns_config(&mut enabled_config, &addon_host, payload(true)).is_err());
    }
}
