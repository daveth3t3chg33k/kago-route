//! Shared application state handed to every handler.

use std::sync::Arc;
use std::time::Instant;

use sqlx::PgPool;

use crate::config::Security;
use crate::schema::Flow;
use crate::session::SessionStore;

pub struct AppState {
    pub flow: Arc<Flow>,
    pub session_store: Arc<SessionStore>,
    pub db: Option<PgPool>,
    pub started_at: Instant,
    pub security: Security,
}
