//! Transcription entry point for voice addons.
//!
//! Voice addons differ from LLM addons in one crucial way: they receive raw
//! audio bytes. QuickJS strings are UTF-8 and rquickjs has no ergonomic
//! `Uint8Array` interop, so we keep binary audio **out of JS**. Instead the
//! addon's JS function returns a **descriptor** describing how to call the
//! provider, wrapped in a standard envelope:
//!
//! ```js
//! function transcribe_descriptor(audio, options) {
//!   // audio    = { filename, mime, sample_count, duration_s }
//!   // options  = {} (reserved; per-call overrides may land here later)
//!   // Per-provider settings (e.g. language) are read from the addon config
//!   // via addon.get_config("language") and folded into the descriptor.
//!   return JSON.stringify({
//!     code: 0,                          // 0 = success; nonzero = error
//!     descriptor: {
//!       url: "...",                     // absolute URL to POST to
//!       headers: { Authorization: "..." }, // optional
//!       response_text_path: "text",     // top-level JSON key whose value is the transcript
//!       query: { language: "..." },     // optional
//!     },
//!   });
//!   // On error: return JSON.stringify({ code: <nonzero>, error: "..." });
//! }
//! ```
//!
//! Rust parses the envelope, and on `code: 0` constructs the WAV + multipart
//! body from the audio samples, POSTs via the shared [`reqwest::Client`], and
//! pulls the transcript out of the response using `response_text_path`. On a
//! nonzero `code`, the addon's `error` string is surfaced verbatim. Adding a
//! new cloud STT provider is therefore a pure-JS change — no Rust recompile.
//!
//! This mirrors [`super::llm_executor::run_chat_completion`] structurally
//! (Runtime/Context setup, `addon` global with `get_config`/`http_request`/
//! `log`), so most of the boilerplate is shared in spirit. We don't reuse the
//! same closure to keep the two execution paths cleanly separable.

use std::collections::HashMap;
use std::time::Duration;

use rquickjs::prelude::Opt;
use rquickjs::{CatchResultExt, CaughtError, Context, Ctx, Function, Object, Runtime};
use tracing::{debug, warn};

use super::AddonHost;
use crate::config::TerminalServerConfig;
use crate::voice::wav;

#[derive(Debug, Clone, Default)]
pub struct TranscriptionResult {
    pub success: bool,
    pub text: Option<String>,
    pub error: Option<String>,
    pub raw_body: Option<String>,
}

/// What the JS addon returned: a description of how to call the voice
/// provider. Built inside [`build_descriptor`] by walking the rquickjs object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestUrlScheme {
    Http,
    Https,
    Ws,
    Wss,
}

/// Parse and classify the protocol rather than relying on reqwest's generic
/// scheme error. Descriptor execution is an HTTP POST operation, so WebSocket
/// endpoints are recognized but deliberately rejected.
fn classify_request_url(url: &str) -> Result<RequestUrlScheme, String> {
    let parsed =
        reqwest::Url::parse(url).map_err(|e| format!("invalid request URL '{url}': {e}"))?;
    match parsed.scheme() {
        "http" => Ok(RequestUrlScheme::Http),
        "https" => Ok(RequestUrlScheme::Https),
        "ws" => Ok(RequestUrlScheme::Ws),
        "wss" => Ok(RequestUrlScheme::Wss),
        scheme => Err(format!(
            "unsupported request URL scheme '{scheme}'; expected http, https, ws, or wss"
        )),
    }
}

fn require_http_request_url(url: &str) -> Result<(), String> {
    match classify_request_url(url)? {
        RequestUrlScheme::Http | RequestUrlScheme::Https => Ok(()),
        RequestUrlScheme::Ws | RequestUrlScheme::Wss => Err(format!(
            "WebSocket URL '{url}' is not supported by this HTTP request operation"
        )),
    }
}

#[derive(Debug, Clone, Default)]
struct TranscriptionDescriptor {
    url: String,
    headers: HashMap<String, String>,
    /// Top-level JSON key in the response whose value is the transcript.
    /// Defaults to `"text"` (the shape both vterm-asr-server and OpenAI's
    /// Whisper endpoint use).
    response_text_path: String,
    /// Optional query parameters to merge into the request URL.
    query: HashMap<String, String>,
}

