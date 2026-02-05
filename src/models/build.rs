//! Build model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// Build status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "build_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BuildStatus {
    Queued,
    Building,
    Succeeded,
    Failed,
    Cancelled,
}

impl Default for BuildStatus {
    fn default() -> Self {
        Self::Queued
    }
}

/// Build model - represents a build process
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Build {
    pub id: Uuid,
    pub hex_id: String,
    pub app_id: Uuid,
    pub source_sha: String,
    pub source_ref: Option<String>,
    pub status: BuildStatus,
    pub output_artifact_digest: Option<String>,
    pub logs_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Build hex_id prefix
impl HexId for Build {
    const PREFIX: &'static str = "bld";
}

/// Build for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResponse {
    pub id: String, // hex_id
    pub app_id: String, // app hex_id
    pub source_sha: String,
    pub source_ref: Option<String>,
    pub status: BuildStatus,
    pub output_artifact_digest: Option<String>,
    pub logs_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl BuildResponse {
    pub fn from_build(build: Build, app_hex_id: String) -> Self {
        Self {
            id: build.hex_id,
            app_id: app_hex_id,
            source_sha: build.source_sha,
            source_ref: build.source_ref,
            status: build.status,
            output_artifact_digest: build.output_artifact_digest,
            logs_url: build.logs_url,
            created_at: build.created_at,
            finished_at: build.finished_at,
        }
    }
}

/// Request to trigger a new build
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBuildRequest {
    pub source_sha: String,
    pub source_ref: Option<String>,
}

/// Request to update build status
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBuildRequest {
    pub status: Option<BuildStatus>,
    pub output_artifact_digest: Option<String>,
    pub logs_url: Option<String>,
}
