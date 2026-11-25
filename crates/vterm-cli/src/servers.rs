//! Known servers for `vtermc -s`, mirroring the list the GUI shows in its
//! server button menu: the implicit Local server plus the client-managed
//! servers stored in `~/.vibeterminal/client.json`.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

const CLIENT_CONFIG_FILE_ENV: &str = "TERMINAL_CLIENT_CONFIG_FILE";
const DEFAULT_CONFIG_DIR: &str = ".vibeterminal";
const CLIENT_CONFIG_FILE: &str = "client.json";
const SERVER_CONFIG_FILE: &str = "server.json";
const CONFIG_VERSION: u32 = 1;

/// A server the user can connect to.
#[derive(Debug, Clone)]
pub struct KnownServer {
    pub name: String,
    pub url: String,
    /// Plaintext secret key for the handshake challenge. Empty means "use
    /// `TERMINAL_GUI_SECRET_KEY` from the environment" (the usual case for
    /// Local, whose `server.json` stores the key encrypted).
    pub secret_key: String,
    /// Whether this is the implicit Local entry (never deletable, always
    /// shown without name/address in the `-s` list).
    pub local: bool,
}

impl KnownServer {
    /// Label shown in the `vtermc -s` server list: the local machine is
    /// just "Local"; managed servers show `name  (address)`.
    pub fn label(&self) -> String {
        if self.local {
            "Local".to_string()
        } else {
            format!("{}  ({})", self.name, self.url)
        }
    }
}

