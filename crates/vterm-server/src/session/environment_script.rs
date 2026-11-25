use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use vterm_common::{SessionId, SessionTerminalOs};

use crate::config::{EnvironmentNode, EnvironmentVariable};

use super::session_core::ShellKind;
use super::session_error::SessionError;

pub(super) fn write_environment_script(
    session_id: SessionId,
    terminal_os: SessionTerminalOs,
    shell_kind: Option<ShellKind>,
    node: &EnvironmentNode,
    enabled: bool,
) -> Result<Vec<u8>, SessionError> {
    // Only Leaf nodes carry environment variables; Groups are folders. The
    // caller (`apply_environment`) guards against Group nodes, but we assert
    // here defensively too.
    let variables = node.as_leaf_variables().ok_or_else(|| {
        SessionError::PtyError("environment profile must be a leaf node".to_string())
    })?;
    let kind = script_kind(terminal_os, shell_kind);
    let path = temp_script_path(session_id, kind.extension());
    let content = match kind {
        ScriptKind::Batch => batch_script(&path, variables, enabled),
        ScriptKind::PowerShell => powershell_script(&path, variables, enabled),
        ScriptKind::Unix => unix_script(&path, variables, enabled),
    };

    fs::write(&path, content).map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    #[cfg(unix)]
    if kind == ScriptKind::Unix {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions)
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    }

    Ok(kind.invocation(&path).into_bytes())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
                batch_value(&path),
                batch_value(&path)
            ),
            Self::PowerShell => format!(". '{}'\r", powershell_single_quote(&path)),
            Self::Unix => format!(". '{}'\r", shell_single_quote(&path)),
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
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("vterm-env-{session_id}-{nonce}.{extension}"))
}

fn batch_script(_path: &Path, variables: &[EnvironmentVariable], enabled: bool) -> String {
    let mut script = String::from("@echo off\r\n");
    for var in variables {
        if enabled {
            script.push_str(&format!(
                "set \"{}={}\"\r\n",
                batch_key(&var.key),
                batch_value(&var.value)
            ));
        } else {
            script.push_str(&format!("set \"{}=\"\r\n", batch_key(&var.key)));
        }
    }
    script
}

fn powershell_script(path: &Path, variables: &[EnvironmentVariable], enabled: bool) -> String {
    let mut script = String::new();
    for var in variables {
        if enabled {
            script.push_str(&format!(
                "$env:{} = '{}'\r\n",
                powershell_env_key(&var.key),
                powershell_single_quote(&var.value)
            ));
        } else {
            script.push_str(&format!(
                "Remove-Item -LiteralPath 'Env:\\{}' -ErrorAction SilentlyContinue\r\n",
                powershell_single_quote(&var.key)
            ));
        }
    }
    let path = path.to_string_lossy();
    script.push_str(&format!(
        "Remove-Item -LiteralPath '{}' -Force -ErrorAction SilentlyContinue\r\n",
        powershell_single_quote(&path)
    ));
    script
}

fn unix_script(path: &Path, variables: &[EnvironmentVariable], enabled: bool) -> String {
    let mut script = String::new();
    for var in variables {
        if enabled {
            script.push_str(&format!(
                "export {}='{}'\n",
                shell_key(&var.key),
                shell_single_quote(&var.value)
            ));
        } else {
            script.push_str(&format!("unset {}\n", shell_key(&var.key)));
        }
    }
    let path = path.to_string_lossy();
    script.push_str(&format!("rm -f -- '{}'\n", shell_single_quote(&path)));
    script
}

fn batch_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect()
}

fn batch_value(value: &str) -> String {
    value.replace('%', "%%").replace('"', "")
}

fn powershell_env_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect()
}

fn powershell_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

fn shell_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect()
}

fn shell_single_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EnvironmentVariable;

    #[test]
    fn unix_script_sources_and_removes_literal_path() {
        let variables = vec![EnvironmentVariable {
            key: "API_KEY".to_string(),
            value: "a'b".to_string(),
        }];

        let script = unix_script(Path::new("/tmp/vterm-env-test.sh"), &variables, true);

        assert!(script.contains("export API_KEY='a'\\''b'"));
        assert!(script.contains("rm -f -- '/tmp/vterm-env-test.sh'"));
    }

    #[test]
    fn batch_invocation_deletes_after_call_returns() {
        let invocation = ScriptKind::Batch.invocation(Path::new(
            r"C:\Users\user\AppData\Local\Temp\vterm-env-test.bat",
        ));

        assert!(invocation.starts_with("call \""));
        assert!(invocation.contains("\" & del /f /q \""));
    }

    #[test]
    fn batch_script_does_not_delete_itself() {
        let variables = vec![EnvironmentVariable {
            key: "API_KEY".to_string(),
            value: "abc".to_string(),
        }];

        let script = batch_script(Path::new("ignored.bat"), &variables, true);

        assert!(script.contains("set \"API_KEY=abc\""));
        assert!(!script.contains("del /f /q"));
    }
}
