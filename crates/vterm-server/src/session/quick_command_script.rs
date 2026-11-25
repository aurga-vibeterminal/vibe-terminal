use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use vterm_common::{SessionId, SessionTerminalOs};

use super::session_core::ShellKind;
use super::session_error::SessionError;

pub(super) fn write_quick_command_script(
    session_id: SessionId,
    terminal_os: SessionTerminalOs,
    shell_kind: Option<ShellKind>,
    repo_root: &Path,
    command: &str,
    env: &std::collections::HashMap<String, String>,
) -> Result<Vec<u8>, SessionError> {
    let kind = script_kind(terminal_os, shell_kind);
    let path = temp_script_path(session_id, kind.extension());
    let path_text = path.to_string_lossy();
    let repo_root = repo_root.to_string_lossy();
    let content = match kind {
        ScriptKind::Batch => format!(
            "@echo off\r\ncd /d \"{}\" || exit /b 1\r\n{}{}\r\n",
            batch_quote(&repo_root),
            batch_env_lines(env),
            command
        ),
        ScriptKind::PowerShell => format!(
            "Set-Location -LiteralPath '{}'\r\n{}& {{\r\n{}\r\n}}\r\nRemove-Item -LiteralPath '{}' -Force -ErrorAction SilentlyContinue\r\n",
            powershell_quote(&repo_root),
            powershell_env_lines(env),
            command,
            powershell_quote(&path_text)
        ),
        ScriptKind::Unix => format!(
            "cd -- '{}' || return 1\n{{\n{}{}\n}}\nrm -f -- '{}'\n",
            shell_quote(&repo_root),
            unix_env_lines(env),
            command,
            shell_quote(&path_text)
        ),
    };
    fs::write(&path, content).map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    Ok(kind.invocation(&path).into_bytes())
}

/// Render env vars as `set KEY=VALUE\r\n` lines for Windows batch.
fn batch_env_lines(env: &std::collections::HashMap<String, String>) -> String {
    env.iter()
        .map(|(k, v)| format!("set \"{}={}\"\r\n", k, batch_quote(v)))
        .collect()
}

/// Render env vars as `$env:KEY='VALUE'\r\n` lines for PowerShell.
fn powershell_env_lines(env: &std::collections::HashMap<String, String>) -> String {
    env.iter()
        .map(|(k, v)| format!("$env:{}='{}'\r\n", k, powershell_quote(v)))
        .collect()
}

/// Render env vars as `export KEY="VALUE"\n` lines for Unix shells.
/// Uses double quotes so that $HOME, $PATH etc. in values expand at runtime.
/// Escapes ", ` and ! to prevent unintended shell injection.
fn unix_env_lines(env: &std::collections::HashMap<String, String>) -> String {
    env.iter()
        .map(|(k, v)| {
            let escaped = v
                .replace('"', "\\\"")
                .replace('`', "\\`")
                .replace('!', "\\!");
            format!("export {}=\"{}\"\n", k, escaped)
        })
        .collect()
}

#[derive(Clone, Copy)]
enum ScriptKind {
    Batch,
    PowerShell,
    Unix,
}

impl ScriptKind {
    fn extension(self) -> &'static str {
        match self {
            Self::Batch => "bat",
            Self::PowerShell => "ps1",
            Self::Unix => "sh",
        }
    }
    fn invocation(self, path: &Path) -> String {
        let path = path.to_string_lossy();
        match self {
            Self::Batch => format!(
                "call \"{}\" & del /f /q \"{}\" >nul 2>nul\r",
                batch_quote(&path),
                batch_quote(&path)
            ),
            Self::PowerShell => format!(". '{}'\r", powershell_quote(&path)),
            Self::Unix => format!(". '{}'\r", shell_quote(&path)),
        }
    }
}

fn script_kind(terminal_os: SessionTerminalOs, shell_kind: Option<ShellKind>) -> ScriptKind {
    match (terminal_os, shell_kind) {
        (SessionTerminalOs::Windows, Some(ShellKind::PowerShell)) => ScriptKind::PowerShell,
        (SessionTerminalOs::Windows, _) => ScriptKind::Batch,
        _ => ScriptKind::Unix,
    }
}

fn temp_script_path(session_id: SessionId, extension: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("vterm-command-{session_id}-{nonce}.{extension}"))
}

fn batch_quote(value: &str) -> String {
    value.replace('%', "%%").replace('"', "")
}
fn powershell_quote(value: &str) -> String {
    value.replace('\'', "''")
}
fn shell_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}