/// Build the server list shown by `vtermc -s`: Local first, then every
/// managed server from `client.json`.
pub fn load_servers() -> Result<Vec<KnownServer>> {
    let mut servers = vec![local_server()];

    let path = client_config_path()?;
    if !path.exists() {
        return Ok(servers);
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let config: ClientConfigFile = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    for server in config.servers {
        // Skip blank entries; anything else is kept as-is, including
        // loopback addresses — under WSL `localhost` reaches the Windows
        // host's server, a different machine than the implicit Local entry.
        if server.host.trim().is_empty() {
            continue;
        }
        servers.push(KnownServer {
            name: if server.name.trim().is_empty() {
                server.host.clone()
            } else {
                server.name
            },
            url: server.host,
            secret_key: server.secret_key,
            local: false,
        });
    }
    Ok(servers)
}

/// The implicit Local entry. When `~/.vibeterminal/server.json` exists, take
/// its name/port (and plaintext secret, if stored unencrypted) from there.
fn local_server() -> KnownServer {
    let mut server = KnownServer {
        name: "Local".to_string(),
        url: format!("wss://localhost:{}", vterm_common::DEFAULT_SERVER_PORT),
        secret_key: String::new(),
        local: true,
    };

    let Ok(path) = server_config_path() else {
        return server;
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return server;
    };
    let Ok(config) = serde_json::from_str::<ServerConfigFile>(&content) else {
        return server;
    };
    if let Some(name) = config.server.name {
        if !name.trim().is_empty() {
            server.name = name.trim().to_string();
        }
    }
    let port = match config.server.relay_port {
        Some(port) if port != 0 => port,
        _ => vterm_common::DEFAULT_SERVER_PORT,
    };
    server.url = format!("wss://localhost:{port}");
    // The local server.json usually stores the secret encrypted (enc-v2) or
    // hashed; only a plaintext value can be used directly.
    if let Some(secret) = config.server.secret_key {
        let secret = secret.trim().to_string();
        if !secret.is_empty() && !secret.starts_with("enc-v") && !secret.starts_with("sha256:") {
            server.secret_key = secret;
        }
    }
    server
}

#[derive(Debug, Deserialize)]
struct ClientConfigFile {
    #[serde(default)]
    servers: Vec<ClientManagedServerEntry>,
}

#[derive(Debug, Deserialize)]
struct ClientManagedServerEntry {
    #[serde(default)]
    #[allow(dead_code)] // kept for schema compatibility / future use
    id: Option<String>,
    #[serde(default)]
    name: String,
    #[serde(default)]
    host: String,
    #[serde(default)]
    secret_key: String,
}

#[derive(Debug, Deserialize)]
struct ServerConfigFile {
    #[serde(default)]
    server: ServerEntry,
}

impl Default for ServerConfigFile {
    fn default() -> Self {
        Self {
            server: ServerEntry::default(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct ServerEntry {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    relay_port: Option<u16>,
    #[serde(default)]
    secret_key: Option<String>,
}

fn client_config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var(CLIENT_CONFIG_FILE_ENV) {
        return Ok(PathBuf::from(path));
    }
    Ok(config_dir()?.join(CLIENT_CONFIG_FILE))
}

fn server_config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(SERVER_CONFIG_FILE))
}

fn config_dir() -> Result<PathBuf> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .context("unable to determine user home directory")?;
    Ok(home.join(DEFAULT_CONFIG_DIR))
}

/// Validate a `wss://host:port` address (IPv6 literals must be bracketed,
/// e.g. `wss://[::1]:7878`). Returns the normalized address.
pub fn validate_server_url(input: &str) -> Result<String> {
    let input = input.trim();
    let Some(rest) = input.strip_prefix("wss://") else {
        return Err(anyhow::anyhow!("Address must be in wss://host:port format"));
    };
    if rest.is_empty() || rest.contains(['/', '?', '#']) {
        return Err(anyhow::anyhow!(
            "Address must be wss://host:port (no path or query)"
        ));
    }

    let (host, port_str) = if let Some(inner) = rest.strip_prefix('[') {
        let Some((host, tail)) = inner.split_once(']') else {
            return Err(anyhow::anyhow!("Unterminated IPv6 address literal"));
        };
        let Some(port_str) = tail.strip_prefix(':') else {
            return Err(anyhow::anyhow!("Missing :port"));
        };
        (host.to_string(), port_str.to_string())
    } else {
        let Some((host, port_str)) = rest.split_once(':') else {
            return Err(anyhow::anyhow!("Missing :port"));
        };
        if host.contains(':') {
            return Err(anyhow::anyhow!(
                "IPv6 addresses must be bracketed: wss://[::1]:7878"
            ));
        }
        (host.to_string(), port_str.to_string())
    };

    if host.is_empty() || host.contains(['[', ']']) {
        return Err(anyhow::anyhow!("Invalid host"));
    }
    let port: u16 = port_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid port '{port_str}'"))?;
    if port == 0 {
        return Err(anyhow::anyhow!("Port must be between 1 and 65535"));
    }

    if host.contains(':') {
        Ok(format!("wss://[{host}]:{port}"))
    } else {
        Ok(format!("wss://{host}:{port}"))
    }
}

/// Persist a managed server in `client.json`, replacing any existing entry
/// with the same address (same rule the GUI edit form uses: same host means
/// same server). Loopback addresses are allowed: under WSL `localhost`
/// reaches the Windows host, so such an entry is not the same machine as
/// the implicit Local server. All other config fields are preserved.
/// Returns the saved entry.
pub fn add_server(name: &str, url: &str, secret_key: &str) -> Result<KnownServer> {
    let url = validate_server_url(url)?;
    let name = if name.trim().is_empty() {
        url.clone()
    } else {
        name.trim().to_string()
    };

    let path = client_config_path()?;
    let mut root = read_config_root(&path)?;
    ensure_servers_array(&mut root);
    let servers = root
        .get_mut("servers")
        .and_then(|value| value.as_array_mut())
        .expect("servers array just ensured");

    let existing_idx = servers
        .iter()
        .position(|entry| entry.get("host").and_then(|h| h.as_str()) == Some(url.as_str()));
    match existing_idx {
        Some(index) => {
            // Update in place, keeping the stable id.
            let entry = &mut servers[index];
            entry["name"] = serde_json::Value::String(name.clone());
            entry["host"] = serde_json::Value::String(url.clone());
            entry["secret_key"] = serde_json::Value::String(secret_key.to_string());
            if entry
                .get("id")
                .and_then(|id| id.as_str())
                .map_or(true, |id| id.trim().is_empty() || id == "local")
            {
                entry["id"] = serde_json::Value::String(uuid::Uuid::new_v4().to_string());
            }
        }
        None => servers.push(serde_json::json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "name": name,
            "host": url,
            "secret_key": secret_key,
        })),
    }
    write_config_root(&path, &root)?;

    Ok(KnownServer {
        name,
        url,
        secret_key: secret_key.to_string(),
        local: false,
    })
}

/// Remove every managed entry whose address matches `url`. Returns the
/// number of entries removed (0 = nothing matched).
pub fn remove_server(url: &str) -> Result<usize> {
    let url = url.trim().to_ascii_lowercase();
    let path = client_config_path()?;
    if !path.exists() {
        return Ok(0);
    }
    let mut root = read_config_root(&path)?;
    let Some(servers) = root
        .get_mut("servers")
        .and_then(|value| value.as_array_mut())
    else {
        return Ok(0);
    };
    let before = servers.len();
    servers.retain(|entry| {
        entry
            .get("host")
            .and_then(|host| host.as_str())
            .map(|host| host.trim().to_ascii_lowercase() != url)
            .unwrap_or(true)
    });
    let removed = before - servers.len();
    if removed > 0 {
        write_config_root(&path, &root)?;
    }
    Ok(removed)
}

/// Read `client.json` as raw JSON so fields the CLI does not know about
/// (theme, fonts, …) survive a round trip.
fn read_config_root(path: &Path) -> Result<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::json!({
            "version": CONFIG_VERSION,
            "servers": [],
        }));
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let root: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    if !root.is_object() {
        return Err(anyhow::anyhow!("{} is not a JSON object", path.display()));
    }
    Ok(root)
}

