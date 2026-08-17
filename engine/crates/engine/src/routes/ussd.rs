//! Inbound USSD callback handler.
//!
//! Accepts the fields Africa's Talking (and most aggregators) POST on every
//! session step — `sessionId`, `serviceCode`, `phoneNumber`, `text` — either
//! as `application/x-www-form-urlencoded` or `application/json`, walks the
//! loaded menu schema, and replies with a `CON` / `END` string exactly like a
//! carrier expects.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use serde::Deserialize;

use crate::schema::walk::{walk, WalkRequest};
use crate::state::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UssdCallback {
    pub session_id: String,
    pub service_code: String,
    pub phone_number: String,
    pub text: String,
}

/// Parse a callback body that may be form-encoded or JSON.
fn parse_callback(body: &str) -> Option<UssdCallback> {
    if let Ok(cb) = serde_urlencoded::from_str::<UssdCallback>(body) {
        return Some(cb);
    }
    serde_json::from_str::<UssdCallback>(body).ok()
}

pub async fn callback(State(state): State<Arc<AppState>>, body: String) -> Response {
    let Some(cb) = parse_callback(&body) else {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid USSD callback: expected form-encoded or JSON fields \
             (sessionId, serviceCode, phoneNumber, text)",
        )
            .into_response();
    };

    tracing::info!(
        session_id = %cb.session_id,
        phone_number = %cb.phone_number,
        text = %cb.text,
        "USSD callback received"
    );

    let outcome = walk(
        &state.session_store,
        &WalkRequest {
            flow: &state.flow,
            text: &cb.text,
            phone: &cb.phone_number,
            session_id: &cb.session_id,
            service_code: &cb.service_code,
        },
    )
    .await;

    tracing::info!(
        session_id = %cb.session_id,
        node_id = %outcome.node_id,
        ended = outcome.ended,
        "USSD callback processed"
    );

    // Persist the exchange when Postgres is available; never fail the request
    // on a logging hiccup.
    if let Some(pool) = &state.db {
        let variables_json = serde_json::to_string(&outcome.variables).unwrap_or_default();
        let result = sqlx::query(
            "INSERT INTO callback_logs \
             (session_id, service_code, phone_number, ussd_text, reply, flow_id, flow_version, variables) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&cb.session_id)
        .bind(&cb.service_code)
        .bind(&cb.phone_number)
        .bind(&cb.text)
        .bind(&outcome.body)
        .bind(&state.flow.id)
        .bind(state.flow.version as i64)
        .bind(variables_json)
        .execute(pool)
        .await;

        if let Err(err) = result {
            tracing::warn!("failed to persist callback log: {err}");
        }
    }

    outcome.body.into_response()
}
