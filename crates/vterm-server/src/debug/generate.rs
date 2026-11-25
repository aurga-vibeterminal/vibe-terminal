use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

/// Create a conservative starter file for a selected `.vterm` directory or
/// for the Git repository containing a selected directory. Existing,
/// non-empty configurations are never replaced.
pub async fn generate(vterm_path: &str, config_type: &str) -> Result<()> {
    let requested_path = PathBuf::from(vterm_path);
    if !requested_path.is_absolute() {
        bail!(".vterm path must be absolute: {vterm_path}");
    }

    let file_name = match config_type {
        "commands" => "commands.json",
        "launch" => "launch.json",
        _ => bail!("unsupported .vterm configuration type: {config_type}"),
    };
    let (vterm_dir, scan_root) = resolve_vterm_dir(&requested_path).await?;
    ensure_vterm_dir(&vterm_dir).await?;

    let destination = vterm_dir.join(file_name);
    if has_configuration(&destination, config_type).await? {
        return Ok(());
    }

    let project = detect_project(&scan_root).await;
    match config_type {
        "commands" => {
            let content = command_template(project);
            tokio::fs::write(&destination, content.into_bytes())
                .await
                .with_context(|| format!("Failed to write {}", destination.display()))?;
        }
        "launch" => {
            let content = launch_template(project);
            tokio::fs::write(&destination, serde_json::to_vec_pretty(&content)?)
                .await
                .with_context(|| format!("Failed to write {}", destination.display()))?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

async fn resolve_vterm_dir(requested_path: &Path) -> Result<(PathBuf, PathBuf)> {
    if requested_path.file_name().and_then(|name| name.to_str()) == Some(".vterm") {
        let parent = requested_path
            .parent()
            .context(".vterm path must have a parent directory")?;
        let scan_root = git_root(parent)
            .await
            .unwrap_or_else(|| parent.to_path_buf());
        return Ok((requested_path.to_path_buf(), scan_root));
    }

    let metadata = tokio::fs::metadata(requested_path)
        .await
        .with_context(|| format!("Failed to inspect {}", requested_path.display()))?;
    if !metadata.is_dir() {
        bail!("{} is not a directory", requested_path.display());
    }

    let repo_root = git_root(requested_path).await.with_context(|| {
        format!(
            "{} is neither a .vterm directory nor inside a Git repository",
            requested_path.display()
        )
    })?;
    Ok((repo_root.join(".vterm"), repo_root))
}

async fn ensure_vterm_dir(vterm_dir: &Path) -> Result<()> {
    match tokio::fs::metadata(vterm_dir).await {
        Ok(metadata) if !metadata.is_dir() => {
            bail!("{} is not a directory", vterm_dir.display())
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            tokio::fs::create_dir_all(vterm_dir)
                .await
                .with_context(|| format!("Failed to create {}", vterm_dir.display()))
        }
        Err(error) => {
            Err(error).with_context(|| format!("Failed to inspect {}", vterm_dir.display()))
        }
    }
}

async fn has_configuration(path: &Path, config_type: &str) -> Result<bool> {
    let data = match tokio::fs::read_to_string(path).await {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(error).with_context(|| format!("Failed to read {}", path.display()))
        }
    };
    let value: Value = match serde_json::from_str(&data) {
        Ok(value) => value,
        Err(_) => return Ok(false),
    };
    let keys: &[&str] = match config_type {
        "commands" => &["commands", "quickCommands"],
        "launch" => &["launch", "configurations"],
        _ => return Ok(false),
    };
    Ok(keys.iter().any(|key| {
        value
            .get(key)
            .and_then(Value::as_array)
            .is_some_and(|items| !items.is_empty())
    }))
}

#[derive(Clone, Copy)]
enum Project {
    Android,
    Xcode,
    Dotnet,
    Npm,
    Cargo,
    Generic,
}

async fn detect_project(root: &Path) -> Project {
    let has = |name: &str| root.join(name).exists();
    if has("gradlew") || has("gradlew.bat") || has("settings.gradle") || has("settings.gradle.kts")
    {
        Project::Android
    } else if has("package.json") {
        Project::Npm
    } else if has("Cargo.toml") {
        Project::Cargo
    } else if has("global.json") || has_extension(root, "sln", "csproj").await {
        Project::Dotnet
    } else if has_extension(root, "xcodeproj", "xcworkspace").await {
        Project::Xcode
    } else {
        Project::Generic
    }
}

async fn has_extension(root: &Path, first: &str, second: &str) -> bool {
    let Ok(mut entries) = tokio::fs::read_dir(root).await else {
        return false;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case(first) || ext.eq_ignore_ascii_case(second))
        {
            return true;
        }
    }
    false
}

async fn git_root(path: &Path) -> Option<PathBuf> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut command = std::process::Command::new("git");
        command
            .arg("-C")
            .arg(&path)
            .arg("rev-parse")
            .arg("--show-toplevel");
        vterm_common::command_no_window(&mut command);
        let output = command.output().ok()?;
        if !output.status.success() {
            return None;
        }
        let root = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        (!root.is_empty()).then(|| PathBuf::from(root))
    })
    .await
    .ok()
    .flatten()
}

