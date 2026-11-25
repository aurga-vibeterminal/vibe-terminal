//! In-process MCP host: serves multiple independent MCP services behind one
//! HTTP port. See docs/superpowers/specs/2026-07-05-multi-mcp-architecture-design.md.
//!
//! The host is service-agnostic: it iterates [`registry::all_services`], asks
//! each for its axum route, merges them, applies shared bearer-token auth, and
//! serves the merged router. Adding a service = one line in `all_services()`.
//!
//! Services live under [`services`]; each implements [`registry::McpService`].
//! Today: terminal and dynamically registered addon services.

#![allow(dead_code)]

pub(crate) mod auth;
pub(crate) mod registry;
pub(crate) mod services;
pub(crate) mod shared;
pub(crate) mod templates;

use std::sync::Arc;

use axum::{middleware, Extension, Router};
use tracing::{error, info, warn};

use crate::addon::AddonHost;
use crate::config::{ConfigManager, McpConfig};
use crate::session::SessionManager as VtSessionManager;
use vterm_common::McpCommandTarget;

/// Build the setup line that registers this server's MCP endpoint with an
/// agent CLI running in a terminal on that server. For CLI-style apps
/// (Claude Code, Codex) it is an `mcp add` command; for conversation-style
/// apps (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, and the
/// generic fallback) it is the chat prompt to send. The line is typed into
/// the terminal without a trailing newline.
///
/// The setup line comes from the merged per-app template view (see
/// [`templates`]); user entries in `McpConfig.templates` override builtins or
/// define new apps.
/// Unknown executables fall back to the generic template.
pub fn add_command_for_app(executable: &str, config: &McpConfig) -> String {
    let views = templates::merged_templates(config);
    let view = templates::find_template(&views, executable);
    templates::render_for_view(view, executable, config)
}

/// Classify where an executable's MCP setup line must be typed: a plain
/// terminal (a shell `mcp add` command) or inside the running app (a chat
/// prompt). Resolution follows the same merged-template lookup as
/// [`add_command_for_app`], so unknown executables fall back to the generic
/// in-app template. Used to annotate the known-app list the terminal's MCP
/// context menu filters on.
pub fn mcp_command_target_for_app(executable: &str, config: &McpConfig) -> McpCommandTarget {
    let views = templates::merged_templates(config);
    let view = templates::find_template(&views, executable);
    templates::command_target(view)
}

#[cfg(test)]
mod add_command_tests {
    use super::*;

    fn config_with_token(token: &str) -> McpConfig {
        McpConfig {
            access_token: token.to_string(),
            ..McpConfig::default()
        }
    }

    #[test]
    fn claude_style_with_token() {
        assert_eq!(
            add_command_for_app("claude", &config_with_token("abc")),
            "claude mcp add --transport http vterm-mcp http://localhost:6180/terminal/mcp --header \"Authorization: Bearer abc\""
        );
    }

    #[test]
    fn claude_style_without_token() {
        assert_eq!(
            add_command_for_app("claude", &config_with_token("")),
            "claude mcp add --transport http vterm-mcp http://localhost:6180/terminal/mcp"
        );
    }

    /// Kimi Code registers MCP servers through the agent conversation, so its
    /// setup line is the chat prompt, not a shell command. The prompt passes
    /// the VTERM_MCP_TOKEN environment variable via bearerTokenEnvVar (the
    /// managed "Vibe Terminal MCP Bearer Token" profile) instead of pasting
    /// the token into the chat.
    #[test]
    fn kimi_conversational_with_token() {
        assert_eq!(
            add_command_for_app("kimi", &config_with_token("abc")),
            "/mcp-config add an HTTP MCP, url is http://localhost:6180/terminal/mcp, bearerTokenEnvVar is VTERM_MCP_TOKEN"
        );
    }

    #[test]
    fn kimi_conversational_without_token() {
        assert_eq!(
            add_command_for_app("kimi", &config_with_token("")),
            "/mcp-config add an HTTP MCP, url is http://localhost:6180/terminal/mcp"
        );
    }

    /// Pi, OpenCode, Qwen Code, DeepSeek Harness, and unknown apps register
    /// MCP servers through the agent conversation. Their prompt uses
    /// VTERM_MCP_TOKEN from the managed "Vibe Terminal MCP Bearer Token"
    /// profile instead of exposing it in the chat.
    #[test]
    fn pi_opencode_qwen_deepseek_and_generic_are_conversational() {
        for executable in ["pi", "opencode", "qwen", "dsh", "unknown-cli"] {
            assert_eq!(
                add_command_for_app(executable, &config_with_token("abc")),
                "Add an HTTP MCP server named vterm-mcp with URL http://localhost:6180/terminal/mcp. Authenticate with the Bearer token from the VTERM_MCP_TOKEN environment variable. Update your MCP configuration directly."
            );
            assert_eq!(
                add_command_for_app(executable, &config_with_token("")),
                "Add an HTTP MCP server named vterm-mcp with URL http://localhost:6180/terminal/mcp. Update your MCP configuration directly."
            );
        }
    }

