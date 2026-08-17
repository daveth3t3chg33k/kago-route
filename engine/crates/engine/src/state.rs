//! Shared application state handed to every handler.

use std::sync::Arc;
use std::time::Instant;

use sqlx::PgPool;

use crate::session::SessionStore;

pub struct AppState {
    pub session_store: Arc<SessionStore>,
    pub db: Option<PgPool>,
    pub started_at: Instant,
}
