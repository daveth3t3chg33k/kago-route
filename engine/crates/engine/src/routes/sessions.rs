//! Live session listing — powers the dashboard's "live sessions" panel.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;

use crate::state::AppState;

pub async fn sessions(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    // `(key, vars_json)` where key is `session:{id}:vars`.
    let active = state.session_store.active_sessions().await;

    let sessions: Vec<serde_json::Value> = active
        .into_iter()
        .map(|(key, vars_json)| {
            let id = key
                .strip_prefix("session:")
                .and_then(|k| k.strip_suffix(":vars"))
                .unwrap_or(&key)
                .to_string();
            let vars = serde_json::from_str::<serde_json::Value>(&vars_json)
                .unwrap_or(serde_json::Value::Null);
            serde_json::json!({ "id": id, "vars": vars })
        })
        .collect();

    Json(serde_json::json!({ "count": sessions.len(), "sessions": sessions }))
}
