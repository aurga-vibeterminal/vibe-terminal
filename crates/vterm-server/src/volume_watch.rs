//! Watches for newly mounted volumes and re-runs path-dependent scans.
//!
//! The server auto-starts at login, possibly before encrypted or removable
//! volumes are mounted (an APFS encrypted volume unlocked after login, a
//! LUKS device, a BitLocker drive). Scans that ran at startup (recent
//! projects, ...) then miss everything on those volumes, so each mount
//! event re-triggers the callback registered through [`spawn`].
//!
//! macOS: two notification mechanisms, both delivered on global dispatch
//! queues (no CFRunLoop/AppKit run loop, so this works headless; CFRunLoop
//! scheduling delivered the initial enumeration but never live events in
//! testing on macOS 26):
//! - DiskArbitration: `DARegisterDiskAppearedCallback` plus a volume-path
//!   `DARegisterDiskDescriptionChangedCallback`. Live events fire *before*
//!   the mount completes (an encrypted-volume unlock included), so the disk
//!   description at callback time carries no volume path; every event is
//!   treated as a trigger and left to the debounce + rescan.
//! - FSEvents on /Volumes: an encrypted volume that was present-but-locked
//!   at boot fires neither DA callback when it is unlocked later (observed
//!   on macOS 26), but the mount always creates the mount-point directory,
//!   which FSEvents reports.
//!
//! Linux: poll(2) on /proc/self/mounts, which reports POLLPRI when the
//! mount table changes (documented kernel interface since 2.6.15). Covers
//! LUKS unlock + mount, udisks automounts, and manual mounts alike.
//!
//! Windows: three mechanisms, because a BitLocker volume present-but-locked
//! at boot is invisible to the first one:
//! - a hidden top-level window registered with RegisterDeviceNotification
//!   for GUID_DEVINTERFACE_VOLUME: catches newly arriving volumes (USB plug,
//!   VHD mount) instantly, and re-subscribes per-volume handles for them;
//! - per-volume handle notifications (RegisterDeviceNotification with
//!   DBT_DEVTYP_HANDLE on a zero-access volume handle — raw volume opens
//!   are admin-gated, query-only handles are not): GUID_IO_VOLUME_MOUNT and
//!   GUID_IO_VOLUME_FVE_STATUS_CHANGE arrive the instant a locked BitLocker
//!   volume is unlocked (measured: FVE at unlock start, MOUNT ~0.7s later;
//!   no DBT_DEVICEARRIVAL fires at all for that flow);
//! - a slow poll of mounted volumes as a final backstop should both
//!   notification paths silently fail.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// One mount produces a burst of events (the volume, Disk Arbitration
/// re-notifications, successive mount-table changes during automount).
/// Fire the callback once the burst has been quiet for this long.
const QUIET_PERIOD: Duration = Duration::from_millis(750);

