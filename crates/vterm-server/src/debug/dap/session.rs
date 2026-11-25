use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, LazyLock, Mutex};

use anyhow::{Context, Result};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{info, warn};
use vterm_common::SessionId;
use vterm_core::protocol::{DebugLaunchConfig, Message};

use crate::relay::RelayManager;

use super::codec::{read_message, write_message};
use crate::debug::extensions::codelldb_adapter_path;

static DAP_SESSIONS: LazyLock<Mutex<HashMap<SessionId, mpsc::UnboundedSender<Value>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn send_to_adapter(session_id: SessionId, message: Value) -> Result<()> {
    let sender = {
        let sessions = DAP_SESSIONS.lock().expect("DAP session registry poisoned");
        sessions.get(&session_id).cloned()
    }
    .with_context(|| format!("No active DAP session for {session_id}"))?;
    sender
        .send(message)
        .map_err(|_| anyhow::anyhow!("DAP session for {session_id} is closed"))
}

pub async fn start_codelldb(
    session_id: SessionId,
    repo_root: PathBuf,
    launch: DebugLaunchConfig,
    cwd: PathBuf,
    relay_manager: Arc<RelayManager>,
) -> Result<()> {
    let adapter = codelldb_adapter_path()?;
    info!(
        "[debug][dap] starting codelldb session={} launch={} adapter={} cwd={}",
        session_id,
        launch.name,
        adapter.display(),
        cwd.display()
    );

    let mut cmd = Command::new(&adapter);
    cmd.current_dir(&repo_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Same CREATE_NO_WINDOW rationale as vterm_common::command_no_window,
    // inline because this is a tokio Command.
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x0800_0000);
    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn CodeLLDB adapter {}", adapter.display()))?;

    let mut stdin = child.stdin.take().context("CodeLLDB stdin unavailable")?;
    let stdout = child.stdout.take().context("CodeLLDB stdout unavailable")?;
    let stderr = child.stderr.take();
    let launch_name = launch.name.clone();
    let (tx, mut rx) = mpsc::unbounded_channel::<Value>();
    {
        let mut sessions = DAP_SESSIONS.lock().expect("DAP session registry poisoned");
        sessions.insert(session_id, tx.clone());
    }

    let writer_session = session_id;
    let writer_launch = launch_name.clone();
    let writer_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            info!(
                "[debug][dap] -> adapter session={} launch={} message={}",
                writer_session, writer_launch, message
            );
            if let Err(error) = write_message(&mut stdin, &message).await {
                warn!(
                    "[debug][dap] write failed session={} launch={} error={:#}",
                    writer_session, writer_launch, error
                );
                break;
            }
        }
    });

    if let Some(stderr) = stderr {
        let stderr_relay = Arc::clone(&relay_manager);
        let stderr_launch = launch_name.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                super::super::send_output(
                    Arc::clone(&stderr_relay),
                    session_id,
                    Some(stderr_launch.clone()),
                    "stderr",
                    &(line + "\n"),
                )
                .await;
            }
        });
    }

    send_initialize(session_id, &tx)?;

    let mut reader = BufReader::new(stdout);
    let mut launch_sent = false;
    let mut configuration_done_sent = false;
    loop {
        let message = match read_message(&mut reader).await {
            Ok(Some(message)) => message,
            Ok(None) => {
                warn!(
                    "[debug][dap] adapter stdout closed session={} launch={}",
                    session_id, launch_name
                );
                break;
            }
            Err(error) => {
                warn!(
                    "[debug][dap] read failed session={} launch={} error={:#}",
                    session_id, launch_name, error
                );
                break;
            }
        };
        info!(
            "[debug][dap] <- adapter session={} launch={} message={}",
            session_id, launch_name, message
        );
        if is_response_to(&message, "initialize") && !launch_sent {
            send_launch(session_id, &launch, &cwd, &tx)?;
            launch_sent = true;
        }
        if is_event(&message, "initialized") && !configuration_done_sent {
            send_configuration_done(session_id, &tx)?;
            configuration_done_sent = true;
        }
        relay_dap_message(
            Arc::clone(&relay_manager),
            session_id,
            Some(launch_name.clone()),
            message,
        )
        .await;
    }

    {
        let mut sessions = DAP_SESSIONS.lock().expect("DAP session registry poisoned");
        sessions.remove(&session_id);
    }
    drop(tx);
    let _ = writer_task.await;
    let status = child.wait().await.context("Failed to wait for CodeLLDB")?;
    info!(
        "[debug][dap] codelldb exited session={} launch={} status={}",
        session_id, launch_name, status
    );
    Ok(())
}

