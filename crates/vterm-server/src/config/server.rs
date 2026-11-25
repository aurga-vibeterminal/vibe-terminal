use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::sync::Arc;

use socket2::{Domain, Protocol, Socket, Type};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

use crate::addon::AddonHost;
use crate::ddns_client::DdnsManager;
use crate::port_forward::PortForwardManager;
use crate::relay::RelayManager;
use crate::screen::ScreenServerSupervisor;
use crate::static_http::StaticHttpManager;
use crate::voice::AsrSupervisor;

use super::api;
use super::store::ConfigManager;

const MAX_REQUEST_SIZE: usize = 50 * 1024 * 1024;
const MAX_HTTP_HEADER_SIZE: usize = 64 * 1024;
const LISTEN_BACKLOG: i32 = 256;

pub struct ConfigHttpServer {
    port: u16,
    config_manager: Arc<ConfigManager>,
    private_ca: Arc<crate::private_ca::PrivateCaManager>,
    relay_manager: Arc<RelayManager>,
    ddns_manager: Arc<DdnsManager>,
    addon_host: Arc<AddonHost>,
    voice_supervisor: Arc<AsrSupervisor>,
    screen_supervisor: Arc<ScreenServerSupervisor>,
    static_http_manager: Arc<StaticHttpManager>,
    port_forward_manager: Arc<PortForwardManager>,
}

impl ConfigHttpServer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        port: u16,
        config_manager: Arc<ConfigManager>,
        private_ca: Arc<crate::private_ca::PrivateCaManager>,
        relay_manager: Arc<RelayManager>,
        ddns_manager: Arc<DdnsManager>,
        addon_host: Arc<AddonHost>,
        voice_supervisor: Arc<AsrSupervisor>,
        screen_supervisor: Arc<ScreenServerSupervisor>,
            static_http_manager: Arc<StaticHttpManager>,
        port_forward_manager: Arc<PortForwardManager>,
        ) -> Self {
        Self {
            port,
            config_manager,
            private_ca,
            relay_manager,
            ddns_manager,
            addon_host,
            voice_supervisor,
            screen_supervisor,
            static_http_manager,
            port_forward_manager,
        }
    }

    pub async fn run(self) -> std::io::Result<()> {
        let listeners = bind_dual_stack(self.port)?;
        info!("Config server listening on http://localhost:{}", self.port);

        // Serve one-time settings tickets to the headless CLI (`--open`) over
        // a local abstract socket — Linux only, see settings_ipc.rs.
        #[cfg(target_os = "linux")]
        crate::settings_ipc::spawn_listener();

        let shared_manager = self.config_manager;
        let shared_private_ca = self.private_ca;
        let shared_relay_manager = self.relay_manager;
        let shared_ddns_manager = self.ddns_manager;
        let shared_addon_host = self.addon_host;
        let shared_voice = self.voice_supervisor;
        let shared_screen = self.screen_supervisor;
        let shared_static_http = self.static_http_manager;
        let shared_port_forward = self.port_forward_manager;
        let mut tasks = tokio::task::JoinSet::new();
        for listener in listeners {
            let config_manager = shared_manager.clone();
            let private_ca = shared_private_ca.clone();
            let relay_manager = shared_relay_manager.clone();
            let ddns_manager = shared_ddns_manager.clone();
            let addon_host = shared_addon_host.clone();
            let voice_supervisor = shared_voice.clone();
            let screen_supervisor = shared_screen.clone();
            let static_http_manager = shared_static_http.clone();
            let port_forward_manager = shared_port_forward.clone();
            tasks.spawn(async move {
                accept_loop(
                    listener,
                    config_manager,
                    private_ca,
                    relay_manager,
                    ddns_manager,
                    addon_host,
                    voice_supervisor,
                    screen_supervisor,
                            static_http_manager,
                    port_forward_manager,
                        )
                .await;
            });
        }

        while tasks.join_next().await.is_some() {}

        Ok(())
    }
}

