//! IPC control-channel schema shared by the coordinator (vterm-server) and
//! the media process (vterm-screen-server) — spec
//! `docs/superpowers/specs/2026-08-06-screen-streaming-coordination.md` §3.
//!
//! Types and framing only; no transport implementation and no pipeline
//! dependencies, so the daemon shares one schema with the screen-server
//! without linking the capture/encode stack.
//!
//! Wire format (§3.1): length-prefixed JSON over a pipe/UDS stream —
//! `[u32 LE length][UTF-8 JSON object]`. Requests carry `"id"`, responses
//! echo it; events are unsolicited (`"event"` tag, no id). The first frame
//! after connect is server→client [`Hello`].

use std::io::{self, Read, Write};
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// IPC protocol version ([`Hello::protocol`]); bump on schema breaks.
pub const PROTOCOL: u32 = 1;

/// Largest accepted frame (1 MiB) — the catalog and stats are small;
/// anything bigger is a bug or an attack, so the reader errors out.
pub const MAX_FRAME_LEN: u32 = 1 << 20;

/// Environment variable overriding the IPC endpoint (named pipe name on
/// Windows, socket path elsewhere). Used by integration tests so they never
/// collide with a production instance.
pub const ENDPOINT_ENV: &str = "VTERM_SCREEN_CTL_ENDPOINT";

// ---------------------------------------------------------------------------
// Endpoint naming (spec §2.3)
// ---------------------------------------------------------------------------

/// The IPC endpoint this process should connect to / listen on:
/// `VTERM_SCREEN_CTL_ENDPOINT` if set, otherwise the platform default.
/// On Windows the override may be a bare pipe name (`vterm-foo`), expanded
/// to `\\.\pipe\vterm-foo`.
pub fn endpoint() -> String {
    if let Ok(custom) = std::env::var(ENDPOINT_ENV) {
        if !custom.is_empty() {
            return normalize_endpoint(&custom);
        }
    }
    default_endpoint()
}

#[cfg(target_os = "windows")]
fn normalize_endpoint(custom: &str) -> String {
    if custom.starts_with(r"\\") {
        custom.to_string()
    } else {
        format!(r"\\.\pipe\{custom}")
    }
}

#[cfg(not(target_os = "windows"))]
fn normalize_endpoint(custom: &str) -> String {
    custom.to_string()
}

/// The platform-default endpoint (no env override).
#[cfg(target_os = "windows")]
fn default_endpoint() -> String {
    r"\\.\pipe\vterm-screen-ctl".to_string()
}

/// The platform-default endpoint: `$XDG_RUNTIME_DIR/vterm-screen-ctl.sock`,
/// falling back to `~/.vibeterminal/vterm-screen-ctl.sock`.
#[cfg(not(target_os = "windows"))]
fn default_endpoint() -> String {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir)
                .join("vterm-screen-ctl.sock")
                .to_string_lossy()
                .into_owned();
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".vibeterminal")
        .join("vterm-screen-ctl.sock")
        .to_string_lossy()
        .into_owned()
}

// ---------------------------------------------------------------------------
// Message envelopes
// ---------------------------------------------------------------------------

/// First frame after connect, server → client (spec §3.1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hello {
    pub protocol: u32,
    /// Screen-server crate version.
    pub version: String,
    pub pid: u32,
    /// Random per-process id: a reconnecting coordinator can tell a
    /// screen-server restart (new session id ⇒ volatile state like token
    /// grants was lost) from a dropped pipe.
    pub session_id: u64,
    pub udp_port: u16,
    pub capabilities: Vec<String>,
}

/// One frame on the wire. Untagged: the shape of the JSON object decides
/// (`{"hello": …}` / `{"id": …, "cmd": …}` / `{"id": …, "result": …}` /
/// `{"event": …}`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Frame {
    Hello { hello: Hello },
    Request(Request),
    Response(Response),
    Event(Event),
}

