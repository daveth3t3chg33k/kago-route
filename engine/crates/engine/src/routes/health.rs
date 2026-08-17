//! Liveness probe: reports engine status, session store and database health.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;

use crate::state::AppState;

pub async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let database = match &state.db {
        Some(pool) => match sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(pool).await {
            Ok(_) => "ok",
            Err(_) => "unavailable",
        },
        None => "not-configured",
    };

    Json(serde_json::json!({
        "status": "ok",
        "service": "kagoroute-engine",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": state.started_at.elapsed().as_secs(),
        "session_store": state.session_store.name(),
        "database": database,
    }))
}
