//! Process supervisor for the locally-managed `vterm-voice` binary (the renamed
//! `vterm-asr-server` crate, which produces a `vterm-voice` executable).
//!
//! Responsibilities:
//! - Locate the `vterm-voice` binary as a sibling of the running
//!   `vterm-server` executable (or under `$HOME/.vibeterminal[dev]/bin/`).
//! - When ASR is enabled, verify the authenticated `/v1/config` endpoint and
//!   **attach** to an already-running compatible instance (single-instance
//!   semantics — the asr-server itself holds the lock, so a duplicate would
//!   have exited). If unreachable, spawn the binary as a child with the right
//!   `--host`/`--port`/`--model`/`--device`/`--access-token` flags, then poll
//!   `/health` until ready.
//! - Expose [`AsrSupervisor::endpoint`] and [`AsrSupervisor::bearer_token`]
//!   so the voice API handlers can target the live process instead of the
//!   legacy hardcoded `localhost:8398`.
//! - Hot-restart on `AsrConfig` changes that affect the endpoint
//!   (`bind_address`/`port`/`access_token`). Configuration-only
//!   changes (model path, device) are pushed via `PUT /v1/config` without a
//!   restart.
//!
//! The supervisor owns one `reqwest::Client` (with a generous timeout — model
//! loads are slow) that handlers borrow for proxied requests.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

use crate::config::AsrConfig;
use crate::config::ConfigManager;

/// Snapshot of the asr-server's model lifecycle state. Mirrors the
/// `/v1/model/status` payload shape returned by vterm-asr-server, with the
/// same `snake_case` status tag we expose over the web API.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum VoiceServiceState {
    NotLoaded,
    Loading,
    Loaded { model_id: String, device: String },
    Error { message: String },
}

/// How long to wait for the freshly-spawned asr-server to answer `/health`.
const HEALTH_PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
/// Poll interval during the readiness wait.
const HEALTH_PROBE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);

/// What the supervisor currently knows about the local asr-server process.
#[derive(Debug, Clone)]
struct SupervisedEndpoint {
    /// `https://<host>:<port>` (no trailing slash). Uses the configured port
    /// verbatim — the supervisor always binds a fixed port.
    base_url: String,
    /// The bearer token presented to the asr-server, or `None` if
    /// unauthenticated.
    token: Option<String>,
}

struct SupervisorInner {
    /// The supervised child process, if we spawned one. `None` when attached
    /// to an external instance or when stopped.
    child: Option<Child>,
    endpoint: Option<SupervisedEndpoint>,
}

pub struct AsrSupervisor {
    config_manager: Arc<ConfigManager>,
    http_client: reqwest::Client,
    inner: Arc<Mutex<SupervisorInner>>,
    /// Snapshot of the config used for the currently-running (or attached)
    /// process. Used by the watch loop to detect restart-worthy changes.
    /// Guarded separately from `inner` so callers reading endpoint/token
    /// don't have to take the spawn mutex.
    current: RwLock<Option<AsrConfig>>,
}

