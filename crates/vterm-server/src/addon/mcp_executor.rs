//! Executor for `type:"mcp"` addons — the dynamic-MCP-tool counterpart of
//! [`llm_executor`]. A MCP addon is a QuickJS script exposing two entry
//! points:
//!
//! - `list_tools()` → JSON string of `[{name, description, inputSchema}]`.
//! - `call_tool(name, argsJson)` → JSON string of an MCP result object
//!   (`{content: [{type:"text",text}|{type:"image",data,mimeType}], isError?}`).
//!
//! The injected host object mirrors `llm_executor` (`get_config`, `log`,
//! `http_request`) plus `addon.bridge_request(payloadJson)` which forwards a
//! frame to the addon's daemon over the WebSocket bridge
//! ([`crate::mcp::services::addon::bridge`]) and returns the response as a
//! JSON string (`{"ok":true,"result":…}` / `{"ok":false,"error":"…"}`).

use std::collections::HashMap;
use std::time::Duration;

use rquickjs::prelude::Opt;
use rquickjs::{CatchResultExt, CaughtError, Context, Ctx, Function, Object, Runtime};
use serde_json::Value;
use tracing::{debug, warn};

use super::{resolve_addon_config, AddonHost};
use crate::config::TerminalServerConfig;

const DEFAULT_BRIDGE_TIMEOUT_SECS: u64 = 120;

/// Run the addon's `list_tools()` entry point. The returned string is the
/// raw JSON the addon produced; the caller (rmcp handler) validates it.
pub fn run_list_tools(
    addon_host: &AddonHost,
    config: &TerminalServerConfig,
    addon_id: &str,
) -> Result<String, String> {
    run_entry(addon_host, config, addon_id, "list_tools()".to_string())
}

/// Run the addon's `call_tool(name, argsJson)` entry point. `args_json` is
/// the tool arguments already serialized as JSON (an empty object when the
/// client sent none). Oversized text content is spilled to a temp file
/// (see [`spill_oversize_content`]).
pub fn run_call_tool(
    addon_host: &AddonHost,
    config: &TerminalServerConfig,
    addon_id: &str,
    name: &str,
    args_json: &str,
) -> Result<String, String> {
    let name_lit = serde_json::to_string(name).map_err(|e| e.to_string())?;
    let args_lit = serde_json::to_string(args_json).map_err(|e| e.to_string())?;
    let raw = run_entry(
        addon_host,
        config,
        addon_id,
        format!("call_tool({name_lit}, {args_lit})"),
    )?;
    Ok(spill_oversize_content(&raw, name))
}

/// Run the addon's optional `get_instructions()` entry point. Returns an
/// empty string when the addon doesn't define it (the handler falls back to
/// a generic sentence then).
pub fn run_get_instructions(
    addon_host: &AddonHost,
    config: &TerminalServerConfig,
    addon_id: &str,
) -> Result<String, String> {
    run_entry(
        addon_host,
        config,
        addon_id,
        "typeof get_instructions === \"function\" ? String(get_instructions()) : \"\"".to_string(),
    )
}

/// Hard cap for one text content item (context-strategy spec D10). Addons
/// following the catalog conventions spill at 32 KB on their own; this
/// backstop covers addons that don't.
const RESULT_TEXT_SPILL_CAP: usize = 256 * 1024;
/// How much of a spilled payload stays in the context as a preview.
const SPILL_PREVIEW_BYTES: usize = 2048;

/// Replace any text content item larger than [`RESULT_TEXT_SPILL_CAP`] with
/// a `{savedTo, sizeBytes, preview}` envelope after writing the full text to
/// `$TMP/vterm-mcp-spill/`. Unparseable or already-small results pass
/// through unchanged.
fn spill_oversize_content(raw: &str, tool_name: &str) -> String {
    let Ok(mut parsed) = serde_json::from_str::<Value>(raw) else {
        return raw.to_string();
    };
    let Some(content) = parsed.get_mut("content").and_then(Value::as_array_mut) else {
        return raw.to_string();
    };
    for item in content.iter_mut() {
        if item.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        let oversize = item
            .get("text")
            .and_then(Value::as_str)
            .is_some_and(|text| text.len() > RESULT_TEXT_SPILL_CAP);
        if oversize {
            let text = item
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            *item = spill_text_item(&text, tool_name);
        }
    }
    parsed.to_string()
}

