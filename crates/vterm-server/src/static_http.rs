use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

use anyhow::{bail, Context, Result};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{watch, Mutex};
use tracing::{debug, info, warn};

use crate::config::{ConfigManager, StaticHttpBindAddress};
use crate::ServerTlsAcceptor;

pub struct StaticHttpManager {
    config_manager: Arc<ConfigManager>,
    /// Shared with the terminal WebSocket listener: TLS is terminated with
    /// the same managed/configured identity (cloned `Arc<ServerConfig>`,
    /// never re-read from disk per connection).
    tls_acceptor: ServerTlsAcceptor,
    hosted: Mutex<HashMap<PathBuf, HostedService>>,
}

struct HostedService {
    port: u16,
    share_token: String,
    stop_tx: watch::Sender<bool>,
    /// Serve tasks owning the listeners. Awaited on stop so the ports are
    /// actually released before `stop_directory` returns — a restart can
    /// then rebind the very same port instead of drifting to the next one
    /// in the range.
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

pub(crate) struct HostedDirectory {
    pub(crate) port: u16,
    pub(crate) share_token: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HostedHttpService {
    pub port: u16,
    pub root: String,
    pub url: String,
}
pub type SharedHostedHttpServices = Arc<RwLock<Vec<HostedHttpService>>>;
static HOSTED_SERVICES: OnceLock<SharedHostedHttpServices> = OnceLock::new();

/// Render a path for display. On Windows, `canonicalize()` returns verbatim
/// paths (`\\?\D:\foo` or `\\?\UNC\server\share`); users expect the plain
/// form (`D:\foo`, `\\server\share`). Paths that genuinely need the prefix
/// (non-drive-letter device paths) are left untouched.
fn display_path(path: &Path) -> String {
    let text = path.display().to_string();
    #[cfg(windows)]
    {
        for prefix in [r"\\?\", r"\\.\"] {
            if let Some(rest) = text.strip_prefix(prefix) {
                if let Some(unc) = rest.strip_prefix(r"UNC\") {
                    return format!(r"\\{unc}");
                }
                let bytes = rest.as_bytes();
                if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
                    return rest.to_string();
                }
            }
        }
    }
    text
}

pub fn hosted_services() -> SharedHostedHttpServices {
    HOSTED_SERVICES
        .get_or_init(|| Arc::new(RwLock::new(Vec::new())))
        .clone()
}

impl StaticHttpManager {
    pub fn new(config_manager: Arc<ConfigManager>, tls_acceptor: ServerTlsAcceptor) -> Self {
        Self {
            config_manager,
            tls_acceptor,
            hosted: Mutex::new(HashMap::new()),
        }
    }

    pub async fn share_token_for_port(&self, port: u16) -> Option<String> {
        self.hosted
            .lock()
            .await
            .values()
            .find(|service| service.port == port)
            .map(|service| service.share_token.clone())
    }

    pub async fn host_directory(&self, directory: &str) -> Result<HostedDirectory> {
        let root = PathBuf::from(directory)
            .canonicalize()
            .with_context(|| format!("Hosted directory does not exist: {directory}"))?;
        if !root.is_dir() {
            bail!("Hosted path is not a directory: {}", root.display());
        }

        let server = self.config_manager.snapshot().await.server;
        let mut hosted = self.hosted.lock().await;
        if let Some(service) = hosted.get(&root) {
            info!(root = %root.display(), port = service.port, "static HTTP server is already running");
            return Ok(HostedDirectory {
                port: service.port,
                share_token: service.share_token.clone(),
            });
        }
        info!(root = %root.display(), port_start = server.static_http_port_start, port_end = server.static_http_port_end, bind_address = ?server.static_http_bind_address, "starting static HTTP server");
        let listeners = match bind_available(
            server.static_http_port_start,
            server.static_http_port_end,
            server.static_http_bind_address,
        )
        .await
        {
            Ok(listeners) => listeners,
            Err(error) => {
                warn!(root = %root.display(), error = %error, "failed to start static HTTP server");
                return Err(error);
            }
        };
        let port = listeners[0].local_addr()?.port();
        let (access, share_token) = crate::http_auth::HttpAccess::new_static(
            std::time::Duration::from_secs(7 * 24 * 60 * 60),
        );
        let access = Arc::new(access);
        let (stop_tx, stop_rx) = watch::channel(false);
        let mut tasks = Vec::with_capacity(listeners.len());
        for listener in listeners {
            let root = root.clone();
            let stop_rx = stop_rx.clone();
            let tls_acceptor = self.tls_acceptor.clone();
            let access = Arc::clone(&access);
            tasks.push(tokio::spawn(async move {
                serve(listener, root, tls_acceptor, stop_rx, access).await;
            }));
        }
        hosted.insert(
            root.clone(),
            HostedService {
                port,
                share_token: share_token.clone(),
                stop_tx,
                tasks,
            },
        );
        drop(hosted);
        hosted_services()
            .write()
            .expect("hosted HTTP services poisoned")
            .push(HostedHttpService {
                port,
                root: display_path(&root),
                url: format!("https://127.0.0.1:{port}/"),
            });
        info!(root = %root.display(), port, "static HTTP server started");
        Ok(HostedDirectory { port, share_token })
    }

