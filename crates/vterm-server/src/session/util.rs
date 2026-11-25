use std::path::Path;

use vterm_common::SessionTerminalOs;

/// Infer the path semantics used by a launched shell session.
pub(crate) fn infer_terminal_os(program: &str, args: &[String]) -> SessionTerminalOs {
    let program_name = Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase();
    let args_are_wsl = args
        .iter()
        .any(|arg| arg.eq_ignore_ascii_case("--distribution") || arg.eq_ignore_ascii_case("-d"));

    if program_name == "wsl.exe" || program_name == "wsl" || args_are_wsl {
        return SessionTerminalOs::Linux;
    }

    default_terminal_os()
}

/// Parse the current shell prompt and derive its working directory.
pub(crate) fn prompt_working_path(
    line: &str,
    terminal_os: Option<SessionTerminalOs>,
) -> Option<String> {
    let trimmed = line.trim_matches(char::from(0)).trim_end();
    if trimmed.is_empty() {
        return None;
    }

    parse_powershell_prompt_path(trimmed, terminal_os)
        .or_else(|| parse_cmd_prompt_path(trimmed, terminal_os))
        .or_else(|| parse_unix_prompt_path(trimmed, terminal_os))
}

/// Parse a PowerShell prompt like `PS C:\repo>` into a filesystem path.
fn parse_powershell_prompt_path(
    line: &str,
    terminal_os: Option<SessionTerminalOs>,
) -> Option<String> {
    let path = line.strip_prefix("PS ")?.strip_suffix('>')?.trim();
    normalize_working_path(path, terminal_os)
}

/// Parse a cmd.exe prompt like `C:\repo>` into a filesystem path.
fn parse_cmd_prompt_path(line: &str, terminal_os: Option<SessionTerminalOs>) -> Option<String> {
    let path = line.strip_suffix('>')?.trim();
    if path.starts_with("PS ") {
        return None;
    }
    normalize_working_path(path, terminal_os)
}

/// Parse a Unix-style prompt like `user@host:/repo$` into a filesystem path.
fn parse_unix_prompt_path(line: &str, terminal_os: Option<SessionTerminalOs>) -> Option<String> {
    let marker_index = line.rfind(['$', '#'])?;
    let before_marker = line[..marker_index].trim_end();
    if before_marker.is_empty() {
        return None;
    }

    let candidate = before_marker
        .rsplit_once(':')
        .map(|(_, path)| path)
        .unwrap_or(before_marker)
        .split_whitespace()
        .last()
        .unwrap_or(before_marker);

    normalize_working_path(candidate, terminal_os)
}

/// Normalize shell-reported paths into the OS-specific form used by the session.
pub(crate) fn normalize_working_path(
    path: &str,
    terminal_os: Option<SessionTerminalOs>,
) -> Option<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    match terminal_os.unwrap_or(default_terminal_os()) {
        SessionTerminalOs::Windows => normalize_windows_working_path(trimmed),
        SessionTerminalOs::Linux | SessionTerminalOs::Macos => normalize_unix_working_path(trimmed),
    }
}

/// Return the default working directory for a new session when no explicit CWD is given,
/// based on the terminal's OS (not the server OS, since e.g. WSL runs Linux on Windows).
///
/// - Unix (macOS/Linux): `$HOME`
/// - Windows: `%USERPROFILE%`
pub(crate) fn default_home_cwd(terminal_os: Option<SessionTerminalOs>) -> Option<String> {
    match terminal_os.unwrap_or(default_terminal_os()) {
        SessionTerminalOs::Windows => std::env::var("USERPROFILE").ok(),
        SessionTerminalOs::Linux | SessionTerminalOs::Macos => std::env::var("HOME").ok(),
    }
}

/// Return the default terminal OS for the current server platform.
pub(crate) fn default_terminal_os() -> SessionTerminalOs {
    #[cfg(target_os = "windows")]
    {
        SessionTerminalOs::Windows
    }
    #[cfg(target_os = "macos")]
    {
        SessionTerminalOs::Macos
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        SessionTerminalOs::Linux
    }
}

