//! Screen-streaming tool-path REST handlers (coordination spec §4.3).
//!
//! The webui edits the persisted Android SDK / Android Studio directory
//! overrides here. `GET` reports both views: `configured` (what the user
//! saved, from the config snapshot) and the resolved `adb` / `agentDir`
//! (what the screen-server actually uses, via the supervisor). `PUT`
//! persists the overrides first, then pushes them to the screen-server; a
//! failed push is reported as `pushError` rather than an HTTP error because
//! the config is saved either way (the supervisor's watch loop re-pushes on
//! the next connection).

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use crate::screen::ctl as ctl;

use super::super::server::HttpResponse;
use crate::config::store::ConfigManager;
use crate::config::types::ConfigError;
use crate::screen::ScreenServerSupervisor;

/// One resolved tool path, camelCase for the webui.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebPathInfo {
    pub value: Option<String>,
    pub origin: ctl::PathOrigin,
    pub valid: bool,
    pub detail: String,
}

impl WebPathInfo {
    fn from(info: ctl::PathInfo) -> Self {
        WebPathInfo {
            value: info.value.map(|p| p.to_string_lossy().into_owned()),
            origin: info.origin,
            valid: info.valid,
            detail: info.detail,
        }
    }

    /// Placeholder used when the screen-server is unreachable (spec §4.3:
    /// `valid: false`, empty detail; the real reason goes into `error`).
    fn placeholder() -> Self {
        WebPathInfo {
            value: None,
            origin: ctl::PathOrigin::Auto,
            valid: false,
            detail: String::new(),
        }
    }
}

/// The persisted overrides (None = auto-detect).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebConfiguredPaths {
    pub android_sdk_path: Option<String>,
    pub android_studio_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebScreenPaths {
    pub sdk: WebPathInfo,
    pub agent_dir: WebPathInfo,
    pub configured: WebConfiguredPaths,
    /// Set when the supervisor could not reach the screen-server at all
    /// (GET), or when persisting succeeded but the push failed (PUT, as
    /// `push_error`).
    pub error: Option<String>,
    pub push_error: Option<String>,
}

/// The webui's Windows-service status / (un)install surface (camelCase).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebScreenService {
    pub installed: bool,
    pub running: bool,
    pub message: String,
    /// Whether a Virtual Display Driver device is present on the host —
    /// `Some` on Windows (the webui's service card hides the VDD download
    /// instructions), `None` on other platforms.
    pub vdd_installed: Option<bool>,
}

