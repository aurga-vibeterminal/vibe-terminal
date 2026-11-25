use std::sync::Arc;

use serde::{Deserialize, Serialize};
use vterm_common::private_ca::CertificateInfo;

use crate::config::store::ConfigManager;
use crate::config::types::{ConfigError, TerminalServerConfig};
use crate::private_ca::{self, PrivateCaManager};

use crate::config::server::HttpResponse;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CertificateStatus {
    root_ca: Option<CertificateInfo>,
    leaf_certificate: Option<CertificateInfo>,
    automatic_sans: Vec<String>,
    extra_sans: Vec<String>,
    san_preview: Vec<String>,
    managed_identity_active: bool,
    last_error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtraSansRequest {
    extra_sans: Vec<String>,
}

pub(super) fn get_status(
    private_ca: &PrivateCaManager,
    config: &TerminalServerConfig,
) -> Result<HttpResponse, ConfigError> {
    Ok(HttpResponse::json(&status(private_ca, config))?)
}

pub(super) async fn update_extra_sans(
    private_ca: Arc<PrivateCaManager>,
    config_manager: Arc<ConfigManager>,
    body: &[u8],
) -> Result<HttpResponse, ConfigError> {
    let payload: ExtraSansRequest = serde_json::from_slice(body)?;
    let extra_sans = private_ca::normalized_extra_sans(&payload.extra_sans)
        .map_err(|error| ConfigError::BadRequest(error.to_string()))?;
    let manager = Arc::clone(&private_ca);
    let issue_sans = extra_sans.clone();
    tokio::task::spawn_blocking(move || manager.reissue_certificate(&issue_sans))
        .await
        .map_err(join_error)?
        .map_err(|error| ConfigError::BadRequest(error.to_string()))?;

    let mut config = config_manager.snapshot().await;
    config.server.tls_extra_sans = extra_sans;
    let update = config_manager.replace(config).await?;
    Ok(HttpResponse::json(&status(&private_ca, &update.config))?)
}

pub(super) async fn reissue(
    private_ca: Arc<PrivateCaManager>,
    config_manager: Arc<ConfigManager>,
) -> Result<HttpResponse, ConfigError> {
    let config = config_manager.snapshot().await;
    let manager = Arc::clone(&private_ca);
    let extra_sans = config.server.tls_extra_sans.clone();
    tokio::task::spawn_blocking(move || manager.reissue_certificate(&extra_sans))
        .await
        .map_err(join_error)?
        .map_err(|error| ConfigError::BadRequest(error.to_string()))?;
    Ok(HttpResponse::json(&status(&private_ca, &config))?)
}

pub(super) fn download_root_ca(private_ca: &PrivateCaManager) -> Result<HttpResponse, ConfigError> {
    let pem = private_ca
        .root_ca_pem()
        .map_err(|error| ConfigError::Io(std::io::Error::other(error)))?;
    Ok(HttpResponse::bytes_with_headers(
        "application/x-pem-file",
        pem.as_bytes(),
        vec![
            (
                "Content-Disposition".to_string(),
                "attachment; filename=\"vibe-terminal-root-ca.pem\"".to_string(),
            ),
            ("Cache-Control".to_string(), "no-store".to_string()),
        ],
    ))
}

fn status(private_ca: &PrivateCaManager, config: &TerminalServerConfig) -> CertificateStatus {
    CertificateStatus {
        root_ca: private_ca.root_certificate_info(),
        leaf_certificate: private_ca.leaf_certificate_info(),
        automatic_sans: private_ca::automatic_sans(),
        extra_sans: config.server.tls_extra_sans.clone(),
        san_preview: private_ca::combined_sans(&config.server.tls_extra_sans).unwrap_or_default(),
        managed_identity_active: private_ca.managed_identity_active(),
        last_error: private_ca.last_error(),
    }
}

fn join_error(error: tokio::task::JoinError) -> ConfigError {
    ConfigError::Io(std::io::Error::other(error))
}