impl AsrSupervisor {
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            // The locally supervised ASR server uses an embedded development
            // identity when no custom certificate is configured. This client
            // only targets that local service, so it must accept that
            // self-signed identity.
            .danger_accept_invalid_certs(true)
            // Loopback-only client (see is_loopback_endpoint): never route
            // through the system proxy env, which would send local management
            // calls to a proxy that may not even be running anymore.
            .no_proxy()
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            config_manager,
            http_client,
            inner: Arc::new(Mutex::new(SupervisorInner {
                child: None,
                endpoint: None,
            })),
            current: RwLock::new(None),
        }
    }

    /// Borrow the shared HTTP client for proxied asr-server requests.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// `https://<host>:<port>` for the currently-running asr-server, or `None`
    /// if the supervisor has no live endpoint. Safe to call from any handler.
    pub async fn endpoint(&self) -> Option<String> {
        self.inner
            .lock()
            .await
            .endpoint
            .as_ref()
            .map(|e| e.base_url.clone())
    }

    /// The bearer token to present to the asr-server, or `None`.
    pub async fn bearer_token(&self) -> Option<String> {
        self.inner
            .lock()
            .await
            .endpoint
            .as_ref()
            .and_then(|e| e.token.clone())
    }

    /// Convenience: build a URL `"<endpoint><path>"`, falling back to the
    /// configured-as-default `https://127.0.0.1:8398` when the supervisor has
    /// not (yet) established an endpoint. The fallback keeps legacy
    /// behavior workable when ASR is disabled but a request sneaks through.
    pub async fn url_for(&self, path: &str) -> String {
        self.endpoint()
            .await
            .unwrap_or_else(|| format!("https://127.0.0.1:{}", crate::config::DEFAULT_ASR_PORT))
            + path
    }

    /// Ensure the asr-server is running per `cfg`. Verifies authenticated
    /// configuration before attaching; spawns only if no instance is
    /// reachable. Safe to call repeatedly — if the current endpoint already
    /// matches `cfg`, returns immediately.
    pub async fn ensure_running(&self, cfg: &AsrConfig) -> Result<()> {
        // Fast path: already running with matching config.
        {
            let inner = self.inner.lock().await;
            if let Some(ep) = &inner.endpoint {
                let desired_port = desired_port(cfg);
                let desired_token = cfg.effective_token();
                if endpoint_matches(ep, &cfg.bind_address, desired_port)
                    && ep.token == desired_token
                {
                    return Ok(());
                }
            }
        }

        // Wildcard addresses are for listening, not connecting. Manage the
        // local process over loopback even when it is exposed on every interface.
        let candidate_url = management_base_url(&cfg.bind_address, desired_port(cfg));
        let token = cfg.effective_token();
        match probe_attachable(&self.http_client, &candidate_url, token.as_deref()).await {
            Ok(()) => {
                let ep = SupervisedEndpoint {
                    base_url: candidate_url,
                    token: token.clone(),
                };
                info!(
                    "Attached to existing vterm-asr-server at {} (not spawned by us)",
                    ep.base_url
                );
                self.record_endpoint(Some(ep.clone()), cfg.clone()).await;
                return Ok(());
            }
            Err(attach_error) => {
                // `/health` is intentionally public, so use it only to tell
                // an unavailable server from a reachable server with an
                // incompatible token. Spawning in the latter case would fail
                // due to the singleton lock and incorrectly look successful.
                if probe_health(&self.http_client, &candidate_url, None)
                    .await
                    .is_ok()
                {
                    anyhow::bail!(
                        "vterm-asr-server is already running at {candidate_url}, but its access-token configuration does not match this server: {attach_error}"
                    );
                }
            }
        }

        // Spawn a fresh child.
        let ep = self.spawn_child(cfg).await.with_context(|| {
            format!(
                "Failed to start vterm-asr-server at {}:{}",
                cfg.bind_address,
                desired_port(cfg)
            )
        })?;
        self.record_endpoint(Some(ep.clone()), cfg.clone()).await;
        Ok(())
    }

    /// Stop the Voice API at loopback endpoints, then kill/reap any child we
    /// spawned as a fallback. The API request also cleans up a stale local
    /// instance that we attached to instead of spawning ourselves.
    pub async fn stop(&self) {
        let mut inner = self.inner.lock().await;
        let endpoint = inner.endpoint.take();
        let child = inner.child.take();
        drop(inner);

        if let Some(endpoint) = endpoint
            .as_ref()
            .filter(|ep| is_loopback_endpoint(&ep.base_url))
        {
            let mut request = self
                .http_client
                .post(format!("{}/v1/shutdown", endpoint.base_url));
            if let Some(token) = endpoint.token.as_deref() {
                request = request.bearer_auth(token);
            }
            match tokio::time::timeout(std::time::Duration::from_secs(3), request.send()).await {
                Ok(Ok(response)) if response.status().is_success() => {
                    info!("Requested local vterm-asr-server shutdown via Voice API");
                }
                Ok(Ok(response)) => {
                    warn!(status = %response.status(), "Voice API shutdown request was rejected")
                }
                Ok(Err(e)) => debug!("Voice API shutdown request failed: {e}"),
                Err(_) => debug!("Voice API shutdown request timed out"),
            }
        }

        if let Some(mut child) = child {
            // The API request above is graceful; retain process termination as
            // a second safeguard for our direct child.
            let _ = child.kill().await;
            let _ = child.wait().await;
            info!("Stopped supervised vterm-asr-server child");
        }
        *self.current.write().await = None;
    }

    /// Snapshot of the config currently in effect for the running process.
    pub async fn current_config(&self) -> Option<AsrConfig> {
        self.current.read().await.clone()
    }

    /// Probe the asr-server's `/v1/model/status` and translate the response
    /// into a [`VoiceServiceState`]. Returns `NotLoaded` if the supervisor has
    /// no live endpoint or the probe fails — this mirrors the legacy
    /// in-process behavior where an unreachable model reads as "not loaded".
    pub async fn model_status(&self) -> VoiceServiceState {
        let url = match self.endpoint().await {
            Some(u) => u,
            None => return VoiceServiceState::NotLoaded,
        };
        let token = self.bearer_token().await;
        let mut req = self.http_client.get(format!("{url}/v1/model/status"));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let resp = match req.send().await {
            Ok(r) => r,
            Err(_) => return VoiceServiceState::NotLoaded,
        };
        if !resp.status().is_success() {
            return VoiceServiceState::NotLoaded;
        }
        let json: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => return VoiceServiceState::NotLoaded,
        };
        match json
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("not_loaded")
        {
            "loading" => VoiceServiceState::Loading,
            "loaded" => VoiceServiceState::Loaded {
                model_id: json
                    .get("model_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                device: json
                    .get("device")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            },
            "error" => VoiceServiceState::Error {
                message: json
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string(),
            },
            _ => VoiceServiceState::NotLoaded,
        }
    }

    /// Trigger a remote model load by POSTing to the asr-server's
    /// `/v1/model/load`. Requires the supervisor's endpoint to be live.
    /// The asr-server loads asynchronously and reports progress via
    /// `/v1/model/status`; callers should poll [`Self::model_status`].
    pub async fn request_model_load(&self) -> Result<()> {
        let url = self.url_for("/v1/model/load").await;
        let token = self.bearer_token().await;
        let mut req = self.http_client.post(&url);
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let resp = req.send().await?.error_for_status()?;
        let _ = resp.bytes().await;
        Ok(())
    }

    /// Trigger a remote model unload by POSTing to `/v1/model/unload`.
    pub async fn request_model_unload(&self) -> Result<()> {
        let url = self.url_for("/v1/model/unload").await;
        let token = self.bearer_token().await;
        let mut req = self.http_client.post(&url);
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let resp = req.send().await?.error_for_status()?;
        let _ = resp.bytes().await;
        Ok(())
    }

    /// Transcribe raw f32 audio samples by POSTing them as a WAV to the
    /// asr-server's `/v1/audio/transcriptions` endpoint. Used by the session
    /// `TranscribeAudio` handler and equivalent to the legacy in-process
    /// `VoiceService::transcribe` call. Returns the transcribed text.
    ///
    /// (Phase 4 will additionally route this through the configured voice
    /// addon for providers other than `local`; the supervisor path remains
    /// the default for the `local` provider.)
    pub async fn transcribe_samples(
        &self,
        samples: Vec<f32>,
        language: Option<&str>,
    ) -> Result<String> {
        let url = self.url_for("/v1/audio/transcriptions").await;
        let token = self.bearer_token().await;

        // Wrap f32 samples into a 16kHz mono WAV so the asr-server's standard
        // multipart parser accepts them.
        let wav = super::wav::encode_pcm_f32_16k_mono(&samples);
        let boundary = super::wav::random_multipart_boundary();
        let body =
            super::wav::build_multipart_file(&boundary, "file", "audio.wav", "audio/wav", &wav);
        let content_type = format!("multipart/form-data; boundary={boundary}");

        let mut req = self
            .http_client
            .post(&url)
            .header("content-type", content_type)
            .body(body);
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        if let Some(lang) = language {
            req = req.query(&[("language", lang)]);
        }
        let resp = req.send().await?.error_for_status()?;
        let json: serde_json::Value = resp.json().await?;
        let text = json
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("asr-server response missing 'text' field"))?
            .to_string();
        Ok(text)
    }

    async fn record_endpoint(&self, ep: Option<SupervisedEndpoint>, cfg: AsrConfig) {
        let mut inner = self.inner.lock().await;
        inner.endpoint = ep;
        *self.current.write().await = Some(cfg);
    }

    async fn spawn_child(&self, cfg: &AsrConfig) -> Result<SupervisedEndpoint> {
        let bin = asr_server_binary_path().with_context(|| {
            "vterm-voice binary not found. Build it with `cargo build -p vterm-asr-server` and place the resulting `vterm-voice` next to vterm-server (or under ~/.vibeterminal/bin/)."
        })?;

        let mut cmd = Command::new(&bin);
        cmd.kill_on_drop(true)
            .arg("--host")
            .arg(&cfg.bind_address)
            .arg("--port")
            .arg(cfg.port.to_string())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        // Same CREATE_NO_WINDOW rationale as vterm_common::command_no_window,
        // inline because this is a tokio Command.
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x0800_0000);
        let model_path =
            crate::config::resolve_model_path(&cfg.model_path, &cfg.model_relative_path);
        if !model_path.trim().is_empty() {
            cmd.arg("--model").arg(model_path);
        }
        if !cfg.device.trim().is_empty() {
            cmd.arg("--device").arg(&cfg.device);
        }
        if let Some(t) = cfg.effective_token() {
            cmd.arg("--access-token").arg(t);
        }

        info!("Spawning vterm-asr-server: {:?}", cmd.as_std());
        let mut child = cmd
            .spawn()
            .with_context(|| format!("failed to spawn {}", bin.display()))?;

        // Drain stdout + stderr → tracing for the lifetime of the child. We
        // always bind the configured port, so there's no need to parse startup
        // output for the bound port.
