use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[cfg(DEVELOPMENT)]
pub const DEFAULT_CONFIG_PORT: u16 = 5181;
#[cfg(not(DEVELOPMENT))]
pub const DEFAULT_CONFIG_PORT: u16 = 5180;
#[cfg(DEVELOPMENT)]
pub const DEFAULT_RELAY_PORT: u16 = 5154;
#[cfg(not(DEVELOPMENT))]
pub const DEFAULT_RELAY_PORT: u16 = vterm_common::DEFAULT_SERVER_PORT;
pub const DEFAULT_PROXY_PORT_START: u16 = 62000;
pub const DEFAULT_PROXY_PORT_END: u16 = 62999;
/// Default auto-disconnect (idle timeout) for port forwards, in minutes.
/// `0` means "never".
pub const DEFAULT_PROXY_IDLE_TIMEOUT_MINUTES: u32 = 0;
pub const DEFAULT_STATIC_HTTP_PORT_START: u16 = 63000;
pub const DEFAULT_STATIC_HTTP_PORT_END: u16 = 63999;
pub const DEFAULT_RELAY_UDP_PORT_START: u16 = 36500;
pub const DEFAULT_RELAY_UDP_PORT_END: u16 = 36599;
// WSL first-startup defaults. WSL shares the Windows host's network namespace,
// so the standard ranges tend to collide with host services. Applied only when
// a config is first generated under WSL (see apply_platform_default_ports).
const WSL_DEFAULT_RELAY_PORT: u16 = 15153;
const WSL_DEFAULT_CONFIG_PORT: u16 = 15180;
const WSL_DEFAULT_MCP_PORT: u16 = 16180;
const WSL_DEFAULT_STATIC_HTTP_PORT_START: u16 = 53000;
const WSL_DEFAULT_STATIC_HTTP_PORT_END: u16 = 53999;
const WSL_DEFAULT_PROXY_PORT_START: u16 = 52000;
const WSL_DEFAULT_PROXY_PORT_END: u16 = 52999;
/// v2: one-time flip of `mcp.enabled` to the new default (`true`) for configs
/// persisted while the default was `false` (see
/// `store::migrate_mcp_default_enabled`).
pub const CONFIG_VERSION: u32 = 2;
const SECRET_KEY_ENCRYPTED_PREFIX_V2: &str = "enc-v2:";
const SECRET_KEY_HASHED_PREFIX: &str = "sha256:";
const SECRET_KEY_NONCE_SIZE: usize = 12;
const SECRET_KEY_KEY_CONTEXT: &[u8] = b"vibe-terminal/server-config-secret-key/v2";
const ENV_VAR_KEY_CONTEXT: &[u8] = b"vibe-terminal/env-var-encryption/v1";
const ADDON_CONFIG_KEY_CONTEXT: &[u8] = b"vibe-terminal/addon-config/v1";
const DEFAULT_UART_TERMINAL_TYPE: &str = "vt100";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TerminalServerConfig {
    pub version: u32,
    pub server: ServerConfig,
    pub device: DeviceConfig,
    pub uart: UartConfig,
    pub ddns: DdnsConfig,
    pub notification: NotificationConfig,
    #[serde(alias = "terminal_observer")]
    pub terminal: TerminalConfig,
    #[serde(default)]
    pub environment: EnvironmentConfig,
    #[serde(default)]
    pub addons: AddonsConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub asr: AsrConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub remote_desktop: RemoteDesktopConfig,
}

impl Default for TerminalServerConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            server: ServerConfig::default(),
            device: DeviceConfig::default(),
            uart: UartConfig::default(),
            ddns: DdnsConfig::default(),
            notification: NotificationConfig::default(),
            terminal: TerminalConfig::default(),
            environment: EnvironmentConfig::default(),
            addons: AddonsConfig::default(),
            llm: LlmConfig::default(),
            asr: AsrConfig::default(),
            mcp: McpConfig::default(),
            remote_desktop: RemoteDesktopConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsrConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub load_on_startup: bool,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub provider_config: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub model_path: String,
    /// Identifier of the selected model in the local ASR catalog.
    #[serde(default)]
    pub model_id: String,
    /// Selected model path relative to `model_path` (the model root directory).
    #[serde(default)]
    pub model_relative_path: String,
    #[serde(default)]
    pub device: String,
    #[serde(default)]
    pub input_device: String,
    /// Bind address for the locally-managed vterm-asr-server process.
    /// Defaults to the loopback interface to avoid exposing the ASR service
    /// to the local network.
    #[serde(default = "default_asr_bind_address")]
    pub bind_address: String,
    /// Preferred port for the locally-managed vterm-asr-server process.
    #[serde(default = "default_asr_port")]
    pub port: u16,
    /// Optional bearer token used to authenticate requests to the local
    /// vterm-asr-server. Stored at rest as `enc-v2:...` (same envelope as
    /// addon secrets). Empty means no authentication.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}

/// Default bind address for the locally-managed ASR server.
pub fn default_asr_bind_address() -> String {
    "127.0.0.1".to_string()
}

/// Default port for the locally-managed ASR server. Matches the historical
/// default that `vterm-asr-server` ships with.
pub const DEFAULT_ASR_PORT: u16 = 8398;

pub fn default_asr_port() -> u16 {
    DEFAULT_ASR_PORT
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            load_on_startup: false,
            provider: String::new(),
            provider_config: std::collections::HashMap::new(),
            model_path: String::new(),
            model_id: String::new(),
            model_relative_path: String::new(),
            device: String::new(),
            input_device: String::new(),
            bind_address: default_asr_bind_address(),
            port: default_asr_port(),
            access_token: None,
        }
    }
}

