use anyhow::Context;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Shell::{
            Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
        },
        WindowsAndMessaging::{
            AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
            DestroyMenu, DispatchMessageW, GetCursorPos, KillTimer, LoadImageW, PostQuitMessage,
            RegisterClassW, RegisterWindowMessageW, SetForegroundWindow, SetTimer, TrackPopupMenu,
            TranslateMessage, CS_DBLCLKS, CW_USEDEFAULT, HICON, IMAGE_ICON, LR_DEFAULTSIZE,
            LR_LOADFROMFILE, MF_GRAYED, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG, PM_REMOVE,
            TPM_BOTTOMALIGN, TPM_LEFTALIGN, WM_COMMAND, WM_DESTROY, WM_ENDSESSION,
            WM_LBUTTONDBLCLK, WM_QUERYENDSESSION, WM_RBUTTONUP, WM_USER, WNDCLASSW, WS_OVERLAPPED,
        },
    },
};

use super::open_client::open_client;
use super::{open_settings, TrayAction, TrayVoiceState, TRAY_TOOLTIP};

const TRAY_CLASS: &str = "vterm_server_tray_icon";
const WM_TRAY_ICON: u32 = WM_USER + 42;
const MENU_SETTINGS: usize = 1;
const MENU_EXIT: usize = 2;
const MENU_VOICE_INPUT: usize = 3;
const MENU_OPEN: usize = 4;
const MENU_HTTP_BASE: usize = 10_000;
const TRAY_ID: u32 = 1;
const TRAY_RETRY_TIMER_ID: usize = 1;
const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(1);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(30);

static mut CONFIG_PORT: u16 = 0;
static mut VOICE_STATE: Option<Arc<std::sync::RwLock<TrayVoiceState>>> = None;
static mut ACTION_TX: Option<std::sync::mpsc::Sender<TrayAction>> = None;
static TASKBAR_CREATED_MESSAGE: AtomicU32 = AtomicU32::new(0);
static TASKBAR_CREATED_PENDING: AtomicBool = AtomicBool::new(false);

#[cfg(DEVELOPMENT)]
const APP_ICO: &[u8] = include_bytes!("../assets/app_dev.ico");
#[cfg(not(DEVELOPMENT))]
const APP_ICO: &[u8] = include_bytes!("../assets/app.ico");

pub fn has_desktop() -> bool {
    true
}

pub struct TrayIcon {
    hwnd: HWND,
    hicon: HICON,
    icon_path: Option<std::path::PathBuf>,
    registered: bool,
    next_register_attempt: Instant,
    register_retry_delay: Duration,
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

        let class_name = wide_null(TRAY_CLASS);
        let hinstance = unsafe { GetModuleHandleW(std::ptr::null()) };
        let wnd_class = WNDCLASSW {
            style: CS_DBLCLKS,
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance,
            lpszClassName: class_name.as_ptr(),
            ..unsafe { std::mem::zeroed() }
        };