//         if let Some(stdout) = child.stdout.take() {
//             tokio::spawn(async move {
//                 let mut lines = BufReader::new(stdout).lines();
//                 while let Ok(Some(line)) = lines.next_line().await {
//                     debug!("[asr-server:stdout] {line}");
//                 }
//             });
//         }
//         if let Some(stderr) = child.stderr.take() {
//             tokio::spawn(async move {
//                 let mut lines = BufReader::new(stderr).lines();
//                 while let Ok(Some(line)) = lines.next_line().await {
//                     // Forward asr-server logs verbatim; they're already formatted.
//                     eprintln!("[asr-server] {line}");
//                 }
//             });
//         }

        let base_url = management_base_url(&cfg.bind_address, cfg.port);
        // Wait for /health to go green before declaring success.
        wait_health(
            &self.http_client,
            &base_url,
            cfg.effective_token().as_deref(),
            HEALTH_PROBE_TIMEOUT,
        )
        .await
        .with_context(|| format!("asr-server never became healthy at {base_url}"))?;

        let ep = SupervisedEndpoint {
            base_url,
            token: cfg.effective_token(),
        };

        // Stash the child handle.
        {
            let mut inner = self.inner.lock().await;
            // If there's a prior child (e.g. race), kill it.
            if let Some(mut prev) = inner.child.take() {
                let _ = prev.kill().await;
            }
            inner.child = Some(child);
            inner.endpoint = Some(ep.clone());
        }
        info!("vterm-asr-server is ready at {}", ep.base_url);
        Ok(ep)
    }
}

