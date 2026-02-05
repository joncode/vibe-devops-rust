//! Redis module - Connection pool and key management

use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use tracing::info;

use crate::config::RedisConfig;

/// Redis connection pool wrapper
#[derive(Clone)]
pub struct RedisPool {
    conn: ConnectionManager,
    prefix: String,
}

impl RedisPool {
    /// Create a new Redis connection pool
    pub async fn connect(config: &RedisConfig) -> Result<Self> {
        info!("Connecting to Redis...");
        
        let client = Client::open(config.url.as_str())?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self {
            conn,
            prefix: config.key_prefix.clone(),
        })
    }

    /// Get a clone of the connection manager
    pub fn connection(&self) -> ConnectionManager {
        self.conn.clone()
    }

    /// Get the key prefix
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Build a namespaced key
    pub fn key(&self, parts: &[&str]) -> String {
        format!("{}:{}", self.prefix, parts.join(":"))
    }

    /// Check Redis health
    pub async fn health_check(&self) -> Result<bool> {
        let mut conn = self.conn.clone();
        let result: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await?;
        
        Ok(result == "PONG")
    }

    /// Set a key with expiration
    pub async fn set_ex(&self, key: &str, value: &str, seconds: u64) -> Result<()> {
        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(key, value, seconds).await?;
        Ok(())
    }

    /// Get a value
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.conn.clone();
        let result: Option<String> = conn.get(key).await?;
        Ok(result)
    }

    /// Delete a key
    pub async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        conn.del::<_, ()>(key).await?;
        Ok(())
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.conn.clone();
        let result: bool = conn.exists(key).await?;
        Ok(result)
    }
}

/// Redis key namespace helpers
pub struct RedisKeys<'a> {
    pool: &'a RedisPool,
}

impl<'a> RedisKeys<'a> {
    pub fn new(pool: &'a RedisPool) -> Self {
        Self { pool }
    }

    /// Session key: {prefix}:session:user:{user_id}:{session_id}
    pub fn user_session(&self, user_id: &uuid::Uuid, session_id: &uuid::Uuid) -> String {
        self.pool.key(&["session", "user", &user_id.to_string(), &session_id.to_string()])
    }

    /// Rate limit key: {prefix}:rate:ip:{ip}:{endpoint}
    pub fn rate_limit_ip(&self, ip: &str, endpoint: &str) -> String {
        self.pool.key(&["rate", "ip", ip, endpoint])
    }

    /// Nonce key: {prefix}:nonce:user:{user_id}:{nonce}
    pub fn nonce(&self, user_id: &uuid::Uuid, nonce: &str) -> String {
        self.pool.key(&["nonce", "user", &user_id.to_string(), nonce])
    }

    /// Refresh token key: {prefix}:refresh:{token_hash}
    pub fn refresh_token(&self, token_hash: &str) -> String {
        self.pool.key(&["refresh", token_hash])
    }
}
