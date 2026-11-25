use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum ShellType {
    PowerShell,
    PowerShellCore,
    Cmd,
    Bash,
    Zsh,
    Fish,
    Sh,
    Unknown(String),
}

impl ShellType {
    /// Get the executable name for this shell
    pub fn executable(&self) -> String {
        match self {
            ShellType::PowerShell => "powershell.exe".to_string(),
            ShellType::PowerShellCore => "pwsh.exe".to_string(),
            ShellType::Cmd => "cmd.exe".to_string(),
            ShellType::Bash => "/bin/bash".to_string(),
            ShellType::Zsh => "/bin/zsh".to_string(),
            ShellType::Fish => "/usr/bin/fish".to_string(),
            ShellType::Sh => "/bin/sh".to_string(),
            ShellType::Unknown(cmd) => cmd.clone(),
        }
    }

    /// Get the profile ID for vterm-server
    pub fn profile_id(&self) -> Option<String> {
        match self {
            ShellType::PowerShell | ShellType::PowerShellCore => Some("powershell".to_string()),
            ShellType::Cmd => Some("cmd".to_string()),
            ShellType::Bash => Some("bash".to_string()),
            ShellType::Zsh => Some("zsh".to_string()),
            ShellType::Fish => Some("fish".to_string()),
            _ => None,
        }
    }

    /// Get default arguments for this shell
    pub fn default_args(&self) -> Vec<String> {
        match self {
            ShellType::PowerShell | ShellType::PowerShellCore => {
                vec![
                    "-NoExit".to_string(),
                    "-Command".to_string(),
                    "-".to_string(),
                ]
            }
            ShellType::Cmd => vec!["/K".to_string()],
            _ => vec!["-l".to_string()],
        }
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::PowerShell => write!(f, "PowerShell"),
            ShellType::PowerShellCore => write!(f, "PowerShell Core"),
            ShellType::Cmd => write!(f, "Command Prompt"),
            ShellType::Bash => write!(f, "Bash"),
            ShellType::Zsh => write!(f, "Zsh"),
            ShellType::Fish => write!(f, "Fish"),
            ShellType::Sh => write!(f, "Sh"),
            ShellType::Unknown(name) => write!(f, "{}", name),
        }
    }
}

/// Detected environment from parent terminal
#[derive(Debug, Clone)]
pub struct Environment {
    /// Current working directory
    pub cwd: String,
    /// Detected shell type
    pub shell: ShellType,
    /// Inherited environment variables
    pub vars: HashMap<String, String>,
}

impl Environment {
    /// Detect environment from current process
    pub fn detect() -> anyhow::Result<Self> {
        let cwd = env::current_dir()?.to_string_lossy().to_string();

        let shell = detect_shell();
        let vars = HashMap::new();

        Ok(Self { cwd, shell, vars })
    }

    /// Inherit environment variables based on patterns
    pub fn inherit_vars(&mut self, patterns: &[String]) {
        self.vars.clear();

        for (key, value) in env::vars() {
            if should_inherit_var(&key, patterns) {
                self.vars.insert(key, value);
            }
        }
    }

    /// Add extra environment variables
    pub fn add_vars(&mut self, vars: &[(String, String)]) {
        for (key, value) in vars {
            self.vars.insert(key.clone(), value.clone());
        }
    }

    /// Get environment variables as a HashMap
    pub fn to_hash_map(&self) -> HashMap<String, String> {
        self.vars.clone()
    }

    /// Set working directory
    pub fn set_cwd(&mut self, cwd: String) {
        self.cwd = cwd;
    }
}

/// Detect shell type from environment
fn detect_shell() -> ShellType {
    // Check for Windows Terminal profile first
    if let Ok(_wt_profile) = env::var("WT_PROFILE_ID") {
        // Try to determine shell from WT_PROFILE_ID or other env vars
        if let Ok(_ps_version) = env::var("PSVersionTable") {
            return ShellType::PowerShellCore;
        }
        if let Ok(_ps_module_path) = env::var("PSMODULEPATH") {
            if env::var("PSCORE_EDITION").is_ok() {
                return ShellType::PowerShellCore;
            }
            return ShellType::PowerShell;
        }
    }

    // Check SHELL environment variable (Unix)
    #[cfg(unix)]
    if let Ok(shell_path) = env::var("SHELL") {
        return shell_from_path(&shell_path);
    }

    // Check COMSPEC (Windows)
    #[cfg(windows)]
    if let Some(parent_shell) = detect_parent_shell_windows() {
        return parent_shell;
    }

    // Check COMSPEC (Windows)
    #[cfg(windows)]
    if let Ok(comspec) = env::var("COMSPEC") {
        let lower = comspec.to_lowercase();
        if lower.contains("cmd") {
            return ShellType::Cmd;
        }
        if lower.contains("powershell") {
            return ShellType::PowerShell;
        }
    }

    #[cfg(unix)]
    if let Some(parent_shell) = detect_parent_shell_unix() {
        return parent_shell;
    }

    // Default fallbacks
    #[cfg(windows)]
    return ShellType::PowerShell;

    #[cfg(unix)]
    return ShellType::Sh;
}

