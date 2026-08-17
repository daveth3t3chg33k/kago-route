//! Inbound USSD callback handler.
//!
//! Accepts the fields Africa's Talking (and most aggregators) POST on every
//! session step — `sessionId`, `serviceCode`, `phoneNumber`, `text` — either
//! as `application/x-www-form-urlencoded` or `application/json`, walks the
//! menu, and replies with a `CON` / `END` string exactly like a carrier expects.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use serde::Deserialize;

use crate::menu::run_demo_menu;
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

    let reply = run_demo_menu(&state.session_store, &cb.session_id, &cb.phone_number, &cb.text).await;

    // Persist the exchange when Postgres is available; never fail the request
    // on a logging hiccup.
    if let Some(pool) = &state.db {
        let result = sqlx::query(
            "INSERT INTO callback_logs (session_id, service_code, phone_number, ussd_text, reply) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&cb.session_id)
        .bind(&cb.service_code)
        .bind(&cb.phone_number)
        .bind(&cb.text)
        .bind(&reply.text)
        .execute(pool)
        .await;

        if let Err(err) = result {
            tracing::warn!("failed to persist callback log: {err}");
        }
    }

    reply.to_body().into_response()
}