    pub async fn stop_directory(&self, root: &str) -> bool {
        // Callers pass the display form (tray menu) or an arbitrary path
        // (config API); the `hosted` map is keyed by canonicalized paths.
        let root = PathBuf::from(root)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(root));
        let display_root = display_path(&root);
        info!(root = %display_root, "stopping static HTTP server");
        let service = self.hosted.lock().await.remove(&root);
        let Some(service) = service else {
            warn!(root = %display_root, "static HTTP server was not running when stop was requested");
            return false;
        };
        let port = service.port;
        let _ = service.stop_tx.send(true);
        // Wait for the serve tasks to drop their listeners: only then are
        // the ports free again. This keeps a stop→start cycle on the same
        // port deterministic (restart_running relies on it). A panic inside
        // a serve task surfaces as a JoinError and is ignored — stopping is
        // best-effort cleanup.
        for task in service.tasks {
            let _ = task.await;
        }
        hosted_services()
            .write()
            .expect("hosted HTTP services poisoned")
            .retain(|service| service.root != display_root);
        info!(root = %display_root, port, "static HTTP server stopped");
        true
    }

    /// Restart every hosted service so it picks up the *current* bind
    /// address and port range. Running services keep the listeners they were
    /// started with, so a settings change must rebind them to take effect.
    /// Each root is handled independently: if one fails to rebind it is
    /// stopped with a warning and the others continue.
    pub async fn restart_running(&self) {
        let roots: Vec<PathBuf> = self.hosted.lock().await.keys().cloned().collect();
        if roots.is_empty() {
            return;
        }
        info!(
            count = roots.len(),
            "static HTTP settings changed; restarting hosted services"
        );
        for root in roots {
            let root = root.display().to_string();
            self.stop_directory(&root).await;
            if let Err(error) = self.host_directory(&root).await {
                warn!(
                    root = %root,
                    error = %error,
                    "failed to restart static HTTP server after settings change"
                );
            }
        }
    }

    /// Watch for config changes that affect running static HTTP services
    /// (bind address / port range) and hot-restart them. Mirrors
    /// `McpServer::run` and `AsrSupervisor::run`: without this, a
    /// bind-address change in the webui would only apply to services
    /// started *after* the change, never to already-running ones.
    pub async fn run(self: Arc<Self>) {
        let mut config_rx = self.config_manager.subscribe();
        let mut last = {
            let server = &config_rx.borrow().server;
            (
                server.static_http_bind_address,
                server.static_http_port_start,
                server.static_http_port_end,
            )
        };
        loop {
            if config_rx.changed().await.is_err() {
                return;
            }
            let server = config_rx.borrow().server.clone();
            let current = (
                server.static_http_bind_address,
                server.static_http_port_start,
                server.static_http_port_end,
            );
            let settings_changed = current != last;
            last = current;
            if settings_changed {
                self.restart_running().await;
            }
        }
    }
}