async fn accept_loop(
    listener: TcpListener,
    config_manager: Arc<ConfigManager>,
    private_ca: Arc<crate::private_ca::PrivateCaManager>,
    relay_manager: Arc<RelayManager>,
    ddns_manager: Arc<DdnsManager>,
    addon_host: Arc<AddonHost>,
    voice_supervisor: Arc<AsrSupervisor>,
    screen_supervisor: Arc<ScreenServerSupervisor>,
    static_http_manager: Arc<StaticHttpManager>,
    port_forward_manager: Arc<PortForwardManager>,
) {
    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let config_manager = config_manager.clone();
                let private_ca = private_ca.clone();
                let relay_manager = relay_manager.clone();
                let ddns_manager = ddns_manager.clone();
                let addon_host = addon_host.clone();
                let voice_supervisor = voice_supervisor.clone();
                let screen_supervisor = screen_supervisor.clone();
                let static_http_manager = static_http_manager.clone();
                let port_forward_manager = port_forward_manager.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_connection(
                        stream,
                        peer_addr,
                        config_manager,
                        private_ca,
                        relay_manager,
                        ddns_manager,
                        addon_host,
                        voice_supervisor,
                        screen_supervisor,
                                    static_http_manager,
                        port_forward_manager,
                                )
                    .await
                    {
                        error!("Config server request failed for {}: {}", peer_addr, error);
                    }
                });
            }
            Err(error) => {
                error!("Config server accept error: {}", error);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_connection(
    mut stream: TcpStream,
    peer_addr: SocketAddr,
    config_manager: Arc<ConfigManager>,
    private_ca: Arc<crate::private_ca::PrivateCaManager>,
    relay_manager: Arc<RelayManager>,
    ddns_manager: Arc<DdnsManager>,
    addon_host: Arc<AddonHost>,
    voice_supervisor: Arc<AsrSupervisor>,
    screen_supervisor: Arc<ScreenServerSupervisor>,
    static_http_manager: Arc<StaticHttpManager>,
    port_forward_manager: Arc<PortForwardManager>,
) -> std::io::Result<()> {
    let request = match read_request(&mut stream).await {
        Ok(request) => request,
        Err(error) => {
            let response = HttpResponse::text(400, "Bad Request", &error.to_string());
            write_response(&mut stream, response).await?;
            return Ok(());
        }
    };

    match api::settings_auth_decision(&request) {
        api::SettingsAuthDecision::Allowed => {}
        api::SettingsAuthDecision::Respond(response) => {
            write_response(&mut stream, response).await?;
            stream.shutdown().await?;
            return Ok(());
        }
        api::SettingsAuthDecision::Close => {
            stream.shutdown().await?;
            return Ok(());
        }
    }

    if request.is_websocket_upgrade {
        crate::config::api::live_ws::handle_upgrade(stream, request, voice_supervisor).await;
        return Ok(());
    }

    let response = api::handle_request(
        request,
        config_manager,
        private_ca,
        relay_manager,
        ddns_manager,
        addon_host,
        voice_supervisor,
        screen_supervisor,
        static_http_manager,
        port_forward_manager,
    )
    .await;
    write_response(&mut stream, response).await?;
    stream.shutdown().await?;
    // info!("Config server completed request for {}", peer_addr);
    Ok(())
}

async fn read_request(stream: &mut TcpStream) -> std::io::Result<HttpRequest> {
    let mut buffer = Vec::new();
    let mut temp = [0u8; 4096];
    let header_end;

    loop {
        let bytes_read = stream.read(&mut temp).await?;
        if bytes_read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected end of stream",
            ));
        }

        buffer.extend_from_slice(&temp[..bytes_read]);
        if buffer.len() > MAX_HTTP_HEADER_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "HTTP header exceeds maximum allowed size",
            ));
        }

        if let Some(index) = find_header_end(&buffer) {
            header_end = index;
            break;
        }
    }

    let header_bytes = &buffer[..header_end];
    let header_text = std::str::from_utf8(header_bytes)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid HTTP header"))?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing request line")
    })?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing HTTP method")
    })?;
    let path = request_parts.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing request path")
    })?;

    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }

        let (name, value) = line.split_once(':').ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid HTTP header line")
        })?;
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
    }

    let content_length = headers
        .get("content-length")
        .map(|value| value.parse::<u64>())
        .transpose()
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid content-length")
        })?
        .unwrap_or(0);

    if (header_end as u64 + 4 + content_length) > MAX_REQUEST_SIZE as u64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "request body exceeds maximum allowed size",
        ));
    }

    let initial_body = &buffer[(header_end + 4)..];
    let content_length = usize::try_from(content_length).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "request body is too large")
    })?;
    let mut body = initial_body[..initial_body.len().min(content_length)].to_vec();
    while body.len() < content_length {
        let remaining = content_length - body.len();
        let read_len = remaining.min(temp.len());
        let bytes_read = stream.read(&mut temp[..read_len]).await?;
        if bytes_read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "request body truncated",
            ));
        }

        body.extend_from_slice(&temp[..bytes_read]);
    }

    let is_websocket_upgrade = headers
        .get("upgrade")
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false);

    Ok(HttpRequest {
        method: method.to_string(),
        path: path.to_string(),
        body,
        headers,
        raw_buffer: buffer,
        is_websocket_upgrade,
    })
}

async fn write_response(stream: &mut TcpStream, response: HttpResponse) -> std::io::Result<()> {
    let status_line = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status_code,
        response.reason_phrase,
        response.content_type,
        response.body.len()
    );

    stream.write_all(status_line.as_bytes()).await?;
    for (name, value) in response.headers {
        let header_line = format!("{}: {}\r\n", name, value);
        stream.write_all(header_line.as_bytes()).await?;
    }
    stream.write_all(b"\r\n").await?;
    stream.write_all(&response.body).await?;
    Ok(())
}