/// The VDD presence probe (see `WebScreenService::vdd_installed`).
fn vdd_probe() -> Option<bool> {
    #[cfg(target_os = "windows")]
    {
        Some(crate::screen::supervisor::vdd_installed())
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// GET /api/screen/service — SCM status of `VTermScreenSvc`.
pub(super) async fn get_screen_service(
    screen_supervisor: &Arc<ScreenServerSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    Ok(service_response(screen_supervisor.service_status().await))
}

/// POST /api/screen/service/install — elevated install (UAC prompt).
pub(super) async fn install_screen_service(
    screen_supervisor: &Arc<ScreenServerSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    Ok(service_response(screen_supervisor.service_install().await))
}

/// POST /api/screen/service/remove — elevated stop+delete (UAC prompt).
pub(super) async fn remove_screen_service(
    screen_supervisor: &Arc<ScreenServerSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    Ok(service_response(screen_supervisor.service_remove().await))
}

fn service_web(message: String) -> HttpResponse {
    // Keep the 200 + payload shape so the webui can always render the
    // message; `installed` stays false on failures.
    HttpResponse::json(&WebScreenService {
        installed: false,
        running: false,
        message,
        vdd_installed: vdd_probe(),
    })
    .unwrap_or_else(|_| HttpResponse::json_with_status(500, Vec::new()))
}

fn service_response(result: Result<ctl::ResponsePayload, anyhow::Error>) -> HttpResponse {
    match result {
        Ok(ctl::ResponsePayload::Service {
            installed,
            running,
            message,
        }) => HttpResponse::json(&WebScreenService {
            installed,
            running,
            message,
            vdd_installed: vdd_probe(),
        })
        .unwrap_or_else(|_| HttpResponse::json_with_status(500, Vec::new())),
        Ok(ctl::ResponsePayload::Error { message }) => service_web(message),
        Err(e) => service_web(format!("{e:#}")),
        Ok(other) => HttpResponse::json_with_status(
            500,
            serde_json::to_vec(&serde_json::json!({
                "error": format!("unexpected service response: {other:?}")
            }))
            .unwrap_or_default(),
        ),
    }
}

/// The webui's uinput probe (camelCase): whether the consent-free Linux
/// injection route is available. `error` is set when the screen-server was
/// unreachable; the flags stay false then.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebScreenInput {
    pub uinput_exists: bool,
    pub uinput_accessible: bool,
    pub detail: String,
    pub error: Option<String>,
}

/// GET /api/screen/input — the `/dev/uinput` probe, run by the
/// screen-server (the process that would use it).
pub(super) async fn get_screen_input_status(
    screen_supervisor: &Arc<ScreenServerSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    let (info, error) = match screen_supervisor.get_input_status().await {
        Ok(info) => (info, None),
        Err(e) => (
            ctl::InputStatusInfo {
                uinput_exists: false,
                uinput_accessible: false,
                detail: String::new(),
                accessibility_trusted: None,
            },
            Some(format!("{e:#}")),
        ),
    };
    Ok(HttpResponse::json(&WebScreenInput {
        uinput_exists: info.uinput_exists,
        uinput_accessible: info.uinput_accessible,
        detail: info.detail,
        error,
    })?)
}

/// The webui's screen-consent reset answer (camelCase).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebScreenConsentReset {
    pub cleared: u32,
    pub error: Option<String>,
}

/// POST /api/screen/consent/reset — forget the persisted portal ScreenCast
/// restore tokens so the next stream start shows the source picker again.
pub(super) async fn reset_screen_consent(
    screen_supervisor: &Arc<ScreenServerSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    let (cleared, error) = match screen_supervisor.reset_screen_consent().await {
        Ok(cleared) => (cleared, None),
        Err(e) => (0, Some(format!("{e:#}"))),
    };
    Ok(HttpResponse::json(&WebScreenConsentReset {
        cleared,
        error,
    })?)
}

pub(super) async fn get_screen_paths(
    config_manager: &Arc<ConfigManager>,
    screen_supervisor: &Arc<ScreenServerSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    let config = config_manager.snapshot().await;
    let configured = configured_paths(&config.remote_desktop);

    let (sdk, agent_dir, error) = match screen_supervisor.get_paths().await {
        Ok(paths) => (
            WebPathInfo::from(paths.sdk),
            WebPathInfo::from(paths.agent_dir),
            None,
        ),
        Err(e) => (
            WebPathInfo::placeholder(),
            WebPathInfo::placeholder(),
            Some(format!("{e:#}")),
        ),
    };

    Ok(HttpResponse::json(&WebScreenPaths {
        sdk,
        agent_dir,
        configured,
        error,
        push_error: None,
    })?)
}

pub(super) async fn update_screen_paths(
    config_manager: &Arc<ConfigManager>,
    screen_supervisor: &Arc<ScreenServerSupervisor>,
    body: &[u8],
) -> Result<HttpResponse, ConfigError> {
    let payload = super::parse_json_request::<WebConfiguredPaths>(body)?;
    let payload = WebConfiguredPaths {
        android_sdk_path: normalize(payload.android_sdk_path),
        android_studio_path: normalize(payload.android_studio_path),
    };

    // Persist first: even when the push below fails, the saved config is the
    // source of truth and the supervisor's watch loop re-pushes it later.
    let mut config = config_manager.snapshot().await;
    config.remote_desktop.android_sdk_path = payload.android_sdk_path.clone();
    config.remote_desktop.adb_path = None;
    config.remote_desktop.android_studio_path = payload.android_studio_path.clone();
    let update = config_manager.replace(config).await?;
    let configured = configured_paths(&update.config.remote_desktop);

    let (sdk, agent_dir, push_error) = match screen_supervisor
        .set_paths(
            payload.android_sdk_path.clone(),
            payload.android_studio_path.clone(),
            payload.android_sdk_path.is_none(),
            payload.android_studio_path.is_none(),
        )
        .await
    {
        Ok(paths) => (
            WebPathInfo::from(paths.sdk),
            WebPathInfo::from(paths.agent_dir),
            None,
        ),
        // Includes user-readable validation errors ("adb override ... is not
        // usable: ...") as well as plain connection failures; the webui
        // shows the text verbatim.
        Err(e) => (
            WebPathInfo::placeholder(),
            WebPathInfo::placeholder(),
            Some(format!("{e:#}")),
        ),
    };

    Ok(HttpResponse::json(&WebScreenPaths {
        sdk,
        agent_dir,
        configured,
        error: None,
        push_error,
    })?)
}