fn send_initialize(session_id: SessionId, tx: &mpsc::UnboundedSender<Value>) -> Result<()> {
    let initialize = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {
            "clientID": "vibe-terminal",
            "clientName": "Vibe Terminal",
            "adapterID": "lldb",
            "pathFormat": "path",
            "linesStartAt1": true,
            "columnsStartAt1": true,
            "supportsVariableType": true,
            "supportsVariablePaging": true,
            "supportsRunInTerminalRequest": false
        }
    });
    tx.send(initialize)
        .map_err(|_| anyhow::anyhow!("Failed to queue initialize DAP request for {session_id}"))
}

fn send_launch(
    session_id: SessionId,
    launch: &DebugLaunchConfig,
    cwd: &Path,
    tx: &mpsc::UnboundedSender<Value>,
) -> Result<()> {
    let launch_request = json!({
        "seq": 2,
        "type": "request",
        "command": if launch.request.trim().eq_ignore_ascii_case("attach") { "attach" } else { "launch" },
        "arguments": dap_launch_arguments(launch, cwd)
    });
    tx.send(launch_request)
        .map_err(|_| anyhow::anyhow!("Failed to queue launch DAP request for {session_id}"))
}

fn send_configuration_done(session_id: SessionId, tx: &mpsc::UnboundedSender<Value>) -> Result<()> {
    let configuration_done = json!({
        "seq": 3,
        "type": "request",
        "command": "configurationDone",
        "arguments": {}
    });
    tx.send(configuration_done).map_err(|_| {
        anyhow::anyhow!("Failed to queue configurationDone DAP request for {session_id}")
    })
}

fn dap_launch_arguments(launch: &DebugLaunchConfig, cwd: &Path) -> Value {
    let mut object = serde_json::Map::new();
    object.insert("name".to_string(), Value::String(launch.name.clone()));
    object.insert("type".to_string(), Value::String("lldb".to_string()));
    object.insert(
        "request".to_string(),
        Value::String(if launch.request.trim().is_empty() {
            "launch".to_string()
        } else {
            launch.request.clone()
        }),
    );
    if !launch.program.trim().is_empty() {
        object.insert("program".to_string(), Value::String(launch.program.clone()));
    }
    object.insert(
        "args".to_string(),
        Value::Array(launch.args.iter().cloned().map(Value::String).collect()),
    );
    object.insert(
        "cwd".to_string(),
        Value::String(
            launch
                .cwd
                .clone()
                .unwrap_or_else(|| cwd.display().to_string()),
        ),
    );
    if !launch.env.is_empty() {
        object.insert("env".to_string(), json!(launch.env));
    }
    object.insert("stopOnEntry".to_string(), Value::Bool(false));
    Value::Object(object)
}

fn is_response_to(message: &Value, command: &str) -> bool {
    message.get("type").and_then(Value::as_str) == Some("response")
        && message.get("command").and_then(Value::as_str) == Some(command)
}

fn is_event(message: &Value, event: &str) -> bool {
    message.get("type").and_then(Value::as_str) == Some("event")
        && message.get("event").and_then(Value::as_str) == Some(event)
}

async fn relay_dap_message(
    relay_manager: Arc<RelayManager>,
    session_id: SessionId,
    launch_name: Option<String>,
    message: Value,
) {
    if message.get("type").and_then(Value::as_str) == Some("event")
        && message.get("event").and_then(Value::as_str) == Some("output")
    {
        if let Some(output) = message
            .get("body")
            .and_then(|body| body.get("output"))
            .and_then(Value::as_str)
        {
            super::super::send_output(
                Arc::clone(&relay_manager),
                session_id,
                launch_name.clone(),
                "stdout",
                output,
            )
            .await;
        }
    }

    relay_manager
        .broadcast_client_message(Message::DebugDap {
            session_id,
            launch_name,
            message,
        })
        .await;
}
