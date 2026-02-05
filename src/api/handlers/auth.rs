//! Authentication handlers

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    api::extractors::AuthenticatedUser,
    api::responses::ApiResponse,
    errors::AppResult,
    services::auth::{AuthService, AuthTokens},
    AppState,
};

/// Register request body
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

/// Register response
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub message: String,
}

/// Login request body
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub device_info: serde_json::Value,
}

/// Refresh request body
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Register a new user
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<RegisterResponse>>)> {
    let auth_service = AuthService::new(&state.db, &state.redis, &state.config.auth);

    let user = auth_service
        .register(&req.email, &req.password, req.display_name.as_deref())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(RegisterResponse {
            user_id: user.id.to_string(),
            message: "Registration successful. Please verify your email.".to_string(),
        })),
    ))
}

/// Login with email and password
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<AuthTokens>>> {
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Get client IP from X-Forwarded-For or connection
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string());

    let device_info = if req.device_info.is_null() {
        serde_json::json!({})
    } else {
        req.device_info
    };

    let auth_service = AuthService::new(&state.db, &state.redis, &state.config.auth);

    let tokens = auth_service
        .login(
            &req.email,
            &req.password,
            device_info,
            ip_address.as_deref(),
            user_agent.as_deref(),
        )
        .await?;

    Ok(Json(ApiResponse::success(tokens)))
}

/// Refresh access token
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<ApiResponse<AuthTokens>>> {
    let auth_service = AuthService::new(&state.db, &state.redis, &state.config.auth);

    let tokens = auth_service.refresh_tokens(&req.refresh_token).await?;

    Ok(Json(ApiResponse::success(tokens)))
}

/// Logout (revoke current session)
pub async fn logout(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<StatusCode> {
    let auth_service = AuthService::new(&state.db, &state.redis, &state.config.auth);

    auth_service.logout(user.session_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
