//! Authentication service

mod jwt;
mod password;

pub use jwt::*;
pub use password::*;

use anyhow::Result;
use uuid::Uuid;

use crate::config::AuthConfig;
use crate::db::Database;
use crate::errors::{AppError, AppResult};
use crate::models::{IdentifierType, User, UserStatus};
use crate::redis::RedisPool;
use crate::repositories::{IdentifierRepository, SessionRepository, UserRepository};

/// Authentication service
pub struct AuthService<'a> {
    db: &'a Database,
    redis: &'a RedisPool,
    config: &'a AuthConfig,
}

impl<'a> AuthService<'a> {
    pub fn new(db: &'a Database, redis: &'a RedisPool, config: &'a AuthConfig) -> Self {
        Self { db, redis, config }
    }

    /// Register a new user with email and password
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: Option<&str>,
    ) -> AppResult<User> {
        // Validate email
        if !crate::utils::is_valid_email(email) {
            return Err(AppError::Validation("Invalid email address".to_string()));
        }

        // Validate password
        crate::utils::is_valid_password(password)
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let pool = self.db.pool();

        // Check if email already exists
        let identifier_repo = IdentifierRepository::new(pool);
        if identifier_repo.exists(IdentifierType::Email, email).await? {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        // Create user
        let user_repo = UserRepository::new(pool);
        let user = user_repo.create(display_name).await?;

        // Create email identifier
        identifier_repo
            .create(user.id, IdentifierType::Email, email, true)
            .await?;

        // Store password hash
        let password_hash = hash_password(password)?;
        self.store_password(user.id, &password_hash).await?;

        // TODO: Send verification email

        Ok(user)
    }

    /// Login with email and password
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        device_info: serde_json::Value,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> AppResult<AuthTokens> {
        let pool = self.db.pool();

        // Find identifier
        let identifier_repo = IdentifierRepository::new(pool);
        let identifier = identifier_repo
            .find_by_value(IdentifierType::Email, email)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        // Get user
        let user_repo = UserRepository::new(pool);
        let user = user_repo
            .find_by_id(identifier.user_id)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        // Check user status
        if user.status == UserStatus::Suspended {
            return Err(AppError::Forbidden);
        }

        // Verify password
        let stored_hash = self.get_password(user.id).await?
            .ok_or(AppError::InvalidCredentials)?;

        if !verify_password(password, &stored_hash)? {
            return Err(AppError::InvalidCredentials);
        }

        // Create session
        let session_repo = SessionRepository::new(pool);
        let session_token = crate::utils::generate_token(64);
        let token_hash = crate::utils::sha256_hash(&session_token);

        let session = session_repo
            .create(
                user.id,
                &token_hash,
                device_info,
                ip_address,
                user_agent,
                self.config.session_expiry_days,
            )
            .await?;

        // Generate JWT
        let access_token = create_access_token(
            user.id,
            session.id,
            &self.config.jwt_secret,
            self.config.jwt_expiry_hours,
        )?;

        // Generate refresh token
        let refresh_token = crate::utils::generate_token(64);
        let refresh_token_hash = crate::utils::sha256_hash(&refresh_token);
        
        // Store refresh token in Redis
        let refresh_key = format!("{}:refresh:{}", self.redis.prefix(), refresh_token_hash);
        let refresh_data = serde_json::json!({
            "user_id": user.id,
            "session_id": session.id,
        });
        self.redis
            .set_ex(
                &refresh_key,
                &refresh_data.to_string(),
                (self.config.refresh_token_expiry_days * 24 * 60 * 60) as u64,
            )
            .await?;

        // Update user status if pending
        if user.status == UserStatus::Pending {
            user_repo.update_status(user.id, UserStatus::Active).await?;
        }

        Ok(AuthTokens {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt_expiry_hours * 3600,
            session_token,
        })
    }

    /// Refresh access token
    pub async fn refresh_tokens(&self, refresh_token: &str) -> AppResult<AuthTokens> {
        let refresh_token_hash = crate::utils::sha256_hash(refresh_token);
        let refresh_key = format!("{}:refresh:{}", self.redis.prefix(), refresh_token_hash);

        // Get refresh token data from Redis
        let data = self.redis.get(&refresh_key).await?
            .ok_or(AppError::TokenExpired)?;

        let data: serde_json::Value = serde_json::from_str(&data)
            .map_err(|_| AppError::TokenExpired)?;

        let user_id: Uuid = data["user_id"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or(AppError::TokenExpired)?;

        let session_id: Uuid = data["session_id"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or(AppError::TokenExpired)?;

        // Verify session is still valid
        let pool = self.db.pool();
        let session_repo = SessionRepository::new(pool);
        let session = session_repo
            .find_by_user(user_id)
            .await?
            .into_iter()
            .find(|s| s.id == session_id)
            .ok_or(AppError::TokenExpired)?;

        if !session.is_active() {
            return Err(AppError::TokenExpired);
        }

        // Delete old refresh token
        self.redis.del(&refresh_key).await?;

        // Generate new tokens
        let access_token = create_access_token(
            user_id,
            session_id,
            &self.config.jwt_secret,
            self.config.jwt_expiry_hours,
        )?;

        let new_refresh_token = crate::utils::generate_token(64);
        let new_refresh_token_hash = crate::utils::sha256_hash(&new_refresh_token);
        
        // Store new refresh token
        let new_refresh_key = format!("{}:refresh:{}", self.redis.prefix(), new_refresh_token_hash);
        let refresh_data = serde_json::json!({
            "user_id": user_id,
            "session_id": session_id,
        });
        self.redis
            .set_ex(
                &new_refresh_key,
                &refresh_data.to_string(),
                (self.config.refresh_token_expiry_days * 24 * 60 * 60) as u64,
            )
            .await?;

        Ok(AuthTokens {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt_expiry_hours * 3600,
            session_token: String::new(), // Not returned on refresh
        })
    }

    /// Logout (revoke session)
    pub async fn logout(&self, session_id: Uuid) -> AppResult<()> {
        let pool = self.db.pool();
        let session_repo = SessionRepository::new(pool);
        session_repo.revoke(session_id).await?;
        Ok(())
    }

    /// Logout all sessions
    pub async fn logout_all(&self, user_id: Uuid) -> AppResult<u64> {
        let pool = self.db.pool();
        let session_repo = SessionRepository::new(pool);
        let count = session_repo.revoke_all(user_id).await?;
        Ok(count)
    }

    // Helper: Store password hash
    async fn store_password(&self, user_id: Uuid, password_hash: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO app_user_passwords (user_id, password_hash)
            VALUES ($1, $2)
            ON CONFLICT (user_id) DO UPDATE SET password_hash = $2, updated_at = NOW()
            "#
        )
        .bind(user_id)
        .bind(password_hash)
        .execute(self.db.pool())
        .await?;
        Ok(())
    }

    // Helper: Get password hash
    async fn get_password(&self, user_id: Uuid) -> Result<Option<String>> {
        let result: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT password_hash FROM app_user_passwords
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(result.map(|r| r.0))
    }
}

/// Authentication tokens returned after login
#[derive(Debug, serde::Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub session_token: String,
}
