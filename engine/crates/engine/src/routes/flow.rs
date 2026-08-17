//! Flow metadata endpoint — describes the loaded (single) schema. Will become
//! a tenant-scoped flows API in a later milestone.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;

use crate::state::AppState;

pub async fn flow(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let flow = &state.flow;
    let nodes: Vec<&str> = flow.nodes.keys().map(String::as_str).collect();
    Json(serde_json::json!({
        "id": flow.id,
        "name": flow.name,
        "description": flow.description,
        "version": flow.version,
        "start": flow.start,
        "timeouts": {
            "session": flow.timeouts.session,
            "step": flow.timeouts.step,
        },
        "nodes": nodes,
    }))
}
