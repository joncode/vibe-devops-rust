//! Service Account repository

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{HexId, ServiceAccount};

/// Service Account repository
pub struct ServiceAccountRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ServiceAccountRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new service account
    pub async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        description: Option<&str>,
        api_key_hash: &str,
        api_key_prefix: &str,
        permissions: serde_json::Value,
        rate_limit_per_minute: i32,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ServiceAccount> {
        let hex_id = ServiceAccount::generate_hex_id();

        let service_account = sqlx::query_as::<_, ServiceAccount>(
            r#"
            INSERT INTO app_service_accounts (
                hex_id, user_id, name, description, api_key_hash, api_key_prefix,
                permissions, rate_limit_per_minute, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(&hex_id)
        .bind(user_id)
        .bind(name)
        .bind(description)
        .bind(api_key_hash)
        .bind(api_key_prefix)
        .bind(&permissions)
        .bind(rate_limit_per_minute)
        .bind(expires_at)
        .fetch_one(self.pool)
        .await?;

        Ok(service_account)
    }

    /// Find service account by hex_id (public ID)
    pub async fn find_by_hex_id(&self, hex_id: &str) -> Result<Option<ServiceAccount>> {
        let service_account = sqlx::query_as::<_, ServiceAccount>(
            r#"
            SELECT * FROM app_service_accounts
            WHERE hex_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(hex_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(service_account)
    }

    /// Find service account by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ServiceAccount>> {
        let service_account = sqlx::query_as::<_, ServiceAccount>(
            r#"
            SELECT * FROM app_service_accounts
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        Ok(service_account)
    }

    /// Find service account by API key prefix
    /// Used for the first step of API key validation
    pub async fn find_by_api_key_prefix(&self, prefix: &str) -> Result<Option<ServiceAccount>> {
        let service_account = sqlx::query_as::<_, ServiceAccount>(
            r#"
            SELECT * FROM app_service_accounts
            WHERE api_key_prefix = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(prefix)
        .fetch_optional(self.pool)
        .await?;

        Ok(service_account)
    }

    /// Find all service accounts by API key prefix (in case of collision)
    pub async fn find_all_by_api_key_prefix(&self, prefix: &str) -> Result<Vec<ServiceAccount>> {
        let service_accounts = sqlx::query_as::<_, ServiceAccount>(
            r#"
            SELECT * FROM app_service_accounts
            WHERE api_key_prefix = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(prefix)
        .fetch_all(self.pool)
        .await?;

        Ok(service_accounts)
    }

    /// List service accounts by user
    pub async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ServiceAccount>> {
        let service_accounts = sqlx::query_as::<_, ServiceAccount>(
            r#"
            SELECT * FROM app_service_accounts
            WHERE user_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?;

        Ok(service_accounts)
    }

    /// Update service account
    pub async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        permissions: Option<serde_json::Value>,
        rate_limit_per_minute: Option<i32>,
        expires_at: Option<DateTime<Utc>>,
        is_active: Option<bool>,
    ) -> Result<Option<ServiceAccount>> {
        let service_account = sqlx::query_as::<_, ServiceAccount>(
            r#"
            UPDATE app_service_accounts
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                permissions = COALESCE($4, permissions),
                rate_limit_per_minute = COALESCE($5, rate_limit_per_minute),
                expires_at = COALESCE($6, expires_at),
                is_active = COALESCE($7, is_active),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(permissions)
        .bind(rate_limit_per_minute)
        .bind(expires_at)
        .bind(is_active)
        .fetch_optional(self.pool)
        .await?;

        Ok(service_account)
    }

    /// Update last used timestamp
    pub async fn update_last_used(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE app_service_accounts
            SET last_used_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Deactivate a service account
    pub async fn deactivate(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE app_service_accounts
            SET is_active = false, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Soft delete service account
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE app_service_accounts
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List all service accounts with pagination
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<ServiceAccount>> {
        let service_accounts = sqlx::query_as::<_, ServiceAccount>(
            r#"
            SELECT * FROM app_service_accounts
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(service_accounts)
    }

    /// Count total service accounts
    pub async fn count(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM app_service_accounts
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(self.pool)
        .await?;

        Ok(count.0)
    }

    /// Count active service accounts for a user
    pub async fn count_by_user(&self, user_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM app_service_accounts
            WHERE user_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(user_id)
        .fetch_one(self.pool)
        .await?;

        Ok(count.0)
    }

    /// Rotate API key (update hash and prefix)
    pub async fn rotate_api_key(
        &self,
        id: Uuid,
        new_api_key_hash: &str,
        new_api_key_prefix: &str,
    ) -> Result<Option<ServiceAccount>> {
        let service_account = sqlx::query_as::<_, ServiceAccount>(
            r#"
            UPDATE app_service_accounts
            SET api_key_hash = $2,
                api_key_prefix = $3,
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(new_api_key_hash)
        .bind(new_api_key_prefix)
        .fetch_optional(self.pool)
        .await?;

        Ok(service_account)
    }
}
