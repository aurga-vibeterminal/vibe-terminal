use std::sync::Arc;

use crate::voice::VoiceServiceState;

#[cfg(DEVELOPMENT)]
const TRAY_TOOLTIP: &str = "AURGA Vibe Terminal Server (Dev)";
#[cfg(not(DEVELOPMENT))]
const TRAY_TOOLTIP: &str = "AURGA Vibe Terminal Server";

#[cfg(all(feature = "ui", target_os = "linux"))]
mod linux;
#[cfg(all(feature = "ui", target_os = "macos"))]
mod macos;
#[cfg(feature = "ui")]
mod open_client;
#[cfg(all(feature = "ui", target_os = "windows"))]
mod windows;

#[cfg(all(feature = "ui", target_os = "linux"))]
pub use linux::{has_desktop, TrayIcon};
#[cfg(all(feature = "ui", target_os = "macos"))]
pub use macos::{has_desktop, TrayIcon};
#[cfg(all(feature = "ui", target_os = "windows"))]
pub use windows::{has_desktop, TrayIcon};

/// Shared state between the server thread and tray thread.
#[derive(Debug, Clone)]
pub struct TrayVoiceState {
    pub voice_enabled: bool,
    pub is_local: bool,
    pub service_state: VoiceServiceState,
}

impl Default for TrayVoiceState {
    fn default() -> Self {
        Self {
            voice_enabled: false,
            is_local: true,
            service_state: VoiceServiceState::NotLoaded,
        }
    }
}

/// Actions sent from the tray thread to the server thread.
pub enum TrayAction {
    StartVoice,
    StopVoice,
    StopStaticHttp { root: String },
}

/// Wrapper type passed to TrayIcon::new().
pub type SharedVoiceState = Arc<std::sync::RwLock<TrayVoiceState>>;

fn settings_url(config_port: u16) -> String {
    let ticket = crate::http_auth::issue_settings_ticket("local-tray");
    format!("http://localhost:{config_port}/?ticket={ticket}")
}

#[cfg(feature = "ui")]
fn open_settings(config_port: u16) {
    let url = settings_url(config_port);
    if let Err(error) = open::that(&url) {
        tracing::warn!("Failed to open settings page: {}", error);
    }
}

/// Voice Input menu item state for tray rendering.
///
/// The menu item controls the **vterm-voice process lifecycle** (Start/Stop
/// Server), mirroring the WebUI's Start/Stop buttons — NOT the model load
/// lifecycle. The label adapts to:
/// - server not running → "Start Vibe Voice"
/// - server running, model loading → "Stop Vibe Voice (loading…)" (disabled)
/// - server running, model loaded → "Stop Vibe Voice (model on device)"
/// - server running, model errored → "Stop Vibe Voice (model error)"
/// - server running, no model → "Stop Vibe Voice"
///
/// The action dispatched (`StartVoice` vs `StopVoice`) is decided by the
/// platform dispatcher based on `is_running`, not by this label.
pub struct VoiceInputMenuItem {
    /// Display label for the menu item.
    pub label: String,
    /// Whether the item should be disabled (only true during Loading).
    pub disabled: bool,
    /// Whether the server is currently running — drives Start vs Stop.
    pub is_running: bool,
}

impl VoiceInputMenuItem {
    pub fn from_state(is_running: bool, model_state: &VoiceServiceState) -> Self {
        if !is_running {
            return Self {
                label: "Start Vibe Voice".to_string(),
                disabled: false,
                is_running: false,
            };
        }
        // Server is up — label reports "Stop" with a model-state suffix.
        let (suffix, disabled) = match model_state {
            VoiceServiceState::Loading => (" (loading…)".to_string(), true),
            VoiceServiceState::Loaded { model_id, device } => {
                // Compact: drop the qwen3-asr- prefix if present.
                let short_model = model_id.strip_prefix("qwen3-asr-").unwrap_or(model_id);
                (format!(" ({} on {})", short_model, device), false)
            }
            VoiceServiceState::Error { message } => {
                let short = message.lines().next().unwrap_or(message);
                let detail = if short.len() > 30 {
                    format!("{}…", &short[..30])
                } else {
                    short.to_string()
                };
                (format!(" (model error: {})", detail), false)
            }
            VoiceServiceState::NotLoaded => (String::new(), false),
        };
        Self {
            label: format!("Stop Vibe Voice{}", suffix),
            disabled,
            is_running: true,
        }
    }
}