/// A coordinator → screen-server request (spec §3.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: Cmd,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Cmd {
    GetStatus,
    /// Set/clear tool-path overrides (§4). `None` fields are left alone;
    /// `clear_*` resets the corresponding path to auto-detection.
    SetConfig {
        #[serde(default)]
        sdk_path: Option<String>,
        #[serde(default)]
        adb_path: Option<String>,
        #[serde(default)]
        agent_dir: Option<String>,
        #[serde(default)]
        clear_sdk: bool,
        #[serde(default)]
        clear_adb: bool,
        #[serde(default)]
        clear_agent: bool,
    },
    GetPaths,
    ListSources,
    /// Mint a one-time stream token (§6). `token` is 32 lowercase hex chars
    /// (16 bytes, see [`token_from_hex`]). `client_addr`, when set, binds
    /// the grant to that client address (security review §3.2): the UDP
    /// handshake must originate from it. Absent for relay pairings and
    /// older coordinators.
    GrantToken {
        token: String,
        sources: Vec<String>,
        client_label: String,
        ttl_ms: u64,
        #[serde(default)]
        client_addr: Option<String>,
    },
    RevokeToken {
        token: String,
    },
    /// Revoke every grant of one client (its WSS session ended).
    RevokeTokens {
        client_label: String,
    },
    /// Boot/shutdown an iOS simulator (macOS screen-server only;
    /// `source::boot_simulator`/`shutdown_simulator` wrappers).
    BootSimulator {
        udid: String,
    },
    ShutdownSimulator {
        udid: String,
    },
    /// Launch/shut down an Android emulator (any host;
    /// `source::boot_android_emulator`/`shutdown_android_emulator` wrappers).
    /// Boot takes an AVD name, shutdown the adb serial of a running emulator.
    BootAndroidEmulator {
        avd: String,
    },
    ShutdownAndroidEmulator {
        serial: String,
    },
    /// Rotate an Android device/emulator display by 90° (`adb` user_rotation
    /// cycle; `source::rotate_android_device` wrapper).
    RotateAndroidDevice {
        serial: String,
    },
    /// Read an iOS simulator's pasteboard (`xcrun simctl pbpaste`), so the
    /// coordinator can relay a device copy to the client.
    GetClipboard {
        udid: String,
    },
    /// Bring the desktop input backend up (the webui "request permission"
    /// action): on Linux this opens the xdg-desktop-portal RemoteDesktop
    /// consent dialog (libei), on macOS it can prompt for Accessibility.
    /// Input injection otherwise constructs the backend lazily on the first
    /// client input event — starting a stream never pops the dialog.
    WarmupInput,
    /// Whether desktop input can bypass the OS consent path on this host
    /// through the kernel uinput module (Linux): does `/dev/uinput` exist,
    /// and can this process open it read-write? The webui shows the setup
    /// script when it cannot.
    GetInputStatus,
    /// Forget every persisted portal ScreenCast restore token (the webui's
    /// "re-select display" action): the next stream start shows the source
    /// picker again instead of silently reusing the remembered selection.
    ResetScreenConsent,
    /// Interactively grant screen-capture consent (Linux only): pop the
    /// portal's ScreenCast source picker NOW — not at the next stream start
    /// — and persist the answered restore token as the remembered selection
    /// of EVERY display spec (the generic screen source plus each
    /// connector-keyed entry), so a client connecting to any of them
    /// captures the freshly granted source. The webui's "grant permission"
    /// action. Non-Linux hosts answer `Error`.
    GrantScreenConsent,
    /// Windows service management (the webui's install/uninstall buttons).
    /// `ServiceStatus` needs no elevation; install/remove go through an
    /// elevated helper copy of the server exe (UAC prompt). Non-Windows
    /// hosts answer `Error`.
    ServiceStatus,
    ServiceInstall,
    ServiceRemove,
    /// Encoder availability catalog + the current stream tunables.
    GetEncoders,
    /// Set the encoder backend and/or the frame-rate override for streams
    /// started AFTER this call (running pipelines are unaffected).
    /// `encoder`: "auto" | "vendor" | "d3d12" | "mf"; `fps`: 15 | 30 | 60;
    /// `max_resolution`: one of [`MAX_RESOLUTION_IDS`] (capture ceiling for
    /// the virtual-display source; default "4k"); `bitrate_kbps`: fixed
    /// average bitrate — the only non-zero values accepted are
    /// [`BITRATE_MBPS_OPTIONS`] × 1000; `Some(0)` = clear back to auto
    /// (area-scaled); `None` = left alone. Running pipelines pick a bitrate
    /// change up dynamically. `virtual_display`: macOS headless main-display
    /// capture through a CGVirtualDisplay — applies to pipelines started
    /// after this call, like the encoder/fps overrides.
    /// `None` fields are left alone.
    SetStreamConfig {
        #[serde(default)]
        encoder: Option<String>,
        #[serde(default)]
        fps: Option<u32>,
        #[serde(default)]
        max_resolution: Option<String>,
        #[serde(default)]
        bitrate_kbps: Option<u32>,
        #[serde(default)]
        virtual_display: Option<bool>,
    },
    Shutdown,
}

/// The max-resolution ids `SetStreamConfig` accepts (and the only ones the
/// webui offers): 4K UHD, 2.5K QHD, 2K FHD. Shared by the coordinator's
/// validation and the screen-server's authoritative reject.
pub const MAX_RESOLUTION_IDS: [&str; 3] = ["4k", "2.5k", "2k"];

