//! Headless command-line interface.
//!
//! All commands here run WITHOUT starting the server and exit immediately —
//! they exist so headless deployments (no desktop, no webui) can onboard
//! clients from a plain console:
//!
//! - `--show-secret-key` prints the client-connection secret key.
//! - `--set-secret-key` / `--generate-secret-key` persist a new key.
//! - `--share` prints the encrypted share link (`vtshare1:...`) — the same
//!   string the webui copies when clicking the pairing QR code
//!   (`webui/src/components/sections/ServerSection.tsx`). The desktop client
//!   decrypts it on its add-server form
//!   (`crates/vterm/src/ui/server_manager/share_payload.rs`).
//! - `--open` prints the Web UI address and opens it in a browser — the
//!   headless/WSL onboarding path to the configuration page. On Linux the
//!   running server mints a one-time ticket over local IPC
//!   (`settings_ipc.rs`) so the printed URL signs straight in.

use clap::{ArgGroup, Parser};
use rand::RngCore;
use serde::Serialize;

use crate::config::{
    self, is_hashed_server_secret_key, ConfigManager, ConfigPatchRequest, ServerConfigPatch,
};

const SHARE_PREFIX: &str = "vtshare1:";
/// Four big-endian words of the ASCII key shared with the webui and the
/// desktop client. The webui derives them from `VibeTermQrKeyV1` and reads
/// the missing 16th byte as 0, hence the trailing NUL here.
const XTEA_KEY: [u8; 16] = *b"VibeTermQrKeyV1\0";
const XTEA_DELTA: u32 = 0x9e37_79b9;
const XTEA_ROUNDS: u32 = 32;
const BLOCK_SIZE: usize = 8;

/// Vibe Terminal Server
#[derive(Parser)]
#[command(name = "vterm-server", version, about)]
#[command(group(
    ArgGroup::new("headless")
        .args(["show_secret_key", "set_secret_key", "generate_secret_key", "share", "open"])
        .multiple(false)
))]
pub struct Cli {
    /// Print the client-connection secret key and exit
    #[arg(long, visible_aliases = ["show-password", "print-secret-key"])]
    pub show_secret_key: bool,

    /// Set the client-connection secret key (12-16 characters) and exit
    #[arg(long, value_name = "KEY")]
    pub set_secret_key: Option<String>,

    /// Generate, persist and print a new random secret key, then exit
    #[arg(long)]
    pub generate_secret_key: bool,

    /// Print the pairing address and encrypted share link, then exit
    #[arg(long)]
    pub share: bool,

    /// Print the Web UI address and open it in a browser, then exit
    #[arg(long)]
    pub open: bool,

    /// Network interface whose address is used by --share (default: first
    /// non-loopback interface with an address)
    #[arg(long, value_name = "NAME", requires = "share")]
    pub interface: Option<String>,

    /// Use an IPv6 address for --share (default: IPv4)
    #[arg(long, requires = "share")]
    pub ipv6: bool,

    /// Use the configured DDNS domain for --share instead of an IP address
    #[arg(long, requires = "share")]
    pub domain: bool,
}

/// Run the requested headless command. Returns `None` when no headless flag
/// was given and normal server startup should proceed.
pub fn run(cli: &Cli) -> Option<anyhow::Result<()>> {
    let headless = cli.show_secret_key
        || cli.set_secret_key.is_some()
        || cli.generate_secret_key
        || cli.share
        || cli.open;
    if !headless {
        return None;
    }

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    Some(rt.block_on(run_inner(cli)))
}

async fn run_inner(cli: &Cli) -> anyhow::Result<()> {
    // Loading the config creates and persists it on first run (platform
    // defaults, generated secret key), so every command below always sees a
    // fully initialized configuration.
    let config_manager = ConfigManager::load().await?;
    let config = config_manager.snapshot().await;

    if cli.show_secret_key {
        return print_secret_key(&config.server.secret_key);
    }

    if let Some(key) = &cli.set_secret_key {
        return set_secret_key(&config_manager, key.clone()).await;
    }

    if cli.generate_secret_key {
        let key = config::generate_server_secret_key();
        set_secret_key(&config_manager, key.clone()).await?;
        println!("Secret Key: {key}");
        return Ok(());
    }

    if cli.open {
        return open_webui(&config);
    }

    // --share
    print_share_link(&config, cli)
}

