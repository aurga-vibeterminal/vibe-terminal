use std::time::SystemTime;

use async_trait::async_trait;

use crate::config::{NotificationChannel, NotificationConfig};

use super::types::{
    DeliveryReceipt, DeliveryState, NotificationError, NotificationProvider, NotificationRequest,
};

const APNS_KEY_ENV: &str = "TERMINAL_SERVER_NOTIFICATION_APNS_KEY";

pub(crate) struct ApnsProvider {
    topic: String,
}

impl ApnsProvider {
    pub(crate) fn from_config(config: &NotificationConfig) -> Result<Self, NotificationError> {
        if !config.apns.enabled {
            return Err(NotificationError::ProviderDisabled(
                NotificationChannel::Apns,
            ));
        }

        let topic = config
            .apns
            .topic
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                NotificationError::InvalidProviderConfig(
                    "notification.apns.topic must be set".to_string(),
                )
            })?
            .to_string();

        Ok(Self { topic })
    }
}

#[async_trait]
impl NotificationProvider for ApnsProvider {
    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Apns
    }

    async fn deliver(
        &self,
        request: NotificationRequest,
    ) -> Result<DeliveryReceipt, NotificationError> {
        std::env::var(APNS_KEY_ENV)
            .map_err(|_| NotificationError::MissingCredentials(format!("set {}", APNS_KEY_ENV)))?;

        let topic = request
            .target
            .topic
            .clone()
            .unwrap_or_else(|| self.topic.clone());
        Ok(DeliveryReceipt {
            channel: NotificationChannel::Apns,
            target_id: request.target.id,
            event: request.event,
            state: DeliveryState::DryRun,
            provider_message: format!("APNs dry run accepted for topic {}", topic),
            delivered_at: SystemTime::now(),
        })
    }
}
