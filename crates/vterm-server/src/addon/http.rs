//! Shared HTTP send path for addon executors (LLM, voice, MCP, DDNS).
//!
//! reqwest reads the `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` environment once,
//! when a `Client` is built, and bakes the result into the connector.
//! vterm-server is a long-running daemon: when the user toggles the system
//! proxy while the server keeps running, a client built at startup keeps
//! using a dead proxy (or keeps bypassing a live one) until restart.
//!
//! Instead, every addon request resolves the CURRENT system proxy state via
//! [`vterm_common::proxy::resolve_system_proxy`] (desktop settings re-read at
//! request time, TTL-cached) and picks a matching client:
//! proxy on → route through it; proxy explicitly off → direct, overriding the
//! stale process env; no desktop settings source (headless) → reqwest's
//! default env behavior. As a safety net, connect-phase failures with a proxy
//! in play are retried once direct — the proxy may have died in the
//! milliseconds between detection and connect.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use tracing::warn;
use vterm_common::proxy::ResolvedProxy;

/// A fully materialized HTTP request. Addon executors capture headers on the
/// JS thread (rquickjs objects cannot cross threads) into this plain data
/// struct, then send it on a worker thread.
#[derive(Debug, Clone, Default)]
pub struct AddonHttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    /// Binary-capable body: LLM/MCP addons send JSON strings, the voice
    /// descriptor path sends multipart WAV bytes.
    pub body: Option<Vec<u8>>,
    pub timeout: Option<Duration>,
}

/// Build a reqwest request from plain data. Reusable for the direct retry:
/// a `RequestBuilder` is tied to the client that created it, so the fallback
/// must rebuild from scratch on the proxy-less client.
pub fn build_request(
    client: &reqwest::Client,
    request: &AddonHttpRequest,
) -> reqwest::RequestBuilder {
    let mut req = match request.method.as_str() {
        "GET" => client.get(&request.url),
        "POST" => client.post(&request.url),
        "PUT" => client.put(&request.url),
        "PATCH" => client.patch(&request.url),
        "DELETE" => client.delete(&request.url),
        _ => client.get(&request.url),
    };
    if let Some(timeout) = request.timeout {
        req = req.timeout(timeout);
    }
    for (key, value) in &request.headers {
        req = req.header(key, value);
    }
    if !request.query.is_empty() {
        req = req.query(&request.query);
    }
    if let Some(body) = &request.body {
        req = req.body(body.clone());
    }
    req
}

/// Loopback never goes through a proxy: addons like vibe-voice default to a
/// local ASR server (`https://127.0.0.1:8398`) and would break if routed.
fn loopback_bypass() -> Option<reqwest::NoProxy> {
    reqwest::NoProxy::from_string("localhost,127.0.0.1,::1")
}

/// True when `url` targets a loopback host.
fn is_loopback_url(url: &str) -> bool {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_string))
        .is_some_and(|host| matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1" | "[::1]"))
}

/// Dedicated client for loopback targets. Loopback never traverses a proxy,
/// whatever the resolved system state is — in `Environment` mode reqwest
/// would honor a stale proxy env var unless the user also set `NO_PROXY`.
fn loopback_client() -> reqwest::Client {
    static CLIENT: Mutex<Option<reqwest::Client>> = Mutex::new(None);
    let mut cached = CLIENT.lock().unwrap();
    if let Some(client) = cached.as_ref() {
        return client.clone();
    }
    let client = client_builder_for(&ResolvedProxy::Direct)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    *cached = Some(client.clone());
    client
}

/// Clients are pooled per proxy state so a stable configuration keeps
/// connection reuse; a proxy toggle simply starts a fresh pool.
static CLIENT_CACHE: Mutex<Option<HashMap<String, reqwest::Client>>> = Mutex::new(None);

/// Client matching the given proxy state. Callers that need extra builder
/// options (e.g. a User-Agent) can use [`client_builder_for`] instead.
pub fn client_for(proxy: &ResolvedProxy) -> reqwest::Client {
    let key = match proxy {
        ResolvedProxy::Direct => String::new(),
        ResolvedProxy::Environment => String::from("\0env"),
        ResolvedProxy::Url(url) => url.clone(),
    };
    let mut cache = CLIENT_CACHE.lock().unwrap();
    let cache = cache.get_or_insert_with(HashMap::new);
    if let Some(client) = cache.get(&key) {
        return client.clone();
    }
    let client = client_builder_for(proxy)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    cache.insert(key, client.clone());
    client
}

