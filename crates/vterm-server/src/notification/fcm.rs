use std::time::SystemTime;

use async_trait::async_trait;

use crate::config::{NotificationChannel, NotificationConfig};

use super::types::{
    DeliveryReceipt, DeliveryState, NotificationError, NotificationProvider, NotificationRequest,
};

const FCM_CREDENTIALS_ENV: &str = "TERMINAL_SERVER_NOTIFICATION_FCM_CREDENTIALS";

pub(crate) struct FcmProvider {
    project_id: String,
}

impl FcmProvider {
    pub(crate) fn from_config(config: &NotificationConfig) -> Result<Self, NotificationError> {
        if !config.fcm.enabled {
            return Err(NotificationError::ProviderDisabled(
                NotificationChannel::Fcm,
            ));
        }

        let project_id = config
            .fcm
            .project_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                NotificationError::InvalidProviderConfig(
                    "notification.fcm.project_id must be set".to_string(),
                )
            })?
            .to_string();

        Ok(Self { project_id })
    }
}

#[async_trait]
impl NotificationProvider for FcmProvider {
    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Fcm
    }

    async fn deliver(
        &self,
        request: NotificationRequest,
    ) -> Result<DeliveryReceipt, NotificationError> {
        std::env::var(FCM_CREDENTIALS_ENV).map_err(|_| {
            NotificationError::MissingCredentials(format!("set {}", FCM_CREDENTIALS_ENV))
        })?;

        Ok(DeliveryReceipt {
            channel: NotificationChannel::Fcm,
            target_id: request.target.id,
            event: request.event,
            state: DeliveryState::DryRun,
            provider_message: format!("FCM dry run accepted for project {}", self.project_id),
            delivered_at: SystemTime::now(),
        })
    }
}
