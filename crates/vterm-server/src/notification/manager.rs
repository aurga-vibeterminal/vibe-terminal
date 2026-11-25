use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, Notify, RwLock};
use tracing::{info, warn};

use crate::config::{
    ConfigManager, NotificationChannel, NotificationConfig, NotificationTargetConfig,
};

use super::apns::ApnsProvider;
use super::fcm::FcmProvider;
use super::queue::{NotificationQueue, QueuedNotification};
use super::tpns::TpnsProvider;
use super::types::{
    DeliveryReceipt, DeliveryState, NotificationError, NotificationPayload, NotificationProvider,
    NotificationRequest, NotificationStatus,
};

pub struct NotificationManager {
    config_manager: Arc<ConfigManager>,
    queue: Mutex<NotificationQueue>,
    notify: Notify,
    status: RwLock<NotificationStatus>,
}

impl NotificationManager {
    pub fn new(config_manager: Arc<ConfigManager>, queue_capacity: u16) -> Self {
        Self {
            config_manager,
            queue: Mutex::new(NotificationQueue::new(usize::from(queue_capacity))),
            notify: Notify::new(),
            status: RwLock::new(NotificationStatus::default()),
        }
    }

    pub async fn run(self: Arc<Self>) {
        if let Err(error) = self.initialize().await {
            warn!("Notification manager initialization failed: {}", error);
        }

        loop {
            if self.process_next().await {
                continue;
            }

            if let Some(deadline) = self.next_deadline().await {
                tokio::select! {
                    _ = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => {},
                    _ = self.notify.notified() => {},
                }
            } else {
                self.notify.notified().await;
            }
        }
    }

    async fn initialize(&self) -> Result<(), NotificationError> {
        let snapshot = self.config_manager.snapshot().await;
        let notification = snapshot.notification.clone();
        let enabled_targets = enabled_targets(&notification);
        self.update_status(|status| {
            status.enabled = notification.enabled;
            status.registered_target_count = enabled_targets.len();
            status.pending_count = 0;
        })
        .await;

        if !notification.enabled {
            self.log_status("disabled").await;
            return Ok(());
        }

        if notification.notify_on_startup {
            let mut payload = NotificationPayload::new(
                "Terminal Server",
                "Terminal server notification manager started",
            );
            payload
                .data
                .insert("event".to_string(), "server_started".to_string());
            payload.data.insert(
                "target_count".to_string(),
                enabled_targets.len().to_string(),
            );
            let enqueued = self
                .enqueue_notifications("server_started", payload, enabled_targets)
                .await?;
            info!(
                "Notification manager queued {} startup notification(s)",
                enqueued
            );
        }

        self.log_status("initialized").await;
        Ok(())
    }

    async fn process_next(&self) -> bool {
        let entry = {
            let mut queue = self.queue.lock().await;
            let entry = queue.dequeue_due();
            let pending_count = queue.len();
            drop(queue);
            self.update_status(|status| status.pending_count = pending_count)
                .await;
            entry
        };

        let Some(entry) = entry else {
            return false;
        };

        let snapshot = self.config_manager.snapshot().await;
        let notification = snapshot.notification.clone();
        match self.deliver(entry.clone(), &notification).await {
            Ok(receipt) => {
                self.record_delivery(receipt).await;
            }
            Err(error) => {
                if notification.enabled && entry.attempt < notification.max_retries {
                    let delay = retry_delay(entry.attempt.saturating_add(1));
                    if let Err(requeue_error) = self.requeue(entry, delay).await {
                        self.record_failure(requeue_error.to_string()).await;
                    } else {
                        self.record_failure(error.to_string()).await;
                    }
                } else {
                    self.record_failure(error.to_string()).await;
                }
            }
        }

        true
    }

    async fn enqueue_notifications(
        &self,
        event: &str,
        payload: NotificationPayload,
        targets: Vec<NotificationTargetConfig>,
    ) -> Result<usize, NotificationError> {
        let mut queue = self.queue.lock().await;
        let mut enqueued = 0usize;

        for target in targets {
            let request = NotificationRequest {
                event: event.to_string(),
                target,
                payload: payload.clone(),
            };
            queue.enqueue(request)?;
            enqueued += 1;
        }

        let pending_count = queue.len();
        drop(queue);

        self.update_status(|status| status.pending_count = pending_count)
            .await;
        if enqueued > 0 {
            self.notify.notify_waiters();
        }
        Ok(enqueued)
    }

    pub async fn enqueue_event(
        &self,
        event: &str,
        payload: NotificationPayload,
    ) -> Result<usize, NotificationError> {
        let snapshot = self.config_manager.snapshot().await;
        let notification = snapshot.notification.clone();
        if !notification.enabled {
            return Err(NotificationError::Disabled);
        }

        self.enqueue_notifications(event, payload, enabled_targets(&notification))
            .await
    }

    async fn requeue(
        &self,
        entry: QueuedNotification,
        delay: Duration,
    ) -> Result<(), NotificationError> {
        let mut queue = self.queue.lock().await;
        queue.enqueue_with_delay(entry.request, entry.attempt.saturating_add(1), delay)?;
        let pending_count = queue.len();
        drop(queue);

        self.update_status(|status| status.pending_count = pending_count)
            .await;
        self.notify.notify_waiters();
        Ok(())
    }

