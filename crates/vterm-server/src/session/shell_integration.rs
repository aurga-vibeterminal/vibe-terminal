use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use vterm_common::SessionId;

use super::session_core::ShellKind;
use super::session_error::SessionError;

/// Holds temp file paths and env modifications needed for shell integration.
pub(super) struct ShellIntegration {
    /// Temp files/dirs to clean up on session close.
    temp_paths: Vec<PathBuf>,
    /// Environment variables to inject into the PTY command.
    env_additions: Vec<(String, String)>,
    /// If set, replace the program args with these (e.g., fish --init-command).
    args_override: Option<Vec<String>>,
}

impl ShellIntegration {
    /// Remove all temp files created for this integration.
    pub(super) fn cleanup(&self) {
        for path in &self.temp_paths {
            if path.is_dir() {
                // Remove files inside first, then the directory
                let _ = fs::remove_dir_all(path);
            } else {
                let _ = fs::remove_file(path);
            }
        }
    }

    pub(super) fn env_additions(&self) -> &[(String, String)] {
        &self.env_additions
    }

    pub(super) fn args_override(&self) -> Option<&[String]> {
        self.args_override.as_deref()
    }

    pub(super) fn temp_paths(&self) -> &[PathBuf] {
        &self.temp_paths
    }
}

/// Detect the shell type and prepare integration if applicable.
///
/// Returns `Ok(None)` if the shell doesn't support integration or isn't recognized,
/// in which case no injection should occur (prompt parsing remains as fallback).
pub(super) fn prepare_integration(
    session_id: SessionId,
    shell_kind: Option<ShellKind>,
    _program: &str,
    args: &[String],
    cwd: Option<&str>,
    env: &HashMap<String, String>,
) -> Result<Option<ShellIntegration>, SessionError> {
    let kind = match shell_kind {
        Some(ShellKind::Zsh)
        | Some(ShellKind::Bash)
        | Some(ShellKind::Fish)
        | Some(ShellKind::PowerShell) => shell_kind.unwrap(),
        _ => return Ok(None),
    };

    // Allow user to disable integration
    if env.get("VTERM_SHELL_INTEGRATION").is_some_and(|v| v == "0") {
        return Ok(None);
    }

    match kind {
        ShellKind::Zsh => prepare_zsh_integration(session_id, cwd, env),
        ShellKind::Bash => prepare_bash_integration(session_id, cwd, args, env),
        ShellKind::Fish => prepare_fish_integration(session_id, cwd, args),
        ShellKind::PowerShell => prepare_powershell_integration(session_id, cwd, args),
        _ => Ok(None),
    }
}

fn integration_temp_path(session_id: SessionId, label: &str, extension: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!(
        "vterm-shell-{label}-{session_id}-{nonce}.{extension}"
    ))
}

fn shell_single_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}

fn powershell_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

// ---------------------------------------------------------------------------
// Zsh: inject via ZDOTDIR
// ---------------------------------------------------------------------------

fn prepare_zsh_integration(
    session_id: SessionId,
    cwd: Option<&str>,
    env: &HashMap<String, String>,
) -> Result<Option<ShellIntegration>, SessionError> {
    let temp_dir = integration_temp_path(session_id, "zdotdir", "d");
    fs::create_dir_all(&temp_dir).map_err(|e| SessionError::FileSystemError(e.to_string()))?;

    // Determine the user's real ZDOTDIR (or HOME)
    let real_zdotdir = env
        .get("ZDOTDIR")
        .cloned()
        .or_else(|| std::env::var("ZDOTDIR").ok())
        .or_else(|| std::env::var("HOME").ok())
        .unwrap_or_else(|| "/".to_string());

    let zshenv_path = temp_dir.join(".zshenv");
    let zprofile_path = temp_dir.join(".zprofile");
    let zshrc_path = temp_dir.join(".zshrc");
    let zlogin_path = temp_dir.join(".zlogin");

    write_script(&zshenv_path, &generate_zshenv_bridge(&real_zdotdir))?;
    write_script(&zprofile_path, &generate_zprofile_bridge(&real_zdotdir))?;
    write_script(&zshrc_path, &generate_zshrc_integration(&real_zdotdir, cwd))?;
    write_script(&zlogin_path, &generate_zlogin_bridge(&real_zdotdir))?;

    let zdotdir_value = temp_dir.to_string_lossy().into_owned();
    Ok(Some(ShellIntegration {
        temp_paths: vec![
            zshenv_path,
            zprofile_path,
            zshrc_path,
            zlogin_path,
            temp_dir,
        ],
        env_additions: vec![("ZDOTDIR".to_string(), zdotdir_value)],
        args_override: None,
    }))
}

