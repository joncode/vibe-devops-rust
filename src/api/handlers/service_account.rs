//! Service account API handlers
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use crate::{
    api::extractors::AuthenticatedUser,
    errors::{AppError, AppResult},
    models::{CreateServiceAccountRequest, PermissionScope, ServiceAccount, ServiceAccountCreatedResponse, ServiceAccountResponse, UpdateServiceAccountRequest, UsageStats},
    repositories::{ServiceAccountRepository, ApiKeyUsageRepository},
    services::generate_api_key,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 { 20 }

pub async fn create_service_account(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateServiceAccountRequest>,
) -> AppResult<(StatusCode, Json<ServiceAccountCreatedResponse>)> {
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("Name required".into()));
    }
    for p in &req.permissions {
        if PermissionScope::from_str(p).is_none() {
            return Err(AppError::Validation(format!("Invalid permission: {}", p)));
        }
    }
    if req.rate_limit_per_minute < 1 || req.rate_limit_per_minute > 10000 {
        return Err(AppError::Validation("Rate limit must be 1-10000".into()));
    }

    let key = generate_api_key()?;
    let perms = serde_json::to_value(&req.permissions).unwrap_or(serde_json::json!([]));
    let repo = ServiceAccountRepository::new(state.db.pool());
    let sa: ServiceAccount = repo.create(user.user_id, &req.name, req.description.as_deref(), &key.hash, &key.prefix, &perms, req.rate_limit_per_minute, req.expires_at)
        .await
        .map_err(|e| { tracing::error!("Create failed: {:?}", e); AppError::Internal(anyhow::anyhow!("Create failed")) })?;

    Ok((StatusCode::CREATED, Json(ServiceAccountCreatedResponse { service_account: ServiceAccountResponse::from(sa), api_key: key.full_key })))
}

pub async fn list_service_accounts(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(p): Query<PaginationParams>,
) -> AppResult<Json<Vec<ServiceAccountResponse>>> {
    let repo = ServiceAccountRepository::new(state.db.pool());
    let accts: Vec<ServiceAccount> = repo.list_by_user(user.user_id, p.limit.min(100).max(1), p.offset.max(0)).await?;
    Ok(Json(accts.into_iter().map(ServiceAccountResponse::from).collect()))
}

pub async fn get_service_account(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(hex_id): Path<String>,
) -> AppResult<Json<ServiceAccountResponse>> {
    let repo = ServiceAccountRepository::new(state.db.pool());
    let a: ServiceAccount = repo.find_by_hex_id(&hex_id).await?.ok_or(AppError::NotFound)?;
    if a.user_id != user.user_id { return Err(AppError::Forbidden); }
    Ok(Json(ServiceAccountResponse::from(a)))
}

pub async fn update_service_account(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(hex_id): Path<String>,
    Json(req): Json<UpdateServiceAccountRequest>,
) -> AppResult<Json<ServiceAccountResponse>> {
    let repo = ServiceAccountRepository::new(state.db.pool());
    let a: ServiceAccount = repo.find_by_hex_id(&hex_id).await?.ok_or(AppError::NotFound)?;
    if a.user_id != user.user_id { return Err(AppError::Forbidden); }

    let perms = req.permissions.as_ref().map(|ps| {
        for p in ps {
            if PermissionScope::from_str(p).is_none() {
                return Err(AppError::Validation(format!("Invalid permission: {}", p)));
            }
        }
        Ok(serde_json::to_value(ps).unwrap())
    }).transpose()?;

    let u: ServiceAccount = repo.update(a.id, req.name.as_deref(), req.description.as_deref(), perms.as_ref(), req.rate_limit_per_minute, req.expires_at, req.is_active)
        .await?.ok_or(AppError::NotFound)?;
    Ok(Json(ServiceAccountResponse::from(u)))
}

pub async fn delete_service_account(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(hex_id): Path<String>,
) -> AppResult<StatusCode> {
    let repo = ServiceAccountRepository::new(state.db.pool());
    let a: ServiceAccount = repo.find_by_hex_id(&hex_id).await?.ok_or(AppError::NotFound)?;
    if a.user_id != user.user_id { return Err(AppError::Forbidden); }
    repo.delete(a.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn rotate_api_key(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(hex_id): Path<String>,
) -> AppResult<Json<ServiceAccountCreatedResponse>> {
    let repo = ServiceAccountRepository::new(state.db.pool());
    let a: ServiceAccount = repo.find_by_hex_id(&hex_id).await?.ok_or(AppError::NotFound)?;
    if a.user_id != user.user_id { return Err(AppError::Forbidden); }
    let key = generate_api_key()?;
    let u: ServiceAccount = repo.rotate_api_key(a.id, &key.hash, &key.prefix).await?.ok_or(AppError::NotFound)?;
    Ok(Json(ServiceAccountCreatedResponse { service_account: ServiceAccountResponse::from(u), api_key: key.full_key }))
}

pub async fn get_service_account_usage(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(hex_id): Path<String>,
) -> AppResult<Json<UsageStats>> {
    let sr = ServiceAccountRepository::new(state.db.pool());
    let a: ServiceAccount = sr.find_by_hex_id(&hex_id).await?.ok_or(AppError::NotFound)?;
    if a.user_id != user.user_id { return Err(AppError::Forbidden); }
    let ur = ApiKeyUsageRepository::new(state.db.pool());
    Ok(Json(ur.get_stats(a.id).await?))
}
