//! Redis-backed session store. Key-value strings with TTL, mirroring the
//! in-memory store's semantics so the two are interchangeable.

use std::time::Duration;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct RedisStore {
    conn: ConnectionManager,
}

impl std::fmt::Debug for RedisStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisStore")
            .field("conn", &"<ConnectionManager>")
            .finish()
    }
}

impl RedisStore {
    pub async fn connect(url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    pub async fn get(&self, key: &str) -> redis::RedisResult<Option<String>> {
        let mut conn = self.conn.clone();
        conn.get(key).await
    }

    pub async fn set(&self, key: &str, value: &str, ttl: Duration) -> redis::RedisResult<()> {
        let mut conn = self.conn.clone();
        conn.set_ex(key, value, ttl.as_secs()).await
    }

    pub async fn delete(&self, key: &str) -> redis::RedisResult<()> {
        let mut conn = self.conn.clone();
        conn.del(key).await
    }
}
