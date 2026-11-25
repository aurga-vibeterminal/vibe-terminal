use vterm_common::TerminalProfile;

use crate::config::{TerminalServerConfig, UartPortConfig};

use super::session_error::SessionError;

pub(super) struct LaunchProfile {
    pub(super) title: String,
    pub(super) program: String,
    pub(super) args: Vec<String>,
}

#[derive(Clone)]
pub(super) struct UartLaunchProfile {
    pub(super) title: String,
    pub(super) port_config: UartPortConfig,
}

pub(super) enum LaunchTarget {
    Shell(LaunchProfile),
    Uart(UartLaunchProfile),
}

pub(super) fn default_shell() -> String {
    std::env::var("COMSPEC")
        .or_else(|_| std::env::var("SHELL"))
        .ok()
        .filter(|shell| !shell.is_empty())
        .unwrap_or_else(login_shell_fallback)
}

/// Fallback when neither COMSPEC nor SHELL is set — e.g. a server spawned by
/// a GUI client under launchd, which does not propagate a shell environment.
#[cfg(unix)]
fn login_shell_fallback() -> String {
    // The account's login shell from the passwd database.
    unsafe {
        let pw = libc::getpwuid(libc::getuid());
        if !pw.is_null() && !(*pw).pw_shell.is_null() {
            let shell = std::ffi::CStr::from_ptr((*pw).pw_shell).to_string_lossy();
            if !shell.is_empty() {
                return shell.into_owned();
            }
        }
    }
    "/bin/sh".to_string()
}

#[cfg(windows)]
fn login_shell_fallback() -> String {
    "powershell.exe".to_string()
}

/// Args that make the spawned shell a login shell on Unix, so the system and
/// user login profiles run. On macOS `/etc/zprofile` runs `path_helper`,
/// which merges `/etc/paths` and `/etc/paths.d/*` into PATH — required to
/// find tools like `go` when the server was launched from a GUI app with
/// launchd's minimal PATH (`/usr/bin:/bin:/usr/sbin:/sbin`).
///
/// bash is excluded: a login bash ignores `--rcfile`, which the shell
/// integration relies on, so adding `-l` would silently break the OSC hooks.
#[cfg(not(target_os = "windows"))]
fn login_shell_args(program: &str) -> Vec<String> {
    let basename = std::path::Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase();
    match basename.as_str() {
        "zsh" | "fish" => vec!["-l".to_string()],
        _ => Vec::new(),
    }
}

#[cfg(target_os = "windows")]
fn login_shell_args(_program: &str) -> Vec<String> {
    Vec::new()
}

pub(super) fn resolve_terminal_launch(
    config: &TerminalServerConfig,
    shell_profiles: &[TerminalProfile],
    profile_id: Option<&str>,
    shell: &str,
) -> Result<LaunchTarget, SessionError> {
    let profiles = available_terminal_profiles(config, shell_profiles);

    if let Some(profile_id) = profile_id {
        if let Some(profile) = profiles
            .iter()
            .find(|profile| profile.id == profile_id)
            .or_else(|| find_profile_alias(&profiles, profile_id))
        {
            return Ok(launch_profile(config, profile)?);
        }
    }

    if !shell.trim().is_empty() {
        return Ok(LaunchTarget::Shell(LaunchProfile {
            title: shell.to_string(),
            program: shell.to_string(),
            args: login_shell_args(shell),
        }));
    }

    if let Some(profile) = profiles
        .iter()
        .find(|profile| profile.is_default)
        .or_else(|| profiles.first())
    {
        return launch_profile(config, profile);
    }

    let program = default_shell();
    Ok(LaunchTarget::Shell(LaunchProfile {
        args: login_shell_args(&program),
        title: program.clone(),
        program,
    }))
}

fn launch_profile(
    config: &TerminalServerConfig,
    profile: &TerminalProfile,
) -> Result<LaunchTarget, SessionError> {
    if let Some(uart_id) = profile.id.strip_prefix("uart:") {
        let port_config = config
            .uart
            .ports
            .iter()
            .find(|entry| entry.id == uart_id)
            .cloned()
            .ok_or_else(|| SessionError::PtyError("UART profile no longer exists".to_string()))?;
        return Ok(LaunchTarget::Uart(UartLaunchProfile {
            title: profile.title.clone(),
            port_config,
        }));
    }

    Ok(LaunchTarget::Shell(LaunchProfile {
        title: profile.title.clone(),
        program: profile.program.clone(),
        args: profile.args.clone(),
    }))
}

pub(super) fn find_profile_alias<'a>(
    profiles: &'a [TerminalProfile],
    profile_id: &str,
) -> Option<&'a TerminalProfile> {
    let alias = profile_id.trim().to_ascii_lowercase();
    if alias.is_empty() {
        return None;
    }

    match alias.as_str() {
        "cmd" | "command-prompt" => profiles.iter().find(|profile| {
            profile.id == "command-prompt" || profile.title.eq_ignore_ascii_case("Command Prompt")
        }),
        "powershell" | "windows-powershell" => profiles.iter().find(|profile| {
            profile.id == "windows-powershell"
                || profile.title.eq_ignore_ascii_case("Windows PowerShell")
        }),
        "pwsh" | "powershell-7" => profiles.iter().find(|profile| {
            profile.id == "powershell-7" || profile.title.eq_ignore_ascii_case("PowerShell")
        }),
        _ => None,
    }
}

pub(super) fn available_terminal_profiles(
    config: &TerminalServerConfig,
    shell_profiles: &[TerminalProfile],
) -> Vec<TerminalProfile> {
    let mut profiles = shell_profiles.to_vec();
    profiles.extend(available_uart_profiles(config));
    profiles
}

/// Discover profiles backed by locally installed shells and terminal providers.
///
/// This may read configuration files and invoke platform tools such as `vswhere`
/// and `wsl`, so the server calls it once during startup and caches the result.
pub(super) fn discover_shell_profiles() -> Vec<TerminalProfile> {
    available_shell_profiles()
}

/// Copies the local server identity onto a terminal profile before it is sent to clients.
pub(super) fn annotate_terminal_profile(
    mut profile: TerminalProfile,
    server_id: &str,
    server_name: Option<&str>,
) -> TerminalProfile {
    profile.server_id = Some(server_id.to_string());
    profile.server_name = server_name.map(|value| value.to_string());
    profile
}

/// Normalizes the configured local server name for outbound session/profile metadata.
pub(super) fn local_server_name(config: &TerminalServerConfig) -> Option<String> {
    let trimmed = config.server.name.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn available_shell_profiles() -> Vec<TerminalProfile> {
    #[cfg(target_os = "windows")]
    {
        super::session_win::available_terminal_profiles()
    }

    #[cfg(not(target_os = "windows"))]
    {
        let program = default_shell();
        vec![TerminalProfile {
            id: "default-shell".to_string(),
            title: "Default Shell".to_string(),
            args: login_shell_args(&program),
            program,
            is_default: true,
            server_id: None,
            server_name: None,
        }]
    }
}

fn available_uart_profiles(config: &TerminalServerConfig) -> Vec<TerminalProfile> {
    config
        .uart
        .ports
        .iter()
        .map(|port| TerminalProfile {
            id: format!("uart:{}", port.id),
            title: format!("UART: {} ({})", port.name, port.port),
            program: port.port.clone(),
            args: vec![port.baud_rate.to_string()],
            is_default: false,
            server_id: None,
            server_name: None,
        })
        .collect()
}