fn ensure_servers_array(root: &mut serde_json::Value) {
    let ok = root
        .get("servers")
        .map(|servers| servers.is_array())
        .unwrap_or(false);
    if !ok {
        root["servers"] = serde_json::Value::Array(Vec::new());
    }
}

fn write_config_root(path: &Path, root: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, serde_json::to_vec_pretty(root)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_config_parses_managed_servers() {
        let json = r#"{
            "version": 1,
            "servers": [
                { "id": "s1", "name": "Office", "host": "wss://office.example.com:7878", "secret_key": "abc" },
                { "id": "s2", "name": "", "host": "wss://backup.example.com:7878", "secret_key": "" },
                { "id": "s3", "name": "Loopback", "host": "wss://localhost:7878", "secret_key": "" }
            ]
        }"#;
        let config: ClientConfigFile = serde_json::from_str(json).unwrap();
        assert_eq!(config.servers.len(), 3);
        assert_eq!(config.servers[0].name, "Office");
        assert_eq!(config.servers[1].host, "wss://backup.example.com:7878");
        assert_eq!(config.servers[2].host, "wss://localhost:7878");
    }

    #[test]
    fn server_labels() {
        let local = KnownServer {
            name: "Dev-PC".to_string(),
            url: "wss://localhost:7878".to_string(),
            secret_key: String::new(),
            local: true,
        };
        assert_eq!(local.label(), "Local");

        let managed = KnownServer {
            name: "Office".to_string(),
            url: "wss://office.example.com:7878".to_string(),
            secret_key: "abc".to_string(),
            local: false,
        };
        assert_eq!(managed.label(), "Office  (wss://office.example.com:7878)");
    }

    #[test]
    fn url_validation() {
        assert_eq!(
            validate_server_url("wss://office.example.com:7878").unwrap(),
            "wss://office.example.com:7878"
        );
        assert_eq!(
            validate_server_url("  wss://[fd00::1a]:8781 ").unwrap(),
            "wss://[fd00::1a]:8781"
        );
        assert_eq!(
            validate_server_url("wss://192.168.1.4:7878").unwrap(),
            "wss://192.168.1.4:7878"
        );

        // Wrong scheme / missing port / junk.
        assert!(validate_server_url("ws://office.example.com:7878").is_err());
        assert!(validate_server_url("office.example.com:7878").is_err());
        assert!(validate_server_url("wss://office.example.com").is_err());
        assert!(validate_server_url("wss://office.example.com:0").is_err());
        assert!(validate_server_url("wss://office.example.com:99999").is_err());
        assert!(validate_server_url("wss://office.example.com:7878/path").is_err());
        // Unbracketed IPv6 is ambiguous and rejected.
        assert!(validate_server_url("wss://fd00::1a:8781").is_err());
        assert!(validate_server_url("wss://[fd00::1a").is_err());
    }

    #[test]
    fn add_and_remove_server_round_trip() {
        let dir = std::env::temp_dir().join(format!("vterm-servers-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let config_path = dir.join(CLIENT_CONFIG_FILE);
        // Keep the rest of the config intact across our edits.
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            &config_path,
            r#"{ "version": 1, "theme_mode": "dark", "servers": [] }"#,
        )
        .unwrap();
        std::env::set_var(CLIENT_CONFIG_FILE_ENV, &config_path);

        let added = add_server("Office", "wss://office.example.com:7878", "key1").unwrap();
        assert_eq!(added.label(), "Office  (wss://office.example.com:7878)");

        let servers = load_servers().unwrap();
        assert_eq!(servers.len(), 2);
        assert!(servers[0].local);
        assert_eq!(servers[1].name, "Office");
        assert_eq!(servers[1].secret_key, "key1");

        // Adding the same address again updates instead of duplicating.
        add_server("Renamed", "WSS://Office.Example.Com:7878", "").unwrap_err();
        add_server("Renamed", "wss://office.example.com:7878", "key2").unwrap();
        let servers = load_servers().unwrap();
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[1].name, "Renamed");
        assert_eq!(servers[1].secret_key, "key2");

        // Loopback addresses are allowed (under WSL `localhost` reaches the
        // Windows host) and show up alongside the implicit Local entry.
        let home = add_server("Home", "wss://localhost:7878", "key3").unwrap();
        assert_eq!(home.label(), "Home  (wss://localhost:7878)");
        let servers = load_servers().unwrap();
        assert_eq!(servers.len(), 3);
        assert_eq!(servers[2].name, "Home");
        assert_eq!(servers[2].secret_key, "key3");

        // Unknown URLs are preserved untouched.
        assert_eq!(remove_server("wss://nope.example.com:1").unwrap(), 0);

        assert_eq!(remove_server("wss://office.example.com:7878").unwrap(), 1);
        assert_eq!(remove_server("wss://localhost:7878").unwrap(), 1);
        assert_eq!(load_servers().unwrap().len(), 1);

        // The unrelated theme field survived every write.
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"theme_mode\": \"dark\""));

        std::env::remove_var(CLIENT_CONFIG_FILE_ENV);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