impl AsrSupervisor {
    /// Drive the supervisor: subscribe to config changes and hot-restart the
    /// child when bind/port/token change. Runs forever; spawn on a background
    /// task. Exits when the config watch sender drops (shutdown).
    ///
    /// **Lifecycle is decoupled from `asr.enabled`.** The `vterm-voice` process
    /// is a standalone ASR server that may serve multiple vterm-server
    /// instances, so toggling "Enable Voice Recognition" must NOT tear it down.
    /// The supervisor:
    /// - brings the child up automatically only when `asr.load_on_startup` is
    ///   set (a one-time boot choice, not a runtime toggle);
    /// - otherwise starts/stops strictly on explicit user action via the
    ///   `ensure_running` / `stop` methods (the WebUI Start/Stop buttons);
    /// - hot-restarts only when the child is ALREADY running and an
    ///   endpoint-affecting field (bind_address/port/access_token) changed.
    pub async fn run(self: Arc<Self>) {
        let mut cfg_rx = self.config_manager.subscribe();
        // Initial bring-up: only if the user opted into load_on_startup. We do
        // NOT consult `asr.enabled` here — that flag gates whether vterm-server
        // itself consumes voice input, not whether the asr-server process runs.
        let initial = cfg_rx.borrow().asr.clone();
        if initial.load_on_startup {
            if let Err(e) = self.ensure_running(&initial).await {
                error!("Initial asr-server bring-up failed: {e:?}");
            }
        }

        loop {
            if cfg_rx.changed().await.is_err() {
                // Sender dropped (config manager going away) — shut down child.
                self.stop().await;
                return;
            }
            let new_cfg = cfg_rx.borrow().asr.clone();
            let cur = self.current_config().await;

            // Only act if a child is currently running. If it's stopped, leave
            // it stopped — the user starts it explicitly via the WebUI. This is
            // the decoupling point: an `asr.enabled=false` change with no
            // running child is a no-op, and a stopped child stays stopped even
            // when ASR is enabled.
            let Some(prev) = cur else {
                continue;
            };

            // Decide whether the change forces a process restart vs. an HTTP
            // config push. Endpoint-affecting fields: bind_address, port,
            // access_token. Everything else (model_path, device) can be pushed
            // via PUT /v1/config without a restart — the voice handlers do
            // that as part of update_voice_config.
            //
            // The token comparison uses effective_token() so a change to
            // provider_config["access_token"] (the addon's own token field)
            // also triggers a restart — the supervisor spawns with whatever
            // effective_token() returns.
            let needs_restart = prev.bind_address != new_cfg.bind_address
                || prev.port != new_cfg.port
                || prev.effective_token() != new_cfg.effective_token();

            if needs_restart {
                warn!("ASR endpoint config changed; restarting supervised asr-server");
                self.stop().await;
                if let Err(e) = self.ensure_running(&new_cfg).await {
                    error!("asr-server restart failed: {e:?}");
                }
            }
            // else: non-restart config change — handlers push PUT /v1/config.
        }
    }
}

