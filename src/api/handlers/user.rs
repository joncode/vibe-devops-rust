//! User handlers

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    api::extractors::AuthenticatedUser,
    api::responses::ApiResponse,
    errors::{AppError, AppResult},
    models::{SessionResponse, UserResponse},
    repositories::{SessionRepository, UserRepository},
    AppState,
};

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Get current user
pub async fn get_me(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let repo = UserRepository::new(state.db.pool());
    
    let user_data = repo
        .find_by_id(user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ApiResponse::success(UserResponse::from(user_data))))
}

/// Update current user
pub async fn update_me(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let repo = UserRepository::new(state.db.pool());
    
    let user_data = repo
        .update(
            user.user_id,
            req.display_name.as_deref(),
            req.avatar_url.as_deref(),
        )
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ApiResponse::success(UserResponse::from(user_data))))
}

/// Delete current user (soft delete)
pub async fn delete_me(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<StatusCode> {
    let repo = UserRepository::new(state.db.pool());
    
    repo.delete(user.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get current user's sessions
pub async fn get_sessions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<Vec<SessionResponse>>>> {
    let repo = SessionRepository::new(state.db.pool());
    
    let sessions = repo.find_by_user(user.user_id).await?;
    
    let responses: Vec<SessionResponse> = sessions
        .into_iter()
        .map(|s| {
            let is_current = s.id == user.session_id;
            SessionResponse {
                id: s.hex_id,  // Use hex_id as public ID
                device_info: s.device_info,
                ip_address: s.ip_address,
                last_used_at: s.last_used_at,
                created_at: s.created_at,
                is_current,
            }
        })
        .collect();

    Ok(Json(ApiResponse::success(responses)))
}
