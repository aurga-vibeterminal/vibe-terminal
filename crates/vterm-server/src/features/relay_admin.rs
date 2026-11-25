//! Relay administration messages handled on the server's own connections:
//! shared-server registration (RelayPrimary) and relay server listing.

use tracing::warn;
use vterm_common::ErrorCode;
use vterm_core::protocol::{ConnectionKind, Message};

use crate::connection::Connection;

impl Connection {
    pub(crate) async fn handle_relay_admin_message(
        &mut self,
        message: Message,
    ) -> anyhow::Result<()> {
        match message {
            Message::RelayRegisterServer { name, request_port } => {
                if self.connection_kind != ConnectionKind::RelayPrimary {
                    self.send_error(
                        ErrorCode::InvalidMessage,
                        "RelayRegisterServer is only allowed on RelayPrimary connections",
                    )
                    .await;
                    return Ok(());
                }
                let (server_id, terminal_tunnel_key) = self
                    .relay_manager
                    .register_shared_server(name, request_port, self.addr, self.tx.clone())
                    .await;
                self.shared_server_id = Some(server_id.clone());
                self.shared_server_connection_kind = Some(ConnectionKind::RelayPrimary);
                self.send_message(Message::RelayServerRegistered {
                    server_id,
                    terminal_tunnel_key: Some(terminal_tunnel_key),
                })
                .await?;
            }
            Message::ListRelayServers => {
                let servers = self.relay_manager.relay_server_list().await;
                self.send_message(Message::RelayServerList { servers })
                    .await?;
            }
            other => {
                warn!("Unhandled relay admin message: {}", other.msg_name());
            }
        }

        Ok(())
    }
}
