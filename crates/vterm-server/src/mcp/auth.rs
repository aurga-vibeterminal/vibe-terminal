//! Bearer-token auth for the MCP HTTP endpoint.
//!
//! The router bootstrap ([`super::McpServer::run`]) stashes the
//! expected token in a request [`axum::Extension`] named [`ExpectedToken`],
//! and attaches the peer address as [`axum::extract::ConnectInfo`] by serving
//! through `into_make_service_with_connect_info`. The [`require_bearer`]
//! middleware enforces:
//!
//! - **Loopback peers** (127.x.x.x, ::1) are always allowed: a process already
//!   running on this machine holds the same power as the MCP endpoint, so a
//!   token adds nothing there.
//! - **Everything else** must carry `Authorization: Bearer <token>` matching
//!   the configured token. The terminal MCP tools can drive the whole
//!   machine, so remote requests are only admitted with a configured
//!   (non-empty, matching) token.
//!
//! ## Layer ordering (IMPORTANT)
//!
//! `axum::Extension` inserts its value into the request *before* calling the
//! inner service. For `require_bearer` to see the token, the `Extension` layers
//! must be the **outermost** (run first). Since `Router::layer` makes the
//! *last* `.layer()` call the outermost, the bootstrap adds them in this order:
//!
//! ```text,ignore
//! Router::new()
//!     .nest_service("/mcp", service)
//!     .layer(middleware::from_fn(require_bearer))  // inner
//!     .layer(Extension(ExpectedToken(token)));      // outer (runs first)
//! ```

use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;

/// The expected access token, stashed in a request extension by the router
/// bootstrap so the middleware can compare against the `Authorization` header.
/// An empty string rejects every non-loopback request.
#[derive(Clone, Debug)]
pub struct ExpectedToken(pub String);

/// Enforce access by origin and bearer token:
///
/// - **Matching bearer token** → request proceeds (any origin).
/// - **Loopback peer** → request proceeds without any token check.
/// - **Anything else** → `401` (including remote peers when no token is
///   configured, and requests whose peer address is unknown — fail closed).
pub async fn require_bearer(req: Request<Body>, next: Next) -> Response {
    let expected = req.extensions().get::<ExpectedToken>().cloned();
    let provided = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));
    let loopback = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip().is_loopback())
        .unwrap_or(false);

    let token_ok = matches!(&expected, Some(exp)
        if !exp.0.is_empty() && provided == Some(exp.0.as_str()));
    if token_ok || loopback {
        return next.run(req).await;
    }
    (StatusCode::UNAUTHORIZED, "invalid or missing bearer token").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{middleware, routing::get, Extension, Router};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use tower::ServiceExt;

    /// A one-route app wrapped like the MCP host bootstrap (auth middleware
    /// inner, token + peer-address extensions outer). `peer: None` simulates
    /// a router served without `into_make_service_with_connect_info`.
    fn app(token: &str, peer: Option<IpAddr>) -> Router {
        let mut app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(middleware::from_fn(require_bearer));
        if let Some(ip) = peer {
            app = app.layer(Extension(ConnectInfo(SocketAddr::new(ip, 52341))));
        }
        app.layer(Extension(ExpectedToken(token.to_string())))
    }

    fn request(authorization: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().uri("/");
        if let Some(value) = authorization {
            builder = builder.header(axum::http::header::AUTHORIZATION, value);
        }
        builder.body(Body::empty()).unwrap()
    }

    const LOCAL: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
    const LOCAL_V6: IpAddr = IpAddr::V6(Ipv6Addr::LOCALHOST);
    const REMOTE: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50));

    #[tokio::test]
    async fn loopback_never_needs_a_token() {
        // Empty configured token, no header — still allowed.
        let response = app("", Some(LOCAL)).oneshot(request(None)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Token configured but header missing — loopback stays exempt.
        let response = app("secret", Some(LOCAL))
            .oneshot(request(None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Even a wrong header passes: loopback skips the token check.
        let response = app("secret", Some(LOCAL))
            .oneshot(request(Some("Bearer wrong")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn ipv6_loopback_is_exempt_too() {
        let response = app("", Some(LOCAL_V6))
            .oneshot(request(None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn remote_requires_matching_token() {
        let response = app("secret", Some(REMOTE))
            .oneshot(request(Some("Bearer secret")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn remote_rejects_missing_or_wrong_token() {
        for authorization in [None, Some("Bearer wrong")] {
            let response = app("secret", Some(REMOTE))
                .oneshot(request(authorization))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }

    /// Without a configured token every remote request is rejected: the
    /// endpoint must never admit unknown remote peers just because the user
    /// never set a token.
    #[tokio::test]
    async fn remote_with_empty_configured_token_is_rejected() {
        let response = app("", Some(REMOTE)).oneshot(request(None)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Unknown peer (no ConnectInfo) fails closed unless a valid token is
    /// presented — a valid token proves authorization regardless of origin.
    #[tokio::test]
    async fn unknown_peer_needs_a_valid_token() {
        let response = app("secret", None)
            .oneshot(request(Some("Bearer secret")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app("secret", None).oneshot(request(None)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
