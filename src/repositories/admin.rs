//! Admin repository - paginated access to all tables

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{User, SessionToken, RefreshToken, SocialIdentifier};

/// User password model for admin (without exposing actual hash in responses)
#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct UserPasswordEntry {
    pub user_id: Uuid,
    pub hex_id: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Admin repository for paginated queries
pub struct AdminRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> AdminRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Table Counts
    // ========================================================================

    pub async fn count_users(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM app_users")
            .fetch_one(self.pool)
            .await?;
        Ok(count.0)
    }

    pub async fn count_social_identifiers(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM app_social_identifiers")
            .fetch_one(self.pool)
            .await?;
        Ok(count.0)
    }

    pub async fn count_user_passwords(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM app_user_passwords")
            .fetch_one(self.pool)
            .await?;
        Ok(count.0)
    }

    pub async fn count_session_tokens(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM app_session_tokens")
            .fetch_one(self.pool)
            .await?;
        Ok(count.0)
    }

    pub async fn count_refresh_tokens(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM app_refresh_tokens")
            .fetch_one(self.pool)
            .await?;
        Ok(count.0)
    }

    // ========================================================================
    // Paginated Queries
    // ========================================================================

    pub async fn list_users(&self, page: u32, per_page: u32) -> Result<Vec<User>> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM app_users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(users)
    }

    pub async fn list_social_identifiers(&self, page: u32, per_page: u32) -> Result<Vec<SocialIdentifier>> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let identifiers = sqlx::query_as::<_, SocialIdentifier>(
            r#"
            SELECT * FROM app_social_identifiers
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(identifiers)
    }

    pub async fn list_user_passwords(&self, page: u32, per_page: u32) -> Result<Vec<UserPasswordEntry>> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let passwords = sqlx::query_as::<_, UserPasswordEntry>(
            r#"
            SELECT * FROM app_user_passwords
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(passwords)
    }

    pub async fn list_session_tokens(&self, page: u32, per_page: u32) -> Result<Vec<SessionToken>> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let sessions = sqlx::query_as::<_, SessionToken>(
            r#"
            SELECT * FROM app_session_tokens
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(sessions)
    }

    pub async fn list_refresh_tokens(&self, page: u32, per_page: u32) -> Result<Vec<RefreshToken>> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let tokens = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT * FROM app_refresh_tokens
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(tokens)
    }
}
