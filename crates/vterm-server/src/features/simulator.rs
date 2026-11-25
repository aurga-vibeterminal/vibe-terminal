//! iOS simulator and Android emulator lifecycle, driven through the
//! screen-server supervisor.

use tracing::{info, warn};
use vterm_core::protocol::Message;

use crate::connection::Connection;

impl Connection {
    pub(crate) async fn handle_simulator_message(
        &mut self,
        message: Message,
    ) -> anyhow::Result<()> {
        match message {
            Message::BootSimulator { spec, server_id } => {
                let udid = simulator_udid(&spec);
                info!(
                    "Client {} booting simulator {udid} (spec {spec})",
                    self.addr
                );
                let error = match self.screen_supervisor.boot_simulator(udid).await {
                    Ok(()) => None,
                    Err(e) => Some(format!("{e:#}")),
                };
                self.send_message(Message::BootSimulatorResult {
                    spec,
                    server_id,
                    error,
                })
                .await?;
            }
            Message::ShutdownSimulator { spec, server_id } => {
                let udid = simulator_udid(&spec);
                info!(
                    "Client {} shutting down simulator {udid} (spec {spec})",
                    self.addr
                );
                let error = match self.screen_supervisor.shutdown_simulator(udid).await {
                    Ok(()) => None,
                    Err(e) => Some(format!("{e:#}")),
                };
                self.send_message(Message::ShutdownSimulatorResult {
                    spec,
                    server_id,
                    error,
                })
                .await?;
            }
            Message::CopySimulatorClipboard { spec } => {
                let udid = simulator_udid(&spec);
                info!(
                    "Client {} copying simulator {udid} clipboard (spec {spec})",
                    self.addr
                );
                let (text, error) = match self.screen_supervisor.get_simulator_clipboard(udid).await
                {
                    Ok(text) => (text, None),
                    Err(e) => (String::new(), Some(format!("{e:#}"))),
                };
                self.send_message(Message::CopySimulatorClipboardResult { spec, text, error })
                    .await?;
            }
            Message::BootAndroidEmulator { avd, server_id } => {
                info!("Client {} booting android emulator {avd}", self.addr);
                self.ensure_screen_event_forwarder().await;
                let (operation_id, error) =
                    match self.screen_supervisor.boot_android_emulator(&avd).await {
                        Ok(operation_id) => (operation_id, None),
                        Err(e) => (None, Some(format!("{e:#}"))),
                    };
                info!(
                    avd,
                    ?server_id,
                    operation_id,
                    ?error,
                    "android ctl: boot acceptance sent"
                );
                if error.is_none() && operation_id.is_none() {
                    self.push_android_booted_legacy(avd.clone(), server_id.clone());
                }
                self.send_message(Message::BootAndroidEmulatorResult {
                    avd,
                    server_id,
                    operation_id,
                    error,
                })
                .await?;
            }
            Message::ShutdownAndroidEmulator { serial, server_id } => {
                info!(
                    "Client {} shutting down android emulator {serial}",
                    self.addr
                );
                self.ensure_screen_event_forwarder().await;
                let (operation_id, error) = match self
                    .screen_supervisor
                    .shutdown_android_emulator(&serial)
                    .await
                {
                    Ok(operation_id) => (operation_id, None),
                    Err(e) => (None, Some(format!("{e:#}"))),
                };
                info!(
                    serial,
                    ?server_id,
                    operation_id,
                    ?error,
                    "android ctl: shutdown acceptance sent"
                );
                if error.is_none() && operation_id.is_none() {
                    let _ = self.tx.send(Message::AndroidEmulatorShutdown {
                        serial: serial.clone(),
                        server_id: server_id.clone(),
                        operation_id: None,
                    });
                }
                self.send_message(Message::ShutdownAndroidEmulatorResult {
                    serial,
                    server_id,
                    operation_id,
                    error,
                })
                .await?;
            }
            Message::RotateAndroidDevice { serial } => {
                info!("Client {} rotating android device {serial}", self.addr);
                let error = match self.screen_supervisor.rotate_android_device(&serial).await {
                    Ok(()) => None,
                    Err(e) => Some(format!("{e:#}")),
                };
                self.send_message(Message::RotateAndroidDeviceResult { serial, error })
                    .await?;
            }
            other => {
                warn!("Unhandled simulator message: {}", other.msg_name());
            }
        }

        Ok(())
    }

    /// Compatibility with a screen-server that predates operation events.
    fn push_android_booted_legacy(&self, avd: String, server_id: Option<String>) {
        let tx = self.tx.clone();
        let supervisor = self.screen_supervisor.clone();
        tokio::spawn(async move {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
            loop {
                if std::time::Instant::now() >= deadline {
                    return;
                }
                if let Ok(entries) = supervisor.list_sources().await {
                    let serial = entries.iter().find_map(|entry| {
                        let serial = entry.spec.strip_prefix("android:")?;
                        (entry.title.trim() == avd
                            && matches!(
                                entry.status,
                                crate::screen::ctl::SourceStatus::Available
                                    | crate::screen::ctl::SourceStatus::Streaming
                            ))
                        .then(|| serial.to_string())
                    });
                    if let Some(serial) = serial {
                        let _ = tx.send(Message::AndroidEmulatorBooted {
                            avd,
                            serial,
                            server_id,
                            operation_id: None,
                        });
                        return;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
    }
}

/// Extract the simulator UDID from a source spec (`sim:UDID` → `UDID`).
/// Non-simulator specs are passed through (the screen-server rejects them).
fn simulator_udid(spec: &str) -> &str {
    spec.strip_prefix("sim:").unwrap_or(spec)
}
