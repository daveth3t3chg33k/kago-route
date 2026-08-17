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
        guard
            .iter()
            .filter(|(k, (_, expires_at))| k.ends_with(":vars") && *expires_at > now)
            .map(|(k, (v, _))| (k.clone(), v.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lists_only_live_vars_keys() {
        let store = MemoryStore::default();
        store.set("session:a:vars", r#"{"x":1}"#, Duration::from_secs(60)).await;
        store.set("session:a:guard", "welcome|2", Duration::from_secs(60)).await;
        store.set("session:b:vars", r#"{"y":2}"#, Duration::from_secs(60)).await;
        store.set("expired:vars", "{}", Duration::from_millis(1)).await;

        // Let the expired key lapse.
        tokio::time::sleep(Duration::from_millis(10)).await;

        let active = store.active_sessions().await;
        assert_eq!(active.len(), 2);
        assert!(active.iter().any(|(k, _)| k == "session:a:vars"));
        assert!(active.iter().any(|(k, _)| k == "session:b:vars"));
        assert!(!active.iter().any(|(k, _)| k == "expired:vars"));
        // Guard keys are excluded.
        assert!(!active.iter().any(|(k, _)| k == "session:a:guard"));
    }

    #[tokio::test]
    async fn expired_keys_are_dropped_from_get() {
        let store = MemoryStore::default();
        store.set("session:c:vars", "{}", Duration::from_millis(1)).await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(store.get("session:c:vars").await.is_none());
    }
}