/// The port the supervisor passes to the CLI and expects on the wire. With
/// `auto_port` removed this is always `cfg.port`, but the helper is retained
/// as a single place to centralize port resolution.
fn desired_port(cfg: &AsrConfig) -> u16 {
    cfg.port
}

fn management_base_url(bind_address: &str, port: u16) -> String {
    let host = match bind_address.trim() {
        "0.0.0.0" | "::" | "[::]" => "127.0.0.1",
        host => host,
    };
    let authority = if host.contains(':') && !(host.starts_with('[') && host.ends_with(']')) {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    format!("https://{authority}:{port}")
}

fn endpoint_matches(ep: &SupervisedEndpoint, host: &str, port: u16) -> bool {
    ep.base_url == management_base_url(host, port)
}

/// Web API shutdown is safe only for a process that is bound to this machine.
/// Do not issue this lifecycle request to a remotely configured ASR endpoint.
fn is_loopback_endpoint(base_url: &str) -> bool {
    base_url
        .strip_prefix("https://")
        .or_else(|| base_url.strip_prefix("http://"))
        .and_then(|authority| authority.rsplit_once(':').map(|(host, _)| host))
        .is_some_and(|host| {
            matches!(
                host.trim_matches(['[', ']']),
                "localhost" | "127.0.0.1" | "::1"
            )
        })
}

async fn probe_health(client: &reqwest::Client, base_url: &str, token: Option<&str>) -> Result<()> {
    let mut req = client.get(format!("{base_url}/health"));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await?.error_for_status()?;
    // Body is "ok" — accept any 2xx.
    let _ = resp.text().await;
    Ok(())
}

/// Verify that an existing Voice API accepts our token and uses the same
/// authentication mode. `/health` cannot serve this purpose because it is
/// deliberately public for startup readiness checks.
async fn probe_attachable(
    client: &reqwest::Client,
    base_url: &str,
    token: Option<&str>,
) -> Result<()> {
    let mut req = client.get(format!("{base_url}/v1/config"));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let config: serde_json::Value = req.send().await?.error_for_status()?.json().await?;
    let server_requires_token = config
        .pointer("/server/hasAccessToken")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow::anyhow!("Voice API config response lacks server.hasAccessToken"))?;
    let supervisor_has_token = token.is_some();
    if server_requires_token != supervisor_has_token {
        anyhow::bail!(
            "Voice API authentication mode differs (server requires token: {server_requires_token}, supervisor has token: {supervisor_has_token})"
        );
    }
    Ok(())
}

async fn wait_health(
    client: &reqwest::Client,
    base_url: &str,
    token: Option<&str>,
    timeout: std::time::Duration,
) -> Result<()> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("health probe timed out after {:?}", timeout);
        }
        match probe_health(client, base_url, token).await {
            Ok(()) => return Ok(()),
            Err(_) => tokio::time::sleep(HEALTH_PROBE_INTERVAL).await,
        }
    }
}