    #[test]
    fn codex_with_token() {
        // The token is delivered through the VTERM_MCP_TOKEN environment
        // variable (the managed "Vibe Terminal MCP Bearer Token" profile),
        // never inlined into the command itself.
        assert_eq!(
            add_command_for_app("codex", &config_with_token("abc")),
            "codex mcp add vterm-mcp --url http://localhost:6180/terminal/mcp --bearer-token-env-var VTERM_MCP_TOKEN"
        );
    }

    #[test]
    fn codex_without_token() {
        assert_eq!(
            add_command_for_app("codex", &config_with_token("")),
            "codex mcp add vterm-mcp --url http://localhost:6180/terminal/mcp"
        );
    }

    #[test]
    fn custom_port_flows_into_url() {
        let mut config = config_with_token("abc");
        config.port = 9999;
        assert_eq!(
            add_command_for_app("claude", &config),
            "claude mcp add --transport http vterm-mcp http://localhost:9999/terminal/mcp --header \"Authorization: Bearer abc\""
        );
    }

    #[test]
    fn user_override_changes_command() {
        let mut config = config_with_token("abc");
        config.templates.insert(
            "claude".to_string(),
            crate::config::McpTemplateConfig {
                display_name: "Claude".to_string(),
                executable: "claude".to_string(),
                template: "{executable} register {url}{#token} tok={token}{/token}".to_string(),
                ..crate::config::McpTemplateConfig::default()
            },
        );
        assert_eq!(
            add_command_for_app("claude", &config),
            "claude register http://localhost:6180/terminal/mcp tok=abc"
        );
    }

    /// Setup lines are typed into a plain terminal by default
    /// (`{executable} mcp add …`); chat-prompt style templates must be typed
    /// inside the running app. A user override decides the target as much as
    /// the builtin does.
    #[test]
    fn command_target_follows_template_style() {
        let config = config_with_token("abc");
        for executable in ["claude", "codex"] {
            assert_eq!(
                mcp_command_target_for_app(executable, &config),
                McpCommandTarget::Terminal,
                "{executable} should be terminal-targeted"
            );
        }
        for executable in ["kimi", "pi", "opencode", "qwen", "dsh", "unknown-cli"] {
            assert_eq!(
                mcp_command_target_for_app(executable, &config),
                McpCommandTarget::InApp,
                "{executable} should be in-app-targeted"
            );
        }

        // A user override reshapes the classification.
        let mut config = config_with_token("abc");
        config.templates.insert(
            "kimi".to_string(),
            crate::config::McpTemplateConfig {
                display_name: "Kimi Code".to_string(),
                executable: "kimi".to_string(),
                template: "{executable} mcp add vterm-mcp --url {url}".to_string(),
                ..crate::config::McpTemplateConfig::default()
            },
        );
        assert_eq!(
            mcp_command_target_for_app("kimi", &config),
            McpCommandTarget::Terminal
        );
    }
}

use auth::{require_bearer, ExpectedToken};
use registry::{all_services, McpContext};

struct AbortTaskOnDrop(Option<tokio::task::JoinHandle<()>>);

impl AbortTaskOnDrop {
    fn new(handle: tokio::task::JoinHandle<()>) -> Self {
        Self(Some(handle))
    }

    async fn stop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.abort();
            let _ = handle.await;
        }
    }
}

impl Drop for AbortTaskOnDrop {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            info!("Stopping MCP HTTP listener task");
            handle.abort();
        }
    }
}

/// The MCP host. Builds a merged axum router from [`all_services`] and serves
/// it on the configured port behind shared bearer-token auth. Hot-restarts on
/// `McpConfig` changes (port/bind_address/access_token).
pub struct McpServer {
    session_manager: Arc<VtSessionManager>,
    config_manager: Arc<ConfigManager>,
    addon_host: Arc<AddonHost>,
    tls_acceptor: crate::ServerTlsAcceptor,
}

impl McpServer {
    pub fn new(
        session_manager: Arc<VtSessionManager>,
        config_manager: Arc<ConfigManager>,
        addon_host: Arc<AddonHost>,
        tls_acceptor: crate::ServerTlsAcceptor,
    ) -> Self {
        Self {
            session_manager,
            config_manager,
            addon_host,
            tls_acceptor,
        }
    }

