use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::addon::{AddonHost, AddonRunResult};
use crate::config::{decrypt_addon_secret, ConfigManager};

use super::executor::execute_addon;
use super::interfaces::discover_local_addresses;
use super::types::{DdnsError, DdnsStatus};

const DDNS_POLL_INTERVAL_ENV: &str = "TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS";
const DEFAULT_DDNS_POLL_INTERVAL_SECS: u64 = 300;

pub struct DdnsManager {
    config_manager: Arc<ConfigManager>,
    addon_host: Arc<AddonHost>,
    poll_interval: Duration,
    status: RwLock<DdnsStatus>,
}

impl DdnsManager {
    pub fn new(config_manager: Arc<ConfigManager>, addon_host: Arc<AddonHost>) -> Self {
        Self {
            config_manager,
            addon_host,
            poll_interval: Duration::from_secs(read_poll_interval_secs()),
            status: RwLock::new(DdnsStatus::default()),
        }
    }

    pub async fn run(self: Arc<Self>) {
        info!(
            "DDNS manager started with poll interval of {} seconds",
            self.poll_interval.as_secs()
        );

        let mut config_changes = self.config_manager.subscribe();

        loop {
            let config = self.config_manager.snapshot().await;
            if !config.ddns.enabled {
                self.store_disabled_status(&config).await;
                info!("DDNS manager idle because it is disabled");

                loop {
                    if config_changes.changed().await.is_err() {
                        warn!("DDNS manager stopped because config updates are unavailable");
                        return;
                    }

                    let config = config_changes.borrow().clone();
                    if config.ddns.enabled {
                        break;
                    }

                    self.store_disabled_status(&config).await;
                }

                continue;
            }

            if let Err(error) = self.refresh_once().await {
                warn!("DDNS refresh failed: {}", error);
            }

            tokio::select! {
                _ = tokio::time::sleep(self.poll_interval) => {}
                changed = config_changes.changed() => {
                    if changed.is_err() {
                        warn!("DDNS manager stopped because config updates are unavailable");
                        return;
                    }
                }
            }
        }
    }

