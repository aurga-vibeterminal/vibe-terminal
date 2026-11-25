use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use tracing::{debug, info, warn};
use vterm_common::TerminalCommandHistory;

const MAX_RECENT_COMMANDS: usize = 100;
const MAX_HISTORY_FILE_BYTES: u64 = 4 * 1024 * 1024;

pub(super) fn scan_recent_terminal_commands() -> Vec<TerminalCommandHistory> {
    let mut commands = Vec::new();
    let mut seen = HashSet::new();

    for path in terminal_history_paths() {
        collect_history_file(&path, &mut commands, &mut seen);
        if commands.len() >= MAX_RECENT_COMMANDS {
            break;
        }
    }

    // info!(
    //     "Terminal command history scan found {} command(s)",
    //     commands.len()
    // );
    // for command in &commands {
    //     info!("Terminal command history: {}", command.command);
    // }

    commands
}

fn collect_history_file(
    path: &Path,
    commands: &mut Vec<TerminalCommandHistory>,
    seen: &mut HashSet<String>,
) {
    if commands.len() >= MAX_RECENT_COMMANDS {
        return;
    }

    let Ok(metadata) = fs::metadata(path) else {
        debug!("Terminal command history not available at {:?}", path);
        return;
    };
    if !metadata.is_file() || metadata.len() > MAX_HISTORY_FILE_BYTES {
        return;
    }

    let Ok(contents) = fs::read_to_string(path) else {
        debug!("Failed to read terminal command history at {:?}", path);
        return;
    };

    let parsed = parse_history_file(path, &contents);
    info!(
        "Terminal command history file {:?} produced {} command(s)",
        path,
        parsed.len()
    );

    for command in parsed.into_iter().rev() {
        let command = command.trim();
        if command.is_empty() || !seen.insert(command.to_string()) {
            continue;
        }
        commands.push(TerminalCommandHistory {
            command: command.to_string(),
        });
        if commands.len() >= MAX_RECENT_COMMANDS {
            break;
        }
    }
}

fn parse_history_file(path: &Path, contents: &str) -> Vec<String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if file_name.eq_ignore_ascii_case("fish_history") {
        return parse_fish_history(contents);
    }
    if file_name.eq_ignore_ascii_case(".zsh_history") {
        return parse_zsh_history(contents);
    }
    contents.lines().map(str::to_string).collect()
}

fn parse_zsh_history(contents: &str) -> Vec<String> {
    contents
        .lines()
        .map(|line| {
            line.strip_prefix(": ")
                .and_then(|rest| rest.split_once(';').map(|(_, command)| command))
                .unwrap_or(line)
                .to_string()
        })
        .collect()
}

fn parse_fish_history(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("- cmd: "))
        .map(|command| command.replace(r"\n", "\n"))
        .collect()
}

fn terminal_history_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(appdata) = std::env::var_os("APPDATA").map(PathBuf::from) {
        paths.push(
            appdata
                .join("Microsoft")
                .join("Windows")
                .join("PowerShell")
                .join("PSReadLine")
                .join("ConsoleHost_history.txt"),
        );
        paths.push(
            appdata
                .join("Microsoft")
                .join("PowerShell")
                .join("PSReadLine")
                .join("ConsoleHost_history.txt"),
        );
    }

    if let Some(home) = user_home_dir() {
        paths.push(home.join(".bash_history"));
        paths.push(home.join(".zsh_history"));
        paths.push(
            home.join(".local")
                .join("share")
                .join("fish")
                .join("fish_history"),
        );
    }

    paths
}

fn user_home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

pub(super) fn spawn_recent_command_scan(
    store: std::sync::Arc<std::sync::RwLock<Vec<TerminalCommandHistory>>>,
) {
    if let Err(error) = std::thread::Builder::new()
        .name("recent-command-scan".to_string())
        .spawn(move || {
            let commands = scan_recent_terminal_commands();
            match store.write() {
                Ok(mut guard) => *guard = commands,
                Err(error) => warn!("Failed to store terminal command histories: {}", error),
            }
        })
    {
        warn!(
            "Failed to spawn terminal command history scan thread: {}",
            error
        );
    }
}
