use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{info, warn};
use vterm_common::SessionId;
use vterm_core::protocol::{DebugLaunchConfig, Message};

use crate::relay::RelayManager;
use crate::session::SessionManager;

mod commands;
mod dap;
mod extensions;
mod generate;
mod launch;
mod vars;

pub(crate) use commands::command_paths;
use commands::load_commands;
pub use dap::send_to_adapter;
use dap::start_codelldb;
pub(crate) use generate::generate as generate_config;
use launch::load_launches;
use vars::{expand_launch_vars, expand_workspace_vars};

pub async fn command_list(session_manager: &SessionManager, session_id: SessionId) -> Message {
    match load_commands(session_manager, session_id).await {
        Ok((repo_root, commands)) => Message::QuickCommandListResult {
            session_id,
            repo_root: Some(repo_root.display().to_string()),
            commands,
            error: None,
        },
        Err(error) => Message::QuickCommandListResult {
            session_id,
            repo_root: None,
            commands: Vec::new(),
            error: Some(format!("{error:#}")),
        },
    }
}

pub async fn run_command(
    session_manager: &SessionManager,
    session_id: SessionId,
    name: String,
) -> Message {
    let result = async {
        let (repo_root, commands) = load_commands(session_manager, session_id).await?;
        let command = commands
            .into_iter()
            .find(|command| command.name == name)
            .context("Quick command not found")?;
        session_manager
            .run_quick_command(session_id, &repo_root, &command.command, &command.env)
            .await?;
        anyhow::Ok(())
    }
    .await;
    match result {
        Ok(()) => Message::RunQuickCommandResult {
            session_id,
            name,
            success: true,
            error: None,
        },
        Err(error) => Message::RunQuickCommandResult {
            session_id,
            name,
            success: false,
            error: Some(format!("{error:#}")),
        },
    }
}

pub async fn launch_list(session_manager: &SessionManager, session_id: SessionId) -> Message {
    info!("[debug] launch_list requested session={}", session_id);
    match load_launches(session_manager, session_id).await {
        Ok((repo_root, launches)) => {
            info!(
                "[debug] launch_list loaded session={} repo_root={} launches={:?}",
                session_id,
                repo_root.display(),
                launches
                    .iter()
                    .map(|launch| launch.name.as_str())
                    .collect::<Vec<_>>()
            );
            Message::DebugLaunchListResult {
                session_id,
                repo_root: Some(repo_root.display().to_string()),
                launches,
                error: None,
            }
        }
        Err(error) => {
            warn!(
                "[debug] launch_list failed session={} error={:#}",
                session_id, error
            );
            Message::DebugLaunchListResult {
                session_id,
                repo_root: None,
                launches: Vec::new(),
                error: Some(format!("{error:#}")),
            }
        }
    }
}

pub async fn start_launch(
    session_manager: Arc<SessionManager>,
    relay_manager: Arc<RelayManager>,
    session_id: SessionId,
    launch_name: String,
) -> Message {
    info!(
        "[debug] start requested session={} launch={}",
        session_id, launch_name
    );
    let (repo_root, launches) = match load_launches(&session_manager, session_id).await {
        Ok(value) => value,
        Err(error) => {
            warn!(
                "[debug] start failed before launch load session={} launch={} error={:#}",
                session_id, launch_name, error
            );
            return debug_start_error(session_id, launch_name, error);
        }
    };

    info!(
        "[debug] start loaded launch file session={} repo_root={} launch_count={} launch_names={:?}",
        session_id,
        repo_root.display(),
        launches.len(),
        launches.iter().map(|launch| launch.name.as_str()).collect::<Vec<_>>()
    );

    let Some(launch) = launches
        .into_iter()
        .find(|launch| launch.name == launch_name)
    else {
        warn!(
            "[debug] start launch not found session={} launch={}",
            session_id, launch_name
        );
        return Message::DebugStartResult {
            session_id,
            launch_name,
            success: false,
            error: Some("Launch configuration not found".to_string()),
        };
    };

    let has_dap_config = is_dap_config(&launch);
    let has_executable = launch_command(&launch).is_some();
    if !has_executable
        && launch.pre_tasks.is_empty()
        && launch.pre_launch_task.is_none()
        && !has_dap_config
    {
        warn!(
            "[debug] start rejected empty launch session={} launch={} config={}",
            session_id,
            launch_name,
            launch_summary(&launch)
        );
        return Message::DebugStartResult {
            session_id,
            launch_name,
            success: false,
            error: Some(
                "Launch command is empty and no DAP/preTask fields were provided".to_string(),
            ),
        };
    }

    info!(
        "[debug] start accepted session={} repo_root={} config={}",
        session_id,
        repo_root.display(),
        launch_summary(&launch)
    );
    let launch_for_task = launch.clone();
    let launch_name_for_task = launch_name.clone();
    tokio::spawn(async move {
        run_launch_task(session_id, repo_root, launch_for_task, relay_manager).await;
    });

    Message::DebugStartResult {
        session_id,
        launch_name: launch_name_for_task,
        success: true,
        error: None,
    }
}