/// The bitrates (Mbps) the webui's remote-desktop section offers; the only
/// `SetStreamConfig.bitrate_kbps` values the coordinator sends (× 1000).
/// Shared by the coordinator's validation and the webui dropdown.
pub const BITRATE_MBPS_OPTIONS: [u8; 11] = [50, 45, 40, 35, 30, 25, 20, 15, 10, 5, 3];

/// A screen-server → coordinator response; `id` echoes the request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub id: u64,
    #[serde(flatten)]
    pub result: ResponsePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum ResponsePayload {
    Status(Status),
    ConfigApplied {
        paths: Paths,
    },
    Paths(Paths),
    Sources {
        entries: Vec<SourceEntry>,
        #[serde(default)]
        revision: u64,
    },
    OperationAccepted {
        operation_id: u64,
    },
    GrantResult {
        granted: bool,
    },
    /// The `WarmupInput` answer: whether every live desktop stream's input
    /// backend came up (or a dialog is still pending on timeout), plus a
    /// human-readable reason.
    WarmupInput {
        granted: bool,
        message: String,
    },
    /// The `GetEncoders` answer: availability catalog plus the effective
    /// stream tunables (`current` is the encoder id; `fps` is the effective
    /// frame rate — the override when set, else the server default;
    /// `fps_override` exposes the override itself; `max_resolution` is the
    /// override in effect, `None` = the "4k" default; `bitrate_kbps` is the
    /// fixed-bitrate override in effect, `None` = auto area-scaled;
    /// `virtual_display` is the virtual-display override in effect, `None` =
    /// the server's startup default (`--virtual-display` / off).
    Encoders {
        entries: Vec<EncoderBackendInfo>,
        current: String,
        fps: u32,
        fps_override: Option<u32>,
        #[serde(default)]
        max_resolution: Option<String>,
        #[serde(default)]
        bitrate_kbps: Option<u32>,
        #[serde(default)]
        virtual_display: Option<bool>,
    },
    /// Echo of the tunables applied by `SetStreamConfig` (`fps` is the
    /// override now in effect, `None` = server default; same for
    /// `max_resolution`, `None` = "4k"; `bitrate_kbps` is the fixed-bitrate
    /// override in effect, `None` = auto area-scaled; `virtual_display` is
    /// the virtual-display override in effect, `None` = server startup
    /// default).
    StreamConfig {
        encoder: String,
        fps: Option<u32>,
        #[serde(default)]
        max_resolution: Option<String>,
        #[serde(default)]
        bitrate_kbps: Option<u32>,
        #[serde(default)]
        virtual_display: Option<bool>,
    },
    /// The `GetClipboard` answer: the simulator pasteboard contents.
    Clipboard {
        text: String,
    },
    /// The service-management answer (Windows only): whether the service
    /// is registered with the SCM / currently running, plus a
    /// user-presentable message (result text, UAC decline, non-Windows
    /// explanation).
    Service {
        installed: bool,
        running: bool,
        message: String,
    },
    /// The `GetInputStatus` answer (see [`Cmd::GetInputStatus`]).
    InputStatus(InputStatusInfo),
    /// The `ResetScreenConsent` answer: how many persisted portal restore
    /// tokens were removed.
    ScreenConsentReset {
        cleared: u32,
    },
    /// The `GrantScreenConsent` answer: whether the user completed the
    /// portal source picker (and the fresh selection was persisted), plus a
    /// human-readable reason on failure — or a note when the compositor
    /// issued no restore token, in which case the picker will show again on
    /// every connect.
    ScreenConsentGrant {
        granted: bool,
        message: String,
    },
    Ok,
    /// Validation/dispatch failure; the message is user-presentable (the
    /// webui shows it verbatim, §4.3).
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AndroidOperation {
    Boot,
    Shutdown,
}

/// Unsolicited screen-server → coordinator event (spec §3.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    StreamStarted {
        stream_id: u16,
        source: String,
    },
    StreamStopped {
        stream_id: u16,
    },
    StreamError {
        source: String,
        message: String,
    },
    ClientConnected {
        addr: String,
        client_label: String,
    },
    ClientDisconnected {
        addr: String,
    },
    /// A tool path changed usability (e.g. adb hot-swap), or a `SetConfig`
    /// changed the override; carries the fresh resolution.
    PathsChanged {
        paths: Paths,
    },
    SourcesChanged {
        revision: u64,
        entries: Vec<SourceEntry>,
    },
    AndroidOperationCompleted {
        operation_id: u64,
        operation: AndroidOperation,
        target: String,
        serial: Option<String>,
        error: Option<String>,
    },
    /// 5-second rollup of the pipeline stats (the structured form of the
    /// `[stats 5s]` log line).
    Stats(Stats),
}

// ---------------------------------------------------------------------------
// Payload types
// ---------------------------------------------------------------------------