fn configured_paths(config: &crate::config::types::RemoteDesktopConfig) -> WebConfiguredPaths {
    WebConfiguredPaths {
        android_sdk_path: config.android_sdk_path.clone().or_else(|| {
            let adb = std::path::Path::new(config.adb_path.as_deref()?);
            adb.parent()?
                .parent()
                .map(|root| root.to_string_lossy().into_owned())
        }),
        android_studio_path: config.android_studio_path.clone(),
    }
}

/// An empty / whitespace-only override means "back to auto-detect".
fn normalize(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

// ---------------------------------------------------------------------------
// Encoder / frame-rate settings (ctl GetEncoders / SetStreamConfig)
// ---------------------------------------------------------------------------

/// One selectable encoder backend, as shown in the webui dropdown.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebEncoderEntry {
    pub id: String,
    pub label: String,
    pub available: bool,
    pub reason: String,
}

/// The persisted stream tunables (None = auto / 60 fps / 4K).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebConfiguredStream {
    pub encoder: Option<String>,
    pub fps: Option<u32>,
    /// Capture-resolution ceiling ("4k" | "2.5k" | "2k"; None = "4k").
    pub max_resolution: Option<String>,
    /// Fixed average bitrate in Mbps (one of the ctl `BITRATE_MBPS_OPTIONS`;
    /// None = auto — area-scaled).
    pub bitrate_mbps: Option<u8>,
    /// macOS headless hosts: capture the main display through a
    /// CGVirtualDisplay (applies to streams started after the push;
    /// ignored elsewhere).
    #[serde(default)]
    pub virtual_display: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebScreenEncoder {
    pub entries: Vec<WebEncoderEntry>,
    pub configured: WebConfiguredStream,
    /// Backend id currently in effect on the screen-server.
    pub current: Option<String>,
    /// Frame rate currently in effect on the screen-server.
    pub fps: Option<u32>,
    /// Max-resolution override in effect on the screen-server (None = "4k").
    pub max_resolution: Option<String>,
    /// Fixed-bitrate override in effect, kbit/s (None = auto area-scaled).
    pub bitrate_kbps: Option<u32>,
    /// Set when the supervisor could not reach the screen-server at all
    /// (GET), or when persisting succeeded but the push failed (PUT, as
    /// `push_error`).
    pub error: Option<String>,
    pub push_error: Option<String>,
}

/// The frame rates the webui offers; validated here AND on the
/// screen-server (which is the authoritative reject).
const ALLOWED_FPS: [u32; 3] = [15, 30, 60];
const ENCODER_IDS: [&str; 4] = ["auto", "vendor", "d3d12", "mf"];

