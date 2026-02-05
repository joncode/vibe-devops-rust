//! Authentication extractor

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    services::auth::{get_session_id, get_user_id, verify_access_token},
    AppState,
};

/// Authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Get authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        // Parse Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        // Verify JWT
        let claims = verify_access_token(token, &state.config.auth.jwt_secret)?;

        // Extract user and session IDs
        let user_id = get_user_id(&claims)?;
        let session_id = get_session_id(&claims)?;

        Ok(AuthenticatedUser { user_id, session_id })
    }
}

/// Optional authentication - doesn't fail if not authenticated
#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<AuthenticatedUser>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthenticatedUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuth(Some(user))),
            Err(_) => Ok(OptionalAuth(None)),
        }
    }
}
