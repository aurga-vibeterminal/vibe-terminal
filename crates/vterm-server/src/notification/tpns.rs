use std::time::SystemTime;

use async_trait::async_trait;

use crate::config::{NotificationChannel, NotificationConfig};

use super::types::{
    DeliveryReceipt, DeliveryState, NotificationError, NotificationProvider, NotificationRequest,
};

const TPNS_SECRET_ENV: &str = "TERMINAL_SERVER_NOTIFICATION_TPNS_SECRET";

pub(crate) struct TpnsProvider {
    access_id: String,
}

impl TpnsProvider {
    pub(crate) fn from_config(config: &NotificationConfig) -> Result<Self, NotificationError> {
        if !config.tpns.enabled {
            return Err(NotificationError::ProviderDisabled(
                NotificationChannel::Tpns,
            ));
        }

        let access_id = config
            .tpns
            .access_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                NotificationError::InvalidProviderConfig(
                    "notification.tpns.access_id must be set".to_string(),
                )
            })?
            .to_string();

        Ok(Self { access_id })
    }
}

#[async_trait]
impl NotificationProvider for TpnsProvider {
    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Tpns
    }

    async fn deliver(
        &self,
        request: NotificationRequest,
    ) -> Result<DeliveryReceipt, NotificationError> {
        std::env::var(TPNS_SECRET_ENV).map_err(|_| {
            NotificationError::MissingCredentials(format!("set {}", TPNS_SECRET_ENV))
        })?;

        Ok(DeliveryReceipt {
            channel: NotificationChannel::Tpns,
            target_id: request.target.id,
            event: request.event,
            state: DeliveryState::DryRun,
            provider_message: format!("TPNS dry run accepted for access id {}", self.access_id),
            delivered_at: SystemTime::now(),
        })
    }
}
