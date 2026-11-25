use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::addon::AddonHost;
use crate::config::types::{ConfigError, LlmProviderConfig, TerminalServerConfig};
use crate::config::{decrypt_addon_secret, encrypt_addon_secret};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebLlmProviderConfig {
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(default)]
    config: HashMap<String, String>,
    #[serde(default = "default_max_failures")]
    max_failures: u16,
    #[serde(default = "default_retry_cooldown_secs")]
    retry_cooldown_secs: u32,
}

impl Default for WebLlmProviderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: None,
            config: HashMap::new(),
            max_failures: default_max_failures(),
            retry_cooldown_secs: default_retry_cooldown_secs(),
        }
    }
}

fn default_max_failures() -> u16 {
    3
}

fn default_retry_cooldown_secs() -> u32 {
    300
}

fn default_output_language() -> String {
    "en".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebCodeAnalysisConfig {
    #[serde(flatten)]
    provider: WebLlmProviderConfig,
    #[serde(default = "default_output_language")]
    output_language: String,
}

impl Default for WebCodeAnalysisConfig {
    fn default() -> Self {
        Self {
            provider: WebLlmProviderConfig {
                enabled: false,
                provider: None,
                config: std::collections::HashMap::new(),
                max_failures: default_max_failures(),
                retry_cooldown_secs: default_retry_cooldown_secs(),
            },
            output_language: default_output_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebTranslationConfig {
    #[serde(flatten)]
    provider: WebLlmProviderConfig,
    #[serde(default = "default_output_language")]
    output_language: String,
}

impl Default for WebTranslationConfig {
    fn default() -> Self {
        Self {
            provider: WebLlmProviderConfig {
                enabled: false,
                provider: None,
                config: std::collections::HashMap::new(),
                max_failures: default_max_failures(),
                retry_cooldown_secs: default_retry_cooldown_secs(),
            },
            output_language: default_output_language(),
        }
    }
}

fn default_auto_output_language() -> String {
    "auto".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebTerminalConfig {
    #[serde(flatten)]
    provider: WebLlmProviderConfig,
    #[serde(default = "default_auto_output_language")]
    output_language: String,
}

impl Default for WebTerminalConfig {
    fn default() -> Self {
        Self {
            provider: WebLlmProviderConfig {
                enabled: false,
                provider: None,
                config: std::collections::HashMap::new(),
                max_failures: default_max_failures(),
                retry_cooldown_secs: default_retry_cooldown_secs(),
            },
            output_language: default_auto_output_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebGitConfig {
    #[serde(flatten)]
    provider: WebLlmProviderConfig,
    #[serde(default = "default_auto_output_language")]
    output_language: String,
}

impl Default for WebGitConfig {
    fn default() -> Self {
        Self {
            provider: WebLlmProviderConfig {
                enabled: false,
                provider: None,
                config: std::collections::HashMap::new(),
                max_failures: default_max_failures(),
                retry_cooldown_secs: default_retry_cooldown_secs(),
            },
            output_language: default_auto_output_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebLlmConfig {
    terminal: WebTerminalConfig,
    git: WebGitConfig,
    #[serde(default)]
    git_conflict: WebGitConfig,
    chat: WebLlmProviderConfig,
    #[serde(default)]
    voice_polish: WebLlmProviderConfig,
    #[serde(default)]
    code_analysis: WebCodeAnalysisConfig,
    #[serde(default)]
    translation: WebTranslationConfig,
}

pub(super) fn llm_config_to_web(
    config: &TerminalServerConfig,
    addon_host: &Arc<AddonHost>,
) -> WebLlmConfig {
    WebLlmConfig {
        terminal: WebTerminalConfig {
            provider: provider_to_web(&config.llm.terminal.provider, addon_host),
            output_language: config.llm.terminal.output_language.clone(),
        },
        git: WebGitConfig {
            provider: provider_to_web(&config.llm.git.provider, addon_host),
            output_language: config.llm.git.output_language.clone(),
        },
        git_conflict: WebGitConfig {
            provider: provider_to_web(&config.llm.git_conflict.provider, addon_host),
            output_language: config.llm.git_conflict.output_language.clone(),
        },
        chat: provider_to_web(&config.llm.chat, addon_host),
        voice_polish: provider_to_web(&config.llm.voice_polish, addon_host),
        code_analysis: WebCodeAnalysisConfig {
            provider: provider_to_web(&config.llm.code_analysis.provider, addon_host),
            output_language: config.llm.code_analysis.output_language.clone(),
        },
        translation: WebTranslationConfig {
            provider: provider_to_web(&config.llm.translation.provider, addon_host),
            output_language: config.llm.translation.output_language.clone(),
        },
    }
}

pub(super) fn apply_web_llm_config(
    config: &mut TerminalServerConfig,
    addon_host: &Arc<AddonHost>,
    payload: WebLlmConfig,
) -> Result<(), ConfigError> {
    apply_provider(
        &mut config.llm.terminal.provider,
        addon_host,
        payload.terminal.provider,
    )?;
    config.llm.terminal.output_language = payload.terminal.output_language;
    apply_provider(
        &mut config.llm.git.provider,
        addon_host,
        payload.git.provider,
    )?;
    config.llm.git.output_language = payload.git.output_language;
    apply_provider(
        &mut config.llm.git_conflict.provider,
        addon_host,
        payload.git_conflict.provider,
    )?;
    config.llm.git_conflict.output_language = payload.git_conflict.output_language;
    apply_provider(&mut config.llm.chat, addon_host, payload.chat)?;
    apply_provider(
        &mut config.llm.voice_polish,
        addon_host,
        payload.voice_polish,
    )?;
    apply_provider(
        &mut config.llm.code_analysis.provider,
        addon_host,
        payload.code_analysis.provider,
    )?;
    config.llm.code_analysis.output_language = payload.code_analysis.output_language;
    apply_provider(
        &mut config.llm.translation.provider,
        addon_host,
        payload.translation.provider,
    )?;
    config.llm.translation.output_language = payload.translation.output_language;
    Ok(())
}

fn provider_to_web(
    provider: &LlmProviderConfig,
    addon_host: &Arc<AddonHost>,
) -> WebLlmProviderConfig {
    WebLlmProviderConfig {
        enabled: provider.enabled,
        provider: provider.provider.clone(),
        config: decrypt_provider_config(provider, addon_host),
        max_failures: provider.max_failures,
        retry_cooldown_secs: provider.retry_cooldown_secs,
    }
}

fn apply_provider(
    config: &mut LlmProviderConfig,
    addon_host: &Arc<AddonHost>,
    payload: WebLlmProviderConfig,
) -> Result<(), ConfigError> {
    config.enabled = payload.enabled;
    config.provider = payload.provider.filter(|v| !v.trim().is_empty());
    config.user_config =
        encrypt_provider_config(config.provider.as_deref(), addon_host, payload.config)?;
    config.max_failures = payload.max_failures.max(3);
    config.retry_cooldown_secs = payload.retry_cooldown_secs.max(10);
    Ok(())
}

fn decrypt_provider_config(
    provider: &LlmProviderConfig,
    addon_host: &Arc<AddonHost>,
) -> HashMap<String, String> {
    let Some(provider_id) = provider.provider.as_deref() else {
        return provider.user_config.clone();
    };

    let Some(_) = addon_host.get_instance(provider_id) else {
        return provider.user_config.clone();
    };

    provider
        .user_config
        .iter()
        .map(|(key, value)| {
            let decrypted = if value.starts_with("enc-v2:") {
                decrypt_addon_secret(value).unwrap_or_else(|_| value.clone())
            } else {
                value.clone()
            };
            (key.clone(), decrypted)
        })
        .collect()
}

fn encrypt_provider_config(
    provider_id: Option<&str>,
    addon_host: &Arc<AddonHost>,
    user_config: HashMap<String, String>,
) -> Result<HashMap<String, String>, ConfigError> {
    let Some(provider_id) = provider_id else {
        return Ok(user_config);
    };

    let Some(_) = addon_host.get_instance(provider_id) else {
        return Ok(user_config);
    };

    let mut encrypted_config = HashMap::new();
    for (key, value) in user_config {
        if !value.is_empty() && !value.starts_with("enc-v2:") {
            let encrypted = encrypt_addon_secret(&value).map_err(|error| {
                ConfigError::Validation(format!("failed to encrypt LLM config: {}", error))
            })?;
            encrypted_config.insert(key, encrypted);
        } else {
            encrypted_config.insert(key, value);
        }
    }

    Ok(encrypted_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_llm_config_encrypts_password_fields_at_rest() {
        let addon_host = Arc::new(AddonHost::new());
        let mut config = TerminalServerConfig::default();

        let mut user_config = HashMap::new();
        user_config.insert("api_key".to_string(), "secret-token".to_string());
        user_config.insert(
            "base_url".to_string(),
            "https://api.example.com/v1".to_string(),
        );
        user_config.insert("model".to_string(), "gpt-test".to_string());

        apply_web_llm_config(
            &mut config,
            &addon_host,
            WebLlmConfig {
                terminal: WebTerminalConfig {
                    provider: WebLlmProviderConfig {
                        enabled: true,
                        provider: Some("openai-llm".to_string()),
                        config: user_config,
                        max_failures: 3,
                        retry_cooldown_secs: 300,
                    },
                    output_language: default_auto_output_language(),
                },
                git: WebGitConfig::default(),
                git_conflict: WebGitConfig::default(),
                chat: WebLlmProviderConfig {
                    enabled: false,
                    provider: None,
                    config: HashMap::new(),
                    max_failures: 3,
                    retry_cooldown_secs: 300,
                },
                voice_polish: WebLlmProviderConfig::default(),
                code_analysis: WebCodeAnalysisConfig::default(),
                translation: WebTranslationConfig::default(),
            },
        )
        .unwrap();

        let stored = config
            .llm
            .terminal
            .provider
            .user_config
            .get("api_key")
            .unwrap();
        assert!(stored.starts_with("enc-v2:"));
        assert!(config
            .llm
            .terminal
            .provider
            .user_config
            .get("base_url")
            .unwrap()
            .starts_with("enc-v2:"));

        let web = llm_config_to_web(&config, &addon_host);
        assert_eq!(
            web.terminal.provider.config.get("api_key").unwrap(),
            "secret-token"
        );
        assert_eq!(
            web.terminal.provider.config.get("model").unwrap(),
            "gpt-test"
        );
    }
}