fn bind_dual_stack(port: u16) -> std::io::Result<Vec<TcpListener>> {
    let mut listeners = Vec::new();

    match bind_listener(SocketAddr::V6(SocketAddrV6::new(
        Ipv6Addr::LOCALHOST,
        port,
        0,
        0,
    ))) {
        Ok(listener) => listeners.push(listener),
        Err(error) if error.kind() == std::io::ErrorKind::AddrNotAvailable => {}
        Err(error) => return Err(error),
    }

    match bind_listener(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port))) {
        Ok(listener) => listeners.push(listener),
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse && !listeners.is_empty() => {}
        Err(error) if error.kind() == std::io::ErrorKind::AddrNotAvailable => {}
        Err(error) => return Err(error),
    }

    if listeners.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AddrNotAvailable,
            "unable to bind config server listener",
        ));
    }

    Ok(listeners)
}

fn bind_listener(addr: SocketAddr) -> std::io::Result<TcpListener> {
    let domain = match addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };

    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_only_v6(matches!(addr, SocketAddr::V6(_))).ok();
    // See vterm-asr-server: SO_REUSEADDR on Windows permits duplicate binds of
    // the same port by another process; only enable it on Unix (TIME_WAIT).
    #[cfg(not(windows))]
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    socket.listen(LISTEN_BACKLOG)?;
    socket.set_nonblocking(true)?;

    TcpListener::from_std(socket.into())
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

pub(crate) struct HttpRequest {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) body: Vec<u8>,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) raw_buffer: Vec<u8>,
    pub(crate) is_websocket_upgrade: bool,
}

pub(crate) struct HttpResponse {
    status_code: u16,
    reason_phrase: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
}

impl HttpResponse {
    #[cfg(test)]
    pub(crate) fn status_code(&self) -> u16 {
        self.status_code
    }

    #[cfg(test)]
    pub(crate) fn content_type(&self) -> &'static str {
        self.content_type
    }

    #[cfg(test)]
    pub(crate) fn body(&self) -> &[u8] {
        &self.body
    }

    #[cfg(test)]
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub(crate) fn json<T: serde::Serialize>(body: &T) -> Result<Self, serde_json::Error> {
        Ok(Self::json_bytes(serde_json::to_vec_pretty(body)?))
    }

    pub(crate) fn json_bytes(body: Vec<u8>) -> Self {
        Self {
            status_code: 200,
            reason_phrase: "OK",
            content_type: "application/json; charset=utf-8",
            body,
            headers: vec![("Cache-Control".to_string(), "no-store".to_string())],
        }
    }

    pub(crate) fn json_with_status(status_code: u16, body: Vec<u8>) -> Self {
        Self {
            status_code,
            reason_phrase: reason_phrase(status_code),
            content_type: "application/json; charset=utf-8",
            body,
            headers: vec![("Cache-Control".to_string(), "no-store".to_string())],
        }
    }

    pub(crate) fn bytes_with_headers(
        content_type: &'static str,
        body: &[u8],
        headers: Vec<(String, String)>,
    ) -> Self {
        Self {
            status_code: 200,
            reason_phrase: "OK",
            content_type,
            body: body.to_vec(),
            headers,
        }
    }

    pub(crate) fn redirect(location: String, session_cookie: String) -> Self {
        Self {
            status_code: 303,
            reason_phrase: "See Other",
            content_type: "text/plain; charset=utf-8",
            body: Vec::new(),
            headers: vec![
                ("Location".to_string(), location),
                ("Set-Cookie".to_string(), session_cookie),
                ("Cache-Control".to_string(), "no-store".to_string()),
                ("Referrer-Policy".to_string(), "no-referrer".to_string()),
            ],
        }
    }

    pub(crate) fn text(status_code: u16, reason_phrase: &'static str, body: &str) -> Self {
        Self {
            status_code,
            reason_phrase,
            content_type: "text/plain; charset=utf-8",
            body: body.as_bytes().to_vec(),
            headers: Vec::new(),
        }
    }

    pub(crate) fn empty(status_code: u16) -> Self {
        Self {
            status_code,
            reason_phrase: reason_phrase(status_code),
            content_type: "text/plain; charset=utf-8",
            body: Vec::new(),
            headers: vec![
                ("Allow".to_string(), "GET, PATCH, PUT, OPTIONS".to_string()),
                (
                    "Access-Control-Allow-Methods".to_string(),
                    "GET, POST, PATCH, PUT, DELETE, OPTIONS".to_string(),
                ),
                ("Cache-Control".to_string(), "no-store".to_string()),
            ],
        }
    }
}

fn reason_phrase(status_code: u16) -> &'static str {
    match status_code {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    }
}
