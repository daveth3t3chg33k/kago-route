//! HTTP route wiring.

pub mod auth;
pub mod flow;
pub mod health;
pub mod sessions;
pub mod ussd;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderName, HeaderValue, Method};
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::Security;
use crate::routes::auth::{require_callback_key, CallbackAuth};
use crate::state::AppState;

/// Header clients must send with USSD callbacks when a secret is configured.
pub const CALLBACK_KEY_HEADER: &str = "x-kagoroute-key";

pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = cors_layer(&state.security);

    Router::new()
        .route("/health", get(health::health))
        .route("/flow", get(flow::flow))
        .route("/flow/schema", get(flow::flow_schema))
        .route("/sessions", get(sessions::sessions))
        .route(
            "/ussd/callback",
            post(ussd::callback)
                // Auth middleware wraps the handler, so unauthorized requests
                // are rejected before their body is read at all.
                .route_layer(from_fn_with_state(
                    CallbackAuth {
                        secrets: state.security.callback_secrets.clone(),
                    },
                    require_callback_key,
                ))
                .layer(DefaultBodyLimit::max(state.security.max_body_bytes)),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Restricted CORS: only configured origins (e.g. the dashboard), only the
/// methods/headers the engine exposes. Requests without an `Origin` header
/// (curl, carriers, server-to-server) pass through untouched.
fn cors_layer(security: &Security) -> CorsLayer {
    let origins: Vec<HeaderValue> = security
        .cors_allowed_origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([
            CONTENT_TYPE,
            AUTHORIZATION,
            HeaderName::from_static(CALLBACK_KEY_HEADER),
        ])
        .max_age(Duration::from_secs(600))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Security;
    use crate::schema::load_flow;
    use crate::session::memory::MemoryStore;
    use crate::session::SessionStore;
    use std::time::Instant;
    use tower::ServiceExt;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};

    fn app(secrets: &[&str], max_body_bytes: usize) -> Router {
        let state = Arc::new(AppState {
            flow: load_flow(None).expect("demo loads"),
            session_store: Arc::new(SessionStore::Memory(MemoryStore::default())),
            db: None,
            started_at: Instant::now(),
            security: Security {
                callback_secrets: secrets.iter().map(|s| s.to_string()).collect(),
                cors_allowed_origins: vec!["http://localhost:3000".to_string()],
                max_body_bytes,
            },
        });
        build_router(state)
    }

    fn callback_request(secret: Option<&str>, body: &str) -> Request<Body> {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/ussd/callback")
            .header("content-type", "application/x-www-form-urlencoded");
        if let Some(key) = secret {
            builder = builder.header(CALLBACK_KEY_HEADER, key);
        }
        builder
            .body(Body::from(
                format!(
                    "sessionId=t1&serviceCode=%2A483%2A42%23&phoneNumber=254712345678&text={body}"
                )
                .to_string(),
            ))
            .unwrap()
    }

    #[tokio::test]
    async fn callback_rejected_without_secret() {
        let res = app(&["sekrit"], 4096)
            .oneshot(callback_request(None, ""))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        // Auth rejects before the walker runs: the body must not be a CON/END.
        let body = axum::body::to_bytes(res.into_body(), 4096).await.unwrap();
        assert!(body.is_empty() || !body.starts_with(b"CON"));
    }

    #[tokio::test]
    async fn callback_rejected_with_wrong_secret() {
        let res = app(&["sekrit"], 4096)
            .oneshot(callback_request(Some("nope"), ""))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn callback_accepted_with_correct_secret() {
        let res = app(&["sekrit"], 4096)
            .oneshot(callback_request(Some("sekrit"), ""))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), 4096).await.unwrap();
        assert!(body.starts_with(b"CON "));
    }

    #[tokio::test]
    async fn any_configured_secret_accepted() {
        // Per-tenant keys: each tenant's key is accepted.
        let res = app(&["tenant-a", "tenant-b"], 4096)
            .oneshot(callback_request(Some("tenant-b"), ""))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn callback_open_when_no_secret_configured() {
        let res = app(&[], 4096)
            .oneshot(callback_request(None, ""))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn oversized_callback_body_rejected() {
        let big = "x".repeat(512);
        let res = app(&[], 256)
            .oneshot(callback_request(None, &big))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn body_at_limit_is_accepted() {
        // 413 only applies *over* the limit, not at it.
        let res = app(&[], 4096)
            .oneshot(callback_request(None, &"y".repeat(4000)))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unauthorized_request_is_rejected_before_body_read() {
        // Auth middleware runs first: an oversized body from an unauthorized
        // caller must get 401, not 413 (its body is never read).
        let big = "x".repeat(4096);
        let res = app(&["sekrit"], 256)
            .oneshot(callback_request(None, &big))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn disallowed_origin_gets_no_cors_headers() {
        // tower-http doesn't 403 actual (non-preflight) requests — it simply
        // omits Access-Control-Allow-Origin, so the browser blocks the read.
        let res = app(&[], 4096)
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("origin", "http://evil.example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().get("access-control-allow-origin").is_none());
    }

    #[tokio::test]
    async fn allowed_origin_gets_cors_headers() {
        let res = app(&[], 4096)
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("origin", "http://localhost:3000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers()
                .get("access-control-allow-origin")
                .and_then(|v| v.to_str().ok()),
            Some("http://localhost:3000")
        );
    }

    #[tokio::test]
    async fn preflight_from_disallowed_origin_gets_no_cors_headers() {
        // tower-http answers disallowed preflights without CORS headers (no
        // 403 status); the browser then rejects the real request.
        let res = app(&[], 4096)
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/ussd/callback")
                    .header("origin", "http://evil.example")
                    .header("access-control-request-method", "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(res.headers().get("access-control-allow-origin").is_none());
    }

    #[tokio::test]
    async fn preflight_from_allowed_origin_accepted() {
        let res = app(&[], 4096)
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/ussd/callback")
                    .header("origin", "http://localhost:3000")
                    .header("access-control-request-method", "POST")
                    .header("access-control-request-headers", "content-type, x-kagoroute-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers()
                .get("access-control-allow-origin")
                .and_then(|v| v.to_str().ok()),
            Some("http://localhost:3000")
        );
    }

    #[tokio::test]
    async fn flow_schema_endpoint_round_trips() {
        // GET /flow/schema must serialize a FlowDocument that the loader can
        // parse back — the builder depends on exact round-trips.
        let app = app(&[], 4096);
        let res = app
            .oneshot(Request::builder().uri("/flow/schema").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), 64 * 1024).await.unwrap();
        let doc: crate::schema::FlowDocument = serde_json::from_slice(&body).unwrap();
        assert_eq!(doc.schema, crate::schema::DSL_IDENTIFIER);
        assert_eq!(doc.flow.id, "farmer-order");
        assert!(doc.flow.nodes.contains_key("welcome"));
        assert!(doc.flow.nodes.contains_key("stk_flagged"));
    }

    #[tokio::test]
    async fn sessions_endpoint_lists_active_sessions() {
        let app = app(&[], 4096);

        // No sessions yet.
        let res = app
            .clone()
            .oneshot(Request::builder().uri("/sessions").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["count"], 0);

        // Walk a session, then it should appear.
        let _ = app
            .clone()
            .oneshot(callback_request(None, "1*1*2"))
            .await
            .unwrap();

        let res = app
            .oneshot(Request::builder().uri("/sessions").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = axum::body::to_bytes(res.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["count"], 1);
        assert_eq!(json["sessions"][0]["id"], "t1");
        assert!(json["sessions"][0]["vars"]["product"].is_string());
    }

    #[tokio::test]
    async fn request_without_origin_passes_cors() {
        // Server-to-server calls (carriers, curl) carry no Origin header and
        // must never be blocked by CORS.
        let res = app(&[], 4096)
            .oneshot(callback_request(None, ""))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