    /// Run until shutdown. Subscribes to config changes for hot reload.
    pub async fn run(self) -> std::io::Result<()> {
        let mut cfg_rx = self.config_manager.subscribe();
        loop {
            let cfg = cfg_rx.borrow().mcp.clone();
            if !cfg.enabled {
                if cfg_rx.changed().await.is_err() {
                    return Ok(());
                }
                continue;
            }
            let token = cfg.access_token.clone();

            // Build the merged router from the registry. Each service provides
            // its own nest_service route; the host merges them and applies the
            // shared auth layer once.
            let ctx = McpContext {
                session_manager: Arc::clone(&self.session_manager),
                config_manager: Arc::clone(&self.config_manager),
                addon_host: Arc::clone(&self.addon_host),
            };
            let mut app = all_services()
                .into_iter()
                .map(|svc| svc.build_route(&ctx))
                .fold(Router::new(), |acc, r| acc.merge(r));
            // Layer order (axum: LAST `.layer()` is outermost = runs first):
            // `Extension` must be outermost so it inserts the token BEFORE
            // `require_bearer` reads it.
            app = app
                .layer(middleware::from_fn(require_bearer))
                .layer(Extension(ExpectedToken(token.clone())));

            let loopback = is_loopback_bind(&cfg.bind_address);
            let (listener, loopback_v6) = if loopback {
                let bind = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, cfg.port));
                let listener = match tokio::net::TcpListener::bind(bind).await {
                    Ok(l) => l,
                    Err(e) => {
                        error!("MCP bind {bind} failed: {e}");
                        return Err(e);
                    }
                };
                info!("MCP host listening on http://{bind}");
                let bind = std::net::SocketAddr::from((std::net::Ipv6Addr::LOCALHOST, cfg.port));
                let ipv6 = match tokio::net::TcpListener::bind(bind).await {
                    Ok(l) => {
                        info!("MCP host listening on http://{bind}");
                        Some(l)
                    }
                    Err(e) => {
                        warn!("MCP IPv6 loopback bind {bind} failed: {e}; using IPv4 only");
                        None
                    }
                };
                (listener, ipv6)
            } else {
                let bind = cfg
                    .bind_address
                    .parse::<std::net::IpAddr>()
                    .map(|ip| std::net::SocketAddr::new(ip, cfg.port).to_string())
                    .unwrap_or_else(|_| format!("{}:{}", cfg.bind_address, cfg.port));
                let listener = match tokio::net::TcpListener::bind(&bind).await {
                    Ok(l) => l,
                    Err(e) => {
                        error!("MCP bind {bind} failed: {e}");
                        return Err(e);
                    }
                };
                info!("MCP host listening on {bind} (HTTPS; loopback peers may use HTTP)");
                (listener, None)
            };

            let tls_acceptor = self.tls_acceptor.clone();
            let mut server_handle = AbortTaskOnDrop::new(tokio::spawn(async move {
                if loopback {
                    if let Err(e) = serve_http(listener, loopback_v6, app).await {
                        error!("MCP host exited: {e}");
                    }
                } else if let Err(e) = serve_https(listener, app, tls_acceptor).await {
                    error!("MCP HTTPS host exited: {e}");
                }
            }));

            // Watch for a config change requiring restart.
            loop {
                if cfg_rx.changed().await.is_err() {
                    return Ok(());
                }
                let new_cfg = cfg_rx.borrow().mcp.clone();
                let needs_restart = !new_cfg.enabled
                    || new_cfg.port != cfg.port
                    || new_cfg.bind_address != cfg.bind_address
                    || new_cfg.access_token != cfg.access_token;
                if needs_restart {
                    warn!("MCP config changed; restarting host");
                    server_handle.stop().await;
                    break;
                }
            }
        }
    }
}

