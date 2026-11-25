use std::path::PathBuf;

use tokio::sync::{watch, RwLock};
use uuid::Uuid;

use super::types::{
    default_log_to_file, encrypt_addon_secret, generate_random_token, generate_server_secret_key,
    ConfigDocument, ConfigError, ConfigPatchRequest, ConfigUpdateResult, LlmProviderConfig,
    TerminalServerConfig, DEFAULT_ASR_PORT,
};
use super::validation::{apply_patch, validate_config};

const CONFIG_FILE_ENV: &str = "TERMINAL_SERVER_CONFIG_FILE";
#[cfg(DEVELOPMENT)]
const DEFAULT_CONFIG_DIR: &str = ".vibeterminal-dev";
#[cfg(not(DEVELOPMENT))]
const DEFAULT_CONFIG_DIR: &str = ".vibeterminal";
const DEFAULT_CONFIG_FILE: &str = "server.json";

pub struct ConfigManager {
    path: PathBuf,
    state: RwLock<TerminalServerConfig>,
    changes: watch::Sender<TerminalServerConfig>,
}

impl ConfigManager {
    pub async fn load() -> Result<Self, ConfigError> {
        Self::load_from_path(config_file_path()?).await
    }

    pub async fn load_from_path(path: PathBuf) -> Result<Self, ConfigError> {
        let mut config = if tokio::fs::try_exists(&path).await? {
            let content = tokio::fs::read_to_string(&path).await?;
            serde_json::from_str::<TerminalServerConfig>(&content)?
        } else {
            let mut default_config = TerminalServerConfig::default();
            // Platform-specific first-startup defaults (e.g. WSL port set).
            // Also runs for `--show-password`, which loads the config the same way.
            crate::config::types::apply_platform_default_ports(&mut default_config);
            ensure_server_id(&mut default_config);
            ensure_server_name(&mut default_config);
            persist_config(&path, &default_config).await?;
            default_config
        };

        let id_persist = ensure_server_id(&mut config);
        let name_persist = ensure_server_name(&mut config);
        let secret_key_persist = ensure_server_secret_key(&mut config);
        let migrated = migrate_llm_provider(&mut config);
        let ddns_migrated = migrate_ddns_user_config(&mut config);
        let asr_migrated = migrate_asr_local_config(&mut config);
        let mcp_migrated = migrate_mcp_default_enabled(&mut config);
        let mcp_token_persist = ensure_mcp_access_token(&mut config);
        let addon_config_migrated = addon_config_needs_encryption(&config);
        validate_config(&config)?;
        if id_persist
            || name_persist
            || secret_key_persist
            || migrated
            || ddns_migrated
            || asr_migrated
            || mcp_migrated
            || mcp_token_persist
            || addon_config_migrated
        {
            persist_config(&path, &config).await?;
        }

        let (changes, _) = watch::channel(config.clone());

        Ok(Self {
            path,
            state: RwLock::new(config),
            changes,
        })
    }

    pub async fn snapshot(&self) -> TerminalServerConfig {
        self.state.read().await.clone()
    }