impl AsrConfig {
    /// Returns the decrypted bearer token for the local ASR server, or `None`
    /// when no token is configured.
    ///
    /// The token has two possible sources, checked in order:
    /// 1. `access_token` — set via the Local ASR Server card in the WebUI.
    ///    Stored encrypted (`enc-v2:...`); decrypted here.
    /// 2. `provider_config["access_token"]` — the vibe-voice addon's own
    ///    field. This is what the addon itself reads (via
    ///    `addon.get_config("access_token")`). When the user edits it
    ///    directly (e.g. for a remote server), the supervisor picks it up
    ///    here as a fallback so the supervised process is spawned with the
    ///    same token the addon will present.
    ///
    /// This dual-source resolution keeps the two token entry points in
    /// agreement without forcing a mirror write on every config update: the
    /// Local card's token wins; if it's absent, the addon's token is used.
    pub fn effective_token(&self) -> Option<String> {
        // 1. Canonical: asr.access_token (Local ASR Server card).
        if let Some(raw) = &self.access_token {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                if let Ok(decrypted) = decrypt_addon_secret(trimmed) {
                    if !decrypted.is_empty() {
                        return Some(decrypted);
                    }
                }
                // Fall through if decryption failed (tolerate plain values).
            }
        }
        // 2. Fallback: provider_config["access_token"] (addon field).
        if let Some(t) = self.provider_config.get("access_token") {
            let trimmed = t.trim();
            if !trimmed.is_empty() {
                return Some(decrypt_addon_secret(trimmed).unwrap_or_else(|_| trimmed.to_string()));
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServerConfig {
    #[serde(default)]
    pub server_id: String,
    pub relay_port: u16,
    pub config_port: u16,
    pub proxy_port_start: u16,
    pub proxy_port_end: u16,
    pub proxy_enabled: bool,
    pub proxy_idle_timeout_minutes: u32,
    pub static_http_port_start: u16,
    pub static_http_port_end: u16,
    pub relay_udp_port_start: u16,
    pub relay_udp_port_end: u16,
    #[serde(default)]
    pub static_http_bind_address: StaticHttpBindAddress,
    pub name: String,
    pub relay_address: String,
    #[serde(
        default,
        serialize_with = "serialize_server_secret_key",
        deserialize_with = "deserialize_server_secret_key"
    )]
    pub secret_key: String,
    #[serde(
        default,
        serialize_with = "serialize_server_secret_key",
        deserialize_with = "deserialize_server_secret_key"
    )]
    pub relay_secret_key: String,
    pub relay_strict_tls: bool,
    pub relay_ca_pem: String,
    pub cert_path: String,
    pub key_path: String,
    #[serde(default)]
    pub tls_extra_sans: Vec<String>,
    /// Mirror tracing output into per-run log files under
    /// `~/.vibeterminal/logs/vterm-server`. Read once at startup (before
    /// tracing init), so toggles apply on the next launch.
    pub log_to_file: bool,
    /// Start vterm-server automatically at login (Linux: systemd user
    /// service). Applied at toggle time by the config API, read here only
    /// for persistence.
    #[serde(default)]
    pub autostart: bool,
    /// Trust loopback / own-interface peers: skip the secret-key challenge
    /// and the message-encryption requirement for them (the zero-config
    /// local companion flow). Public deployments fronted by a same-host
    /// reverse proxy MUST set this to false — the proxy makes every remote
    /// peer appear local, which would bypass authentication entirely
    /// (security review §4-D).
    #[serde(default = "default_trust_local_peers")]
    pub trust_local_peers: bool,
}

/// Desktop builds keep file logging on by default; headless builds
/// (`--no-default-features`, typically systemd-managed servers where the
/// journal already captures stdout) default to off.
pub fn default_log_to_file() -> bool {
    cfg!(feature = "ui")
}

/// Desktop builds ship port forwarding on by default; headless builds
/// (`--no-default-features`) default to off — a server without a desktop is
/// typically exposed to more networks, so listeners are not opened until an
/// operator enables the feature in the webui.
pub fn default_proxy_enabled() -> bool {
    cfg!(feature = "ui")
}

/// Desktop builds ship remote desktop on by default; headless builds
/// (`--no-default-features`, no display to capture) default to off.
pub fn default_remote_desktop_enabled() -> bool {
    cfg!(feature = "ui")
}

