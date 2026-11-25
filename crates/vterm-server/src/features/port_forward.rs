//! Reverse-proxy port forwarding requests.

use tracing::{info, warn};
use vterm_common::ErrorCode;
use vterm_core::protocol::{Message, ReverseProxyPurpose};

use crate::connection::Connection;

impl Connection {
    pub(crate) async fn handle_port_forward_message(
        &mut self,
        message: Message,
    ) -> anyhow::Result<()> {
        match message {
            Message::RequestReverseProxy {
                target_port,
                auth_token,
                purpose,
            } => {
                if !self.has_valid_auth_token(auth_token.as_deref()) {
                    warn!(
                        "Rejected port forwarding request from {} for target port {} due to invalid auth token",
                        self.addr, target_port
                    );
                    self.send_error(
                        ErrorCode::AuthenticationFailed,
                        "Invalid authentication token",
                    )
                    .await;
                    return Ok(());
                }

                info!(
                    "Authenticated port forwarding request from {} for target port {}",
                    self.addr, target_port
                );

                let response = match self.port_forward_manager.ensure_forward(target_port).await {
                    Ok(lease) => {
                        info!(
                            "Port forwarding ready for {}: target port {} mapped to proxy port {} (reused_existing={})",
                            self.addr, lease.target_port, lease.proxy_port, lease.reused_existing
                        );
                        let settings_ticket = (purpose
                            == Some(ReverseProxyPurpose::ServerSettings))
                        .then(|| {
                            self.auth_token
                                .as_deref()
                                .map(crate::http_auth::issue_settings_ticket)
                        })
                        .flatten();
                        Message::ReverseProxyReady {
                            target_port: lease.target_port,
                            proxy_port: lease.proxy_port,
                            reused_existing: lease.reused_existing,
                            settings_ticket,
                            error: None,
                        }
                    }
                    Err(error) => {
                        warn!(
                            "Port forwarding request from {} for target port {} failed: {}",
                            self.addr, target_port, error
                        );
                        Message::ReverseProxyReady {
                            target_port,
                            proxy_port: 0,
                            reused_existing: false,
                            settings_ticket: None,
                            error: Some(format!("{error:#}")),
                        }
                    }
                };
                self.send_message(response).await?;
            }
            Message::ListReverseProxies { auth_token } => {
                if !self.has_valid_auth_token(auth_token.as_deref()) {
                    warn!(
                        "Rejected port forwarding listing request from {} due to invalid auth token",
                        self.addr
                    );
                    self.send_error(
                        ErrorCode::AuthenticationFailed,
                        "Invalid authentication token",
                    )
                    .await;
                    return Ok(());
                }
                let proxies = self.port_forward_manager.list_forwards().await;
                info!(
                    "Sending {} port forward mappings to {}",
                    proxies.len(),
                    self.addr
                );
                self.send_message(Message::ReverseProxyList { proxies })
                    .await?;
            }
            other => {
                warn!("Unhandled port forward message: {}", other.msg_name());
            }
        }

        Ok(())
    }
}
