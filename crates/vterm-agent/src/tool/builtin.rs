use async_trait::async_trait;
use serde::Deserialize;
use tokio::process::Command;
use tracing::{debug, info, warn};

use super::{FunctionDefinition, Tool, ToolDefinition};

// --- file_read ---

pub struct FileReadTool;

#[derive(Deserialize)]
struct FileReadArgs {
    path: String,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
}

#[async_trait]
impl Tool for FileReadTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            r#type: "function".into(),
            function: FunctionDefinition {
                name: "file_read".into(),
                description: "Read file content at the given path. Returns file content as text."
                    .into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Absolute or relative file path" },
                        "offset": { "type": "integer", "description": "Starting line number (1-based)" },
                        "limit": { "type": "integer", "description": "Maximum number of lines to read" }
                    },
                    "required": ["path"]
                }),
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
        let args: FileReadArgs =
            serde_json::from_value(args).map_err(|e| format!("Invalid arguments: {e}"))?;

        info!(path = %args.path, offset = ?args.offset, limit = ?args.limit, "file_read: reading file");

        let content = tokio::fs::read_to_string(&args.path)
            .await
            .map_err(|e| format!("Failed to read file '{}': {e}", args.path))?;

        let lines: Vec<&str> = content.lines().collect();
        let offset = args.offset.unwrap_or(1).saturating_sub(1);
        let limit = args.limit.unwrap_or(lines.len().saturating_sub(offset));

        let selected: Vec<String> = lines[offset..]
            .iter()
            .take(limit)
            .enumerate()
            .map(|(i, line)| format!("{}: {}", offset + i + 1, line))
            .collect();

        let result = selected.join("\n");
        debug!(
            path = %args.path,
            total_lines = lines.len(),
            selected_lines = selected.len(),
            result_len = result.len(),
            "file_read: complete"
        );
        Ok(result)
    }
}

// --- file_find ---

pub struct FileFindTool;

#[derive(Deserialize)]
struct FileFindArgs {
    pattern: String,
    #[serde(default)]
    directory: Option<String>,
}

#[async_trait]
impl Tool for FileFindTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            r#type: "function".into(),
            function: FunctionDefinition {
                name: "file_find".into(),
                description: "Find files matching a glob pattern in the given directory.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Glob pattern to match (e.g. '**/*.rs')" },
                        "directory": { "type": "string", "description": "Directory to search in (defaults to current directory)" }
                    },
                    "required": ["pattern"]
                }),
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
        let args: FileFindArgs =
            serde_json::from_value(args).map_err(|e| format!("Invalid arguments: {e}"))?;

        let dir = args.directory.as_deref().unwrap_or(".");
        let pattern = &args.pattern;

        info!(pattern = %pattern, directory = %dir, "file_find: searching");

        let paths: Vec<String> = glob::glob(&format!("{dir}/{pattern}"))
            .map_err(|e| format!("Invalid pattern: {e}"))?
            .filter_map(|entry| entry.ok())
            .filter(|path| path.is_file())
            .map(|path| path.display().to_string())
            .take(100)
            .collect();

        let count = paths.len();
        debug!(pattern = %pattern, directory = %dir, match_count = count, "file_find: complete");

        if paths.is_empty() {
            Ok("No files found.".into())
        } else {
            Ok(paths.join("\n"))
        }
    }
}

// --- code_search ---

pub struct CodeSearchTool;

#[derive(Deserialize)]
struct CodeSearchArgs {
    pattern: String,
    #[serde(default)]
    file_glob: Option<String>,
    #[serde(default)]
    directory: Option<String>,
}