fn debug_start_error(session_id: SessionId, launch_name: String, error: anyhow::Error) -> Message {
    Message::DebugStartResult {
        session_id,
        launch_name,
        success: false,
        error: Some(format!("{error:#}")),
    }
}

pub(crate) fn launch_summary(launch: &DebugLaunchConfig) -> String {
    format!(
        "name='{}' type='{}' request='{}' command='{}' program='{}' runtimeExecutable='{}' args={:?} runtimeArgs={:?} cwd={:?} preLaunchTask={:?} preTasks={:?} adapter={:?} adapterArgs={:?} debugServer={:?} debugServerHost={:?} env_count={}",
        launch.name,
        launch.adapter_type,
        launch.request,
        launch.command,
        launch.program,
        launch.runtime_executable,
        launch.args,
        launch.runtime_args,
        launch.cwd,
        launch.pre_launch_task,
        launch.pre_tasks,
        launch.adapter,
        launch.adapter_args,
        launch.debug_server,
        launch.debug_server_host,
        launch.env.len()
    )
}

fn is_dap_config(launch: &DebugLaunchConfig) -> bool {
    !launch.adapter_type.trim().is_empty()
        || !launch.request.trim().is_empty()
        || launch.adapter.is_some()
        || launch.debug_server.is_some()
}

fn launch_command(launch: &DebugLaunchConfig) -> Option<(String, Vec<String>)> {
    let command = launch.command.trim();
    if !command.is_empty() {
        return Some((launch.command.clone(), launch.args.clone()));
    }

    let runtime_executable = launch.runtime_executable.trim();
    if !runtime_executable.is_empty() {
        let mut args = launch.runtime_args.clone();
        if !launch.program.trim().is_empty() {
            args.push(launch.program.clone());
        }
        args.extend(launch.args.clone());
        return Some((launch.runtime_executable.clone(), args));
    }

    if !launch.program.trim().is_empty() {
        return Some((launch.program.clone(), launch.args.clone()));
    }

    None
}

