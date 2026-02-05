//! User repository

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{User, UserStatus};

/// User repository
pub struct UserRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new user
    pub async fn create(&self, display_name: Option<&str>) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO app_users (display_name, status)
            VALUES ($1, $2)
            RETURNING *
            "#
        )
        .bind(display_name)
        .bind(UserStatus::Pending)
        .fetch_one(self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM app_users
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        Ok(user)
    }

    /// Update user
    pub async fn update(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE app_users
            SET display_name = COALESCE($2, display_name),
                avatar_url = COALESCE($3, avatar_url),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#
        )
        .bind(id)
        .bind(display_name)
        .bind(avatar_url)
        .fetch_optional(self.pool)
        .await?;

        Ok(user)
    }

    /// Update user status
    pub async fn update_status(&self, id: Uuid, status: UserStatus) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE app_users
            SET status = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(id)
        .bind(status)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Soft delete user
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE app_users
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List users with pagination
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM app_users
            WHERE deleted_at IS NULL
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

    /// Count total users
    pub async fn count(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM app_users
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_one(self.pool)
        .await?;

        Ok(count.0)
    }
}