/// Validate/normalize a PUT body: empty encoder → None (auto); unknown ids,
/// frame rates, and resolutions are rejected before anything is persisted.
fn normalize_stream(payload: WebConfiguredStream) -> Result<WebConfiguredStream, ConfigError> {
    let encoder = match normalize(payload.encoder) {
        Some(id) if ENCODER_IDS.contains(&id.as_str()) => Some(id),
        Some(id) => {
            return Err(ConfigError::BadRequest(format!(
                "unknown encoder {id:?} (want one of {ENCODER_IDS:?})"
            )))
        }
        None => None,
    };
    let fps = match payload.fps {
        Some(fps) if ALLOWED_FPS.contains(&fps) => Some(fps),
        Some(fps) => {
            return Err(ConfigError::BadRequest(format!(
                "unsupported fps {fps} (want one of {ALLOWED_FPS:?})"
            )))
        }
        None => None,
    };
    let max_resolution = match normalize(payload.max_resolution) {
        Some(id) if ctl::MAX_RESOLUTION_IDS.contains(&id.as_str()) => Some(id),
        Some(id) => {
            return Err(ConfigError::BadRequest(format!(
                "unknown max resolution {id:?} (want one of {:?})",
                ctl::MAX_RESOLUTION_IDS
            )))
        }
        None => None,
    };
    let bitrate_mbps = match payload.bitrate_mbps {
        Some(mbps) if ctl::BITRATE_MBPS_OPTIONS.contains(&mbps) => Some(mbps),
        Some(mbps) => {
            return Err(ConfigError::BadRequest(format!(
                "unsupported bitrate {mbps} Mbps (want one of {:?})",
                ctl::BITRATE_MBPS_OPTIONS
            )))
        }
        None => None,
    };
    Ok(WebConfiguredStream {
        encoder,
        fps,
        max_resolution,
        bitrate_mbps,
        virtual_display: payload.virtual_display,
    })
}

fn configured_stream(config: &crate::config::types::RemoteDesktopConfig) -> WebConfiguredStream {
    WebConfiguredStream {
        encoder: config.screen_encoder.clone(),
        fps: config.screen_fps,
        max_resolution: config.screen_max_resolution.clone(),
        bitrate_mbps: config.screen_bitrate_mbps,
        virtual_display: config.screen_virtual_display,
    }
}

/// Placeholder entry list for when the screen-server is unreachable: every
/// backend shown as unavailable, the real reason carried by `error`.
fn unreachable_entries() -> Vec<WebEncoderEntry> {
    ENCODER_IDS
        .iter()
        .map(|id| WebEncoderEntry {
            id: id.to_string(),
            label: id.to_string(),
            available: false,
            reason: String::new(),
        })
        .collect()
}

pub(super) async fn get_screen_encoder(
    config_manager: &Arc<ConfigManager>,
    screen_supervisor: &Arc<ScreenServerSupervisor>,
) -> Result<HttpResponse, ConfigError> {
    let config = config_manager.snapshot().await;
    let configured = configured_stream(&config.remote_desktop);

    let (entries, current, fps, max_resolution, bitrate_kbps, error) =
        match screen_supervisor.get_encoders().await {
            Ok(catalog) => (
                catalog
                    .entries
                    .into_iter()
                    .map(|e| WebEncoderEntry {
                        id: e.id,
                        label: e.label,
                        available: e.available,
                        reason: e.reason,
                    })
                    .collect(),
                Some(catalog.current),
                Some(catalog.fps),
                catalog.max_resolution,
                catalog.bitrate_kbps,
                None,
            ),
            Err(e) => (
                unreachable_entries(),
                None,
                None,
                None,
                None,
                Some(format!("{e:#}")),
            ),
        };

    Ok(HttpResponse::json(&WebScreenEncoder {
        entries,
        configured,
        current,
        fps,
        max_resolution,
        bitrate_kbps,
        error,
        push_error: None,
    })?)
}

