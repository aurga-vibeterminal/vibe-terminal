//! Voice / ASR REST API handlers.
//!
//! vterm-server no longer runs an in-process ASR engine. All transcription,
//! model lifecycle, and configuration flows are proxied to the
//! locally-managed `vterm-asr-server` child process via [`AsrSupervisor`].
//! The supervisor owns the live endpoint (`http://<bind>:<port>`) and the
//! current bearer token; we borrow its shared HTTP client for every proxied
//! request so callers don't each spin up a throwaway `reqwest::Client`.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::super::server::{HttpRequest, HttpResponse};
use crate::addon::AddonHost;
use crate::config::store::ConfigManager;
use crate::config::types::ConfigError;
use crate::voice::{AsrSupervisor, VoicePolishContext, VoiceServiceState, VOICE_DISABLED_ERROR};
use vterm_common::SessionId;
use vterm_core::protocol::VoiceInputOptimizationContext;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebAsrServerConfig {
    port: u16,
    host: String,
    /// Root directory containing locally managed ASR models.
    model_path: String,
    #[serde(default)]
    model_id: String,
    #[serde(default)]
    model_relative_path: String,
    device: String,
    /// Local-only, write-only: a plaintext bearer token the web client sends
    /// to *set* the access token. Never serialized back out — clients that
    /// want to read the current value use `access_token_value`.
    #[serde(default, skip_serializing)]
    access_token: Option<String>,
    /// Local-only, read-only: the decrypted current access token, surfaced so
    /// the WebUI's Copy-to-clipboard affordance can hand the value to the
    /// user. The WebUI runs as the trusted admin console for vterm-server (it
    /// already sees relay secrets, MCP tokens, etc.), so this is the same
    /// trust level as the rest of the page. Ignored on update.
    #[serde(default, skip_deserializing)]
    access_token_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(super) enum WebAsrModelStatus {
    NotLoaded,
    Loading,
    Loaded { model_id: String, device: String },
    Error { message: String },
}

impl From<VoiceServiceState> for WebAsrModelStatus {
    fn from(state: VoiceServiceState) -> Self {
        match state {
            VoiceServiceState::NotLoaded => WebAsrModelStatus::NotLoaded,
            VoiceServiceState::Loading => WebAsrModelStatus::Loading,
            VoiceServiceState::Loaded { model_id, device } => {
                WebAsrModelStatus::Loaded { model_id, device }
            }
            VoiceServiceState::Error { message } => WebAsrModelStatus::Error { message },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebVoiceConfig {
    enabled: bool,
    load_on_startup: bool,
    provider: String,
    provider_config: HashMap<String, String>,
    server: WebAsrServerConfig,
    model_status: WebAsrModelStatus,
    available_devices: Vec<String>,
    asr_server_reachable: bool,
    input_device: String,
    /// Whether the voice text-polish LLM (`llm.voice_polish`) is configured
    /// and enabled. The WebUI uses this to enable/disable its Polish button.
    polish_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebVoiceConfigUpdate {
    enabled: bool,
    load_on_startup: bool,
    provider: String,
    provider_config: HashMap<String, String>,
    server: Option<WebAsrServerConfig>,
    input_device: Option<String>,
}

pub(super) async fn get_voice_config(
    config_manager: &Arc<ConfigManager>,
    voice_supervisor: &Arc<AsrSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    let config = config_manager.snapshot().await;
    let asr = &config.asr;

    // The provider field holds the selected ASR addon id directly (there's no
    // longer a "local" sentinel — the supervised vterm-voice is reached via
    // the vibe-voice builtin addon). Empty string means "no provider selected
    // yet" and the UI shows its empty state.
    let provider = asr.provider.clone();

    let configured_server = WebAsrServerConfig {
        port: asr.port,
        host: asr.bind_address.clone(),
        model_path: asr.model_path.clone(),
        model_id: asr.model_id.clone(),
        model_relative_path: asr.model_relative_path.clone(),
        device: if asr.device.is_empty() {
            "auto".to_string()
        } else {
            asr.device.clone()
        },
        access_token: None,
        access_token_value: asr.effective_token(),
    };

    // Reachability + live model status come from the supervisor's HTTP probe.
    let model_status = voice_supervisor.model_status().await;
    let asr_reachable = !matches!(model_status, VoiceServiceState::NotLoaded)
        || voice_supervisor.endpoint().await.is_some();

    // Polish service readiness mirrors resolve_llm_addon_id("voice_polish"):
    // enabled + a non-empty provider addon id. Computed here so the WebUI can
    // gate its Polish button without a second round-trip to /api/llm.
    let polish_enabled = config.llm.voice_polish.enabled
        && config.llm.voice_polish
            .provider
            .as_deref()
            .is_some_and(|p| !p.trim().is_empty());

    let web = WebVoiceConfig {
        enabled: asr.enabled,
        load_on_startup: asr.load_on_startup,
        provider,
        provider_config: asr
            .provider_config
            .iter()
            .map(|(key, value)| {
                let value =
                    crate::config::decrypt_addon_secret(value).unwrap_or_else(|_| value.clone());
                (key.clone(), value)
            })
            .collect(),
        server: configured_server,
        model_status: model_status.into(),
        available_devices: available_devices(),
        asr_server_reachable: asr_reachable,
        input_device: asr.input_device.clone(),
        polish_enabled,
    };

    Ok(HttpResponse::json(&web)?)
}

pub(super) async fn update_voice_config(
    config_manager: &Arc<ConfigManager>,
    voice_supervisor: &Arc<AsrSupervisor>,
    body: &[u8],
) -> Result<HttpResponse, ConfigError> {
    let payload: WebVoiceConfigUpdate = serde_json::from_slice(body)
        .map_err(|e| ConfigError::BadRequest(format!("invalid JSON body: {}", e)))?;

    let mut config = config_manager.snapshot().await;
    let old_model_path = config.asr.model_path.clone();
    let old_model_relative_path = config.asr.model_relative_path.clone();
    let old_device = config.asr.device.clone();

    config.asr.enabled = payload.enabled;
    config.asr.load_on_startup = payload.load_on_startup;
    config.asr.provider = payload.provider;
    config.asr.provider_config = payload
        .provider_config
        .into_iter()
        .map(|(key, value)| {
            if value.is_empty() || value.starts_with("enc-v2:") {
                Ok((key, value))
            } else {
                crate::config::types::encrypt_addon_secret(&value)
                    .map(|encrypted| (key, encrypted))
                    .map_err(|error| {
                        ConfigError::Validation(format!(
                            "failed to encrypt ASR provider config: {error}"
                        ))
                    })
            }
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    if let Some(ref server) = payload.server {
        config.asr.model_path = server.model_path.clone();
        config.asr.model_id = server.model_id.clone();
        config.asr.model_relative_path = server.model_relative_path.clone();
        config.asr.device = server.device.clone();
        config.asr.bind_address = if server.host.trim().is_empty() {
            crate::config::types::default_asr_bind_address()
        } else {
            server.host.clone()
        };
        config.asr.port = server.port;
        // access_token is write-only: presence in the payload means "replace".
        // An explicit empty string clears the token. `None` means "leave as-is".
        match &server.access_token {
            Some(value) => {
                if value.is_empty() {
                    config.asr.access_token = None;
                } else {
                    config.asr.access_token = Some(
                        crate::config::types::encrypt_addon_secret(value)
                            .unwrap_or_else(|_| value.clone()),
                    );
                }
            }
            None => { /* unchanged */ }
        }
    }
    if let Some(input_device) = payload.input_device {
        config.asr.input_device = input_device;
    }

    // Token resolution: AsrConfig::effective_token() now resolves from BOTH
    // asr.access_token (Local ASR Server card) and
    // provider_config["access_token"] (vibe-voice addon field, for remote
    // servers). No mirror write is needed here — the supervisor reads
    // effective_token() at spawn time and the addon reads its own field at
    // transcription time. When the Local card sets a token, it lands in
    // asr.access_token and wins; when the user edits the addon's token for a
    // remote server, effective_token() picks it up as a fallback.

    let _ = config_manager.replace(config.clone()).await;

    // Note on restart semantics: the supervisor subscribes to config changes
    // itself and restarts the child when bind/port/token change,
    // so we don't trigger anything explicitly here. We DO push a config
    // refresh (model path / device) to the running process so the change
    // takes effect without a restart when possible.
    let model_changed = old_model_path != config.asr.model_path
        || old_model_relative_path != config.asr.model_relative_path
        || old_device != config.asr.device;
    if (model_changed
        || !payload
            .server
            .as_ref()
            .map(|s| s.host.is_empty())
            .unwrap_or(true))
        && voice_supervisor.endpoint().await.is_some()
    {
        if let Some(ref server) = payload.server {
            if let Err(e) = forward_asr_config(voice_supervisor, server).await {
                tracing::warn!("Failed to push ASR config to server: {}", e);
            }
        }
    }

    get_voice_config(config_manager, voice_supervisor).await
}

pub(super) async fn get_voice_model_status(
    voice_supervisor: &Arc<AsrSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    let status: WebAsrModelStatus = voice_supervisor.model_status().await.into();
    Ok(HttpResponse::json(&status)?)
}

pub(super) async fn load_voice_model(
    _config_manager: &Arc<ConfigManager>,
    voice_supervisor: &Arc<AsrSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    proxy_asr_post(voice_supervisor, "/v1/model/load").await
}

pub(super) async fn unload_voice_model(
    voice_supervisor: &Arc<AsrSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    proxy_asr_post(voice_supervisor, "/v1/model/unload").await
}

pub(super) async fn transcribe(
    config_manager: &Arc<ConfigManager>,
    voice_supervisor: &Arc<AsrSupervisor>,
    request: &HttpRequest,
) -> Result<HttpResponse, ConfigError> {
    require_voice_enabled(config_manager).await?;

    // Forward the raw multipart body to the asr-server. The supervisor
    // already speaks the right endpoint + bearer token; we just relay the
    // bytes (the Content-Type header is in the multipart body, so we pass
    // the client's header through when present).
    let query = request
        .path
        .split('?')
        .nth(1)
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let url = voice_supervisor
        .url_for(&format!("/v1/audio/transcriptions{}", query))
        .await;
    let token = voice_supervisor.bearer_token().await;

    let content_type = request
        .headers
        .get("content-type")
        .cloned()
        .unwrap_or_else(|| "multipart/form-data".to_string());

    let mut req = voice_supervisor
        .http_client()
        .post(&url)
        .header("content-type", content_type)
        .body(request.body.clone());
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await.map_err(|e| {
        ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            e.to_string(),
        ))
    })?;

    relay_response(resp).await
}

pub(super) async fn transcribe_stream(
    config_manager: &Arc<ConfigManager>,
    voice_supervisor: &Arc<AsrSupervisor>,
    request: &HttpRequest,
) -> Result<HttpResponse, ConfigError> {
    require_voice_enabled(config_manager).await?;

    // For now this is a buffered proxy (the asr-server emits SSE; vterm-server
    // collects it and returns it as a single text body). True SSE passthrough
    // (reqwest stream feature) is a follow-up.
    let query = request
        .path
        .split('?')
        .nth(1)
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let url = voice_supervisor
        .url_for(&format!("/v1/audio/transcriptions/stream{}", query))
        .await;
    let token = voice_supervisor.bearer_token().await;

    let content_type = request
        .headers
        .get("content-type")
        .cloned()
        .unwrap_or_else(|| "multipart/form-data".to_string());

    let mut req = voice_supervisor
        .http_client()
        .post(&url)
        .header("content-type", content_type)
        .body(request.body.clone());
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await.map_err(|e| {
        ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            e.to_string(),
        ))
    })?;

    let status = resp.status().as_u16();
    let body = resp.bytes().await.map_err(|e| {
        ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    // The upstream returns `text/event-stream`; we forward it as text so the
    // SSE frames are preserved verbatim.
    Ok(HttpResponse::text(
        status,
        "OK",
        &String::from_utf8_lossy(&body),
    ))
}

/// Manual Start button from the WebUI: ensure the asr-server child is
/// running (spawning if necessary), then return current model status.
pub(super) async fn start_voice_server(
    config_manager: &Arc<ConfigManager>,
    voice_supervisor: &Arc<AsrSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    let config = config_manager.snapshot().await;
    let asr = config.asr.clone();
    if let Err(e) = voice_supervisor.ensure_running(&asr).await {
        tracing::warn!("start_voice_server: ensure_running failed: {e:?}");
        return Err(ConfigError::BadRequest(format!(
            "Failed to start local ASR server: {e}"
        )));
    }
    let status: WebAsrModelStatus = voice_supervisor.model_status().await.into();
    Ok(HttpResponse::json(&status)?)
}

/// Manual Stop button from the WebUI: request a local Voice API shutdown, then
/// tear down any supervised child as a fallback. Remote endpoints are never
/// sent a shutdown request.
pub(super) async fn stop_voice_server(
    voice_supervisor: &Arc<AsrSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    voice_supervisor.stop().await;
    let status: WebAsrModelStatus = voice_supervisor.model_status().await.into();
    Ok(HttpResponse::json(&status)?)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebPolishRequest {
    text: String,
}

/// One-click transcript polish for the WebUI (`POST /api/voice/polish`).
///
/// Reuses the exact same prompt + LLM pipeline as the native voice dialog
/// (see `crate::voice::polish`). The WebUI has no terminal to attach, so the
/// request carries a synthetic `Transparent` session id and an empty expanded
/// context, which selects the general (non-terminal) system prompt.
pub(super) async fn polish(
    config_manager: &Arc<ConfigManager>,
    addon_host: &Arc<AddonHost>,
    body: &[u8],
) -> Result<HttpResponse, ConfigError> {
    let payload: WebPolishRequest = serde_json::from_slice(body)
        .map_err(|e| ConfigError::BadRequest(format!("invalid JSON body: {}", e)))?;
    if payload.text.trim().is_empty() {
        return Ok(HttpResponse::json(&serde_json::json!({ "text": "" }))?);
    }

    let config = config_manager.snapshot().await;
    let addon_host = Arc::clone(addon_host);
    // run_chat_completion drives QuickJS synchronously; keep it off the async
    // runtime, same as the native PolishVoiceInputText path.
    let result = tokio::task::spawn_blocking(move || {
        crate::voice::polish_voice_input_text(
            &addon_host,
            &config,
            &payload.text,
            &VoiceInputOptimizationContext::transparent(SessionId::default()),
            &VoicePolishContext::general(String::new()),
        )
    })
    .await
    .map_err(|e| ConfigError::BadRequest(format!("voice polish task failed: {e}")))?
    .map_err(ConfigError::BadRequest)?;

    Ok(HttpResponse::json(&serde_json::json!({ "text": result }))?)
}

// ---------------------------------------------------------------------------
// Internal proxy helpers
// ---------------------------------------------------------------------------

async fn require_voice_enabled(config_manager: &Arc<ConfigManager>) -> Result<(), ConfigError> {
    if !config_manager.snapshot().await.asr.enabled {
        return Err(ConfigError::BadRequest(VOICE_DISABLED_ERROR.to_string()));
    }
    Ok(())
}

async fn forward_asr_config(
    voice_supervisor: &Arc<AsrSupervisor>,
    server: &WebAsrServerConfig,
) -> Result<(), String> {
    let url = voice_supervisor.url_for("/v1/config").await;
    let token = voice_supervisor.bearer_token().await;
    let body = serde_json::json!({
        "server": {
            "port": server.port,
            "host": server.host,
            "model_path": super::voice_models::resolve_model_path(
                &server.model_path,
                &server.model_relative_path,
            ),
            "device": server.device,
        }
    });
    let mut req = voice_supervisor.http_client().put(&url).json(&body);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    req.send().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn proxy_asr_post(
    voice_supervisor: &Arc<AsrSupervisor>,
    path: &str,
) -> Result<HttpResponse, ConfigError> {
    let url = voice_supervisor.url_for(path).await;
    let token = voice_supervisor.bearer_token().await;
    let mut req = voice_supervisor.http_client().post(&url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await.map_err(|e| {
        ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            e.to_string(),
        ))
    })?;
    relay_response(resp).await
}

async fn relay_response(resp: reqwest::Response) -> Result<HttpResponse, ConfigError> {
    let status = resp.status().as_u16();
    let resp_body = resp.bytes().await.map_err(|e| {
        ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    Ok(HttpResponse::json_with_status(status, resp_body.to_vec()))
}

fn available_devices() -> Vec<String> {
    if cfg!(target_os = "macos") {
        vec!["cpu".to_string(), "mlx".to_string()]
    } else {
        vec!["cpu".to_string(), "cuda".to_string(), "vulkan".to_string()]
    }
}
