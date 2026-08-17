//! In-process session store with TTL expiry — used when Redis is unreachable.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct MemoryStore {
    inner: RwLock<HashMap<String, (String, Instant)>>,
}

impl MemoryStore {
    pub async fn get(&self, key: &str) -> Option<String> {
        let mut guard = self.inner.write().await;
        if let Some((value, expires_at)) = guard.get(key) {
            if *expires_at > Instant::now() {
                return Some(value.clone());
            }
            // Expired — lazily remove.
            guard.remove(key);
        }
        None
    }

    pub async fn set(&self, key: &str, value: &str, ttl: Duration) {
        let mut guard = self.inner.write().await;
        guard.insert(key.to_string(), (value.to_string(), Instant::now() + ttl));
    }

    pub async fn delete(&self, key: &str) {
        self.inner.write().await.remove(key);
    }

    /// All non-expired variable caches, as `(key, value)` pairs.
    pub async fn active_sessions(&self) -> Vec<(String, String)> {
        let guard = self.inner.read().await;
        let now = Instant::now();
        let mut items: Vec<(String, String)> = guard
            .iter()
            .filter(|(k, (_, expires_at))| k.ends_with(":vars") && **expires_at > now)
            .map(|(k, (v, _))| (k.clone(), v.clone()))
            .collect();
        // Newest-first (insertion order is not reliable; sort by nothing we
        // track beyond expiry — keep it simple, order is informational).
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items
    }
}