fn is_loopback_bind(address: &str) -> bool {
    address.eq_ignore_ascii_case("localhost")
        || address
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

pub(crate) fn endpoint_url(config: &McpConfig) -> String {
    format!("http://localhost:{}/terminal/mcp", config.port)
}

async fn serve_http(
    listener: tokio::net::TcpListener,
    ipv6_listener: Option<tokio::net::TcpListener>,
    app: Router,
) -> std::io::Result<()> {
    // ConnectInfo supplies the peer address to `require_bearer` (loopback
    // peers are exempt from the token check).
    if let Some(ipv6_listener) = ipv6_listener {
        tokio::select! {
            result = axum::serve(
                listener,
                app.clone().into_make_service_with_connect_info::<std::net::SocketAddr>(),
            ) => result,
            result = axum::serve(
                ipv6_listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            ) => result,
        }
    } else {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
    }
}

fn use_loopback_http(peer_addr: std::net::SocketAddr, first_byte: u8) -> bool {
    peer_addr.ip().is_loopback() && first_byte != 0x16
}

async fn serve_https(
    listener: tokio::net::TcpListener,
    app: Router,
    acceptor: crate::ServerTlsAcceptor,
) -> std::io::Result<()> {
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use hyper_util::server::conn::auto::Builder;
    use hyper_util::service::TowerToHyperService;

    let builder = Arc::new(Builder::new(TokioExecutor::new()));
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let app = app.clone();
        let builder = builder.clone();
        tokio::spawn(async move {
            let mut first_byte = [0u8; 1];
            let loopback_http = match stream.peek(&mut first_byte).await {
                Ok(1) => use_loopback_http(peer_addr, first_byte[0]),
                Ok(_) => false,
                Err(error) => {
                    tracing::debug!(
                        "MCP connection closed before transport detection for {peer_addr}: {error}"
                    );
                    return;
                }
            };
            let app = app.layer(Extension(axum::extract::ConnectInfo(peer_addr)));
            if loopback_http {
                let service = TowerToHyperService::new(app);
                let connection =
                    builder.serve_connection_with_upgrades(TokioIo::new(stream), service);
                if let Err(error) = connection.await {
                    tracing::debug!("MCP loopback HTTP connection closed for {peer_addr}: {error}");
                }
                return;
            }

            let stream = match acceptor.accept(stream).await {
                Ok(stream) => stream,
                Err(error) => {
                    tracing::debug!("MCP TLS handshake failed for {peer_addr}: {error}");
                    return;
                }
            };
            let service = TowerToHyperService::new(app);
            let connection = builder.serve_connection_with_upgrades(TokioIo::new(stream), service);
            if let Err(error) = connection.await {
                tracing::debug!("MCP HTTPS connection closed for {peer_addr}: {error}");
            }
        });
    }
}

#[cfg(test)]
mod transport_policy_tests {
    use super::{endpoint_url, is_loopback_bind};
    use crate::config::McpConfig;

    #[test]
    fn only_loopback_bindings_use_plain_http() {
        assert!(is_loopback_bind("localhost"));
        assert!(is_loopback_bind("127.0.0.1"));
        assert!(is_loopback_bind("::1"));
        assert!(!is_loopback_bind("0.0.0.0"));
        assert!(!is_loopback_bind("::"));
        assert!(!is_loopback_bind("192.168.1.10"));
    }

    #[test]
    fn local_endpoint_url_always_uses_plain_http() {
        let mut config = McpConfig::default();
        for address in [
            "localhost",
            "127.0.0.1",
            "::1",
            "0.0.0.0",
            "::",
            "192.168.1.10",
        ] {
            config.bind_address = address.to_string();
            assert_eq!(endpoint_url(&config), "http://localhost:6180/terminal/mcp");
        }
    }

    #[test]
    fn plaintext_fallback_is_limited_to_loopback_peers() {
        let ipv4: std::net::SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let ipv6: std::net::SocketAddr = "[::1]:12345".parse().unwrap();
        let remote: std::net::SocketAddr = "192.168.1.20:12345".parse().unwrap();
        assert!(super::use_loopback_http(ipv4, b'G'));
        assert!(super::use_loopback_http(ipv6, b'G'));
        assert!(!super::use_loopback_http(ipv4, 0x16));
        assert!(!super::use_loopback_http(remote, b'G'));
        assert!(!super::use_loopback_http(remote, 0x16));
    }

    #[tokio::test]
    async fn loopback_http_serves_ipv4_and_ipv6_on_the_same_port() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let ipv4 = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let port = ipv4.local_addr().unwrap().port();
        let Ok(ipv6) = tokio::net::TcpListener::bind((std::net::Ipv6Addr::LOCALHOST, port)).await
        else {
            return;
        };
        let app = axum::Router::new().route("/", axum::routing::get(|| async { "ok" }));
        let server = tokio::spawn(super::serve_http(ipv4, Some(ipv6), app));

        for address in [
            std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port)),
            std::net::SocketAddr::from((std::net::Ipv6Addr::LOCALHOST, port)),
        ] {
            let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
            stream
                .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .await
                .unwrap();
            let mut response = Vec::new();
            tokio::time::timeout(
                std::time::Duration::from_secs(2),
                stream.read_to_end(&mut response),
            )
            .await
            .unwrap()
            .unwrap();
            assert!(response.starts_with(b"HTTP/1.1 200 OK"), "{address}");
        }
        server.abort();
    }

    #[tokio::test]
    async fn https_listener_accepts_plain_http_from_loopback() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let certified_key = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(
            certified_key.signing_key.serialize_der().into(),
        );
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certified_key.cert.der().clone()], key)
            .unwrap();
        let acceptor = crate::ServerTlsAcceptor::from(std::sync::Arc::new(config));
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let app = axum::Router::new().route("/", axum::routing::get(|| async { "ok" }));
        let server = tokio::spawn(super::serve_https(listener, app, acceptor));

        let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let mut response = Vec::new();
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            stream.read_to_end(&mut response),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(response.starts_with(b"HTTP/1.1 200 OK"));
        server.abort();
    }
}