/// Run a voice addon's `transcribe_descriptor(audio, options)` JS function,
/// then execute the described HTTP request with the supplied f32 samples.
///
/// `config_overrides` is the `AsrConfig::provider_config` map (feature-level
/// provider config, including any per-provider `language` field), decrypted on
/// the way in. `samples` is the raw 16kHz mono f32 audio (any sample rate is
/// acceptable — the WAV helper tags it as 16k).
///
/// Language is no longer a parameter here — it lives in the addon's config
/// (e.g. the vibe-voice addon declares a `language` field) and the JS
/// descriptor folds it into the request URL's query string.
pub fn run_transcription(
    addon_host: &AddonHost,
    config: &TerminalServerConfig,
    addon_id: &str,
    samples: &[f32],
    config_overrides: Option<&HashMap<String, String>>,
    timeout: Option<Duration>,
) -> TranscriptionResult {
    let source = match addon_host.get_source(addon_id) {
        Some(s) => s,
        None => {
            return TranscriptionResult {
                success: false,
                error: Some(format!("addon not found: {addon_id}")),
                ..Default::default()
            }
        }
    };

    let config_values =
        super::llm_executor::resolve_addon_config(config, addon_id, config_overrides);
    let timeout = timeout.unwrap_or_else(|| Duration::from_secs(180));

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(error) => {
            return TranscriptionResult {
                success: false,
                error: Some(format!("failed to create QuickJS runtime: {error}")),
                ..Default::default()
            }
        }
    };

    let ctx = match Context::full(&rt) {
        Ok(ctx) => ctx,
        Err(error) => {
            return TranscriptionResult {
                success: false,
                error: Some(format!("failed to create QuickJS context: {error}")),
                ..Default::default()
            };
        }
    };

    let result: TranscriptionResult = ctx.with(|ctx| -> TranscriptionResult {
        // Register the same `addon` global surface as run_chat_completion so a
        // voice addon can call out to e.g. a cloud provider to fetch a short
        // token before describing the transcription call.
        register_addon_global(&ctx, &config_values, timeout);

        // Eval the addon source, then call transcribe_descriptor(...).
        if let Err(error) = ctx.eval::<(), _>(source.as_str()).catch(&ctx) {
            return TranscriptionResult {
                success: false,
                error: Some(format!("addon eval failed: {}", format_caught(&error))),
                ..Default::default()
            };
        }

        let audio_meta = serde_json::json!({
            "filename": "audio.wav",
            "mime": "audio/wav",
            "sample_count": samples.len(),
            "duration_s": (samples.len() as f64) / 16000.0,
        });
        // `options` is currently empty — language and other per-provider
        // settings are read by the JS descriptor from the addon config, not
        // passed per-call. Kept as an empty object so the JS signature
        // `transcribe_descriptor(audio, options)` stays stable for future
        // per-call overrides.
        let options = serde_json::json!({});
        let call_expr = format!("transcribe_descriptor({}, {})", audio_meta, options);

        let envelope_str: String = match ctx.eval::<String, _>(call_expr.as_str()).catch(&ctx) {
            Ok(s) => s,
            Err(error) => {
                return TranscriptionResult {
                    success: false,
                    error: Some(format!(
                        "transcribe_descriptor failed: {}",
                        format_caught(&error)
                    )),
                    ..Default::default()
                }
            }
        };

        let descriptor = match parse_envelope(&envelope_str) {
            Ok(d) => d,
            Err(e) => {
                return TranscriptionResult {
                    success: false,
                    error: Some(e),
                    raw_body: Some(envelope_str),
                    ..Default::default()
                }
            }
        };

        // Rust owns the binary side: encode WAV, build multipart, POST. The
        // descriptor already carries any `?language=` query the JS layer
        // derived from the addon config, so we don't merge anything extra.
        let transcription = execute_descriptor(samples, &descriptor, timeout);
        let _ = ctx.eval::<(), _>("addon = undefined").catch(&ctx);
        ctx.run_gc();
        transcription
    });
    rt.run_gc();
    result
}