fn shell_from_path(path: &str) -> ShellType {
    let lower = path.to_lowercase();
    if lower.contains("zsh") {
        ShellType::Zsh
    } else if lower.contains("bash") {
        ShellType::Bash
    } else if lower.contains("fish") {
        ShellType::Fish
    } else if lower.contains("pwsh") || lower.contains("powershell") {
        if lower.contains("pwsh") {
            ShellType::PowerShellCore
        } else {
            ShellType::PowerShell
        }
    } else if lower.contains("cmd") {
        ShellType::Cmd
    } else if lower.contains("sh") {
        ShellType::Sh
    } else {
        ShellType::Unknown(path.to_string())
    }
}

/// Check if a variable should be inherited based on patterns
fn should_inherit_var(name: &str, patterns: &[String]) -> bool {
    let upper_name = name.to_uppercase();

    for pattern in patterns {
        let pattern_upper = pattern.to_uppercase();

        // Exact match
        if upper_name == pattern_upper {
            return true;
        }

        // Wildcard patterns
        if pattern == "*" {
            return true;
        }

        // Prefix wildcard (e.g., "PATH*" matches "PATH", "PATHEXT")
        if pattern_upper.ends_with('*') {
            let prefix = &pattern_upper[..pattern_upper.len() - 1];
            if upper_name.starts_with(prefix) {
                return true;
            }
        }

        // Contains pattern (e.g., "*SSH*" matches "SSH_AGENT", "MY_SSH_KEY")
        if pattern_upper.starts_with('*') && pattern_upper.ends_with('*') {
            let middle = &pattern_upper[1..pattern_upper.len() - 1];
            if upper_name.contains(middle) {
                return true;
            }
        }

        // Suffix wildcard (e.g., "*PATH" matches "LD_LIBRARY_PATH")
        if pattern_upper.starts_with('*') {
            let suffix = &pattern_upper[1..];
            if upper_name.ends_with(suffix) {
                return true;
            }
        }
    }

    false
}

#[cfg(windows)]
fn detect_parent_shell_windows() -> Option<ShellType> {
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;

    #[derive(Clone)]
    struct ProcessInfo {
        parent_pid: u32,
        exe_name: String,
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return None;
    }

    let mut processes = HashMap::<u32, ProcessInfo>::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..unsafe { std::mem::zeroed() }
    };

    let first_ok = unsafe { Process32FirstW(snapshot, &mut entry) != 0 };
    if first_ok {
        loop {
            let name_end = entry
                .szExeFile
                .iter()
                .position(|&value| value == 0)
                .unwrap_or(entry.szExeFile.len());
            let exe_name = OsString::from_wide(&entry.szExeFile[..name_end])
                .to_string_lossy()
                .into_owned();
            processes.insert(
                entry.th32ProcessID,
                ProcessInfo {
                    parent_pid: entry.th32ParentProcessID,
                    exe_name,
                },
            );

            if unsafe { Process32NextW(snapshot, &mut entry) } == 0 {
                break;
            }
        }
    }

    unsafe {
        CloseHandle(snapshot);
    }

    let mut pid = unsafe { GetCurrentProcessId() };
    for _ in 0..8 {
        let process = processes.get(&pid)?;
        let parent = processes.get(&process.parent_pid)?;
        let shell = shell_from_path(&parent.exe_name);
        match shell {
            ShellType::Unknown(_) | ShellType::Sh => {
                pid = process.parent_pid;
            }
            other => return Some(other),
        }
    }

    None
}

#[cfg(unix)]
fn detect_parent_shell_unix() -> Option<ShellType> {
    use std::fs;
    use std::process::Command;

    // Try to get parent process name from /proc
    if let Ok(ppid) = get_parent_pid() {
        let exe_path = format!("/proc/{}/exe", ppid);
        if let Ok(target) = fs::read_link(&exe_path) {
            let path = target.to_string_lossy();
            return Some(shell_from_path(&path));
        }

        // Fallback: read cmdline
        let cmdline_path = format!("/proc/{}/cmdline", ppid);
        if let Ok(content) = fs::read_to_string(&cmdline_path) {
            let parts: Vec<&str> = content.split('\0').collect();
            if let Some(first) = parts.first() {
                return Some(shell_from_path(first));
            }
        }
    }

    None
}

#[cfg(unix)]
fn get_parent_pid() -> anyhow::Result<u32> {
    use std::fs;
    use std::io::{self, BufRead};

    let file = fs::File::open("/proc/self/status")?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("PPid:") {
            let ppid_str = line[5..].trim();
            return Ok(ppid_str.parse()?);
        }
    }

    Err(anyhow::anyhow!("PPid not found in /proc/self/status"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_inherit_var_exact_match() {
        let patterns = vec!["PATH".to_string()];
        assert!(should_inherit_var("PATH", &patterns));
        assert!(!should_inherit_var("PATH_EXT", &patterns));
    }

    #[test]
    fn test_should_inherit_var_wildcard() {
        let patterns = vec!["PATH*".to_string()];
        assert!(should_inherit_var("PATH", &patterns));
        assert!(should_inherit_var("PATHEXT", &patterns));
        assert!(!should_inherit_var("XPATH", &patterns));
    }

    #[test]
    fn test_should_inherit_var_all() {
        let patterns = vec!["*".to_string()];
        assert!(should_inherit_var("ANYTHING", &patterns));
    }

    #[test]
    fn test_shell_type_profile_id() {
        assert_eq!(
            ShellType::PowerShell.profile_id(),
            Some("powershell".to_string())
        );
        assert_eq!(ShellType::Cmd.profile_id(), Some("cmd".to_string()));
        assert_eq!(ShellType::Bash.profile_id(), Some("bash".to_string()));
    }
}
