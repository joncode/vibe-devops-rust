//! Service Account model for programmatic API access
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use super::hex_id::HexId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionScope {
    #[serde(rename = "deploy:code")] DeployCode,
    #[serde(rename = "deploy:service")] DeployService,
    #[serde(rename = "deploy:full")] DeployFull,
    #[serde(rename = "manage:nodes")] ManageNodes,
    #[serde(rename = "read:status")] ReadStatus,
    #[serde(rename = "manage:deployments")] ManageDeployments,
    #[serde(rename = "read:all")] ReadAll,
    #[serde(rename = "admin")] Admin,
}

impl PermissionScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeployCode => "deploy:code",
            Self::DeployService => "deploy:service",
            Self::DeployFull => "deploy:full",
            Self::ManageNodes => "manage:nodes",
            Self::ReadStatus => "read:status",
            Self::ManageDeployments => "manage:deployments",
            Self::ReadAll => "read:all",
            Self::Admin => "admin",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "deploy:code" => Some(Self::DeployCode),
            "deploy:service" => Some(Self::DeployService),
            "deploy:full" => Some(Self::DeployFull),
            "manage:nodes" => Some(Self::ManageNodes),
            "read:status" => Some(Self::ReadStatus),
            "manage:deployments" => Some(Self::ManageDeployments),
            "read:all" => Some(Self::ReadAll),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    pub fn implies(&self, other: &Self) -> bool {
        match self {
            Self::Admin => true,
            Self::ManageDeployments => matches!(other, Self::DeployCode | Self::DeployService | Self::DeployFull | Self::ReadStatus),
            Self::DeployFull => matches!(other, Self::DeployCode | Self::DeployService),
            Self::ReadAll => matches!(other, Self::ReadStatus),
            _ => self == other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServiceAccount {
    pub id: Uuid,
    pub hex_id: String,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub api_key_hash: String,
    pub api_key_prefix: String,
    pub permissions: serde_json::Value,
    pub rate_limit_per_minute: i32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl HexId for ServiceAccount {
    const PREFIX: &'static str = "svc";
}

impl ServiceAccount {
    pub fn is_valid(&self) -> bool {
        self.is_active && self.deleted_at.is_none() && self.expires_at.map_or(true, |e| e > Utc::now())
    }

    pub fn get_permissions(&self) -> Vec<PermissionScope> {
        self.permissions.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().and_then(PermissionScope::from_str)).collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub api_key_prefix: String,
    pub permissions: Vec<String>,
    pub rate_limit_per_minute: i32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<ServiceAccount> for ServiceAccountResponse {
    fn from(s: ServiceAccount) -> Self {
        let permissions = s.get_permissions().iter().map(|p| p.as_str().to_string()).collect();
        Self {
            id: s.hex_id,
            name: s.name,
            description: s.description,
            api_key_prefix: s.api_key_prefix,
            permissions,
            rate_limit_per_minute: s.rate_limit_per_minute,
            last_used_at: s.last_used_at,
            expires_at: s.expires_at,
            is_active: s.is_active,
            created_at: s.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceAccountCreatedResponse {
    pub service_account: ServiceAccountResponse,
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateServiceAccountRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: i32,
    pub expires_at: Option<DateTime<Utc>>,
}

fn default_rate_limit() -> i32 { 60 }

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateServiceAccountRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub rate_limit_per_minute: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_implies() {
        assert!(PermissionScope::Admin.implies(&PermissionScope::DeployCode));
        assert!(!PermissionScope::DeployCode.implies(&PermissionScope::Admin));
    }

    #[test]
    fn test_hex_id() {
        assert!(ServiceAccount::generate_hex_id().starts_with("svc_"));
    }
}