fn spill_text_item(text: &str, tool_name: &str) -> Value {
    let sanitized: String = tool_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join("vterm-mcp-spill");
    let path = dir.join(format!("{sanitized}-{millis}.txt"));

    let mut end = SPILL_PREVIEW_BYTES.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let preview = &text[..end];

    let envelope = match std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, text)) {
        Ok(()) => serde_json::json!({
            "savedTo": path.to_string_lossy(),
            "sizeBytes": text.len(),
            "preview": preview,
            "note": "Result exceeded the executor size cap and was written to a file. \
                     Analyze it with file tools (grep, ranged reads); do not re-fetch \
                     the full payload into the conversation."
        }),
        // Spill failed (read-only temp dir?): truncate instead of dropping the
        // result entirely.
        Err(error) => serde_json::json!({
            "sizeBytes": text.len(),
            "preview": preview,
            "note": format!("Result exceeded the executor size cap and spilling to \
                             a file failed ({error}); it was truncated to the preview.")
        }),
    };
    serde_json::json!({ "type": "text", "text": envelope.to_string() })
}

fn run_entry(
    addon_host: &AddonHost,
    config: &TerminalServerConfig,
    addon_id: &str,
    call_expr: String,
) -> Result<String, String> {
    let source = addon_host
        .get_source(addon_id)
        .ok_or_else(|| format!("addon not found: {addon_id}"))?;

    let config_values = resolve_addon_config(config, addon_id, None);
    let bridge_timeout = config_values
        .get("timeout")
        .cloned()
        .or_else(|| addon_host.config_field_default(addon_id, "timeout"))
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(DEFAULT_BRIDGE_TIMEOUT_SECS));

    let rt = Runtime::new().map_err(|e| format!("failed to create JS runtime: {e}"))?;
    let ctx = Context::full(&rt).map_err(|e| format!("failed to create JS context: {e}"))?;

    let result = ctx.with(|ctx| -> Result<String, String> {
		let addon_obj =
			Object::new(ctx.clone()).map_err(|e| format!("failed to create addon object: {e}"))?;

		// addon.get_config(key)
		let config_for_cb: HashMap<String, String> = config_values.clone();
		addon_obj
			.set(
				"get_config",
				Function::new(ctx.clone(), move |key: String| -> Option<String> {
					config_for_cb.get(&key).cloned()
				}),
			)
			.ok();

		// addon.log(msg)
		addon_obj
			.set(
				"log",
				Function::new(ctx.clone(), move |msg: String| {
					debug!("[mcp-addon] {}", msg);
				}),
			)
			.ok();

		// addon.bridge_request(payloadJson) → response JSON string. Blocks the
		// QuickJS thread the same way llm_executor's http_request does: spawn a
		// thread with a current-thread tokio runtime and wait on an mpsc channel.
		let addon_id_for_cb = addon_id.to_string();
		addon_obj
			.set(
				"bridge_request",
				Function::new(
					ctx.clone(),
					move |payload: String| -> rquickjs::Result<String> {
						let (resp_tx, resp_rx) = std::sync::mpsc::channel();
						let addon_id = addon_id_for_cb.clone();
						std::thread::spawn(move || {
							let rt = match tokio::runtime::Builder::new_current_thread()
								.enable_time()
								.build()
							{
								Ok(rt) => rt,
								Err(e) => {
									let _ = resp_tx.send(
										serde_json::json!({"ok": false, "error": format!("failed to create runtime: {e}")})
											.to_string(),
									);
									return;
								}
							};
							let result = rt.block_on(async {
								match crate::mcp::services::addon::bridge::global_hub()
									.request(&addon_id, &payload, bridge_timeout)
									.await
								{
									Ok(value) => {
										serde_json::json!({"ok": true, "result": value}).to_string()
									}
									Err(error) => {
										serde_json::json!({"ok": false, "error": error}).to_string()
									}
								}
							});
							let _ = resp_tx.send(result);
						});
						match resp_rx.recv() {
							Ok(result_str) => Ok(result_str),
							Err(_) => Ok(
								serde_json::json!({"ok": false, "error": "bridge request channel closed"})
									.to_string(),
							),
						}
					},
				),
			)
			.ok();

		// addon.http_request(method, url, headers, body) — same blocking
		// thread+runtime pattern as llm_executor, with a per-call client.
		let http_timeout = bridge_timeout;
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
							timeout: Some(http_timeout),
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
										serde_json::json!({"status": 0, "body": format!("Failed to create runtime: {e}")}).to_string(),
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
												serde_json::json!({"status": status, "body": format!("Failed to read body: {e}")}).to_string()
											}
										}
									}
									Err(error) => {
										serde_json::json!({"status": 0, "body": format!("HTTP request failed: {error}")}).to_string()
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

		ctx.globals().set("addon", addon_obj).ok();

		if let Err(error) = ctx.eval::<(), _>(source.as_str()).catch(&ctx) {			let error = format_js_error(error);
			warn!("MCP addon JS source evaluation failed: {}", error);
			let _ = ctx.eval::<(), _>("addon = undefined");
			ctx.run_gc();
			return Err(format!("JS evaluation error: {error}"));
		}

		let eval_result: Result<String, String> = (|| {
			let result_str: String = ctx
				.eval::<String, _>(call_expr.as_str())
				.catch(&ctx)
				.map_err(format_js_error)?;
			Ok(result_str)
		})();

		let _ = ctx.eval::<(), _>("addon = undefined");
		ctx.run_gc();

		eval_result.map_err(|e| format!("JS execution error: {e}"))
	});

    rt.run_gc();
    result
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

    const LIST_TOOLS_FIXTURE: &str = r#"