/// Execute the descriptor's HTTP request and pull the transcript out of the
/// response. Lives outside the rquickjs closure so it can be `async`-free
/// (it uses the blocking reqwest::blocking interface via a worker thread,
/// matching how `addon.http_request` works).
fn execute_descriptor(
    samples: &[f32],
    descriptor: &TranscriptionDescriptor,
    timeout: Duration,
) -> TranscriptionResult {
    if descriptor.url.trim().is_empty() {
        return TranscriptionResult {
            success: false,
            error: Some("addon descriptor missing 'url'".into()),
            ..Default::default()
        };
    }

    if let Err(error) = require_http_request_url(&descriptor.url) {
        return TranscriptionResult {
            success: false,
            error: Some(error),
            ..Default::default()
        };
    }

    let wav_bytes = wav::encode_pcm_f32_16k_mono(samples);
    let boundary = wav::random_multipart_boundary();
    let body = wav::build_multipart_file(&boundary, "file", "audio.wav", "audio/wav", &wav_bytes);

    let (tx, rx) = std::sync::mpsc::channel::<Result<(u16, String), String>>();
    let mut headers: Vec<(String, String)> = vec![(
        "content-type".to_string(),
        format!("multipart/form-data; boundary={boundary}"),
    )];
    headers.extend(
        descriptor
            .headers
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    let request = super::http::AddonHttpRequest {
        method: "POST".to_string(),
        url: descriptor.url.clone(),
        headers,
        // The descriptor's `query` already carries anything the JS layer
        // derived (e.g. `?language=zh` from the addon config) — passed
        // through as-is.
        query: descriptor
            .query
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
        body: Some(body),
        timeout: Some(timeout),
    };

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(Err(format!("failed to build tokio runtime: {e}")));
                return;
            }
        };
        let result = rt.block_on(async move {
            let resp = super::http::send_addon_request(&request)
                .await
                .map_err(|e| format_reqwest_error(&e))?;
            let status = resp.status().as_u16();
            let body_text = resp
                .text()
                .await
                .map_err(|e| format!("response read failed: {}", format_reqwest_error(&e)))?;
            Ok((status, body_text))
        });
        let _ = tx.send(result);
    });

    let outcome = rx
        .recv()
        .map_err(|e| format!("addon http worker dropped: {e}"))
        .and_then(|r| r);
    match outcome {
        Ok((status, body)) => {
            if !(200..300).contains(&status) {
                return TranscriptionResult {
                    success: false,
                    error: Some(format!("HTTP {status}")),
                    raw_body: Some(body),
                    ..Default::default()
                };
            }
            let parsed: serde_json::Value = match serde_json::from_str(&body) {
                Ok(v) => v,
                Err(_) => {
                    // Some providers return plain text; accept it verbatim.
                    return TranscriptionResult {
                        success: true,
                        text: Some(body.clone()),
                        raw_body: Some(body),
                        ..Default::default()
                    };
                }
            };
            let text = parsed
                .get(&descriptor.response_text_path)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let success = text.is_some();
            let error = if success {
                None
            } else {
                Some(format!(
                    "response missing key '{}'",
                    descriptor.response_text_path
                ))
            };
            TranscriptionResult {
                success,
                text,
                error,
                raw_body: Some(body),
            }
        }
        Err(e) => TranscriptionResult {
            success: false,
            error: Some(e),
            ..Default::default()
        },
    }
}

/// Register the `addon` global object with `get_config` / `http_request` /
/// `log` — the same surface as `run_chat_completion`, so voice addons can use
/// identical config plumbing and make auxiliary HTTP calls.
fn register_addon_global(
    ctx: &Ctx,
    config_values: &HashMap<String, String>,
    request_timeout: Duration,
) {
    let addon_obj = Object::new(ctx.clone()).expect("addon object");

    let config_for_cb = config_values.clone();
    addon_obj
        .set(
            "get_config",
            Function::new(ctx.clone(), move |key: String| -> Option<String> {
                config_for_cb.get(&key).cloned()
            }),
        )
        .ok();

    let timeout_for_cb = request_timeout;
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
                    blocking_http_request(&method, &url, headers, body, timeout_for_cb)
                },
            ),
        )
        .ok();

    addon_obj
        .set(
            "log",
            Function::new(ctx.clone(), move |msg: String| {
                debug!("[voice-addon] {}", msg);
            }),
        )
        .ok();

    ctx.globals().set("addon", addon_obj).ok();
}

