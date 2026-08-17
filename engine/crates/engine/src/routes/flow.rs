//! Flow endpoints — describe and export the loaded (single) schema. Will
//! become a tenant-scoped flows API in a later milestone.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;

use crate::schema::FlowDocument;
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

/// The full loaded schema as a `FlowDocument` — what the visual builder loads
/// and edits. Serializes through the same serde types used to parse it, so
/// round-trips are exact.
pub async fn flow_schema(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let doc = FlowDocument {
        schema: crate::schema::DSL_IDENTIFIER.to_string(),
        flow: (*state.flow).clone(),
    };
    Json(serde_json::to_value(&doc).unwrap_or_else(|_| serde_json::Value::Null))
}