#[async_trait]
impl Tool for CodeSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            r#type: "function".into(),
            function: FunctionDefinition {
                name: "code_search".into(),
                description:
                    "Search for a pattern in file contents using regex. Returns matching lines."
                        .into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Regex pattern to search for" },
                        "file_glob": { "type": "string", "description": "File glob filter (e.g. '*.rs')" },
                        "directory": { "type": "string", "description": "Directory to search in" }
                    },
                    "required": ["pattern"]
                }),
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
        let args: CodeSearchArgs =
            serde_json::from_value(args).map_err(|e| format!("Invalid arguments: {e}"))?;

        let dir = args.directory.as_deref().unwrap_or(".");

        info!(
            pattern = %args.pattern,
            file_glob = ?args.file_glob,
            directory = %dir,
            "code_search: searching"
        );

        let mut cmd = Command::new("grep");
        cmd.arg("-rn")
            .arg("--color=never")
            .arg("-E")
            .arg(&args.pattern)
            .arg(dir);

        if let Some(glob) = &args.file_glob {
            cmd.arg("--include").arg(glob);
        }

        // Same CREATE_NO_WINDOW rationale as vterm_common::command_no_window,
        // inline because this is a tokio Command.
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x0800_0000);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("grep failed: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().take(100).collect();

        debug!(
            pattern = %args.pattern,
            match_count = lines.len(),
            "code_search: complete"
        );

        if lines.is_empty() {
            Ok("No matches found.".into())
        } else {
            Ok(lines.join("\n"))
        }
    }
}

// --- shell ---

/// Threshold above which a git command's output is considered oversized and
/// truncated. A single multi-MB `git diff` (e.g. a 7.6MB glTF blob git treats
/// as text) would otherwise be forwarded verbatim and blow the model's context
/// limit. Only git commands are capped — other shell commands keep full output.
const SHELL_OUTPUT_TRUNCATE_AT: usize = 128 * 1024;
/// When truncation kicks in, keep only this many leading bytes so the agent
/// still gets a meaningful sample of the output.
const SHELL_OUTPUT_KEEP: usize = 1024;

/// True if `command` is a git invocation. Detects `git` whether it appears
/// bare (`git diff`), as an absolute/relative path (`/usr/bin/git log`), on
/// Windows (`git.exe`), or after shell connectors like `&&` / `;` / `|`
/// (e.g. `cd repo && git diff`).
fn is_git_command(command: &str) -> bool {
    for segment in command.split(|c: char| matches!(c, '&' | '|' | ';' | '\n')) {
        for tok in segment.split_whitespace() {
            // strip surrounding quotes
            let tok = tok.trim_matches(|c: char| c == '"' || c == '\'');
            let base = tok.rsplit(['/', '\\']).next().unwrap_or(tok);
            if matches!(base, "git" | "git.exe") {
                return true;
            }
        }
    }
    false
}

/// Trims oversized git command output to the first `SHELL_OUTPUT_KEEP` bytes
/// and appends a marker so the LLM knows content was dropped. Non-git output is
/// returned untouched regardless of size.
fn truncate_shell_output(command: &str, stdout: &str) -> String {
    if !is_git_command(command) || stdout.len() <= SHELL_OUTPUT_TRUNCATE_AT {
        return stdout.trim().to_string();
    }
    let kept = stdout.chars().take(SHELL_OUTPUT_KEEP).collect::<String>();
    format!(
        "{}\n[output truncated: {} bytes total, kept first {}]",
        kept.trim_end(),
        stdout.len(),
        SHELL_OUTPUT_KEEP
    )
}

pub struct ShellTool;

#[cfg(target_os = "windows")]
#[derive(Debug, PartialEq, Eq)]
struct ExplicitPowerShellCommand<'a> {
    executable: &'a str,
    command_switch: &'a str,
    script: &'a str,
}

/// Recognize a standalone, explicit PowerShell invocation so it can bypass
/// Git Bash. Passing `powershell -Command "$value ..."` through `bash -lc`
/// expands PowerShell's `$value` as a Bash variable before PowerShell starts.
#[cfg(target_os = "windows")]
fn explicit_powershell_command(command: &str) -> Option<ExplicitPowerShellCommand<'_>> {
    let command = command.trim();
    let executable_end = command.find(char::is_whitespace)?;
    let executable = &command[..executable_end];
    if !matches!(
        executable.to_ascii_lowercase().as_str(),
        "powershell" | "powershell.exe" | "pwsh" | "pwsh.exe"
    ) {
        return None;
    }

    let remainder = command[executable_end..].trim_start();
    let switch_end = remainder
        .find(char::is_whitespace)
        .unwrap_or(remainder.len());
    let command_switch = &remainder[..switch_end];
    if !command_switch.eq_ignore_ascii_case("-command")
        && !command_switch.eq_ignore_ascii_case("-c")
    {
        return None;
    }

    let script = remainder[switch_end..].trim();
    if script.is_empty() {
        return None;
    }
    let script = match (script.as_bytes().first(), script.as_bytes().last()) {
        (Some(b'"'), Some(b'"')) | (Some(b'\''), Some(b'\'')) if script.len() >= 2 => {
            &script[1..script.len() - 1]
        }
        _ => script,
    };

    Some(ExplicitPowerShellCommand {
        executable,
        command_switch,
        script,
    })
}