    pub async fn refresh_once(&self) -> Result<DdnsStatus, DdnsError> {
        let config = self.config_manager.snapshot().await;
        let ddns_config = &config.ddns;

        info!(
            "DDNS refresh_once: enabled={}, provider={:?}, polling_interval={}min",
            ddns_config.enabled, ddns_config.provider, ddns_config.polling_interval_minutes
        );

        let mut status = DdnsStatus {
            enabled: ddns_config.enabled,
            provider: ddns_config.provider.clone(),
            last_checked_at: Some(SystemTime::now()),
            ..Default::default()
        };

        if !ddns_config.enabled {
            info!(
                "DDNS refresh skipped because it is disabled (provider={:?}, checked_at={:?})",
                status.provider, status.last_checked_at
            );
            self.store_status(status.clone()).await;
            return Ok(status);
        }

        // Resolve provider: if not set or not found, auto-select from available DDNS addons
        let ddns_addons = self.addon_host.ddns_addons();
        let provider = match &ddns_config.provider {
            Some(p) if self.addon_host.get_instance(p).is_some() => p.clone(),
            other => {
                // Auto-select the first DDNS addon
                match ddns_addons.first() {
                    Some(addon) => {
                        let resolved = addon.manifest.id.clone();
                        if other.is_some() {
                            warn!(
                                "DDNS provider '{}' not found in registry, auto-selected '{}'",
                                other.as_ref().unwrap_or(&String::new()),
                                resolved
                            );
                        } else {
                            info!("DDNS provider not set, auto-selected '{}'", resolved);
                        }
                        // Persist the resolved provider
                        if let Err(e) = self
                            .config_manager
                            .apply_patch(crate::config::ConfigPatchRequest {
                                ddns: Some(crate::config::DdnsConfigPatch {
                                    provider: Some(Some(resolved.clone())),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            })
                            .await
                        {
                            warn!("DDNS: failed to persist auto-resolved provider: {}", e);
                        }
                        resolved
                    }
                    None => {
                        warn!("DDNS refresh failed: no DDNS addons available");
                        let error =
                            DdnsError::IncompleteConfig("no DDNS addons available".to_string());
                        status.last_error = Some(error.to_string());
                        self.store_status(status.clone()).await;
                        return Err(error);
                    }
                }
            }
        };

        // Verify the addon exists
        let instance = match self.addon_host.get_instance(&provider) {
            Some(i) => i,
            None => {
                warn!(
                    "DDNS refresh failed: addon '{}' not found in registry",
                    provider
                );
                let error = DdnsError::UnsupportedProvider(provider.clone());
                status.last_error = Some(error.to_string());
                self.store_status(status.clone()).await;
                return Err(error);
            }
        };

        info!(
            "DDNS refresh: using addon '{}' (type={}, builtin={})",
            provider, instance.manifest.addon_type, instance.builtin
        );

        // Decrypt addon user_config values from ddns.user_config
        let mut config_values = std::collections::HashMap::new();
        for (key, value) in &ddns_config.user_config {
            let decrypted = if value.starts_with("enc-v2:") {
                decrypt_addon_secret(value).unwrap_or_else(|_| value.clone())
            } else {
                value.clone()
            };
            debug!(
                "DDNS config: key='{}', encrypted={}",
                key,
                value.starts_with("enc-v2:")
            );
            config_values.insert(key.clone(), decrypted);
        }

        // Apply default values for missing fields
        for field in &instance.manifest.config_fields {
            if !config_values.contains_key(&field.key) {
                if let Some(default) = &field.default_value {
                    debug!(
                        "DDNS config: applying default for '{}' = '{}'",
                        field.key, default
                    );
                    config_values.insert(field.key.clone(), default.clone());
                }
            }
        }

        info!(
            "DDNS config values prepared: keys=[{}]",
            config_values.keys().cloned().collect::<Vec<_>>().join(", ")
        );

        // Discover local IPs
        let local_snapshot =
            discover_local_addresses(ddns_config.network_interface.as_deref()).await;
        status.local_snapshot = local_snapshot.clone();
        info!(
            "DDNS local IPs: ipv4={:?}, ipv6={:?}, ip_family={}",
            local_snapshot.ipv4, local_snapshot.ipv6, ddns_config.ip_family
        );

        // Filter IPs based on configured ip_family
        let (ipv4_pass, ipv6_pass) = match ddns_config.ip_family.as_str() {
            "ipv6" => (None, local_snapshot.ipv6.map(|ip| ip.to_string())),
            "dual" => (
                local_snapshot.ipv4.map(|ip| ip.to_string()),
                local_snapshot.ipv6.map(|ip| ip.to_string()),
            ),
            _ => (local_snapshot.ipv4.map(|ip| ip.to_string()), None),
        };
        info!(
            "DDNS filtered IPs for addon: ipv4={:?}, ipv6={:?}",
            ipv4_pass, ipv6_pass
        );

        // Run the addon
        info!("DDNS: invoking addon '{}' run()", provider);
        let source = self
            .addon_host
            .get_source(&provider)
            .unwrap_or_default()
            .to_string();
        let result: AddonRunResult = tokio::task::spawn_blocking(move || {
            execute_addon(&source, config_values, ipv4_pass, ipv6_pass)
        })
        .await
        .unwrap_or_else(|error| {
            warn!("DDNS addon '{}' execution panicked: {}", provider, error);
            AddonRunResult {
                success: false,
                error: Some(format!("JS execution panicked: {}", error)),
                ..Default::default()
            }
        });

        // for log in &result.logs {
        //     info!("[addon:{}] {}", provider, log);
        // }

        if result.success {
            status.ip = result.ip.clone();
            info!(
                "DDNS refresh completed (enabled={}, provider={:?}, ip={:?}, checked_at={:?})",
                status.enabled, status.provider, status.ip, status.last_checked_at
            );
        } else {
            status.last_error = result
                .error
                .clone()
                .or_else(|| Some("unknown error".to_string()));
            warn!(
                "DDNS refresh failed for addon '{}': {:?}",
                provider, status.last_error
            );
        }

        self.store_status(status.clone()).await;

        if result.success {
            Ok(status)
        } else {
            Err(DdnsError::AddonExecution(
                result
                    .error
                    .unwrap_or_else(|| "unknown addon error".to_string()),
            ))
        }
    }

    async fn store_status(&self, status: DdnsStatus) {
        *self.status.write().await = status;
    }

    async fn store_disabled_status(&self, config: &crate::config::TerminalServerConfig) {
        self.store_status(DdnsStatus {
            enabled: false,
            provider: config.ddns.provider.clone(),
            ..Default::default()
        })
        .await;
    }
}

fn read_poll_interval_secs() -> u64 {
    std::env::var(DDNS_POLL_INTERVAL_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_DDNS_POLL_INTERVAL_SECS)
}