/// A client builder preconfigured for the given proxy state.
pub fn client_builder_for(proxy: &ResolvedProxy) -> reqwest::ClientBuilder {
    // Addon endpoints are frequently self-hosted (local/LAN ASR or LLM
    // servers) with self-signed certificates, so every addon client accepts
    // them — matching the pre-refactor voice path, which borrowed the
    // supervisor's `danger_accept_invalid_certs(true)` client.
    let builder = reqwest::Client::builder().danger_accept_invalid_certs(true);
    match proxy {
        ResolvedProxy::Direct => builder.no_proxy(),
        ResolvedProxy::Environment => builder,
        ResolvedProxy::Url(url) => match reqwest::Proxy::all(url) {
            Ok(proxy) => builder.proxy(proxy.no_proxy(loopback_bypass())),
            // Malformed proxy URL from the desktop settings: fall back to
            // reqwest's default env behavior rather than failing the request.
            Err(_) => builder,
        },
    }
}

/// True when any proxy env var reqwest honors is set. Only used to decide
/// whether a direct retry could possibly help in `Environment` mode.
fn env_proxy_configured() -> bool {
    [
        "HTTP_PROXY",
        "http_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "ALL_PROXY",
        "all_proxy",
    ]
    .iter()
    .any(|key| std::env::var_os(key).is_some_and(|value| !value.is_empty()))
}

/// Send `request` with a client matching the current system proxy state
/// (re-resolved per call). On a connect-phase failure while a proxy is in
/// play, retry once with a proxy-less client — connect errors are what a dead
/// proxy produces (connection refused / unreachable). HTTP error statuses and
/// body-read failures are returned as-is.
pub async fn send_addon_request(
    request: &AddonHttpRequest,
) -> Result<reqwest::Response, reqwest::Error> {
    // Loopback targets (local ASR server) never involve a proxy; send
    // directly so a stale proxy env cannot capture them in Environment mode.
    if is_loopback_url(&request.url) {
        return build_request(&loopback_client(), request).send().await;
    }
    let proxy = vterm_common::proxy::resolve_system_proxy();
    let client = client_for(&proxy);
    match build_request(&client, request).send().await {
        Ok(response) => Ok(response),
        Err(error) => {
            if !error.is_connect() {
                return Err(error);
            }
            let proxy_in_play = match &proxy {
                ResolvedProxy::Url(_) => true,
                ResolvedProxy::Environment => env_proxy_configured(),
                ResolvedProxy::Direct => false,
            };
            if !proxy_in_play {
                return Err(error);
            }
            warn!(
                url = %request.url,
                "addon HTTP connect failed with a proxy in play; the system proxy may have \
                 just changed — retrying without proxy"
            );
            let direct = client_for(&ResolvedProxy::Direct);
            build_request(&direct, request).send().await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_applies_method_headers_query_body() {
        let client = reqwest::Client::new();
        let request = AddonHttpRequest {
            method: "POST".to_string(),
            url: "https://api.example.com/v1/chat".to_string(),
            headers: vec![
                ("authorization".to_string(), "Bearer key".to_string()),
                ("content-type".to_string(), "application/json".to_string()),
            ],
            query: vec![("language".to_string(), "zh".to_string())],
            body: Some(b"{}".to_vec()),
            timeout: Some(Duration::from_secs(60)),
        };
        let built = build_request(&client, &request).build().unwrap();
        assert_eq!(built.method(), reqwest::Method::POST);
        assert_eq!(
            built.url().as_str(),
            "https://api.example.com/v1/chat?language=zh"
        );
        assert_eq!(built.headers().get("authorization").unwrap(), "Bearer key");
        assert_eq!(
            built.body().and_then(|b| b.as_bytes()),
            Some(b"{}".as_slice())
        );
    }

    #[test]
    fn build_request_defaults_to_get_for_unknown_methods() {
        let client = reqwest::Client::new();
        let request = AddonHttpRequest {
            method: "WEIRD".to_string(),
            url: "https://example.com".to_string(),
            ..Default::default()
        };
        let built = build_request(&client, &request).build().unwrap();
        assert_eq!(built.method(), reqwest::Method::GET);
    }

    #[test]
    fn loopback_url_detection() {
        assert!(is_loopback_url(
            "https://127.0.0.1:8398/v1/audio/transcriptions"
        ));
        assert!(is_loopback_url("http://localhost:8398/health"));
        assert!(is_loopback_url("https://[::1]:8398/health"));
        assert!(!is_loopback_url("https://api.example.com/v1/chat"));
        assert!(!is_loopback_url("not a url"));
    }

    #[test]
    fn client_for_builds_and_caches_per_proxy_state() {
        // Repeated calls for the same state must succeed and hit the cache;
        // a proxied state must build without error.
        let _ = client_for(&ResolvedProxy::Direct);
        let _ = client_for(&ResolvedProxy::Direct);
        let _ = client_for(&ResolvedProxy::Url("http://127.0.0.1:7890".to_string()));
        let cache = CLIENT_CACHE.lock().unwrap();
        let cache = cache.as_ref().unwrap();
        assert!(cache.contains_key(""));
        assert!(cache.contains_key("http://127.0.0.1:7890"));
    }
}