async fn bind_available(
    start: u16,
    end: u16,
    bind_address: StaticHttpBindAddress,
) -> Result<Vec<TcpListener>> {
    let v4_base = match bind_address {
        StaticHttpBindAddress::Local => Ipv4Addr::LOCALHOST,
        StaticHttpBindAddress::Any => Ipv4Addr::UNSPECIFIED,
    };
    let v6_base = match bind_address {
        StaticHttpBindAddress::Local => Ipv6Addr::LOCALHOST,
        StaticHttpBindAddress::Any => Ipv6Addr::UNSPECIFIED,
    };
    for port in start..=end {
        let v4 = match bind_listener(SocketAddr::from((v4_base, port)), false).await {
            Ok(listener) => listener,
            Err(error) if port_unavailable(&error) => {
                debug!(port, error = %error, "static HTTP port unavailable, trying next");
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        let v6 = match bind_listener(SocketAddr::from((v6_base, port)), true).await {
            Ok(listener) => listener,
            Err(error) if port_unavailable(&error) => {
                debug!(port, error = %error, "static HTTP port unavailable, trying next");
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        return Ok(vec![v4, v6]);
    }
    // The whole configured range can be unavailable — most commonly on
    // Windows, where WinNAT/Hyper-V excluded ranges (bind fails with os
    // error 10013) can cover it entirely and move between reboots. Fall
    // back to an OS-assigned port so hosting still works; the v6 listener
    // on that port stays best-effort.
    warn!("no available static HTTP port in range {start}-{end}; using an OS-assigned port");
    let v4 = bind_listener(SocketAddr::from((v4_base, 0)), false).await?;
    let port = v4.local_addr()?.port();
    let listeners = match bind_listener(SocketAddr::from((v6_base, port)), true).await {
        Ok(v6) => vec![v4, v6],
        Err(_) => vec![v4],
    };
    Ok(listeners)
}

/// Bind failures that mean "this port is unavailable" rather than "binding
/// is broken": `AddrInUse` (another process holds the port) and
/// `PermissionDenied` (Windows WSAEACCES / os error 10013 — the port sits
/// inside a WinNAT/Hyper-V excluded range; those reservations move between
/// reboots, so a configured range can be blocked in part or whole).
fn port_unavailable(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::AddrInUse | std::io::ErrorKind::PermissionDenied
    )
}

async fn bind_listener(address: SocketAddr, v6_only: bool) -> std::io::Result<TcpListener> {
    let socket = Socket::new(
        if address.is_ipv4() {
            Domain::IPV4
        } else {
            Domain::IPV6
        },
        Type::STREAM,
        Some(Protocol::TCP),
    )?;
    if address.is_ipv6() {
        socket.set_only_v6(v6_only)?;
    }
    socket.bind(&address.into())?;
    socket.listen(1024)?;
    socket.set_nonblocking(true)?;
    TcpListener::from_std(socket.into())
}

async fn serve(
    listener: TcpListener,
    root: PathBuf,
    tls_acceptor: ServerTlsAcceptor,
    mut stop_rx: watch::Receiver<bool>,
    access: Arc<crate::http_auth::HttpAccess>,
) {
    let address = listener.local_addr().ok();
    loop {
        let accepted = tokio::select! {
            result = listener.accept() => result,
            _ = stop_rx.changed() => break,
        };
        let (stream, peer) = match accepted {
            Ok(accepted) => accepted,
            Err(error) => {
                warn!(root = %root.display(), address = ?address, error = %error, "static HTTP listener accept failed");
                break;
            }
        };
        let root = root.clone();
        let tls_acceptor = tls_acceptor.clone();
        let access = Arc::clone(&access);
        tokio::spawn(async move {
            // Protocol sniffing: a TLS ClientHello starts with the handshake
            // record byte 0x16, an HTTP request line with an ASCII method.
            // Plain HTTP is accepted only from loopback peers (localhost /
            // 127.0.0.1 / ::1) — which also covers the server's reverse
            // proxy, whose remote "Host with HTTP server" shares terminate
            // the client TLS themselves and forward decrypted bytes from
            // 127.0.0.1 to this port. Plain HTTP from any other address is
            // refused: remote clients must use HTTPS.
            let mut first = [0u8; 1];
            let mut stream = stream;
            match stream.peek(&mut first).await {
                Ok(1) if first[0] == 0x16 => match tls_acceptor.accept(stream).await {
                    Ok(mut tls_stream) => {
                        let _ = handle(&mut tls_stream, root, &access).await;
                        // The response head says `Connection: close`; finish
                        // the TLS shutdown too so rustls clients do not see
                        // an unexpected EOF (missing close_notify).
                        let _ = tls_stream.shutdown().await;
                    }
                    // A failed handshake (port scanners, https misprobes) is
                    // routine noise: log at debug and drop the connection.
                    Err(error) => {
                        debug!(root = %root.display(), peer = %peer, error = %error, "static HTTP TLS handshake failed");
                    }
                },
                Ok(_) if peer.ip().is_loopback() => {
                    let _ = handle(&mut stream, root, &access).await;
                }
                Ok(_) => {
                    let _ = send(
                        &mut stream,
                        "403 Forbidden",
                        &[("Content-Type", "text/plain; charset=utf-8")],
                        b"HTTPS is required for non-loopback clients",
                    )
                    .await;
                }
                Err(error) => {
                    debug!(root = %root.display(), peer = %peer, error = %error, "static HTTP connection peek failed");
                }
            }
        });
    }
    info!(root = %root.display(), address = ?address, "static HTTP listener stopped");
}

async fn handle<S>(
    mut stream: S,
    root: PathBuf,
    access: &crate::http_auth::HttpAccess,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut request = vec![0u8; 16 * 1024];
    let count = stream.read(&mut request).await?;
    let request = String::from_utf8_lossy(&request[..count]);
    let mut lines = request.lines();
    let Some(first) = lines.next() else {
        return Ok(());
    };
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/");
    let headers = lines.collect::<Vec<_>>();
    let range = headers
        .iter()
        .find_map(|line| {
            line.strip_prefix("Range:")
                .or_else(|| line.strip_prefix("range:"))
        })
        .and_then(|value| value.trim().strip_prefix("bytes="))
        .map(str::to_string);
    let cookie = headers.iter().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("cookie")
            .then(|| value.trim().to_string())
    });
    if !access.has_valid_cookie(cookie.as_deref()) {
        if let Some(token) = crate::http_auth::query_param(target, "share_token") {
            if let Some(session) = access.exchange_grant(&token) {
                let location = crate::http_auth::without_query_param(target, "share_token");
                let session_cookie = access.session_cookie(&session, true);
                return send(
                    &mut stream,
                    "303 See Other",
                    &[
                        ("Location", location.as_str()),
                        ("Set-Cookie", session_cookie.as_str()),
                        ("Cache-Control", "no-store"),
                        ("Referrer-Policy", "no-referrer"),
                    ],
                    &[],
                )
                .await;
            }
        }
        stream.shutdown().await?;
        return Ok(());
    }
    if method != "GET" && method != "HEAD" {
        return send(
            &mut stream,
            "405 Method Not Allowed",
            &[("Allow", "GET, HEAD")],
            &[],
        )
        .await;
    }
    let path = match resolve_path(&root, target) {
        Some(path) => path,
        None => return send(&mut stream, "403 Forbidden", &[], b"Forbidden").await,
    };
    if path.is_dir() {
        return directory_response(&mut stream, &root, &path, method == "HEAD").await;
    }
    if !path.is_file() {
        return send(&mut stream, "404 Not Found", &[], b"Not found").await;
    }
    file_response(&mut stream, &path, range.as_deref(), method == "HEAD").await
}

fn resolve_path(root: &Path, target: &str) -> Option<PathBuf> {
    let raw = target.split('?').next().unwrap_or("/");
    let mut path = root.to_path_buf();
    for part in raw.trim_start_matches('/').split('/') {
        if part.is_empty() {
            continue;
        }
        let part = percent_decode(part)?;
        if part.contains(['/', '\\']) || part == "." || part == ".." {
            return None;
        }
        path.push(part);
    }
    let candidate = path.canonicalize().ok()?;
    candidate.starts_with(root).then_some(candidate)
}

fn percent_decode(value: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut chars = value.as_bytes().iter().copied();
    while let Some(byte) = chars.next() {
        if byte == b'%' {
            let hi = chars.next()?;
            let lo = chars.next()?;
            bytes.push((hex(hi)? << 4) | hex(lo)?);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).ok()
}
fn hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

async fn directory_response<S>(stream: &mut S, root: &Path, path: &Path, head: bool) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let index = path.join("index.html");
    if index.is_file() {
        return file_response(stream, &index, None, head).await;
    }
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let mut body = format!("<!doctype html><meta charset=utf-8><title>Index of /{relative}</title><h1>Index of /{relative}</h1><ul>");
    if path != root {
        body.push_str("<li><a href=\"../\">../</a></li>");
    }
    let mut entries = std::fs::read_dir(path)?.flatten().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let suffix = entry.path().is_dir().then_some("/").unwrap_or("");
        body.push_str(&format!(
            "<li><a href=\"{}{}\">{}{} </a></li>",
            url_encode(&name),
            suffix,
            html_escape(&name),
            suffix
        ));
    }
    body.push_str("</ul>");
    send(
        stream,
        "200 OK",
        &[("Content-Type", "text/html; charset=utf-8")],
        if head { &[] } else { body.as_bytes() },
    )
    .await
}