    async fn next_deadline(&self) -> Option<std::time::Instant> {
        let queue = self.queue.lock().await;
        queue.next_deadline()
    }

    async fn deliver(
        &self,
        entry: QueuedNotification,
        config: &NotificationConfig,
    ) -> Result<DeliveryReceipt, NotificationError> {
        if !config.enabled {
            return Err(NotificationError::Disabled);
        }

        let provider = provider_for_channel(entry.request.target.channel.clone(), config)?;
        let channel = provider.channel();
        let receipt = provider.deliver(entry.request).await?;

        if matches!(receipt.state, DeliveryState::DryRun) {
            info!(
                "Notification {:?} dry run: {}",
                channel, receipt.provider_message
            );
        }

        Ok(receipt)
    }

    async fn record_delivery(&self, receipt: DeliveryReceipt) {
        let summary = format!(
            "{:?} {} {} {:?}",
            receipt.channel, receipt.target_id, receipt.event, receipt.state
        );
        self.update_status(|status| {
            status.delivered_count = status.delivered_count.saturating_add(1);
            if matches!(receipt.state, DeliveryState::DryRun) {
                status.dry_run_count = status.dry_run_count.saturating_add(1);
            }
            status.last_error = None;
        })
        .await;
        self.log_status(&summary).await;
    }

    async fn record_failure(&self, error: String) {
        self.update_status(|status| {
            status.failed_count = status.failed_count.saturating_add(1);
            status.last_error = Some(error.clone());
        })
        .await;
        self.log_status("delivery failed").await;
    }

    async fn log_status(&self, context: &str) {
        let status = self.status.read().await.clone();
        info!(
            "Notification manager {}: enabled={}, targets={}, pending={}, delivered={}, dry_run={}, failed={}, last_error={}",
            context,
            status.enabled,
            status.registered_target_count,
            status.pending_count,
            status.delivered_count,
            status.dry_run_count,
            status.failed_count,
            status.last_error.as_deref().unwrap_or("none")
        );
    }

    async fn update_status(&self, update: impl FnOnce(&mut NotificationStatus)) {
        let mut status = self.status.write().await;
        update(&mut status);
    }
}

fn enabled_targets(config: &NotificationConfig) -> Vec<NotificationTargetConfig> {
    config
        .targets
        .iter()
        .filter(|target| target.enabled)
        .filter(|target| !target.id.trim().is_empty())
        .filter(|target| !target.device_token.trim().is_empty())
        .cloned()
        .collect()
}

fn provider_for_channel(
    channel: NotificationChannel,
    config: &NotificationConfig,
) -> Result<Box<dyn NotificationProvider>, NotificationError> {
    match channel {
        NotificationChannel::Apns => Ok(Box::new(ApnsProvider::from_config(config)?)),
        NotificationChannel::Fcm => Ok(Box::new(FcmProvider::from_config(config)?)),
        NotificationChannel::Tpns => Ok(Box::new(TpnsProvider::from_config(config)?)),
    }
}

fn retry_delay(attempt: u8) -> Duration {
    let seconds = 2u64.saturating_pow(u32::from(attempt)).min(60);
    Duration::from_secs(seconds)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::config::{
        ConfigManager, ConfigPatchRequest, FcmNotificationConfigPatch, NotificationChannel,
        NotificationConfigPatch, NotificationTargetConfigPatch,
    };

    use super::*;

    #[tokio::test]
    async fn initialize_queues_startup_notifications() {
        let path = std::env::temp_dir().join(format!(
            "vterm-server-notification-test-{}.json",
            std::process::id()
        ));
        let _ = tokio::fs::remove_file(&path).await;
        let config_manager = Arc::new(ConfigManager::load_from_path(path.clone()).await.unwrap());
        config_manager
            .apply_patch(ConfigPatchRequest {
                notification: Some(NotificationConfigPatch {
                    enabled: Some(true),
                    notify_on_startup: Some(true),
                    targets: Some(vec![NotificationTargetConfigPatch {
                        id: Some("device-1".to_string()),
                        channel: Some(NotificationChannel::Fcm),
                        device_token: Some("token".to_string()),
                        enabled: Some(true),
                        ..NotificationTargetConfigPatch::default()
                    }]),
                    fcm: Some(FcmNotificationConfigPatch {
                        enabled: Some(true),
                        service_account_json: Some(Some("service-account".to_string())),
                        project_id: Some(Some("project".to_string())),
                    }),
                    ..NotificationConfigPatch::default()
                }),
                ..ConfigPatchRequest::default()
            })
            .await
            .unwrap();

        let manager = NotificationManager::new(config_manager, 8);
        manager.initialize().await.unwrap();

        let status = manager.status.read().await.clone();
        assert_eq!(status.pending_count, 1);
        assert_eq!(status.registered_target_count, 1);

        let _ = tokio::fs::remove_file(&path).await;
    }
}