pub(super) async fn update_screen_encoder(
    config_manager: &Arc<ConfigManager>,
    screen_supervisor: &Arc<ScreenServerSupervisor>,
    body: &[u8],
) -> Result<HttpResponse, ConfigError> {
    let payload = normalize_stream(super::parse_json_request::<WebConfiguredStream>(body)?)?;

    // Persist first: even when the push below fails, the saved config is the
    // source of truth and the supervisor's watch loop re-pushes it later.
    let mut config = config_manager.snapshot().await;
    config.remote_desktop.screen_encoder = payload.encoder.clone();
    config.remote_desktop.screen_fps = payload.fps;
    config.remote_desktop.screen_max_resolution = payload.max_resolution.clone();
    config.remote_desktop.screen_bitrate_mbps = payload.bitrate_mbps;
    config.remote_desktop.screen_virtual_display = payload.virtual_display;
    let update = config_manager.replace(config).await?;
    let configured = configured_stream(&update.config.remote_desktop);

    // Push the effective values (None config → "auto" / default fps / "4k" /
    // bitrate auto — Some(0) clears a previous fixed-bitrate override).
    let (current, fps, max_resolution, bitrate_kbps, push_error) = match screen_supervisor
        .set_stream_config(
            Some(
                payload
                    .encoder
                    .clone()
                    .unwrap_or_else(|| "auto".to_string()),
            ),
            Some(payload.fps.unwrap_or(crate::screen::DEFAULT_STREAM_FPS)),
            Some(
                payload
                    .max_resolution
                    .clone()
                    .unwrap_or_else(|| crate::screen::DEFAULT_MAX_RESOLUTION.to_string()),
            ),
            Some(u32::from(payload.bitrate_mbps.unwrap_or(0)) * 1000),
            Some(payload.virtual_display),
        )
        .await
    {
        Ok((encoder, fps_override, max_resolution, bitrate_kbps, _)) => (
            Some(encoder),
            Some(fps_override.unwrap_or(crate::screen::DEFAULT_STREAM_FPS)),
            max_resolution,
            bitrate_kbps,
            None,
        ),
        // Includes user-readable validation errors as well as plain
        // connection failures; the webui shows the text verbatim.
        Err(e) => (None, None, None, None, Some(format!("{e:#}"))),
    };

    let entries = match screen_supervisor.get_encoders().await {
        Ok(catalog) => catalog
            .entries
            .into_iter()
            .map(|e| WebEncoderEntry {
                id: e.id,
                label: e.label,
                available: e.available,
                reason: e.reason,
            })
            .collect(),
        Err(_) => unreachable_entries(),
    };

    Ok(HttpResponse::json(&WebScreenEncoder {
        entries,
        configured,
        current,
        fps,
        max_resolution,
        bitrate_kbps,
        error: None,
        push_error,
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_config_validation() {
        // Empty/absent encoder normalizes to auto (None).
        let s = normalize_stream(WebConfiguredStream {
            encoder: Some("  ".into()),
            fps: None,
            max_resolution: None,
            bitrate_mbps: None,
            virtual_display: false,
        })
        .unwrap();
        assert_eq!(s.encoder, None);
        assert_eq!(s.fps, None);
        assert_eq!(s.virtual_display, false);
        let s = normalize_stream(WebConfiguredStream {
            encoder: Some("vendor".into()),
            fps: Some(30),
            max_resolution: Some("2.5k".into()),
            bitrate_mbps: Some(25),
            virtual_display: true,
        })
        .unwrap();
        assert_eq!(s.encoder.as_deref(), Some("vendor"));
        assert_eq!(s.fps, Some(30));
        assert_eq!(s.max_resolution.as_deref(), Some("2.5k"));
        assert_eq!(s.virtual_display, true);
        // Unknown encoder / fps / max resolution rejected.
        assert!(normalize_stream(WebConfiguredStream {
            encoder: Some("hevc".into()),
            fps: None,
            max_resolution: None,
            bitrate_mbps: None,
            virtual_display: false,
        })
        .is_err());
        assert!(normalize_stream(WebConfiguredStream {
            encoder: None,
            fps: Some(45),
            max_resolution: None,
            bitrate_mbps: None,
            virtual_display: false,
        })
        .is_err());
        assert!(normalize_stream(WebConfiguredStream {
            encoder: None,
            fps: Some(0),
            max_resolution: None,
            bitrate_mbps: None,
            virtual_display: false,
        })
        .is_err());
        assert!(normalize_stream(WebConfiguredStream {
            encoder: None,
            fps: None,
            max_resolution: Some("8k".into()),
            bitrate_mbps: None,
            virtual_display: false,
        })
        .is_err());
        // Bitrate outside the offered options rejected.
        assert!(normalize_stream(WebConfiguredStream {
            encoder: None,
            fps: None,
            max_resolution: None,
            bitrate_mbps: Some(42),
            virtual_display: false,
        })
        .is_err());
    }

    #[test]
    fn web_encoder_serialize_camel_case() {
        let response = WebScreenEncoder {
            entries: vec![WebEncoderEntry {
                id: "vendor".into(),
                label: "GPU vendor".into(),
                available: true,
                reason: String::new(),
            }],
            configured: WebConfiguredStream {
                encoder: None,
                fps: Some(60),
                max_resolution: Some("4k".into()),
                bitrate_mbps: None,
                virtual_display: true,
            },
            current: Some("auto".into()),
            fps: Some(60),
            max_resolution: Some("4k".into()),
            bitrate_kbps: None,
            error: None,
            push_error: None,
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["entries"][0]["id"], "vendor");
        assert_eq!(json["configured"]["fps"], 60);
        assert_eq!(json["configured"]["maxResolution"], "4k");
        assert_eq!(json["configured"]["virtualDisplay"], true);
        assert_eq!(json["current"], "auto");
        assert_eq!(json["maxResolution"], "4k");
        assert!(json.get("pushError").is_some());
        // An absent virtualDisplay deserializes as false (older webui).
        let back: WebConfiguredStream = serde_json::from_value(
            serde_json::json!({"encoder": null, "fps": 60, "maxResolution": "4k", "bitrateMbps": null}),
        )
        .unwrap();
        assert_eq!(back.virtual_display, false);
    }

    #[test]
    fn web_input_status_serialize_camel_case() {
        let response = WebScreenInput {
            uinput_exists: true,
            uinput_accessible: false,
            detail: "Permission denied (os error 13)".into(),
            error: None,
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["uinputExists"], true);
        assert_eq!(json["uinputAccessible"], false);
        assert_eq!(json["detail"], "Permission denied (os error 13)");
        // The unreachable-screen-server case still renders all fields.
        let json = serde_json::to_value(&WebScreenInput {
            uinput_exists: false,
            uinput_accessible: false,
            detail: String::new(),
            error: Some("screen-server unreachable".into()),
        })
        .unwrap();
        assert_eq!(json["error"], "screen-server unreachable");
    }

    #[test]
    fn empty_override_normalizes_to_auto() {
        assert_eq!(normalize(Some("  ".to_string())), None);
        assert_eq!(normalize(Some(String::new())), None);
        assert_eq!(normalize(None), None);
        assert_eq!(
            normalize(Some("  C:\\tools\\adb.exe ".to_string())),
            Some("C:\\tools\\adb.exe".to_string())
        );
    }

    #[test]
    fn legacy_adb_path_is_exposed_as_sdk_root() {
        let mut config = crate::config::types::RemoteDesktopConfig::default();
        config.adb_path = Some("/home/user/Android/Sdk/platform-tools/adb".into());
        assert_eq!(
            configured_paths(&config).android_sdk_path.as_deref(),
            Some("/home/user/Android/Sdk")
        );
    }

    #[test]
    fn web_paths_serialize_camel_case() {
        let response = WebScreenPaths {
            sdk: WebPathInfo::placeholder(),
            agent_dir: WebPathInfo {
                value: Some("C:\\Android\\platform-tools\\adb.exe".to_string()),
                origin: ctl::PathOrigin::Env,
                valid: true,
                detail: String::new(),
            },
            configured: WebConfiguredPaths {
                android_sdk_path: Some("C:\\Android\\Sdk".to_string()),
                android_studio_path: Some("C:\\Android Studio".to_string()),
            },
            error: None,
            push_error: None,
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["sdk"]["valid"], false);
        assert_eq!(json["agentDir"]["origin"], "env");
        assert_eq!(json["configured"]["androidSdkPath"], "C:\\Android\\Sdk");
        assert_eq!(
            json["configured"]["androidStudioPath"],
            "C:\\Android Studio"
        );
        assert!(json.get("pushError").is_some());
    }
}