/// `--open`: print the Web UI address (config server port) and open it in a
/// browser. On headless/WSL boxes the browser helper may be missing or may
/// target the Windows host — the URL is always printed so it can be opened
/// manually either way.
///
/// The settings page exchanges a one-time ticket for a session cookie
/// (docs/settings-static-http-authentication.md), so on Linux the CLI first
/// asks the running server to mint a ticket over its local IPC socket and
/// attaches it to the URL; a bare URL would only get a 401.
fn open_webui(config: &crate::config::TerminalServerConfig) -> anyhow::Result<()> {
    // The server honors the CONFIG_PORT env var at startup; reflect it here
    // so the URL matches the listener the deployment actually runs.
    let port = std::env::var("CONFIG_PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|port| *port != 0)
        .unwrap_or(config.server.config_port);

    // Ask the running server for a one-time ticket. `None` when no server
    // answered (not running, different flavor or uid) — the plain URL is
    // still printed, with a hint why it carries no token.
    #[cfg(target_os = "linux")]
    let ticket = crate::settings_ipc::request_settings_ticket();
    #[cfg(not(target_os = "linux"))]
    let ticket: Option<String> = None;
    let url = match &ticket {
        Some(ticket) => format!("http://localhost:{port}/?ticket={ticket}"),
        None => format!("http://localhost:{port}/"),
    };

    println!("Web UI: {url}");

    if ticket.is_none() {
        #[cfg(target_os = "linux")]
        println!(
            "No running vterm-server answered, so the URL carries no access token and the\n\
             connection will be closed without a response. Start vterm-server first, then\n\
             re-run --open for a link that signs straight in."
        );
    }

    // Prefer the platform opener. Under WSL, `wslview` (wslu) opens the
    // Windows host's browser; plain xdg-open inside a WSL distro usually
    // has nothing to open. All failures are soft — the URL was printed.
    let opened = if cfg!(target_os = "linux") && crate::config::is_running_under_wsl() {
        std::process::Command::new("wslview")
            .arg(&url)
            .spawn()
            .is_ok()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(&url).spawn().is_ok()
    } else {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .is_ok()
    };
    if !opened {
        println!("Could not launch a browser here; open the URL above manually.");
    }
    Ok(())
}

fn print_secret_key(secret_key: &str) -> anyhow::Result<()> {
    let secret_key = secret_key.trim();
    if secret_key.is_empty() {
        println!(
            "No secret key configured. Set one with --set-secret-key or --generate-secret-key."
        );
    } else if is_hashed_server_secret_key(secret_key) {
        anyhow::bail!(
            "The stored secret key is a one-way hash and cannot be displayed. \
             Set a new one with --set-secret-key or --generate-secret-key."
        );
    } else {
        println!("Secret Key: {secret_key}");
    }
    Ok(())
}

async fn set_secret_key(config_manager: &ConfigManager, key: String) -> anyhow::Result<()> {
    let key = key.trim().to_string();
    // Mirror of the config validation rule (config/validation.rs).
    if !(12..=16).contains(&key.len()) {
        anyhow::bail!(
            "Secret key must be between 12 and 16 characters (got {}).",
            key.len()
        );
    }

    if crate::server_instance_running() {
        eprintln!(
            "Warning: a vterm-server instance is currently running. Restart it for the \
             new secret key to take effect; a running server may also overwrite this \
             change on its next config save."
        );
    }

    config_manager
        .apply_patch(ConfigPatchRequest {
            server: Some(ServerConfigPatch {
                secret_key: Some(key),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await?;
    println!("Secret key updated.");
    Ok(())
}

/// The JSON payload encrypted into the share link — same shape and field
/// order as the webui's share string. `host` is the raw address literal (no
/// IPv6 brackets; the client re-wraps).
#[derive(Serialize)]
struct SharePayload<'a> {
    r#type: &'static str,
    name: &'a str,
    host: &'a str,
    port: u16,
    secret_key: &'a str,
}

fn print_share_link(config: &crate::config::TerminalServerConfig, cli: &Cli) -> anyhow::Result<()> {
    let secret_key = config.server.secret_key.trim();
    if is_hashed_server_secret_key(secret_key) {
        anyhow::bail!(
            "The stored secret key is a one-way hash and cannot be shared. \
             Set a new one with --set-secret-key or --generate-secret-key first."
        );
    }

    // The server honors the PORT env var at startup; reflect it here so the
    // link matches the listener the deployment actually runs.
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|port| *port != 0)
        .unwrap_or(config.server.relay_port);

    let host = if cli.domain {
        let domain = config::ddns_domain_from_config(config);
        if domain.is_empty() {
            anyhow::bail!("No DDNS domain is configured; drop --domain or configure DDNS first.");
        }
        domain
    } else {
        select_address(cli)?
    };

    let payload = serde_json::to_string(&SharePayload {
        r#type: "vibe-terminal-server",
        name: &config.server.name,
        host: &host,
        port,
        secret_key,
    })?;
    let mut iv = [0u8; BLOCK_SIZE];
    rand::rngs::OsRng.fill_bytes(&mut iv);
    let share_link = encrypt_share_payload(payload.as_bytes(), &iv);

    // IPv6 literals must be bracketed in URLs (RFC 3986).
    let display_host = if !cli.domain && host.contains(':') {
        format!("[{host}]")
    } else {
        host.clone()
    };

    println!("Server:  {}", config.server.name);
    println!("Address: wss://{display_host}:{port}");
    println!();
    println!("Share link (paste into the client's add-server form):");
    println!("{share_link}");
    Ok(())
}

/// Pick the interface address used by --share, mirroring the webui's
/// interface/family selection.
fn select_address(cli: &Cli) -> anyhow::Result<String> {
    let interfaces = config::detect_network_interfaces();
    if interfaces.is_empty() {
        anyhow::bail!("No non-loopback network interface with an address was found.");
    }

    let interface = match &cli.interface {
        Some(name) => interfaces
            .iter()
            .find(|interface| &interface.id == name || &interface.name == name)
            .ok_or_else(|| {
                let available = interfaces
                    .iter()
                    .map(|interface| interface.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                anyhow::anyhow!("Unknown interface '{name}'. Available: {available}")
            })?,
        None => &interfaces[0],
    };

    let addresses = if cli.ipv6 {
        &interface.ipv6_addresses
    } else {
        &interface.ipv4_addresses
    };
    let family = if cli.ipv6 { "IPv6" } else { "IPv4" };
    addresses.first().cloned().ok_or_else(|| {
        anyhow::anyhow!(
            "Interface '{}' has no {family} address; try {} or another --interface.",
            interface.name,
            if cli.ipv6 {
                "dropping --ipv6"
            } else {
                "--ipv6"
            },
        )
    })
}

/// XTEA-CBC encrypt `plain` and format as `vtshare1:<iv hex>:<ciphertext hex>`.
/// The inverse of the webui's `encryptWithXteaCbc` and the client's decrypt.
fn encrypt_share_payload(plain: &[u8], iv: &[u8; BLOCK_SIZE]) -> String {
    // PKCS-style padding: 1..=8 bytes of the pad length (8 when aligned).
    let padding = BLOCK_SIZE - (plain.len() % BLOCK_SIZE);
    let mut padded = plain.to_vec();
    padded.extend(std::iter::repeat(padding as u8).take(padding));

    let key = xtea_key_words();
    let mut ciphertext = Vec::with_capacity(padded.len());
    let mut previous = *iv;
    for block in padded.chunks(BLOCK_SIZE) {
        let mut xored = [0u8; BLOCK_SIZE];
        for (index, byte) in block.iter().enumerate() {
            xored[index] = byte ^ previous[index];
        }
        let mut left = be_word(&xored, 0);
        let mut right = be_word(&xored, 4);
        encrypt_xtea_block(&mut left, &mut right, &key);
        let mut out = [0u8; BLOCK_SIZE];
        write_be_word(&mut out, 0, left);
        write_be_word(&mut out, 4, right);
        ciphertext.extend_from_slice(&out);
        previous = out;
    }

    format!(
        "{SHARE_PREFIX}{}:{}",
        encode_hex(iv),
        encode_hex(&ciphertext)
    )
}

/// XTEA block encryption — the counterpart of the webui's `encryptXteaBlock`.
fn encrypt_xtea_block(left: &mut u32, right: &mut u32, key: &[u32; 4]) {
    let mut sum = 0u32;
    for _ in 0..XTEA_ROUNDS {
        *left = left.wrapping_add(
            (((*right << 4) ^ (*right >> 5)).wrapping_add(*right))
                ^ sum.wrapping_add(key[(sum & 3) as usize]),
        );
        sum = sum.wrapping_add(XTEA_DELTA);
        *right = right.wrapping_add(
            (((*left << 4) ^ (*left >> 5)).wrapping_add(*left))
                ^ sum.wrapping_add(key[((sum >> 11) & 3) as usize]),
        );
    }
}

fn xtea_key_words() -> [u32; 4] {
    [
        be_word(&XTEA_KEY, 0),
        be_word(&XTEA_KEY, 4),
        be_word(&XTEA_KEY, 8),
        be_word(&XTEA_KEY, 12),
    ]
}

fn be_word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes.get(offset).copied().unwrap_or(0),
        bytes.get(offset + 1).copied().unwrap_or(0),
        bytes.get(offset + 2).copied().unwrap_or(0),
        bytes.get(offset + 3).copied().unwrap_or(0),
    ])
}

fn write_be_word(out: &mut [u8], offset: usize, word: u32) {
    out[offset..offset + 4].copy_from_slice(&word.to_be_bytes());
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ciphertext produced by the webui implementation with fixed IV
    /// `0001020304050607` — the same vector the desktop client tests its
    /// decrypt against (vterm share_payload.rs).
    const VECTOR_CIPHERTEXT: &str = "a85b448077b55fbe4e7237db78380b1b1cbae9d47d2457d143a5fab478d44e63c3d5acc8b0de33a72200a032ecd069d13d394f734e249842a43fbc74d81aaf458b0eeb28ee9df904b1136a88364070e81476a97270f1177f59762481406ff0ad89a3c3175f82e15c10f9f34b19ee9f7db3812c1ba14da132";

    #[test]
    fn encrypt_matches_webui_vector() {
        let payload = "{\"type\":\"vibe-terminal-server\",\"name\":\"Test Server\",\"host\":\"192.168.1.10\",\"port\":8780,\"secret_key\":\"mySecretKey123\"}";
        let iv = [0, 1, 2, 3, 4, 5, 6, 7];
        let encrypted = encrypt_share_payload(payload.as_bytes(), &iv);
        assert_eq!(
            encrypted,
            format!("vtshare1:0001020304050607:{VECTOR_CIPHERTEXT}")
        );
    }

    #[test]
    fn pads_to_block_boundary() {
        let iv = [0u8; BLOCK_SIZE];
        // 8-byte input pads with a full block of 0x08.
        let encrypted = encrypt_share_payload(b"12345678", &iv);
        let ciphertext_hex = encrypted
            .strip_prefix("vtshare1:0000000000000000:")
            .expect("prefix");
        assert_eq!(ciphertext_hex.len(), 2 * 2 * BLOCK_SIZE);
    }
}
