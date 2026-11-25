use std::collections::HashMap;
use std::time::{Duration, Instant};

use rquickjs::prelude::Opt;
use rquickjs::{CatchResultExt, CaughtError, Context, Ctx, Function, Object, Runtime};
use tracing::{debug, warn};

use super::llm_stats::{
    duration_millis, estimate_tokens, extract_usage_tokens, record_llm_request,
    unix_timestamp_millis, LlmRequestContext, LlmRequestStats,
};
use super::AddonHost;
use crate::config::{decrypt_addon_secret, TerminalServerConfig};

#[derive(Debug, Clone, Default)]
pub struct ChatCompletionResult {
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
    pub raw_body: Option<String>,
}

/// Resolve the LLM addon ID for a given feature from the config.
pub fn resolve_llm_addon_id(config: &TerminalServerConfig, feature: &str) -> Option<String> {
    let provider_config = match feature {
        "terminal" => &config.llm.terminal.provider,
        "git" => &config.llm.git.provider,
        "git_conflict" => &config.llm.git_conflict.provider,
        "chat" => &config.llm.chat,
        "voice_polish" => &config.llm.voice_polish,
        "code_analysis" => &config.llm.code_analysis.provider,
        "translation" => &config.llm.translation.provider,
        _ => return None,
    };

    if !provider_config.enabled {
        return None;
    }

    provider_config
        .provider
        .clone()
        .filter(|p| !p.trim().is_empty())
}

/// Resolve config for an LLM addon, decrypting secrets.
///
/// When feature-level config is provided, it is the source of truth. The legacy
/// addon config entry is only used as a fallback for older callers.
pub fn resolve_addon_config(
    config: &TerminalServerConfig,
    addon_id: &str,
    feature_config: Option<&HashMap<String, String>>,
) -> HashMap<String, String> {
    if let Some(feature_config) = feature_config {
        return decrypt_config_values(feature_config);
    }

    let mut result = HashMap::new();
    if let Some(entry) = config.addons.entries.get(addon_id) {
        result = decrypt_config_values(&entry.user_config);
    }
    result
}