/// The `GetInputStatus` answer: the OS-level input-injection grants. On
/// Linux, whether desktop input can bypass the OS consent path via the
/// kernel uinput module — the webui pairs a missing grant with the one-time
/// setup script (udev rule + `input` group). On macOS, whether the
/// Accessibility grant needed for CGEvent injection is in place.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputStatusInfo {
    /// `/dev/uinput` exists — the kernel module is loaded/available. False
    /// means the setup script's `modprobe` step has not run either.
    pub uinput_exists: bool,
    /// This process can open `/dev/uinput` read-write (the udev/group grant
    /// is in place). On non-Linux hosts both flags are false.
    pub uinput_accessible: bool,
    /// Raw probe error text (e.g. the `std::io::Error` display), empty when
    /// accessible. The webui explains the states itself; this is for logs.
    pub detail: String,
    /// macOS only: whether this process holds the Accessibility (TCC input
    /// control) grant — the real gate for CGEvent input injection (event
    /// creation succeeds without it and posting then silently drops every
    /// event). `None` on other platforms, where it does not apply.
    #[serde(default)]
    pub accessibility_trusted: Option<bool>,
}

/// Where a resolved tool path came from (§4).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PathOrigin {
    Auto,
    Env,
    Override,
}

/// One resolved tool path, as shown in the webui (§4.3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathInfo {
    /// Resolved path (empty when resolution failed).
    pub value: Option<PathBuf>,
    pub origin: PathOrigin,
    pub valid: bool,
    /// Actionable reason when `!valid`; empty on success.
    pub detail: String,
}

impl Default for PathInfo {
    fn default() -> Self {
        PathInfo {
            value: None,
            origin: PathOrigin::Auto,
            valid: false,
            detail: String::new(),
        }
    }
}

impl PathInfo {
    /// `valid: true` placeholder for paths a platform does not use.
    pub fn unavailable(detail: impl Into<String>) -> Self {
        PathInfo {
            value: None,
            origin: PathOrigin::Auto,
            valid: false,
            detail: detail.into(),
        }
    }
}

/// Both tool paths (the `GetPaths` / `SetConfig` payload).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Paths {
    #[serde(default)]
    pub sdk: PathInfo,
    pub adb: PathInfo,
    pub agent_dir: PathInfo,
}

/// Source kinds on the wire (mirrors the video-metadata `source_kind`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Screen,
    Window,
    IosSimulator,
    AndroidDevice,
}

/// Live source status (§5.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SourceStatus {
    Available,
    Busy,
    Streaming,
    Unavailable { reason: String },
}

/// One entry of the `GetEncoders` catalog: a selectable encoder backend and
/// whether it can actually run on this host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncoderBackendInfo {
    /// Stable id used in `SetStreamConfig` ("auto" | "vendor" | "d3d12" | "mf").
    pub id: String,
    /// Human-readable label for the webui dropdown.
    pub label: String,
    pub available: bool,
    /// Why it is unavailable (empty when `available`).
    pub reason: String,
}

/// One entry of the `ListSources` catalog (§5.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceEntry {
    /// Source spec syntax (`main`, `display:N`, `sim:UDID`, `android:S`).
    pub spec: String,
    pub kind: SourceKind,
    pub title: String,
    #[serde(flatten)]
    pub status: SourceStatus,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub primary: bool,
}

/// One live stream in the `GetStatus` answer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveStream {
    pub stream_id: u16,
    /// Source spec this stream captures.
    pub source: String,
    /// Subscribed client addresses (`ip:port` strings).
    pub clients: Vec<String>,
    /// Encoder backend name, once the pipeline reported it.
    #[serde(default)]
    pub encoder: Option<String>,
}

/// The `GetStatus` answer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Status {
    pub udp_port: u16,
    pub active_streams: Vec<ActiveStream>,
    pub uptime_ms: u64,
    pub paths: Paths,
}

/// 5-second pipeline stats rollup (§3.2). Averages are over the window.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Stats {
    pub frames: u64,
    pub access_units: u64,
    /// Mean capture → encode-pickup latency, microseconds.
    pub avg_pickup_us: u64,
    /// Mean encode time, milliseconds.
    pub avg_encode_ms: f64,
    pub delivered: u64,
    pub dropped: u64,
    /// Mean network delivery time (send → full ACK), milliseconds.
    pub avg_net_ms: f64,
    /// Connected client addresses.
    pub clients: Vec<String>,
}

// ---------------------------------------------------------------------------
// Token hex coding (JSON has no byte arrays; hex is debuggable)
// ---------------------------------------------------------------------------

