//! JWT token handling

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Session ID
    pub sid: String,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration (Unix timestamp)
    pub exp: i64,
    /// JWT ID
    pub jti: String,
}

/// Create an access token
pub fn create_access_token(
    user_id: Uuid,
    session_id: Uuid,
    secret: &str,
    expiry_hours: i64,
) -> AppResult<String> {
    let now = Utc::now();
    let exp = now + Duration::hours(expiry_hours);

    let claims = Claims {
        sub: user_id.to_string(),
        sid: session_id.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create JWT: {}", e)))?;

    Ok(token)
}

/// Verify and decode an access token
pub fn verify_access_token(token: &str, secret: &str) -> AppResult<Claims> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
        _ => AppError::Unauthorized,
    })?;

    Ok(token_data.claims)
}

/// Extract user ID from claims
pub fn get_user_id(claims: &Claims) -> AppResult<Uuid> {
    claims
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized)
}

/// Extract session ID from claims
pub fn get_session_id(claims: &Claims) -> AppResult<Uuid> {
    claims
        .sid
        .parse()
        .map_err(|_| AppError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_roundtrip() {
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let secret = "test_secret_key_32_chars_min____";

        let token = create_access_token(user_id, session_id, secret, 1).unwrap();
        let claims = verify_access_token(&token, secret).unwrap();

        assert_eq!(get_user_id(&claims).unwrap(), user_id);
        assert_eq!(get_session_id(&claims).unwrap(), session_id);
    }

    #[test]
    fn test_expired_token() {
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let secret = "test_secret_key_32_chars_min____";

        // Create token that expired
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            sid: session_id.to_string(),
            iat: (now - Duration::hours(2)).timestamp(),
            exp: (now - Duration::hours(1)).timestamp(),
            jti: Uuid::new_v4().to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let result = verify_access_token(&token, secret);
        assert!(matches!(result, Err(AppError::TokenExpired)));
    }
}