#[cfg(target_os = "windows")]
fn windows_bash_path() -> Option<std::path::PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("VTERM_AGENT_BASH") {
        candidates.push(path.into());
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        candidates.push(
            std::path::PathBuf::from(program_files)
                .join("Git")
                .join("bin")
                .join("bash.exe"),
        );
    }
    candidates.extend([
        r"C:\\Program Files\\Git\\bin\\bash.exe".into(),
        r"C:\\msys64\\usr\\bin\\bash.exe".into(),
        r"C:\\cygwin64\\bin\\bash.exe".into(),
        r"C:\\cygwin\\bin\\bash.exe".into(),
    ]);
    candidates.into_iter().find(|path| path.is_file())
}

#[cfg(target_os = "windows")]
fn add_windows_shell_paths(cmd: &mut Command, bash: &std::path::Path) {
    let Some(bin_dir) = bash.parent() else {
        return;
    };

    let mut paths = vec![bin_dir.to_path_buf()];
    // Git for Windows keeps Unix utilities such as `wc` in usr/bin.  MSYS2
    // uses the same layout, with native toolchains in mingw64/bin.
    if let Some(root) = bin_dir.parent() {
        let usr_bin = root.join("usr").join("bin");
        if usr_bin.is_dir() {
            paths.push(usr_bin);
        }
        let mingw_bin = root.join("mingw64").join("bin");
        if mingw_bin.is_dir() {
            paths.push(mingw_bin);
        }
    }
    if let Some(existing) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&existing));
    }
    if let Ok(path) = std::env::join_paths(paths) {
        cmd.env("PATH", path);
    }
}

#[derive(Deserialize)]
struct ShellArgs {
    command: String,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    timeout: Option<u64>,
}

#[async_trait]
impl Tool for ShellTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            r#type: "function".into(),
            function: FunctionDefinition {
                name: "shell".into(),
                description: "Execute a shell command and return its output.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Shell command to execute" },
                        "cwd": { "type": "string", "description": "Working directory" },
                        "timeout": { "type": "integer", "description": "Timeout in seconds (default 30)" }
                    },
                    "required": ["command"]
                }),
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
        let args: ShellArgs =
            serde_json::from_value(args).map_err(|e| format!("Invalid arguments: {e}"))?;

        let timeout_secs = args.timeout.unwrap_or(30);

        info!(
            command = %args.command,
            cwd = ?args.cwd,
            timeout_secs,
            "shell: executing"
        );

        #[cfg(target_os = "windows")]
        let mut cmd = if let Some(power_shell) = explicit_powershell_command(&args.command) {
            info!(
                executable = power_shell.executable,
                "shell: executing explicit PowerShell command directly"
            );
            let mut c = Command::new(power_shell.executable);
            c.arg(power_shell.command_switch).arg(power_shell.script);
            c
        } else if let Some(bash) = windows_bash_path() {
            info!(bash = %bash.display(), "shell: using Unix-compatible Windows shell");
            let mut c = Command::new(&bash);
            c.arg("-lc").arg(&args.command);
            add_windows_shell_paths(&mut c, &bash);
            c
        } else {
            warn!("shell: Git Bash/MSYS2/Cygwin not found; falling back to cmd.exe");
            let mut c = Command::new("cmd");
            c.arg("/C").arg(&args.command);
            c
        };
        #[cfg(not(target_os = "windows"))]
        let mut cmd = {
            let mut c = Command::new("sh");
            c.arg("-c").arg(&args.command);
            c
        };

        // No CREATE_NO_WINDOW here: the server allocates one hidden console
        // at startup (see allocate_hidden_console in vterm-server), and the
        // shell must inherit it so console-subsystem grandchildren (bash
        // spawning node/npm, …) attach to it instead of flashing a window.

        if let Some(cwd) = &args.cwd {
            cmd.current_dir(cwd);
        }
        // Git commands the agent runs must not take `.git/index.lock` just
        // to refresh the stat cache — the server's own `git status`
        // refreshes would fail with lock contention, and the client treats
        // such errors as "clear the whole Git panel".
        cmd.env("GIT_OPTIONAL_LOCKS", "0");

        let output =
            tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), cmd.output())
                .await
                .map_err(|_| {
                    warn!(command = %args.command, timeout_secs, "shell: timed out");
                    format!("Command timed out after {timeout_secs}s")
                })?
                .map_err(|e| format!("Command failed: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            let truncated =
                is_git_command(&args.command) && stdout.len() > SHELL_OUTPUT_TRUNCATE_AT;
            let result = truncate_shell_output(&args.command, &stdout);
            debug!(
                command = %args.command,
                exit_code = output.status.code(),
                stdout_len = stdout.len(),
                truncated,
                "shell: completed"
            );
            if result.is_empty() && !stderr.is_empty() {
                Ok(format!("(stderr: {})", stderr.trim()))
            } else {
                Ok(result)
            }
        } else {
            warn!(
                command = %args.command,
                exit_code = output.status.code(),
                stderr_len = stderr.len(),
                "shell: failed"
            );
            Err(format!(
                "Exit code {:?}\nstdout: {}\nstderr: {}",
                output.status.code(),
                stdout.trim(),
                stderr.trim()
            ))
        }
    }
}