/// 16-byte token → 32 lowercase hex chars.
pub fn token_to_hex(token: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in token {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Parse a hex token; accepts upper/lowercase. `None` when not exactly 16
/// bytes of hex.
pub fn token_from_hex(s: &str) -> Option<[u8; 16]> {
    let s = s.trim();
    if s.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for (i, chunk) in s.as_bytes().chunks_exact(2).enumerate() {
        let hi = (chunk[0] as char).to_digit(16)?;
        let lo = (chunk[1] as char).to_digit(16)?;
        out[i] = ((hi << 4) | lo) as u8;
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// Framing (§3.1): [u32 LE length][UTF-8 JSON]
// ---------------------------------------------------------------------------

/// Serialize `value` and write one length-prefixed frame.
pub fn write_frame(w: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    let body =
        serde_json::to_vec(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if body.len() > MAX_FRAME_LEN as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large to send",
        ));
    }
    w.write_all(&(body.len() as u32).to_le_bytes())?;
    w.write_all(&body)?;
    w.flush()
}

/// Read one frame and deserialize it. `Ok(None)` on a clean EOF at a frame
/// boundary (peer closed); a truncated frame is an error.
pub fn read_frame<T: DeserializeOwned>(r: &mut impl Read) -> io::Result<Option<T>> {
    let mut len_buf = [0u8; 4];
    match r.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_le_bytes(len_buf);
    if len > MAX_FRAME_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame length {len} exceeds the {MAX_FRAME_LEN}-byte cap"),
        ));
    }
    let mut body = vec![0u8; len as usize];
    r.read_exact(&mut body)?;
    let value =
        serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(value))
}

/// A fresh random session id for [`Hello`] (time ⊕ pid ⊕ address entropy;
/// uniqueness across restarts is all it needs).
pub fn new_session_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs.rotate_left(21) ^ nanos.rotate_left(7) ^ (std::process::id() as u64)
}

// ---------------------------------------------------------------------------
// Windows named-pipe frame reader (PeekNamedPipe polling)
// ---------------------------------------------------------------------------

/// Frame reader for Windows named pipes that NEVER issues a long-blocking
/// ReadFile. Required because duplicated synchronous pipe handles share one
/// file object whose operations serialize: an outstanding blocking read
/// would freeze the peer's (and our own) writes on the same pipe. The
/// reader polls with `PeekNamedPipe` and only reads bytes that are already
/// buffered, so writes on either side never wait.
///
/// Usage: call [`Self::poll_frame`] in a loop with a short sleep (50–100
/// ms) between iterations. Unix domain sockets have no such serialization —
/// plain [`read_frame`] is fine there.
#[cfg(target_os = "windows")]
pub struct PipeFrameReader {
    file: std::fs::File,
    buf: Vec<u8>,
}

#[cfg(target_os = "windows")]
impl PipeFrameReader {
    pub fn new(file: std::fs::File) -> Self {
        PipeFrameReader {
            file,
            buf: Vec::with_capacity(8192),
        }
    }

