//! Release model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// Release status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "release_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ReleaseStatus {
    Pending,
    Deploying,
    Active,
    Failed,
    RolledBack,
}

impl Default for ReleaseStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Release model - represents a deployed version of an app
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Release {
    pub id: Uuid,
    pub hex_id: String,
    pub app_id: Uuid,
    pub build_id: Uuid,
    pub artifact_digest: String,
    pub version: i32,
    pub config_snapshot: Option<serde_json::Value>,
    pub status: ReleaseStatus,
    pub process_formation: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Release hex_id prefix
impl HexId for Release {
    const PREFIX: &'static str = "rel";
}

/// Release for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseResponse {
    pub id: String, // hex_id
    pub app_id: String, // app hex_id
    pub build_id: String, // build hex_id
    pub artifact_digest: String,
    pub version: i32,
    pub config_snapshot: Option<serde_json::Value>,
    pub status: ReleaseStatus,
    pub process_formation: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl ReleaseResponse {
    pub fn from_release(release: Release, app_hex_id: String, build_hex_id: String) -> Self {
        Self {
            id: release.hex_id,
            app_id: app_hex_id,
            build_id: build_hex_id,
            artifact_digest: release.artifact_digest,
            version: release.version,
            config_snapshot: release.config_snapshot,
            status: release.status,
            process_formation: release.process_formation,
            created_at: release.created_at,
        }
    }
}

/// Request to create a new release
#[derive(Debug, Clone, Deserialize)]
pub struct CreateReleaseRequest {
    pub build_id: String, // hex_id
    pub artifact_digest: String,
    pub config_snapshot: Option<serde_json::Value>,
    pub process_formation: Option<serde_json::Value>,
}

/// Request to update release status
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateReleaseRequest {
    pub status: Option<ReleaseStatus>,
    pub process_formation: Option<serde_json::Value>,
}

/// Request to rollback to a specific version
#[derive(Debug, Clone, Deserialize)]
pub struct RollbackRequest {
    pub version: i32,
}