/// Locate the `vterm-voice` binary (the renamed `vterm-asr-server` executable).
///
/// Lookup order:
/// 1. **Sibling of the running `vterm-server` executable.** In dev this is the
///    shared workspace `target/<profile>/` (both crates build there side by
///    side); in a packaged install it's whatever directory ships alongside
///    `vterm-server`. This is the primary path because it matches how the
///    binaries are actually laid out at runtime.
/// 2. **`$HOME/.vibeterminal[dev]/bin/vterm-voice[.exe]`** as a fallback for
///    deployments that ship the supervisor binary separately from the ASR
///    server.
///
/// Returns an error naming both probed locations so the user knows where to
/// drop the binary.
pub fn asr_server_binary_path() -> Result<PathBuf> {
    /// Append `vterm-voice` (+ `.exe` on Windows) to a directory, returning
    /// the path only if it exists.
    fn candidate(dir: &std::path::Path) -> Option<PathBuf> {
        let mut p = dir.join("vterm-voice");
        if cfg!(target_os = "windows") {
            p.set_extension("exe");
        }
        if p.exists() {
            Some(p)
        } else {
            None
        }
    }

    // 1. Sibling of current_exe.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(p) = candidate(parent) {
                return Ok(p);
            }
        }
    }

    // 2. Per-flavor install dir under $HOME.
    if let Some(home) = dirs::home_dir() {
        let dir = if cfg!(DEVELOPMENT) {
            ".vibeterminal-dev"
        } else {
            ".vibeterminal"
        };
        if let Some(p) = candidate(&home.join(dir).join("bin")) {
            return Ok(p);
        }
    }

    anyhow::bail!(
        "vterm-voice binary not found. Build it with `cargo build -p vterm-asr-server` \
         (it produces a binary named `vterm-voice`) and either place it next to the \
         running vterm-server executable or under $HOME/.vibeterminal/bin/."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_binds_use_loopback_for_local_management() {
        assert_eq!(
            management_base_url("0.0.0.0", 8398),
            "https://127.0.0.1:8398"
        );
        assert_eq!(management_base_url("::", 8398), "https://127.0.0.1:8398");
        assert_eq!(management_base_url("[::]", 8398), "https://127.0.0.1:8398");

        let ep = SupervisedEndpoint {
            base_url: "https://127.0.0.1:8398".into(),
            token: None,
        };
        assert!(endpoint_matches(&ep, "127.0.0.1", 8398));
        assert!(endpoint_matches(&ep, "0.0.0.0", 8398));
        assert!(endpoint_matches(&ep, "::", 8398));
        assert!(!endpoint_matches(&ep, "0.0.0.0", 9000));
    }

    #[test]
    fn ipv6_management_urls_are_bracketed() {
        assert_eq!(management_base_url("::1", 8398), "https://[::1]:8398");
        assert_eq!(management_base_url("[::1]", 8398), "https://[::1]:8398");
    }

    #[test]
    fn desired_port_returns_configured() {
        let mut cfg = AsrConfig::default();
        cfg.port = 12345;
        assert_eq!(desired_port(&cfg), 12345);
    }

    #[test]
    fn api_shutdown_is_limited_to_requested_loopback_hosts() {
        assert!(is_loopback_endpoint("https://localhost:8398"));
        assert!(is_loopback_endpoint("https://127.0.0.1:8398"));
        assert!(is_loopback_endpoint("https://[::1]:8398"));
        assert!(!is_loopback_endpoint("https://192.168.1.10:8398"));
        assert!(!is_loopback_endpoint("https://voice.example.com:8398"));
    }

    #[test]
    fn resolve_model_path_joins_root_and_relative_model_path() {
        use crate::config::resolve_model_path;

        let mut cfg = AsrConfig::default();
        cfg.model_path = "/models".into();
        cfg.model_relative_path = "qwen3-asr-0.6b".into();
        // The joined directory does not exist in this test and holds no
        // discoverable copy under /models, so the join is both the fast path
        // miss and the fallback result.
        assert_eq!(
            resolve_model_path(&cfg.model_path, &cfg.model_relative_path),
            std::path::Path::new("/models")
                .join("qwen3-asr-0.6b")
                .display()
                .to_string(),
        );
    }

    #[test]
    fn resolve_model_path_finds_manually_named_model_directory() {
        use crate::config::resolve_model_path;

        // A manually cloned repository keeps its upstream directory name; the
        // discovery-by-config-signature path must still find it where the
        // plain join would miss.
        let root = std::env::temp_dir().join(format!(
            "vterm-resolve-model-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let model = root.join("Qwen3-ASR-0.6B-4bit");
        std::fs::create_dir_all(&model).unwrap();
        std::fs::write(
            model.join("config.json"),
            r#"{
                "model_type": "qwen3_asr",
                "quantization": { "bits": 4 },
                "thinker_config": {
                    "audio_config": { "encoder_layers": 18 },
                    "text_config": {}
                }
            }"#,
        )
        .unwrap();
        std::fs::write(model.join("model.safetensors"), b"weights").unwrap();

        assert_eq!(
            resolve_model_path(root.to_str().unwrap(), "qwen3-asr-0.6b-4bit"),
            model.display().to_string(),
        );
        std::fs::remove_dir_all(&root).unwrap();
    }
}