    /// Non-blocking: `Ok(Some(frame))` when a complete frame arrived,
    /// `Ok(None)` when no data is available right now, `Err` on a broken
    /// pipe (peer closed / mid-frame truncation) or a protocol violation.
    pub fn poll_frame<T: DeserializeOwned>(&mut self) -> io::Result<Option<T>> {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::System::Pipes::PeekNamedPipe;

        let mut avail = 0u32;
        let raw = HANDLE(self.file.as_raw_handle() as _);
        match unsafe { PeekNamedPipe(raw, None, 0, None, Some(&mut avail), None) } {
            Ok(()) => {}
            // ERROR_BROKEN_PIPE / ERROR_INVALID_HANDLE: peer gone.
            Err(e) => return Err(io::Error::new(io::ErrorKind::BrokenPipe, e)),
        }
        if avail > 0 {
            let mut chunk = vec![0u8; avail as usize];
            // Exactly `avail` bytes are buffered, so this returns at once.
            self.file.read_exact(&mut chunk)?;
            self.buf.extend_from_slice(&chunk);
        }
        if self.buf.len() < 4 {
            return Ok(None);
        }
        let len = u32::from_le_bytes(self.buf[..4].try_into().unwrap()) as usize;
        if len > MAX_FRAME_LEN as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("frame length {len} exceeds the {MAX_FRAME_LEN}-byte cap"),
            ));
        }
        if self.buf.len() < 4 + len {
            return Ok(None);
        }
        let body: Vec<u8> = self.buf.drain(..4 + len).skip(4).collect();
        let value = serde_json::from_slice(&body)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Some(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_roundtrip() {
        let req = Frame::Request(Request {
            id: 7,
            cmd: Cmd::GrantToken {
                token: token_to_hex(&[0xAB; 16]),
                sources: vec!["main".into(), "sim:U1".into()],
                client_label: "wss-42".into(),
                ttl_ms: 60_000,
                client_addr: Some("192.0.2.10".into()),
            },
        });
        let mut buf = Vec::new();
        write_frame(&mut buf, &req).unwrap();
        let back: Frame = read_frame(&mut &buf[..]).unwrap().unwrap();
        assert_eq!(back, req);
        // Clean EOF at a boundary → None.
        assert!(read_frame::<Frame>(&mut &buf[..0]).unwrap().is_none());
    }

    #[test]
    fn wire_shapes_match_the_spec() {
        // Request: {"id":1,"cmd":"set_config",...}
        let req = Request {
            id: 1,
            cmd: Cmd::SetConfig {
                sdk_path: Some(r"C:\Android\Sdk".into()),
                adb_path: Some(r"C:\adb.exe".into()),
                agent_dir: None,
                clear_sdk: false,
                clear_adb: false,
                clear_agent: true,
            },
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["cmd"], "set_config");
        assert_eq!(json["sdk_path"], r"C:\Android\Sdk");
        assert_eq!(json["adb_path"], r"C:\adb.exe");
        assert_eq!(json["clear_agent"], true);

        // Event: {"event":"stream_started",...} with no id.
        let ev = Event::StreamStarted {
            stream_id: 2,
            source: "display:1".into(),
        };
        let json = serde_json::to_value(&ev).unwrap();
        assert_eq!(json["event"], "stream_started");
        assert!(json.get("id").is_none());

        let ev = Event::SourcesChanged {
            revision: 7,
            entries: Vec::new(),
        };
        let json = serde_json::to_value(&ev).unwrap();
        assert_eq!(json["event"], "sources_changed");
        assert_eq!(json["revision"], 7);
        assert!(json.get("id").is_none());

        let response = ResponsePayload::OperationAccepted { operation_id: 11 };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["result"], "operation_accepted");
        assert_eq!(json["operation_id"], 11);

        let legacy: ResponsePayload = serde_json::from_value(serde_json::json!({
            "result": "sources",
            "entries": []
        }))
        .unwrap();
        assert!(matches!(
            legacy,
            ResponsePayload::Sources { revision: 0, .. }
        ));

        // SourceEntry flattens its status tag: {"spec":…, "status":"streaming"}.
        let entry = SourceEntry {
            spec: "display:0".into(),
            kind: SourceKind::Screen,
            title: "DISPLAY1".into(),
            status: SourceStatus::Streaming,
            width: Some(3840),
            height: Some(2160),
            primary: true,
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["status"], "streaming");
        assert_eq!(json["kind"], "screen");
        let back: SourceEntry = serde_json::from_value(json).unwrap();
        assert_eq!(back, entry);

        // Response echoes the id and tags the payload.
        let resp = Response {
            id: 9,
            result: ResponsePayload::Error {
                message: "nope".into(),
            },
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["id"], 9);
        assert_eq!(json["result"], "error");
    }

    #[test]
    fn frame_decode_untagged() {
        let hello: Frame = serde_json::from_str(
            r#"{"hello":{"protocol":1,"version":"0.1.0","pid":123,"session_id":5,"udp_port":36459,"capabilities":["v4"]}}"#,
        )
        .unwrap();
        assert!(matches!(hello, Frame::Hello { .. }));

        let ev: Frame =
            serde_json::from_str(r#"{"event":"stream_stopped","stream_id":1}"#).unwrap();
        assert!(matches!(
            ev,
            Frame::Event(Event::StreamStopped { stream_id: 1 })
        ));
    }

    #[test]
    fn token_hex_roundtrip() {
        let token = [0xC3u8; 16];
        let hex = token_to_hex(&token);
        assert_eq!(hex.len(), 32);
        assert_eq!(token_from_hex(&hex), Some(token));
        assert_eq!(token_from_hex(&hex.to_uppercase()), Some(token));
        assert_eq!(token_from_hex("abc"), None);
        assert_eq!(token_from_hex(&"zz".repeat(16)), None);
    }

    #[test]
    fn oversized_frame_rejected() {
        let mut buf = (MAX_FRAME_LEN + 1).to_le_bytes().to_vec();
        buf.extend_from_slice(&[0u8; 8]);
        let err = read_frame::<Frame>(&mut &buf[..]).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn warmup_input_roundtrip() {
        // Request shape: a bare unit command on the wire.
        let req = Frame::Request(Request {
            id: 21,
            cmd: Cmd::WarmupInput,
        });
        let mut buf = Vec::new();
        write_frame(&mut buf, &req).unwrap();
        assert_eq!(read_frame::<Frame>(&mut &buf[..]).unwrap(), Some(req));
        let json = serde_json::to_value(&Request {
            id: 21,
            cmd: Cmd::WarmupInput,
        })
        .unwrap();
        assert_eq!(json["cmd"], "warmup_input");

        // Answer shape: granted + human-readable reason.
        let res = Response {
            id: 21,
            result: ResponsePayload::WarmupInput {
                granted: false,
                message: "timed out".into(),
            },
        };
        let mut buf = Vec::new();
        write_frame(&mut buf, &res).unwrap();
        assert_eq!(read_frame::<Response>(&mut &buf[..]).unwrap(), Some(res));
    }

    #[test]
    fn grant_screen_consent_roundtrip() {
        // Request shape: a bare unit command on the wire.
        let req = Frame::Request(Request {
            id: 22,
            cmd: Cmd::GrantScreenConsent,
        });
        let mut buf = Vec::new();
        write_frame(&mut buf, &req).unwrap();
        assert_eq!(read_frame::<Frame>(&mut &buf[..]).unwrap(), Some(req));
        let json = serde_json::to_value(&Request {
            id: 22,
            cmd: Cmd::GrantScreenConsent,
        })
        .unwrap();
        assert_eq!(json["cmd"], "grant_screen_consent");

        // Answer shape: granted + reason (empty when the token persisted).
        let res = Response {
            id: 22,
            result: ResponsePayload::ScreenConsentGrant {
                granted: true,
                message: String::new(),
            },
        };
        let mut buf = Vec::new();
        write_frame(&mut buf, &res).unwrap();
        assert_eq!(read_frame::<Response>(&mut &buf[..]).unwrap(), Some(res));
    }

    #[test]
    fn input_status_roundtrip() {
        // Request: bare unit command, tagged snake_case.
        let req = Frame::Request(Request {
            id: 41,
            cmd: Cmd::GetInputStatus,
        });
        let mut buf = Vec::new();
        write_frame(&mut buf, &req).unwrap();
        assert_eq!(read_frame::<Frame>(&mut &buf[..]).unwrap(), Some(req));
        let json = serde_json::to_value(&Request {
            id: 41,
            cmd: Cmd::GetInputStatus,
        })
        .unwrap();
        assert_eq!(json["cmd"], "get_input_status");

        // Answer: the uinput probe flags + raw detail.
        let res = Response {
            id: 41,
            result: ResponsePayload::InputStatus(InputStatusInfo {
                uinput_exists: true,
                uinput_accessible: false,
                detail: "Permission denied (os error 13)".into(),
                accessibility_trusted: None,
            }),
        };
        let json = serde_json::to_value(&res).unwrap();
        // Internally-tagged payload: the struct's fields sit next to the
        // `result` tag, not nested under it.
        assert_eq!(json["result"], "input_status");
        assert_eq!(json["uinput_accessible"], false);
        assert_eq!(json["uinput_exists"], true);
        let mut buf = Vec::new();
        write_frame(&mut buf, &res).unwrap();
        assert_eq!(read_frame::<Response>(&mut &buf[..]).unwrap(), Some(res));
    }

    #[test]
    fn screen_consent_reset_roundtrip() {
        let req = Frame::Request(Request {
            id: 51,
            cmd: Cmd::ResetScreenConsent,
        });
        let mut buf = Vec::new();
        write_frame(&mut buf, &req).unwrap();
        assert_eq!(read_frame::<Frame>(&mut &buf[..]).unwrap(), Some(req));
        let json = serde_json::to_value(&Request {
            id: 51,
            cmd: Cmd::ResetScreenConsent,
        })
        .unwrap();
        assert_eq!(json["cmd"], "reset_screen_consent");

        let res = Response {
            id: 51,
            result: ResponsePayload::ScreenConsentReset { cleared: 2 },
        };
        let json = serde_json::to_value(&res).unwrap();
        assert_eq!(json["result"], "screen_consent_reset");
        assert_eq!(json["cleared"], 2);
    }

    #[test]
    fn service_messages_roundtrip() {
        // Unit commands on the wire, tagged snake_case.
        for (id, cmd, tag) in [
            (31, Cmd::ServiceStatus, "service_status"),
            (32, Cmd::ServiceInstall, "service_install"),
            (33, Cmd::ServiceRemove, "service_remove"),
        ] {
            let req = Frame::Request(Request {
                id,
                cmd: cmd.clone(),
            });
            let mut buf = Vec::new();
            write_frame(&mut buf, &req).unwrap();
            assert_eq!(read_frame::<Frame>(&mut &buf[..]).unwrap(), Some(req));
            let json = serde_json::to_value(&Request { id, cmd }).unwrap();
            assert_eq!(json["cmd"], tag);
        }

        // Answer shape: installed/running + user-presentable message.
        let res = Response {
            id: 31,
            result: ResponsePayload::Service {
                installed: true,
                running: false,
                message: "service installed".into(),
            },
        };
        let mut buf = Vec::new();
        write_frame(&mut buf, &res).unwrap();
        assert_eq!(read_frame::<Response>(&mut &buf[..]).unwrap(), Some(res));
        let json = serde_json::to_value(&Response {
            id: 31,
            result: ResponsePayload::Service {
                installed: true,
                running: false,
                message: "service installed".into(),
            },
        })
        .unwrap();
        // `Response` flattens the payload: the tag and fields merge into
        // the top level.
        assert_eq!(json["result"], "service");
        assert_eq!(json["installed"], true);
        assert_eq!(json["running"], false);
    }

    #[test]
    fn stream_config_messages_roundtrip() {
        let req = Frame::Request(Request {
            id: 11,
            cmd: Cmd::SetStreamConfig {
                encoder: Some("vendor".into()),
                fps: Some(30),
                max_resolution: Some("2.5k".into()),
                bitrate_kbps: None,
                virtual_display: None,
            },
        });
        let mut buf = Vec::new();
        write_frame(&mut buf, &req).unwrap();
        assert_eq!(read_frame::<Frame>(&mut &buf[..]).unwrap(), Some(req));

        // Wire shape: {"id":11,"cmd":"set_stream_config","encoder":"vendor","fps":30}.
        let json = serde_json::to_value(&Request {
            id: 11,
            cmd: Cmd::SetStreamConfig {
                encoder: Some("d3d12".into()),
                fps: Some(60),
                max_resolution: None,
                bitrate_kbps: None,
                virtual_display: Some(true),
            },
        })
        .unwrap();
        assert_eq!(json["cmd"], "set_stream_config");
        assert_eq!(json["encoder"], "d3d12");
        assert_eq!(json["fps"], 60);
        assert_eq!(json["virtual_display"], true);
        // Absent fields deserialize as None (partial update semantics).
        let back: Request =
            serde_json::from_str(r#"{"id":12,"cmd":"set_stream_config","fps":15}"#).unwrap();
        assert_eq!(
            back.cmd,
            Cmd::SetStreamConfig {
                encoder: None,
                fps: Some(15),
                max_resolution: None,
                bitrate_kbps: None,
                virtual_display: None,
            }
        );

        // GetEncoders answer shape.
        let resp = Response {
            id: 13,
            result: ResponsePayload::Encoders {
                entries: vec![EncoderBackendInfo {
                    id: "vendor".into(),
                    label: "GPU vendor encoder (NVENC/AMF/QSV)".into(),
                    available: false,
                    reason: "no discrete GPU".into(),
                }],
                current: "auto".into(),
                fps: 60,
                fps_override: Some(30),
                max_resolution: Some("2k".into()),
                bitrate_kbps: Some(20_000),
                virtual_display: Some(true),
            },
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["result"], "encoders");
        assert_eq!(json["entries"][0]["id"], "vendor");
        assert_eq!(json["fps_override"], 30);
        assert_eq!(json["bitrate_kbps"], 20_000);
        assert_eq!(json["max_resolution"], "2k");
        assert_eq!(json["virtual_display"], true);
        let back: Response = serde_json::from_value(json).unwrap();
        assert_eq!(back, resp);

        // SetStreamConfig echo shape.
        let resp = Response {
            id: 14,
            result: ResponsePayload::StreamConfig {
                encoder: "mf".into(),
                fps: None,
                max_resolution: None,
                bitrate_kbps: None,
                virtual_display: None,
            },
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["result"], "stream_config");
        assert_eq!(json["encoder"], "mf");
    }

    #[test]
    fn simulator_control_commands_roundtrip() {
        for cmd in [
            Cmd::BootSimulator {
                udid: "ABCD-1234".into(),
            },
            Cmd::ShutdownSimulator {
                udid: "ABCD-1234".into(),
            },
            Cmd::BootAndroidEmulator {
                avd: "Medium_Phone_API_36.0".into(),
            },
            Cmd::ShutdownAndroidEmulator {
                serial: "emulator-5554".into(),
            },
            Cmd::RotateAndroidDevice {
                serial: "emulator-5554".into(),
            },
        ] {
            let frame = Frame::Request(Request { id: 3, cmd });
            let mut buf = Vec::new();
            write_frame(&mut buf, &frame).unwrap();
            assert_eq!(read_frame::<Frame>(&mut &buf[..]).unwrap(), Some(frame));
        }
        // Wire shape: {"id":3,"cmd":"boot_simulator","udid":"..."}.
        let json = serde_json::to_value(&Request {
            id: 3,
            cmd: Cmd::BootSimulator { udid: "U".into() },
        })
        .unwrap();
        assert_eq!(json["cmd"], "boot_simulator");
        assert_eq!(json["udid"], "U");
    }
}
