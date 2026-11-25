use std::collections::BTreeMap;
use std::time::SystemTime;

use async_trait::async_trait;

use crate::config::{NotificationChannel, NotificationTargetConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub data: BTreeMap<String, String>,
}

impl NotificationPayload {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            data: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationRequest {
    pub event: String,
    pub target: NotificationTargetConfig,
    pub payload: NotificationPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryState {
    DryRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryReceipt {
    pub channel: NotificationChannel,
    pub target_id: String,
    pub event: String,
    pub state: DeliveryState,
    pub provider_message: String,
    pub delivered_at: SystemTime,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationStatus {
    pub enabled: bool,
    pub registered_target_count: usize,
    pub pending_count: usize,
    pub delivered_count: u64,
    pub dry_run_count: u64,
    pub failed_count: u64,
    pub last_error: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Notifications are disabled")]
    Disabled,
    #[error("Notification queue is full")]
    QueueFull,
    #[error("Notification provider '{0:?}' is disabled")]
    ProviderDisabled(NotificationChannel),
    #[error("Notification credentials are missing: {0}")]
    MissingCredentials(String),
    #[error("Notification provider configuration is incomplete: {0}")]
    InvalidProviderConfig(String),
}

#[async_trait]
pub(crate) trait NotificationProvider: Send + Sync {
    fn channel(&self) -> NotificationChannel;

    async fn deliver(
        &self,
        request: NotificationRequest,
    ) -> Result<DeliveryReceipt, NotificationError>;
}