function list_tools() {
	return JSON.stringify([
		{
			name: "demo_ping",
			description: "Ping the demo daemon",
			inputSchema: { type: "object", properties: {} }
		},
		{
			name: "demo_echo",
			description: "Echo text",
			inputSchema: {
				type: "object",
				properties: { text: { type: "string" } },
				required: ["text"]
			}
		}
	]);
}
function call_tool(name, argsJson) {
	if (name === "demo_fail") {
		return JSON.stringify({
			content: [{ type: "text", text: "daemon exploded" }],
			isError: true
		});
	}
	return JSON.stringify({
		content: [{ type: "text", text: "called " + name + " with " + argsJson }]
	});
}
"#;

    fn host_with_fixture() -> AddonHost {
        let host = AddonHost::new();
        host.insert_test_instance("demo-mcp", "mcp", LIST_TOOLS_FIXTURE);
        host
    }

    #[test]
    fn list_tools_returns_valid_descriptor_array() {
        let host = host_with_fixture();
        let config = TerminalServerConfig::default();
        let raw = run_list_tools(&host, &config, "demo-mcp").expect("list_tools ok");
        let tools: Vec<serde_json::Value> = serde_json::from_str(&raw).expect("valid JSON array");
        assert_eq!(tools.len(), 2);
        for tool in &tools {
            assert!(tool["name"].is_string(), "tool needs a name");
            assert!(tool["description"].is_string(), "tool needs a description");
            assert!(tool["inputSchema"].is_object(), "tool needs an inputSchema");
        }
        assert_eq!(tools[0]["name"], "demo_ping");
    }

    #[test]
    fn call_tool_passes_arguments_and_maps_error_flag() {
        let host = host_with_fixture();
        let config = TerminalServerConfig::default();

        let raw = run_call_tool(&host, &config, "demo-mcp", "demo_echo", r#"{"text":"hi"}"#)
            .expect("call_tool ok");
        let result: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(
            result["content"][0]["text"],
            r#"called demo_echo with {"text":"hi"}"#
        );
        assert!(result.get("isError").is_none());

        let raw = run_call_tool(&host, &config, "demo-mcp", "demo_fail", "{}").expect("call ok");
        let result: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(result["isError"], true);
        assert_eq!(result["content"][0]["text"], "daemon exploded");
    }

    #[test]
    fn unknown_addon_and_js_errors_surface_as_err() {
        let host = AddonHost::new();
        let config = TerminalServerConfig::default();
        let err = run_list_tools(&host, &config, "no-such-addon").unwrap_err();
        assert!(err.contains("addon not found"), "got: {err}");

        let host = host_with_fixture();
        let err = run_call_tool(&host, &config, "demo-mcp", "unterminated\"name", "{}");
        // Escaping must keep the call expression valid — this should succeed,
        // not produce a JS syntax error.
        assert!(err.is_ok(), "got: {err:?}");
    }

    #[test]
    fn bridge_request_reports_missing_daemon() {
        let fixture = r#"
function list_tools() { return "[]"; }
function call_tool(name, argsJson) {
	var resp = JSON.parse(addon.bridge_request(JSON.stringify({ action: "ping" })));
	return JSON.stringify({
		content: [{ type: "text", text: "ok=" + resp.ok + " err=" + resp.error }]
	});
}
"#;
        let host = AddonHost::new();
        host.insert_test_instance("demo-mcp", "mcp", fixture);
        let config = TerminalServerConfig::default();
        let raw = run_call_tool(&host, &config, "demo-mcp", "demo", "{}").unwrap();
        let result: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.starts_with("ok=false"), "got: {text}");
        assert!(text.contains("no daemon connected"), "got: {text}");
    }

    #[test]
    fn get_instructions_returns_addon_text_or_empty() {
        let with = r#"
function list_tools() { return "[]"; }
function call_tool(n, a) { return "{}"; }
function get_instructions() { return "do the thing"; }
"#;
        let host = AddonHost::new();
        host.insert_test_instance("with-instr", "mcp", with);
        host.insert_test_instance("without-instr", "mcp", LIST_TOOLS_FIXTURE);
        let config = TerminalServerConfig::default();

        let text = run_get_instructions(&host, &config, "with-instr").unwrap();
        assert_eq!(text, "do the thing");

        let text = run_get_instructions(&host, &config, "without-instr").unwrap();
        assert_eq!(text, "");
    }

    #[test]
    fn oversized_text_content_spills_to_temp_file() {
        let fixture = r#"
function list_tools() { return "[]"; }
function call_tool(name, argsJson) {
	var big = "";
	while (big.length < 300000) { big += "0123456789abcdef"; }
	return JSON.stringify({ content: [{ type: "text", text: big }] });
}
"#;
        let host = AddonHost::new();
        host.insert_test_instance("big-mcp", "mcp", fixture);
        let config = TerminalServerConfig::default();

        let raw = run_call_tool(&host, &config, "big-mcp", "big", "{}").unwrap();
        let result: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        let envelope: serde_json::Value = serde_json::from_str(text).unwrap();

        let saved_to = envelope["savedTo"].as_str().expect("spill path");
        assert_eq!(envelope["sizeBytes"].as_u64().unwrap(), 300000);
        assert!(envelope["preview"].as_str().unwrap().len() <= 2048);
        let on_disk = std::fs::read_to_string(saved_to).expect("spill file exists");
        assert_eq!(on_disk.len(), 300000);
        let _ = std::fs::remove_file(saved_to);
    }

    #[test]
    fn small_text_content_is_not_spilled() {
        let host = host_with_fixture();
        let config = TerminalServerConfig::default();
        let raw =
            run_call_tool(&host, &config, "demo-mcp", "demo_echo", r#"{"text":"hi"}"#).unwrap();
        let result: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(
            result["content"][0]["text"],
            r#"called demo_echo with {"text":"hi"}"#
        );
    }

    // The real builtin addon, executed for real: guards JS syntax, the entry
    // points, and the catalog paths against accidental regressions.
    const EASYEDA_BUILTIN: &str = include_str!("builtins/easyeda.js");

    fn host_with_easyeda_builtin() -> AddonHost {
        let host = AddonHost::new();
        host.insert_test_instance("easyeda", "mcp", EASYEDA_BUILTIN);
        host
    }

    #[test]
    fn easyeda_builtin_list_tools_and_instructions_are_valid() {
        let host = host_with_easyeda_builtin();
        let config = TerminalServerConfig::default();

        let raw = run_list_tools(&host, &config, "easyeda").expect("list_tools ok");
        let tools: Vec<serde_json::Value> = serde_json::from_str(&raw).unwrap();
        assert_eq!(tools.len(), 12, "Tier-1 surface must stay small");
        for tool in &tools {
            assert!(tool["name"].as_str().unwrap().starts_with("easyeda_"));
        }

        let instructions = run_get_instructions(&host, &config, "easyeda").unwrap();
        assert!(
            instructions.contains("easyeda_search_tools"),
            "got: {instructions}"
        );
    }

    #[test]
    fn easyeda_builtin_offline_tools_and_catalog_paths() {
        let host = host_with_easyeda_builtin();
        let config = TerminalServerConfig::default();

        // Categories work without a daemon.
        let raw =
            run_call_tool(&host, &config, "easyeda", "easyeda_list_categories", "{}").unwrap();
        let result: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let categories = result["content"][0]["text"].as_str().unwrap();
        for expected in ["schematic", "pcb", "library", "document", "system"] {
            assert!(categories.contains(expected), "missing {expected}");
        }

        // Every catalog tool resolves through describe; these paths were
        // verified against @jlceda/pro-api-types (context-strategy Phase A).
        for (name, path) in [
            (
                "doc_get_current_document_info",
                "eda.dmt_SelectControl.getCurrentDocumentInfo",
            ),
            (
                "sys_get_version",
                "eda.sys_Environment.getEditorCurrentVersion",
            ),
            (
                "sch_cross_probe_select",
                "eda.sch_SelectControl.doCrossProbeSelect",
            ),
            ("pcb_get_bom_file", "eda.pcb_ManufactureData.getBomFile"),
            (
                "pcb_get_netlist_file",
                "eda.pcb_ManufactureData.getNetlistFile",
            ),
            (
                "pcb_get_gerber_file",
                "eda.pcb_ManufactureData.getGerberFile",
            ),
        ] {
            let args = serde_json::json!({ "name": name }).to_string();
            let raw =
                run_call_tool(&host, &config, "easyeda", "easyeda_describe_tool", &args).unwrap();
            let result: serde_json::Value = serde_json::from_str(&raw).unwrap();
            let text = result["content"][0]["text"].as_str().unwrap();
            assert!(
                text.contains(&format!("\"path\": \"{path}\"")),
                "{name} should map to {path}, got: {text}"
            );
        }
    }
}
