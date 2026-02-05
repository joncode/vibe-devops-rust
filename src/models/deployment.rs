//! Deployment models for the Deployer Agent

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// Deployment status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "deployment_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending,
    Planning,
    Planned,
    PrCreated,
    Deploying,
    Verifying,
    Succeeded,
    Failed,
    Cancelled,
}

impl Default for DeploymentStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for DeploymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Planning => write!(f, "planning"),
            Self::Planned => write!(f, "planned"),
            Self::PrCreated => write!(f, "pr_created"),
            Self::Deploying => write!(f, "deploying"),
            Self::Verifying => write!(f, "verifying"),
            Self::Succeeded => write!(f, "succeeded"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Deployment event type enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "deployment_event_type", rename_all = "lowercase")]
#[serde(rename_all = "snake_case")]
pub enum DeploymentEventType {
    Created,
    PlanStarted,
    PlanCompleted,
    PlanFailed,
    PrCreated,
    PrMerged,
    PrClosed,
    DeployStarted,
    DeployProgressing,
    VerifyStarted,
    VerifyPassed,
    VerifyFailed,
    Succeeded,
    Failed,
    Cancelled,
    RollbackStarted,
    RollbackCompleted,
    Comment,
}

/// Deployment model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Deployment {
    pub id: Uuid,
    pub hex_id: String,
    pub app_name: String,
    pub environment: String,
    pub target_sha: Option<String>,
    pub target_ref: Option<String>,
    pub status: DeploymentStatus,
    pub pr_url: Option<String>,
    pub pr_number: Option<i32>,
    pub pr_branch: Option<String>,
    pub plan_json: Option<serde_json::Value>,
    pub triggered_by: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl HexId for Deployment {
    const PREFIX: &'static str = "dep";
}

/// Deployment event model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeploymentEvent {
    pub id: Uuid,
    pub hex_id: String,
    pub deployment_id: Uuid,
    pub event_type: DeploymentEventType,
    pub event_data: Option<serde_json::Value>,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl HexId for DeploymentEvent {
    const PREFIX: &'static str = "evt";
}

/// Deploy intent - the request to deploy something
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployIntent {
    pub app_name: String,
    pub environment: String,
    pub target_sha: Option<String>,
    pub target_ref: Option<String>,
    pub triggered_by: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl DeployIntent {
    pub fn validate(&self) -> Result<(), String> {
        if self.app_name.is_empty() {
            return Err("app_name is required".to_string());
        }
        if self.environment.is_empty() {
            return Err("environment is required".to_string());
        }
        if self.target_sha.is_none() && self.target_ref.is_none() {
            return Err("Either target_sha or target_ref is required".to_string());
        }
        Ok(())
    }
}

/// Infrastructure check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraCheck {
    pub name: String,
    pub exists: bool,
    pub details: Option<serde_json::Value>,
}

/// Deployment plan - computed changes needed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployPlan {
    pub intent: DeployIntent,
    pub infra_checks: Vec<InfraCheck>,
    pub platform_checks: Vec<InfraCheck>,
    pub app_checks: Vec<InfraCheck>,
    pub file_changes: Vec<FileChange>,
    pub terraform_changes: Option<TerraformPlan>,
    pub helm_changes: Vec<HelmChange>,
    pub requires_approval: bool,
    pub warnings: Vec<String>,
    pub estimated_time_seconds: Option<u32>,
}

/// File change in the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub action: FileAction,
    pub content: Option<String>,
    pub diff: Option<String>,
}

/// File action type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileAction {
    Create,
    Modify,
    Delete,
}

/// Terraform plan summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformPlan {
    pub resources_to_add: Vec<String>,
    pub resources_to_change: Vec<String>,
    pub resources_to_destroy: Vec<String>,
    pub plan_output: Option<String>,
}

/// Helm release change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmChange {
    pub release_name: String,
    pub chart: String,
    pub current_version: Option<String>,
    pub target_version: String,
    pub values_changes: Option<serde_json::Value>,
}

/// Deployment response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub id: String,
    pub app_name: String,
    pub environment: String,
    pub target_sha: Option<String>,
    pub target_ref: Option<String>,
    pub status: DeploymentStatus,
    pub pr_url: Option<String>,
    pub pr_number: Option<i32>,
    pub triggered_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<Deployment> for DeploymentResponse {
    fn from(d: Deployment) -> Self {
        Self {
            id: d.hex_id,
            app_name: d.app_name,
            environment: d.environment,
            target_sha: d.target_sha,
            target_ref: d.target_ref,
            status: d.status,
            pr_url: d.pr_url,
            pr_number: d.pr_number,
            triggered_by: d.triggered_by,
            created_at: d.created_at,
            started_at: d.started_at,
            completed_at: d.completed_at,
        }
    }
}

/// Deployment event response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentEventResponse {
    pub id: String,
    pub event_type: DeploymentEventType,
    pub message: Option<String>,
    pub event_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<DeploymentEvent> for DeploymentEventResponse {
    fn from(e: DeploymentEvent) -> Self {
        Self {
            id: e.hex_id,
            event_type: e.event_type,
            message: e.message,
            event_data: e.event_data,
            created_at: e.created_at,
        }
    }
}

/// Plan request from API
#[derive(Debug, Clone, Deserialize)]
pub struct PlanRequest {
    pub app_name: String,
    pub environment: String,
    #[serde(default)]
    pub target_sha: Option<String>,
    #[serde(default)]
    pub target_ref: Option<String>,
    #[serde(default)]
    pub triggered_by: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl From<PlanRequest> for DeployIntent {
    fn from(req: PlanRequest) -> Self {
        Self {
            app_name: req.app_name,
            environment: req.environment,
            target_sha: req.target_sha,
            target_ref: req.target_ref,
            triggered_by: req.triggered_by,
            metadata: req.metadata.unwrap_or_default(),
        }
    }
}

/// Create PR request from API
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePrRequest {
    pub deployment_id: String,
    #[serde(default)]
    pub pr_title: Option<String>,
    #[serde(default)]
    pub pr_body: Option<String>,
}

/// Status query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct StatusQuery {
    #[serde(default)]
    pub app: Option<String>,
    #[serde(default)]
    pub env: Option<String>,
    #[serde(default)]
    pub deployment_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

/// Detailed deployment status response
#[derive(Debug, Clone, Serialize)]
pub struct DeploymentStatusResponse {
    pub deployment: DeploymentResponse,
    pub events: Vec<DeploymentEventResponse>,
    pub plan: Option<DeployPlan>,
}
