//! Social identifier repository

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{IdentifierType, SocialIdentifier};

/// Identifier repository
pub struct IdentifierRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> IdentifierRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new identifier
    pub async fn create(
        &self,
        user_id: Uuid,
        identifier_type: IdentifierType,
        identifier_value: &str,
        is_primary: bool,
    ) -> Result<SocialIdentifier> {
        let identifier = sqlx::query_as::<_, SocialIdentifier>(
            r#"
            INSERT INTO app_social_identifiers (user_id, identifier_type, identifier_value, is_primary)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(&identifier_type)
        .bind(identifier_value.to_lowercase())
        .bind(is_primary)
        .fetch_one(self.pool)
        .await?;

        Ok(identifier)
    }

    /// Find by type and value (for login)
    pub async fn find_by_value(
        &self,
        identifier_type: IdentifierType,
        identifier_value: &str,
    ) -> Result<Option<SocialIdentifier>> {
        let identifier = sqlx::query_as::<_, SocialIdentifier>(
            r#"
            SELECT * FROM app_social_identifiers
            WHERE identifier_type = $1 AND identifier_value = $2
            "#
        )
        .bind(&identifier_type)
        .bind(identifier_value.to_lowercase())
        .fetch_optional(self.pool)
        .await?;

        Ok(identifier)
    }

    /// Find all identifiers for a user
    pub async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<SocialIdentifier>> {
        let identifiers = sqlx::query_as::<_, SocialIdentifier>(
            r#"
            SELECT * FROM app_social_identifiers
            WHERE user_id = $1
            ORDER BY is_primary DESC, created_at ASC
            "#
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?;

        Ok(identifiers)
    }

    /// Mark identifier as verified
    pub async fn verify(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE app_social_identifiers
            SET verified = TRUE, verified_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Set an identifier as primary
    pub async fn set_primary(&self, id: Uuid, user_id: Uuid) -> Result<bool> {
        // First, unset primary for all user's identifiers of the same type
        let identifier = sqlx::query_as::<_, SocialIdentifier>(
            r#"SELECT * FROM app_social_identifiers WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        if let Some(identifier) = identifier {
            // Unset primary for other identifiers of the same type
            sqlx::query(
                r#"
                UPDATE app_social_identifiers
                SET is_primary = FALSE, updated_at = NOW()
                WHERE user_id = $1 AND identifier_type = $2 AND id != $3
                "#
            )
            .bind(user_id)
            .bind(&identifier.identifier_type)
            .bind(id)
            .execute(self.pool)
            .await?;

            // Set primary for this identifier
            let result = sqlx::query(
                r#"
                UPDATE app_social_identifiers
                SET is_primary = TRUE, updated_at = NOW()
                WHERE id = $1 AND user_id = $2
                "#
            )
            .bind(id)
            .bind(user_id)
            .execute(self.pool)
            .await?;

            Ok(result.rows_affected() > 0)
        } else {
            Ok(false)
        }
    }

    /// Delete an identifier
    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM app_social_identifiers
            WHERE id = $1 AND user_id = $2
            "#
        )
        .bind(id)
        .bind(user_id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if identifier exists
    pub async fn exists(&self, identifier_type: IdentifierType, identifier_value: &str) -> Result<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM app_social_identifiers
                WHERE identifier_type = $1 AND identifier_value = $2
            )
            "#
        )
        .bind(&identifier_type)
        .bind(identifier_value.to_lowercase())
        .fetch_one(self.pool)
        .await?;

        Ok(exists.0)
    }
}
