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
}
