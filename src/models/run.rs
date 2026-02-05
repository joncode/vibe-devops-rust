//! Run model (one-off commands)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// Run status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "run_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Starting,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl Default for RunStatus {
    fn default() -> Self {
        Self::Starting
    }
}

/// Run model - represents a one-off command execution
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Run {
    pub id: Uuid,
    pub hex_id: String,
    pub app_id: Uuid,
    pub release_id: Uuid,
    pub command: Vec<String>,
    pub status: RunStatus,
    pub exit_code: Option<i32>,
    pub logs_stream_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Run hex_id prefix
impl HexId for Run {
    const PREFIX: &'static str = "run";
}

/// Run for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResponse {
    pub id: String, // hex_id
    pub app_id: String, // app hex_id
    pub release_id: String, // release hex_id
    pub command: Vec<String>,
    pub status: RunStatus,
    pub exit_code: Option<i32>,
    pub logs_stream_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl RunResponse {
    pub fn from_run(run: Run, app_hex_id: String, release_hex_id: String) -> Self {
        Self {
            id: run.hex_id,
            app_id: app_hex_id,
            release_id: release_hex_id,
            command: run.command,
            status: run.status,
            exit_code: run.exit_code,
            logs_stream_url: run.logs_stream_url,
            created_at: run.created_at,
            finished_at: run.finished_at,
        }
    }
}

/// Request to start a new run
#[derive(Debug, Clone, Deserialize)]
pub struct CreateRunRequest {
    pub command: Vec<String>,
    pub release_id: Option<String>, // hex_id, uses current release if not specified
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,
    pub size: Option<String>,
    pub timeout: Option<u32>,
    #[serde(default)]
    pub attach: Option<bool>,
}

/// Request to update run status
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRunRequest {
    pub status: Option<RunStatus>,
    pub exit_code: Option<i32>,
    pub logs_stream_url: Option<String>,
}
