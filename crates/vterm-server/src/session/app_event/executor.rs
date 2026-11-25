use rquickjs::{CatchResultExt, Context, Function, Object, Runtime};
use tracing::trace;
use vterm_common::{SessionAppStatus, SessionTerminalOs};

use crate::addon::{AddonHost, AppDetectResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedCommand {
    pub seq_type: String,
    pub code: String,
}

fn parse_app_status(status_str: &str) -> SessionAppStatus {
    match status_str {
        "idle" => SessionAppStatus::Idle,
        "busy" => SessionAppStatus::Busy,
        "selection" => SessionAppStatus::Selection,
        "confirming" => SessionAppStatus::ConfirmingNormal,
        "confirming_special" => SessionAppStatus::ConfirmingSpecial,
        _ => SessionAppStatus::None,
    }
}

/// Call `get_program_name()` on the addon JS source. Returns None on failure.
pub fn run_get_program_name(addon_host: &AddonHost, addon_id: &str) -> Option<String> {
    let source = addon_host.get_source(addon_id)?;
    execute_string_fn(&source, "get_program_name")
}

/// Call `get_supported_commands()` on the addon JS source. Returns empty vec on failure.
pub fn run_get_supported_commands(addon_host: &AddonHost, addon_id: &str) -> Vec<SupportedCommand> {
    let Some(source) = addon_host.get_source(addon_id) else {
        return Vec::new();
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return Vec::new(),
    };
    let ctx = match Context::full(&rt) {
        Ok(ctx) => ctx,
        Err(_) => return Vec::new(),
    };

    let commands: Vec<SupportedCommand> = ctx.with(|ctx| -> Vec<SupportedCommand> {
        // Inject addon.log so get_supported_commands can use it
        let addon_obj = match Object::new(ctx.clone()) {
            Ok(obj) => obj,
            Err(_) => return Vec::new(),
        };
        addon_obj
            .set(
                "log",
                Function::new(ctx.clone(), move |msg: String| {
                    trace!("[app-addon] {}", msg);
                }),
            )
            .ok();
        ctx.globals().set("addon", addon_obj).ok();

        if ctx.eval::<(), _>(source).catch(&ctx).is_err() {
            return Vec::new();
        }

        let result_str: String = match ctx
            .eval::<String, _>("JSON.stringify(get_supported_commands())")
            .catch(&ctx)
        {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let parsed: serde_json::Value = serde_json::from_str(&result_str).unwrap_or_default();
        let Some(arr) = parsed.as_array() else {
            return Vec::new();
        };

        let mut commands = Vec::new();
        for item in arr {
            let seq_type = item
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let code = item
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            commands.push(SupportedCommand { seq_type, code });
        }

        let _ = ctx.eval::<(), _>("undefined");
        ctx.run_gc();
        commands
    });

    rt.run_gc();
    commands
}

/// Call `detect_command(os, type, code, data)` on the addon JS source.
/// `data` is the raw bytes from the terminal callback.
pub fn run_detect_command(
    addon_host: &AddonHost,
    addon_id: &str,
    seq_type: &str,
    code: &str,
    data: &[u8],
    terminal_os: Option<SessionTerminalOs>,
) -> AppDetectResult {
    let Some(source) = addon_host.get_source(addon_id) else {
        return AppDetectResult::default();
    };
    // Pass data as JSON array of byte values for JS to decode
    let data_array: Vec<u8> = data.to_vec();
    execute_app_fn_with_addon(
        &source,
        "detect_command",
        &[
            terminal_os_value(terminal_os),
            serde_json::json!(seq_type),
            serde_json::json!(code),
            serde_json::json!(data_array),
        ],
    )
}

/// Call `detect_output(text)` on the addon JS source.
pub fn run_detect_output(addon_host: &AddonHost, addon_id: &str, text: &str) -> AppDetectResult {
    let Some(source) = addon_host.get_source(addon_id) else {
        return AppDetectResult::default();
    };
    execute_app_fn_with_addon(&source, "detect_output", &[serde_json::json!(text)])
}

fn terminal_os_value(terminal_os: Option<SessionTerminalOs>) -> serde_json::Value {
    let Some(terminal_os) = terminal_os else {
        return serde_json::Value::Null;
    };
    let name = match terminal_os {
        SessionTerminalOs::Windows => "windows",
        SessionTerminalOs::Linux => "linux",
        SessionTerminalOs::Macos => "macos",
    };
    serde_json::json!(name)
}

/// Execute a JS function that returns a string (with `addon.log` injected).
fn execute_string_fn(source: &str, fn_name: &str) -> Option<String> {
    let rt = Runtime::new().ok()?;
    let ctx = Context::full(&rt).ok()?;

    let result: Option<String> = ctx.with(|ctx| -> Option<String> {
        // Inject addon.log so get_program_name / get_supported_commands can use it
        let addon_obj = Object::new(ctx.clone()).ok()?;
        addon_obj
            .set(
                "log",
                Function::new(ctx.clone(), move |msg: String| {
                    trace!("[app-addon] {}", msg);
                }),
            )
            .ok();
        ctx.globals().set("addon", addon_obj).ok()?;

        if ctx.eval::<(), _>(source).catch(&ctx).is_err() {
            return None;
        }
        let result_str: String = ctx
            .eval::<String, _>(&*format!("JSON.stringify({}())", fn_name))
            .catch(&ctx)
            .ok()?;
        let parsed: serde_json::Value = serde_json::from_str(&result_str).unwrap_or_default();
        let val = parsed.as_str().unwrap_or_default().to_string();
        let _ = ctx.eval::<(), _>("addon = undefined");
        ctx.run_gc();
        Some(val)
    });

    rt.run_gc();
    let val = result?;
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

/// Execute an app addon function with `addon.log` injected.
fn execute_app_fn_with_addon(
    source: &str,
    fn_name: &str,
    args: &[serde_json::Value],
) -> AppDetectResult {
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return AppDetectResult::default(),
    };

    let ctx = match Context::full(&rt) {
        Ok(ctx) => ctx,
        Err(_) => return AppDetectResult::default(),
    };

    let result: AppDetectResult = ctx.with(|ctx| -> AppDetectResult {
        // Inject addon.log
        let addon_obj = match Object::new(ctx.clone()) {
            Ok(obj) => obj,
            Err(_) => return AppDetectResult::default(),
        };
        addon_obj
            .set(
                "log",
                Function::new(ctx.clone(), move |msg: String| {
                    trace!("[app-addon] {}", msg);
                }),
            )
            .ok();
        ctx.globals().set("addon", addon_obj).ok();

        // Evaluate addon source
        if ctx.eval::<(), _>(source).catch(&ctx).is_err() {
            return AppDetectResult::default();
        }

        let args_json: Vec<String> = args
            .iter()
            .map(|arg| serde_json::to_string(arg).unwrap_or_default())
            .collect();

        let call_expr = format!("JSON.stringify({}({}))", fn_name, args_json.join(","));

        let result_str: String = match ctx.eval::<String, _>(&*call_expr).catch(&ctx) {
            Ok(s) => s,
            Err(_) => return AppDetectResult::default(),
        };

        let parsed: serde_json::Value = serde_json::from_str(&result_str).unwrap_or_default();
        let status_str = parsed
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        let message = parsed
            .get("message")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let idle_after_ms = parsed
            .get("idle_after_ms")
            .and_then(|v| v.as_u64())
            .filter(|ms| *ms > 0);
        let refresh_only = parsed
            .get("refresh_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Clean up
        let _ = ctx.eval::<(), _>("addon = undefined");
        ctx.run_gc();

        AppDetectResult {
            status: parse_app_status(status_str),
            message,
            idle_after_ms,
            refresh_only,
            stopped: status_str == "stop",
        }
    });

    rt.run_gc();
    result
}

#[cfg(test)]
mod tests {
    use super::{
        run_detect_command, run_detect_output, run_get_program_name, run_get_supported_commands,
    };
    use crate::addon::AddonHost;
    use vterm_common::SessionAppStatus;

    #[test]
    fn codex_builtin_reports_program_and_commands() {
        let host = AddonHost::new();

        assert_eq!(
            run_get_program_name(&host, "codex").as_deref(),
            Some("codex")
        );

        let commands = run_get_supported_commands(&host, "codex");
        assert!(commands
            .iter()
            .any(|cmd| cmd.seq_type == "osc" && cmd.code == "0"));
        assert!(commands
            .iter()
            .any(|cmd| cmd.seq_type == "osc" && cmd.code == "2"));
    }

    #[test]
    fn codex_builtin_detects_title_statuses() {
        let host = AddonHost::new();

        let busy = run_detect_command(
            &host,
            "codex",
            "osc",
            "0",
            "⠋ Codex".as_bytes(),
            Some(vterm_common::SessionTerminalOs::Linux),
        );
        assert_eq!(busy.status, SessionAppStatus::Busy);

        let idle = run_detect_command(
            &host,
            "codex",
            "osc",
            "0",
            "✳ Codex".as_bytes(),
            Some(vterm_common::SessionTerminalOs::Linux),
        );
        assert_eq!(idle.status, SessionAppStatus::Idle);

        let stop = run_detect_command(
            &host,
            "codex",
            "osc",
            "0",
            "".as_bytes(),
            Some(vterm_common::SessionTerminalOs::Windows),
        );
        assert!(stop.stopped);
    }

    #[test]
    fn codex_builtin_detects_confirmation_output() {
        let host = AddonHost::new();

        let normal = run_detect_output(&host, "codex", "Codex needs your approval to continue");
        assert_eq!(normal.status, SessionAppStatus::ConfirmingNormal);

        let special = run_detect_output(
            &host,
            "codex",
            "Allow command?\nrm -rf target\nYes, allow once",
        );
        assert_eq!(special.status, SessionAppStatus::ConfirmingSpecial);
    }

    #[test]
    fn deepseek_harness_builtin_detects_osc_progress_states() {
        let host = AddonHost::new();

        assert_eq!(
            run_get_program_name(&host, "deepseek-harness").as_deref(),
            Some("dsh")
        );
        assert!(run_get_supported_commands(&host, "deepseek-harness")
            .iter()
            .any(|cmd| cmd.seq_type == "osc" && cmd.code == "9"));

        let busy = run_detect_command(
            &host,
            "deepseek-harness",
            "osc",
            "9",
            b"4;1;0",
            Some(vterm_common::SessionTerminalOs::Windows),
        );
        assert_eq!(busy.status, SessionAppStatus::Busy);

        let idle = run_detect_command(
            &host,
            "deepseek-harness",
            "osc",
            "9",
            b"4;0;0",
            Some(vterm_common::SessionTerminalOs::Windows),
        );
        assert_eq!(idle.status, SessionAppStatus::Idle);
    }

    #[test]
    fn qwen_code_builtin_detects_osc_progress_states() {
        let host = AddonHost::new();

        assert_eq!(
            run_get_program_name(&host, "qwen-code").as_deref(),
            Some("qwen")
        );
        assert!(run_get_supported_commands(&host, "qwen-code")
            .iter()
            .any(|cmd| cmd.seq_type == "osc" && cmd.code == "9"));

        let busy = run_detect_command(
            &host,
            "qwen-code",
            "osc",
            "9",
            b"4;3;",
            Some(vterm_common::SessionTerminalOs::Windows),
        );
        assert_eq!(busy.status, SessionAppStatus::Busy);

        let idle = run_detect_command(
            &host,
            "qwen-code",
            "osc",
            "9",
            b"4;0;",
            Some(vterm_common::SessionTerminalOs::Windows),
        );
        assert_eq!(idle.status, SessionAppStatus::Idle);

        let notification = run_detect_command(
            &host,
            "qwen-code",
            "osc",
            "9",
            b"Qwen Code: task complete",
            Some(vterm_common::SessionTerminalOs::Macos),
        );
        assert_eq!(notification.status, SessionAppStatus::None);

        let malformed = run_detect_command(
            &host,
            "qwen-code",
            "osc",
            "9",
            b"4;;",
            Some(vterm_common::SessionTerminalOs::Linux),
        );
        assert_eq!(malformed.status, SessionAppStatus::None);

        let output = run_detect_output(&host, "qwen-code", "Qwen Code is responding");
        assert_eq!(output.status, SessionAppStatus::None);

        let spinner = run_detect_output(&host, "qwen-code", "  ⠸ Thinking…");
        assert_eq!(spinner.status, SessionAppStatus::Busy);
        assert_eq!(spinner.idle_after_ms, Some(500));

        let last_spinner_frame = run_detect_output(&host, "qwen-code", "  ⠏ Thinking…");
        assert_eq!(last_spinner_frame.status, SessionAppStatus::Busy);
        assert_eq!(last_spinner_frame.idle_after_ms, Some(500));

        let multiline = run_detect_output(
            &host,
            "qwen-code",
            "⠸ Thinking…\r\n────────\r\n> prompt\r\n➜ qwen · deepseek-flash",
        );
        assert_eq!(multiline.status, SessionAppStatus::Busy);

        let stopped_multiline = run_detect_output(
            &host,
            "qwen-code",
            "⠸ Thinking…\r\n是否应用此更改？\r\n› 1. 是，允许一次\r\n  2. 否，建议更改 (esc)\r\n⠏ 等待用户确认...",
        );
        assert_eq!(stopped_multiline.status, SessionAppStatus::Busy);
        assert_eq!(stopped_multiline.idle_after_ms, Some(500));

        let inline = run_detect_output(&host, "qwen-code", "The ⠸ character is only text");
        assert_eq!(inline.status, SessionAppStatus::None);
    }

    #[test]
    fn opencode_builtin_detects_classic_and_mini_spinners() {
        let host = AddonHost::new();

        assert_eq!(
            run_get_program_name(&host, "opencode").as_deref(),
            Some("opencode")
        );
        for output in [
            "  ⠸ Thinking",
            "🯦 esc stop · Build · GPT-5",
            "🯧 Running",
            "▪ esc stop · Build",
            "- esc stop - Build",
        ] {
            let spinner = run_detect_output(&host, "opencode", output);
            assert_eq!(spinner.status, SessionAppStatus::Busy, "output: {output}");
            assert_eq!(spinner.idle_after_ms, Some(1000), "output: {output}");
            assert!(!spinner.refresh_only, "output: {output}");
        }
        let refresh = run_detect_output(&host, "opencode", "🯧");
        assert_eq!(refresh.status, SessionAppStatus::Busy);
        assert!(refresh.refresh_only);
        for output in [
            "The ⠸ character is only text",
            "▪ opencode 1.0",
            "🯦 Build · GPT-5",
            "- ordinary output",
        ] {
            let ordinary = run_detect_output(&host, "opencode", output);
            assert_eq!(ordinary.status, SessionAppStatus::None, "output: {output}");
        }
    }

    #[test]
    fn pi_builtin_detects_status_sources() {
        let host = AddonHost::new();

        assert_eq!(run_get_program_name(&host, "pi").as_deref(), Some("pi"));
        let commands = run_get_supported_commands(&host, "pi");
        assert!(commands
            .iter()
            .any(|cmd| cmd.seq_type == "osc" && cmd.code == "9"));
        assert!(commands
            .iter()
            .any(|cmd| cmd.seq_type == "osc" && cmd.code == "7501"));

        for payload in ["4;3", "4;1;50"] {
            let busy = run_detect_command(
                &host,
                "pi",
                "osc",
                "9",
                payload.as_bytes(),
                Some(vterm_common::SessionTerminalOs::Windows),
            );
            assert_eq!(busy.status, SessionAppStatus::Busy);
        }

        let idle = run_detect_command(
            &host,
            "pi",
            "osc",
            "9",
            b"4;0;",
            Some(vterm_common::SessionTerminalOs::Windows),
        );
        assert_eq!(idle.status, SessionAppStatus::Idle);

        let malformed = run_detect_command(
            &host,
            "pi",
            "osc",
            "9",
            b"4;;",
            Some(vterm_common::SessionTerminalOs::Windows),
        );
        assert_eq!(malformed.status, SessionAppStatus::None);

        for payload in [
            "state=idle:app=pi",
            "state=done:app=pi",
            "state=error:app=pi",
        ] {
            let idle = run_detect_command(
                &host,
                "pi",
                "osc",
                "7501",
                payload.as_bytes(),
                Some(vterm_common::SessionTerminalOs::Windows),
            );
            assert_eq!(idle.status, SessionAppStatus::Idle);
        }
        let working = run_detect_command(
            &host,
            "pi",
            "osc",
            "7501",
            b"state=working:app=pi",
            Some(vterm_common::SessionTerminalOs::Windows),
        );
        assert_eq!(working.status, SessionAppStatus::Busy);

        for output in ["  ⠸ Working", "── ⠋ Working ─────", "───⠏──"]
        {
            let spinner = run_detect_output(&host, "pi", output);
            assert_eq!(spinner.status, SessionAppStatus::Busy, "output: {output}");
            assert_eq!(spinner.idle_after_ms, Some(500), "output: {output}");
        }
        for output in ["The ⠸ character is only text", "────────", "── ⠋Working ──"]
        {
            let ordinary = run_detect_output(&host, "pi", output);
            assert_eq!(ordinary.status, SessionAppStatus::None, "output: {output}");
        }
    }
}
