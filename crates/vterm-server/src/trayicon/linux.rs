use std::sync::Arc;

use ksni::blocking::TrayMethods;
use ksni::menu::{MenuItem, StandardItem, SubMenu};
use ksni::{Icon, ToolTip, Tray};

use super::open_client::open_client;
use super::{open_settings, TrayAction, TrayVoiceState, TRAY_TOOLTIP};

#[cfg(DEVELOPMENT)]
const APP_PNG: &[u8] = include_bytes!("../assets/app_dev.png");
#[cfg(not(DEVELOPMENT))]
const APP_PNG: &[u8] = include_bytes!("../assets/app.png");

static mut CONFIG_PORT: u16 = 0;
static mut VOICE_STATE: Option<Arc<std::sync::RwLock<TrayVoiceState>>> = None;
static mut ACTION_TX: Option<std::sync::mpsc::Sender<TrayAction>> = None;

pub fn has_desktop() -> bool {
    std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
}

struct VTermTray {
    icon: Vec<Icon>,
}

impl Tray for VTermTray {
    fn id(&self) -> String {
        "vterm-server".into()
    }

    fn title(&self) -> String {
        TRAY_TOOLTIP.into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        self.icon.clone()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            icon_name: String::new(),
            icon_pixmap: self.icon.clone(),
            title: TRAY_TOOLTIP.into(),
            description: String::new(),
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        open_client();
    }

    /// The watcher (GNOME's appindicator extension) can claim its bus name
    /// after this service starts — graphical-session.target only orders
    /// against session activation, not the shell's tray host. Keep waiting
    /// and re-register when it appears instead of losing the icon for the
    /// process lifetime.
    fn watcher_offline(&self, reason: ksni::OfflineReason) -> bool {
        tracing::info!(
            "StatusNotifierWatcher offline ({reason:?}); will re-register when it appears"
        );
        true
    }

