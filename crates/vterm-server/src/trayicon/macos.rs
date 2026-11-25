use std::sync::Arc;

use cocoa::base::{id, nil, YES};
use cocoa::foundation::{NSAutoreleasePool, NSSize, NSString as NSNSString};
use objc::{class, declare::ClassDecl, msg_send, runtime::Class, sel, sel_impl};

use super::open_client::open_client;
use super::{open_settings, TrayAction, TrayVoiceState, TRAY_TOOLTIP};

static mut CONFIG_PORT: u16 = 0;
static mut VOICE_STATE: Option<Arc<std::sync::RwLock<TrayVoiceState>>> = None;
static mut STATUS_ITEM: id = nil;
static mut ACTION_TX: Option<std::sync::mpsc::Sender<TrayAction>> = None;

#[cfg(DEVELOPMENT)]
const APP_PNG: &[u8] = include_bytes!("../assets/app_dev.png");
#[cfg(not(DEVELOPMENT))]
const APP_PNG: &[u8] = include_bytes!("../assets/app.png");
const STATUS_ITEM_LENGTH: f64 = 24.0;

pub fn has_desktop() -> bool {
    true
}

pub struct TrayIcon {
    _status_item: id,
    _target: id,
    _image: id,
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

            let pool = NSAutoreleasePool::new(nil);

            let target = create_target();

            let status_bar: id = msg_send![class!(NSStatusBar), systemStatusBar];
            let status_item: id = msg_send![status_bar, statusItemWithLength: STATUS_ITEM_LENGTH];
            let _: () = msg_send![status_item, retain];
            STATUS_ITEM = status_item;
            let button: id = msg_send![status_item, button];

            let image = create_status_image();
            let title = NSNSString::alloc(nil).init_str("");
            let _: () = msg_send![button, setTitle: title];
            let _: () = msg_send![button, setImage: image];
            let tooltip = NSNSString::alloc(nil).init_str(TRAY_TOOLTIP);
            let _: () = msg_send![button, setToolTip: tooltip];

            let _: () = msg_send![button, setTarget: target];
            let _: () = msg_send![button, setAction: sel!(trayClicked:)];
            let _: () =
                msg_send![button, sendActionOn: left_mouse_down_mask() | right_mouse_down_mask()];

            let _: () = msg_send![pool, drain];
            tracing::info!("macOS tray status item initialized");

            Ok(Self {
                _status_item: status_item,
                _target: target,
                _image: image,
            })
        }
    }

    pub fn poll(&mut self) {}
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            let status_bar: id = msg_send![class!(NSStatusBar), systemStatusBar];
            let _: () = msg_send![status_bar, removeStatusItem: self._status_item];
            let _: () = msg_send![self._image, release];
            let _: () = msg_send![self._target, release];
            let _: () = msg_send![self._status_item, release];
        }
    }
}

unsafe fn build_menu() -> id {
    let pool = NSAutoreleasePool::new(nil);
    let menu: id = msg_send![class!(NSMenu), new];

    let voice = VOICE_STATE
        .as_ref()
        .and_then(|s| s.read().ok())
        .map(|g| g.clone())
        .unwrap_or_default();

    let target: id = msg_send![get_target_class(), new];
    let empty = NSNSString::alloc(nil).init_str("");

    // Open — focus a running client or launch one.
    let open_title = NSNSString::alloc(nil).init_str("Open");
    let open_item: id = msg_send![class!(NSMenuItem), alloc];
    let open_item: id = msg_send![
        open_item,
        initWithTitle: open_title
        action: sel!(openClient:)
        keyEquivalent: empty
    ];
    let _: () = msg_send![open_item, setTarget: target];
    let _: () = msg_send![menu, addItem: open_item];

    // Settings
    let settings_title = NSNSString::alloc(nil).init_str("Settings");
    let settings: id = msg_send![class!(NSMenuItem), alloc];
    let settings: id = msg_send![
        settings,
        initWithTitle: settings_title
        action: sel!(openSettings:)
        keyEquivalent: empty
    ];
    let _: () = msg_send![settings, setTarget: target];
    let _: () = msg_send![menu, addItem: settings];

    if voice.voice_enabled {
        // Start/Stop Vibe Voice — always shown. Dispatch on process running
        // state (is_local), not model load state. The voiceInput: handler
        // reads the live state when invoked, so we don't need to decide here.
        let sep: id = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![menu, addItem: sep];

        let vi = super::VoiceInputMenuItem::from_state(voice.is_local, &voice.service_state);
        let vi_label = NSNSString::alloc(nil).init_str(&vi.label);
        let vi_item: id = msg_send![class!(NSMenuItem), alloc];
        let vi_item: id = msg_send![
            vi_item,
            initWithTitle: vi_label
            action: sel!(voiceInput:)
            keyEquivalent: empty
        ];
        if vi.disabled {
            let _: () = msg_send![vi_item, setEnabled: cocoa::base::NO];
        }
        let _: () = msg_send![vi_item, setTarget: target];
        let _: () = msg_send![menu, addItem: vi_item];
    }

    // HTTP Servers — one Stop item per hosted static HTTP directory,
    // mirroring the Windows tray. The root is stored in each item's
    // representedObject and read back by the stopHttp: handler.
    let services = crate::static_http::hosted_services()
        .read()
        .map(|services| services.clone())
        .unwrap_or_default();
    if !services.is_empty() {
        let sep: id = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![menu, addItem: sep];

        let http_menu: id = msg_send![class!(NSMenu), new];
        for service in &services {
            let title = NSNSString::alloc(nil)
                .init_str(&format!("Stop {} ({})", service.url, service.root));
            let item: id = msg_send![class!(NSMenuItem), alloc];
            let item: id = msg_send![
                item,
                initWithTitle: title
                action: sel!(stopHttp:)
                keyEquivalent: empty
            ];
            let root = NSNSString::alloc(nil).init_str(&service.root);
            let _: () = msg_send![item, setRepresentedObject: root];
            let _: () = msg_send![item, setTarget: target];
            let _: () = msg_send![http_menu, addItem: item];
        }
        let http_title = NSNSString::alloc(nil).init_str("HTTP Servers");
        let http_item: id = msg_send![class!(NSMenuItem), alloc];
        let http_item: id = msg_send![
            http_item,
            initWithTitle: http_title
            action: nil
            keyEquivalent: empty
        ];
        let _: () = msg_send![http_item, setSubmenu: http_menu];
        let _: () = msg_send![menu, addItem: http_item];
    }

    // Exit
    let sep: id = msg_send![class!(NSMenuItem), separatorItem];
    let _: () = msg_send![menu, addItem: sep];
    let exit_title = NSNSString::alloc(nil).init_str("Exit");
    let exit: id = msg_send![class!(NSMenuItem), alloc];
    let exit: id = msg_send![
        exit,
        initWithTitle: exit_title
        action: sel!(exitApp:)
        keyEquivalent: empty
    ];
    let _: () = msg_send![exit, setTarget: target];
    let _: () = msg_send![menu, addItem: exit];

    let _: () = msg_send![pool, drain];
    menu
}

