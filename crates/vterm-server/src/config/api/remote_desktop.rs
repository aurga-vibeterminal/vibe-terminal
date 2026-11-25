use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::config::types::{RemoteDesktopConfig, TerminalServerConfig};
use crate::screen::ScreenServerSupervisor;

/// Web-facing remote-desktop config. `enabled` is writable; `platform`,
/// `arch`, `requires_permission` and `permission_granted` are read-only
/// runtime facts reported so the UI can show the permission row only on
/// Linux/macOS and pick the right driver download for the host's CPU.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebRemoteDesktopConfig {
    pub enabled: bool,
    pub platform: String,
    pub arch: String,
    pub requires_permission: bool,
    pub permission_granted: Option<bool>,
}

/// Result of a "request permission" action. Reflects whether the system
/// dialog was satisfied: the portal's ScreenCast source picker on Linux
/// (its answer also becomes the remembered display selection), the
/// Accessibility prompt on macOS.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RequestPermissionResult {
    pub granted: bool,
    pub message: String,
}

pub(super) fn remote_desktop_config_to_web(config: &RemoteDesktopConfig) -> WebRemoteDesktopConfig {
    let platform = current_platform();
    WebRemoteDesktopConfig {
        enabled: config.enabled,
        platform: platform.to_string(),
        // Host CPU (`std::env::consts::ARCH`, e.g. "x86_64", "aarch64") —
        // the webui may run in a remote browser, so the UA cannot be trusted
        // to pick e.g. the ARM64 virtual-display-driver download.
        arch: std::env::consts::ARCH.to_string(),
        requires_permission: requires_permission(platform),
        // The handlers enrich this via `probe_permission_granted` — the
        // live OS state needs an async screen-server query.
        permission_granted: None,
    }
}

/// The live permission state behind the webui's status row. Windows needs
/// no permission (`Some(true)`). macOS asks the screen-server — the process
/// that would inject — for its Accessibility trust state. Linux reports
/// unknown: portal consent is a per-stream restore token, not a process
/// grant. A failed probe is unknown rather than an error.
pub(super) async fn probe_permission_granted(supervisor: &ScreenServerSupervisor) -> Option<bool> {
    match current_platform() {
        "macos" => supervisor
            .get_input_status()
            .await
            .ok()
            .and_then(|info| info.accessibility_trusted),
        "linux" => None,
        _ => Some(true),
    }
}

pub(super) fn apply_web_remote_desktop_config(
    config: &mut TerminalServerConfig,
    payload: WebRemoteDesktopConfig,
) {
    config.remote_desktop.enabled = payload.enabled;
}

pub(super) async fn request_remote_desktop_permission(
    supervisor: &ScreenServerSupervisor,
) -> RequestPermissionResult {
    let platform = current_platform();
    if !requires_permission(platform) {
        // No OS consent dialog on this platform (Windows).
        return RequestPermissionResult {
            granted: true,
            message: String::new(),
        };
    }
    // The consent dialog opens now — not at stream start — and the answer
    // reflects it. Linux: screen capture is consented through the portal's
    // ScreenCast source picker (input needs no portal consent — it goes
    // through the kernel uinput route), so the picker pops immediately and
    // the selection is remembered. macOS: the input warm-up drives the
    // Accessibility (TCC) prompt and waits for the toggle.
    let result = if platform == "linux" {
        supervisor.request_screen_consent().await
    } else {
        supervisor.request_input_permission().await
    };
    match result {
        Ok((granted, message)) => RequestPermissionResult { granted, message },
        Err(e) => RequestPermissionResult {
            granted: false,
            message: e.to_string(),
        },
    }
}

/// Host operating system as reported by the compiler target
/// (`std::env::consts::OS`), e.g. `"macos"`, `"linux"`, `"windows"`.
fn current_platform() -> &'static str {
    std::env::consts::OS
}

/// Linux and macOS require an explicit screen-capture permission; Windows does not.
fn requires_permission(platform: &str) -> bool {
    matches!(platform, "macos" | "linux")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_writable_fields() {
        let mut config = TerminalServerConfig::default();
        let payload = WebRemoteDesktopConfig {
            enabled: true,
            platform: "macos".to_string(),
            arch: "aarch64".to_string(),
            requires_permission: true,
            permission_granted: None,
        };
        apply_web_remote_desktop_config(&mut config, payload);
        assert!(config.remote_desktop.enabled);
    }

    #[tokio::test]
    async fn request_permission_shape_matches_platform() {
        // With remote-desktop explicitly disabled the supervisor errors
        // before touching the OS, so on Linux/macOS the result is a
        // not-granted with an actionable message; on Windows the dialog-free
        // fast path reports granted without contacting the screen-server.
        // Disabled is forced here so the test stays deterministic even
        // though desktop builds ship remote desktop enabled by default.
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rd-permission-tests-{unique}.json"));
        let config_manager = Arc::new(
            crate::config::store::ConfigManager::load_from_path(path)
                .await
                .unwrap(),
        );
        {
            let mut config = config_manager.snapshot().await;
            config.remote_desktop.enabled = false;
            let _ = config_manager.replace(config).await;
        }
        let supervisor = ScreenServerSupervisor::new(config_manager);
        let result = request_remote_desktop_permission(&supervisor).await;
        let platform = current_platform();
        assert_eq!(result.granted, !requires_permission(platform));
        if requires_permission(platform) {
            assert!(!result.message.is_empty(), "failure must carry a reason");
        }
    }
}
