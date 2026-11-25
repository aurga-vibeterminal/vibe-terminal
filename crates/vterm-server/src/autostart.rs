//! Runtime management of "start vterm-server automatically at login".
//!
//! Linux: a systemd user service. The install target depends on whether a
//! display manager (gdm/sddm/lightdm/…) is installed: with one, the unit is
//! bound to `graphical-session.target` so the server starts together with
//! the desktop session and inherits its environment (DISPLAY,
//! WAYLAND_DISPLAY, XDG_RUNTIME_DIR — tray icon and screen capture work);
//! without one (headless), it keeps `WantedBy=default.target` so the server
//! starts at boot before any login. Enabling (re)writes the per-user
//! override unit at ~/.config/systemd/user/vterm-server.service so the
//! target tracks display-manager presence, and `systemctl reenable`
//! refreshes the wants symlinks. Enabling also attempts
//! `loginctl enable-linger` (best effort) so the user manager is alive at
//! boot where the system's polkit policy allows it. When no systemd user
//! bus is reachable but a desktop session exists, an XDG autostart
//! `.desktop` entry is used instead.
//!
//! Windows: a per-user `HKCU\...\CurrentVersion\Run` registry value (no
//! administrator), started at the next interactive login. Dev builds use a
//! separate value name so they never overwrite the production entry.
//!
//! macOS: on macOS 13+ in a packaged app bundle, an `SMAppService` login
//! item registered from the launchd plist embedded at
//! `Contents/Library/LaunchAgents/<label>.plist` inside the server app. Only
//! items registered this way appear in System Settings → Login Items with
//! the app name and icon; a legacy agent falls back to the signature's
//! developer name and a blank icon. The fallback (macOS 12, dev builds) is a
//! per-user LaunchAgent (`~/Library/LaunchAgents/<label>.plist`, no
//! administrator), loaded with `launchctl bootstrap` so the server also
//! starts immediately, not only at the next login. `KeepAlive` with
//! `SuccessfulExit=false` mirrors the Linux unit's `Restart=on-failure`.
//! Dev builds use a separate label so they never overwrite the production
//! agent.

#[cfg(target_os = "linux")]
mod imp {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use tracing::warn;

    const SERVICE_NAME: &str = "vterm-server.service";
    const DESKTOP_FILE_NAME: &str = "vterm-server.desktop";

    pub(crate) fn is_supported() -> bool {
        systemd_user_available() || xdg_autostart_supported()
    }

    pub(crate) fn set_enabled(enabled: bool) -> Result<(), String> {
        if systemd_user_available() {
            return set_systemd_enabled(enabled);
        }
        if xdg_autostart_supported() {
            return set_xdg_enabled(enabled);
        }
        Err(
            "neither a systemd user session nor a desktop autostart location is available"
                .to_string(),
        )
    }

