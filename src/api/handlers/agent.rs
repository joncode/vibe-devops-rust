use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use crate::{api::responses::{ApiResponse, PaginatedResponse}, errors::{AppError, AppResult},
    models::{AgentTaskResponse, AgentTaskStatus, CreateDeployTaskRequest},
    services::agent::{AgentService, OpenAIToolDefinition, TaskLogsResponse, TaskStatusResponse, ToolContext}, AppState};
#[derive(Debug, Serialize)]
pub struct DeployResponse { pub task_id: String, pub status: AgentTaskStatus, pub message: String }
#[derive(Debug, Deserialize)]
pub struct ListTasksQuery { #[serde(default)] pub status: Option<String>, #[serde(default)] pub agent_type: Option<String>,
    #[serde(default = "default_limit")] pub limit: i64, #[serde(default)] pub offset: i64 }
fn default_limit() -> i64 { 20 }
#[derive(Debug, Deserialize)]
pub struct TaskLogsQuery { #[serde(default)] pub include_outputs: bool }
#[derive(Debug, Serialize)]
pub struct ToolsResponse { pub tools: Vec<OpenAIToolDefinition> }
pub async fn deploy(State(state): State<AppState>, Json(req): Json<CreateDeployTaskRequest>) -> AppResult<(StatusCode, Json<ApiResponse<DeployResponse>>)> {
    if req.message.trim().is_empty() { return Err(AppError::Validation("message required".into())); }
    let svc = AgentService::new(&state.db); let task = svc.create_task(&req).await?; let task = svc.start_task(task.id).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::success(DeployResponse { task_id: task.hex_id, status: task.status, message: format!("Task created: {}", req.message) })))) }
pub async fn get_status(State(state): State<AppState>, Path(task_id): Path<String>) -> AppResult<Json<ApiResponse<TaskStatusResponse>>> {
    Ok(Json(ApiResponse::success(AgentService::new(&state.db).get_task_status(&task_id).await?))) }
pub async fn get_logs(State(state): State<AppState>, Path(task_id): Path<String>, Query(q): Query<TaskLogsQuery>) -> AppResult<Json<ApiResponse<TaskLogsResponse>>> {
    Ok(Json(ApiResponse::success(AgentService::new(&state.db).get_task_logs(&task_id, q.include_outputs).await?))) }
pub async fn list_tasks(State(state): State<AppState>, Query(q): Query<ListTasksQuery>) -> AppResult<Json<PaginatedResponse<AgentTaskResponse>>> {
    let status = q.status.as_ref().and_then(|s| match s.to_lowercase().as_str() { "pending" => Some(AgentTaskStatus::Pending), "running" => Some(AgentTaskStatus::Running),
        "completed" => Some(AgentTaskStatus::Completed), "failed" => Some(AgentTaskStatus::Failed), "cancelled" => Some(AgentTaskStatus::Cancelled), _ => None });
    let r = AgentService::new(&state.db).list_tasks(status, q.agent_type.as_deref(), q.limit, q.offset).await?;
    Ok(Json(PaginatedResponse::new(r.tasks, (q.offset / q.limit) as u32 + 1, q.limit as u32, r.total as u64))) }
pub async fn cancel_task(State(state): State<AppState>, Path(task_id): Path<String>) -> AppResult<Json<ApiResponse<AgentTaskResponse>>> {
    let svc = AgentService::new(&state.db); let task = svc.get_task(&task_id).await?;
    if !matches!(task.status, AgentTaskStatus::Running | AgentTaskStatus::Pending) { return Err(AppError::Validation(format!("Cannot cancel {:?}", task.status))); }
    Ok(Json(ApiResponse::success(svc.cancel_task(task.id).await?.into()))) }
pub async fn list_tools(State(state): State<AppState>) -> AppResult<Json<ApiResponse<ToolsResponse>>> {
    Ok(Json(ApiResponse::success(ToolsResponse { tools: AgentService::new(&state.db).get_available_tools() }))) }
#[derive(Debug, Deserialize)]
pub struct ExecuteToolRequest { pub tool_name: String, pub tool_input: serde_json::Value, pub environment: Option<String> }
pub async fn execute_tool(State(state): State<AppState>, Path(task_id): Path<String>, Json(req): Json<ExecuteToolRequest>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = AgentService::new(&state.db); let task = svc.get_task(&task_id).await?; let mut ctx = ToolContext::new(task.id);
    if let Some(env) = req.environment { ctx = ctx.with_environment(&env); }
    let e = svc.execute_tool(task.id, &req.tool_name, req.tool_input, &ctx).await?;
    Ok(Json(ApiResponse::success(serde_json::json!({"execution_id": e.hex_id, "tool_name": e.tool_name, "status": e.status, "output": e.tool_output})))) }
#[derive(Debug, Deserialize)]
pub struct CompleteTaskRequest { pub result: serde_json::Value }
pub async fn complete_task(State(state): State<AppState>, Path(task_id): Path<String>, Json(req): Json<CompleteTaskRequest>) -> AppResult<Json<ApiResponse<AgentTaskResponse>>> {
    let svc = AgentService::new(&state.db); let task = svc.get_task(&task_id).await?; Ok(Json(ApiResponse::success(svc.complete_task(task.id, req.result).await?.into()))) }
#[derive(Debug, Deserialize)]
pub struct FailTaskRequest { pub error: String }
pub async fn fail_task(State(state): State<AppState>, Path(task_id): Path<String>, Json(req): Json<FailTaskRequest>) -> AppResult<Json<ApiResponse<AgentTaskResponse>>> {
    let svc = AgentService::new(&state.db); let task = svc.get_task(&task_id).await?; Ok(Json(ApiResponse::success(svc.fail_task(task.id, &req.error).await?.into()))) }
