use std::sync::Arc;

use serde::Deserialize;

use crate::addon::AddonHost;
use crate::ddns_client::DdnsManager;
use crate::port_forward::{PortForwardError, PortForwardManager};
use crate::relay::RelayManager;
use crate::screen::ScreenServerSupervisor;
use crate::static_http::StaticHttpManager;
use crate::voice::AsrSupervisor;

use super::server::{HttpRequest, HttpResponse};
use super::store::ConfigManager;
use super::types::{
    ConfigError, ConfigPatchRequest, EnvironmentGroup, EnvironmentNode, EnvironmentVariable,
};
use super::wwwroot;

mod addon;
mod certificates;
pub(crate) mod ddns;
mod device;
mod environment;
pub(crate) mod live_ws;
mod llm;
mod mcp;
mod mcp_templates;
mod notification;
mod port_forwarding;
mod remote_desktop;
mod screen;
pub(crate) mod server;
mod static_http;
mod terminal;
mod uart;
mod voice;
pub(crate) mod voice_models;

pub(crate) enum SettingsAuthDecision {
    Allowed,
    Respond(HttpResponse),
    Close,
}

pub(crate) fn settings_auth_decision(request: &HttpRequest) -> SettingsAuthDecision {
    let path = normalized_path(&request.path);
    if path == "/api/health" {
        return SettingsAuthDecision::Allowed;
    }

    let access = crate::http_auth::settings_access();
    if request.method == "GET" && matches!(path, "/" | "/index.html") {
        if let Some(ticket) = crate::http_auth::query_param(&request.path, "ticket") {
            if let Some(session) = access.exchange_grant(&ticket) {
                let secure = !request
                    .headers
                    .get("host")
                    .is_some_and(|host| is_loopback_host(host));
                return SettingsAuthDecision::Respond(HttpResponse::redirect(
                    crate::http_auth::without_query_param(&request.path, "ticket"),
                    access.session_cookie(&session, secure),
                ));
            }
        }
    }

    let cookie = request.headers.get("cookie").map(String::as_str);
    if !access.has_valid_cookie(cookie) {
        return SettingsAuthDecision::Close;
    }

    if matches!(request.method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE")
        && !is_same_origin_request(request)
    {
        return SettingsAuthDecision::Respond(HttpResponse::text(
            403,
            "Forbidden",
            "Cross-site request rejected",
        ));
    }
    SettingsAuthDecision::Allowed
}

fn is_loopback_host(host: &str) -> bool {
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return ip.is_loopback();
    }
    let hostname = if let Some(bracketed) = host.strip_prefix('[') {
        bracketed
            .split_once(']')
            .map_or(bracketed, |(hostname, _)| hostname)
    } else {
        host.split_once(':').map_or(host, |(hostname, _)| hostname)
    };
    hostname.eq_ignore_ascii_case("localhost")
        || hostname
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

fn is_same_origin_request(request: &HttpRequest) -> bool {
    if request
        .headers
        .get("sec-fetch-site")
        .is_some_and(|value| value.eq_ignore_ascii_case("cross-site"))
    {
        return false;
    }
    let Some(origin) = request.headers.get("origin") else {
        return true;
    };
    let Some(host) = request.headers.get("host") else {
        return false;
    };
    origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
        .unwrap_or(origin)
        .trim_end_matches('/')
        .eq_ignore_ascii_case(host)
}

pub(crate) async fn handle_request(
    request: HttpRequest,
    config_manager: Arc<ConfigManager>,
    private_ca: Arc<crate::private_ca::PrivateCaManager>,
    relay_manager: Arc<RelayManager>,
    ddns_manager: Arc<DdnsManager>,
    addon_host: Arc<AddonHost>,
    voice_supervisor: Arc<AsrSupervisor>,
    screen_supervisor: Arc<ScreenServerSupervisor>,
    static_http_manager: Arc<StaticHttpManager>,
    port_forward_manager: Arc<PortForwardManager>,
) -> HttpResponse {
    match route_request(
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
    .await
    {
        Ok(response) => response,
        Err(error) => error_response(error),
    }
}

async fn route_request(
    request: HttpRequest,
    config_manager: Arc<ConfigManager>,
    private_ca: Arc<crate::private_ca::PrivateCaManager>,
    relay_manager: Arc<RelayManager>,
    ddns_manager: Arc<DdnsManager>,
    addon_host: Arc<AddonHost>,
    voice_supervisor: Arc<AsrSupervisor>,
    screen_supervisor: Arc<ScreenServerSupervisor>,
    static_http_manager: Arc<StaticHttpManager>,
    port_forward_manager: Arc<PortForwardManager>,
) -> Result<HttpResponse, ConfigError> {
    let path = normalized_path(&request.path);

    match (request.method.as_str(), path) {
        ("GET", "/api/static-http/services") => static_http::list_services().await,
        ("POST", "/api/static-http/stop") => {
            static_http::stop_service(&static_http_manager, &request).await
        }
        ("GET", "/api/health") => Ok(HttpResponse::json_bytes(br#"{"ok":true}"#.to_vec())),
        ("POST", "/api/system/open-full-disk-access") => {
            // A JSON-only endpoint cannot be triggered by a foreign web page.
            // endpoint cannot be triggered by a foreign web page.
            if !request
                .headers
                .get("content-type")
                .is_some_and(|value| value.starts_with("application/json"))
            {
                return Err(ConfigError::BadRequest(
                    "open-full-disk-access requires application/json".to_string(),
                ));
            }
            #[cfg(target_os = "macos")]
            {
                open_full_disk_access_settings()
                    .map(|()| HttpResponse::json_bytes(br#"{"ok":true}"#.to_vec()))
                    .map_err(|message| {
                        ConfigError::Io(std::io::Error::new(std::io::ErrorKind::Other, message))
                    })
            }
            #[cfg(not(target_os = "macos"))]
            {
                Ok(HttpResponse::text(
                    400,
                    "Bad Request",
                    "full disk access settings are only available on macOS",
                ))
            }
        }
        ("GET", "/api/config") => {
            let document = config_manager.document().await;
            Ok(HttpResponse::json(&document)?)
        }
        ("PATCH", "/api/config") | ("PUT", "/api/config") => {
            let patch = parse_patch_request(&request.body)?;
            let update = config_manager.apply_patch(patch).await?;
            broadcast_terminal_profiles(&relay_manager).await;
            broadcast_terminal_environments(&relay_manager).await;
            Ok(HttpResponse::json(&update)?)
        }
        ("GET", "/api/certificates") => {
            let config = config_manager.snapshot().await;
            certificates::get_status(&private_ca, &config)
        }
        ("PUT", "/api/certificates/sans") => {
            certificates::update_extra_sans(
                Arc::clone(&private_ca),
                Arc::clone(&config_manager),
                &request.body,
            )
            .await
        }
        ("POST", "/api/certificates/reissue") => {
            certificates::reissue(Arc::clone(&private_ca), Arc::clone(&config_manager)).await
        }
        ("GET", "/api/certificates/root-ca") => certificates::download_root_ca(&private_ca),
        ("GET", "/api/server") => {
            let config = config_manager.snapshot().await;
            Ok(HttpResponse::json(&server::server_config_to_web(&config))?)
        }
        ("PUT", "/api/server") => {
            let payload = parse_json_request::<server::WebServerConfig>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            let old_autostart = config.server.autostart;
            server::apply_web_server_config(&mut config, payload);
            if config.server.autostart != old_autostart {
                // Apply the OS-side change first; only persist when it
                // succeeds so the stored toggle matches reality.
                let enabled = config.server.autostart;
                tokio::task::spawn_blocking(move || crate::autostart::set_enabled(enabled))
                    .await
                    .map_err(|e| {
                        ConfigError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                    })?
                    .map_err(|message| {
                        ConfigError::Io(std::io::Error::new(std::io::ErrorKind::Other, message))
                    })?;
            }
            let update = config_manager.replace(config).await?;
            Ok(HttpResponse::json(&server::server_config_to_web(
                &update.config,
            ))?)
        }
        ("GET", "/api/device") => {
            let config = config_manager.snapshot().await;
            Ok(HttpResponse::json(&device::device_config_to_web(&config))?)
        }
        ("PUT", "/api/device") => {
            let payload = parse_json_request::<device::WebDeviceConfig>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            device::apply_web_device_config(&mut config, payload);
            let update = config_manager.replace(config).await?;
            Ok(HttpResponse::json(&device::device_config_to_web(
                &update.config,
            ))?)
        }
        ("GET", "/api/device/serial-ports") => {
            Ok(HttpResponse::json(&uart::detect_serial_ports())?)
        }
        ("GET", "/api/uart") => {
            let config = config_manager.snapshot().await;
            Ok(HttpResponse::json(&uart::uart_config_to_web(&config))?)
        }
        ("PUT", "/api/uart") => {
            let payload = parse_json_request::<uart::WebUartConfig>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            uart::apply_web_uart_config(&mut config, payload);
            let update = config_manager.replace(config).await?;
            broadcast_terminal_profiles(&relay_manager).await;
            Ok(HttpResponse::json(&uart::uart_config_to_web(
                &update.config,
            ))?)
        }
        ("GET", "/api/uart/serial-ports") => Ok(HttpResponse::json(&uart::detect_serial_ports())?),
        ("GET", "/api/port-forwarding") => {
            let config = config_manager.snapshot().await;
            let forwards = port_forward_manager.list_forwards().await;
            Ok(HttpResponse::json(
                &port_forwarding::port_forwarding_config_to_web(&config, &forwards),
            )?)
        }
        ("PUT", "/api/port-forwarding") => {
            let payload =
                parse_json_request::<port_forwarding::WebPortForwardingConfig>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            port_forwarding::apply_web_port_forwarding_config(&mut config, payload);
            let update = config_manager.replace(config).await?;
            port_forward_manager.update_settings(
                update.config.server.proxy_enabled,
                update.config.server.proxy_idle_timeout_minutes,
            );
            let forwards = port_forward_manager.list_forwards().await;
            Ok(HttpResponse::json(
                &port_forwarding::port_forwarding_config_to_web(&update.config, &forwards),
            )?)
        }
        ("POST", "/api/port-forwarding") => {
            let payload =
                parse_json_request::<port_forwarding::WebCreatePortForwardRequest>(&request.body)?;
            port_forwarding::validate_create_request(&payload)?;
            match port_forward_manager
                .ensure_forward(payload.target_port)
                .await
            {
                Ok(lease) => {
                    let forwards = port_forward_manager.list_forwards().await;
                    let info = forwards
                        .iter()
                        .find(|forward| forward.proxy_port == lease.proxy_port)
                        .ok_or_else(|| {
                            ConfigError::Validation(
                                "created port forward was not found".to_string(),
                            )
                        })?;
                    Ok(HttpResponse::json(&port_forwarding::web_port_forward(
                        info,
                    ))?)
                }
                Err(error) => {
                    let (status, message) = match &error {
                        PortForwardError::Disabled => (403, error.to_string()),
                        PortForwardError::InvalidTargetPort => {
                            (400, "Target port must be non-zero".to_string())
                        }
                        PortForwardError::TargetUnavailable(port) => {
                            (400, format!("No local service is reachable on port {port}"))
                        }
                        PortForwardError::PortRangeExhausted { start, end } => (
                            400,
                            format!("No free port available in range {start}-{end}"),
                        ),
                        other => (500, other.to_string()),
                    };
                    let body = serde_json::to_vec(&serde_json::json!({ "error": message }))?;
                    Ok(HttpResponse::json_with_status(status, body))
                }
            }
        }
        ("POST", _) if path.starts_with("/api/port-forwarding/") && path.ends_with("/share") => {
            let raw = path
                .trim_end_matches("/share")
                .strip_prefix("/api/port-forwarding/")
                .filter(|raw| !raw.is_empty())
                .ok_or_else(|| ConfigError::BadRequest("invalid port forward path".to_string()))?;
            let proxy_port = raw
                .parse::<u16>()
                .map_err(|_| ConfigError::BadRequest("invalid forward port".to_string()))?;
            let forwards = port_forward_manager.list_forwards().await;
            let Some(info) = forwards.iter().find(|forward| forward.proxy_port == proxy_port)
            else {
                return Ok(HttpResponse::text(
                    404,
                    "Not Found",
                    "No active forward on that port",
                ));
            };
            let config = config_manager.snapshot().await;
            let share_token = static_http_manager
                .share_token_for_port(info.target_port)
                .await;
            Ok(HttpResponse::json(&port_forwarding::web_port_forward_share(
                info,
                &config,
                share_token,
            ))?)
        }
        ("POST", _) if path.starts_with("/api/port-forwarding/") && path.ends_with("/stop") => {
            let raw = path
                .trim_end_matches("/stop")
                .strip_prefix("/api/port-forwarding/")
                .filter(|raw| !raw.is_empty())
                .ok_or_else(|| ConfigError::BadRequest("invalid port forward path".to_string()))?;
            let proxy_port = raw
                .parse::<u16>()
                .map_err(|_| ConfigError::BadRequest("invalid forward port".to_string()))?;
            match port_forward_manager.stop_forward(proxy_port).await {
                Ok(info) => Ok(HttpResponse::json(&port_forwarding::web_port_forward(
                    &info,
                ))?),
                Err(crate::port_forward::PortForwardError::PortNotFound(_)) => Ok(
                    HttpResponse::text(404, "Not Found", "No active forward on that port"),
                ),
                Err(error) => Err(ConfigError::Validation(error.to_string())),
            }
        }
        // Addon routes
        ("GET", "/api/addons") => {
            let config = config_manager.snapshot().await;
            addon::list_addons(&addon_host, &config)
        }
        ("GET", _) if path.starts_with("/api/addons/") && path.matches('/').count() == 3 => {
            let id = addon::addon_id_from_path(path, "/api/addons/")?;
            let config = config_manager.snapshot().await;
            addon::get_addon_detail(&addon_host, &config, id)
        }
        ("GET", _) if path.starts_with("/api/addons/") && path.ends_with("/source") => {
            let id = path
                .trim_end_matches("/source")
                .strip_prefix("/api/addons/")
                .ok_or_else(|| ConfigError::BadRequest("invalid addon path".to_string()))?;
            addon::get_addon_source(&addon_host, id)
        }
        // Download an addon as a zip (manifest + source), ready to re-upload.
        ("GET", _) if path.starts_with("/api/addons/") && path.ends_with("/export") => {
            let id = path
                .trim_end_matches("/export")
                .strip_prefix("/api/addons/")
                .ok_or_else(|| ConfigError::BadRequest("invalid addon path".to_string()))?;
            addon::export_addon_zip(&addon_host, id)
        }
        ("PUT", _) if path.starts_with("/api/addons/") && path.ends_with("/config") => {
            let id = path
                .trim_end_matches("/config")
                .strip_prefix("/api/addons/")
                .ok_or_else(|| ConfigError::BadRequest("invalid addon config path".to_string()))?;
            let payload = parse_json_request::<addon::WebAddonConfigUpdate>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            addon::apply_addon_config(&mut config, &addon_host, id, payload)?;
            let update = config_manager.replace(config).await?;
            addon::get_addon_detail(&addon_host, &update.config, id)
        }
        // Save a user-authored source override (the "snapshot" that survives
        // restart). PUT /api/addons/:id/source { source: "..." }.
        ("PUT", _) if path.starts_with("/api/addons/") && path.ends_with("/source") => {
            let id = path
                .trim_end_matches("/source")
                .strip_prefix("/api/addons/")
                .ok_or_else(|| ConfigError::BadRequest("invalid addon source path".to_string()))?;
            let payload = parse_json_request::<addon::WebAddonSourceUpdate>(&request.body)?;
            let config = config_manager.snapshot().await;
            addon::save_addon_source(&addon_host, &config, id, payload)
        }
        // Reset a builtin's source override back to the pristine builtin.
        ("DELETE", _) if path.starts_with("/api/addons/") && path.ends_with("/source") => {
            let id = path
                .trim_end_matches("/source")
                .strip_prefix("/api/addons/")
                .ok_or_else(|| ConfigError::BadRequest("invalid addon source path".to_string()))?;
            let config = config_manager.snapshot().await;
            addon::reset_addon_source(&addon_host, &config, id)
        }
        // Install an addon from an uploaded zip (raw body).
        ("POST", "/api/addons/upload") => {
            let config = config_manager.snapshot().await;
            addon::upload_addon_zip(&addon_host, &config, &request.body)
        }
        // Delete an uploaded addon. Must run after the `/source` DELETE above
        // so resetting an override isn't mistaken for a full delete.
        ("DELETE", _) if path.starts_with("/api/addons/") => {
            let id = addon::addon_id_from_path(path, "/api/addons/")?;
            addon::delete_addon(&addon_host, id)
        }
        // DDNS routes
        ("GET", "/api/ddns/interfaces") => {
            Ok(HttpResponse::json(&ddns::ddns_network_interfaces())?)
        }
        ("GET", "/api/ddns") => {
            let config = config_manager.snapshot().await;
            Ok(HttpResponse::json(&ddns::ddns_config_to_web(
                &config,
                &addon_host,
                None,
            ))?)
        }
        ("PUT", "/api/ddns") => {
            let payload = parse_json_request::<ddns::WebDdnsConfig>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            ddns::apply_web_ddns_config(&mut config, &addon_host, payload)?;
            let update = config_manager.replace(config).await?;
            Ok(HttpResponse::json(&ddns::ddns_config_to_web(
                &update.config,
                &addon_host,
                None,
            ))?)
        }
        ("POST", "/api/ddns/sync") => {
            let sync = match ddns_manager.refresh_once().await {
                Ok(status) => ddns::WebDdnsSyncResult {
                    success: status.last_error.is_none(),
                    last_ip: status.ip.clone(),
                    error: status.last_error,
                },
                Err(error) => ddns::WebDdnsSyncResult {
                    success: false,
                    last_ip: None,
                    error: Some(error.to_string()),
                },
            };
            Ok(HttpResponse::json(&sync)?)
        }
        ("GET", "/api/notifications") => {
            let config = config_manager.snapshot().await;
            Ok(HttpResponse::json(
                &notification::notification_config_to_web(&config.notification),
            )?)
        }
        ("GET", "/api/terminal") => {
            let config = config_manager.snapshot().await;
            Ok(HttpResponse::json(&terminal::terminal_config_to_web(
                &config,
            ))?)
        }
        ("PUT", "/api/notifications") => {
            let payload = parse_json_request::<notification::WebNotificationConfig>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            notification::apply_web_notification_config(&mut config, payload);
            let update = config_manager.replace(config).await?;
            Ok(HttpResponse::json(
                &notification::notification_config_to_web(&update.config.notification),
            )?)
        }
        ("GET", "/api/mcp") => {
            let config = config_manager.snapshot().await;
            Ok(HttpResponse::json(&mcp::mcp_config_to_web(&config.mcp))?)
        }
        // The MCP endpoints the host currently serves (builtin services plus
        // one per live type:"mcp" addon), for the webui's endpoint list.
        ("GET", "/api/mcp/endpoints") => {
            Ok(HttpResponse::json(&mcp::list_mcp_endpoints(&addon_host))?)
        }
        ("PUT", "/api/mcp") => {
            let payload = parse_json_request::<mcp::WebMcpConfig>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            mcp::apply_web_mcp_config(&mut config, payload)?;
            let update = config_manager.replace(config).await?;
            // The MCP settings drive the managed bearer-token environment
            // leaf; push the refreshed environment list to open terminals.
            broadcast_terminal_environments(&relay_manager).await;
            Ok(HttpResponse::json(&mcp::mcp_config_to_web(
                &update.config.mcp,
            ))?)
        }
        // Per-app MCP setup templates.
        ("GET", "/api/mcp/templates") => {
            let config = config_manager.snapshot().await;
            mcp_templates::list_templates(&config)
        }
        ("PUT", _) if path.starts_with("/api/mcp/templates/") => {
            let id = path.trim_start_matches("/api/mcp/templates/");
            let payload = parse_json_request::<mcp_templates::WebMcpTemplateUpdate>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            mcp_templates::apply_template_update(&mut config, id, payload)?;
            let update = config_manager.replace(config).await?;
            mcp_templates::template_response(&update.config, id)
        }
        // Delete a user app or reset a builtin override (the builtin shows
        // through again).
        ("DELETE", _) if path.starts_with("/api/mcp/templates/") => {
            let id = path.trim_start_matches("/api/mcp/templates/");
            let mut config = config_manager.snapshot().await;
            mcp_templates::delete_template(&mut config, id)?;
            config_manager.replace(config).await?;
            Ok(HttpResponse::empty(200))
        }
        ("GET", "/api/remote-desktop") => {
            let config = config_manager.snapshot().await;
            let mut web = remote_desktop::remote_desktop_config_to_web(&config.remote_desktop);
            web.permission_granted =
                remote_desktop::probe_permission_granted(&screen_supervisor).await;
            Ok(HttpResponse::json(&web)?)
        }
        ("PUT", "/api/remote-desktop") => {
            let payload =
                parse_json_request::<remote_desktop::WebRemoteDesktopConfig>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            remote_desktop::apply_web_remote_desktop_config(&mut config, payload);
            let update = config_manager.replace(config).await?;
            let mut web =
                remote_desktop::remote_desktop_config_to_web(&update.config.remote_desktop);
            web.permission_granted =
                remote_desktop::probe_permission_granted(&screen_supervisor).await;
            Ok(HttpResponse::json(&web)?)
        }
        ("POST", "/api/remote-desktop/request-permission") => {
            let result =
                remote_desktop::request_remote_desktop_permission(&screen_supervisor).await;
            Ok(HttpResponse::json(&result)?)
        }
        // Screen-streaming tool-path overrides (coordination spec §4.3).
        ("GET", "/api/screen/paths") => {
            screen::get_screen_paths(&config_manager, &screen_supervisor).await
        }
        ("PUT", "/api/screen/paths") => {
            screen::update_screen_paths(&config_manager, &screen_supervisor, &request.body).await
        }
        // Screen-streaming encoder / frame-rate settings.
        ("GET", "/api/screen/service") => screen::get_screen_service(&screen_supervisor).await,
        // Input-injection status: the Linux uinput probe behind the webui's
        // input-setup card.
        ("GET", "/api/screen/input") => screen::get_screen_input_status(&screen_supervisor).await,
        // Forget the persisted portal ScreenCast consent; the next stream
        // start pops the source picker again (re-select display).
        ("POST", "/api/screen/consent/reset") => {
            screen::reset_screen_consent(&screen_supervisor).await
        }
        ("POST", "/api/screen/service/install") => {
            screen::install_screen_service(&screen_supervisor).await
        }
        ("POST", "/api/screen/service/remove") => {
            screen::remove_screen_service(&screen_supervisor).await
        }
        ("GET", "/api/screen/encoder") => {
            screen::get_screen_encoder(&config_manager, &screen_supervisor).await
        }
        ("PUT", "/api/screen/encoder") => {
            screen::update_screen_encoder(&config_manager, &screen_supervisor, &request.body).await
        }
        ("PUT", "/api/terminal") => {
            let payload = parse_json_request::<terminal::WebTerminalConfig>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            terminal::apply_web_terminal_config(&mut config, payload);
            let update = config_manager.replace(config).await?;
            Ok(HttpResponse::json(&terminal::terminal_config_to_web(
                &update.config,
            ))?)
        }
        ("GET", "/api/llm") => {
            let config = config_manager.snapshot().await;
            Ok(HttpResponse::json(&llm::llm_config_to_web(
                &config,
                &addon_host,
            ))?)
        }
        ("GET", "/api/llm/stats") => {
            let query = parse_query_params(&request.path);
            let from_ms = query.get("fromMs").and_then(|v| v.parse::<i64>().ok());
            let to_ms = query.get("toMs").and_then(|v| v.parse::<i64>().ok());
            let category = query
                .get("category")
                .filter(|v| !v.is_empty())
                .map(String::as_str);
            let addon_filter = query
                .get("addon")
                .filter(|v| !v.is_empty())
                .map(String::as_str);
            match crate::addon::query_llm_stats(from_ms, to_ms, category, addon_filter) {
                Ok(stats) => Ok(HttpResponse::json(&stats)?),
                Err(error) => {
                    tracing::warn!("llm/stats query failed: {error}");
                    Ok(HttpResponse::json_with_status(
                        500,
                        serde_json::to_vec(&serde_json::json!({"error": error}))
                            .unwrap_or_default(),
                    ))
                }
            }
        }
        ("PUT", "/api/llm") => {
            let payload = parse_json_request::<llm::WebLlmConfig>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            llm::apply_web_llm_config(&mut config, &addon_host, payload)?;
            let update = config_manager.replace(config).await?;
            Ok(HttpResponse::json(&llm::llm_config_to_web(
                &update.config,
                &addon_host,
            ))?)
        }
        // Voice / ASR routes
        ("GET", "/api/voice") => voice::get_voice_config(&config_manager, &voice_supervisor).await,
        ("PUT", "/api/voice") => {
            voice::update_voice_config(&config_manager, &voice_supervisor, &request.body).await
        }
        ("GET", "/api/voice/model") => voice::get_voice_model_status(&voice_supervisor).await,
        ("POST", "/api/voice/model/load") => {
            voice::load_voice_model(&config_manager, &voice_supervisor).await
        }
        ("POST", "/api/voice/model/unload") => voice::unload_voice_model(&voice_supervisor).await,
        ("POST", "/api/voice/transcribe") => {
            voice::transcribe(&config_manager, &voice_supervisor, &request).await
        }
        ("POST", "/api/voice/transcribe/stream") => {
            voice::transcribe_stream(&config_manager, &voice_supervisor, &request).await
        }
        ("POST", "/api/voice/polish") => {
            voice::polish(&config_manager, &addon_host, &request.body).await
        }
        ("POST", "/api/voice/server/start") => {
            voice::start_voice_server(&config_manager, &voice_supervisor).await
        }
        ("POST", "/api/voice/server/stop") => voice::stop_voice_server(&voice_supervisor).await,
        ("GET", "/api/voice/models") => {
            let mut query = parse_query_params(&request.path);
            let directory = match query.remove("directory") {
                Some(directory) if !directory.trim().is_empty() => directory,
                _ => config_manager.snapshot().await.asr.model_path,
            };
            Ok(HttpResponse::json(&voice_models::scan(&directory).await)?)
        }
        ("POST", path) if path.starts_with("/api/voice/models/") && path.ends_with("/download") => {
            let id = path
                .trim_start_matches("/api/voice/models/")
                .trim_end_matches("/download")
                .trim_end_matches('/');
            let payload = parse_json_request::<voice_models::DownloadRequest>(&request.body)?;
            Ok(HttpResponse::json(
                &voice_models::start_download(id, payload)
                    .await
                    .map_err(ConfigError::BadRequest)?,
            )?)
        }
        ("POST", "/api/notifications/apns/test") => Ok(HttpResponse::json(
            &notification::test_notification_result(
                "APNs",
                config_manager.snapshot().await.notification.apns.enabled,
            ),
        )?),
        ("POST", "/api/notifications/fcm/test") => Ok(HttpResponse::json(
            &notification::test_notification_result(
                "FCM",
                config_manager.snapshot().await.notification.fcm.enabled,
            ),
        )?),
        ("POST", "/api/notifications/tpns/test") => Ok(HttpResponse::json(
            &notification::test_notification_result(
                "TPNS",
                config_manager.snapshot().await.notification.tpns.enabled,
            ),
        )?),
        ("GET", "/api/environment") => {
            let config = config_manager.snapshot().await;
            Ok(HttpResponse::json(
                &environment::environment_config_to_web(&config),
            )?)
        }
        ("POST", "/api/environment/nodes") => {
            let payload = parse_json_request::<environment::WebCreateNodeRequest>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            let (parent_id, node) = environment::create_node(&config, payload)?;
            let node_id = node.id().to_string();
            let inserted = config
                .environment
                .root
                .insert_node(parent_id.as_deref(), node, None);
            if !inserted {
                return Err(ConfigError::BadRequest(
                    "environment node could not be inserted under the requested parent".to_string(),
                ));
            }
            let update = config_manager.replace(config).await?;
            broadcast_terminal_environments(&relay_manager).await;
            let created = update
                .config
                .environment
                .root
                .find_node(&node_id)
                .ok_or_else(|| {
                    ConfigError::Validation(
                        "created environment node was not persisted".to_string(),
                    )
                })?;
            Ok(HttpResponse::json(&environment::web_environment_node(
                created,
            ))?)
        }
        ("POST", _)
            if path.starts_with("/api/environment/nodes/") && path.ends_with("/duplicate") =>
        {
            let (node_id, suffix) = environment::split_node_path(path, "/api/environment/nodes/")?;
            if suffix != "/duplicate" {
                return Ok(HttpResponse::text(
                    404,
                    "Not Found",
                    "Unknown environment route",
                ));
            }
            reject_mcp_token_leaf_mutation(node_id)?;
            let mut config = config_manager.snapshot().await;
            let source = config
                .environment
                .root
                .find_node(node_id)
                .ok_or_else(|| ConfigError::BadRequest("environment node not found".to_string()))?
                .clone();
            // Per the design: a Leaf duplicates with its variables; a Group
            // duplicates as an empty group (no descendants).
            let duplicate = match source {
                EnvironmentNode::Leaf {
                    name, variables, ..
                } => EnvironmentNode::Leaf {
                    id: environment::next_node_id(&config),
                    name: format!("{} (Copy)", name),
                    variables,
                },
                EnvironmentNode::Group { name, .. } => EnvironmentNode::Group {
                    id: environment::next_node_id(&config),
                    name: format!("{} (Copy)", name),
                    children: Vec::new(),
                },
            };
            let inserted_id = duplicate.id().to_string();
            // Insert as a sibling right after the source. Find the source's
            // parent (or root) and index.
            let parent_idx_path = locate_sibling_slot(&mut config.environment.root, node_id);
            match parent_idx_path {
                SiblingSlot::Root(idx) => {
                    config.environment.root.children.insert(idx + 1, duplicate);
                }
                SiblingSlot::Nested(parent_id, idx) => {
                    if let EnvironmentNode::Group { children, .. } = config
                        .environment
                        .root
                        .find_node_mut(&parent_id)
                        .expect("parent located above")
                    {
                        children.insert(idx + 1, duplicate);
                    }
                }
            }
            let update = config_manager.replace(config).await?;
            broadcast_terminal_environments(&relay_manager).await;
            let created = update
                .config
                .environment
                .root
                .find_node(&inserted_id)
                .ok_or_else(|| {
                    ConfigError::Validation(
                        "duplicated environment node was not persisted".to_string(),
                    )
                })?;
            Ok(HttpResponse::json(&environment::web_environment_node(
                created,
            ))?)
        }
        ("POST", _) if path.starts_with("/api/environment/nodes/") && path.ends_with("/move") => {
            let (node_id, suffix) = environment::split_node_path(path, "/api/environment/nodes/")?;
            if suffix != "/move" {
                return Ok(HttpResponse::text(
                    404,
                    "Not Found",
                    "Unknown environment route",
                ));
            }
            let payload = parse_json_request::<environment::WebMoveNodeRequest>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            reject_mcp_token_leaf_mutation(node_id)?;
            let (target_parent, index) = environment::validate_move(&config, node_id, &payload)?;
            // Detach the node.
            let node = detach_node(&mut config.environment.root, node_id)
                .ok_or_else(|| ConfigError::BadRequest("environment node not found".to_string()))?;
            // Re-insert under the target parent. Note: after detach the indices
            // may shift, so a caller-supplied index is clamped to the new
            // destination length.
            let inserted =
                config
                    .environment
                    .root
                    .insert_node(target_parent.as_deref(), node, index);
            if !inserted {
                return Err(ConfigError::BadRequest(
                    "environment move target was not found or is not a group".to_string(),
                ));
            }
            config_manager.replace(config).await?;
            broadcast_terminal_environments(&relay_manager).await;
            Ok(HttpResponse::empty(204))
        }
        ("PUT", _)
            if path.starts_with("/api/environment/nodes/") && path.ends_with("/variables") =>
        {
            let (node_id, suffix) = environment::split_node_path(path, "/api/environment/nodes/")?;
            if suffix != "/variables" {
                return Ok(HttpResponse::text(
                    404,
                    "Not Found",
                    "Unknown environment route",
                ));
            }
            let payload = parse_json_request::<environment::WebUpdateLeafRequest>(&request.body)?;
            let mut config = config_manager.snapshot().await;
            reject_mcp_token_leaf_mutation(node_id)?;
            let node = config
                .environment
                .root
                .find_node_mut(node_id)
                .ok_or_else(|| ConfigError::BadRequest("environment node not found".to_string()))?;
            match node {
                EnvironmentNode::Leaf { variables, .. } => {
                    *variables = payload
                        .variables
                        .into_iter()
                        .map(|v| EnvironmentVariable {
                            key: v.key,
                            value: v.value,
                        })
                        .collect();
                }
                EnvironmentNode::Group { .. } => {
                    return Err(ConfigError::BadRequest(
                        "environment variables can only be set on leaf nodes".to_string(),
                    ));
                }
            }
            let update = config_manager.replace(config).await?;
            broadcast_terminal_environments(&relay_manager).await;
            let updated = update
                .config
                .environment
                .root
                .find_node(node_id)
                .ok_or_else(|| {
                    ConfigError::Validation(
                        "updated environment node was not persisted".to_string(),
                    )
                })?;
            Ok(HttpResponse::json(&environment::web_environment_node(
                updated,
            ))?)
        }
        ("PUT", _) if path.starts_with("/api/environment/nodes/") => {
            let (node_id, suffix) = environment::split_node_path(path, "/api/environment/nodes/")?;
            if !suffix.is_empty() {
                return Ok(HttpResponse::text(
                    404,
                    "Not Found",
                    "Unknown environment route",
                ));
            }
            let payload = parse_json_request::<environment::WebRenameNodeRequest>(&request.body)?;
            let name = payload.name.trim();
            if name.is_empty() {
                return Err(ConfigError::BadRequest(
                    "environment node name cannot be empty".to_string(),
                ));
            }
            let mut config = config_manager.snapshot().await;
            reject_mcp_token_leaf_mutation(node_id)?;
            let node = config
                .environment
                .root
                .find_node_mut(node_id)
                .ok_or_else(|| ConfigError::BadRequest("environment node not found".to_string()))?;
            node.set_name(name.to_string());
            let update = config_manager.replace(config).await?;
            broadcast_terminal_environments(&relay_manager).await;
            let updated = update
                .config
                .environment
                .root
                .find_node(node_id)
                .ok_or_else(|| {
                    ConfigError::Validation(
                        "renamed environment node was not persisted".to_string(),
                    )
                })?;
            Ok(HttpResponse::json(&environment::web_environment_node(
                updated,
            ))?)
        }
        ("DELETE", _) if path.starts_with("/api/environment/nodes/") => {
            let (node_id, suffix) = environment::split_node_path(path, "/api/environment/nodes/")?;
            if !suffix.is_empty() {
                return Ok(HttpResponse::text(
                    404,
                    "Not Found",
                    "Unknown environment route",
                ));
            }
            let mut config = config_manager.snapshot().await;
            reject_mcp_token_leaf_mutation(node_id)?;
            if !config.environment.root.remove_node(node_id) {
                return Ok(HttpResponse::text(
                    404,
                    "Not Found",
                    "Environment node not found",
                ));
            }
            config_manager.replace(config).await?;
            broadcast_terminal_environments(&relay_manager).await;
            Ok(HttpResponse::empty(204))
        }
        ("GET", _) => Ok(serve_embedded_asset(path)),
        ("OPTIONS", _) => Ok(HttpResponse::empty(204)),
        _ => Ok(HttpResponse::text(404, "Not Found", "Route not found")),
    }
}

/// Where to insert a duplicated node: either directly under the root (with the
/// source's index) or under a nested parent group identified by id.
enum SiblingSlot {
    Root(usize),
    Nested(String, usize),
}

/// Locate the position of `node_id` among its siblings so a duplicate can be
/// inserted right after it. Returns `Root(idx)` when the node is a top-level
/// child, or `Nested(parent_id, idx)` otherwise.
fn locate_sibling_slot(root: &mut EnvironmentGroup, node_id: &str) -> SiblingSlot {
    if let Some(idx) = root.children.iter().position(|c| c.id() == node_id) {
        return SiblingSlot::Root(idx);
    }
    // Search nested groups. We re-find the index by id within each group's
    // children, returning once a match is found.
    let parent_id_and_idx = find_sibling_index_in_tree(&root.children, node_id);
    match parent_id_and_idx {
        Some((parent_id, idx)) => SiblingSlot::Nested(parent_id, idx),
        None => SiblingSlot::Root(root.children.len()),
    }
}

fn find_sibling_index_in_tree(
    children: &[EnvironmentNode],
    node_id: &str,
) -> Option<(String, usize)> {
    for node in children {
        if let EnvironmentNode::Group {
            id,
            children: group_children,
            ..
        } = node
        {
            if let Some(idx) = group_children.iter().position(|c| c.id() == node_id) {
                return Some((id.clone(), idx));
            }
            if let Some(found) = find_sibling_index_in_tree(group_children, node_id) {
                return Some(found);
            }
        }
    }
    None
}

/// Remove and return the node identified by `node_id` from anywhere in the
/// tree. Returns `None` when the node is not found.
fn detach_node(root: &mut EnvironmentGroup, node_id: &str) -> Option<EnvironmentNode> {
    if let Some(idx) = root.children.iter().position(|c| c.id() == node_id) {
        return Some(root.children.remove(idx));
    }
    for child in &mut root.children {
        if let EnvironmentNode::Group { children, .. } = child {
            if let Some(idx) = children.iter().position(|c| c.id() == node_id) {
                return Some(children.remove(idx));
            }
            if let Some(found) = detach_node_inner(children, node_id) {
                return Some(found);
            }
        }
    }
    None
}

fn detach_node_inner(
    children: &mut Vec<EnvironmentNode>,
    node_id: &str,
) -> Option<EnvironmentNode> {
    if let Some(idx) = children.iter().position(|c| c.id() == node_id) {
        return Some(children.remove(idx));
    }
    for child in children {
        if let EnvironmentNode::Group {
            children: group_children,
            ..
        } = child
        {
            if let Some(found) = detach_node_inner(group_children, node_id) {
                return Some(found);
            }
        }
    }
    None
}

async fn broadcast_terminal_profiles(relay_manager: &Arc<RelayManager>) {
    relay_manager.broadcast_terminal_profile_list().await;
}

async fn broadcast_terminal_environments(relay_manager: &Arc<RelayManager>) {
    relay_manager.broadcast_session_list().await;
}

/// The "Vibe Terminal MCP Bearer Token" environment leaf is generated from
/// the MCP settings; reject direct user mutations (rename/move/duplicate/
/// delete/variable edits) so it cannot drift out of sync with the access token.
fn reject_mcp_token_leaf_mutation(node_id: &str) -> Result<(), ConfigError> {
    if environment::is_mcp_token_leaf(node_id) {
        return Err(ConfigError::BadRequest(
            "the 'Vibe Terminal MCP Bearer Token' environment profile is managed by the MCP \
             settings and cannot be modified"
                .to_string(),
        ));
    }
    Ok(())
}

/// Server-startup hook: reconcile the managed MCP bearer-token environment
/// leaf with the persisted MCP config (created/refreshed while MCP is
/// enabled, removed when disabled), so it exists for upgrades without a
/// settings round-trip. Returns whether the config changed.
pub(crate) fn sync_mcp_token_environment(config: &mut super::types::TerminalServerConfig) -> bool {
    environment::sync_mcp_token_leaf(config)
}

fn parse_patch_request(body: &[u8]) -> Result<ConfigPatchRequest, ConfigError> {
    if body.is_empty() {
        return Err(ConfigError::BadRequest(
            "configuration update body cannot be empty".to_string(),
        ));
    }

    serde_json::from_slice(body)
        .map_err(|error| ConfigError::BadRequest(format!("invalid JSON body: {}", error)))
}

fn parse_json_request<T: for<'de> Deserialize<'de>>(body: &[u8]) -> Result<T, ConfigError> {
    if body.is_empty() {
        return Err(ConfigError::BadRequest(
            "request body cannot be empty".to_string(),
        ));
    }

    serde_json::from_slice(body)
        .map_err(|error| ConfigError::BadRequest(format!("invalid JSON body: {}", error)))
}

/// macOS-only: open System Settings on the Full Disk Access pane. Launchd-
/// started background processes are their own TCC client, so the nested
/// server app needs a grant separate from the main app — this pane is where
/// the user adds it.
#[cfg(target_os = "macos")]
fn open_full_disk_access_settings() -> Result<(), String> {
    let status = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
        .status()
        .map_err(|error| format!("cannot run open: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("open exited with {status}"))
    }
}

fn normalized_path(path: &str) -> &str {
    path.split('?').next().unwrap_or(path)
}

fn parse_query_params(raw_path: &str) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    if let Some(query) = raw_path.split('?').nth(1) {
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(urldecode(key), urldecode(value));
            }
        }
    }
    params
}

fn urldecode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let hi = chars.next().unwrap_or(b'0');
            let lo = chars.next().unwrap_or(b'0');
            let decoded = hex_nibble(hi) << 4 | hex_nibble(lo);
            result.push(decoded as char);
        } else if b == b'+' {
            result.push(' ');
        } else {
            result.push(b as char);
        }
    }
    result
}

fn hex_nibble(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

fn error_response(error: ConfigError) -> HttpResponse {
    let status = match error {
        ConfigError::BadRequest(_) | ConfigError::Validation(_) => 400,
        ConfigError::Io(_) | ConfigError::Serialize(_) => 500,
    };
    let body = serde_json::to_vec(&serde_json::json!({
        "error": error.to_string(),
    }))
    .unwrap_or_else(|_| br#"{"error":"internal server error"}"#.to_vec());
    HttpResponse::json_with_status(status, body)
}

fn serve_embedded_asset(path: &str) -> HttpResponse {
    if let Some(asset) = wwwroot::get(path) {
        return HttpResponse::bytes_with_headers(
            asset.content_type,
            &asset.bytes,
            vec![("Cache-Control".to_string(), asset.cache_control.to_string())],
        );
    }

    HttpResponse::text(404, "Not Found", "Route not found")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        normalized_path, serve_embedded_asset, settings_auth_decision, SettingsAuthDecision,
    };
    use crate::config::server::HttpRequest;
    use crate::http_auth::{issue_settings_ticket, settings_access, SETTINGS_SESSION_COOKIE};

    fn request(method: &str, path: &str, cookie: Option<&str>) -> HttpRequest {
        let mut headers = HashMap::new();
        headers.insert("host".to_string(), "localhost:8780".to_string());
        if let Some(cookie) = cookie {
            headers.insert("cookie".to_string(), cookie.to_string());
        }
        HttpRequest {
            method: method.to_string(),
            path: path.to_string(),
            body: Vec::new(),
            headers,
            raw_buffer: Vec::new(),
            is_websocket_upgrade: false,
        }
    }

    #[test]
    fn settings_ticket_exchanges_once_and_cookie_authorizes_later_requests() {
        assert!(matches!(
            settings_auth_decision(&request("GET", "/api/config", None)),
            SettingsAuthDecision::Close
        ));
        let ticket = issue_settings_ticket("config-test");
        let SettingsAuthDecision::Respond(response) = settings_auth_decision(&request(
            "GET",
            &format!("/?ticket={ticket}&theme=dark"),
            None,
        )) else {
            panic!("valid ticket must exchange for a session");
        };
        assert_eq!(response.status_code(), 303);
        assert_eq!(response.header("location"), Some("/?theme=dark"));
        let cookie = response
            .header("set-cookie")
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string();
        assert!(matches!(
            settings_auth_decision(&request("GET", "/api/config", Some(&cookie))),
            SettingsAuthDecision::Allowed
        ));
        assert!(matches!(
            settings_auth_decision(&request("GET", &format!("/?ticket={ticket}"), None)),
            SettingsAuthDecision::Close
        ));
    }

    #[test]
    fn settings_cookie_security_matches_local_or_remote_entry() {
        for host in ["localhost:8780", "127.0.0.1:8780", "[::1]:8780"] {
            let ticket = issue_settings_ticket("local-cookie-test");
            let mut request = request("GET", &format!("/?ticket={ticket}"), None);
            request.headers.insert("host".to_string(), host.to_string());
            let SettingsAuthDecision::Respond(response) = settings_auth_decision(&request) else {
                panic!("valid ticket must exchange for a session");
            };
            let cookie = response.header("set-cookie").unwrap();
            assert!(!cookie.split(';').any(|part| part.trim() == "Secure"));
            assert!(cookie.contains("HttpOnly"));
            assert!(cookie.contains("SameSite=Strict"));
        }

        let ticket = issue_settings_ticket("remote-cookie-test");
        let mut request = request("GET", &format!("/?ticket={ticket}"), None);
        request
            .headers
            .insert("host".to_string(), "server.example:62000".to_string());
        let SettingsAuthDecision::Respond(response) = settings_auth_decision(&request) else {
            panic!("valid ticket must exchange for a session");
        };
        assert!(response
            .header("set-cookie")
            .unwrap()
            .split(';')
            .any(|part| part.trim() == "Secure"));
    }

    #[test]
    fn settings_invalid_websocket_auth_closes_and_health_stays_public() {
        let mut websocket = request("GET", "/api/live", Some("vterm_settings_session=invalid"));
        websocket.is_websocket_upgrade = true;
        assert!(matches!(
            settings_auth_decision(&websocket),
            SettingsAuthDecision::Close
        ));
        assert!(matches!(
            settings_auth_decision(&request("GET", "/api/health", None)),
            SettingsAuthDecision::Allowed
        ));
    }

    #[test]
    fn settings_mutations_reject_cross_site_requests() {
        let ticket = issue_settings_ticket("origin-test");
        let session = settings_access().exchange_grant(&ticket).unwrap();
        let cookie = format!("{SETTINGS_SESSION_COOKIE}={session}");
        let mut request = request("POST", "/api/config", Some(&cookie));
        request
            .headers
            .insert("origin".to_string(), "https://evil.example".to_string());
        let SettingsAuthDecision::Respond(response) = settings_auth_decision(&request) else {
            panic!("cross-site mutation must be rejected");
        };
        assert_eq!(response.status_code(), 403);
    }

    #[test]
    fn normalizes_query_string_for_embedded_assets() {
        assert_eq!(normalized_path("/index.js?v=1"), "/index.js");
    }

    #[test]
    fn serves_embedded_index_html() {
        let response = serve_embedded_asset("/");

        assert_eq!(response.status_code(), 200);
        assert_eq!(response.content_type(), "text/html; charset=utf-8");
        assert!(!response.body().is_empty());
    }
}
