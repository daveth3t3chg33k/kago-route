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
        }
    }
}