unsafe fn create_status_image() -> id {
    let data: id = msg_send![
        class!(NSData),
        dataWithBytes: APP_PNG.as_ptr()
        length: APP_PNG.len()
    ];

    let bitmap_rep: id = msg_send![class!(NSBitmapImageRep), alloc];
    let bitmap_rep: id = msg_send![bitmap_rep, initWithData: data];

    let image: id = msg_send![class!(NSImage), alloc];
    let image: id = msg_send![image, init];
    let _: () = msg_send![image, addRepresentation: bitmap_rep];
    let _: () = msg_send![image, setTemplate: cocoa::base::NO];
    let _: () = msg_send![image, setSize: NSSize::new(18.0, 18.0)];
    image
}

fn get_target_class() -> &'static Class {
    use objc::runtime::{Object, Sel};
    use std::sync::OnceLock;

    static CLASS: OnceLock<&'static Class> = OnceLock::new();
    CLASS.get_or_init(|| {
        let superclass = Class::get("NSObject").expect("NSObject class missing");
        let mut decl = ClassDecl::new("VTermServerTrayTarget", superclass)
            .expect("VTermServerTrayTarget already registered");

        // A single left click (or right click) on the status item shows the
        // context menu. `sendActionOn` on the button limits this to mouse-down
        // events, so we simply rebuild and pop up the menu on every click.
        extern "C" fn tray_clicked(_this: &Object, _sel: Sel, _sender: id) {
            unsafe {
                let menu = build_menu();
                let _: () = msg_send![STATUS_ITEM, popUpStatusItemMenu: menu];
            }
        }

        extern "C" fn open_settings_action(_: &Object, _: Sel, _: id) {
            unsafe {
                open_settings(CONFIG_PORT);
            }
        }

        extern "C" fn open_client_action(_: &Object, _: Sel, _: id) {
            open_client();
        }

        extern "C" fn exit_action(_: &Object, _: Sel, _: id) {
            tracing::info!("Tray exit requested; shutting down server");
            std::process::exit(0);
        }

        extern "C" fn voice_input_action(_: &Object, _: Sel, _: id) {
            unsafe {
                if let Some(ref tx) = ACTION_TX {
                    // Dispatch on process running state (is_local), not model
                    // load state — the menu item is Start when the process is
                    // down, Stop when it's up.
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
        }

        extern "C" fn stop_http_action(_: &Object, _: Sel, sender: id) {
            unsafe {
                let root: id = msg_send![sender, representedObject];
                let root = nsstring_to_string(root);
                if let Some(ref tx) = ACTION_TX {
                    let _ = tx.send(TrayAction::StopStaticHttp { root });
                }
            }
        }

        unsafe {
            decl.add_method(
                sel!(trayClicked:),
                tray_clicked as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(openSettings:),
                open_settings_action as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(openClient:),
                open_client_action as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(exitApp:),
                exit_action as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(voiceInput:),
                voice_input_action as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(stopHttp:),
                stop_http_action as extern "C" fn(&Object, Sel, id),
            );

            decl.register()
        }
    })
}

unsafe fn nsstring_to_string(value: id) -> String {
    let utf8: *const std::os::raw::c_char = msg_send![value, UTF8String];
    std::ffi::CStr::from_ptr(utf8)
        .to_string_lossy()
        .into_owned()
}

unsafe fn create_target() -> id {
    let cls = get_target_class();
    let target: id = msg_send![cls, new];
    target
}

type NSUInteger = u64;

const fn left_mouse_down_mask() -> NSUInteger {
    1 << 1
}

const fn right_mouse_down_mask() -> NSUInteger {
    1 << 3
}