    fn systemd_user_available() -> bool {
        Command::new("systemctl")
            .args(["--user", "status"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn set_systemd_enabled(enabled: bool) -> Result<(), String> {
        if !enabled {
            return run_systemctl(&["disable", "--now", SERVICE_NAME]);
        }
        let desired = unit_file_contents()?;
        if ensure_user_unit(&desired)? {
            run_systemctl(&["daemon-reload"])?;
        }
        // reenable refreshes the wants symlinks when the install target
        // changed (default.target ↔ graphical-session.target); unlike
        // `disable --now` it does not stop a running unit.
        run_systemctl(&["reenable", SERVICE_NAME])?;
        run_systemctl(&["start", SERVICE_NAME])?;
        enable_linger_best_effort();
        Ok(())
    }

    /// Writes the per-user override unit when its content differs from the
    /// desired one; returns true when the file was (re)written, meaning a
    /// daemon-reload is needed. The override shadows the packaged
    /// /usr/lib/systemd/user unit, whose install target (default.target) is
    /// headless-oriented.
    fn ensure_user_unit(desired: &str) -> Result<bool, String> {
        let path = config_home()?.join("systemd/user").join(SERVICE_NAME);
        if std::fs::read_to_string(&path)
            .map(|existing| existing == desired)
            .unwrap_or(false)
        {
            return Ok(false);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {parent:?}: {e}"))?;
        }
        std::fs::write(&path, desired).map_err(|e| format!("cannot write {path:?}: {e}"))?;
        Ok(true)
    }

    fn unit_file_contents() -> Result<String, String> {
        Ok(render_unit(
            &current_exe_string()?,
            display_manager_installed(),
        ))
    }

    /// `with_desktop_session` binds the unit to the desktop session so the
    /// server inherits its environment; otherwise it starts at boot
    /// (default.target + lingering) for headless hosts.
    fn render_unit(exe: &str, with_desktop_session: bool) -> String {
        let (after, part_of, wanted_by) = if with_desktop_session {
            (
                "graphical-session.target",
                "PartOf=graphical-session.target\n",
                "graphical-session.target",
            )
        } else {
            ("network.target", "", "default.target")
        };
        format!(
            "[Unit]\n\
             Description=Vibe Terminal Server\n\
             After={after}\n\
             {part_of}\
             \n\
             [Service]\n\
             ExecStart=\"{exe}\"\n\
             Restart=on-failure\n\
             RestartSec=3\n\
             \n\
             [Install]\n\
             WantedBy={wanted_by}\n"
        )
    }

    /// True when a display manager is installed, i.e. the machine can host a
    /// graphical login session. Desktops without a display manager (plain
    /// startx) deliberately take the headless path: graphical-session.target
    /// would never activate there and the server would never start.
    fn display_manager_installed() -> bool {
        const UNIT_DIRS: [&str; 3] = [
            "/etc/systemd/system",
            "/usr/lib/systemd/system",
            "/lib/systemd/system",
        ];
        // display-manager.service is the canonical alias a DM registers when
        // enabled; the per-DM unit names cover installed-but-disabled DMs.
        const DM_UNITS: [&str; 7] = [
            "display-manager.service",
            "gdm.service",
            "gdm3.service",
            "sddm.service",
            "lightdm.service",
            "lxdm.service",
            "xdm.service",
        ];
        UNIT_DIRS.iter().any(|dir| {
            DM_UNITS
                .iter()
                .any(|unit| Path::new(dir).join(unit).exists())
        })
    }

    /// Boot-before-login: lingering keeps the user manager alive without an
    /// interactive session. Polkit may deny this for non-admins — that only
    /// means autostart waits for the first login, so failure is not an error.
    fn enable_linger_best_effort() {
        let mut command = Command::new("loginctl");
        command.arg("enable-linger");
        if let Ok(user) = std::env::var("USER") {
            if !user.is_empty() {
                command.arg(user);
            }
        }
        match command.output() {
            Ok(output) if output.status.success() => {}
            Ok(output) => warn!(
                "loginctl enable-linger failed (starts on first login instead): {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(e) => warn!("loginctl not available: {e}"),
        }
    }

    fn run_systemctl(args: &[&str]) -> Result<(), String> {
        let output = Command::new("systemctl")
            .arg("--user")
            .args(args)
            .output()
            .map_err(|e| format!("cannot run systemctl: {e}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "systemctl --user {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }

    fn xdg_autostart_supported() -> bool {
        std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some()
    }

    fn set_xdg_enabled(enabled: bool) -> Result<(), String> {
        let path = config_home()?.join("autostart").join(DESKTOP_FILE_NAME);
        if !enabled {
            match std::fs::remove_file(&path) {
                Ok(()) => return Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(e) => return Err(format!("cannot remove {path:?}: {e}")),
            }
        }
        let exe = current_exe_string()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {parent:?}: {e}"))?;
        }
        let entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Vibe Terminal Server\n\
             Exec=\"{exe}\"\n\
             X-GNOME-Autostart-enabled=true\n"
        );
        std::fs::write(&path, entry).map_err(|e| format!("cannot write {path:?}: {e}"))
    }

    fn current_exe_string() -> Result<String, String> {
        std::env::current_exe()
            .map(|path| path.display().to_string())
            .map_err(|e| format!("cannot resolve the vterm-server binary path: {e}"))
    }

    fn config_home() -> Result<PathBuf, String> {
        if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
            if !dir.is_empty() {
                return Ok(PathBuf::from(dir));
            }
        }
        std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(".config"))
            .ok_or_else(|| "HOME is not set".to_string())
    }

    #[cfg(test)]
    mod tests {
        use super::render_unit;

        #[test]
        fn headless_unit_starts_at_boot_before_login() {
            let unit = render_unit("/usr/bin/vterm-server", false);
            assert!(unit.contains("WantedBy=default.target"));
            assert!(!unit.contains("graphical-session.target"));
        }

        #[test]
        fn desktop_unit_waits_for_the_graphical_session() {
            let unit = render_unit("/usr/bin/vterm-server", true);
            assert!(unit.contains("After=graphical-session.target"));
            assert!(unit.contains("PartOf=graphical-session.target"));
            assert!(unit.contains("WantedBy=graphical-session.target"));
            assert!(!unit.contains("default.target"));
        }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_SET_VALUE, REG_SZ,
    };

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

    /// Run-key value name for this build flavor (matches the dev/prod split
    /// used by the desktop client's context-menu registration).
    fn value_name() -> &'static str {
        if cfg!(DEVELOPMENT) {
            "VibeTerminalServerDev"
        } else {
            "VibeTerminalServer"
        }
    }

    pub(crate) fn is_supported() -> bool {
        true
    }

    pub(crate) fn set_enabled(enabled: bool) -> Result<(), String> {
        let key = open_run_key()?;
        let result = if enabled {
            write_run_value(key)
        } else {
            delete_run_value(key)
        };
        unsafe { RegCloseKey(key) };
        result
    }

    fn open_run_key() -> Result<HKEY, String> {
        let subkey = wide_z(RUN_KEY);
        let mut handle: HKEY = std::ptr::null_mut();
        let result = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut handle,
            )
        };
        if result == 0 {
            Ok(handle)
        } else {
            Err(format!("RegOpenKeyExW({RUN_KEY}) failed: {result}"))
        }
    }