/// Synchronous HTTP helper for voice addons. Same shape as the LLM addon's
/// `addon.http_request` — builds the reqwest RequestBuilder on the JS thread
/// (so it can read the rquickjs `Object` headers), then sends it on a worker
/// thread so the call is `await`-free from JS. Returns
/// `{"status": u16, "body": String}` as a JSON string.
fn blocking_http_request(
    method: &str,
    url: &str,
    headers: Opt<Object>,
    body: Option<String>,
    timeout: Duration,
) -> rquickjs::Result<String> {
    if let Err(error) = require_http_request_url(url) {
        return Ok(serde_json::json!({"status": 0, "body": error}).to_string());
    }

    // Read header pairs from the rquickjs Object on this JS thread before
    // crossing the thread boundary.
    let mut header_pairs = Vec::new();
    if let Some(headers_obj) = headers.0 {
        let keys: Vec<String> = headers_obj.keys().filter_map(|k| k.ok()).collect();
        for key in keys {
            if let Ok(val) = headers_obj.get::<_, String>(&key) {
                header_pairs.push((key, val));
            }
        }
    }
    let request = super::http::AddonHttpRequest {
        method: method.to_string(),
        url: url.to_string(),
        headers: header_pairs,
        query: Vec::new(),
        body: body.map(String::into_bytes),
        timeout: Some(timeout),
    };

    let (resp_tx, resp_rx) = std::sync::mpsc::channel::<String>();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = resp_tx.send(
                    serde_json::json!({"status": 0, "body": format!("failed to create runtime: {e}")})
                        .to_string(),
                );
                return;
            }
        };
        let result = rt.block_on(async move {
            match super::http::send_addon_request(&request).await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    match resp.bytes().await {
                        Ok(bytes) => {
                            let body_str = String::from_utf8_lossy(&bytes).to_string();
                            serde_json::json!({"status": status, "body": body_str}).to_string()
                        }
                        Err(e) => serde_json::json!({
                            "status": status,
                            "body": format!("failed to read body: {e}")
                        })
                        .to_string(),
                    }
                }
                Err(error) => serde_json::json!({
                    "status": 0,
                    "body": format!("HTTP request failed: {error}")
                })
                .to_string(),
            }
        });
        let _ = resp_tx.send(result);
    });

    match resp_rx.recv() {
        Ok(s) => Ok(s),
        Err(_) => Ok(serde_json::json!({
            "status": 0,
            "body": "HTTP request channel closed"
        })
        .to_string()),
    }
}

/// Parse the descriptor JSON returned by the addon's `transcribe_descriptor`
/// function. Tolerates missing optional fields.
/// Parse the addon's return envelope:
///
/// ```jsonc
/// { "code": 0, "descriptor": { ... } }   // success — descriptor executed by Rust
/// { "code": <nonzero>, "error": "..." }  // failure — error surfaced verbatim
/// ```
///
/// On a nonzero code, returns `Err(error_message)` so the caller can stamp it
/// into `TranscriptionResult.error`. On `code: 0`, returns
/// `Ok(descriptor)`. Malformed envelopes (missing code, missing descriptor on
/// success, missing error on failure, non-integer code) return `Err` with a
/// diagnostic that includes a snippet of the offending payload.
fn parse_envelope(s: &str) -> Result<TranscriptionDescriptor, String> {
    let v: serde_json::Value =
        serde_json::from_str(s).map_err(|e| format!("addon return is not valid JSON: {e}"))?;
    let obj = v
        .as_object()
        .ok_or_else(|| "addon return must be a JSON object".to_string())?;

    // `code` is required and must be an integer. Strings like "0" are rejected
    // — the contract is a numeric code.
    let code = obj
        .get("code")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| "addon return missing numeric 'code'".to_string())?;

    if code != 0 {
        let error = obj
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("(addon reported an error with no message)");
        return Err(format!("addon error (code={code}): {error}"));
    }

    // Success — extract the descriptor.
    let descriptor_value = obj
        .get("descriptor")
        .ok_or_else(|| "addon return code=0 but missing 'descriptor'".to_string())?;
    parse_descriptor_from_value(descriptor_value)
}

/// Parse the inner `descriptor` object from an already-parsed JSON value. Used
/// by [`parse_envelope`] and by the tests.
fn parse_descriptor_from_value(v: &serde_json::Value) -> Result<TranscriptionDescriptor, String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "descriptor must be a JSON object".to_string())?;
    let url = obj
        .get("url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "descriptor missing non-empty 'url'".to_string())?
        .to_string();
    let response_text_path = obj
        .get("response_text_path")
        .and_then(|v| v.as_str())
        .unwrap_or("text")
        .to_string();
    let mut headers = HashMap::new();
    if let Some(h) = obj.get("headers").and_then(|v| v.as_object()) {
        for (k, v) in h {
            if let Some(s) = v.as_str() {
                headers.insert(k.clone(), s.to_string());
            }
        }
    }
    let mut query = HashMap::new();
    if let Some(q) = obj.get("query").and_then(|v| v.as_object()) {
        for (k, v) in q {
            if let Some(s) = v.as_str() {
                query.insert(k.clone(), s.to_string());
            }
        }
    }
    Ok(TranscriptionDescriptor {
        url,
        headers,
        response_text_path,
        query,
    })
}

fn format_caught(error: &CaughtError) -> String {
    // CaughtError implements Display; this is the same approach llm_executor
    // takes (see llm_executor.rs ~L309).
    format!("{error}")
}

