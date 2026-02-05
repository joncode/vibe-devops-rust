//! Deployment handlers for the Deployer Agent API

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    api::responses::ApiResponse,
    errors::AppResult,
    models::{
        CreatePrRequest, DeployPlan, DeploymentResponse, DeploymentStatusResponse, PlanRequest,
        StatusQuery,
    },
    services::DeployerService,
    AppState,
};

/// Plan response
#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub deployment: DeploymentResponse,
    pub plan: DeployPlan,
}

/// Create a deployment plan
///
/// POST /api/v1/deploy/plan
pub async fn create_plan(
    State(state): State<AppState>,
    Json(req): Json<PlanRequest>,
) -> AppResult<Json<ApiResponse<PlanResponse>>> {
    let service = DeployerService::new(&state.db);

    let (deployment, plan) = service.plan(req.into()).await?;

    Ok(Json(ApiResponse::success(PlanResponse { deployment, plan })))
}

/// Create a PR from a planned deployment
///
/// POST /api/v1/deploy/pr
pub async fn create_pr(
    State(state): State<AppState>,
    Json(req): Json<CreatePrRequest>,
) -> AppResult<Json<ApiResponse<DeploymentResponse>>> {
    let service = DeployerService::new(&state.db);

    let deployment = service.create_pr(req).await?;

    Ok(Json(ApiResponse::success(deployment)))
}

/// Get deployment status
///
/// GET /api/v1/deploy/status/:deployment_id
pub async fn get_status(
    State(state): State<AppState>,
    Path(deployment_id): Path<String>,
) -> AppResult<Json<ApiResponse<DeploymentStatusResponse>>> {
    let service = DeployerService::new(&state.db);

    let status = service.get_status(&deployment_id).await?;

    Ok(Json(ApiResponse::success(status)))
}

/// List deployments with optional filtering
///
/// GET /api/v1/deploy/status
pub async fn list_deployments(
    State(state): State<AppState>,
    Query(query): Query<StatusQuery>,
) -> AppResult<Json<ApiResponse<Vec<DeploymentResponse>>>> {
    let service = DeployerService::new(&state.db);

    let deployments = service.list_deployments(query).await?;

    Ok(Json(ApiResponse::success(deployments)))
}

/// Cancel a deployment
///
/// POST /api/v1/deploy/:deployment_id/cancel
#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    #[serde(default)]
    pub reason: Option<String>,
}

pub async fn cancel_deployment(
    State(state): State<AppState>,
    Path(deployment_id): Path<String>,
    Json(req): Json<CancelRequest>,
) -> AppResult<Json<ApiResponse<DeploymentResponse>>> {
    let service = DeployerService::new(&state.db);

    let deployment = service.cancel(&deployment_id, req.reason.as_deref()).await?;

    Ok(Json(ApiResponse::success(deployment)))
}

/// Notify that a PR was merged (webhook handler)
///
/// POST /api/v1/deploy/:deployment_id/pr-merged
pub async fn notify_pr_merged(
    State(state): State<AppState>,
    Path(deployment_id): Path<String>,
) -> AppResult<Json<ApiResponse<DeploymentResponse>>> {
    let service = DeployerService::new(&state.db);

    let deployment = service.notify_pr_merged(&deployment_id).await?;

    Ok(Json(ApiResponse::success(deployment)))
}

/// Start verification for a deployment
///
/// POST /api/v1/deploy/:deployment_id/verify/start
pub async fn start_verification(
    State(state): State<AppState>,
    Path(deployment_id): Path<String>,
) -> AppResult<Json<ApiResponse<DeploymentResponse>>> {
    let service = DeployerService::new(&state.db);

    let deployment = service.start_verification(&deployment_id).await?;

    Ok(Json(ApiResponse::success(deployment)))
}

/// Complete verification
///
/// POST /api/v1/deploy/:deployment_id/verify/complete
#[derive(Debug, Deserialize)]
pub struct CompleteVerificationRequest {
    pub passed: bool,
    #[serde(default)]
    pub message: Option<String>,
}

pub async fn complete_verification(
    State(state): State<AppState>,
    Path(deployment_id): Path<String>,
    Json(req): Json<CompleteVerificationRequest>,
) -> AppResult<Json<ApiResponse<DeploymentResponse>>> {
    let service = DeployerService::new(&state.db);

    let deployment = service
        .complete_verification(&deployment_id, req.passed, req.message.as_deref())
        .await?;

    Ok(Json(ApiResponse::success(deployment)))
}
