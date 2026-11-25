use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use tracing::{info, trace, warn};
use vterm_common::SessionId;
use vterm_core::protocol::DebugLaunchConfig;

use crate::session::SessionManager;

#[derive(Debug, Default, Deserialize)]
struct LaunchFile {
    #[serde(default)]
    launch: Vec<DebugLaunchConfig>,
    #[serde(default)]
    configurations: Vec<DebugLaunchConfig>,
}

pub async fn load_launches(
    session_manager: &SessionManager,
    session_id: SessionId,
) -> Result<(PathBuf, Vec<DebugLaunchConfig>)> {
    let repo_root = repo_root_for_session(session_manager, session_id).await?;
    info!(
        "[debug] load_launches session={} repo_root={}",
        session_id,
        repo_root.display()
    );
    let launch_path = repo_root.join(".vterm").join("launch.json");
    info!(
        "[debug] reading launch file session={} path={}",
        session_id,
        launch_path.display()
    );
    let data = match tokio::fs::read_to_string(&launch_path).await {
        Ok(data) => data,
        Err(_) => return Ok((repo_root, Vec::new())),
    };
    info!(
        "[debug] launch file read session={} path={} bytes={}",
        session_id,
        launch_path.display(),
        data.len()
    );
    if let Ok(value) = serde_json::from_str::<Value>(&data) {
        info!(
            "[debug] launch file top-level keys session={} keys={:?}",
            session_id,
            value
                .as_object()
                .map(|object| object.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        );
    }
    let launch_file: LaunchFile = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse {}", launch_path.display()))?;

    let mut launches = launch_file.launch;
    launches.extend(launch_file.configurations);
    launches.retain(|launch| !launch.name.trim().is_empty());
    info!(
        "[debug] launch file parsed session={} launches={}",
        session_id,
        launches
            .iter()
            .map(super::launch_summary)
            .collect::<Vec<_>>()
            .join(" | ")
    );
    Ok((repo_root, launches))
}

pub(crate) async fn repo_root_for_session(
    session_manager: &SessionManager,
    session_id: SessionId,
) -> Result<PathBuf> {
    let workspace = session_manager
        .session_workspace_path(session_id)
        .await?
        .context("Session workspace is unknown")?;
    let workspace = PathBuf::from(workspace);
    Ok(match repo_root_for_workspace(&workspace).await {
        Some(repo_root) => {
            info!(
                "[debug] git repo root resolved session={} workspace={} repo_root={}",
                session_id,
                workspace.display(),
                repo_root.display()
            );
            repo_root
        }
        None => {
            warn!(
                "[debug] git repo root not found, using workspace session={} workspace={}",
                session_id,
                workspace.display()
            );
            workspace
        }
    })
}

/// Resolve a repository from the shell's current directory. This deliberately
/// ignores a pinned file-browser path: terminal quick commands must follow
/// `cd` immediately.
pub(crate) async fn repo_root_for_current_working_path(
    session_manager: &SessionManager,
    session_id: SessionId,
) -> Result<PathBuf> {
    let workspace = session_manager
        .session_current_working_path(session_id)
        .await?
        .context("Session current working path is unknown")?;
    let workspace = PathBuf::from(workspace);
    Ok(match repo_root_for_workspace(&workspace).await {
        Some(repo_root) => repo_root,
        None => workspace,
    })
}

async fn repo_root_for_workspace(workspace: &Path) -> Option<PathBuf> {
    let workspace = workspace.to_path_buf();
    tokio::task::spawn_blocking(move || {
        //         info!(
        //             "[debug] running git rev-parse --show-toplevel workspace={}",
        //             workspace.display()
        //         );
        let mut command = std::process::Command::new("git");
        command
            .arg("-C")
            .arg(&workspace)
            .arg("rev-parse")
            .arg("--show-toplevel");
        vterm_common::command_no_window(&mut command);
        let output = command
            .output()
            .map_err(|error| {
                warn!(
                    "[debug] git rev-parse spawn failed workspace={} error={}",
                    workspace.display(),
                    error
                );
                error
            })
            .ok()?;
        if !output.status.success() {
            // Expected for plain directories (not git workspaces); the git
            // features simply stay unavailable. Keep it below the default
            // level to avoid spamming every launch.
            trace!(
                "[debug] git rev-parse failed workspace={} status={} stderr={}",
                workspace.display(),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            );
            return None;
        }
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if root.is_empty() {
            None
        } else {
            Some(PathBuf::from(root))
        }
    })
    .await
    .ok()
    .flatten()
}