pub fn register_all(registry: &mut super::ToolRegistry) {
    registry.register(Box::new(FileReadTool));
    registry.register(Box::new(FileFindTool));
    registry.register(Box::new(CodeSearchTool));
    registry.register(Box::new(ShellTool));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_git_output_is_not_truncated() {
        let out = truncate_shell_output("git diff", "hello world");
        assert_eq!(out, "hello world");
    }

    #[test]
    fn oversized_git_output_keeps_first_1k_and_marks_truncation() {
        let big = "x".repeat(200_000); // > 128K
        let out = truncate_shell_output("git diff", &big);
        assert!(out.contains("[output truncated: 200000 bytes total, kept first 1024]"));
        // The kept sample is far smaller than the original.
        assert!(out.len() < big.len() / 10);
    }

    #[test]
    fn oversized_git_output_char_boundary_safe() {
        // Input begins with a multibyte sequence; truncation must not panic and
        // must not split a UTF-8 codepoint (chars().take is boundary-safe).
        let prefix = "日本語".repeat(400);
        let big = format!("{}{}", prefix, "y".repeat(200_000));
        let out = truncate_shell_output("git log", &big);
        assert!(out.contains("[output truncated"));
        assert!(out.starts_with("日"));
    }

    #[test]
    fn non_git_output_is_never_truncated() {
        let big = "x".repeat(200_000); // > 128K, but not a git command
        let out = truncate_shell_output("cat large.log", &big);
        assert_eq!(out, big);
        assert!(!out.contains("[output truncated"));
    }

    #[test]
    fn is_git_command_detects_various_forms() {
        assert!(is_git_command("git diff"));
        assert!(is_git_command("git.exe diff"));
        assert!(is_git_command("cd repo && git diff --stat"));
        assert!(is_git_command("git diff | head"));
        assert!(is_git_command("/usr/bin/git log"));
        assert!(is_git_command("echo hi; git status"));
    }

    #[test]
    fn is_git_command_rejects_non_git() {
        assert!(!is_git_command("cargo build"));
        assert!(!is_git_command("cat file.txt"));
        assert!(!is_git_command("digit --version")); // contains "git" substring
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn explicit_powershell_bypasses_bash_without_losing_variables() {
        let command = r#"powershell -Command "$p='C:\work\1.txt'; $f=Get-Item -LiteralPath $p; Write-Output ('Length: ' + $f.Length)""#;
        let parsed = explicit_powershell_command(command).expect("PowerShell command");

        assert_eq!(parsed.executable, "powershell");
        assert_eq!(parsed.command_switch, "-Command");
        assert!(parsed.script.contains("$p='C:\\work\\1.txt'"));
        assert!(parsed.script.contains("$f.Length"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn ordinary_windows_command_still_uses_default_shell() {
        assert!(explicit_powershell_command("git status --short").is_none());
        assert!(explicit_powershell_command("echo powershell -Command test").is_none());
    }
}