    fn write_run_value(key: HKEY) -> Result<(), String> {
        let exe = std::env::current_exe()
            .map_err(|e| format!("cannot resolve the vterm-server binary path: {e}"))?;
        let command = format!("\"{}\"", exe.display());
        let name = wide_z(value_name());
        // REG_SZ data must be a NUL-terminated wide string; the byte length
        // includes the terminating NUL.
        let data = wide_z(&command);
        let result = unsafe {
            RegSetValueExW(
                key,
                name.as_ptr(),
                0,
                REG_SZ,
                data.as_ptr().cast(),
                (data.len() * 2) as u32,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(format!("RegSetValueExW failed: {result}"))
        }
    }

    fn delete_run_value(key: HKEY) -> Result<(), String> {
        let name = wide_z(value_name());
        let result = unsafe { RegDeleteValueW(key, name.as_ptr()) };
        // ERROR_FILE_NOT_FOUND (2): nothing to delete — the desired state.
        if result == 0 || result == 2 {
            Ok(())
        } else {
            Err(format!("RegDeleteValueW failed: {result}"))
        }
    }

    /// NUL-terminated UTF-16 for PCWSTR parameters.
    fn wide_z(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use tracing::warn;

    /// LaunchAgent label and plist base name for this build flavor (matches
    /// the dev/prod split used by the Windows Run-key value name).
    fn label() -> &'static str {
        if cfg!(DEVELOPMENT) {
            "com.aurga.vibeterminal.server-dev"
        } else {
            "com.aurga.vibeterminal.server"
        }
    }

    pub(crate) fn is_supported() -> bool {
        true
    }

    pub(crate) fn set_enabled(enabled: bool) -> Result<(), String> {
        if let Some(plist_name) = bundled_plist_name() {
            // Migrate off the legacy agent so only one registration survives.
            remove_legacy_agent();
            return smappservice_set_enabled(&plist_name, enabled);
        }
        legacy_set_enabled(enabled)
    }

    /// Name of the launchd plist embedded in the running bundle, when this
    /// process runs from a packaged app on macOS 13+ (SMAppService requires
    /// both). Dev builds run unpackaged and take the legacy path.
    fn bundled_plist_name() -> Option<String> {
        if cfg!(DEVELOPMENT) || !smappservice_available() {
            return None;
        }
        // .../AURGA Vibe Terminal.app/Contents/MacOS/vterm-server → Contents.
        let contents = std::env::current_exe()
            .ok()?
            .parent()?
            .parent()?
            .to_path_buf();
        let name = format!("{}.plist", label());
        contents
            .join("Library/LaunchAgents")
            .join(&name)
            .is_file()
            .then_some(name)
    }

    // ── SMAppService (macOS 13+) ──────────────────────────────────────────

    #[link(name = "ServiceManagement", kind = "framework")]
    extern "C" {}

    /// True on macOS 13+, where the SMAppService class exists.
    fn smappservice_available() -> bool {
        objc::runtime::Class::get("SMAppService").is_some()
    }

    /// SMAppServiceStatus values we act on (ServiceManagement.h).
    const STATUS_NOT_REGISTERED: i64 = 0;

    fn smappservice_set_enabled(plist_name: &str, enabled: bool) -> Result<(), String> {
        use objc::runtime::Object;
        use objc::{class, msg_send, sel, sel_impl};

        let c_name = std::ffi::CString::new(plist_name)
            .map_err(|_| format!("invalid plist name: {plist_name:?}"))?;
        unsafe {
            let ns_name: *mut Object =
                msg_send![class!(NSString), stringWithUTF8String: c_name.as_ptr()];
            if ns_name.is_null() {
                return Err("NSString creation failed".to_string());
            }
            let service: *mut Object =
                msg_send![class!(SMAppService), agentServiceWithPlistName: ns_name];
            if service.is_null() {
                return Err("SMAppService agentServiceWithPlistName returned nil".to_string());
            }
            // Unregistering an agent that was never registered fails with a
            // "not registered" error even though the desired state is already
            // reached — treat it as success.
            let status: i64 = msg_send![service, status];
            if !enabled && status == STATUS_NOT_REGISTERED {
                return Ok(());
            }
            let mut error: *mut Object = std::ptr::null_mut();
            let ok: bool = if enabled {
                msg_send![service, registerAndReturnError: &mut error]
            } else {
                msg_send![service, unregisterAndReturnError: &mut error]
            };
            if ok {
                Ok(())
            } else {
                Err(ns_error_description(error))
            }
        }
    }

    unsafe fn ns_error_description(error: *mut objc::runtime::Object) -> String {
        use objc::runtime::Object;
        use objc::{msg_send, sel, sel_impl};

        let fallback = || "SMAppService failed (no error description)".to_string();
        if error.is_null() {
            return fallback();
        }
        let desc: *mut Object = msg_send![error, localizedDescription];
        if desc.is_null() {
            return fallback();
        }
        let utf8: *const std::os::raw::c_char = msg_send![desc, UTF8String];
        if utf8.is_null() {
            return fallback();
        }
        std::ffi::CStr::from_ptr(utf8)
            .to_string_lossy()
            .into_owned()
    }

    // ── Legacy per-user LaunchAgent (macOS 12, dev builds) ───────────────

    fn legacy_set_enabled(enabled: bool) -> Result<(), String> {
        let path = plist_path()?;
        if enabled {
            write_plist(&path)?;
            // Start now, not only at the next login: drop any previously
            // loaded copy of the agent, then bootstrap the fresh plist.
            bootout_best_effort(&path);
            let domain = gui_domain();
            let plist = path_str(&path)?;
            run_launchctl(&["bootstrap", &domain, &plist])
        } else {
            bootout_best_effort(&path);
            match std::fs::remove_file(&path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(format!("cannot remove {path:?}: {e}")),
            }
        }
    }

    /// Best-effort removal of a legacy agent installed by older builds, so
    /// switching to SMAppService never leaves both registrations alive.
    fn remove_legacy_agent() {
        let Ok(path) = plist_path() else {
            return;
        };
        if !path.exists() {
            return;
        }
        bootout_best_effort(&path);
        if let Err(e) = std::fs::remove_file(&path) {
            warn!("cannot remove legacy LaunchAgent {path:?}: {e}");
        }
    }

    fn plist_path() -> Result<PathBuf, String> {
        let home = std::env::var_os("HOME").ok_or_else(|| "HOME is not set".to_string())?;
        Ok(PathBuf::from(home)
            .join("Library/LaunchAgents")
            .join(format!("{}.plist", label())))
    }

    fn write_plist(path: &Path) -> Result<(), String> {
        let exe = std::env::current_exe()
            .map_err(|e| format!("cannot resolve the vterm-server binary path: {e}"))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {parent:?}: {e}"))?;
        }
        let plist = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\">\n\
             <dict>\n\
             \t<key>Label</key>\n\
             \t<string>{label}</string>\n\
             \t<key>ProgramArguments</key>\n\
             \t<array>\n\
             \t\t<string>{exe}</string>\n\
             \t</array>\n\
             \t<key>RunAtLoad</key>\n\
             \t<true/>\n\
             \t<key>KeepAlive</key>\n\
             \t<dict>\n\
             \t\t<key>SuccessfulExit</key>\n\
             \t\t<false/>\n\
             \t</dict>\n\
             </dict>\n\
             </plist>\n",
            label = label(),
            exe = xml_escape(&exe.display().to_string()),
        );
        std::fs::write(path, plist).map_err(|e| format!("cannot write {path:?}: {e}"))
    }

    /// `gui/<uid>` — the per-user launchd domain of the login session.
    fn gui_domain() -> String {
        format!("gui/{}", unsafe { libc::getuid() })
    }

    /// Unload the agent when it is currently loaded. Not being loaded is the
    /// common case (fresh enable, already-disabled disable) and never an
    /// error: the plist on disk alone decides the state at the next login.
    fn bootout_best_effort(path: &Path) {
        let domain = gui_domain();
        let Ok(plist) = path_str(path) else {
            return;
        };
        match Command::new("launchctl")
            .args(["bootout", &domain, &plist])
            .output()
        {
            Ok(output) if output.status.success() => {}
            Ok(output) => warn!(
                "launchctl bootout failed (agent may not be loaded): {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(e) => warn!("launchctl not available: {e}"),
        }
    }

    fn run_launchctl(args: &[&str]) -> Result<(), String> {
        let output = Command::new("launchctl")
            .args(args)
            .output()
            .map_err(|e| format!("cannot run launchctl: {e}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "launchctl {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }

    fn path_str(path: &Path) -> Result<String, String> {
        path.to_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("non-UTF-8 path: {path:?}"))
    }

    /// Minimal escaping for a string placed inside a plist `<string>` node.
    fn xml_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
mod imp {
    pub(crate) fn is_supported() -> bool {
        false
    }

    pub(crate) fn set_enabled(_enabled: bool) -> Result<(), String> {
        Err("auto-start on login is not supported on this platform".to_string())
    }
}

pub(crate) use imp::{is_supported, set_enabled};
