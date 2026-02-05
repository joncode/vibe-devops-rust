//! Artifact model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// Artifact type enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "artifact_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ArtifactType {
    Slug,
    #[sqlx(rename = "oci-image")]
    #[serde(rename = "oci-image")]
    OciImage,
}

/// Artifact model - represents a built artifact (slug or image)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Artifact {
    pub id: Uuid,
    pub hex_id: String,
    pub digest: String,
    pub artifact_type: ArtifactType,
    pub size_bytes: Option<i64>,
    pub uri: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Artifact hex_id prefix
impl HexId for Artifact {
    const PREFIX: &'static str = "art";
}

/// Artifact for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactResponse {
    pub id: String, // hex_id
    pub digest: String,
    pub artifact_type: ArtifactType,
    pub size_bytes: Option<i64>,
    pub uri: String,
    pub created_at: DateTime<Utc>,
}

impl From<Artifact> for ArtifactResponse {
    fn from(artifact: Artifact) -> Self {
        Self {
            id: artifact.hex_id,
            digest: artifact.digest,
            artifact_type: artifact.artifact_type,
            size_bytes: artifact.size_bytes,
            uri: artifact.uri,
            created_at: artifact.created_at,
        }
    }
}

/// Request to create a new artifact
#[derive(Debug, Clone, Deserialize)]
pub struct CreateArtifactRequest {
    pub digest: String,
    pub artifact_type: ArtifactType,
    pub size_bytes: Option<i64>,
    pub uri: String,
}
