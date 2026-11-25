use std::sync::Arc;

use serde::Deserialize;

use crate::config::types::ConfigError;
use crate::static_http::StaticHttpManager;

use super::super::server::{HttpRequest, HttpResponse};

/// List the directories currently served over static HTTP — the same data
/// the tray icon menu shows.
pub(crate) async fn list_services() -> Result<HttpResponse, ConfigError> {
    let services = crate::static_http::hosted_services()
        .read()
        .expect("hosted HTTP services poisoned")
        .clone();
    Ok(HttpResponse::json(&services)?)
}

#[derive(Deserialize)]
struct StopServiceRequest {
    root: String,
}

/// Stop serving a directory (mirrors the tray icon's stop action).
pub(crate) async fn stop_service(
    static_http_manager: &Arc<StaticHttpManager>,
    request: &HttpRequest,
) -> Result<HttpResponse, ConfigError> {
    let body: StopServiceRequest = serde_json::from_slice(&request.body)
        .map_err(|error| ConfigError::BadRequest(error.to_string()))?;
    let stopped = static_http_manager.stop_directory(&body.root).await;
    Ok(HttpResponse::json(
        &serde_json::json!({ "stopped": stopped }),
    )?)
}
