use serde::{Deserialize, Serialize};

use crate::config::types::{normalize_terminal_encoding, TerminalServerConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebDeviceConfig {
    pub(super) baud_rate: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) encoding: Option<String>,
}

pub(super) fn device_config_to_web(config: &TerminalServerConfig) -> WebDeviceConfig {
    WebDeviceConfig {
        baud_rate: config.device.serial_baud_rate,
        encoding: config.device.serial_encoding.clone(),
    }
}

pub(super) fn apply_web_device_config(config: &mut TerminalServerConfig, payload: WebDeviceConfig) {
    config.device.serial_baud_rate = payload.baud_rate;
    config.device.serial_encoding = normalize_terminal_encoding(payload.encoding.as_deref());
}
