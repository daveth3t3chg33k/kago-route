//! Ephemeral session storage mapping `sessionId` -> USSD state.
//!
//! A `SessionStore` is either backed by Redis (production) or an in-process
//! map (local dev with no infrastructure). The enum keeps dispatch explicit
//! and dependency-free; the Redis variant is preferred at boot.

pub mod memory;
pub mod redis;

use std::time::Duration;

use memory::MemoryStore;
use redis::RedisStore;

#[derive(Debug)]
pub enum SessionStore {
    Memory(MemoryStore),
    Redis(RedisStore),
}

impl SessionStore {
    pub async fn get(&self, key: &str) -> Option<String> {
        match self {
            Self::Memory(store) => store.get(key).await,
            Self::Redis(store) => match store.get(key).await {
                Ok(value) => value,
                Err(err) => {
                    tracing::warn!("redis get failed for {key}: {err}");
                    None
                }
            },
        }
    }

    pub async fn set(&self, key: &str, value: &str, ttl: Duration) {
        match self {
            Self::Memory(store) => store.set(key, value, ttl).await,
            Self::Redis(store) => {
                if let Err(err) = store.set(key, value, ttl).await {
                    tracing::warn!("redis set failed for {key}: {err}");
                }
            }
        }
    }

    pub async fn delete(&self, key: &str) {
        match self {
            Self::Memory(store) => store.delete(key).await,
            Self::Redis(store) => {
                if let Err(err) = store.delete(key).await {
                    tracing::warn!("redis delete failed for {key}: {err}");
                }
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Memory(_) => "memory",
            Self::Redis(_) => "redis",
        }
    }

    /// Live sessions: unique `sessionId`s that have a cached variable set.
    /// Returns `(session_id, vars_json)` pairs, newest-first where known.
    pub async fn active_sessions(&self) -> Vec<(String, String)> {
        match self {
            Self::Memory(store) => store.active_sessions().await,
            Self::Redis(store) => match store.active_sessions().await {
                Ok(items) => items,
                Err(err) => {
                    tracing::warn!("redis active_sessions failed: {err}");
                    Vec::new()
                }
            },
        }
    }
}