/// Normalize Windows-oriented working directory formats.
fn normalize_windows_working_path(path: &str) -> Option<String> {
    if is_windows_absolute_path(path) || path.starts_with(r"\\") {
        return Some(path.replace('/', "\\"));
    }

    if let Some(rest) = path.strip_prefix("/mnt/") {
        let mut segments = rest.splitn(2, '/');
        let drive = segments.next()?;
        let suffix = segments.next().unwrap_or_default();
        if drive.len() == 1 {
            let drive = drive.chars().next()?.to_ascii_uppercase();
            let suffix = suffix.replace('/', "\\");
            return Some(if suffix.is_empty() {
                format!("{drive}:\\")
            } else {
                format!("{drive}:\\{suffix}")
            });
        }
    }

    if path.len() >= 4
        && path.starts_with('/')
        && path.as_bytes()[1].is_ascii_alphabetic()
        && path.as_bytes()[2] == b':'
        && path.as_bytes()[3] == b'/'
    {
        let drive = (path.as_bytes()[1] as char).to_ascii_uppercase();
        let suffix = path[4..].replace('/', "\\");
        return Some(if suffix.is_empty() {
            format!("{drive}:\\")
        } else {
            format!("{drive}:\\{suffix}")
        });
    }

    if path.len() >= 3
        && path.starts_with('/')
        && path.as_bytes()[2] == b'/'
        && path.as_bytes()[1].is_ascii_alphabetic()
    {
        let drive = (path.as_bytes()[1] as char).to_ascii_uppercase();
        let suffix = path[3..].replace('/', "\\");
        return Some(if suffix.is_empty() {
            format!("{drive}:\\")
        } else {
            format!("{drive}:\\{suffix}")
        });
    }

    if path.starts_with('~') {
        return Some(path.to_string());
    }

    None
}

/// Normalize Linux/macOS working directory formats.
fn normalize_unix_working_path(path: &str) -> Option<String> {
    if path.starts_with('/') || path.starts_with('~') {
        return Some(path.replace('\\', "/"));
    }

    None
}

/// Check whether a path uses a Windows drive-letter absolute form.
fn is_windows_absolute_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_working_path_parses_common_shell_prompts() {
        assert_eq!(
            prompt_working_path(
                r"PS C:\Projects\vibe-terminal>",
                Some(SessionTerminalOs::Windows)
            ),
            Some(r"C:\Projects\vibe-terminal".to_string())
        );
        assert_eq!(
            prompt_working_path(
                r"C:\Projects\vibe-terminal>",
                Some(SessionTerminalOs::Windows)
            ),
            Some(r"C:\Projects\vibe-terminal".to_string())
        );
        assert_eq!(
            prompt_working_path("user@host:/repo/path$", Some(SessionTerminalOs::Linux)),
            Some("/repo/path".to_string())
        );
        assert_eq!(
            prompt_working_path(
                "MINGW64 /c/Projects/vibe-terminal$",
                Some(SessionTerminalOs::Windows)
            ),
            Some(r"C:\Projects\vibe-terminal".to_string())
        );
        assert_eq!(
            prompt_working_path(
                "user@host:/mnt/c/Projects/vibe-terminal$",
                Some(SessionTerminalOs::Linux)
            ),
            Some("/mnt/c/Projects/vibe-terminal".to_string())
        );
    }

    #[test]
    fn test_normalize_working_path_uses_terminal_os() {
        assert_eq!(
            normalize_working_path(
                "/C:/Projects/vibe-terminal",
                Some(SessionTerminalOs::Windows)
            ),
            Some(r"C:\Projects\vibe-terminal".to_string())
        );
        assert_eq!(
            normalize_working_path(
                "/mnt/c/Projects/vibe-terminal",
                Some(SessionTerminalOs::Windows)
            ),
            Some(r"C:\Projects\vibe-terminal".to_string())
        );
        assert_eq!(
            normalize_working_path(
                "/mnt/c/Projects/vibe-terminal",
                Some(SessionTerminalOs::Linux)
            ),
            Some("/mnt/c/Projects/vibe-terminal".to_string())
        );
    }

    #[test]
    fn test_infer_terminal_os_marks_wsl_as_linux() {
        assert_eq!(
            infer_terminal_os(r"C:\Windows\System32\wsl.exe", &[]),
            SessionTerminalOs::Linux
        );
        assert_eq!(
            infer_terminal_os(
                r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
                &[]
            ),
            default_terminal_os()
        );
    }
}
