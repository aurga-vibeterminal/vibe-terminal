use serde::{Deserialize, Serialize};

use crate::config::types::{NotificationConfig, TerminalServerConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct WebApnsConfig {
    enabled: bool,
    certificate_path: String,
    certificate_password: String,
    environment: String,
    topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct WebFcmConfig {
    enabled: bool,
    service_account_json: String,
    project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct WebTpnsConfig {
    enabled: bool,
    access_id: String,
    secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct WebNotificationConfig {
    apns: WebApnsConfig,
    fcm: WebFcmConfig,
    tpns: WebTpnsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct WebNotificationTestResult {
    success: bool,
    message: String,
}

pub(super) fn notification_config_to_web(config: &NotificationConfig) -> WebNotificationConfig {
    WebNotificationConfig {
        apns: WebApnsConfig {
            enabled: config.apns.enabled,
            certificate_path: config.apns.certificate_path.clone().unwrap_or_default(),
            certificate_password: config.apns.certificate_password.clone().unwrap_or_default(),
            environment: config.apns.environment.clone(),
            topic: config.apns.topic.clone().unwrap_or_default(),
        },
        fcm: WebFcmConfig {
            enabled: config.fcm.enabled,
            service_account_json: config.fcm.service_account_json.clone().unwrap_or_default(),
            project_id: config.fcm.project_id.clone().unwrap_or_default(),
        },
        tpns: WebTpnsConfig {
            enabled: config.tpns.enabled,
            access_id: config.tpns.access_id.clone().unwrap_or_default(),
            secret_key: config.tpns.secret_key.clone().unwrap_or_default(),
        },
    }
}

pub(super) fn apply_web_notification_config(
    config: &mut TerminalServerConfig,
    payload: WebNotificationConfig,
) {
    config.notification.enabled =
        payload.apns.enabled || payload.fcm.enabled || payload.tpns.enabled;
    config.notification.apns.enabled = payload.apns.enabled;
    config.notification.apns.certificate_path =
        Some(payload.apns.certificate_path.trim().to_string()).filter(|value| !value.is_empty());
    config.notification.apns.certificate_password =
        Some(payload.apns.certificate_password.trim().to_string())
            .filter(|value| !value.is_empty());
    config.notification.apns.environment = payload.apns.environment.trim().to_string();
    config.notification.apns.topic =
        Some(payload.apns.topic.trim().to_string()).filter(|value| !value.is_empty());

    config.notification.fcm.enabled = payload.fcm.enabled;
    config.notification.fcm.service_account_json =
        Some(payload.fcm.service_account_json.trim().to_string()).filter(|value| !value.is_empty());
    config.notification.fcm.project_id =
        Some(payload.fcm.project_id.trim().to_string()).filter(|value| !value.is_empty());

    config.notification.tpns.enabled = payload.tpns.enabled;
    config.notification.tpns.access_id =
        Some(payload.tpns.access_id.trim().to_string()).filter(|value| !value.is_empty());
    config.notification.tpns.secret_key =
        Some(payload.tpns.secret_key.trim().to_string()).filter(|value| !value.is_empty());
}

pub(super) fn test_notification_result(service: &str, enabled: bool) -> WebNotificationTestResult {
    if enabled {
        WebNotificationTestResult {
            success: true,
            message: format!("{service} test request accepted"),
        }
    } else {
        WebNotificationTestResult {
            success: false,
            message: format!("{service} is not enabled"),
        }
    }
}