/// Format a reqwest error with enough context to diagnose a connection
/// failure: the request URL, the high-level error category (connect / timeout
/// / body / etc.), and the underlying io / hyper source if present. Without
/// this the caller only sees "error sending request for url (...)" which
/// hides whether it was a refused connection, DNS, TLS, or a body read.
fn format_reqwest_error(e: &reqwest::Error) -> String {
    let mut out = String::new();
    if let Some(url) = e.url() {
        out.push_str(&format!("for {url} "));
    }
    // High-level category.
    let category = if e.is_connect() {
        "connect"
    } else if e.is_timeout() {
        "timeout"
    } else if e.is_request() {
        "request"
    } else if e.is_body() {
        "body"
    } else if e.is_decode() {
        "decode"
    } else if e.is_redirect() {
        "redirect"
    } else {
        "unknown"
    };
    out.push_str(&format!("({category}) "));

    // Walk the error chain so the underlying io error surfaces (e.g.
    // "ConnectionRefused", "dns error", "invalid HTTP header ..."). reqwest's
    // own Display already does this but truncates; we re-format to keep the
    // full chain.
    let mut current: Option<&dyn std::error::Error> = Some(e);
    let mut chain: Vec<String> = Vec::new();
    while let Some(err) = current {
        chain.push(err.to_string());
        current = err.source();
    }
    // Deduplicate consecutive identical entries (reqwest tends to repeat).
    // The separator is 5 bytes (space + 3-byte arrow + space); truncating by
    // a hard-coded byte count would slice the arrow in half and panic.
    const SEPARATOR: &str = " → ";
    let mut prev: Option<&str> = None;
    for entry in &chain {
        if Some(entry.as_str()) != prev {
            out.push_str(entry);
            out.push_str(SEPARATOR);
            prev = Some(entry.as_str());
        }
    }
    if out.ends_with(SEPARATOR) {
        out.truncate(out.len() - SEPARATOR.len());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_url_scheme_is_explicitly_classified() {
        assert_eq!(
            classify_request_url("http://example.test").unwrap(),
            RequestUrlScheme::Http
        );
        assert_eq!(
            classify_request_url("https://example.test").unwrap(),
            RequestUrlScheme::Https
        );
        assert_eq!(
            classify_request_url("ws://example.test").unwrap(),
            RequestUrlScheme::Ws
        );
        assert_eq!(
            classify_request_url("wss://example.test").unwrap(),
            RequestUrlScheme::Wss
        );
        assert!(require_http_request_url("ws://example.test").is_err());
        assert!(classify_request_url("ftp://example.test").is_err());
    }
    #[test]
    fn envelope_success_minimal() {
        let d = parse_envelope(r#"{"code":0,"descriptor":{"url":"http://x/y"}}"#).unwrap();
        assert_eq!(d.url, "http://x/y");
        assert_eq!(d.response_text_path, "text");
        assert!(d.headers.is_empty());
        assert!(d.query.is_empty());
    }

    #[test]
    fn envelope_success_with_headers_and_query() {
        let d = parse_envelope(
            r#"{"code":0,"descriptor":{"url":"http://x/y","headers":{"Authorization":"Bearer t"},"query":{"language":"zh"},"response_text_path":"result"}}"#,
        )
        .unwrap();
        assert_eq!(d.headers.get("Authorization").unwrap(), "Bearer t");
        assert_eq!(d.query.get("language").unwrap(), "zh");
        assert_eq!(d.response_text_path, "result");
    }

    #[test]
    fn envelope_failure_surfaces_error_message() {
        let err = parse_envelope(r#"{"code":7,"error":"nope"}"#).unwrap_err();
        assert!(
            err.contains("code=7"),
            "error should mention the code; got: {err}"
        );
        assert!(
            err.contains("nope"),
            "error should include the addon's message; got: {err}"
        );
    }

    #[test]
    fn envelope_failure_without_message_still_errors() {
        let err = parse_envelope(r#"{"code":1}"#).unwrap_err();
        assert!(err.contains("code=1"));
    }

    #[test]
    fn envelope_rejects_malformed() {
        // Not JSON.
        assert!(parse_envelope("not json").is_err());
        // JSON but not an object.
        assert!(parse_envelope(r#""hello""#).is_err());
        assert!(parse_envelope(r#"42"#).is_err());
        // Missing code entirely.
        assert!(parse_envelope(r#"{"descriptor":{"url":"x"}}"#).is_err());
        // Non-numeric code.
        assert!(parse_envelope(r#"{"code":"0"}"#).is_err());
        // Success but no descriptor.
        assert!(parse_envelope(r#"{"code":0}"#).is_err());
        // Descriptor present but missing url.
        assert!(parse_envelope(r#"{"code":0,"descriptor":{}}"#).is_err());
        // Descriptor url is empty/whitespace.
        assert!(parse_envelope(r#"{"code":0,"descriptor":{"url":"  "}}"#).is_err());
    }
}