/// Build a hand-formatted `commands.json` string so the field order is exactly
/// `comment` first, then each entry as `name`, `command`, `env` — something
/// `serde_json`'s default (alphabetical) key ordering cannot guarantee.
fn command_template(project: Project) -> String {
    let commands = match project {
        Project::Android => vec![
            ("Build debug", "./gradlew assembleDebug"),
            ("Run tests", "./gradlew test"),
        ],
        Project::Xcode => vec![
            ("Build", "xcodebuild build"),
            ("Run tests", "xcodebuild test"),
        ],
        Project::Dotnet => vec![("Build", "dotnet build"), ("Run tests", "dotnet test")],
        Project::Npm => vec![
            ("Install dependencies", "npm install"),
            ("Build", "npm run build"),
            ("Run tests", "npm test"),
        ],
        Project::Cargo => vec![
            ("Build", "cargo build"),
            ("Run tests", "cargo test"),
            ("Run", "cargo run"),
        ],
        Project::Generic => vec![("Git pull", "git pull"), ("List files", "ls")],
    };
    let entries: Vec<String> = commands
        .into_iter()
        .map(|(name, command)| {
            format!(
                "    {{\n      \"name\": {},\n      \"command\": {},\n      \"env\": {{}}\n    }}",
                serde_json::to_string(name).unwrap(),
                serde_json::to_string(command).unwrap(),
            )
        })
        .collect();
    format!(
        "{{\n  \"comment\": {},\n  \"commands\": [\n{}\n  ]\n}}\n",
        serde_json::to_string(
            "Ask a code agent (e.g. Claude Code or Codex) to refine these commands for your project."
        )
        .unwrap(),
        entries.join(",\n")
    )
}

fn launch_template(project: Project) -> Value {
    let (name, command) = match project {
        Project::Android => ("Android debug build", "./gradlew installDebug"),
        Project::Xcode => ("Xcode build", "xcodebuild build"),
        Project::Dotnet => (".NET run", "dotnet run"),
        Project::Npm => ("npm start", "npm start"),
        Project::Cargo => ("Cargo run", "cargo run"),
        Project::Generic => (
            "Project launch",
            "echo 'Configure this launch command in .vterm/launch.json'",
        ),
    };
    json!({ "launch": [{ "name": name, "command": command }] })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn creates_missing_vterm_directory_for_direct_path() {
        let temp = tempfile::tempdir().unwrap();
        let vterm_dir = temp.path().join(".vterm");

        generate(vterm_dir.to_str().unwrap(), "launch")
            .await
            .unwrap();

        assert!(vterm_dir.is_dir());
        assert!(vterm_dir.join("launch.json").is_file());
    }

    #[tokio::test]
    async fn resolves_repo_root_from_regular_directory() {
        let temp = tempfile::tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        let nested = repo_root.join("src").join("nested");
        std::fs::create_dir_all(&nested).unwrap();
        let status = std::process::Command::new("git")
            .arg("init")
            .arg(&repo_root)
            .status()
            .unwrap();
        assert!(status.success());

        generate(nested.to_str().unwrap(), "commands")
            .await
            .unwrap();

        assert!(repo_root.join(".vterm").join("commands.json").is_file());
        assert!(!nested.join(".vterm").exists());
    }

    #[test]
    fn command_template_field_order_and_valid_json() {
        let text = command_template(Project::Cargo);
        // Valid JSON.
        let value: Value = serde_json::from_str(&text).unwrap();
        // comment is first (before "commands" in the raw text).
        let comment_pos = text.find("\"comment\"").unwrap();
        let commands_pos = text.find("\"commands\"").unwrap();
        assert!(comment_pos < commands_pos);
        // Each entry: name before command before env.
        let first_entry = text.find("{\n      \"name\"").unwrap();
        let cmd_pos = text[first_entry..].find("\"command\"").unwrap() + first_entry;
        let env_pos = text[first_entry..].find("\"env\"").unwrap() + first_entry;
        assert!(first_entry < cmd_pos);
        assert!(cmd_pos < env_pos);
        // comment is in English.
        let comment = value.get("comment").unwrap().as_str().unwrap();
        assert!(
            !comment.is_empty() && comment.is_ascii(),
            "comment should be English ASCII"
        );
    }
}