async fn run_launch_task(
    session_id: SessionId,
    repo_root: PathBuf,
    mut launch: DebugLaunchConfig,
    relay_manager: Arc<RelayManager>,
) {
    expand_launch_vars(&mut launch, &repo_root);

    info!(
        "[debug] task begin session={} repo_root={} config={}",
        session_id,
        repo_root.display(),
        launch_summary(&launch)
    );

    let cwd = launch
        .cwd
        .as_deref()
        .map(|cwd| resolve_cwd(&repo_root, cwd))
        .unwrap_or_else(|| repo_root.clone());
    info!(
        "[debug] task cwd resolved session={} launch={} cwd={}",
        session_id,
        launch.name,
        cwd.display()
    );

    let mut pre_tasks = Vec::new();
    if let Some(pre_launch_task) = launch.pre_launch_task.as_deref() {
        pre_tasks.push(pre_launch_task);
    }
    pre_tasks.extend(launch.pre_tasks.iter().map(String::as_str));

    for (index, pre_task) in pre_tasks.iter().enumerate() {
        info!(
            "[debug] preTask start session={} launch={} index={} command={}",
            session_id, launch.name, index, pre_task
        );
        if let Err(error) = run_shell_command(
            session_id,
            &launch.name,
            "preTask",
            pre_task,
            &cwd,
            &launch.env,
            Arc::clone(&relay_manager),
        )
        .await
        {
            warn!(
                "[debug] preTask failed session={} launch={} index={} error={:#}",
                session_id, launch.name, index, error
            );
            send_output(
                Arc::clone(&relay_manager),
                session_id,
                Some(launch.name.clone()),
                "stderr",
                &format!("preTask failed: {error}\n"),
            )
            .await;
            send_start_result(
                Arc::clone(&relay_manager),
                session_id,
                launch.name.clone(),
                false,
                Some(error.to_string()),
            )
            .await;
            return;
        }
        info!(
            "[debug] preTask finished session={} launch={} index={}",
            session_id, launch.name, index
        );
    }

    if is_dap_config(&launch) {
        if launch.adapter_type.trim().eq_ignore_ascii_case("lldb")
            || launch.adapter.as_deref().is_some_and(|adapter| {
                adapter.to_ascii_lowercase().contains("codelldb")
                    || adapter.to_ascii_lowercase().contains("lldb")
            })
        {
            info!(
                "[debug] DAP codelldb start session={} launch={}",
                session_id, launch.name
            );
            if let Err(error) = start_codelldb(
                session_id,
                repo_root,
                launch.clone(),
                cwd,
                Arc::clone(&relay_manager),
            )
            .await
            {
                warn!(
                    "[debug] DAP codelldb failed session={} launch={} error={:#}",
                    session_id, launch.name, error
                );
                send_output(
                    Arc::clone(&relay_manager),
                    session_id,
                    Some(launch.name.clone()),
                    "stderr",
                    &format!("DAP failed: {error:#}\n"),
                )
                .await;
                send_start_result(
                    relay_manager,
                    session_id,
                    launch.name,
                    false,
                    Some(error.to_string()),
                )
                .await;
            }
            return;
        }

        let message = format!(
            "Unsupported DAP adapter type '{}': {}\n",
            launch.adapter_type,
            launch_summary(&launch)
        );
        warn!(
            "[debug] unsupported DAP adapter session={} launch={} config={}",
            session_id,
            launch.name,
            launch_summary(&launch)
        );
        send_output(
            Arc::clone(&relay_manager),
            session_id,
            Some(launch.name.clone()),
            "stderr",
            &message,
        )
        .await;
        send_start_result(relay_manager, session_id, launch.name, false, Some(message)).await;
        return;
    }

    if launch_command(&launch).is_none() {
        let message = if is_dap_config(&launch) {
            format!(
                "DAP launch config detected but adapter forwarding is not implemented yet: {}\n",
                launch_summary(&launch)
            )
        } else {
            format!(
                "No launch command configured: {}\n",
                launch_summary(&launch)
            )
        };
        warn!(
            "[debug] no executable after preTasks session={} launch={} dap={} config={}",
            session_id,
            launch.name,
            is_dap_config(&launch),
            launch_summary(&launch)
        );
        send_output(
            Arc::clone(&relay_manager),
            session_id,
            Some(launch.name.clone()),
            "stderr",
            &message,
        )
        .await;
        send_start_result(relay_manager, session_id, launch.name, false, Some(message)).await;
        return;
    }

    info!(
        "[debug] launch command start session={} launch={}",
        session_id, launch.name
    );
    if let Err(error) = run_program(session_id, &launch, &cwd, Arc::clone(&relay_manager)).await {
        warn!(
            "[debug] launch command failed session={} launch={} error={:#}",
            session_id, launch.name, error
        );
        send_output(
            Arc::clone(&relay_manager),
            session_id,
            Some(launch.name.clone()),
            "stderr",
            &format!("launch failed: {error}\n"),
        )
        .await;
        send_start_result(
            relay_manager,
            session_id,
            launch.name,
            false,
            Some(error.to_string()),
        )
        .await;
    } else {
        info!(
            "[debug] launch command finished session={} launch={}",
            session_id, launch.name
        );
        send_start_result(relay_manager, session_id, launch.name, false, None).await;
    }
}

