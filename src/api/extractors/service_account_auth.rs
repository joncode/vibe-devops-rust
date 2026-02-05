//! Service account authentication extractor
use axum::{async_trait, extract::FromRequestParts, http::{header::AUTHORIZATION, request::Parts}};
use uuid::Uuid;
use crate::{errors::AppError, models::{PermissionScope, ServiceAccount}, repositories::ServiceAccountRepository, services::{extract_api_key_prefix, verify_api_key}, AppState};

#[derive(Debug, Clone)]
pub struct AuthenticatedServiceAccount {
    pub id: Uuid,
    pub hex_id: String,
    pub user_id: Uuid,
    pub name: String,
    pub permissions: Vec<PermissionScope>,
    pub rate_limit_per_minute: i32,
}

impl AuthenticatedServiceAccount {
    pub fn has_permission(&self, req: &PermissionScope) -> bool {
        self.permissions.iter().any(|p: &PermissionScope| p.implies(req))
    }
}

impl From<ServiceAccount> for AuthenticatedServiceAccount {
    fn from(s: ServiceAccount) -> Self {
        let perms = s.get_permissions();
        Self {
            id: s.id,
            hex_id: s.hex_id,
            user_id: s.user_id,
            name: s.name,
            permissions: perms,
            rate_limit_per_minute: s.rate_limit_per_minute,
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedServiceAccount {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let header = parts.headers.get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        let key = header.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)?;
        let prefix = extract_api_key_prefix(key).ok_or(AppError::Unauthorized)?;

        let repo = ServiceAccountRepository::new(state.db.pool());
        let candidates: Vec<ServiceAccount> = repo.find_by_api_key_prefix(&prefix)
            .await
            .map_err(|_| AppError::Unauthorized)?;

        for c in candidates {
            if c.is_valid() && verify_api_key(key, &c.api_key_hash).unwrap_or(false) {
                let pool = state.db.pool().clone();
                let id = c.id;
                tokio::spawn(async move {
                    let _ = ServiceAccountRepository::new(&pool).update_last_used(id).await;
                });
                return Ok(Self::from(c));
            }
        }
        Err(AppError::Unauthorized)
    }
}