/// Local peers (loopback / own interface addresses) are trusted by default:
/// the companion-server zero-config flow relies on it. See
/// [`ServerConfig::trust_local_peers`].
pub fn default_trust_local_peers() -> bool {
    true
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server_id: String::new(),
            relay_port: DEFAULT_RELAY_PORT,
            config_port: DEFAULT_CONFIG_PORT,
            proxy_port_start: DEFAULT_PROXY_PORT_START,
            proxy_port_end: DEFAULT_PROXY_PORT_END,
            proxy_enabled: default_proxy_enabled(),
            proxy_idle_timeout_minutes: DEFAULT_PROXY_IDLE_TIMEOUT_MINUTES,
            static_http_port_start: DEFAULT_STATIC_HTTP_PORT_START,
            static_http_port_end: DEFAULT_STATIC_HTTP_PORT_END,
            relay_udp_port_start: DEFAULT_RELAY_UDP_PORT_START,
            relay_udp_port_end: DEFAULT_RELAY_UDP_PORT_END,
            static_http_bind_address: StaticHttpBindAddress::default(),
            name: String::new(),
            relay_address: String::new(),
            secret_key: generate_server_secret_key(),
            relay_secret_key: String::new(),
            relay_strict_tls: false,
            relay_ca_pem: String::new(),
            cert_path: String::new(),
            key_path: String::new(),
            tls_extra_sans: Vec::new(),
            log_to_file: default_log_to_file(),
            autostart: false,
            trust_local_peers: default_trust_local_peers(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum StaticHttpBindAddress {
    #[default]
    Local,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DeviceConfig {
    pub serial_port: String,
    pub serial_baud_rate: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial_encoding: Option<String>,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            serial_port: String::new(),
            serial_baud_rate: 115_200,
            serial_encoding: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UartParity {
    #[default]
    None,
    Odd,
    Even,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UartStopBits {
    #[default]
    One,
    Two,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UartFlowControl {
    #[default]
    None,
    Software,
    Hardware,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct UartPortConfig {
    pub id: String,
    pub name: String,
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: UartStopBits,
    pub parity: UartParity,
    pub flow_control: UartFlowControl,
    #[serde(default = "default_uart_terminal_type")]
    pub terminal_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

impl Default for UartPortConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            port: String::new(),
            baud_rate: 115_200,
            data_bits: 8,
            stop_bits: UartStopBits::default(),
            parity: UartParity::default(),
            flow_control: UartFlowControl::default(),
            terminal_type: default_uart_terminal_type(),
            encoding: None,
        }
    }
}

pub(crate) fn default_uart_terminal_type() -> String {
    DEFAULT_UART_TERMINAL_TYPE.to_string()
}

pub(crate) fn is_supported_uart_terminal_type(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "linux" | "vt100" | "vt220" | "xterm" | "ansi"
    )
}

pub(crate) fn normalize_uart_terminal_type(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase();
    if is_supported_uart_terminal_type(&normalized) {
        normalized
    } else {
        default_uart_terminal_type()
    }
}

pub(crate) fn normalize_terminal_encoding(value: Option<&str>) -> Option<String> {
    let value = value?.trim().to_ascii_lowercase();
    matches!(
        value.as_str(),
        "utf8"
            | "utf16le"
            | "utf16be"
            | "gbk"
            | "gb18030"
            | "gb2312"
            | "cp950"
            | "big5hkscs"
            | "shiftjis"
            | "eucjp"
            | "euckr"
            | "windows1252"
            | "iso88591"
            | "iso88593"
            | "iso885915"
            | "macroman"
            | "windows1250"
            | "iso88592"
            | "cp852"
            | "windows1257"
            | "iso88594"
            | "iso885914"
            | "iso885910"
            | "cp865"
            | "iso885913"
            | "windows1251"
            | "koi8r"
            | "koi8u"
            | "koi8ru"
            | "iso88595"
            | "cp866"
            | "cp1125"
            | "koi8t"
            | "windows1253"
            | "iso88597"
            | "windows1254"
            | "iso88599"
            | "cp857"
            | "windows1255"
            | "iso88598"
            | "windows1256"
            | "iso88596"
            | "windows874"
            | "iso885911"
            | "iso885916"
            | "windows1258"
            | "cp437"
            | "cp850"
    )
    .then_some(value)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct UartConfig {
    pub ports: Vec<UartPortConfig>,
}

impl Default for UartConfig {
    fn default() -> Self {
        Self { ports: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DdnsConfig {
    pub enabled: bool,
    pub provider: Option<String>,
    #[serde(default)]
    pub polling_interval_minutes: u32,
    #[serde(default = "default_ddns_ip_family")]
    pub ip_family: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_interface: Option<String>,
    #[serde(default)]
    pub user_config: std::collections::HashMap<String, String>,
}

fn default_ddns_ip_family() -> String {
    "ipv4".to_string()
}

impl Default for DdnsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: None,
            polling_interval_minutes: 5,
            ip_family: default_ddns_ip_family(),
            network_interface: None,
            user_config: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AddonsConfig {
    #[serde(default)]
    pub entries: std::collections::HashMap<String, AddonEntryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddonEntryConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub user_config: std::collections::HashMap<String, String>,
}

impl Default for AddonEntryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            user_config: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LlmProviderConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default)]
    pub user_config: std::collections::HashMap<String, String>,
    #[serde(default = "default_max_failures")]
    pub max_failures: u16,
    /// Seconds to wait before retrying the LLM after it has been disabled by
    /// consecutive failures. Once the cooldown elapses, the next analysis
    /// retries the LLM once; on success it is re-enabled, on failure the
    /// cooldown restarts.
    #[serde(default = "default_retry_cooldown_secs")]
    pub retry_cooldown_secs: u32,
}

fn default_max_failures() -> u16 {
    3
}

fn default_retry_cooldown_secs() -> u32 {
    300
}

impl Default for LlmProviderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: None,
            user_config: std::collections::HashMap::new(),
            max_failures: default_max_failures(),
            retry_cooldown_secs: default_retry_cooldown_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CodeAnalysisLlmConfig {
    #[serde(flatten)]
    pub provider: LlmProviderConfig,
    #[serde(default = "default_output_language")]
    pub output_language: String,
}

fn default_output_language() -> String {
    "en".to_string()
}

impl Default for CodeAnalysisLlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProviderConfig::default(),
            output_language: default_output_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TranslationLlmConfig {
    #[serde(flatten)]
    pub provider: LlmProviderConfig,
    #[serde(default = "default_output_language")]
    pub output_language: String,
}

impl Default for TranslationLlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProviderConfig::default(),
            output_language: default_output_language(),
        }
    }
}

fn default_auto_output_language() -> String {
    "auto".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TerminalLlmConfig {
    #[serde(flatten)]
    pub provider: LlmProviderConfig,
    #[serde(default = "default_auto_output_language")]
    pub output_language: String,
}

impl Default for TerminalLlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProviderConfig::default(),
            output_language: default_auto_output_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GitLlmConfig {
    #[serde(flatten)]
    pub provider: LlmProviderConfig,
    #[serde(default = "default_auto_output_language")]
    pub output_language: String,
}

impl Default for GitLlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProviderConfig::default(),
            output_language: default_auto_output_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LlmConfig {
    pub terminal: TerminalLlmConfig,
    pub git: GitLlmConfig,
    /// LLM for AI conflict resolution (rebase / merge conflicts). Separate
    /// from `git` (commit messages) so a slower reasoning model or a bigger
    /// output budget can be assigned to conflict resolution alone.
    pub git_conflict: GitLlmConfig,
    pub chat: LlmProviderConfig,
    pub voice_polish: LlmProviderConfig,
    pub code_analysis: CodeAnalysisLlmConfig,
    pub translation: TranslationLlmConfig,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            terminal: TerminalLlmConfig::default(),
            git: GitLlmConfig::default(),
            git_conflict: GitLlmConfig::default(),
            chat: LlmProviderConfig::default(),
            voice_polish: LlmProviderConfig::default(),
            code_analysis: CodeAnalysisLlmConfig::default(),
            translation: TranslationLlmConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    #[default]
    Apns,
    Fcm,
    Tpns,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationTargetConfig {
    pub id: String,
    pub channel: NotificationChannel,
    pub device_token: String,
    pub topic: Option<String>,
    pub enabled: bool,
}

impl Default for NotificationTargetConfig {
    fn default() -> Self {
        Self {
            id: "default-device".to_string(),
            channel: NotificationChannel::Apns,
            device_token: String::new(),
            topic: None,
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ApnsNotificationConfig {
    pub enabled: bool,
    pub certificate_path: Option<String>,
    pub certificate_password: Option<String>,
    pub environment: String,
    pub topic: Option<String>,
}

impl Default for ApnsNotificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            certificate_path: None,
            certificate_password: None,
            environment: "sandbox".to_string(),
            topic: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct FcmNotificationConfig {
    pub enabled: bool,
    pub service_account_json: Option<String>,
    pub project_id: Option<String>,
}

impl Default for FcmNotificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            service_account_json: None,
            project_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TpnsNotificationConfig {
    pub enabled: bool,
    pub access_id: Option<String>,
    pub secret_key: Option<String>,
}

impl Default for TpnsNotificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            access_id: None,
            secret_key: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub queue_capacity: u16,
    pub max_retries: u8,
    pub notify_on_startup: bool,
    pub targets: Vec<NotificationTargetConfig>,
    pub apns: ApnsNotificationConfig,
    pub fcm: FcmNotificationConfig,
    pub tpns: TpnsNotificationConfig,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            queue_capacity: 128,
            max_retries: 3,
            notify_on_startup: false,
            targets: Vec::new(),
            apns: ApnsNotificationConfig::default(),
            fcm: FcmNotificationConfig::default(),
            tpns: TpnsNotificationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct McpConfig {
    pub enabled: bool,
    pub port: u16,
    pub bind_address: String,
    pub access_token: String,
    pub stream_throttle_ms: u32,
    /// Per-app MCP setup templates, keyed by app id. A user entry with a
    /// builtin id overrides that builtin; other ids define new apps.
    pub templates: std::collections::HashMap<String, McpTemplateConfig>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            // On by default. Loopback requests are always allowed; remote
            // requests must present the bearer token, so a fresh install
            // ships a random one and is usable locally without setup while
            // admitting nothing from the network. The token must never be
            // empty — an empty token rejects all remote requests (see
            // mcp::auth) and the config API refuses to store it.
            enabled: true,
            port: 6180,
            bind_address: "127.0.0.1".to_string(),
            access_token: generate_random_token(32),
            stream_throttle_ms: 100,
            templates: std::collections::HashMap::new(),
        }
    }
}

/// User-editable MCP setup template for one agent CLI. The local MCP endpoint
/// URL derives from `McpConfig.port`; the bearer token is always
/// `McpConfig.access_token`. Neither is per-template overridable.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct McpTemplateConfig {
    pub display_name: String,
    pub executable: String,
    /// Setup template. Placeholders: `{executable}`, `{url}`, `{token}`;
    /// the conditional block `{#token}...{/token}` is included only when the
    /// server access token is non-empty.
    pub template: String,
    /// Optional setup notes shown in the webui for apps whose flow needs
    /// more than the template line itself (chat-prompt setup, environment
    /// prerequisites, extra steps). Empty for plain-CLI apps.
    pub description: String,
}

/// Remote-desktop streaming configuration. When enabled, a connected remote
/// client can view the host desktop and inject input. Whether the OS actually
/// permits capture is surfaced separately via the API as a runtime fact
/// (`platform` / `requires_permission` / `permission_granted`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RemoteDesktopConfig {
    pub enabled: bool,
    /// Legacy adb executable override retained for config migration. New
    /// clients persist `android_sdk_path` instead.
    pub adb_path: Option<String>,
    /// Manual Android SDK root override (None = auto-detect). adb, emulator,
    /// and avdmanager are resolved below this directory.
    pub android_sdk_path: Option<String>,
    /// Manual Android Studio (screen-sharing-agent) directory override
    /// (None = auto-detect).
    pub android_studio_path: Option<String>,
    /// Encoder backend for screen streams ("auto" | "vendor" | "d3d12" |
    /// "mf"; None = auto). Pushed to the screen-server via ctl
    /// `SetStreamConfig`.
    pub screen_encoder: Option<String>,
    /// Capture/encode frame rate for screen streams (15 | 30 | 60;
    /// None = 60).
    pub screen_fps: Option<u32>,
    /// Capture-resolution ceiling ("4k" | "2.5k" | "2k"; None = "4k").
    /// Pushed to the screen-server via ctl `SetStreamConfig`; sizes the
    /// macOS virtual-display source.
    pub screen_max_resolution: Option<String>,
    /// Fixed average bitrate for screen streams in Mbps (one of the ctl
    /// `BITRATE_MBPS_OPTIONS`; None = auto — area-scaled). Pushed to the
    /// screen-server via ctl `SetStreamConfig`; a client-side
    /// `VideoQuality` request still takes priority on the server.
    pub screen_bitrate_mbps: Option<u8>,
    /// macOS, headless hosts: capture the main display through a
    /// CGVirtualDisplay instead of the physical panel. Pushed to the
    /// screen-server via ctl `SetStreamConfig`; applies to streams started
    /// after the push (reconnect to pick it up). Ignored on non-macOS hosts.
    pub screen_virtual_display: bool,
}

impl Default for RemoteDesktopConfig {
    fn default() -> Self {
        Self {
            enabled: default_remote_desktop_enabled(),
            adb_path: None,
            android_sdk_path: None,
            android_studio_path: None,
            screen_encoder: None,
            screen_fps: None,
            screen_max_resolution: None,
            screen_bitrate_mbps: None,
            screen_virtual_display: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminalObserverPatternKind {
    #[default]
    Confirmation,
    Selection,
    Completion,
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TerminalObserverPatternConfig {
    pub id: String,
    pub kind: TerminalObserverPatternKind,
    pub substrings: Vec<String>,
    pub enabled: bool,
}

impl Default for TerminalObserverPatternConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: TerminalObserverPatternKind::default(),
            substrings: Vec::new(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TerminalEnvVar {
    pub name: String,
    pub value: String,
    pub enabled: bool,
}

impl Default for TerminalEnvVar {
    fn default() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TerminalConfig {
    pub enabled: bool,
    pub auto_confirm: bool,
    pub analysis_delay_seconds: u16,
    pub confirmation_grace_seconds: u16,
    pub notify_on_completion: bool,
    pub patterns: Vec<TerminalObserverPatternConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_provider: Option<String>,
    pub env_vars: Vec<TerminalEnvVar>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_confirm: true,
            analysis_delay_seconds: 3,
            confirmation_grace_seconds: 30,
            notify_on_completion: true,
            patterns: Vec::new(),
            llm_provider: None,
            env_vars: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvironmentVariable {
    pub key: String,
    #[serde(
        default,
        serialize_with = "serialize_env_var_value",
        deserialize_with = "deserialize_env_var_value"
    )]
    pub value: String,
}

/// A node in the environment tree. Groups are pure folders (never hold
/// variables); Leaves are environment-variable collections that can be applied
/// to a terminal session. The two are strictly separated — a node is one or the
/// other, never both.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum EnvironmentNode {
    Group {
        id: String,
        name: String,
        #[serde(default)]
        children: Vec<EnvironmentNode>,
    },
    Leaf {
        id: String,
        name: String,
        #[serde(default)]
        variables: Vec<EnvironmentVariable>,
    },
}

impl EnvironmentNode {
    pub fn id(&self) -> &str {
        match self {
            EnvironmentNode::Group { id, .. } => id,
            EnvironmentNode::Leaf { id, .. } => id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            EnvironmentNode::Group { name, .. } => name,
            EnvironmentNode::Leaf { name, .. } => name,
        }
    }

    pub fn set_name(&mut self, new_name: String) {
        match self {
            EnvironmentNode::Group { name, .. } => *name = new_name,
            EnvironmentNode::Leaf { name, .. } => *name = new_name,
        }
    }

    pub fn is_group(&self) -> bool {
        matches!(self, EnvironmentNode::Group { .. })
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, EnvironmentNode::Leaf { .. })
    }

    /// Returns the variables if this is a Leaf, else `None`.
    pub fn as_leaf_variables(&self) -> Option<&[EnvironmentVariable]> {
        match self {
            EnvironmentNode::Leaf { variables, .. } => Some(variables),
            EnvironmentNode::Group { .. } => None,
        }
    }

    /// Collect the ids of this node and (if a Group) all of its descendants.
    /// Used to reject moves that would create a cycle.
    pub fn subtree_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        collect_subtree_ids(self, &mut ids);
        ids
    }
}

fn collect_subtree_ids(node: &EnvironmentNode, ids: &mut Vec<String>) {
    ids.push(node.id().to_string());
    if let EnvironmentNode::Group { children, .. } = node {
        for child in children {
            collect_subtree_ids(child, ids);
        }
    }
}

/// The synthetic root container. It is always a Group conceptually but is
/// modelled as its own type so that the "root must be a group" invariant is
/// enforced by the type system rather than checked at runtime. It does not
/// carry a `kind` tag in the serialized form.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EnvironmentGroup {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub children: Vec<EnvironmentNode>,
}

impl EnvironmentGroup {
    /// Find a descendant node by id anywhere under the root. Returns `None`
    /// when the id does not match any node (the root itself is never returned).
    pub fn find_node(&self, id: &str) -> Option<&EnvironmentNode> {
        self.children.iter().find_map(|c| find_node_inner(c, id))
    }

    /// Mutable variant of [`Self::find_node`].
    pub fn find_node_mut(&mut self, id: &str) -> Option<&mut EnvironmentNode> {
        self.children
            .iter_mut()
            .find_map(|c| find_node_mut_inner(c, id))
    }

    /// Collect every Leaf node in pre-order traversal. Only leaves are exposed
    /// to the terminal profile picker / `apply_environment`.
    pub fn collect_leaves(&self) -> Vec<&EnvironmentNode> {
        let mut leaves = Vec::new();
        for child in &self.children {
            collect_leaves_inner(child, &mut leaves);
        }
        leaves
    }

    /// Remove the descendant node identified by `id` from anywhere in the tree.
    /// Returns `true` if a node was removed. The root itself is never removed.
    pub fn remove_node(&mut self, id: &str) -> bool {
        let before = self.children.len();
        self.children.retain(|c| c.id() != id);
        if self.children.len() != before {
            return true;
        }
        for child in &mut self.children {
            if remove_node_inner(child, id) {
                return true;
            }
        }
        false
    }

    /// Generate the next numeric node id by scanning every node id in the tree.
    pub fn next_node_id(&self) -> String {
        let mut max_id = 0u64;
        collect_max_numeric_id(self, &mut max_id);
        max_id.saturating_add(1).to_string()
    }

    /// Insert `node` into the group identified by `parent_id` (anywhere in the
    /// tree). When `parent_id` is `None`, insert directly under the root. `index`
    /// controls the position (0-based); `None` appends. Returns `true` on success.
    /// Fails (returns `false`) when the named parent is not found or is not a
    /// Group, or when `index` is out of bounds.
    pub fn insert_node(
        &mut self,
        parent_id: Option<&str>,
        node: EnvironmentNode,
        index: Option<usize>,
    ) -> bool {
        match parent_id {
            None => insert_into_children(&mut self.children, node, index),
            Some(pid) => {
                if let Some(parent) = self.find_node_mut(pid) {
                    if let EnvironmentNode::Group { children, .. } = parent {
                        insert_into_children(children, node, index)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }
}

fn insert_into_children(
    children: &mut Vec<EnvironmentNode>,
    node: EnvironmentNode,
    index: Option<usize>,
) -> bool {
    match index {
        None => {
            children.push(node);
            true
        }
        Some(i) if i <= children.len() => {
            children.insert(i, node);
            true
        }
        Some(_) => false,
    }
}

fn find_node_inner<'a>(node: &'a EnvironmentNode, id: &str) -> Option<&'a EnvironmentNode> {
    if node.id() == id {
        return Some(node);
    }
    if let EnvironmentNode::Group { children, .. } = node {
        return children.iter().find_map(|c| find_node_inner(c, id));
    }
    None
}

fn find_node_mut_inner<'a>(
    node: &'a mut EnvironmentNode,
    id: &str,
) -> Option<&'a mut EnvironmentNode> {
    if node.id() == id {
        return Some(node);
    }
    if let EnvironmentNode::Group { children, .. } = node {
        return children.iter_mut().find_map(|c| find_node_mut_inner(c, id));
    }
    None
}

fn collect_leaves_inner<'a>(node: &'a EnvironmentNode, leaves: &mut Vec<&'a EnvironmentNode>) {
    match node {
        EnvironmentNode::Leaf { .. } => leaves.push(node),
        EnvironmentNode::Group { children, .. } => {
            for child in children {
                collect_leaves_inner(child, leaves);
            }
        }
    }
}

fn remove_node_inner(node: &mut EnvironmentNode, id: &str) -> bool {
    if let EnvironmentNode::Group { children, .. } = node {
        let before = children.len();
        children.retain(|c| c.id() != id);
        if children.len() != before {
            return true;
        }
        for child in children {
            if remove_node_inner(child, id) {
                return true;
            }
        }
    }
    false
}

fn collect_max_numeric_id(group: &EnvironmentGroup, max_id: &mut u64) {
    for child in &group.children {
        collect_max_numeric_id_node(child, max_id);
    }
}

fn collect_max_numeric_id_node(node: &EnvironmentNode, max_id: &mut u64) {
    if let Ok(parsed) = node.id().parse::<u64>() {
        if parsed > *max_id {
            *max_id = parsed;
        }
    }
    if let EnvironmentNode::Group { children, .. } = node {
        for child in children {
            collect_max_numeric_id_node(child, max_id);
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct EnvironmentConfig {
    #[serde(default)]
    pub root: EnvironmentGroup,
}

impl<'de> Deserialize<'de> for EnvironmentConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct RawEnvironmentConfig {
            /// New tree-shaped format.
            root: EnvironmentGroup,
            /// Legacy flat-list format (`server.json` written by older builds).
            /// Each legacy entry becomes a Leaf under the root.
            #[serde(alias = "entries")]
            entries: Vec<LegacyEntry>,
        }

        #[derive(Deserialize)]
        struct LegacyEntry {
            #[serde(default)]
            id: String,
            #[serde(default)]
            name: String,
            #[serde(default)]
            variables: Vec<EnvironmentVariable>,
        }

        let raw = RawEnvironmentConfig::deserialize(deserializer)?;
        let mut root = raw.root;
        if !raw.entries.is_empty() {
            // Migrate legacy flat entries into the tree as top-level Leaf nodes.
            // Use a fallback synthetic id if a legacy entry is missing one.
            let mut next_fallback = 0u64;
            for entry in raw.entries {
                let id = if entry.id.trim().is_empty() {
                    next_fallback += 1;
                    next_fallback.to_string()
                } else {
                    entry.id
                };
                root.children.push(EnvironmentNode::Leaf {
                    id,
                    name: entry.name,
                    variables: entry.variables,
                });
            }
        }
        Ok(EnvironmentConfig { root })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ConfigPatchRequest {
    pub server: Option<ServerConfigPatch>,
    pub device: Option<DeviceConfigPatch>,
    pub ddns: Option<DdnsConfigPatch>,
    pub notification: Option<NotificationConfigPatch>,
    #[serde(alias = "terminal_observer")]
    pub terminal: Option<TerminalConfigPatch>,
    pub llm: Option<LlmConfigPatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ServerConfigPatch {
    pub relay_port: Option<u16>,
    pub config_port: Option<u16>,
    pub proxy_port_start: Option<u16>,
    pub proxy_port_end: Option<u16>,
    pub static_http_port_start: Option<u16>,
    pub static_http_port_end: Option<u16>,
    pub static_http_bind_address: Option<StaticHttpBindAddress>,
    pub name: Option<String>,
    pub relay_address: Option<String>,
    pub secret_key: Option<String>,
    pub relay_secret_key: Option<String>,
    pub relay_strict_tls: Option<bool>,
    pub relay_ca_pem: Option<String>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub tls_extra_sans: Option<Vec<String>>,
    pub log_to_file: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DeviceConfigPatch {
    pub serial_port: Option<String>,
    pub serial_baud_rate: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DdnsConfigPatch {
    pub enabled: Option<bool>,
    pub provider: Option<Option<String>>,
    pub polling_interval_minutes: Option<u32>,
    pub ip_family: Option<String>,
    pub network_interface: Option<Option<String>>,
    pub user_config: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct NotificationTargetConfigPatch {
    pub id: Option<String>,
    pub channel: Option<NotificationChannel>,
    pub device_token: Option<String>,
    pub topic: Option<Option<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ApnsNotificationConfigPatch {
    pub enabled: Option<bool>,
    pub certificate_path: Option<Option<String>>,
    pub certificate_password: Option<Option<String>>,
    pub environment: Option<String>,
    pub topic: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FcmNotificationConfigPatch {
    pub enabled: Option<bool>,
    pub service_account_json: Option<Option<String>>,
    pub project_id: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TpnsNotificationConfigPatch {
    pub enabled: Option<bool>,
    pub access_id: Option<Option<String>>,
    pub secret_key: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct NotificationConfigPatch {
    pub enabled: Option<bool>,
    pub queue_capacity: Option<u16>,
    pub max_retries: Option<u8>,
    pub notify_on_startup: Option<bool>,
    pub targets: Option<Vec<NotificationTargetConfigPatch>>,
    pub apns: Option<ApnsNotificationConfigPatch>,
    pub fcm: Option<FcmNotificationConfigPatch>,
    pub tpns: Option<TpnsNotificationConfigPatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TerminalObserverPatternConfigPatch {
    pub id: Option<String>,
    pub kind: Option<TerminalObserverPatternKind>,
    pub substrings: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TerminalConfigPatch {
    pub enabled: Option<bool>,
    pub auto_confirm: Option<bool>,
    pub analysis_delay_seconds: Option<u16>,
    pub confirmation_grace_seconds: Option<u16>,
    pub notify_on_completion: Option<bool>,
    pub patterns: Option<Vec<TerminalObserverPatternConfigPatch>>,
    pub llm_provider: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LlmProviderConfigPatch {
    pub enabled: Option<bool>,
    pub provider: Option<Option<String>>,
    pub user_config: Option<std::collections::HashMap<String, String>>,
    pub max_failures: Option<u16>,
    pub retry_cooldown_secs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CodeAnalysisLlmConfigPatch {
    pub enabled: Option<bool>,
    pub provider: Option<Option<String>>,
    pub user_config: Option<std::collections::HashMap<String, String>>,
    pub max_failures: Option<u16>,
    pub retry_cooldown_secs: Option<u32>,
    pub output_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TranslationLlmConfigPatch {
    pub enabled: Option<bool>,
    pub provider: Option<Option<String>>,
    pub user_config: Option<std::collections::HashMap<String, String>>,
    pub max_failures: Option<u16>,
    pub retry_cooldown_secs: Option<u32>,
    pub output_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LlmConfigPatch {
    pub terminal: Option<LlmProviderConfigPatch>,
    pub git: Option<LlmProviderConfigPatch>,
    pub git_conflict: Option<LlmProviderConfigPatch>,
    pub chat: Option<LlmProviderConfigPatch>,
    pub voice_polish: Option<LlmProviderConfigPatch>,
    pub code_analysis: Option<CodeAnalysisLlmConfigPatch>,
    pub translation: Option<TranslationLlmConfigPatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigUpdateResult {
    pub config: TerminalServerConfig,
    pub restart_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigDocument {
    pub config_path: PathBuf,
    pub config: TerminalServerConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to serialize configuration: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("Configuration validation failed: {0}")]
    Validation(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

fn serialize_server_secret_key<S>(secret_key: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&normalize_server_secret_key_for_storage(secret_key))
}

fn deserialize_server_secret_key<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    Ok(normalize_loaded_server_secret_key(&value))
}

pub(crate) fn normalize_loaded_server_secret_key(value: &str) -> String {
    decrypt_server_secret_key(value).unwrap_or_else(|_| value.trim().to_string())
}

pub(crate) fn is_protected_server_secret_key(secret_key: &str) -> bool {
    let secret_key = secret_key.trim();
    secret_key.starts_with(SECRET_KEY_HASHED_PREFIX) || secret_key.starts_with("enc-v")
}

pub(crate) fn is_hashed_server_secret_key(secret_key: &str) -> bool {
    secret_key.trim().starts_with(SECRET_KEY_HASHED_PREFIX)
}

pub(crate) fn stored_server_secret_key_hash(secret_key: &str) -> Option<&str> {
    secret_key.trim().strip_prefix(SECRET_KEY_HASHED_PREFIX)
}

fn normalize_server_secret_key_for_storage(secret_key: &str) -> String {
    let secret_key = secret_key.trim();
    if secret_key.is_empty() {
        return String::new();
    }

    if is_protected_server_secret_key(secret_key) {
        return secret_key.to_string();
    }

    encrypt_server_secret_key(secret_key).unwrap_or_else(|_| secret_key.to_string())
}

pub(crate) fn generate_server_secret_key() -> String {
    generate_random_token(16)
}

/// Random URL-safe alphanumeric token from `OsRng` — the same alphabet and
/// default length as the webui's `generateRandomToken`. Used for config
/// values that must never ship empty, like the MCP access token.
pub(crate) fn generate_random_token(length: usize) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut bytes = vec![0u8; length];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes
        .iter()
        .map(|byte| CHARS[usize::from(*byte) % CHARS.len()] as char)
        .collect()
}

/// Detect whether the process is running under Windows Subsystem for Linux.
///
/// WSL kernels expose a lowercase `microsoft` marker in `/proc/version` and
/// `/proc/sys/kernel/osrelease` (e.g. `6.6.87.1-microsoft-standard-WSL2`).
/// Interactive WSL sessions also set `WSL_INTEROP`; `/proc` is checked as well
/// because that env var is absent under systemd / non-interactive launch.
#[cfg(target_os = "linux")]
pub(crate) fn is_running_under_wsl() -> bool {
    if std::env::var_os("WSL_INTEROP").is_some() || std::env::var_os("WSL_DISTRO_NAME").is_some() {
        return true;
    }
    ["/proc/version", "/proc/sys/kernel/osrelease"]
        .iter()
        .any(|path| {
            std::fs::read_to_string(path)
                .map(|content| content.to_lowercase().contains("microsoft"))
                .unwrap_or(false)
        })
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn is_running_under_wsl() -> bool {
    false
}

/// Apply environment-specific defaults to a freshly generated config.
///
/// Only adjusts ports, and only when running under WSL. Called solely from the
/// first-startup config-creation path; existing configs keep their values.
pub(crate) fn apply_platform_default_ports(config: &mut TerminalServerConfig) {
    if !is_running_under_wsl() {
        return;
    }
    config.server.relay_port = WSL_DEFAULT_RELAY_PORT;
    config.server.config_port = WSL_DEFAULT_CONFIG_PORT;
    config.server.static_http_port_start = WSL_DEFAULT_STATIC_HTTP_PORT_START;
    config.server.static_http_port_end = WSL_DEFAULT_STATIC_HTTP_PORT_END;
    config.server.proxy_port_start = WSL_DEFAULT_PROXY_PORT_START;
    config.server.proxy_port_end = WSL_DEFAULT_PROXY_PORT_END;
    config.mcp.port = WSL_DEFAULT_MCP_PORT;
}

fn encrypt_server_secret_key(secret_key: &str) -> Result<String, String> {
    let key = config_secret_key_encryption_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|error| format!("failed to initialize secret key encryption: {}", error))?;
    let mut nonce = [0u8; SECRET_KEY_NONCE_SIZE];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), secret_key.as_bytes())
        .map_err(|error| format!("failed to encrypt secret key: {}", error))?;

    Ok(format!(
        "{}{}:{}",
        SECRET_KEY_ENCRYPTED_PREFIX_V2,
        encode_hex(&nonce),
        encode_hex(&encrypted)
    ))
}

fn decrypt_server_secret_key(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }

    if let Some(payload) = value.strip_prefix(SECRET_KEY_ENCRYPTED_PREFIX_V2) {
        let (nonce_hex, encrypted_hex) = payload
            .split_once(':')
            .ok_or_else(|| "encrypted secret key is missing nonce or payload".to_string())?;
        let nonce = decode_hex(nonce_hex)?;
        let encrypted = decode_hex(encrypted_hex)?;
        if nonce.len() != SECRET_KEY_NONCE_SIZE {
            return Err("encrypted secret key has an invalid nonce size".to_string());
        }

        let key = config_secret_key_encryption_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|error| format!("failed to initialize secret key decryption: {}", error))?;
        let decrypted = cipher
            .decrypt(Nonce::from_slice(&nonce), encrypted.as_ref())
            .map_err(|error| format!("failed to decrypt secret key: {}", error))?;

        return String::from_utf8(decrypted)
            .map_err(|error| format!("decrypted secret key is not valid utf-8: {}", error));
    }

    Ok(value.to_string())
}

fn config_secret_key_encryption_key() -> Result<[u8; 32], String> {
    #[cfg(test)]
    let machine_id = "vterm-server-test-machine-id".to_string();

    #[cfg(not(test))]
    let machine_id = machine_uid::get()
        .map_err(|error| format!("failed to derive machine identifier: {}", error))?;

    let mut hasher = Sha256::new();
    hasher.update(SECRET_KEY_KEY_CONTEXT);
    hasher.update(machine_id.as_bytes());
    let digest = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    Ok(key)
}

fn env_var_encryption_key() -> Result<[u8; 32], String> {
    #[cfg(test)]
    let machine_id = "vterm-server-test-machine-id".to_string();

    #[cfg(not(test))]
    let machine_id = machine_uid::get()
        .map_err(|error| format!("failed to derive machine identifier: {}", error))?;

    let mut hasher = Sha256::new();
    hasher.update(ENV_VAR_KEY_CONTEXT);
    hasher.update(machine_id.as_bytes());
    let digest = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    Ok(key)
}

fn encrypt_env_var_value(value: &str) -> Result<String, String> {
    let key = env_var_encryption_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|error| format!("failed to initialize env var encryption: {}", error))?;
    let mut nonce = [0u8; SECRET_KEY_NONCE_SIZE];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), value.as_bytes())
        .map_err(|error| format!("failed to encrypt env var value: {}", error))?;

    Ok(format!(
        "{}{}:{}",
        SECRET_KEY_ENCRYPTED_PREFIX_V2,
        encode_hex(&nonce),
        encode_hex(&encrypted)
    ))
}

fn decrypt_env_var_value(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }

    if let Some(payload) = value.strip_prefix(SECRET_KEY_ENCRYPTED_PREFIX_V2) {
        let (nonce_hex, encrypted_hex) = payload
            .split_once(':')
            .ok_or_else(|| "encrypted env var value is missing nonce or payload".to_string())?;
        let nonce = decode_hex(nonce_hex)?;
        let encrypted = decode_hex(encrypted_hex)?;
        if nonce.len() != SECRET_KEY_NONCE_SIZE {
            return Err("encrypted env var value has an invalid nonce size".to_string());
        }

        let key = env_var_encryption_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|error| format!("failed to initialize env var decryption: {}", error))?;
        let decrypted = cipher
            .decrypt(Nonce::from_slice(&nonce), encrypted.as_ref())
            .map_err(|error| format!("failed to decrypt env var value: {}", error))?;

        return String::from_utf8(decrypted)
            .map_err(|error| format!("decrypted env var value is not valid utf-8: {}", error));
    }

    Ok(value.to_string())
}

fn serialize_env_var_value<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if value.is_empty() {
        return serializer.serialize_str("");
    }
    let encrypted = encrypt_env_var_value(value).unwrap_or_else(|_| value.to_string());
    serializer.serialize_str(&encrypted)
}

fn deserialize_env_var_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.is_empty() || !value.starts_with(SECRET_KEY_ENCRYPTED_PREFIX_V2) {
        return Ok(value);
    }
    decrypt_env_var_value(&value).map_err(|e| serde::de::Error::custom(e))
}

fn addon_config_encryption_key() -> Result<[u8; 32], String> {
    #[cfg(test)]
    let machine_id = "vterm-server-test-machine-id".to_string();

    #[cfg(not(test))]
    let machine_id = machine_uid::get()
        .map_err(|error| format!("failed to derive machine identifier: {}", error))?;

    let mut hasher = Sha256::new();
    hasher.update(ADDON_CONFIG_KEY_CONTEXT);
    hasher.update(machine_id.as_bytes());
    let digest = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    Ok(key)
}

pub fn encrypt_addon_secret(value: &str) -> Result<String, String> {
    let key = addon_config_encryption_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|error| format!("failed to initialize addon config encryption: {}", error))?;
    let mut nonce = [0u8; SECRET_KEY_NONCE_SIZE];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), value.as_bytes())
        .map_err(|error| format!("failed to encrypt addon config value: {}", error))?;
    Ok(format!(
        "{}{}:{}",
        SECRET_KEY_ENCRYPTED_PREFIX_V2,
        encode_hex(&nonce),
        encode_hex(&encrypted)
    ))
}

pub fn decrypt_addon_secret(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }

    if let Some(payload) = value.strip_prefix(SECRET_KEY_ENCRYPTED_PREFIX_V2) {
        let (nonce_hex, encrypted_hex) = payload.split_once(':').ok_or_else(|| {
            "encrypted addon config value is missing nonce or payload".to_string()
        })?;
        let nonce = decode_hex(nonce_hex)?;
        let encrypted = decode_hex(encrypted_hex)?;
        if nonce.len() != SECRET_KEY_NONCE_SIZE {
            return Err("encrypted addon config value has an invalid nonce size".to_string());
        }

        let key = addon_config_encryption_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|error| format!("failed to initialize addon config decryption: {}", error))?;
        let decrypted = cipher
            .decrypt(Nonce::from_slice(&nonce), encrypted.as_ref())
            .map_err(|error| format!("failed to decrypt addon config value: {}", error))?;

        return String::from_utf8(decrypted).map_err(|error| {
            format!("decrypted addon config value is not valid utf-8: {}", error)
        });
    }

    Ok(value.to_string())
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(hex_nibble(byte >> 4));
        encoded.push(hex_nibble(byte & 0x0f));
    }
    encoded
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 {
        return Err(
            "encrypted secret key must contain an even number of hex characters".to_string(),
        );
    }

    let mut decoded = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    for index in (0..bytes.len()).step_by(2) {
        let high = from_hex_nibble(bytes[index])?;
        let low = from_hex_nibble(bytes[index + 1])?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

fn hex_nibble(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + (value - 10)) as char,
    }
}

fn from_hex_nibble(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err("encrypted secret key contains non-hex characters".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        decrypt_server_secret_key, encrypt_server_secret_key, is_hashed_server_secret_key,
        normalize_loaded_server_secret_key, normalize_server_secret_key_for_storage,
        stored_server_secret_key_hash, McpConfig, RemoteDesktopConfig, ServerConfig,
    };
    use vterm_common::hash_secret_key;

    #[test]
    fn mcp_default_access_token_is_non_empty_and_random() {
        let first = McpConfig::default();
        let second = McpConfig::default();
        assert!(!first.access_token.is_empty());
        assert_ne!(first.access_token, second.access_token);
    }

    /// Port forwarding and remote desktop follow the build flavor: desktop
    /// (`ui`) builds ship both on, headless builds (`--no-default-features`)
    /// ship both off. Covers the `Default` impls and the serde
    /// missing-field paths (container `#[serde(default)]`).
    #[test]
    fn proxy_and_remote_desktop_defaults_follow_build_flavor() {
        let expected = cfg!(feature = "ui");

        assert_eq!(ServerConfig::default().proxy_enabled, expected);
        assert_eq!(RemoteDesktopConfig::default().enabled, expected);

        // Configs persisted before these fields existed (or written by a
        // minimal writer) deserialize missing fields from the same defaults.
        let server: ServerConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(server.proxy_enabled, expected);
        let remote_desktop: RemoteDesktopConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(remote_desktop.enabled, expected);

        // Explicit values always win over the default, in either flavor.
        let server: ServerConfig =
            serde_json::from_str(&format!(r#"{{"proxy_enabled": {}}}"#, !expected)).unwrap();
        assert_eq!(server.proxy_enabled, !expected);
    }

    #[test]
    fn encrypts_and_decrypts_server_secret_key() {
        let encrypted = encrypt_server_secret_key("abcdefghijkl").unwrap();
        assert!(encrypted.starts_with("enc-v2:"));

        let decrypted = decrypt_server_secret_key(&encrypted).unwrap();
        assert_eq!(decrypted, "abcdefghijkl");
    }

    #[test]
    fn decrypt_rejects_invalid_ciphertext() {
        let error = decrypt_server_secret_key("enc-v2:0011:zz").unwrap_err();
        assert!(error.contains("non-hex"));
    }

    #[test]
    fn hashes_plain_server_secret_key_for_storage() {
        let stored = normalize_server_secret_key_for_storage("abcdefghijkl");
        assert!(stored.starts_with("enc-v2:"));
        assert_eq!(normalize_loaded_server_secret_key(&stored), "abcdefghijkl");
    }

    #[test]
    fn preserves_existing_hashed_server_secret_key() {
        let stored = normalize_server_secret_key_for_storage(
            "sha256:d682ed4ca4d989c134ec94f1551e1ec580dd6d5a6ecde9f3d35e6e4a717fbde4",
        );
        assert_eq!(
            stored,
            "sha256:d682ed4ca4d989c134ec94f1551e1ec580dd6d5a6ecde9f3d35e6e4a717fbde4"
        );
        assert!(is_hashed_server_secret_key(&stored));
        assert_eq!(
            stored_server_secret_key_hash(&stored),
            Some("d682ed4ca4d989c134ec94f1551e1ec580dd6d5a6ecde9f3d35e6e4a717fbde4")
        );
    }

    #[test]
    fn keeps_legacy_encrypted_value_loadable_for_recovery() {
        assert_eq!(
            normalize_loaded_server_secret_key("enc-v2:anything"),
            "enc-v2:anything"
        );
    }

    #[test]
    fn keeps_legacy_hashed_value_loadable_for_recovery() {
        let hashed = format!("sha256:{}", hash_secret_key("abcdefghijkl"));
        assert_eq!(normalize_loaded_server_secret_key(&hashed), hashed);
    }
}