    pub async fn document(&self) -> ConfigDocument {
        ConfigDocument {
            config_path: self.path.clone(),
            config: self.snapshot().await,
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<TerminalServerConfig> {
        self.changes.subscribe()
    }

    pub async fn apply_patch(
        &self,
        patch: ConfigPatchRequest,
    ) -> Result<ConfigUpdateResult, ConfigError> {
        let current = self.snapshot().await;
        let update = apply_patch(&current, patch)?;
        persist_config(&self.path, &update.config).await?;
        *self.state.write().await = update.config.clone();
        let _ = self.changes.send(update.config.clone());
        Ok(update)
    }

    pub async fn replace(
        &self,
        config: TerminalServerConfig,
    ) -> Result<ConfigUpdateResult, ConfigError> {
        validate_config(&config)?;
        persist_config(&self.path, &config).await?;
        *self.state.write().await = config.clone();
        let _ = self.changes.send(config.clone());
        Ok(ConfigUpdateResult {
            config,
            restart_required: false,
        })
    }
}

/// Ensures the persisted server config always carries a stable local server ID.
fn ensure_server_id(config: &mut TerminalServerConfig) -> bool {
    if !config.server.server_id.trim().is_empty() {
        return false;
    }

    config.server.server_id = Uuid::new_v4().to_string();
    true
}

/// Ensures the server has a human-readable name. Falls back to the computer's
/// hostname when the name field is empty (first launch or legacy config).
/// Under WSL the hostname is the distro's Linux hostname, which is easy to
/// mistake for the Windows host once the server registers there — tag it with
/// a `WSL-` prefix so the origin is obvious in the host's server list.
fn ensure_server_name(config: &mut TerminalServerConfig) -> bool {
    if !config.server.name.trim().is_empty() {
        return false;
    }

    config.server.name = default_server_name(crate::config::types::is_running_under_wsl());
    true
}

/// Build the first-launch default server name: the machine hostname, prefixed
/// with `WSL-` when running inside a WSL distro (see `ensure_server_name`).
fn default_server_name(is_wsl: bool) -> String {
    match gethostname::gethostname()
        .into_string()
        .ok()
        .filter(|s| !s.is_empty())
    {
        Some(hostname) => {
            if is_wsl {
                format!("WSL-{}", hostname)
            } else {
                hostname
            }
        }
        None => "Vibe Terminal".to_string(),
    }
}

/// Ensures the server always requires a shared secret for non-local clients.
fn ensure_server_secret_key(config: &mut TerminalServerConfig) -> bool {
    if !config.server.secret_key.trim().is_empty() {
        return false;
    }

    config.server.secret_key = generate_server_secret_key();
    true
}

/// Migrate legacy `terminal.llm_provider` to the new `llm.terminal` config section.
fn migrate_llm_provider(config: &mut TerminalServerConfig) -> bool {
    if let Some(provider) = config.terminal.llm_provider.take() {
        if !provider.trim().is_empty() && config.llm.terminal.provider.provider.is_none() {
            config.llm.terminal.provider.enabled = true;
            config.llm.terminal.provider.provider = Some(provider.trim().to_string());
            return true;
        }
    }
    false
}

/// Migrate legacy `addons.entries[ddns_provider].user_config` to `ddns.user_config`.
fn migrate_ddns_user_config(config: &mut TerminalServerConfig) -> bool {
    if !config.ddns.user_config.is_empty() {
        return false;
    }

    if let Some(provider) = &config.ddns.provider {
        if let Some(entry) = config.addons.entries.remove(provider) {
            if !entry.user_config.is_empty() {
                config.ddns.user_config = entry.user_config;
                return true;
            }
        }
    }
    false
}

/// Normalize the locally-managed ASR server fields so existing deployments
/// pick up the new `bind_address`/`port` defaults without manual
/// editing. We only touch values that are empty or `0`; we never overwrite a
/// user-chosen port or bind address.
///
/// Note: serde `#[serde(default = ...)]` already fills these in when the keys
/// are absent on disk, so this migration only matters for the rare case where
/// a field was explicitly persisted as `0` / `""` by an older code path.
fn migrate_asr_local_config(config: &mut TerminalServerConfig) -> bool {
    let mut changed = false;

    if config.asr.port == 0 {
        config.asr.port = DEFAULT_ASR_PORT;
        changed = true;
    }
    if config.asr.bind_address.trim().is_empty() {
        config.asr.bind_address = super::types::default_asr_bind_address();
        changed = true;
    }

    changed
}

/// One-time migration for configs persisted while the MCP default was
/// `enabled: false`: turn the endpoint on so existing deployments match the
/// current default without manual toggling. Gated on the config version —
/// files saved by this build already carry version 2, so an opt-out made
/// afterwards is respected and never re-flipped.
fn migrate_mcp_default_enabled(config: &mut TerminalServerConfig) -> bool {
    if config.version >= 2 {
        return false;
    }
    if !config.mcp.enabled {
        config.mcp.enabled = true;
    }
    config.version = 2;
    true
}

/// The MCP access token must never be empty: an empty token rejects every
/// remote request (fail closed, see `mcp::auth`) and the config API refuses
/// to store one. Fresh installs ship a random token via `McpConfig::default`;
/// this repairs configs persisted by older builds whose default was an
/// empty string (and any hand-edited file), generating a fresh random token
/// so the saved config always carries one.
fn ensure_mcp_access_token(config: &mut TerminalServerConfig) -> bool {
    if config.mcp.access_token.trim().is_empty() {
        config.mcp.access_token = generate_random_token(32);
        return true;
    }
    false
}

async fn persist_config(path: &PathBuf, config: &TerminalServerConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    // Addon configuration is opaque provider-owned data. Encrypt every
    // non-empty value here instead of relying on each addon's manifest to
    // correctly identify secrets. This is the final persistence boundary, so
    // it also protects configs written through legacy and patch APIs.
    let mut persisted = config.clone();
    encrypt_addon_config_values(&mut persisted)?;
    let body = serde_json::to_vec_pretty(&persisted)?;
    tokio::fs::write(path, body).await?;
    Ok(())
}

fn addon_config_needs_encryption(config: &TerminalServerConfig) -> bool {
    let maps = config
        .addons
        .entries
        .values()
        .map(|entry| &entry.user_config)
        .chain(std::iter::once(&config.ddns.user_config))
        .chain(std::iter::once(&config.asr.provider_config))
        .chain(llm_provider_configs(config).map(|provider| &provider.user_config));

    maps.flatten()
        .any(|(_, value)| !value.is_empty() && !value.starts_with("enc-v2:"))
        || config
            .asr
            .access_token
            .as_deref()
            .is_some_and(|value| !value.is_empty() && !value.starts_with("enc-v2:"))
}

fn encrypt_addon_config_values(config: &mut TerminalServerConfig) -> Result<(), ConfigError> {
    for entry in config.addons.entries.values_mut() {
        encrypt_config_map(&mut entry.user_config)?;
    }
    encrypt_config_map(&mut config.ddns.user_config)?;
    encrypt_config_map(&mut config.asr.provider_config)?;
    for provider in llm_provider_configs_mut(config) {
        encrypt_config_map(&mut provider.user_config)?;
    }
    if let Some(token) = &mut config.asr.access_token {
        encrypt_config_value(token)?;
    }
    Ok(())
}

fn encrypt_config_map(
    values: &mut std::collections::HashMap<String, String>,
) -> Result<(), ConfigError> {
    for value in values.values_mut() {
        encrypt_config_value(value)?;
    }
    Ok(())
}

fn encrypt_config_value(value: &mut String) -> Result<(), ConfigError> {
    if value.is_empty() || value.starts_with("enc-v2:") {
        return Ok(());
    }
    *value = encrypt_addon_secret(value).map_err(|error| {
        ConfigError::Validation(format!("failed to encrypt addon config: {error}"))
    })?;
    Ok(())
}

fn llm_provider_configs(config: &TerminalServerConfig) -> impl Iterator<Item = &LlmProviderConfig> {
    [
        &config.llm.terminal.provider,
        &config.llm.git.provider,
        &config.llm.git_conflict.provider,
        &config.llm.chat,
        &config.llm.voice_polish,
        &config.llm.code_analysis.provider,
        &config.llm.translation.provider,
    ]
    .into_iter()
}

fn llm_provider_configs_mut(
    config: &mut TerminalServerConfig,
) -> impl Iterator<Item = &mut LlmProviderConfig> {
    [
        &mut config.llm.terminal.provider,
        &mut config.llm.git.provider,
        &mut config.llm.git_conflict.provider,
        &mut config.llm.chat,
        &mut config.llm.voice_polish,
        &mut config.llm.code_analysis.provider,
        &mut config.llm.translation.provider,
    ]
    .into_iter()
}

fn config_file_path() -> Result<PathBuf, ConfigError> {
    if let Ok(path) = std::env::var(CONFIG_FILE_ENV) {
        return Ok(PathBuf::from(path));
    }

    Ok(user_home_dir()?
        .join(DEFAULT_CONFIG_DIR)
        .join(DEFAULT_CONFIG_FILE))
}

/// Whether the server's config file exists yet. False on a fresh install —
/// the (non-headless) startup uses this to open the WebUI on the help page
/// instead of waiting for the user to find the tray icon.
pub fn config_file_exists() -> bool {
    config_file_path()
        .map(|path| path.exists())
        .unwrap_or(false)
}

/// The `server.log_to_file` setting as persisted on disk. Tracing is
/// initialized before `ConfigManager::load()`, so startup probes the file
/// synchronously through this instead; a missing/unreadable file or field
/// falls back to the build flavor's default ([`default_log_to_file`]).
pub fn log_to_file_enabled() -> bool {
    #[derive(serde::Deserialize)]
    struct LogToFileProbe {
        #[serde(default)]
        server: LogToFileProbeServer,
    }

    #[derive(Default, serde::Deserialize)]
    struct LogToFileProbeServer {
        #[serde(default = "default_log_to_file")]
        log_to_file: bool,
    }

    let content = config_file_path()
        .ok()
        .and_then(|path| std::fs::read_to_string(path).ok());
    content
        .and_then(|content| serde_json::from_str::<LogToFileProbe>(&content).ok())
        .map(|probe| probe.server.log_to_file)
        .unwrap_or_else(default_log_to_file)
}

fn user_home_dir() -> Result<PathBuf, ConfigError> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| {
            ConfigError::Validation("unable to determine user home directory".to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{ConfigPatchRequest, DeviceConfigPatch, TerminalServerConfig};

    #[tokio::test]
    async fn load_creates_default_config_file() {
        let path = unique_test_config_path("default");
        let _ = tokio::fs::remove_file(&path).await;

        let manager = ConfigManager::load_from_path(path.clone()).await.unwrap();

        let config = manager.snapshot().await;
        assert!(!config.server.server_id.is_empty());
        assert!(!config.server.name.is_empty()); // name defaults to hostname
        assert!(!config.server.secret_key.is_empty());
        let mut expected = TerminalServerConfig::default();
        // Match the platform-adjusted first-startup defaults (e.g. WSL ports).
        crate::config::types::apply_platform_default_ports(&mut expected);
        expected.server.server_id = config.server.server_id.clone();
        expected.server.name = config.server.name.clone();
        expected.server.secret_key = config.server.secret_key.clone();
        // The MCP access token is randomly generated per config, so mirror the
        // persisted one instead of comparing two fresh defaults.
        expected.mcp.access_token = config.mcp.access_token.clone();
        assert!(!config.mcp.access_token.is_empty());
        assert_eq!(config, expected);
        assert!(tokio::fs::try_exists(&path).await.unwrap());

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[test]
    fn default_server_name_tags_wsl_prefix() {
        let name = default_server_name(true);
        assert!(
            name.starts_with("WSL-") && name.len() > "WSL-".len(),
            "WSL default name should be `WSL-<hostname>`, got {:?}",
            name
        );
    }

    #[test]
    fn default_server_name_plain_off_wsl() {
        let name = default_server_name(false);
        assert!(!name.is_empty());
        assert!(!name.starts_with("WSL-"));
    }

    #[tokio::test]
    async fn load_migrates_mcp_disabled_by_old_default() {
        let path = unique_test_config_path("mcp-migration");
        let _ = tokio::fs::remove_file(&path).await;
        // A config persisted while the MCP default was `false` (version 1).
        tokio::fs::write(&path, r#"{"version": 1, "mcp": {"enabled": false}}"#)
            .await
            .unwrap();

        let manager = ConfigManager::load_from_path(path.clone()).await.unwrap();
        let config = manager.snapshot().await;
        assert!(
            config.mcp.enabled,
            "MCP must be flipped on by the migration"
        );
        assert_eq!(config.version, 2);

        // The migrated state is persisted.
        let saved = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(saved.contains("\"version\": 2"));

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[tokio::test]
    async fn load_respects_mcp_opt_out_after_migration() {
        let path = unique_test_config_path("mcp-opt-out");
        let _ = tokio::fs::remove_file(&path).await;
        // Version 2 carries the current defaults: an explicit opt-out is
        // a deliberate choice and must survive a reload.
        tokio::fs::write(&path, r#"{"version": 2, "mcp": {"enabled": false}}"#)
            .await
            .unwrap();

        let manager = ConfigManager::load_from_path(path.clone()).await.unwrap();
        assert!(!manager.snapshot().await.mcp.enabled);

        let _ = tokio::fs::remove_file(&path).await;
    }

    /// Configs persisted by older builds (whose token default was an empty
    /// string) are repaired with a freshly generated random token, matching
    /// what a fresh install ships — and the repair is persisted.
    #[tokio::test]
    async fn load_generates_mcp_token_when_missing_or_empty() {
        for (label, body) in [
            ("empty", r#"{"version": 2, "mcp": {"access_token": ""}}"#),
            ("missing", r#"{"version": 2, "mcp": {"port": 7880}}"#),
        ] {
            let path = unique_test_config_path("mcp-token-repair");
            let _ = tokio::fs::remove_file(&path).await;
            tokio::fs::write(&path, body).await.unwrap();

            let manager = ConfigManager::load_from_path(path.clone()).await.unwrap();
            let token = manager.snapshot().await.mcp.access_token;
            assert!(!token.trim().is_empty(), "{label}: token must be generated");
            assert_eq!(token.len(), 32, "{label}: generated token length");

            // The generated token is persisted so it survives a restart.
            let saved = tokio::fs::read_to_string(&path).await.unwrap();
            assert!(saved.contains(&token), "{label}: token must be persisted");

            let _ = tokio::fs::remove_file(&path).await;
        }
    }

    #[tokio::test]
    async fn apply_patch_persists_config_changes() {
        let path = unique_test_config_path("persist");
        let _ = tokio::fs::remove_file(&path).await;

        let manager = ConfigManager::load_from_path(path.clone()).await.unwrap();
        manager
            .apply_patch(ConfigPatchRequest {
                device: Some(DeviceConfigPatch {
                    serial_baud_rate: Some(460_800),
                    ..DeviceConfigPatch::default()
                }),
                ..ConfigPatchRequest::default()
            })
            .await
            .unwrap();

        let saved = tokio::fs::read_to_string(&path).await.unwrap();
        let saved_config: TerminalServerConfig = serde_json::from_str(&saved).unwrap();
        assert_eq!(saved_config.device.serial_baud_rate, 460_800);

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[tokio::test]
    async fn replace_persists_encrypted_server_secret_key() {
        let path = unique_test_config_path("secret");
        let _ = tokio::fs::remove_file(&path).await;

        let manager = ConfigManager::load_from_path(path.clone()).await.unwrap();
        let mut config = manager.snapshot().await;
        config.server.secret_key = "abcdefghijkl".to_string();
        manager.replace(config).await.unwrap();

        let saved = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(saved.contains("\"secret_key\":"));
        assert!(saved.contains("\"secret_key\": \"enc-v2:"));
        assert!(!saved.contains("abcdefghijkl"));

        let reloaded = ConfigManager::load_from_path(path.clone()).await.unwrap();
        assert_eq!(reloaded.snapshot().await.server.secret_key, "abcdefghijkl");

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[tokio::test]
    async fn replace_encrypts_every_addon_configuration_value() {
        let path = unique_test_config_path("all-addon-values");
        let _ = tokio::fs::remove_file(&path).await;
        let manager = ConfigManager::load_from_path(path.clone()).await.unwrap();
        let mut config = manager.snapshot().await;

        config.addons.entries.insert(
            "example-addon".to_string(),
            super::super::types::AddonEntryConfig {
                enabled: true,
                user_config: [("endpoint".to_string(), "https://example.test".to_string())]
                    .into_iter()
                    .collect(),
            },
        );
        config
            .ddns
            .user_config
            .insert("domain".to_string(), "example.test".to_string());
        config
            .llm
            .terminal
            .provider
            .user_config
            .insert("model".to_string(), "model-id".to_string());
        config
            .asr
            .provider_config
            .insert("language".to_string(), "en-US".to_string());
        config.asr.access_token = Some("asr-token".to_string());

        manager.replace(config).await.unwrap();
        let saved = tokio::fs::read_to_string(&path).await.unwrap();
        for value in [
            "https://example.test",
            "example.test",
            "model-id",
            "en-US",
            "asr-token",
        ] {
            assert!(
                !saved.contains(value),
                "plaintext value {value:?} was persisted"
            );
        }

        let reloaded = ConfigManager::load_from_path(path.clone()).await.unwrap();
        let reloaded = reloaded.snapshot().await;
        assert_eq!(reloaded.asr.effective_token().as_deref(), Some("asr-token"));
        assert!(reloaded
            .llm
            .terminal
            .provider
            .user_config
            .get("model")
            .unwrap()
            .starts_with("enc-v2:"));

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[tokio::test]
    async fn load_accepts_legacy_plaintext_server_secret_key() {
        let path = unique_test_config_path("legacy-secret");
        let _ = tokio::fs::remove_file(&path).await;

        let legacy = serde_json::json!({
            "version": 1,
            "server": {
                "relay_port": crate::config::types::DEFAULT_RELAY_PORT,
                "config_port": crate::config::types::DEFAULT_CONFIG_PORT,
                "proxy_port_start": crate::config::types::DEFAULT_PROXY_PORT_START,
                "proxy_port_end": crate::config::types::DEFAULT_PROXY_PORT_END,
                "name": "",
                "secret_key": "abcdefghijkl",
                "cert_path": "",
                "key_path": ""
            },
            "device": {
                "power_selection": "usb5v",
                "serial_port": "",
                "serial_baud_rate": 115200,
                "flash_gpio": null,
                "firmware_update_method": "sd"
            },
            "ddns": {
                "enabled": false,
                "provider": null,
                "api_key": null,
                "zone": null,
                "record_name": null,
                "record_type": "A",
                "update_ipv4": true,
                "update_ipv6": false,
                "polling_interval_minutes": 10,
                "custom_endpoint": null
            },
            "notification": {
                "targets": [],
                "apns": {
                    "enabled": false,
                    "certificate_path": null,
                    "certificate_password": null,
                    "environment": "sandbox",
                    "topic": null
                },
                "fcm": {
                    "enabled": false,
                    "service_account_json": null,
                    "project_id": null
                },
                "tpns": {
                    "enabled": false,
                    "access_id": null,
                    "secret_key": null
                }
            }
        });
        tokio::fs::write(&path, serde_json::to_vec_pretty(&legacy).unwrap())
            .await
            .unwrap();

        let manager = ConfigManager::load_from_path(path.clone()).await.unwrap();
        assert_eq!(manager.snapshot().await.server.secret_key, "abcdefghijkl");

        let config = manager.snapshot().await;
        manager.replace(config).await.unwrap();

        let migrated = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(migrated.contains("\"secret_key\": \"enc-v2:"));
        assert!(!migrated.contains("abcdefghijkl"));

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[tokio::test]
    async fn load_persists_generated_server_id_for_legacy_config() {
        let path = unique_test_config_path("server-id");
        let _ = tokio::fs::remove_file(&path).await;

        let legacy = serde_json::json!({
            "version": 1,
            "server": {
                "relay_port": crate::config::types::DEFAULT_RELAY_PORT,
                "config_port": crate::config::types::DEFAULT_CONFIG_PORT,
                "proxy_port_start": crate::config::types::DEFAULT_PROXY_PORT_START,
                "proxy_port_end": crate::config::types::DEFAULT_PROXY_PORT_END,
                "name": "",
                "secret_key": "",
                "relay_secret_key": "",
                "cert_path": "",
                "key_path": ""
            },
            "device": {
                "power_selection": "usb5v",
                "serial_port": "",
                "serial_baud_rate": 115200,
                "flash_gpio": null,
                "firmware_update_method": "sd"
            },
            "uart": {
                "ports": []
            },
            "ddns": {
                "enabled": false,
                "provider": null,
                "api_key": null,
                "zone": null,
                "record_name": null,
                "record_type": "A",
                "update_ipv4": true,
                "update_ipv6": false,
                "polling_interval_minutes": 10,
                "custom_endpoint": null
            },
            "notification": {
                "targets": [],
                "apns": {
                    "enabled": false,
                    "certificate_path": null,
                    "certificate_password": null,
                    "environment": "sandbox",
                    "topic": null
                },
                "fcm": {
                    "enabled": false,
                    "service_account_json": null,
                    "project_id": null
                },
                "tpns": {
                    "enabled": false,
                    "access_id": null,
                    "secret_key": null
                }
            },
            "terminal": {
                "enabled": true,
                "auto_confirm": false,
                "confirmation_grace_seconds": 30,
                "notify_on_completion": false,
                "patterns": []
            }
        });
        tokio::fs::write(&path, serde_json::to_vec_pretty(&legacy).unwrap())
            .await
            .unwrap();

        let manager = ConfigManager::load_from_path(path.clone()).await.unwrap();
        let config = manager.snapshot().await;
        assert!(!config.server.server_id.is_empty());
        assert!(!config.server.secret_key.is_empty());

        let saved = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(saved.contains("\"secret_key\": \"enc-v2:"));
        let saved_config: TerminalServerConfig = serde_json::from_str(&saved).unwrap();
        assert_eq!(saved_config.server.server_id, config.server.server_id);
        assert_eq!(saved_config.server.secret_key, config.server.secret_key);

        let _ = tokio::fs::remove_file(&path).await;
    }

    fn unique_test_config_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "vterm-server-config-{}-{}.json",
            name,
            std::process::id()
        ))
    }
}
