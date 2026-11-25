use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;
use vterm_common::SessionId;
use vterm_core::protocol::QuickCommand;

use crate::session::SessionManager;

use super::launch::repo_root_for_current_working_path;

#[derive(Deserialize)]
struct CommandEntry {
    #[serde(flatten)]
    command: QuickCommand,
    #[serde(default)]
    os: CommandOs,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(from = "String")]
enum CommandOs {
    #[default]
    All,
    Win,
    Linux,
    Osx,
}

impl From<String> for CommandOs {
    fn from(os: String) -> Self {
        match os.as_str() {
            "win" => Self::Win,
            "linux" => Self::Linux,
            "osx" => Self::Osx,
            _ => Self::All,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CommandFile {
    List(Vec<CommandEntry>),
    Object {
        #[serde(default, alias = "quickCommands")]
        commands: Vec<CommandEntry>,
    },
}

pub async fn load_commands(
    session_manager: &SessionManager,
    session_id: SessionId,
) -> Result<(PathBuf, Vec<QuickCommand>)> {
    let (repo_root, path) = command_paths(session_manager, session_id).await?;
    let data = match tokio::fs::read_to_string(&path).await {
        Ok(data) => data,
        // Commands are an opt-in repository feature. A missing file simply
        // means this session has no Commands submenu.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((repo_root, Vec::new()));
        }
        Err(error) => {
            return Err(error).with_context(|| format!("Failed to read {}", path.display()))
        }
    };
    let commands = match serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse {}", path.display()))?
    {
        CommandFile::List(commands) => commands,
        CommandFile::Object { commands } => commands,
    };
    validate_commands(&commands).with_context(|| format!("Invalid {}", path.display()))?;
    let commands = filter_commands_for_os(commands, std::env::consts::OS);
    Ok((repo_root, commands))
}

pub(crate) async fn command_paths(
    session_manager: &SessionManager,
    session_id: SessionId,
) -> Result<(PathBuf, PathBuf)> {
    let repo_root = repo_root_for_current_working_path(session_manager, session_id).await?;
    let path = repo_root.join(".vterm").join("commands.json");
    Ok((repo_root, path))
}

fn filter_commands_for_os(commands: Vec<CommandEntry>, server_os: &str) -> Vec<QuickCommand> {
    commands
        .into_iter()
        .filter(|entry| match entry.os {
            CommandOs::All => true,
            CommandOs::Win => server_os == "windows",
            CommandOs::Linux => server_os == "linux",
            CommandOs::Osx => server_os == "macos",
        })
        .map(|entry| entry.command)
        .collect()
}

fn validate_commands(commands: &[CommandEntry]) -> Result<()> {
    let mut names = HashSet::new();
    for entry in commands {
        let command = &entry.command;
        if command.name.trim().is_empty() {
            anyhow::bail!("command name cannot be empty");
        }
        if command.command.trim().is_empty() {
            anyhow::bail!("command '{}' has an empty command", command.name);
        }
        if !names.insert(command.name.clone()) {
            anyhow::bail!("command name '{}' is duplicated", command.name);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commands_with_all_os_values() -> Vec<CommandEntry> {
        serde_json::from_str(
            r#"[
                {"name":"default","command":"default"},
                {"name":"all","command":"all","os":"all"},
                {"name":"unknown","command":"unknown","os":"windows"},
                {"name":"windows","command":"windows","os":"win"},
                {"name":"linux","command":"linux","os":"linux"},
                {"name":"osx","command":"osx","os":"osx"}
            ]"#,
        )
        .unwrap()
    }

    #[test]
    fn missing_os_defaults_to_all() {
        let entry: CommandEntry =
            serde_json::from_str(r#"{"name":"build","command":"cargo build"}"#).unwrap();
        assert_eq!(entry.os, CommandOs::All);
    }

    #[test]
    fn unknown_os_defaults_to_all() {
        let entry: CommandEntry =
            serde_json::from_str(r#"{"name":"build","command":"cargo build","os":"windows"}"#)
                .unwrap();
        assert_eq!(entry.os, CommandOs::All);
    }

    #[test]
    fn command_entry_preserves_existing_fields() {
        let entry: CommandEntry = serde_json::from_str(
            r#"{"name":"build","value":"cargo build","env":{"RUST_LOG":"debug"}}"#,
        )
        .unwrap();
        assert_eq!(entry.command.command, "cargo build");
        assert_eq!(entry.command.env.get("RUST_LOG").unwrap(), "debug");
    }

    #[test]
    fn filters_commands_by_server_os() {
        let cases = [
            ("windows", vec!["default", "all", "unknown", "windows"]),
            ("linux", vec!["default", "all", "unknown", "linux"]),
            ("macos", vec!["default", "all", "unknown", "osx"]),
            ("freebsd", vec!["default", "all", "unknown"]),
        ];

        for (server_os, expected) in cases {
            let filtered = filter_commands_for_os(commands_with_all_os_values(), server_os);
            let names: Vec<_> = filtered
                .iter()
                .map(|command| command.name.as_str())
                .collect();
            assert_eq!(names, expected, "server OS: {server_os}");
            assert!(serde_json::to_value(filtered).unwrap()[0]
                .get("os")
                .is_none());
        }
    }
}