/// Start watching for volume mounts; `on_volume_mounted` runs (debounced)
/// on a dedicated thread for the rest of the process lifetime.
pub(crate) fn spawn<F>(on_volume_mounted: F)
where
    F: Fn() + Send + Sync + 'static,
{
    let (tx, rx) = mpsc::channel::<()>();
    imp::spawn_watcher(tx);

    let builder = thread::Builder::new().name("volume-watch".to_string());
    if let Err(error) = builder.spawn(move || {
        while rx.recv().is_ok() {
            // Wait until the burst goes quiet, then fire once.
            while rx.recv_timeout(QUIET_PERIOD).is_ok() {}
            on_volume_mounted();
        }
    }) {
        tracing::warn!("cannot spawn the volume-watch debounce thread: {error}");
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::ffi::c_void;
    use std::sync::mpsc::Sender;

    use core_foundation::array::CFArray;
    use core_foundation::base::{CFAllocatorRef, CFTypeRef, TCFType};
    use core_foundation::string::{CFString, CFStringRef};
    use core_foundation::url::CFURLRef;
    use tracing::{info, warn};

    type DASessionRef = *mut c_void;
    type DADiskRef = *mut c_void;
    type CFDictionaryRef = *const c_void;
    type CFArrayRef = *const c_void;
    type DADiskAppearedCallback = unsafe extern "C" fn(DADiskRef, *mut c_void);
    type DADiskDescriptionChangedCallback =
        unsafe extern "C" fn(DADiskRef, CFArrayRef, *mut c_void);
    type FSEventStreamRef = *mut c_void;
    type FSEventStreamCallback = unsafe extern "C" fn(
        FSEventStreamRef,
        *mut c_void,
        usize,
        *mut c_void,
        *const u32,
        *const u64,
    );

    /// FSEventStreamCreate's `context` argument is a pointer to this struct,
    /// NOT the user pointer itself: FSEvents reads `info` out of it and calls
    /// `retain`/`release` when non-NULL. Passing the user pointer directly
    /// makes FSEvents interpret the pointee's bytes as function pointers and
    /// jump into garbage (observed as SIGBUS/SIGSEGV inside
    /// `_FSEventStreamCreate`).
    #[repr(C)]
    struct FSEventStreamContext {
        version: isize, // CFIndex
        info: *mut c_void,
        retain: Option<unsafe extern "C" fn(*const c_void) -> *const c_void>,
        release: Option<unsafe extern "C" fn(*const c_void)>,
        copy_description: Option<unsafe extern "C" fn(*const c_void) -> CFStringRef>,
    }

    const K_CFURL_POSIX_PATH_STYLE: isize = 0;

    #[link(name = "DiskArbitration", kind = "framework")]
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kDADiskDescriptionVolumePathKey: CFStringRef;
        static kDADiskDescriptionMediaBSDNameKey: CFStringRef;
        static kDADiskDescriptionWatchVolumePath: CFStringRef;
        fn DASessionCreate(allocator: CFAllocatorRef) -> DASessionRef;
        fn DARegisterDiskAppearedCallback(
            session: DASessionRef,
            matches: CFDictionaryRef,
            callback: DADiskAppearedCallback,
            context: *mut c_void,
        );
        fn DARegisterDiskDescriptionChangedCallback(
            session: DASessionRef,
            matches: CFDictionaryRef,
            watch: CFArrayRef,
            callback: DADiskDescriptionChangedCallback,
            context: *mut c_void,
        );
        fn DASessionSetDispatchQueue(session: DASessionRef, queue: *mut c_void) -> i32;
        fn DADiskCopyDescription(disk: DADiskRef) -> CFDictionaryRef;
        fn CFDictionaryGetValue(dict: CFDictionaryRef, key: *const c_void) -> *const c_void;
        fn CFURLCopyFileSystemPath(url: CFURLRef, path_style: isize) -> CFStringRef;
        fn CFRelease(cf: CFTypeRef);
        fn dispatch_get_global_queue(identifier: isize, flags: usize) -> *mut c_void;
    }

    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn FSEventStreamCreate(
            allocator: CFAllocatorRef,
            callback: FSEventStreamCallback,
            context: *mut c_void,
            paths_to_watch: CFArrayRef,
            since_when: u64,
            latency: f64,
            flags: u32,
        ) -> FSEventStreamRef;
        fn FSEventStreamSetDispatchQueue(stream: FSEventStreamRef, queue: *mut c_void);
        fn FSEventStreamStart(stream: FSEventStreamRef) -> u8;
    }

    pub(super) fn spawn_watcher(sender: Sender<()>) {
        spawn_fs_events_watcher(sender.clone());
        // A dispatch queue, not a CFRunLoop: with DASessionScheduleWithRunLoop
        // the initial enumeration arrives but live mount/unmount events never
        // did (observed on macOS 26); dispatch-queue delivery works.
        unsafe {
            let session = DASessionCreate(std::ptr::null());
            if session.is_null() {
                warn!("DASessionCreate failed; DA volume watch disabled");
                return;
            }
            // The callback context and the session live for the process
            // lifetime: intentionally never reclaimed.
            let context = Box::into_raw(Box::new(sender)) as *mut c_void;
            DARegisterDiskAppearedCallback(session, std::ptr::null(), disk_appeared, context);
            // Belt and braces: on paper an encrypted-volume unlock is a
            // description change (the locked disk appeared at boot and gains
            // a volume path on unlock). In practice neither this callback nor
            // "appeared" fired for that flow in testing on macOS 26 — the
            // FSEvents watcher covers it — but keep the registration in case
            // other flows (or OS versions) report mounts only this way.
            let watch_keys = CFArray::from_CFTypes(&[CFString::wrap_under_get_rule(
                kDADiskDescriptionWatchVolumePath,
            )]);
            DARegisterDiskDescriptionChangedCallback(
                session,
                std::ptr::null(),
                watch_keys.as_concrete_TypeRef() as CFArrayRef,
                disk_description_changed,
                context,
            );
            // It is undocumented whether registration retains the watch
            // array; leak it like the context so it cannot dangle.
            std::mem::forget(watch_keys);
            // QOS_CLASS_UNSPECIFIED (0): the default-priority global queue.
            let queue = dispatch_get_global_queue(0, 0);
            if DASessionSetDispatchQueue(session, queue) != 0 {
                warn!("DASessionSetDispatchQueue failed; DA volume watch disabled");
            }
        }
    }

    /// Mounting a volume creates its mount-point directory under /Volumes
    /// (an encrypted-volume unlock included), so an FSEvents stream on the
    /// directory catches mounts the DiskArbitration callbacks miss. Pure
    /// notification: the stream is delivered on a global dispatch queue.
    fn spawn_fs_events_watcher(sender: Sender<()>) {
        let paths = CFArray::from_CFTypes(&[CFString::new("/Volumes")]);
        // The context struct and the sender it points to live for the process
        // lifetime: intentionally never reclaimed.
        let context = Box::into_raw(Box::new(FSEventStreamContext {
            version: 0,
            info: Box::into_raw(Box::new(sender)) as *mut c_void,
            retain: None,
            release: None,
            copy_description: None,
        }));
        unsafe {
            let stream = FSEventStreamCreate(
                std::ptr::null(),
                fs_events_callback,
                context as *mut c_void,
                paths.as_concrete_TypeRef() as CFArrayRef,
                // kFSEventStreamEventIdSinceNow
                u64::MAX,
                0.5,
                // kFSEventStreamCreateFlagNone
                0,
            );
            std::mem::forget(paths);
            if stream.is_null() {
                warn!("FSEventStreamCreate failed; /Volumes watch disabled");
                return;
            }
            FSEventStreamSetDispatchQueue(stream, dispatch_get_global_queue(0, 0));
            if FSEventStreamStart(stream) == 0 {
                warn!("FSEventStreamStart failed; /Volumes watch disabled");
            }
        }
    }

    unsafe extern "C" fn fs_events_callback(
        _stream: FSEventStreamRef,
        context: *mut c_void,
        num_events: usize,
        event_paths: *mut c_void,
        _event_flags: *const u32,
        _event_ids: *const u64,
    ) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            // Without kFSEventStreamCreateFlagUseCFTypes, event_paths is a
            // raw C array of C strings, NOT a CFArray.
            //
            // FSEvents watches recursively, and we only care about mount
            // points coming and going: react to /Volumes itself or a direct
            // child of it. Deeper changes (file activity inside a mounted
            // volume) are ignored so builds under /Volumes/Data don't
            // trigger rescan storms.
            let paths = event_paths as *const *const std::os::raw::c_char;
            let mut mount_change = false;
            for index in 0..num_events {
                let path = std::ffi::CStr::from_ptr(paths.add(index).read()).to_string_lossy();
                // Overflow/coalescing flags (MustScanSubDirs, UserDropped,
                // KernelDropped): the stream lost track of what changed, so
                // treat it as a possible mount change.
                let flags = *_event_flags.add(index);
                let depth = path.strip_prefix("/Volumes").map(|rest| {
                    rest.trim_matches('/')
                        .split('/')
                        .filter(|c| !c.is_empty())
                        .count()
                });
                if matches!(depth, Some(0 | 1)) || flags & 0x7 != 0 {
                    mount_change = true;
                    info!("fs event: mount change at {path} (flags={flags:#x})");
                }
            }
            if mount_change {
                let sender = &*(context as *const Sender<()>);
                let _ = sender.send(());
            }
        }));
    }

    unsafe extern "C" fn disk_appeared(disk: DADiskRef, context: *mut c_void) {
        report_mounted_volume(disk, context);
    }

    unsafe extern "C" fn disk_description_changed(
        disk: DADiskRef,
        _keys: CFArrayRef,
        context: *mut c_void,
    ) {
        report_mounted_volume(disk, context);
    }

    fn report_mounted_volume(disk: DADiskRef, context: *mut c_void) {
        // Panics cannot cross the FFI boundary; none of this should ever
        // fail, but unwind anyway.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            // Log the BSD name and (when already mounted) the volume path.
            // Do NOT filter on the path: on a live mount or an encrypted
            // volume unlock, "appeared" fires before the mount completes and
            // the description still has no volume path.
            let description = DADiskCopyDescription(disk);
            if description.is_null() {
                return;
            }
            let bsd_name = CFDictionaryGetValue(
                description,
                kDADiskDescriptionMediaBSDNameKey as *const c_void,
            );
            let bsd_name = if bsd_name.is_null() {
                "?".to_string()
            } else {
                CFString::wrap_under_get_rule(bsd_name as CFStringRef).to_string()
            };
            let url = CFDictionaryGetValue(
                description,
                kDADiskDescriptionVolumePathKey as *const c_void,
            );
            let path = if url.is_null() {
                None
            } else {
                let path = CFURLCopyFileSystemPath(url as CFURLRef, K_CFURL_POSIX_PATH_STYLE);
                (!path.is_null()).then(|| CFString::wrap_under_create_rule(path).to_string())
            };
            CFRelease(description);
            //             match path {
            //                 Some(path) => info!("disk event: {bsd_name} mounted at {path}"),
            //                 None => info!("disk event: {bsd_name}"),
            //             }
            let sender = &*(context as *const Sender<()>);
            let _ = sender.send(());
        }));
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::os::unix::io::AsRawFd;
    use std::sync::mpsc::Sender;

    use tracing::warn;

    pub(super) fn spawn_watcher(sender: Sender<()>) {
        let builder = std::thread::Builder::new().name("volume-watch-mounts".to_string());
        if let Err(error) = builder.spawn(move || run(sender)) {
            warn!("cannot spawn the mount-table watch thread: {error}");
        }
    }

    /// poll(2) on /proc/self/mounts reports POLLPRI when the mount table
    /// changes; the file must be re-read to re-arm the notification.
    fn run(sender: Sender<()>) {
        let mut file = match File::open("/proc/self/mounts") {
            Ok(file) => file,
            Err(error) => {
                warn!("cannot open /proc/self/mounts; volume watch disabled: {error}");
                return;
            }
        };
        let mut contents = String::new();
        loop {
            let mut pollfd = libc::pollfd {
                fd: file.as_raw_fd(),
                events: libc::POLLPRI | libc::POLLERR,
                revents: 0,
            };
            let result = unsafe { libc::poll(&mut pollfd, 1, -1) };
            if result < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                warn!("poll on /proc/self/mounts failed; volume watch disabled: {error}");
                return;
            }
            contents.clear();
            if file
                .seek(SeekFrom::Start(0))
                .and_then(|_| file.read_to_string(&mut contents))
                .is_err()
            {
                warn!("cannot re-read /proc/self/mounts; volume watch disabled");
                return;
            }
            let _ = sender.send(());
        }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use std::ffi::c_void;
    use std::mem::{size_of, zeroed};
    use std::ptr;
    use std::sync::mpsc::Sender;
    use std::sync::Mutex;
    use std::time::Duration;

    use tracing::{info, trace, warn};
    use windows_sys::core::GUID;
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, HANDLE, HWND, INVALID_HANDLE_VALUE, LPARAM, LRESULT, WPARAM,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FindFirstVolumeW, FindNextVolumeW, FindVolumeClose, GetDriveTypeW,
        GetLogicalDrives, GetVolumeInformationW,
    };
    use windows_sys::Win32::System::Ioctl::GUID_DEVINTERFACE_VOLUME;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowLongPtrW,
        RegisterClassW, RegisterDeviceNotificationW, SetWindowLongPtrW, DBT_CUSTOMEVENT,
        DBT_DEVICEARRIVAL, DBT_DEVTYP_DEVICEINTERFACE, DBT_DEVTYP_HANDLE,
        DEVICE_NOTIFY_WINDOW_HANDLE, DEV_BROADCAST_DEVICEINTERFACE_W, DEV_BROADCAST_HANDLE,
        DEV_BROADCAST_HDR, GWLP_USERDATA, MSG, WM_DEVICECHANGE, WNDCLASSW,
    };

    /// GUID_IO_VOLUME_MOUNT (ioevent.h): a file system was just mounted on
    /// the volume. THE BitLocker-unlock signal — measured: for a volume that
    /// sat locked since boot, no DBT_DEVICEARRIVAL path fires on unlock, but
    /// this event arrives the moment the file system mounts.
    const GUID_IO_VOLUME_MOUNT: GUID = GUID {
        data1: 0xb580_4878,
        data2: 0x1a96,
        data3: 0x11d2,
        data4: [0x8f, 0xfd, 0x00, 0xa0, 0xc9, 0xa0, 0x6d, 0x32],
    };

    /// GUID_IO_VOLUME_FVE_STATUS_CHANGE (ioevent.h): BitLocker began
    /// unlocking the volume (~0.7s before the mount event, measured).
    const GUID_IO_VOLUME_FVE_STATUS_CHANGE: GUID = GUID {
        data1: 0x0629_98b2,
        data2: 0xee1f,
        data3: 0x4b6a,
        data4: [0xb8, 0x57, 0xe7, 0x6c, 0xbb, 0xe9, 0xa6, 0xda],
    };

    const FILE_SHARE_ALL: u32 = 0x1 | 0x2 | 0x4;
    const OPEN_EXISTING: u32 = 3;
    /// GetDriveTypeW's DRIVE_REMOTE value (the constant lives in the
    /// Win32_System_WindowsProgramming feature, unneeded otherwise).
    const DRIVE_REMOTE: u32 = 4;

    /// Final backstop in case both notification paths silently die: slow on
    /// purpose, it exists to guarantee an eventual refresh, not speed.
    const POLL_INTERVAL: Duration = Duration::from_secs(30);

    /// FindFirstVolumeW/FindNextVolumeW want room for a volume GUID path
    /// ("\\?\Volume{...}\" — 49 characters); MAX_PATH leaves generous slack.
    const VOLUME_PATH_CAPACITY: usize = 260;

    /// Volumes with a live handle registration: (volume GUID path, handle).
    /// Handles stay open — closing one silences its events — and entries are
    /// only dropped once their volume disappears from the enumeration.
    static HANDLE_REGISTRATIONS: Mutex<Vec<(String, usize)>> = Mutex::new(Vec::new());

    pub(super) fn spawn_watcher(sender: Sender<()>) {
        let window_sender = sender.clone();
        let builder = std::thread::Builder::new().name("volume-watch-wm".to_string());
        if let Err(error) = builder.spawn(move || unsafe { run(window_sender) }) {
            warn!("cannot spawn the volume notification window thread: {error}");
        }
        // The window cannot see a BitLocker unlock (see the module docs), so
        // back it with a volume-state poller sharing the same sender.
        let builder = std::thread::Builder::new().name("volume-watch-poll".to_string());
        if let Err(error) = builder.spawn(move || run_poll(sender)) {
            warn!("cannot spawn the volume state poll thread: {error}");
        }
    }

    /// A hidden top-level window: volume broadcasts only reach top-level
    /// windows, and the registered GUID_DEVINTERFACE_VOLUME notification is
    /// delivered to the window handle either way.
    unsafe fn run(sender: Sender<()>) {
        let class_name = wide("vterm-volume-watch");
        let instance = GetModuleHandleW(ptr::null());
        let mut class: WNDCLASSW = zeroed();
        class.lpfnWndProc = Some(wndproc);
        class.hInstance = instance;
        class.lpszClassName = class_name.as_ptr();
        if RegisterClassW(&class) == 0 {
            warn!("RegisterClassW failed; volume watch disabled");
            return;
        }
        let window = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            instance,
            ptr::null_mut(),
        );
        if window.is_null() {
            warn!("CreateWindowExW failed; volume watch disabled");
            return;
        }
        // Process-lifetime: intentionally never reclaimed.
        SetWindowLongPtrW(
            window,
            GWLP_USERDATA,
            Box::into_raw(Box::new(sender)) as isize,
        );

        let mut filter = DEV_BROADCAST_DEVICEINTERFACE_W {
            dbcc_size: size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as u32,
            dbcc_devicetype: DBT_DEVTYP_DEVICEINTERFACE,
            dbcc_reserved: 0,
            dbcc_classguid: GUID_DEVINTERFACE_VOLUME,
            dbcc_name: [0; 1],
        };
        let notification = RegisterDeviceNotificationW(
            window as *mut c_void,
            &mut filter as *mut _ as *const c_void,
            DEVICE_NOTIFY_WINDOW_HANDLE,
        );
        if notification.is_null() {
            warn!("RegisterDeviceNotificationW failed; volume watch disabled");
            return;
        }

        // Per-volume mount events — the BitLocker-unlock signal (see the
        // module docs). After GWLP_USERDATA, so events raised while this runs
        // already reach the sender.
        register_volume_handle_notifications(window);

        let mut message: MSG = zeroed();
        while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
            DispatchMessageW(&message);
        }
    }

    unsafe extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_DEVICECHANGE {
            match wparam as u32 {
                // A new volume (USB plug, VHD): subscribe to its future mount
                // events too, then trigger the debounced rescan.
                DBT_DEVICEARRIVAL => {
                    register_volume_handle_notifications(window);
                    notify_sender(window);
                }
                // Mount events on volumes we hold handles for. This is the
                // only notification a BitLocker unlock produces.
                DBT_CUSTOMEVENT if lparam != 0 => {
                    let header = &*(lparam as *const DEV_BROADCAST_HDR);
                    if header.dbch_devicetype == DBT_DEVTYP_HANDLE {
                        let event = &*(lparam as *const DEV_BROADCAST_HANDLE);
                        if same_guid(&event.dbch_eventguid, &GUID_IO_VOLUME_MOUNT)
                            || same_guid(&event.dbch_eventguid, &GUID_IO_VOLUME_FVE_STATUS_CHANGE)
                        {
                            info!(
                                "volume event: {}",
                                if same_guid(&event.dbch_eventguid, &GUID_IO_VOLUME_MOUNT) {
                                    "file system mounted"
                                } else {
                                    "encryption status changing"
                                }
                            );
                            notify_sender(window);
                        }
                    }
                }
                _ => {}
            }
        }
        DefWindowProcW(window, message, wparam, lparam)
    }

    unsafe fn notify_sender(window: HWND) {
        let sender = GetWindowLongPtrW(window, GWLP_USERDATA) as *const Sender<()>;
        if !sender.is_null() {
            let _ = (*sender).send(());
        }
    }

    fn same_guid(left: &GUID, right: &GUID) -> bool {
        left.data1 == right.data1
            && left.data2 == right.data2
            && left.data3 == right.data3
            && left.data4 == right.data4
    }

    /// Subscribes to mount events for every volume currently in the system,
    /// locked ones included. Raw volume handles are admin-gated, but a
    /// zero-access open is enough for RegisterDeviceNotification. Called at
    /// startup and again on every volume arrival; self-healing: registrations
    /// whose volume has disappeared are closed and dropped so a replugged
    /// volume (same GUID, dead handle) registers fresh.
    unsafe fn register_volume_handle_notifications(window: HWND) {
        let mut volumes = Vec::new();
        let mut path: [u16; VOLUME_PATH_CAPACITY] = zeroed();
        let find: HANDLE = FindFirstVolumeW(path.as_mut_ptr(), path.len() as u32);
        if find == INVALID_HANDLE_VALUE {
            warn!(
                "FindFirstVolumeW failed; per-volume mount events disabled: {}",
                GetLastError()
            );
            return;
        }
        loop {
            let length = path.iter().position(|c| *c == 0).unwrap_or(path.len());
            volumes.push(String::from_utf16_lossy(&path[..length]));
            if FindNextVolumeW(find, path.as_mut_ptr(), path.len() as u32) == 0 {
                break;
            }
        }
        let _ = FindVolumeClose(find);

        if let Ok(mut registrations) = HANDLE_REGISTRATIONS.lock() {
            registrations.retain(|(volume, handle)| {
                let alive = volumes.iter().any(|current| current == volume);
                if !alive {
                    let _ = CloseHandle(*handle as HANDLE);
                }
                alive
            });
        }

        for volume in volumes {
            if HANDLE_REGISTRATIONS
                .lock()
                .map(|registrations| registrations.iter().any(|(path, _)| *path == volume))
                .unwrap_or(false)
            {
                continue;
            }
            // No trailing backslash: open the volume device itself, not the
            // file-system root directory (locked volumes have none).
            let mut device = wide(volume.trim_end_matches('\\'));
            let handle = CreateFileW(
                device.as_mut_ptr(),
                0,
                FILE_SHARE_ALL,
                ptr::null(),
                OPEN_EXISTING,
                0,
                ptr::null_mut(),
            );
            if handle == INVALID_HANDLE_VALUE {
                // Rare; the poll backstop still covers such volumes.
                trace!("cannot open {volume} for mount events: {}", GetLastError());
                continue;
            }
            let mut filter: DEV_BROADCAST_HANDLE = zeroed();
            filter.dbch_size = size_of::<DEV_BROADCAST_HANDLE>() as u32;
            filter.dbch_devicetype = DBT_DEVTYP_HANDLE;
            filter.dbch_handle = handle;
            let notification = RegisterDeviceNotificationW(
                window as *mut c_void,
                &filter as *const _ as *const c_void,
                DEVICE_NOTIFY_WINDOW_HANDLE,
            );
            if notification.is_null() {
                warn!(
                    "RegisterDeviceNotificationW failed for {volume}: {}",
                    GetLastError()
                );
                let _ = CloseHandle(handle);
            } else {
                if let Ok(mut registrations) = HANDLE_REGISTRATIONS.lock() {
                    registrations.push((volume.clone(), handle as usize));
                }
                trace!("watching mount events on {volume}");
            }
        }
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Polling backstop for the (measured) case where neither notification
    /// path delivers: snapshots volume GUID paths and drive letters with
    /// their readiness, so a mount whose events went missing still flips the
    /// snapshot within POLL_INTERVAL.
    fn run_poll(sender: Sender<()>) {
        let mut previous = volume_state();
        loop {
            std::thread::sleep(POLL_INTERVAL);
            let current = volume_state();
            if current != previous {
                info!(
                    "mounted-volume state changed: {}",
                    describe_change(&previous, &current)
                );
                previous = current;
                let _ = sender.send(());
            }
        }
    }

    /// One entry per mounted volume and per drive letter. Volume GUID paths
    /// carry the volume serial when a file system is mounted on them — a
    /// locked BitLocker volume keeps its GUID path (and usually its letter)
    /// with no serial until the unlock flips that. Drive letters also catch
    /// network-drive mappings, which have no volume device and no mount
    /// events at all.
    fn volume_state() -> Vec<(String, Option<u32>)> {
        let mut state = Vec::new();

        let mut path = [0u16; VOLUME_PATH_CAPACITY];
        unsafe {
            let find: HANDLE = FindFirstVolumeW(path.as_mut_ptr(), path.len() as u32);
            if find == INVALID_HANDLE_VALUE {
                return state;
            }
            loop {
                let mut serial = 0u32;
                // Fails fast while the volume carries no file system; the
                // other outputs are not needed for the signature.
                let mounted = GetVolumeInformationW(
                    path.as_ptr(),
                    ptr::null_mut(),
                    0,
                    &mut serial,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    0,
                ) != 0;
                let length = path.iter().position(|c| *c == 0).unwrap_or(path.len());
                state.push((
                    String::from_utf16_lossy(&path[..length]),
                    mounted.then_some(serial),
                ));
                if FindNextVolumeW(find, path.as_mut_ptr(), path.len() as u32) == 0 {
                    break;
                }
            }
            let _ = FindVolumeClose(find);
        }

        let mask = unsafe { GetLogicalDrives() };
        for bit in 0..26 {
            if mask & (1 << bit) == 0 {
                continue;
            }
            let root = format!("{}:\\", (b'A' + bit as u8) as char);
            let root_wide = wide(&root);
            if unsafe { GetDriveTypeW(root_wide.as_ptr()) } == DRIVE_REMOTE {
                // Probing remote roots can block for seconds on stale
                // shares; the letter appearing in the mask is the mapping
                // event, readiness flaps are just noise.
                state.push((root, Some(0)));
                continue;
            }
            let mut serial = 0u32;
            let ready = unsafe {
                GetVolumeInformationW(
                    root_wide.as_ptr(),
                    ptr::null_mut(),
                    0,
                    &mut serial,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    0,
                ) != 0
            };
            state.push((root, ready.then_some(serial)));
        }
        state
    }

    /// Human-readable diff between two snapshots, for the rescan log line.
    fn describe_change(
        previous: &[(String, Option<u32>)],
        current: &[(String, Option<u32>)],
    ) -> String {
        let mut notes = Vec::new();
        for (path, serial) in current {
            match previous
                .iter()
                .find(|(previous_path, _)| previous_path == path)
            {
                None => notes.push(format!("{path} mounted")),
                Some((_, previous_serial)) => match (previous_serial.is_some(), serial.is_some()) {
                    (false, true) => notes.push(format!("{path} became accessible")),
                    (true, false) => notes.push(format!("{path} became inaccessible")),
                    _ => {}
                },
            }
        }
        for (path, _) in previous {
            if !current.iter().any(|(current_path, _)| current_path == path) {
                notes.push(format!("{path} disappeared"));
            }
        }
        if notes.is_empty() {
            notes.push("volume serial changed".to_string());
        }
        notes.join(", ")
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
mod imp {
    use std::sync::mpsc::Sender;

    pub(super) fn spawn_watcher(_sender: Sender<()>) {
        tracing::warn!("volume watch is not supported on this platform");
    }
}