async fn file_response<S>(
    stream: &mut S,
    path: &Path,
    range: Option<&str>,
    head: bool,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let size = tokio::fs::metadata(path).await?.len();
    let (start, end, status) = match range.and_then(|range| parse_range(range, size)) {
        Some((start, end)) => (start, end, "206 Partial Content"),
        None => (0, size.saturating_sub(1), "200 OK"),
    };
    if range.is_some() && size > 0 && status == "200 OK" {
        return send_owned(
            stream,
            "416 Range Not Satisfiable",
            vec![("Content-Range".to_string(), format!("bytes */{size}"))],
            &[],
        )
        .await;
    }
    let length = if size == 0 { 0 } else { end - start + 1 };
    let mut owned_headers = vec![
        ("Content-Type".to_string(), mime(path).to_string()),
        ("Accept-Ranges".to_string(), "bytes".to_string()),
        ("Content-Length".to_string(), length.to_string()),
    ];
    if status == "206 Partial Content" {
        owned_headers.push((
            "Content-Range".to_string(),
            format!("bytes {start}-{end}/{size}"),
        ));
    }
    let refs = owned_headers
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect::<Vec<_>>();
    write_head(stream, status, &refs).await?;
    if !head && length > 0 {
        let mut file = tokio::fs::File::open(path).await?;
        file.seek(std::io::SeekFrom::Start(start)).await?;
        let mut remaining = length;
        let mut buffer = [0u8; 64 * 1024];
        while remaining > 0 {
            let chunk_len = remaining.min(buffer.len() as u64) as usize;
            let read = file.read(&mut buffer[..chunk_len]).await?;
            if read == 0 {
                break;
            }
            stream.write_all(&buffer[..read]).await?;
            remaining -= read as u64;
        }
    }
    Ok(())
}

