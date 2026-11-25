use std::collections::HashMap;
use std::sync::Arc;

use rquickjs::prelude::Opt;
use rquickjs::{CatchResultExt, CaughtError, Context, Ctx, Function, Object, Runtime};
use tracing::{debug, info, warn};

use crate::addon::AddonRunResult;

pub(super) fn execute_addon(
    source: &str,
    config_values: HashMap<String, String>,
    ipv4: Option<String>,
    ipv6: Option<String>,
) -> AddonRunResult {
    info!("execute_addon: creating JS runtime");

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(error) => {
            warn!("execute_addon: failed to create JS runtime: {}", error);
            return AddonRunResult {
                success: false,
                error: Some(format!("failed to create JS runtime: {}", error)),
                ..Default::default()
            };
        }
    };

    let ctx = match Context::full(&rt) {
        Ok(ctx) => ctx,
        Err(error) => {
            warn!("execute_addon: failed to create JS context: {}", error);
            return AddonRunResult {
                success: false,
                error: Some(format!("failed to create JS context: {}", error)),
                ..Default::default()
            };
        }
    };

    info!(
        "execute_addon: JS runtime and context ready, evaluating addon source ({} bytes)",
        source.len()
    );

    let addon_result = ctx.with(|ctx| -> rquickjs::Result<AddonRunResult> {
        let logs: Arc<std::sync::Mutex<Vec<String>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let logs_clone = Arc::clone(&logs);

        let addon_obj = Object::new(ctx.clone())?;

        // get_ipv4
        let ipv4_for_cb = ipv4.clone();
        addon_obj.set(
            "get_ipv4",
            Function::new(ctx.clone(), move |_ctx: Ctx| -> Option<String> {
                ipv4_for_cb.clone()
            })?,
        )?;

        // get_ipv6
        let ipv6_for_cb = ipv6.clone();
        addon_obj.set(
            "get_ipv6",
            Function::new(ctx.clone(), move |_ctx: Ctx| -> Option<String> {
                ipv6_for_cb.clone()
            })?,
        )?;

        // get_config - pre-build a JS Object and return clones from the callback.
        // The object is captured by the Rust closure, which is fine as long as
        // we run GC inside ctx.with() before returning (see cleanup below).
        let config_obj = Object::new(ctx.clone())?;
        for (key, value) in &config_values {
            config_obj.set(key.as_str(), value.as_str())?;
        }
        addon_obj.set(
            "get_config",
            Function::new(ctx.clone(), move |_ctx: Ctx| -> Object<'_> {
                config_obj.clone()
            })?,
        )?;

        // log
        addon_obj.set(
            "log",
            Function::new(ctx.clone(), move |msg: String| {
                logs_clone.lock().unwrap().push(msg);
            })?,
        )?;

        // http_request — uses channel to bridge sync JS with async reqwest.
        // The proxy state is resolved per request (see addon::http), so a
        // system proxy toggled while the server runs takes effect without a
        // restart. The 30s timeout matches the manager's previous client-level
        // default.
        addon_obj.set(
            "http_request",
            Function::new(
                ctx.clone(),
                move |_ctx: Ctx, method: String, url: String, headers: Opt<Object>, body: Option<String>| -> rquickjs::Result<String> {
                    // Header pairs are read on the JS thread: rquickjs
                    // objects cannot cross the thread boundary.
                    let mut header_pairs = Vec::new();
                    if let Some(headers) = headers.0 {
                        let keys: Vec<String> = headers.keys().filter_map(|k| k.ok()).collect();
                        for key in keys {
                            if let Ok(val) = headers.get::<_, String>(&key) {
                                header_pairs.push((key, val));
                            }
                        }
                    }
                    let request = crate::addon::http::AddonHttpRequest {
                        method,
                        url,
                        headers: header_pairs,
                        query: Vec::new(),
                        body: body.map(String::into_bytes),
                        timeout: Some(std::time::Duration::from_secs(30)),
                    };

                    let (resp_tx, resp_rx) = std::sync::mpsc::channel();

                    let rt_handle = tokio::runtime::Handle::current();
                    let rt_handle2 = rt_handle.clone();
                    std::thread::spawn(move || {
                        let resp = rt_handle.block_on(async {
                            crate::addon::http::send_addon_request(&request).await
                        });
                        let _ = resp_tx.send(resp);
                    });

                    let resp_result = resp_rx.recv();
                    match resp_result {
                        Ok(Ok(resp)) => {
                            let status = resp.status().as_u16();
                            let body_bytes = rt_handle2.block_on(async { resp.bytes().await });
                            let body_str = match body_bytes {
                                Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                                Err(_) => String::new(),
                            };
                            Ok(serde_json::json!({"status": status, "body": body_str}).to_string())
                        }
                        Ok(Err(error)) => {
                            Ok(serde_json::json!({"status": 0, "body": format!("HTTP request failed: {}", error)}).to_string())
                        }
                        Err(_) => {
                            Ok(serde_json::json!({"status": 0, "body": "HTTP request channel closed"}).to_string())
                        }
                    }
                },
            )?,
        )?;

        ctx.globals().set("addon", addon_obj)?;

        // Evaluate addon source
        debug!("execute_addon: evaluating addon JS source");
        if let Err(error) = ctx.eval::<(), _>(source).catch(&ctx) {
            let error = format_js_error(error);
            warn!("Addon JS source evaluation failed: {}", error);
            return Ok(AddonRunResult {
                success: false,
                logs: logs.lock().unwrap().clone(),
                error: Some(format!("JS execution error: {}", error)),
                ..Default::default()
            });
        }
        info!("execute_addon: JS source evaluated, calling run()");

        // Run the addon inside a nested closure so cleanup always executes,
        // even if run() throws a JS exception.  Without this, the ? operator
        // skips the GC cleanup, leaking JS objects and triggering a QuickJS
        // assertion (list_empty(&rt->gc_obj_list)) on runtime drop.
        let eval_result: Result<AddonRunResult, String> = (|| {
            let run_result_str: String = ctx
                .eval("JSON.stringify(run())")
                .catch(&ctx)
                .map_err(format_js_error)?;
            debug!("execute_addon: run() returned: {}", run_result_str);
            let collected_logs = logs.lock().unwrap().clone();

            let mut result = AddonRunResult {
                success: true,
                logs: collected_logs,
                ..Default::default()
            };

            let parsed: serde_json::Value =
                serde_json::from_str(&run_result_str).unwrap_or_default();
            if let Some(obj) = parsed.as_object() {
                if let Some(success) = obj.get("success").and_then(|v| v.as_bool()) {
                    result.success = success;
                }
                if let Some(error) = obj.get("error").and_then(|v| v.as_str()) {
                    result.error = Some(error.to_string());
                }
                if let Some(ip) = obj.get("ip").and_then(|v| v.as_str()) {
                    result.ip = Some(ip.to_string());
                }
            }
            Ok(result)
        })();

        // Always clean up global references and force GC to release JS objects
        // captured by Rust closures (e.g. config_obj inside get_config).
        let _ = ctx.eval::<(), _>("addon = undefined");
        ctx.run_gc();

        if let Ok(ref result) = eval_result {
            info!("execute_addon: finished success={}", result.success);
        }
        match eval_result {
            Ok(result) => Ok(result),
            Err(error) => {
                warn!("Addon JS run() failed: {}", error);
                Ok(AddonRunResult {
                    success: false,
                    logs: logs.lock().unwrap().clone(),
                    error: Some(format!("JS execution error: {}", error)),
                    ..Default::default()
                })
            }
        }
    })
    .unwrap_or_else(|error| {
        warn!("Addon JS execution failed: {}", error);
        AddonRunResult {
            success: false,
            error: Some(format!("JS execution error: {}", error)),
            ..Default::default()
        }
    });

    // Force GC before Runtime drops to release any remaining JS objects.
    rt.run_gc();

    addon_result
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
    fn addon_logs_are_preserved_when_run_returns_failure() {
        let source = include_str!("../addon/builtins/cloudflare-ddns.js");
        let result = execute_addon(source, HashMap::new(), Some("192.0.2.10".to_string()), None);

        assert!(!result.success);
        assert_eq!(
            result.error.as_deref(),
            Some("Missing required config: email, api_key, and domain")
        );
        assert!(result
            .logs
            .iter()
            .any(|line| line == "Cloudflare DDNS addon started"));
    }

    #[test]
    fn addon_logs_and_exception_details_are_preserved_when_run_throws() {
        let source = r#"
            function run() {
                addon.log('before throw');
                throw new Error('boom');
            }
        "#;

        let result = execute_addon(source, HashMap::new(), None, None);

        assert!(!result.success);
        assert_eq!(result.logs, vec!["before throw".to_string()]);
        let error = result.error.unwrap_or_default();
        assert!(error.contains("Error: boom"), "{error}");
        assert!(error.contains("run"), "{error}");
    }
}
