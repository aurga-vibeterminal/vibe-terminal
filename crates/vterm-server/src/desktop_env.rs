//! Recovery of graphical-session environment variables when the server was
//! started before the user's graphical login (systemd lingering autostart).
//!
//! GNOME/KDE import `DISPLAY` & friends into the systemd *user manager*
//! environment at login (`dbus-update-activation-environment --systemd` /
//! `systemctl --user import-environment`), but services already running
//! under that manager keep the environment they started with. Querying
//! `systemctl --user show-environment` picks the values up afterwards, so
//! the tray icon can appear and spawned terminal sessions can launch GUI
//! applications.

#![cfg(target_os = "linux")]

use std::collections::HashMap;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

/// Set when the graphical variables in this process' environment came from
/// `import_desktop_environment` rather than from the process' real parent.
/// Imported values go stale across a logout→login cycle (e.g. GNOME
/// regenerates the XAUTHORITY file per login), so spawned sessions keep
/// re-reading the user manager environment in that case.
static ENV_WAS_IMPORTED: AtomicBool = AtomicBool::new(false);

/// Variables a graphical session exports into the user manager environment
/// at login. `XDG_RUNTIME_DIR` is intentionally absent: pam_systemd sets it
/// for the user manager itself, so even a boot-time (lingering) service
/// already has it.
const GRAPHICAL_ENV_VARS: &[&str] = &[
    "DISPLAY",
    "WAYLAND_DISPLAY",
    "XAUTHORITY",
    "DBUS_SESSION_BUS_ADDRESS",
    "XDG_CURRENT_DESKTOP",
    "XDG_SESSION_DESKTOP",
    "XDG_SESSION_TYPE",
    "DESKTOP_SESSION",
];

pub(crate) fn desktop_env_available() -> bool {
    std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some()
}

/// One-shot import of graphical-session variables into this process' own
/// environment (needed by the tray icon, which reads the session-bus and
/// display variables from the process environment). Returns true once a
/// display is known — either it was already present, or values have just
/// been imported.
pub(crate) fn import_desktop_environment() -> bool {
    if desktop_env_available() {
        return true;
    }
    let Some(manager_env) = systemd_user_environment() else {
        return false;
    };
    if !manager_env.contains_key("DISPLAY") && !manager_env.contains_key("WAYLAND_DISPLAY") {
        return false;
    }
    for &key in GRAPHICAL_ENV_VARS {
        if std::env::var_os(key).is_some() {
            continue;
        }
        if let Some(value) = manager_env.get(key) {
            std::env::set_var(key, value);
        }
    }
    ENV_WAS_IMPORTED.store(true, Ordering::Relaxed);
    true
}

/// Merge graphical-session variables into a child process environment map
/// (terminal session spawns). Covers servers started before the graphical
/// login without touching this process' own environment: the PTY child
/// otherwise inherits a display-less environment and GUI applications
/// (e.g. `code`) cannot start.
pub(crate) fn apply_graphical_env(env: &mut HashMap<String, String>) {
    if env.contains_key("DISPLAY") || env.contains_key("WAYLAND_DISPLAY") {
        return;
    }
    // A display inherited from the real parent process is authoritative;
    // imported values may be stale (new login) and are refreshed below.
    if desktop_env_available() && !ENV_WAS_IMPORTED.load(Ordering::Relaxed) {
        return;
    }
    let Some(manager_env) = systemd_user_environment() else {
        return;
    };
    if !manager_env.contains_key("DISPLAY") && !manager_env.contains_key("WAYLAND_DISPLAY") {
        return;
    }
    for &key in GRAPHICAL_ENV_VARS {
        if env.contains_key(key) {
            continue;
        }
        if let Some(value) = manager_env.get(key) {
            env.insert(key.to_string(), value.clone());
        }
    }
}

fn systemd_user_environment() -> Option<HashMap<String, String>> {
    let output = Command::new("systemctl")
        .args(["--user", "show-environment"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let map = stdout
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            Some((key.to_string(), value.to_string()))
        })
        .collect();
    Some(map)
}