        unsafe {
            RegisterClassW(&wnd_class);
        }

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                std::ptr::null(),
                WS_OVERLAPPED,
                CW_USEDEFAULT,
                0,
                CW_USEDEFAULT,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null(),
            )
        };
        if hwnd.is_null() {
            anyhow::bail!(
                "Failed to create tray message window: {}",
                std::io::Error::last_os_error()
            );
        }

        let taskbar_created_name = wide_null("TaskbarCreated");
        let taskbar_created_message = unsafe { RegisterWindowMessageW(taskbar_created_name.as_ptr()) };
        if taskbar_created_message == 0 {
            tracing::warn!(
                "Failed to register TaskbarCreated notification: {}",
                std::io::Error::last_os_error()
            );
        } else {
            TASKBAR_CREATED_MESSAGE.store(taskbar_created_message, Ordering::Relaxed);
        }

        let (hicon, icon_path) = load_tray_icon().context("Failed to load tray icon")?;
        let mut tray_icon = Self {
            hwnd,
            hicon,
            icon_path,
            registered: false,
            next_register_attempt: Instant::now(),
            register_retry_delay: INITIAL_RETRY_DELAY,
        };
        tray_icon.try_register();
        Ok(tray_icon)
    }

    pub fn poll(&mut self) {
        if TASKBAR_CREATED_PENDING.swap(false, Ordering::Relaxed) {
            tracing::info!("Windows taskbar was recreated; restoring tray icon");
            self.registered = false;
            self.next_register_attempt = Instant::now();
        }

        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while windows_sys::Win32::UI::WindowsAndMessaging::PeekMessageW(
                &mut msg, self.hwnd, 0, 0, PM_REMOVE,
            ) != 0
            {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        if !self.registered {
            self.try_register();
        }
    }

    fn try_register(&mut self) {
        let now = Instant::now();
        if now < self.next_register_attempt {
            return;
        }

        let mut nid = notify_icon_data(self.hwnd, self.hicon);
        if unsafe { Shell_NotifyIconW(NIM_ADD, &mut nid) } != 0 {
            self.registered = true;
            self.register_retry_delay = INITIAL_RETRY_DELAY;
            unsafe {
                KillTimer(self.hwnd, TRAY_RETRY_TIMER_ID);
            }
            tracing::info!("Windows tray icon registered");
            return;
        }

        let error = std::io::Error::last_os_error();
        let retry_delay = self.register_retry_delay;
        tracing::warn!(
            "Failed to register tray icon; retrying in {} seconds: {}",
            retry_delay.as_secs(),
            error
        );
        self.next_register_attempt = now + retry_delay;
        self.register_retry_delay = (retry_delay * 2).min(MAX_RETRY_DELAY);
        unsafe {
            if SetTimer(
                self.hwnd,
                TRAY_RETRY_TIMER_ID,
                retry_delay.as_millis() as u32,
                None,
            ) == 0
            {
                tracing::warn!(
                    "Failed to schedule tray icon retry: {}",
                    std::io::Error::last_os_error()
                );
            }
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            KillTimer(self.hwnd, TRAY_RETRY_TIMER_ID);
            if self.registered {
                let mut nid = notify_icon_data(self.hwnd, self.hicon);
                Shell_NotifyIconW(NIM_DELETE, &mut nid);
            }
            if !self.hicon.is_null() {
                DestroyIcon(self.hicon);
            }
            if let Some(path) = &self.icon_path {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let taskbar_created_message = TASKBAR_CREATED_MESSAGE.load(Ordering::Relaxed);
    if taskbar_created_message != 0 && msg == taskbar_created_message {
        TASKBAR_CREATED_PENDING.store(true, Ordering::Relaxed);
        return 0;
    }

    match msg {
        WM_TRAY_ICON => match lparam as u32 {
            WM_LBUTTONDBLCLK => {
                open_client();
                0
            }
            WM_RBUTTONUP => {
                show_menu(hwnd);
                0
            }
            _ => 0,
        },
        WM_COMMAND => {
            let cmd = wparam & 0xffff;
            match cmd {
                MENU_OPEN => open_client(),
                MENU_SETTINGS => open_settings(CONFIG_PORT),
                MENU_VOICE_INPUT => {
                    if let Some(ref tx) = ACTION_TX {
                        // Dispatch on process running state, not model state.
                        // The tray menu item is "Start Vibe Voice" when the
                        // process is down and "Stop Vibe Voice" when it's up.
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
                }
                MENU_EXIT => {
                    tracing::info!("Tray exit requested; shutting down server");
                    PostQuitMessage(0);
                    std::process::exit(0);
                }
                id if id >= MENU_HTTP_BASE => {
                    if let Some(ref tx) = ACTION_TX {
                        let index = id - MENU_HTTP_BASE;
                        if let Some(service) = crate::static_http::hosted_services()
                            .read()
                            .ok()
                            .and_then(|services| services.get(index).cloned())
                        {
                            let _ = tx.send(TrayAction::StopStaticHttp { root: service.root });
                        }
                    }
                }
                _ => {}
            }
            0
        }
        WM_QUERYENDSESSION => {
            tracing::info!(
                "Windows logoff/shutdown requested; keeping the server available while it can still be cancelled"
            );
            1
        }
        WM_ENDSESSION => {
            tracing::info!(
                ending = wparam != 0,
                "Windows session-end notification; a cancelled session leaves the server running"
            );
            0
        }
        WM_DESTROY => 0,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    if menu.is_null() {
        return;
    }

    let voice = match VOICE_STATE.as_ref() {
        Some(s) => match s.read() {
            Ok(guard) => guard.clone(),
            Err(_) => TrayVoiceState::default(),
        },
        None => TrayVoiceState::default(),
    };

    // Open — focus a running client or launch one.
    let open = wide_null("Open");
    AppendMenuW(menu, MF_STRING, MENU_OPEN, open.as_ptr());

    // Settings
    let settings = wide_null("Settings");
    AppendMenuW(menu, MF_STRING, MENU_SETTINGS, settings.as_ptr());

    if voice.voice_enabled {
        // Start/Stop Vibe Voice — always shown. The supervised vterm-voice
        // process is independent of the Enable Voice Recognition toggle, and
        // the user should be able to start/stop it from the tray without
        // opening the WebUI. The label adapts to running state via
        // VoiceInputMenuItem::from_state(is_local, service_state).
        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
        let vi = super::VoiceInputMenuItem::from_state(voice.is_local, &voice.service_state);
        let label = wide_null(&vi.label);
        let flags = if vi.disabled {
            MF_STRING | MF_GRAYED
        } else {
            MF_STRING
        };
        AppendMenuW(menu, flags, MENU_VOICE_INPUT, label.as_ptr());
    }

    let services = crate::static_http::hosted_services()
        .read()
        .map(|services| services.clone())
        .unwrap_or_default();
    if !services.is_empty() {
        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
        let http_menu = CreatePopupMenu();
        for (index, service) in services.iter().enumerate() {
            let label = wide_null(&format!("Stop {} ({})", service.url, service.root));
            AppendMenuW(http_menu, MF_STRING, MENU_HTTP_BASE + index, label.as_ptr());
        }
        let http_label = wide_null("HTTP Servers");
        AppendMenuW(
            menu,
            MF_STRING | MF_POPUP,
            http_menu as usize,
            http_label.as_ptr(),
        );
    }

    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    let exit = wide_null("Exit");
    AppendMenuW(menu, MF_STRING, MENU_EXIT, exit.as_ptr());

    let mut cursor = POINT { x: 0, y: 0 };
    GetCursorPos(&mut cursor);
    SetForegroundWindow(hwnd);
    TrackPopupMenu(
        menu,
        TPM_LEFTALIGN | TPM_BOTTOMALIGN,
        cursor.x,
        cursor.y,
        0,
        hwnd,
        std::ptr::null(),
    );
    DestroyMenu(menu);
}

fn notify_icon_data(hwnd: HWND, hicon: HICON) -> NOTIFYICONDATAW {
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAY_ICON,
        hIcon: hicon,
        ..unsafe { std::mem::zeroed() }
    };

    let tooltip = wide_null(TRAY_TOOLTIP);
    for (idx, ch) in tooltip.iter().copied().take(nid.szTip.len()).enumerate() {
        nid.szTip[idx] = ch;
    }

    nid
}

fn load_tray_icon() -> anyhow::Result<(HICON, Option<std::path::PathBuf>)> {
    let path = std::env::temp_dir().join(format!("vterm-server-tray-{}.ico", std::process::id()));
    std::fs::write(&path, APP_ICO)?;
    let wide_path = wide_null(path.as_os_str());
    let hicon = unsafe {
        LoadImageW(
            std::ptr::null_mut(),
            wide_path.as_ptr(),
            IMAGE_ICON,
            0,
            0,
            LR_LOADFROMFILE | LR_DEFAULTSIZE,
        ) as HICON
    };
    if hicon.is_null() {
        anyhow::bail!("LoadImageW failed: {}", std::io::Error::last_os_error());
    }
    Ok((hicon, Some(path)))
}

fn wide_null(value: impl AsRef<OsStr>) -> Vec<u16> {
    value
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