fn parse_range(value: &str, size: u64) -> Option<(u64, u64)> {
    let (a, b) = value.split_once('-')?;
    if size == 0 {
        return None;
    }
    if a.is_empty() {
        let n = b.parse::<u64>().ok()?.min(size);
        return Some((size - n, size - 1));
    }
    let start = a.parse().ok()?;
    let end = if b.is_empty() {
        size - 1
    } else {
        b.parse::<u64>().ok()?.min(size - 1)
    };
    (start <= end && start < size).then_some((start, end))
}
async fn send<S>(stream: &mut S, status: &str, headers: &[(&str, &str)], body: &[u8]) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    send_owned(
        stream,
        status,
        headers
            .iter()
            .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
            .collect(),
        body,
    )
    .await
}
async fn send_owned<S>(
    stream: &mut S,
    status: &str,
    mut headers: Vec<(String, String)>,
    body: &[u8],
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    headers.push(("Content-Length".to_string(), body.len().to_string()));
    let refs = headers
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect::<Vec<_>>();
    write_head(stream, status, &refs).await?;
    stream.write_all(body).await?;
    Ok(())
}
async fn write_head<S>(stream: &mut S, status: &str, headers: &[(&str, &str)]) -> Result<()>
where
    S: AsyncWrite + Unpin,
{
    let mut head = format!("HTTP/1.1 {status}\r\nConnection: close\r\nCache-Control: no-store\r\n");
    for (name, value) in headers {
        head.push_str(name);
        head.push_str(": ");
        head.push_str(value);
        head.push_str("\r\n");
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).await?;
    Ok(())
}
fn mime(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "txt" | "log" => "text/plain; charset=utf-8",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        _ => "application/octet-stream",
    }
}
fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn url_encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_') {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn display_path_strips_windows_verbatim_prefix() {
        assert_eq!(
            display_path(Path::new(r"\\?\D:\projects\site")),
            r"D:\projects\site"
        );
        assert_eq!(
            display_path(Path::new(r"\\.\D:\projects\site")),
            r"D:\projects\site"
        );
        assert_eq!(
            display_path(Path::new(r"\\?\UNC\server\share")),
            r"\\server\share"
        );
        // Non-drive-letter device paths keep their prefix.
        assert_eq!(
            display_path(Path::new(r"\\?\GLOBALROOT\Device")),
            r"\\?\GLOBALROOT\Device"
        );
        // Plain paths pass through unchanged.
        assert_eq!(
            display_path(Path::new(r"D:\projects\site")),
            r"D:\projects\site"
        );
    }

    /// In-memory self-signed identity for the manager's TLS path; the
    /// reachability probes below only open TCP connections, so no
    /// client-side verification is involved.
    fn test_tls_acceptor() -> ServerTlsAcceptor {
        let certified_key = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(
            certified_key.signing_key.serialize_der().into(),
        );
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certified_key.cert.der().clone()], key)
            .unwrap();
        ServerTlsAcceptor::from(Arc::new(config))
    }

    #[test]
    fn port_unavailable_covers_in_use_and_windows_reserved_ranges() {
        assert!(port_unavailable(&std::io::Error::from(
            std::io::ErrorKind::AddrInUse
        )));
        assert!(port_unavailable(&std::io::Error::from(
            std::io::ErrorKind::PermissionDenied
        )));
        assert!(!port_unavailable(&std::io::Error::from(
            std::io::ErrorKind::AddrNotAvailable
        )));
        assert!(!port_unavailable(&std::io::Error::other("bind broken")));
    }

    #[test]
    fn windows_wseacces_maps_to_permission_denied() {
        // os error 10013 is what a WinNAT/Hyper-V excluded range produces on
        // Windows; the scan must treat it as "port taken".
        assert_eq!(
            std::io::Error::from_raw_os_error(10013).kind(),
            std::io::ErrorKind::PermissionDenied
        );
    }

    /// A non-loopback IPv4 address, used to observe the Local/Any bind
    /// distinction. `None` on machines without one (some CI runners) — the
    /// hot-restart test skips there.
    fn non_loopback_ipv4() -> Option<std::net::Ipv4Addr> {
        let interfaces = if_addrs::get_if_addrs().ok()?;
        interfaces
            .into_iter()
            .filter(|interface| !interface.is_loopback())
            .find_map(|interface| match interface.addr {
                if_addrs::IfAddr::V4(address) => Some(address.ip),
                if_addrs::IfAddr::V6(_) => None,
            })
    }

    #[tokio::test]
    async fn serves_https_and_plain_http_on_the_same_port() {
        let port: u16 = 40_000 + (std::process::id() % 20_000) as u16;
        let tag = format!("static-http-tls-{}", std::process::id());
        let config_path = std::env::temp_dir().join(format!("vterm-server-config-{tag}.json"));
        let root_dir = std::env::temp_dir().join(format!("vterm-static-http-root-{tag}"));
        let _ = tokio::fs::remove_file(&config_path).await;
        let _ = tokio::fs::remove_dir_all(&root_dir).await;
        tokio::fs::create_dir_all(&root_dir).await.unwrap();
        tokio::fs::write(root_dir.join("hello.txt"), "hello over tls")
            .await
            .unwrap();

        let manager = Arc::new(
            ConfigManager::load_from_path(config_path.clone())
                .await
                .unwrap(),
        );
        {
            let mut config = manager.snapshot().await;
            config.server.static_http_bind_address = StaticHttpBindAddress::Local;
            config.server.static_http_port_start = port;
            config.server.static_http_port_end = port;
            manager.replace(config).await.unwrap();
        }

        let static_http = Arc::new(StaticHttpManager::new(
            Arc::clone(&manager),
            test_tls_acceptor(),
        ));
        let root = root_dir.canonicalize().unwrap().display().to_string();
        let hosted = static_http.host_directory(&root).await.unwrap();
        let hosted_port = hosted.port;
        assert_eq!(
            static_http.share_token_for_port(hosted_port).await.as_deref(),
            Some(hosted.share_token.as_str())
        );

        let mut denied = tokio::net::TcpStream::connect(("127.0.0.1", hosted_port))
            .await
            .unwrap();
        denied
            .write_all(b"GET /hello.txt HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();
        let mut response = Vec::new();
        denied.read_to_end(&mut response).await.unwrap();
        assert!(
            response.is_empty(),
            "unauthorized requests receive no bytes"
        );

        let mut denied_post = tokio::net::TcpStream::connect(("127.0.0.1", hosted_port))
            .await
            .unwrap();
        denied_post
            .write_all(b"POST /hello.txt HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
        let mut response = Vec::new();
        denied_post.read_to_end(&mut response).await.unwrap();
        assert!(
            response.is_empty(),
            "method checks must not precede authorization"
        );

        let mut bootstrap = tokio::net::TcpStream::connect(("127.0.0.1", hosted_port))
            .await
            .unwrap();
        bootstrap
            .write_all(
                format!(
                    "GET /hello.txt?share_token={} HTTP/1.1\r\nHost: localhost\r\n\r\n",
                    hosted.share_token
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        let mut response = Vec::new();
        bootstrap.read_to_end(&mut response).await.unwrap();
        let response = String::from_utf8_lossy(&response);
        assert!(response.starts_with("HTTP/1.1 303 See Other"));
        let session_cookie = response
            .lines()
            .find_map(|line| line.strip_prefix("Set-Cookie: "))
            .and_then(|value| value.split(';').next())
            .unwrap()
            .to_string();

        // https: full handshake (verification off — the identity is the
        // in-memory self-signed one) then a plain GET inside the tunnel.
        let connector = tokio_rustls::TlsConnector::from(
            vterm_common::tls::insecure_client_tls_config().unwrap(),
        );
        let server_name = rustls::pki_types::ServerName::try_from("localhost".to_string()).unwrap();
        let tcp = tokio::net::TcpStream::connect(("127.0.0.1", hosted_port))
            .await
            .unwrap();
        let mut tls = connector.connect(server_name, tcp).await.unwrap();
        tls.write_all(
            format!(
                "GET /hello.txt HTTP/1.1\r\nHost: localhost\r\nCookie: {session_cookie}\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .unwrap();
        let mut response = Vec::new();
        tls.read_to_end(&mut response).await.unwrap();
        let response = String::from_utf8_lossy(&response);
        assert!(
            response.starts_with("HTTP/1.1 200 OK"),
            "https response: {response}"
        );
        assert!(
            response.ends_with("hello over tls"),
            "https body: {response}"
        );

        // Plain http on the same port from loopback (allowed only for
        // loopback peers; this also covers the reverse proxy, which forwards
        // decrypted bytes from 127.0.0.1 to the target).
        let mut plain = tokio::net::TcpStream::connect(("127.0.0.1", hosted_port))
            .await
            .unwrap();
        plain
            .write_all(
                format!("GET /hello.txt HTTP/1.1\r\nHost: localhost\r\nCookie: {session_cookie}\r\n\r\n")
                    .as_bytes(),
            )
            .await
            .unwrap();
        let mut response = Vec::new();
        plain.read_to_end(&mut response).await.unwrap();
        let response = String::from_utf8_lossy(&response);
        assert!(
            response.starts_with("HTTP/1.1 200 OK"),
            "http response: {response}"
        );

        static_http.stop_directory(&root).await;
        let _ = tokio::fs::remove_file(&config_path).await;
        let _ = tokio::fs::remove_dir_all(&root_dir).await;
    }

    #[tokio::test]
    async fn bind_address_change_hot_restarts_running_service_on_same_port() {
        let Some(lan_ip) = non_loopback_ipv4() else {
            eprintln!("skipping: no non-loopback IPv4 interface");
            return;
        };
        let port: u16 = 20_000 + (std::process::id() % 20_000) as u16;
        let tag = format!("static-http-{}", std::process::id());
        let config_path = std::env::temp_dir().join(format!("vterm-server-config-{tag}.json"));
        let root_dir = std::env::temp_dir().join(format!("vterm-static-http-root-{tag}"));
        let _ = tokio::fs::remove_file(&config_path).await;
        let _ = tokio::fs::remove_dir_all(&root_dir).await;
        tokio::fs::create_dir_all(&root_dir).await.unwrap();

        let manager = Arc::new(
            ConfigManager::load_from_path(config_path.clone())
                .await
                .unwrap(),
        );
        {
            let mut config = manager.snapshot().await;
            config.server.static_http_bind_address = StaticHttpBindAddress::Local;
            config.server.static_http_port_start = port;
            config.server.static_http_port_end = port;
            manager.replace(config).await.unwrap();
        }

        let static_http = Arc::new(StaticHttpManager::new(
            Arc::clone(&manager),
            test_tls_acceptor(),
        ));
        let root = root_dir.canonicalize().unwrap().display().to_string();
        let hosted_port = static_http.host_directory(&root).await.unwrap().port;

        let watcher = {
            let static_http = Arc::clone(&static_http);
            tokio::spawn(async move {
                static_http.run().await;
            })
        };
        // Let the watcher task run to its first await (subscribe) BEFORE any
        // config change: a `watch` receiver does not see sends that happened
        // before it subscribed, and on the current-thread test runtime the
        // spawned task is not polled until we yield.
        tokio::task::yield_now().await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // While bound Local, the LAN interface address is NOT serving ...
        assert!(
            tokio::time::timeout(
                std::time::Duration::from_secs(2),
                tokio::net::TcpStream::connect((lan_ip, hosted_port)),
            )
            .await
            .is_err(),
            "LAN connect must fail while bind address is Local"
        );
        tokio::net::TcpStream::connect(("127.0.0.1", hosted_port))
            .await
            .expect("loopback connect must work while bound Local");

        // ... flipping the config to Any must hot-restart the service.
        {
            let mut config = manager.snapshot().await;
            config.server.static_http_bind_address = StaticHttpBindAddress::Any;
            manager.replace(config).await.unwrap();
        }

        // Wait for the restart to finish rebinding by watching the service
        // list (stop removes the entry, the restart re-adds it). Binding
        // probes here would race the rebind for the same port.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let restarted_on_same_port = loop {
            let same_port = hosted_services()
                .read()
                .expect("hosted HTTP services poisoned")
                .iter()
                .any(|service| {
                    service.root == display_path(Path::new(&root)) && service.port == hosted_port
                });
            if same_port {
                break true;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "service was not restarted on port {hosted_port} within 10s"
            );
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        };
        assert!(restarted_on_same_port);

        // The restarted wildcard listener now serves every interface,
        // including `lan_ip`.
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            tokio::net::TcpStream::connect((lan_ip, hosted_port)),
        )
        .await
        .expect("LAN connect must succeed once the service is bound to Any")
        .expect("LAN connect stream");

        // Still serving, still loopback-reachable ...
        tokio::net::TcpStream::connect(("127.0.0.1", hosted_port))
            .await
            .expect("loopback connect must work after rebind");
        // ... and on the SAME port: stop_directory awaits listener release,
        // so the restart rebinds deterministically instead of drifting to
        // the next port in the range.
        assert_eq!(
            static_http.host_directory(&root).await.unwrap().port,
            hosted_port
        );

        static_http.stop_directory(&root).await;
        watcher.abort();
        let _ = tokio::fs::remove_file(&config_path).await;
        let _ = tokio::fs::remove_dir_all(&root_dir).await;
    }
}
