//! Developer tools: debug launches and DAP message forwarding, quick
//! commands, MCP app setup, and `.vterm` config generation.

use std::sync::Arc;

use tracing::{info, warn};
use vterm_common::ErrorCode;
use vterm_core::protocol::Message;

use crate::connection::Connection;

impl Connection {
    pub(crate) async fn handle_debug_tool_message(
        &mut self,
        message: Message,
    ) -> anyhow::Result<()> {
        match message {
            Message::GenerateVtermConfig {
                session_id,
                vterm_path,
                config_type,
            } => {
                info!(
                    "Client {} requested .vterm config generation session={:?} path={} type={}",
                    self.addr, session_id, vterm_path, config_type
                );
                let error = crate::debug::generate_config(&vterm_path, &config_type)
                    .await
                    .err()
                    .map(|error| format!("{error:#}"));
                self.send_message(Message::GenerateVtermConfigResult {
                    session_id,
                    vterm_path,
                    config_type,
                    error,
                })
                .await?;
            }
            Message::DebugLaunchList { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let message = crate::debug::launch_list(&self.session_manager, session_id).await;
                self.send_message(message).await?;
            }
            Message::DebugStart {
                session_id,
                launch_name,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let message = crate::debug::start_launch(
                    Arc::clone(&self.session_manager),
                    Arc::clone(&self.relay_manager),
                    session_id,
                    launch_name,
                )
                .await;
                self.send_message(message).await?;
            }
            Message::QuickCommandList { session_id } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                self.send_message(
                    crate::debug::command_list(&self.session_manager, session_id).await,
                )
                .await?;
            }
            Message::RunQuickCommand { session_id, name } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                self.send_message(
                    crate::debug::run_command(&self.session_manager, session_id, name).await,
                )
                .await?;
            }
            Message::SetupMcpApp {
                session_id,
                executable,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let config = self.config_manager.snapshot().await;
                if !config.mcp.enabled {
                    warn!(
                        "{:?} connection {} requested MCP setup for session {} but MCP is disabled",
                        self.connection_kind, self.addr, session_id
                    );
                    // Answer instead of staying silent: SetupMcpApp rides the
                    // request connection, and a request that ends without any
                    // response surfaces on the client as the generic
                    // "Request handler for SetupMcpApp without a response".
                    self.send_message(Message::Error {
                        code: ErrorCode::ServerError,
                        message: "MCP server is not enabled on this host".to_string(),
                    })
                    .await?;
                    return Ok(());
                }
                let command = crate::mcp::add_command_for_app(&executable, &config.mcp);
                // Type the command without a trailing newline: the user
                // reviews it and runs it by pressing Enter. Always answer the
                // request: SetupMcpApp rides the request connection, and a
                // handler that ends without a response surfaces on the client
                // as "Request handler for SetupMcpApp completed without a
                // response".
                let error = self
                    .session_manager
                    .forward_input(session_id, None, command.into_bytes())
                    .await
                    .err()
                    .map(|error| error.to_string());
                self.send_message(Message::SetupMcpAppResult { session_id, error })
                    .await?;
            }
            Message::DebugDap {
                session_id,
                launch_name,
                message,
            } => {
                info!(
                    "{:?} connection {} forwarding DAP message for session {} launch={} message={}",
                    self.connection_kind,
                    self.addr,
                    session_id,
                    launch_name.as_deref().unwrap_or("<none>"),
                    message
                );
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                if let Err(error) = crate::debug::send_to_adapter(session_id, message) {
                    warn!(
                        "Failed to forward DAP message for session {} launch={}: {:#}",
                        session_id,
                        launch_name.as_deref().unwrap_or("<none>"),
                        error
                    );
                    self.send_error(ErrorCode::ServerError, &error.to_string())
                        .await;
                }
            }
            other => {
                warn!("Unhandled debug tool message: {}", other.msg_name());
            }
        }

        Ok(())
    }
}
