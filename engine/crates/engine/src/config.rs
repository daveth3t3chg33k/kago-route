//! Runtime configuration, read from the environment (or `.env`).

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    /// Path to a menu-schema file (`.json` or YAML). When unset, the embedded
    /// demo flow (`farmer-order`) is used.
    pub flow_schema_path: Option<String>,
    pub security: Security,
}

/// Security-sensitive runtime settings.
#[derive(Debug, Clone)]
pub struct Security {
    /// Shared secrets required on USSD callbacks via the `X-KagoRoute-Key`
    /// header. One per tenant. Empty = unauthenticated (dev only; warned loudly
    /// at boot).
    pub callback_secrets: Vec<String>,
    /// Origins allowed by CORS (browser clients, e.g. the dashboard).
    pub cors_allowed_origins: Vec<String>,
    /// Maximum accepted request body size in bytes (guards the callback route).
    pub max_body_bytes: usize,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://kago:kago@localhost:5432/kagoroute".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            flow_schema_path: env::var("FLOW_SCHEMA_PATH").ok().filter(|s| !s.is_empty()),
            security: Security {
                callback_secrets: parse_callback_secrets(),
                cors_allowed_origins: parse_csv(
                    env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default().as_str(),
                    &["http://localhost:3000".to_string()],
                ),
                max_body_bytes: env::var("USSD_MAX_BODY_BYTES")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(4096),
            },
        }
    }
}

/// Parse a comma-separated env var into a list; fall back to `defaults` when
/// unset or empty.
fn parse_csv(raw: &str, defaults: &[String]) -> Vec<String> {
    let value = raw.trim();
    if value.is_empty() {
        return defaults.to_vec();
    }
    let items: Vec<String> = value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if items.is_empty() {
        defaults.to_vec()
    } else {
        items
    }
}

/// Callback secrets: `USSD_CALLBACK_SECRET` (single) or
/// `USSD_CALLBACK_SECRETS` (comma-separated, one per tenant).
fn parse_callback_secrets() -> Vec<String> {
    if let Some(single) = env::var("USSD_CALLBACK_SECRET").ok().filter(|s| !s.is_empty()) {
        return vec![single];
    }
    parse_csv(
        env::var("USSD_CALLBACK_SECRETS").unwrap_or_default().as_str(),
        &[],
    )
}
