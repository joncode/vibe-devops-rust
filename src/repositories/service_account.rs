//! Service account repository
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{HexId, ServiceAccount};

pub struct ServiceAccountRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ServiceAccountRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self, user_id: Uuid, name: &str, description: Option<&str>,
        api_key_hash: &str, api_key_prefix: &str, permissions: &serde_json::Value,
        rate_limit_per_minute: i32, expires_at: Option<DateTime<Utc>>,
    ) -> Result<ServiceAccount> {
        let hex_id = ServiceAccount::generate_hex_id();
        sqlx::query_as::<_, ServiceAccount>(
            r#"INSERT INTO app_service_accounts (hex_id, user_id, name, description, api_key_hash, api_key_prefix, permissions, rate_limit_per_minute, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"#)
            .bind(&hex_id).bind(user_id).bind(name).bind(description)
            .bind(api_key_hash).bind(api_key_prefix).bind(permissions)
            .bind(rate_limit_per_minute).bind(expires_at)
            .fetch_one(self.pool).await.map_err(Into::into)
    }

    pub async fn find_by_hex_id(&self, hex_id: &str) -> Result<Option<ServiceAccount>> {
        sqlx::query_as::<_, ServiceAccount>(
            "SELECT * FROM app_service_accounts WHERE hex_id = $1 AND deleted_at IS NULL")
            .bind(hex_id).fetch_optional(self.pool).await.map_err(Into::into)
    }

    pub async fn find_by_api_key_prefix(&self, prefix: &str) -> Result<Vec<ServiceAccount>> {
        sqlx::query_as::<_, ServiceAccount>(
            "SELECT * FROM app_service_accounts WHERE api_key_prefix = $1 AND deleted_at IS NULL AND is_active = true")
            .bind(prefix).fetch_all(self.pool).await.map_err(Into::into)
    }

    pub async fn list_by_user(&self, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<ServiceAccount>> {
        sqlx::query_as::<_, ServiceAccount>(
            "SELECT * FROM app_service_accounts WHERE user_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind(user_id).bind(limit).bind(offset).fetch_all(self.pool).await.map_err(Into::into)
    }

    pub async fn update(
        &self, id: Uuid, name: Option<&str>, description: Option<&str>,
        permissions: Option<&serde_json::Value>, rate_limit_per_minute: Option<i32>,
        expires_at: Option<DateTime<Utc>>, is_active: Option<bool>,
    ) -> Result<Option<ServiceAccount>> {
        sqlx::query_as::<_, ServiceAccount>(
            r#"UPDATE app_service_accounts SET name = COALESCE($2, name), description = COALESCE($3, description),
            permissions = COALESCE($4, permissions), rate_limit_per_minute = COALESCE($5, rate_limit_per_minute),
            expires_at = COALESCE($6, expires_at), is_active = COALESCE($7, is_active), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL RETURNING *"#)
            .bind(id).bind(name).bind(description).bind(permissions)
            .bind(rate_limit_per_minute).bind(expires_at).bind(is_active)
            .fetch_optional(self.pool).await.map_err(Into::into)
    }

    pub async fn update_last_used(&self, id: Uuid) -> Result<bool> {
        let r = sqlx::query("UPDATE app_service_accounts SET last_used_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id).execute(self.pool).await?;
        Ok(r.rows_affected() > 0)
    }

    pub async fn rotate_api_key(&self, id: Uuid, api_key_hash: &str, api_key_prefix: &str) -> Result<Option<ServiceAccount>> {
        sqlx::query_as::<_, ServiceAccount>(
            "UPDATE app_service_accounts SET api_key_hash = $2, api_key_prefix = $3, updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL RETURNING *")
            .bind(id).bind(api_key_hash).bind(api_key_prefix).fetch_optional(self.pool).await.map_err(Into::into)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let r = sqlx::query("UPDATE app_service_accounts SET deleted_at = NOW(), is_active = false WHERE id = $1 AND deleted_at IS NULL")
            .bind(id).execute(self.pool).await?;
        Ok(r.rows_affected() > 0)
    }
}