    fn watcher_online(&self) {
        tracing::info!("StatusNotifierWatcher online; tray icon registered");
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut items: Vec<MenuItem<Self>> = Vec::new();

        let voice = unsafe {
            VOICE_STATE
                .as_ref()
                .and_then(|s| s.read().ok())
                .map(|g| g.clone())
                .unwrap_or_else(|| TrayVoiceState::default())
        };

        // Open — focus a running client or launch one.
        items.push(
            StandardItem {
                label: "Open".into(),
                activate: Box::new(|_| {
                    open_client();
                }),
                ..Default::default()
            }
            .into(),
        );

        // Settings
        items.push(
            StandardItem {
                label: "Settings".into(),
                activate: Box::new(|_| {
                    unsafe { open_settings(CONFIG_PORT) };
                }),
                ..Default::default()
            }
            .into(),
        );

        if voice.voice_enabled {
            // Start/Stop Vibe Voice — always shown. Dispatch on process
            // running state (is_local), not model load state.
            items.push(MenuItem::Separator);
            let vi = super::VoiceInputMenuItem::from_state(voice.is_local, &voice.service_state);
            let vi_disabled = vi.disabled;
            items.push(
                StandardItem {
                    label: vi.label,
                    enabled: !vi_disabled,
                    activate: Box::new(|_| unsafe {
                        if let Some(ref tx) = ACTION_TX {
                            let running = VOICE_STATE
                                .as_ref()
                                .and_then(|s| s.read().ok())
                                .map(|g| g.is_local)
                                .unwrap_or(false);
                            if running {
                                let _ = tx.send(TrayAction::StopVoice);
                            } else {
                                let _ = tx.send(TrayAction::StartVoice);
                            }
                        }
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        let services = crate::static_http::hosted_services()
            .read()
            .map(|services| services.clone())
            .unwrap_or_default();
        if !services.is_empty() {
            // Stop entries for currently hosted static HTTP directories,
            // mirroring the Windows tray's HTTP Servers submenu.
            let submenu = services
                .into_iter()
                .map(|service| {
                    let root = service.root.clone();
                    StandardItem {
                        label: format!("Stop {} ({})", service.url, service.root),
                        activate: Box::new(move |_| unsafe {
                            if let Some(ref tx) = ACTION_TX {
                                let _ = tx.send(TrayAction::StopStaticHttp { root: root.clone() });
                            }
                        }),
                        ..Default::default()
                    }
                    .into()
                })
                .collect();
            items.push(MenuItem::Separator);
            items.push(
                SubMenu {
                    label: "HTTP Servers".into(),
                    submenu,
                    ..Default::default()
                }
                .into(),
            );
        }

        items.push(MenuItem::Separator);
        items.push(
            StandardItem {
                label: "Exit".into(),
                activate: Box::new(|_| {
                    tracing::info!("Tray exit requested; shutting down server");
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

pub struct TrayIcon {
    _handle: ksni::blocking::Handle<VTermTray>,
}

impl TrayIcon {
    pub fn new(
        config_port: u16,
        voice_state: Arc<std::sync::RwLock<TrayVoiceState>>,
        action_tx: std::sync::mpsc::Sender<TrayAction>,
    ) -> anyhow::Result<Self> {
        unsafe {
            CONFIG_PORT = config_port;
            VOICE_STATE = Some(voice_state);
            ACTION_TX = Some(action_tx);
        }

        let icon = decode_png_to_argb32(APP_PNG)?;
        // assume_sni_available: spawn must not fail outright when the
        // StatusNotifierWatcher has not claimed its name yet (the unit starts
        // with graphical-session.target, potentially before the shell's tray
        // host); watcher_offline/online handle the late registration.
        let handle = VTermTray { icon }
            .assume_sni_available(true)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to create tray icon: {e}"))?;

        // Periodically refresh the D-Bus menu so the tray shows current state.
        let h = handle.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            h.update(|_| {});
        });

        Ok(Self { _handle: handle })
    }

    pub fn poll(&mut self) {}
}

fn decode_png_to_argb32(data: &[u8]) -> anyhow::Result<Vec<Icon>> {
    let decoder = png::Decoder::new(data);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let output_info = reader.next_frame(&mut buf)?;

    let w = output_info.width;
    let h = output_info.height;

    let rgba = match output_info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255u8])
            .collect(),
        other => anyhow::bail!("Unsupported PNG color type: {other:?}"),
    };

    Ok(build_icon_variants(&rgba, w, h))
}

/// One pixmap per common panel size, largest first.
///
/// Some tray hosts do not scale pixmaps down: the ubuntu-appindicators
/// extension shipped with GNOME 3.28 (Ubuntu 18.04) *disqualifies* any
/// pixmap larger than the requested panel icon size (16–24 px), so a
/// single 256×256 pixmap leaves the tray blank there — the menu still
/// works because it travels over the separate DBusMenu channel. Providing
/// the full size ladder keeps old hosts rendering and lets modern hosts
/// (which scale or pick the nearest size) behave exactly as before.
fn build_icon_variants(rgba: &[u8], w: u32, h: u32) -> Vec<Icon> {
    const SIZES: [u32; 8] = [256, 128, 64, 48, 32, 24, 22, 16];
    let min_dim = w.min(h);
    let mut icons = Vec::with_capacity(SIZES.len());
    for &size in &SIZES {
        if size > min_dim {
            continue;
        }
        let pixels = if size == w && size == h {
            rgba.to_vec()
        } else {
            resize_rgba_premultiplied(rgba, w, h, size, size)
        };
        icons.push(to_argb_icon(&pixels, size, size));
    }
    icons
}

fn to_argb_icon(rgba: &[u8], w: u32, h: u32) -> Icon {
    Icon {
        width: w as i32,
        height: h as i32,
        data: rgba
            .chunks_exact(4)
            .flat_map(|p| [p[3], p[0], p[1], p[2]])
            .collect(),
    }
}

/// Area-average (box filter) resample with premultiplied alpha, so
/// antialiased edges stay clean instead of picking up background fringe.
fn resize_rgba_premultiplied(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let premul: Vec<u8> = src
        .chunks_exact(4)
        .flat_map(|p| {
            let a = p[3] as u32;
            [
                (p[0] as u32 * a / 255) as u8,
                (p[1] as u32 * a / 255) as u8,
                (p[2] as u32 * a / 255) as u8,
                p[3],
            ]
        })
        .collect();

    let mut out = Vec::with_capacity((dw * dh * 4) as usize);
    for dy in 0..dh {
        let y0 = dy as f64 * sh as f64 / dh as f64;
        let y1 = (dy + 1) as f64 * sh as f64 / dh as f64;
        for dx in 0..dw {
            let x0 = dx as f64 * sw as f64 / dw as f64;
            let x1 = (dx + 1) as f64 * sw as f64 / dw as f64;
            let (mut ar, mut ag, mut ab, mut aa, mut wt) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
            let mut y = y0.floor() as u32;
            while (y as f64) < y1 && y < sh {
                let vy = ((y as f64 + 1.0).min(y1)) - (y as f64).max(y0);
                if vy > 0.0 {
                    let mut x = x0.floor() as u32;
                    while (x as f64) < x1 && x < sw {
                        let vx = ((x as f64 + 1.0).min(x1)) - (x as f64).max(x0);
                        if vx > 0.0 {
                            let wgt = vx * vy;
                            let i = ((y * sw + x) * 4) as usize;
                            ar += premul[i] as f64 * wgt;
                            ag += premul[i + 1] as f64 * wgt;
                            ab += premul[i + 2] as f64 * wgt;
                            aa += premul[i + 3] as f64 * wgt;
                            wt += wgt;
                        }
                        x += 1;
                    }
                }
                y += 1;
            }

            if wt > 0.0 {
                let a = (aa / wt).round().min(255.0) as u32;
                if a == 0 {
                    // Alpha rounds down to zero: fully transparent — nothing
                    // to unpremultiply (and dividing below would panic).
                    out.extend_from_slice(&[0, 0, 0, 0]);
                } else {
                    // Unpremultiply back to straight alpha.
                    let ur = ((ar / wt).round() as u32 * 255 / a).min(255) as u8;
                    let ug = ((ag / wt).round() as u32 * 255 / a).min(255) as u8;
                    let ub = ((ab / wt).round() as u32 * 255 / a).min(255) as u8;
                    out.extend_from_slice(&[ur, ug, ub, a as u8]);
                }
            } else {
                out.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    out
}