fn resolve_cwd(repo_root: &Path, cwd: &str) -> PathBuf {
    let path = PathBuf::from(expand_workspace_vars(cwd, repo_root));
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

async fn run_shell_command(
    session_id: SessionId,
    launch_name: &str,
    label: &str,
    command: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    relay_manager: Arc<RelayManager>,
) -> Result<()> {
    info!(
        "[debug] shell command spawn session={} launch={} label={} cwd={} command={} env_count={}",
        session_id,
        launch_name,
        label,
        cwd.display(),
        command,
        env.len()
    );
    send_output(
        Arc::clone(&relay_manager),
        session_id,
        Some(launch_name.to_string()),
        "stdout",
        &format!("$ {command}\n"),
    )
    .await;
    let mut cmd = shell_command(command);
    cmd.current_dir(cwd)
        .envs(env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    run_child(
        session_id,
        Some(launch_name.to_string()),
        label,
        cmd,
        relay_manager,
    )
    .await
}

async fn run_program(
    session_id: SessionId,
    launch: &DebugLaunchConfig,
    cwd: &Path,
    relay_manager: Arc<RelayManager>,
) -> Result<()> {
    let (program, args) =
        launch_command(launch).expect("launch_command checked before run_program");
    if is_dap_config(launch)
        && launch.command.trim().is_empty()
        && launch.runtime_executable.trim().is_empty()
        && !launch.program.trim().is_empty()
    {
        warn!(
            "[debug] DAP adapter forwarding not implemented; running program directly as fallback session={} launch={} type={} request={}",
            session_id,
            launch.name,
            launch.adapter_type,
            launch.request
        );
    }
    info!(
        "[debug] program spawn session={} launch={} cwd={} program={} args={:?} env_count={}",
        session_id,
        launch.name,
        cwd.display(),
        program,
        args,
        launch.env.len()
    );
    send_output(
        Arc::clone(&relay_manager),
        session_id,
        Some(launch.name.clone()),
        "stdout",
        &format!("$ {} {}\n", program, args.join(" ")),
    )
    .await;
    let mut cmd = Command::new(&program);
    cmd.args(&args)
        .current_dir(cwd)
        .envs(&launch.env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Same CREATE_NO_WINDOW rationale as vterm_common::command_no_window,
    // inline because this is a tokio Command.
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x0800_0000);
    run_child(
        session_id,
        Some(launch.name.clone()),
        "launch",
        cmd,
        relay_manager,
    )
    .await
}

fn shell_command(command: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(command);
        // Same CREATE_NO_WINDOW rationale as vterm_common::command_no_window,
        // inline because this is a tokio Command.
        cmd.creation_flags(0x0800_0000);
        cmd
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new("sh");
        cmd.arg("-lc").arg(command);
        cmd
    }
}

async fn run_child(
    session_id: SessionId,
    launch_name: Option<String>,
    label: &str,
    mut cmd: Command,
    relay_manager: Arc<RelayManager>,
) -> Result<()> {
    info!(
        "[debug] child spawn begin session={} launch={} label={}",
        session_id,
        launch_name.as_deref().unwrap_or("<none>"),
        label
    );
    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn {label}"))?;
    let child_id = child.id();
    info!(
        "[debug] child spawned session={} launch={} label={} pid={:?}",
        session_id,
        launch_name.as_deref().unwrap_or("<none>"),
        label,
        child_id
    );

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_relay = Arc::clone(&relay_manager);
    let stderr_relay = Arc::clone(&relay_manager);
    let stdout_name = launch_name.clone();
    let stderr_name = launch_name.clone();

    let stdout_task = tokio::spawn(async move {
        if let Some(stdout) = stdout {
            stream_lines(session_id, stdout_name, "stdout", stdout, stdout_relay).await;
        }
    });
    let stderr_task = tokio::spawn(async move {
        if let Some(stderr) = stderr {
            stream_lines(session_id, stderr_name, "stderr", stderr, stderr_relay).await;
        }
    });

    let status = child
        .wait()
        .await
        .with_context(|| format!("Failed to wait for {label}"))?;
    let _ = stdout_task.await;
    let _ = stderr_task.await;

    if status.success() {
        info!(
            "[debug] child exited success session={} launch={} label={} status={}",
            session_id,
            launch_name.as_deref().unwrap_or("<none>"),
            label,
            status
        );
        send_output(
            relay_manager,
            session_id,
            launch_name,
            "stdout",
            &format!("{label} exited with {status}\n"),
        )
        .await;
        Ok(())
    } else {
        warn!(
            "[debug] child exited failure session={} launch={} label={} status={}",
            session_id,
            launch_name.as_deref().unwrap_or("<none>"),
            label,
            status
        );
        anyhow::bail!("{label} exited with {status}");
    }
}

async fn stream_lines<R>(
    session_id: SessionId,
    launch_name: Option<String>,
    stream: &'static str,
    reader: R,
    relay_manager: Arc<RelayManager>,
) where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        send_output(
            Arc::clone(&relay_manager),
            session_id,
            launch_name.clone(),
            stream,
            &(line + "\n"),
        )
        .await;
    }
}

pub(crate) async fn send_output(
    relay_manager: Arc<RelayManager>,
    session_id: SessionId,
    launch_name: Option<String>,
    stream: &str,
    data: &str,
) {
    relay_manager
        .broadcast_client_message(Message::DebugOutput {
            session_id,
            launch_name,
            stream: stream.to_string(),
            data: data.to_string(),
        })
        .await;
}

async fn send_start_result(
    relay_manager: Arc<RelayManager>,
    session_id: SessionId,
    launch_name: String,
    success: bool,
    error: Option<String>,
) {
    relay_manager
        .broadcast_client_message(Message::DebugStartResult {
            session_id,
            launch_name,
            success,
            error,
        })
        .await;
}