fn generate_zshenv_bridge(real_zdotdir: &str) -> String {
    let escaped_zdotdir = shell_single_quote(real_zdotdir);

    format!(
        r#"# vterm shell integration
export VTERM_REAL_ZDOTDIR='{escaped_zdotdir}'

if [ -f "$VTERM_REAL_ZDOTDIR/.zshenv" ]; then
  . "$VTERM_REAL_ZDOTDIR/.zshenv"
fi
"#
    )
}

fn generate_zprofile_bridge(real_zdotdir: &str) -> String {
    let escaped_zdotdir = shell_single_quote(real_zdotdir);

    format!(
        r#"# vterm shell integration
export VTERM_REAL_ZDOTDIR='{escaped_zdotdir}'

if [ -f "$VTERM_REAL_ZDOTDIR/.zprofile" ]; then
  . "$VTERM_REAL_ZDOTDIR/.zprofile"
fi
"#
    )
}

fn generate_zshrc_integration(real_zdotdir: &str, cwd: Option<&str>) -> String {
    let escaped_zdotdir = shell_single_quote(real_zdotdir);
    let initial_osc7 = initial_osc7_expr(cwd);

    format!(
        r#"# vterm shell integration
export VTERM_REAL_ZDOTDIR='{escaped_zdotdir}'

# zsh has already read .zshenv/.zprofile and is reading this file from the
# temp ZDOTDIR. Restore the real one now so anything inspecting $ZDOTDIR later
# (installer scripts, compinit's .zcompdump, manually spawned child shells)
# finds the user's actual rc files instead of this ephemeral directory. zsh
# then reads .zlogin from the real ZDOTDIR directly, which is equivalent to
# the bridge. This must NOT happen in .zshenv: restoring there would make zsh
# look for .zshrc in the real ZDOTDIR and skip this integration entirely.
export ZDOTDIR="$VTERM_REAL_ZDOTDIR"

if [ -f "$VTERM_REAL_ZDOTDIR/.zshrc" ]; then
  . "$VTERM_REAL_ZDOTDIR/.zshrc"
fi

__vterm_osc7() {{
  printf '\033]7;file://%s%s\007' "$HOST" "$PWD"
}}

__vterm_urlencode() {{
  emulate -L zsh
  local input="$1" output="" char hex i
  for (( i = 1; i <= ${{#input}}; i++ )); do
    char="${{input[i]}}"
    case "$char" in
      [A-Za-z0-9.~_-]) output+="$char" ;;
      *) hex=$(printf '%02X' "'$char"); output+="%$hex" ;;
    esac
  done
  print -rn -- "$output"
}}

__vterm_command_start() {{
  local cmdline="$1"
  [ -n "$cmdline" ] || return
  local encoded
  encoded="$(__vterm_urlencode "$cmdline")"
  printf '\033]133;C;aid=%s;cmdline_url=%s\007' "$$" "$encoded"
  __vterm_executing=1
}}

__vterm_command_end() {{
  local exit_code="$1"
  [ -n "${{__vterm_executing:-}}" ] || return
  printf '\033]133;D;%s;aid=%s\007' "$exit_code" "$$"
  unset __vterm_executing
}}

__vterm_preexec() {{
  __vterm_command_start "$1"
}}

__vterm_precmd() {{
  local exit_code="$?"
  __vterm_command_end "$exit_code"
  __vterm_osc7
}}

chpwd_functions=(__vterm_osc7 $chpwd_functions)
preexec_functions=(__vterm_preexec ${{preexec_functions:#__vterm_preexec}})
precmd_functions=(__vterm_precmd ${{precmd_functions:#__vterm_precmd}})

# vterm replays raw session history when a viewer attaches or the terminal
# is rebuilt. zsh's PROMPT_SP partial-line padding emits a width-sensitive
# "% <fill spaces> CR" sequence before every prompt; replayed at any width
# different from the emission width it leaves stray inverse "%" lines (or
# blank spacer lines) burned into scrollback. Disable the whole mechanism
# for vterm sessions. PROMPT_EOL_MARK='' is a fallback for user hooks that
# re-enable prompt_sp: the padding then stays invisible.
unsetopt prompt_sp
PROMPT_EOL_MARK=''

{initial_osc7}
"#
    )
}

fn generate_zlogin_bridge(real_zdotdir: &str) -> String {
    let escaped_zdotdir = shell_single_quote(real_zdotdir);

    format!(
        r#"# vterm shell integration
export VTERM_REAL_ZDOTDIR='{escaped_zdotdir}'

if [ -f "$VTERM_REAL_ZDOTDIR/.zlogin" ]; then
  . "$VTERM_REAL_ZDOTDIR/.zlogin"
fi
"#
    )
}

// ---------------------------------------------------------------------------
// Bash: inject via --rcfile so child non-interactive bash scripts are not polluted.
// ---------------------------------------------------------------------------

fn prepare_bash_integration(
    session_id: SessionId,
    cwd: Option<&str>,
    args: &[String],
    env: &HashMap<String, String>,
) -> Result<Option<ShellIntegration>, SessionError> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--rcfile" | "--init-file" | "-c"))
    {
        return Ok(None);
    }

    let script_path = integration_temp_path(session_id, "bashrc", "sh");
    let home = env
        .get("HOME")
        .cloned()
        .or_else(|| std::env::var("HOME").ok());
    let script_content = generate_bash_integration(home.as_deref(), cwd);
    write_script(&script_path, &script_content)?;

    let mut new_args = Vec::with_capacity(args.len() + 3);
    new_args.push("--rcfile".to_string());
    new_args.push(script_path.to_string_lossy().into_owned());
    new_args.push("-i".to_string());
    new_args.extend(args.iter().cloned());

    Ok(Some(ShellIntegration {
        temp_paths: vec![script_path.clone()],
        env_additions: Vec::new(),
        args_override: Some(new_args),
    }))
}

const BASH_LOGIN_ENV_MARKER: &[u8] = b"\x1eVTERM_LOGIN_ENV\x1f";
const BASH_LOGIN_ENV_EXCLUDED: &[&str] = &[
    "PWD",
    "OLDPWD",
    "SHLVL",
    "BASHPID",
    "BASH_SUBSHELL",
    "BASH_EXECUTION_STRING",
    "_",
];

/// Collect exported login variables in a short-lived process. The interactive
/// shell is started separately with --rcfile, so exiting it never returns to a
/// second interactive login shell.
pub(super) fn bash_login_environment(
    program: &str,
    env: &HashMap<String, String>,
) -> HashMap<String, String> {
    let Ok(output) = Command::new(program)
        .args([
            "--login",
            "-c",
            "printf '\\036VTERM_LOGIN_ENV\\037' && env -0",
        ])
        .envs(env)
        .stdin(Stdio::null())
        .output()
    else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }
    let Some(marker) = output
        .stdout
        .windows(BASH_LOGIN_ENV_MARKER.len())
        .position(|window| window == BASH_LOGIN_ENV_MARKER)
    else {
        return HashMap::new();
    };
    parse_bash_login_environment(&output.stdout[marker + BASH_LOGIN_ENV_MARKER.len()..])
}

fn parse_bash_login_environment(data: &[u8]) -> HashMap<String, String> {
    data.split(|byte| *byte == 0)
        .filter_map(|entry| {
            let entry = std::str::from_utf8(entry).ok()?;
            let (name, value) = entry.split_once('=')?;
            let mut bytes = name.bytes();
            if !bytes
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
                || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                || BASH_LOGIN_ENV_EXCLUDED.contains(&name)
            {
                return None;
            }
            Some((name.to_string(), value.to_string()))
        })
        .collect()
}

fn generate_bash_integration(home: Option<&str>, cwd: Option<&str>) -> String {
    let initial_osc7 = initial_osc7_expr(cwd);
    let source_user_bashrc = home
        .map(|home| format_bash_source_line(&Path::new(home).join(".bashrc")))
        .unwrap_or_default();

    format!(
        r#"# vterm shell integration
if [ -f /etc/bash.bashrc ]; then
  . /etc/bash.bashrc
fi
{source_user_bashrc}

__vterm_osc7() {{
  printf '\033]7;file://%s%s\007' "$HOSTNAME" "$PWD"
}}

__vterm_urlencode() {{
  local input="$1" output="" char hex i
  local LC_ALL=C
  for (( i = 0; i < ${{#input}}; i++ )); do
    char="${{input:i:1}}"
    case "$char" in
      [A-Za-z0-9.~_-]) output+="$char" ;;
      *) hex=$(printf '%02X' "'$char"); output+="%$hex" ;;
    esac
  done
  printf '%s' "$output"
}}

__vterm_command_start() {{
  local cmdline="$1"
  case "$cmdline" in
    ""|__vterm_*|__vte_*|case\ *|PROMPT_COMMAND=*) return ;;
  esac
  local encoded
  encoded="$(__vterm_urlencode "$cmdline")"
  printf '\033]133;C;aid=%s;cmdline_url=%s\007' "$$" "$encoded"
  __vterm_executing=1
}}

__vterm_command_end() {{
  local exit_code="$1"
  [ -n "${{__vterm_executing:-}}" ] || return
  printf '\033]133;D;%s;aid=%s\007' "$exit_code" "$$"
  unset __vterm_executing
}}

__vterm_prompt_command() {{
  local exit_code="$?"
  __vterm_command_end "$exit_code"
  __vterm_osc7
  return "$exit_code"
}}

trap '__vterm_command_start "$BASH_COMMAND"' DEBUG

case ";${{PROMPT_COMMAND:-}};" in
  *";__vterm_prompt_command;"*) ;;
  *)
    if [ -n "${{PROMPT_COMMAND:-}}" ]; then
      PROMPT_COMMAND="__vterm_prompt_command; $PROMPT_COMMAND"
    else
      PROMPT_COMMAND="__vterm_prompt_command"
    fi
    ;;
esac
export PROMPT_COMMAND

{initial_osc7}
"#
    )
}

// ---------------------------------------------------------------------------
// Fish: inject via --init-command
// ---------------------------------------------------------------------------

fn prepare_fish_integration(
    session_id: SessionId,
    cwd: Option<&str>,
    args: &[String],
) -> Result<Option<ShellIntegration>, SessionError> {
    let script_path = integration_temp_path(session_id, "fish", "fish");
    let script_content = generate_fish_integration(cwd);
    write_script(&script_path, &script_content)?;

    let mut new_args = args.to_vec();
    new_args.push("--init-command".to_string());
    new_args.push(script_path.to_string_lossy().into_owned());

    Ok(Some(ShellIntegration {
        temp_paths: vec![script_path],
        env_additions: Vec::new(),
        args_override: Some(new_args),
    }))
}

fn generate_fish_integration(cwd: Option<&str>) -> String {
    let initial_osc7 = initial_osc7_expr_fish(cwd);

    format!(
        r#"# vterm shell integration
function __vterm_osc7 --on-variable PWD
    printf '\033]7;file://%s%s\007' "$hostname" "$PWD"
end

function __vterm_urlencode
    string escape --style=url -- "$argv[1]" 2>/dev/null; or string replace -a ' ' '%20' -- "$argv[1]"
end

function __vterm_command_start --on-event fish_preexec
    set -l cmdline "$argv[1]"
    test -n "$cmdline"; or return
    set -l encoded (__vterm_urlencode "$cmdline")
    printf '\033]133;C;aid=%s;cmdline_url=%s\007' "$fish_pid" "$encoded"
    set -g __vterm_executing 1
end

function __vterm_command_end --on-event fish_postexec
    set -q __vterm_executing; or return
    set -l exit_code $status
    printf '\033]133;D;%s;aid=%s\007' "$exit_code" "$fish_pid"
    set -e __vterm_executing
end

{initial_osc7}
"#
    )
}

// ---------------------------------------------------------------------------
// PowerShell: inject via a temporary profile bootstrap script
// ---------------------------------------------------------------------------

/// Prepare PowerShell integration by launching a temporary bootstrap script.
fn prepare_powershell_integration(
    session_id: SessionId,
    cwd: Option<&str>,
    args: &[String],
) -> Result<Option<ShellIntegration>, SessionError> {
    if has_powershell_unsupported_args(args) {
        return Ok(None);
    }

    let script_path = integration_temp_path(session_id, "powershell", "ps1");
    // Windows PowerShell 5.1 decodes BOM-less scripts with the system ANSI
    // codepage (e.g. GBK), which mangles non-ASCII paths embedded in the
    // script. A UTF-8 BOM forces UTF-8 decoding on both 5.1 and pwsh 7.
    write_script_utf8_bom(&script_path, &generate_powershell_integration(cwd))?;

    let mut new_args = filtered_powershell_passthrough_args(args);
    ensure_powershell_flag(&mut new_args, "-NoExit");
    ensure_powershell_flag(&mut new_args, "-NoLogo");
    ensure_powershell_flag(&mut new_args, "-NoProfile");
    if !new_args
        .iter()
        .any(|arg| arg.eq_ignore_ascii_case("-ExecutionPolicy"))
    {
        new_args.push("-ExecutionPolicy".to_string());
        new_args.push("Bypass".to_string());
    }
    new_args.push("-File".to_string());
    new_args.push(script_path.to_string_lossy().into_owned());

    Ok(Some(ShellIntegration {
        temp_paths: vec![script_path.clone()],
        env_additions: Vec::new(),
        args_override: Some(new_args),
    }))
}

/// Generate a bootstrap script that restores user profiles and installs hooks.
fn generate_powershell_integration(cwd: Option<&str>) -> String {
    let initial_osc7 = initial_osc7_expr_powershell(cwd);
    format!(
        r#"# vterm shell integration
$global:__vtermEsc = [char]27
$global:__vtermBell = [char]7
$global:__vtermExecuting = $false
$global:__vtermLastReportedCwd = $null

$__vtermProfileCandidates = @(
    $PROFILE.AllUsersAllHosts,
    $PROFILE.AllUsersCurrentHost,
    $PROFILE.CurrentUserAllHosts,
    $PROFILE.CurrentUserCurrentHost
) | Where-Object {{ $_ -and $_ -ne $PSCommandPath }} | Select-Object -Unique

foreach ($__vtermProfilePath in $__vtermProfileCandidates) {{
    if (Test-Path -LiteralPath $__vtermProfilePath) {{
        . $__vtermProfilePath
    }}
}}

function global:__vterm_write_osc([string]$payload) {{
    [Console]::Out.Write("$global:__vtermEsc]$payload$global:__vtermBell")
}}

function global:__vterm_emit_osc7([string]$path) {{
    if ([string]::IsNullOrWhiteSpace($path)) {{
        return
    }}

    $hostName = if ([string]::IsNullOrWhiteSpace($env:COMPUTERNAME)) {{ "localhost" }} else {{ $env:COMPUTERNAME }}
    $normalized = $path -replace '\\', '/'
    if ($normalized -match '^[A-Za-z]:/') {{
        $normalized = '/' + $normalized
    }}

    $encoded = [System.Uri]::EscapeDataString($normalized).Replace('%2F', '/')
    __vterm_write_osc("7;file://$hostName$encoded")
}}

function global:__vterm_emit_current_cwd() {{
    try {{
        $location = Get-Location
        if (-not $location -or $location.Provider.Name -ne 'FileSystem') {{
            return
        }}

        $cwdNow = $null
        try {{
            $cwdNow = Convert-Path -LiteralPath $location.Path
        }} catch {{
            $cwdNow = $location.Path
        }}

        if (-not [string]::IsNullOrWhiteSpace($cwdNow) -and $cwdNow -ne $global:__vtermLastReportedCwd) {{
            $global:__vtermLastReportedCwd = $cwdNow
            __vterm_emit_osc7 $cwdNow
        }}
    }} catch {{
    }}
}}

function global:__vterm_emit_command_start([string]$cmdline) {{
    if ([string]::IsNullOrWhiteSpace($cmdline)) {{
        return
    }}

    $encoded = [System.Uri]::EscapeDataString($cmdline)
    __vterm_write_osc("133;C;aid=$PID;cmdline_url=$encoded")
    $global:__vtermExecuting = $true
}}

function global:__vterm_emit_command_end([int]$exitCode) {{
    __vterm_write_osc("133;D;$exitCode;aid=$PID")
}}

function global:Set-Location {{
    $result = Microsoft.PowerShell.Management\Set-Location @args
    if ($?) {{
        __vterm_emit_current_cwd
    }}
    return $result
}}

function global:Push-Location {{
    $result = Microsoft.PowerShell.Management\Push-Location @args
    if ($?) {{
        __vterm_emit_current_cwd
    }}
    return $result
}}

function global:Pop-Location {{
    $result = Microsoft.PowerShell.Management\Pop-Location @args
    if ($?) {{
        __vterm_emit_current_cwd
    }}
    return $result
}}

Import-Module PSReadLine -ErrorAction SilentlyContinue | Out-Null

if (Get-Command -Name PSConsoleHostReadLine -CommandType Function -ErrorAction SilentlyContinue) {{
    $global:__vtermOriginalReadLine = $function:PSConsoleHostReadLine
    function global:PSConsoleHostReadLine {{
        $line = & $global:__vtermOriginalReadLine
        if (-not [string]::IsNullOrWhiteSpace($line)) {{
            __vterm_emit_command_start $line
        }}
        return $line
    }}
}}

$global:__vtermOriginalPrompt = $function:prompt
function global:prompt {{
    $success = $?
    $nativeExit = $global:LASTEXITCODE
    if ($global:__vtermExecuting) {{
        $exitCode = if ($success) {{
            0
        }} elseif ($nativeExit -is [int] -and $nativeExit -ne 0) {{
            $nativeExit
        }} else {{
            1
        }}
        __vterm_emit_command_end $exitCode
    }}

    __vterm_write_osc("133;A;aid=$PID")
    $global:__vtermExecuting = $false
    & $global:__vtermOriginalPrompt
}}

{initial_osc7}
"#
    )
}

/// Return true when the PowerShell invocation is non-interactive or custom.
fn has_powershell_unsupported_args(args: &[String]) -> bool {
    args.iter().any(|arg| {
        matches!(
            arg.to_ascii_lowercase().as_str(),
            "-command"
                | "-c"
                | "-encodedcommand"
                | "-ec"
                | "-encodedarguments"
                | "-ea"
                | "-file"
                | "-f"
                | "-noninteractive"
        )
    })
}

/// Preserve harmless launch flags while rebuilding the interactive bootstrap.
fn filtered_powershell_passthrough_args(args: &[String]) -> Vec<String> {
    args.iter()
        .filter(|arg| {
            matches!(
                arg.to_ascii_lowercase().as_str(),
                "-nologo" | "-noexit" | "-noprofile" | "-sta" | "-mta" | "-login"
            )
        })
        .cloned()
        .collect()
}

/// Ensure the rebuilt PowerShell argument list contains a specific switch.
fn ensure_powershell_flag(args: &mut Vec<String>, flag: &str) {
    if !args.iter().any(|arg| arg.eq_ignore_ascii_case(flag)) {
        args.push(flag.to_string());
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate the initial OSC 7 emission expression for POSIX shells.
fn initial_osc7_expr(cwd: Option<&str>) -> String {
    match cwd {
        Some(dir) => {
            let escaped = shell_single_quote(dir);
            format!("printf '\\033]7;file://%s%s\\007' \"$HOSTNAME\" '{escaped}'")
        }
        None => "__vterm_osc7".to_string(),
    }
}

/// Generate the initial OSC 7 emission expression for fish.
fn initial_osc7_expr_fish(cwd: Option<&str>) -> String {
    match cwd {
        Some(dir) => {
            let escaped = dir.replace('\\', "\\\\").replace('\'', "\\'");
            format!("printf '\\033]7;file://%s%s\\007' \"$hostname\" '{escaped}'")
        }
        None => "__vterm_osc7".to_string(),
    }
}

/// Generate the initial OSC 7 emission expression for PowerShell.
fn initial_osc7_expr_powershell(cwd: Option<&str>) -> String {
    match cwd {
        Some(dir) => {
            let escaped = powershell_single_quote(dir);
            format!("__vterm_emit_osc7 '{}'", escaped)
        }
        None => "__vterm_emit_osc7 ((Get-Location).Path)".to_string(),
    }
}

fn format_bash_source_line(path: &Path) -> String {
    let escaped = shell_single_quote(&path.to_string_lossy());
    format!("if [ -f '{escaped}' ]; then\n  . '{escaped}'\nfi")
}

/// Write a script file with restrictive permissions (Unix only).
fn write_script(path: &Path, content: &str) -> Result<(), SessionError> {
    fs::write(path, content).map_err(|e| SessionError::FileSystemError(e.to_string()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|e| SessionError::FileSystemError(e.to_string()))?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)
            .map_err(|e| SessionError::FileSystemError(e.to_string()))?;
    }

    Ok(())
}

/// Write a script file prefixed with a UTF-8 BOM so Windows PowerShell 5.1
/// decodes it as UTF-8 instead of the system ANSI codepage. POSIX shells must
/// not use this — a leading BOM breaks their script parsing.
fn write_script_utf8_bom(path: &Path, content: &str) -> Result<(), SessionError> {
    let mut bytes = Vec::with_capacity(content.len() + 3);
    bytes.extend_from_slice(b"\xEF\xBB\xBF");
    bytes.extend_from_slice(content.as_bytes());
    fs::write(path, bytes).map_err(|e| SessionError::FileSystemError(e.to_string()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|e| SessionError::FileSystemError(e.to_string()))?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)
            .map_err(|e| SessionError::FileSystemError(e.to_string()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_integration_uses_rcfile_instead_of_bash_env() {
        let session_id = SessionId::new();
        let mut env = HashMap::new();
        env.insert("HOME".to_string(), "/tmp/test-home".to_string());

        let integration = prepare_bash_integration(session_id, Some("/tmp/project"), &[], &env)
            .expect("bash integration should succeed")
            .expect("bash integration should be enabled");

        assert!(integration.env_additions().is_empty());
        let args = integration
            .args_override()
            .expect("bash should override args");
        assert_eq!(args.first().map(String::as_str), Some("--rcfile"));
        assert_eq!(args.get(2).map(String::as_str), Some("-i"));
        assert_eq!(args.len(), 3);
    }

    #[test]
    fn bash_login_environment_parser_filters_shell_session_state() {
        let parsed = parse_bash_login_environment(
            b"PATH=/login/bin\0VTERM_TEST=value=with=equals\0PWD=/do/not/copy\0SHLVL=4\0INVALID-NAME=x\0NO_EQUALS\0_=bash\0",
        );
        assert_eq!(parsed.get("PATH").map(String::as_str), Some("/login/bin"));
        assert_eq!(
            parsed.get("VTERM_TEST").map(String::as_str),
            Some("value=with=equals")
        );
        assert!(!parsed.contains_key("PWD"));
        assert!(!parsed.contains_key("SHLVL"));
        assert!(!parsed.contains_key("INVALID-NAME"));
        assert!(!parsed.contains_key("_"));
    }

    #[cfg(unix)]
    #[test]
    fn bash_login_environment_reads_profile_and_ignores_profile_output() {
        let home = integration_temp_path(SessionId::new(), "login-home", "d");
        fs::create_dir_all(&home).expect("test home should be created");
        fs::write(
            home.join(".bash_profile"),
            "echo profile output\nexport VTERM_LOGIN_TEST=from-login\n",
        )
        .expect("test profile should be written");
        let env = HashMap::from([("HOME".to_string(), home.to_string_lossy().into_owned())]);

        let parsed = bash_login_environment("bash", &env);
        assert_eq!(
            parsed.get("VTERM_LOGIN_TEST").map(String::as_str),
            Some("from-login")
        );
        assert!(!parsed.contains_key("PWD"));

        fs::remove_dir_all(home).expect("test home should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn bash_login_environment_does_not_enable_interactive_job_control() {
        use std::os::unix::fs::PermissionsExt;

        let dir = integration_temp_path(SessionId::new(), "login-command", "d");
        fs::create_dir_all(&dir).expect("test directory should be created");
        let shell = dir.join("bash");
        fs::write(
            &shell,
            "#!/bin/sh\ncase \" $* \" in *\" -i \"*) exit 1;; esac\nprintf '\\036VTERM_LOGIN_ENV\\037'\nenv -0\n",
        )
        .expect("test shell should be written");
        let mut permissions = fs::metadata(&shell).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&shell, permissions).unwrap();

        let parsed = bash_login_environment(
            shell.to_str().unwrap(),
            &HashMap::from([("VTERM_LOGIN_TEST".to_string(), "value".to_string())]),
        );
        fs::remove_dir_all(dir).unwrap();
        assert_eq!(
            parsed.get("VTERM_LOGIN_TEST").map(String::as_str),
            Some("value")
        );
    }

    #[test]
    fn generated_bash_integration_sources_user_bashrc_and_avoids_duplicate_hook() {
        let script = generate_bash_integration(Some("/tmp/test-home"), Some("/tmp/project"));
        assert!(script.contains(". '/tmp/test-home/.bashrc'"));
        assert!(script.contains("case \";${PROMPT_COMMAND:-};\" in"));
        assert!(script.contains("trap '__vterm_command_start \"$BASH_COMMAND\"' DEBUG"));
        assert!(script.contains("\"\"|__vterm_*|__vte_*|case\\ *|PROMPT_COMMAND=*) return"));
        assert!(script.contains("printf '\\033]133;C;aid=%s;cmdline_url=%s\\007'"));
        assert!(script.contains("printf '\\033]133;D;%s;aid=%s\\007'"));
        assert!(script.contains("printf '\\033]7;file://%s%s\\007' \"$HOSTNAME\" '/tmp/project'"));
    }

    #[test]
    fn generated_zsh_env_bridge_does_not_emit_osc_sequences() {
        let script = generate_zshenv_bridge("/tmp/test-home");
        assert!(script.contains(". \"$VTERM_REAL_ZDOTDIR/.zshenv\""));
        assert!(!script.contains("printf '\\033]7;file://"));
    }

    #[test]
    fn generated_zsh_rc_integration_emits_osc_sequences_only_from_zshrc() {
        let script = generate_zshrc_integration("/tmp/test-home", Some("/tmp/project"));
        assert!(script.contains(". \"$VTERM_REAL_ZDOTDIR/.zshrc\""));
        assert!(script.contains("chpwd_functions=(__vterm_osc7 $chpwd_functions)"));
        assert!(script.contains("preexec_functions=(__vterm_preexec"));
        assert!(script.contains("precmd_functions=(__vterm_precmd"));
        assert!(script.contains("printf '\\033]133;C;aid=%s;cmdline_url=%s\\007'"));
        assert!(script.contains("printf '\\033]133;D;%s;aid=%s\\007'"));
        assert!(script.contains("printf '\\033]7;file://%s%s\\007' \"$HOSTNAME\" '/tmp/project'"));
    }

    #[test]
    fn generated_zsh_rc_integration_restores_real_zdotdir_before_sourcing() {
        let script = generate_zshrc_integration("/tmp/test-home", Some("/tmp/project"));
        // Installers and compinit inspect $ZDOTDIR; they must see the user's
        // real rc directory, not the ephemeral injection dir. The restore must
        // happen before the user's .zshrc is sourced so compinit writes its
        // .zcompdump there too.
        let restore_pos = script
            .find("export ZDOTDIR=\"$VTERM_REAL_ZDOTDIR\"")
            .unwrap();
        let source_pos = script.find(". \"$VTERM_REAL_ZDOTDIR/.zshrc\"").unwrap();
        assert!(restore_pos < source_pos);
        // The other startup bridges must NOT restore ZDOTDIR: doing so in
        // .zshenv would make zsh skip this injected .zshrc entirely.
        assert!(!generate_zshenv_bridge("/tmp/test-home").contains("export ZDOTDIR="));
        assert!(!generate_zprofile_bridge("/tmp/test-home").contains("export ZDOTDIR="));
    }

    #[test]
    fn generated_zsh_rc_integration_disables_prompt_sp_eol_mark() {
        let script = generate_zshrc_integration("/tmp/test-home", Some("/tmp/project"));
        // The width-sensitive PROMPT_SP padding replays as stray inverse "%"
        // lines whenever the replay width differs from the emission width.
        assert!(script.contains("unsetopt prompt_sp"));
        assert!(script.contains("PROMPT_EOL_MARK=''"));
        // The overrides must come after the user's own .zshrc so they win.
        let source_pos = script.find(". \"$VTERM_REAL_ZDOTDIR/.zshrc\"").unwrap();
        let unset_pos = script.find("unsetopt prompt_sp").unwrap();
        assert!(unset_pos > source_pos);
    }

    #[test]
    fn generated_fish_integration_emits_command_start_and_end_markers() {
        let script = generate_fish_integration(Some("/tmp/project"));
        assert!(script.contains("function __vterm_command_start --on-event fish_preexec"));
        assert!(script.contains("function __vterm_command_end --on-event fish_postexec"));
        assert!(script.contains("printf '\\033]133;C;aid=%s;cmdline_url=%s\\007'"));
        assert!(script.contains("printf '\\033]133;D;%s;aid=%s\\007'"));
        assert!(script.contains("printf '\\033]7;file://%s%s\\007' \"$hostname\" '/tmp/project'"));
    }

    #[test]
    fn powershell_integration_uses_bootstrap_file_and_execution_policy_bypass() {
        let session_id = SessionId::new();
        let integration = prepare_powershell_integration(session_id, Some(r"C:\repo"), &[])
            .expect("powershell integration should succeed")
            .expect("powershell integration should be enabled");

        let args = integration
            .args_override()
            .expect("powershell should override args");
        assert!(args.iter().any(|arg| arg.eq_ignore_ascii_case("-NoExit")));
        assert!(args
            .iter()
            .any(|arg| arg.eq_ignore_ascii_case("-NoProfile")));
        assert!(args.iter().any(|arg| arg.eq_ignore_ascii_case("-File")));
        assert!(args
            .iter()
            .any(|arg| arg.eq_ignore_ascii_case("-ExecutionPolicy")));
        assert!(args.iter().any(|arg| arg == "Bypass"));
    }

    #[test]
    fn generated_powershell_integration_wraps_prompt_and_readline() {
        let script = generate_powershell_integration(Some(r"C:\repo"));
        assert!(script.contains("function global:prompt"));
        assert!(script.contains("function global:PSConsoleHostReadLine"));
        assert!(script.contains("function global:Set-Location"));
        assert!(script.contains("function global:Push-Location"));
        assert!(script.contains("function global:Pop-Location"));
        assert!(script.contains("__vterm_emit_current_cwd"));
        assert!(script.contains("__vterm_write_osc(\"133;C;aid=$PID;cmdline_url=$encoded\")"));
        assert!(script.contains("__vterm_write_osc(\"133;D;$exitCode;aid=$PID\")"));
        assert!(script.contains("__vterm_emit_osc7 'C:\\repo'"));
    }

    #[test]
    fn powershell_integration_skips_non_interactive_invocations() {
        assert!(has_powershell_unsupported_args(&[
            "-Command".to_string(),
            "Get-Date".to_string()
        ]));
        assert!(has_powershell_unsupported_args(&[
            "-File".to_string(),
            "script.ps1".to_string()
        ]));
        assert!(!has_powershell_unsupported_args(&[
            "-NoLogo".to_string(),
            "-NoExit".to_string()
        ]));
    }

    #[test]
    fn powershell_bootstrap_script_is_written_with_utf8_bom() {
        // Windows PowerShell 5.1 decodes BOM-less .ps1 files with the system
        // ANSI codepage (GBK on zh-CN), mangling non-ASCII paths embedded in
        // the script. The BOM forces UTF-8 decoding.
        let session_id = SessionId::new();
        let integration =
            prepare_powershell_integration(session_id, Some("F:\\廖桂林\\lgl\\网页"), &[])
                .expect("powershell integration should succeed")
                .expect("powershell integration should be enabled");

        let script_path = integration
            .temp_paths()
            .first()
            .expect("powershell integration should leave a temp script");
        let bytes = fs::read(script_path).expect("bootstrap script should exist");
        assert!(bytes.starts_with(b"\xEF\xBB\xBF"));
        let body = std::str::from_utf8(&bytes[3..]).expect("script body should be UTF-8");
        assert!(body.contains("__vterm_emit_osc7 'F:\\廖桂林\\lgl\\网页'"));

        integration.cleanup();
    }
}
