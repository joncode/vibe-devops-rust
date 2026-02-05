//! Session repository

use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::SessionToken;

/// Session repository
pub struct SessionRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> SessionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new session
    pub async fn create(
        &self,
        user_id: Uuid,
        token_hash: &str,
        device_info: serde_json::Value,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        expires_in_days: i64,
    ) -> Result<SessionToken> {
        let expires_at = Utc::now() + Duration::days(expires_in_days);

        let session = sqlx::query_as::<_, SessionToken>(
            r#"
            INSERT INTO app_session_tokens (user_id, token_hash, device_info, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(&device_info)
        .bind(ip_address)
        .bind(user_agent)
        .bind(expires_at)
        .fetch_one(self.pool)
        .await?;

        Ok(session)
    }

    /// Find session by token hash
    pub async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<SessionToken>> {
        let session = sqlx::query_as::<_, SessionToken>(
            r#"
            SELECT * FROM app_session_tokens
            WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > NOW()
            "#
        )
        .bind(token_hash)
        .fetch_optional(self.pool)
        .await?;

        Ok(session)
    }

    /// Find all active sessions for a user
    pub async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<SessionToken>> {
        let sessions = sqlx::query_as::<_, SessionToken>(
            r#"
            SELECT * FROM app_session_tokens
            WHERE user_id = $1 AND revoked_at IS NULL AND expires_at > NOW()
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?;

        Ok(sessions)
    }

    /// Update last used timestamp
    pub async fn touch(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE app_session_tokens
            SET last_used_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Revoke a session
    pub async fn revoke(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE app_session_tokens
            SET revoked_at = NOW()
            WHERE id = $1 AND revoked_at IS NULL
            "#
        )
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Revoke all sessions for a user
    pub async fn revoke_all(&self, user_id: Uuid) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE app_session_tokens
            SET revoked_at = NOW()
            WHERE user_id = $1 AND revoked_at IS NULL
            "#
        )
        .bind(user_id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Revoke all sessions except the current one
    pub async fn revoke_all_except(&self, user_id: Uuid, current_session_id: Uuid) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE app_session_tokens
            SET revoked_at = NOW()
            WHERE user_id = $1 AND id != $2 AND revoked_at IS NULL
            "#
        )
        .bind(user_id)
        .bind(current_session_id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Count active sessions for a user
    pub async fn count_active(&self, user_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM app_session_tokens
            WHERE user_id = $1 AND revoked_at IS NULL AND expires_at > NOW()
            "#
        )
        .bind(user_id)
        .fetch_one(self.pool)
        .await?;

        Ok(count.0)
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM app_session_tokens
            WHERE expires_at < NOW() - INTERVAL '7 days'
            "#
        )
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
