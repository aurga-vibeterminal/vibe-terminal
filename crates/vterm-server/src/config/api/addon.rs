use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::addon::AddonHost;
use crate::config::types::{AddonEntryConfig, ConfigError, TerminalServerConfig};
use crate::config::{decrypt_addon_secret, encrypt_addon_secret};

use super::super::server::HttpResponse;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebAddonConfigField {
    key: String,
    label: String,
    field_type: String,
    required: bool,
    default_value: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebAddonInfo {
    id: String,
    name: String,
    description: String,
    author: String,
    email: String,
    date: String,
    addon_type: String,
    builtin: bool,
    /// True when the served source is a user-authored override (builtin) or
    /// an uploaded addon the user owns. Drives the "Modified"/"Uploaded"
    /// badge in the UI and the availability of "Reset".
    modified: bool,
    enabled: bool,
    config_fields: Vec<WebAddonConfigField>,
    user_config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebAddonDetail {
    #[serde(flatten)]
    info: WebAddonInfo,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebAddonConfigUpdate {
    enabled: bool,
    user_config: HashMap<String, String>,
}

fn field_type_to_string(ft: &crate::addon::AddonFieldType) -> String {
    match ft {
        crate::addon::AddonFieldType::Text => "text",
        crate::addon::AddonFieldType::Password => "password",
        crate::addon::AddonFieldType::Boolean => "boolean",
        crate::addon::AddonFieldType::Number => "number",
        crate::addon::AddonFieldType::Integer => "integer",
        crate::addon::AddonFieldType::Textarea => "textarea",
    }
    .to_string()
}

fn addon_to_web_info(
    instance: &crate::addon::AddonInstance,
    config: &TerminalServerConfig,
) -> WebAddonInfo {
    let entry = config.addons.entries.get(&instance.manifest.id);
    // Builtins default to enabled for the addon types that ship with the
    // product: LLM providers (chat / git-commit / code-analysis), app
    // launchers (claude-code, codex, ...), and the local ASR provider
    // (vibe-voice — the only builtin of type "asr" today). The user can still
    // disable any of them explicitly in Addon settings; this just makes them
    // show up out of the box without a manual opt-in.
    let is_default_enabled =
        instance.builtin && matches!(instance.manifest.addon_type.as_str(), "llm" | "app" | "asr");
    let enabled = entry
        .as_ref()
        .map(|e| e.enabled)
        .unwrap_or(is_default_enabled);

    // Decrypt secret values for the web API
    let mut user_config = HashMap::new();
    if let Some(entry) = entry {
        for (key, value) in &entry.user_config {
            let decrypted = if value.starts_with("enc-v2:") {
                decrypt_addon_secret(value).unwrap_or_else(|_| value.clone())
            } else {
                value.clone()
            };
            user_config.insert(key.clone(), decrypted);
        }
    }

    // Manifest defaults are NOT merged into user_config on this GET path:
    // the webui shows them as placeholders only, so a field's value must
    // reflect exactly what the user explicitly saved. Defaults are applied
    // at runtime by the executors (see AddonHost::config_field_default).

    WebAddonInfo {
        id: instance.manifest.id.clone(),
        name: instance.manifest.name.clone(),
        description: instance.manifest.description.clone(),
        author: instance.manifest.author.clone(),
        email: instance.manifest.email.clone(),
        date: instance.manifest.date.clone(),
        addon_type: instance.manifest.addon_type.clone(),
        builtin: instance.builtin,
        modified: instance.source_overridden,
        enabled,
        config_fields: instance
            .manifest
            .config_fields
            .iter()
            .map(|f| WebAddonConfigField {
                key: f.key.clone(),
                label: f.label.clone(),
                field_type: field_type_to_string(&f.field_type),
                required: f.required,
                default_value: f.default_value.clone(),
                description: f.description.clone(),
            })
            .collect(),
        user_config,
    }
}

pub(super) fn list_addons(
    addon_host: &Arc<AddonHost>,
    config: &TerminalServerConfig,
) -> Result<HttpResponse, ConfigError> {
    //     debug!("API: list_addons called");
    let addons: Vec<WebAddonInfo> = addon_host
        .instances()
        .iter()
        .map(|i| addon_to_web_info(i, config))
        .collect();
    //     info!("API: list_addons returning {} addon(s)", addons.len());
    Ok(HttpResponse::json(&addons)?)
}

pub(super) fn get_addon_detail(
    addon_host: &Arc<AddonHost>,
    config: &TerminalServerConfig,
    addon_id: &str,
) -> Result<HttpResponse, ConfigError> {
    debug!("API: get_addon_detail('{}')", addon_id);
    let instance = addon_host
        .get_instance(addon_id)
        .ok_or_else(|| ConfigError::BadRequest(format!("addon not found: {}", addon_id)))?;

    let info = addon_to_web_info(&instance, config);
    let detail = WebAddonDetail {
        info,
        source: instance.source.clone(),
    };
    Ok(HttpResponse::json(&detail)?)
}

pub(super) fn get_addon_source(
    addon_host: &Arc<AddonHost>,
    addon_id: &str,
) -> Result<HttpResponse, ConfigError> {
    debug!("API: get_addon_source('{}')", addon_id);
    let source = addon_host
        .get_source(addon_id)
        .ok_or_else(|| ConfigError::BadRequest(format!("addon not found: {}", addon_id)))?;
    Ok(HttpResponse::json(
        &serde_json::json!({ "source": source }),
    )?)
}

pub(super) fn apply_addon_config(
    config: &mut TerminalServerConfig,
    addon_host: &Arc<AddonHost>,
    addon_id: &str,
    payload: WebAddonConfigUpdate,
) -> Result<(), ConfigError> {
    info!(
        "API: apply_addon_config('{}') enabled={}, config_keys=[{}]",
        addon_id,
        payload.enabled,
        payload
            .user_config
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
    addon_host
        .get_instance(addon_id)
        .ok_or_else(|| ConfigError::BadRequest(format!("addon not found: {}", addon_id)))?;

    // Addon configuration is provider-owned and may contain credentials in
    // fields not declared as passwords. Encrypt every non-empty value.
    let mut encrypted_config = HashMap::new();
    for (key, value) in payload.user_config {
        if !value.is_empty() && !value.starts_with("enc-v2:") {
            debug!(
                "API: encrypting config field '{}' for addon '{}'",
                key, addon_id
            );
            let encrypted = encrypt_addon_secret(&value).map_err(|e| {
                ConfigError::Validation(format!("failed to encrypt addon config: {}", e))
            })?;
            encrypted_config.insert(key, encrypted);
        } else {
            encrypted_config.insert(key, value);
        }
    }

    config.addons.entries.insert(
        addon_id.to_string(),
        AddonEntryConfig {
            enabled: payload.enabled,
            user_config: encrypted_config,
        },
    );

    info!(
        "API: addon '{}' config saved: enabled={}, fields={}",
        addon_id,
        payload.enabled,
        config
            .addons
            .entries
            .get(addon_id)
            .map(|e| e.user_config.len())
            .unwrap_or(0)
    );
    Ok(())
}

pub(super) fn addon_id_from_path<'a>(path: &'a str, prefix: &str) -> Result<&'a str, ConfigError> {
    path.strip_prefix(prefix)
        .ok_or_else(|| ConfigError::BadRequest("invalid addon path".to_string()))
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebAddonSourceUpdate {
    pub source: String,
}

/// Save a user-authored source override. Returns the refreshed addon detail
/// (with the updated `source` and `modified: true`).
pub(super) fn save_addon_source(
    addon_host: &Arc<AddonHost>,
    config: &TerminalServerConfig,
    addon_id: &str,
    payload: WebAddonSourceUpdate,
) -> Result<HttpResponse, ConfigError> {
    info!(
        "API: save_addon_source('{}') {} bytes",
        addon_id,
        payload.source.len()
    );
    addon_host
        .save_source_override(addon_id, &payload.source)
        .map_err(ConfigError::BadRequest)?;

    let instance = addon_host
        .get_instance(addon_id)
        .ok_or_else(|| ConfigError::BadRequest(format!("addon not found: {}", addon_id)))?;
    let info = addon_to_web_info(&instance, config);
    let detail = WebAddonDetail {
        info,
        source: instance.source,
    };
    Ok(HttpResponse::json(&detail)?)
}

/// Drop a source override and restore the pristine builtin source.
pub(super) fn reset_addon_source(
    addon_host: &Arc<AddonHost>,
    config: &TerminalServerConfig,
    addon_id: &str,
) -> Result<HttpResponse, ConfigError> {
    info!("API: reset_addon_source('{}')", addon_id);
    addon_host
        .reset_source_override(addon_id)
        .map_err(ConfigError::BadRequest)?;

    let instance = addon_host
        .get_instance(addon_id)
        .ok_or_else(|| ConfigError::BadRequest(format!("addon not found: {}", addon_id)))?;
    let info = addon_to_web_info(&instance, config);
    let detail = WebAddonDetail {
        info,
        source: instance.source,
    };
    Ok(HttpResponse::json(&detail)?)
}

/// Install an addon shipped as a zip (`<name>.json` + `<name>.js`). The body
/// is the raw zip bytes (content-type `application/zip` or any binary).
pub(super) fn upload_addon_zip(
    addon_host: &Arc<AddonHost>,
    config: &TerminalServerConfig,
    body: &[u8],
) -> Result<HttpResponse, ConfigError> {
    if body.is_empty() {
        return Err(ConfigError::BadRequest("upload body is empty".to_string()));
    }
    info!("API: upload_addon_zip() {} bytes", body.len());
    let instance = addon_host
        .install_zip(body)
        .map_err(ConfigError::BadRequest)?;
    let info = addon_to_web_info(&instance, config);
    Ok(HttpResponse::json(&info)?)
}

/// Export an addon as a downloadable zip holding its manifest (`<id>.json`)
/// and source (`<id>.js`) — the same layout the upload endpoint accepts, so
/// the archive can be edited offline and installed again as-is.
pub(super) fn export_addon_zip(
    addon_host: &Arc<AddonHost>,
    addon_id: &str,
) -> Result<HttpResponse, ConfigError> {
    debug!("API: export_addon_zip('{}')", addon_id);
    let bytes = addon_host
        .export_addon_zip(addon_id)
        .map_err(ConfigError::BadRequest)?;
    Ok(HttpResponse::bytes_with_headers(
        "application/zip",
        &bytes,
        vec![(
            "Content-Disposition".to_string(),
            format!("attachment; filename=\"{addon_id}.zip\""),
        )],
    ))
}

/// Permanently remove an uploaded addon. Builtins are rejected.
pub(super) fn delete_addon(
    addon_host: &Arc<AddonHost>,
    addon_id: &str,
) -> Result<HttpResponse, ConfigError> {
    info!("API: delete_addon('{}')", addon_id);
    addon_host
        .delete_addon(addon_id)
        .map_err(ConfigError::BadRequest)?;
    Ok(HttpResponse::empty(204))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The GET /api/addons path must expose manifest defaults through the
    /// config fields' `defaultValue` (the webui renders it as the input's
    /// placeholder) but must NOT merge them into `user_config` — a field's
    /// value reflects exactly what the user explicitly saved.
    #[test]
    fn web_addon_info_keeps_defaults_out_of_user_config() {
        let host = crate::addon::AddonHost::new();
        let instance = host
            .get_instance("openai-llm")
            .expect("builtin openai-llm registered");
        let info = addon_to_web_info(
            &instance,
            &crate::config::types::TerminalServerConfig::default(),
        );

        assert!(
            !info.user_config.contains_key("timeout"),
            "manifest defaults must not be merged into user_config"
        );
        assert!(!info.user_config.contains_key("base_url"));
        assert_eq!(
            info.config_fields
                .iter()
                .find(|field| field.key == "timeout")
                .and_then(|field| field.default_value.clone()),
            Some("60".to_string()),
            "the DTO must expose defaultValue for the webui placeholder"
        );
    }
}
