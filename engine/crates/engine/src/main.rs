//! KagoRoute engine — the USSD/SMS integration layer.
//!
//! Boots an Axum server that:
//!  - loads and validates a menu-schema flow at boot (fail closed),
//!  - answers carrier/aggregator USSD callbacks by walking the schema and
//!    replying with `CON` / `END` text,
//!  - keeps session variables and loop-guard state in Redis (with an
//!    in-memory fallback for local dev),
//!  - persists callback logs to PostgreSQL when available.

mod config;
mod routes;
mod schema;
mod session;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::schema::load_flow;
use crate::session::{memory::MemoryStore, redis::RedisStore, SessionStore};
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::from_env();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug")))
        .init();

    // ── Schema: parse + validate fail-closed at boot ─────────────────────
    let flow = load_flow(config.flow_schema_path.as_deref()).unwrap_or_else(|err| {
        eprintln!("FATAL: {err}");
        std::process::exit(1);
    });
    tracing::info!(
        flow_id = %flow.id,
        flow_version = flow.version,
        start = %flow.start,
        "loaded flow schema"
    );

    // ── Session store: prefer Redis, fall back to in-memory ──────────────
    let session_store: Arc<SessionStore> = match RedisStore::connect(&config.redis_url).await {
        Ok(redis) => {
            tracing::info!("session store: Redis ({})", config.redis_url);
            Arc::new(SessionStore::Redis(redis))
        }
        Err(err) => {
            tracing::warn!("Redis unavailable ({err}); using in-memory session store");
            Arc::new(SessionStore::Memory(MemoryStore::default()))
        }
    };

    // ── Database: optional persistence, degrade gracefully ───────────────
    let db = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("connected to PostgreSQL");
            match sqlx::migrate!("./migrations").run(&pool).await {
                Ok(_) => Some(pool),
                Err(err) => {
                    // Don't keep a pool whose schema is unusable — that would
                    // spam failed INSERTs on every callback.
                    tracing::error!("migration failed: {err}; disabling persistence");
                    None
                }
            }
        }
        Err(err) => {
            tracing::warn!("PostgreSQL unavailable ({err}); running without persistence");
            None
        }
    };

    // ── HTTP server ──────────────────────────────────────────────────────
    if !config.security.callback_secrets.is_empty() {
        tracing::info!(
            secrets = config.security.callback_secrets.len(),
            "USSD callback auth: required (X-KagoRoute-Key header)"
        );
    } else {
        tracing::warn!(
            "USSD_CALLBACK_SECRET(S) not set — /ussd/callback is UNAUTHENTICATED (dev only)"
        );
    }

    let state = Arc::new(AppState {
        flow,
        session_store,
        db,
        started_at: Instant::now(),
        security: config.security,
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("KagoRoute engine listening on http://{addr}");

    axum::serve(listener, routes::build_router(state)).await?;
    Ok(())
}
