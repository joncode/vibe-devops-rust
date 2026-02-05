//! Admin panel handlers

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::api::responses::PaginatedResponse;
use crate::repositories::AdminRepository;
use crate::AppState;

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 25 }

/// Dashboard stats response
#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub users: i64,
    pub social_identifiers: i64,
    pub user_passwords: i64,
    pub session_tokens: i64,
    pub refresh_tokens: i64,
}

/// Get dashboard statistics
pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<DashboardStats>, (StatusCode, String)> {
    let repo = AdminRepository::new(state.db.pool());
    
    let stats = DashboardStats {
        users: repo.count_users().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        social_identifiers: repo.count_social_identifiers().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        user_passwords: repo.count_user_passwords().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        session_tokens: repo.count_session_tokens().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        refresh_tokens: repo.count_refresh_tokens().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    };

    Ok(Json(stats))
}

/// List users with pagination
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let repo = AdminRepository::new(state.db.pool());
    
    let total = repo.count_users().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let users = repo.list_users(params.page, params.per_page).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(PaginatedResponse::new(users, params.page, params.per_page, total as u64)))
}

/// List social identifiers with pagination
pub async fn list_social_identifiers(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let repo = AdminRepository::new(state.db.pool());
    
    let total = repo.count_social_identifiers().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let identifiers = repo.list_social_identifiers(params.page, params.per_page).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(PaginatedResponse::new(identifiers, params.page, params.per_page, total as u64)))
}

/// List user passwords with pagination
pub async fn list_user_passwords(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let repo = AdminRepository::new(state.db.pool());
    
    let total = repo.count_user_passwords().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let passwords = repo.list_user_passwords(params.page, params.per_page).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(PaginatedResponse::new(passwords, params.page, params.per_page, total as u64)))
}

/// List session tokens with pagination
pub async fn list_session_tokens(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let repo = AdminRepository::new(state.db.pool());
    
    let total = repo.count_session_tokens().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let sessions = repo.list_session_tokens(params.page, params.per_page).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(PaginatedResponse::new(sessions, params.page, params.per_page, total as u64)))
}

/// List refresh tokens with pagination
pub async fn list_refresh_tokens(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let repo = AdminRepository::new(state.db.pool());
    
    let total = repo.count_refresh_tokens().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let tokens = repo.list_refresh_tokens(params.page, params.per_page).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(PaginatedResponse::new(tokens, params.page, params.per_page, total as u64)))
}

/// Serve the admin panel HTML
pub async fn admin_panel() -> Response {
    let html = include_str!("../../static/admin.html");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ).into_response()
}