fn decrypt_config_values(values: &HashMap<String, String>) -> HashMap<String, String> {
    values
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

/// Run `chat_completion(messages, options)` on an LLM addon via QuickJS.
/// `config_overrides` is the feature-level provider config to expose to the addon.
/// HTTP requests made by the addon resolve the current system proxy per call
/// (see `addon::http`), so a proxy toggled while the server runs just works.
pub fn run_chat_completion(
    addon_host: &AddonHost,
    config: &TerminalServerConfig,
    addon_id: &str,
    messages_json: &str,
    options_json: &str,
    config_overrides: Option<&HashMap<String, String>>,
    timeout: Option<Duration>,
    stats_context: Option<&LlmRequestContext>,
) -> ChatCompletionResult {
    let requested_at_ms = unix_timestamp_millis();
    let started_at = Instant::now();
    let source = match addon_host.get_source(addon_id) {
        Some(s) => s,
        None => {
            let result = ChatCompletionResult {
                success: false,
                error: Some(format!("addon not found: {}", addon_id)),
                ..Default::default()
            };
            record_completion_stats(
                addon_id,
                "",
                stats_context,
                requested_at_ms,
                started_at.elapsed(),
                messages_json,
                options_json,
                None,
                &result,
            );
            return result;
        }
    };

    let config_values = resolve_addon_config(config, addon_id, config_overrides);
    let configured_model = config_values
        .get("model")
        .or_else(|| config_values.get("llm_model"))
        .cloned()
        .unwrap_or_default();

    // Per-request timeout: the addon's own `timeout` config field (seconds)
    // is the user-tunable authority — every HTTP request this addon makes
    // uses it. When the user has not stored a value, the manifest default
    // applies (e.g. 60s for openai-llm). Addons without the field keep the
    // caller-provided timeout, so their behavior is unchanged.
    let request_timeout = config_values
        .get("timeout")
        .cloned()
        .or_else(|| addon_host.config_field_default(addon_id, "timeout"))
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map(Duration::from_secs)
        .or(timeout);

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(error) => {
            let result = ChatCompletionResult {
                success: false,
                error: Some(format!("failed to create JS runtime: {}", error)),
                ..Default::default()
            };
            record_completion_stats(
                addon_id,
                &configured_model,
                stats_context,
                requested_at_ms,
                started_at.elapsed(),
                messages_json,
                options_json,
                None,
                &result,
            );
            return result;
        }
    };

    let ctx = match Context::full(&rt) {
        Ok(ctx) => ctx,
        Err(error) => {
            let result = ChatCompletionResult {
                success: false,
                error: Some(format!("failed to create JS context: {}", error)),
                ..Default::default()
            };
            record_completion_stats(
                addon_id,
                &configured_model,
                stats_context,
                requested_at_ms,
                started_at.elapsed(),
                messages_json,
                options_json,
                None,
                &result,
            );
            return result;
        }
    };

    let result = ctx.with(
        |ctx| -> (ChatCompletionResult, Option<serde_json::Value>) {
        let addon_obj = match Object::new(ctx.clone()) {
            Ok(obj) => obj,
            Err(error) => {
                return (
                    ChatCompletionResult {
                        success: false,
                        error: Some(format!("failed to create addon object: {}", error)),
                        ..Default::default()
                    },
                    None,
                );
            }
        };

        // addon.get_config(key) — single-key lookup
        let config_for_cb = config_values.clone();
        addon_obj
            .set(
                "get_config",
                Function::new(ctx.clone(), move |key: String| -> Option<String> {
                    config_for_cb.get(&key).cloned()
                }),
            )
            .ok();

        // addon.http_request(method, url, headers, body)
        addon_obj
            .set(
                "http_request",
                Function::new(
                    ctx.clone(),
                    move |_ctx: Ctx,
                          method: String,
                          url: String,
                          headers: Opt<Object>,
                          body: Option<String>|
                          -> rquickjs::Result<String> {
                        // Header pairs are read on the JS thread: rquickjs
                        // objects cannot cross the thread boundary.
                        let mut header_pairs = Vec::new();
                        if let Some(headers) = headers.0 {
                            let keys: Vec<String> =
                                headers.keys().filter_map(|k| k.ok()).collect();
                            for key in keys {
                                if let Ok(val) = headers.get::<_, String>(&key) {
                                    header_pairs.push((key, val));
                                }
                            }
                        }
                        let request = super::http::AddonHttpRequest {
                            method,
                            url,
                            headers: header_pairs,
                            query: Vec::new(),
                            body: body.map(String::into_bytes),
                            timeout: request_timeout,
                        };

                        let (resp_tx, resp_rx) = std::sync::mpsc::channel();

                        std::thread::spawn(move || {
                            let rt = match tokio::runtime::Builder::new_current_thread()
                                .enable_io()
                                .enable_time()
                                .build()
                            {
                                Ok(rt) => rt,
                                Err(e) => {
                                    let _ = resp_tx.send(
                                        serde_json::json!({"status": 0, "body": format!("Failed to create runtime: {}", e)}).to_string()
                                    );
                                    return;
                                }
                            };
                            let result = rt.block_on(async {
                                match super::http::send_addon_request(&request).await {
                                    Ok(resp) => {
                                        let status = resp.status().as_u16();
                                        match resp.bytes().await {
                                            Ok(bytes) => {
                                                let body_str =
                                                    String::from_utf8_lossy(&bytes).to_string();
                                                serde_json::json!({"status": status, "body": body_str}).to_string()
                                            }
                                            Err(e) => {
                                                serde_json::json!({"status": status, "body": format!("Failed to read body: {}", e)}).to_string()
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        serde_json::json!({"status": 0, "body": format!("HTTP request failed: {}", error)}).to_string()
                                    }
                                }
                            });
                            let _ = resp_tx.send(result);
                        });

                        match resp_rx.recv() {
                            Ok(result_str) => Ok(result_str),
                            Err(_) => Ok(
                                serde_json::json!({"status": 0, "body": "HTTP request channel closed"})
                                    .to_string(),
                            ),
                        }
                    },
                ),
            )
            .ok();

        // addon.log(msg)
        addon_obj
            .set(
                "log",
                Function::new(ctx.clone(), move |msg: String| {
                    debug!("[llm-addon] {}", msg);
                }),
            )
            .ok();

        ctx.globals().set("addon", addon_obj).ok();

        // Evaluate addon source
        if let Err(error) = ctx.eval::<(), _>(source.as_str()).catch(&ctx) {
            let error = format_js_error(error);
            warn!("LLM addon JS source evaluation failed: {}", error);
            let _ = ctx.eval::<(), _>("addon = undefined");
            ctx.run_gc();
            return (
                ChatCompletionResult {
                    success: false,
                    error: Some(format!("JS evaluation error: {}", error)),
                    ..Default::default()
                },
                None,
            );
        }

        // Build call expression
        // chat_completion() returns the raw response body string, no need to stringify again
        let call_expr = format!(
            "chat_completion({}, {})",
            messages_json, options_json
        );

        debug!(
            "[llm-executor] Calling chat_completion, messages_json length = {} bytes",
            messages_json.len()
        );

        let eval_result: Result<String, String> = (|| {
            let result_str: String = ctx
                .eval::<String, _>(call_expr.as_str())
                .catch(&ctx)
                .map_err(format_js_error)?;
            Ok(result_str)
        })();

        // Always clean up
        let _ = ctx.eval::<(), _>("addon = undefined");
        ctx.run_gc();

        match eval_result {
            Ok(result_str) => {
                debug!(
                    "[llm-executor] Raw response length = {} bytes",
                    result_str.len()
                );
                let parsed: serde_json::Value =
                    serde_json::from_str(&result_str).unwrap_or_default();

                // The openai-llm.js addon returns the parsed response body directly
                // which is the OpenAI API response format: { choices: [{ message: { content: "..." } }] }
                let content = parsed
                    .get("choices")
                    .and_then(|c| c.get(0))
                    .and_then(|c| c.get("message"))
                    .and_then(|m| m.get("content"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // If there's an error field
                let error = parsed
                    .get("error")
                    .and_then(|e| e.as_str())
                    .map(|s| s.to_string());

                if content.is_none() && error.is_none() {
                    warn!(
                        "LLM response has neither content nor error: {}",
                        &result_str[..result_str.len().min(500)]
                    );
                }

                (
                    ChatCompletionResult {
                        success: content.is_some(),
                        content,
                        error,
                        raw_body: Some(result_str),
                    },
                    Some(parsed),
                )
            }
            Err(error) => {
                warn!("LLM chat_completion failed: {}", error);
                (
                    ChatCompletionResult {
                        success: false,
                        error: Some(format!("JS execution error: {}", error)),
                        ..Default::default()
                    },
                    None,
                )
            }
        }
        },
    );

    rt.run_gc();
    let (result, response_json) = result;
    let response_model = response_json
        .as_ref()
        .and_then(|value| value.get("model"))
        .and_then(|value| value.as_str())
        .unwrap_or(&configured_model);
    record_completion_stats(
        addon_id,
        response_model,
        stats_context,
        requested_at_ms,
        started_at.elapsed(),
        messages_json,
        options_json,
        response_json.as_ref(),
        &result,
    );
    result
}

fn record_completion_stats(
    addon_id: &str,
    model_name: &str,
    stats_context: Option<&LlmRequestContext>,
    requested_at_ms: i64,
    duration: Duration,
    messages_json: &str,
    options_json: &str,
    response_json: Option<&serde_json::Value>,
    result: &ChatCompletionResult,
) {
    let Some(context) = stats_context else {
        return;
    };

    let (input_tokens, output_tokens, total_tokens, tokens_estimated) =
        if let Some((input, output, total)) = response_json.and_then(extract_usage_tokens) {
            (input, output, total, false)
        } else {
            let input =
                estimate_tokens(messages_json).saturating_add(estimate_tokens(options_json));
            let output = result
                .content
                .as_deref()
                .map(estimate_tokens)
                .unwrap_or_default();
            (input, output, input.saturating_add(output), true)
        };

    record_llm_request(&LlmRequestStats {
        addon_name: addon_id.to_string(),
        model_name: model_name.to_string(),
        category: context.category.clone(),
        requested_at_ms,
        duration_ms: duration_millis(duration),
        input_tokens,
        output_tokens,
        total_tokens,
        tokens_estimated,
        success: result.success,
        error: result.error.clone(),
    });
}

fn format_js_error(error: CaughtError<'_>) -> String {
    match error {
        CaughtError::Exception(exception) => exception.to_string(),
        CaughtError::Value(value) => {
            format!(
                "Exception generated by QuickJS: thrown {} value: {:?}",
                value.type_name(),
                value
            )
        }
        CaughtError::Error(error) => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_openai_llm_manifest_has_timeout_default() {
        let host = crate::addon::AddonHost::new();
        assert_eq!(
            host.config_field_default("openai-llm", "timeout"),
            Some("60".to_string())
        );
        assert_eq!(host.config_field_default("openai-llm", "nonexistent"), None);
        assert_eq!(host.config_field_default("no-such-addon", "timeout"), None);
    }

    #[test]
    fn resolve_llm_addon_id_returns_none_when_disabled() {
        let config = TerminalServerConfig::default();
        assert!(resolve_llm_addon_id(&config, "terminal").is_none());
        assert!(resolve_llm_addon_id(&config, "git").is_none());
        assert!(resolve_llm_addon_id(&config, "chat").is_none());
    }

    #[test]
    fn resolve_llm_addon_id_returns_provider_when_enabled() {
        let mut config = TerminalServerConfig::default();
        config.llm.terminal.provider.enabled = true;
        config.llm.terminal.provider.provider = Some("openai-llm".to_string());
        assert_eq!(
            resolve_llm_addon_id(&config, "terminal"),
            Some("openai-llm".to_string())
        );
    }

    #[test]
    fn resolve_addon_config_decrypts_secrets() {
        let mut config = TerminalServerConfig::default();
        let encrypted = crate::config::encrypt_addon_secret("my-secret-key").unwrap();
        let mut user_config = HashMap::new();
        user_config.insert("api_key".to_string(), encrypted);
        user_config.insert(
            "base_url".to_string(),
            "https://api.example.com".to_string(),
        );
        config.addons.entries.insert(
            "openai-llm".to_string(),
            crate::config::AddonEntryConfig {
                enabled: true,
                user_config,
            },
        );

        let resolved = resolve_addon_config(&config, "openai-llm", None);
        assert_eq!(resolved.get("api_key").unwrap(), "my-secret-key");
        assert_eq!(resolved.get("base_url").unwrap(), "https://api.example.com");
    }

    #[test]
    fn resolve_addon_config_uses_feature_config_when_provided() {
        let mut config = TerminalServerConfig::default();
        let mut addon_user_config = HashMap::new();
        addon_user_config.insert("api_key".to_string(), "addon-secret".to_string());
        config.addons.entries.insert(
            "openai-llm".to_string(),
            crate::config::AddonEntryConfig {
                enabled: true,
                user_config: addon_user_config,
            },
        );

        let encrypted = crate::config::encrypt_addon_secret("feature-secret").unwrap();
        let mut feature_config = HashMap::new();
        feature_config.insert("api_key".to_string(), encrypted);
        feature_config.insert("model".to_string(), "git-model".to_string());

        let resolved = resolve_addon_config(&config, "openai-llm", Some(&feature_config));
        assert_eq!(resolved.get("api_key").unwrap(), "feature-secret");
        assert_eq!(resolved.get("model").unwrap(), "git-model");
        assert!(!resolved.contains_key("base_url"));
    }
}
