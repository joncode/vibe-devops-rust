//! Server API handlers
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use crate::api::responses::{PaginatedResponse, SuccessResponse};
use crate::errors::AppResult;
use crate::models::{BulkServiceStatusRequest, CreateServerRequest, CreateServerServiceRequest, ServerEnvironment, ServerHeartbeatRequest, ServerHeartbeatResponse, ServerResponse, ServerServiceResponse, ServerStatus, UpdateServerRequest, UpdateServerServiceRequest};
use crate::services::ServerManagementService;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ServerListParams { #[serde(default = "default_page")] pub page: u32, #[serde(default = "default_per_page")] pub per_page: u32, pub environment: Option<ServerEnvironment>, pub status: Option<ServerStatus> }
#[derive(Debug, Deserialize)]
pub struct ServiceListParams { #[serde(default = "default_page")] pub page: u32, #[serde(default = "default_per_page")] pub per_page: u32, pub namespace: Option<String> }
fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 25 }

pub async fn create_server(State(state): State<AppState>, Json(req): Json<CreateServerRequest>) -> AppResult<(StatusCode, Json<ServerResponse>)> {
    let s = ServerManagementService::new(state.db.pool());
    Ok((StatusCode::CREATED, Json(s.create_server(&req.name, &req.hostname, &req.ip_address, req.environment, req.k3s_version.as_deref(), req.specs.as_ref(), req.metadata.as_ref()).await?)))
}

pub async fn get_server(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<ServerResponse>> {
    Ok(Json(ServerManagementService::new(state.db.pool()).get_server(&id).await?))
}

pub async fn update_server(State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<UpdateServerRequest>) -> AppResult<Json<ServerResponse>> {
    Ok(Json(ServerManagementService::new(state.db.pool()).update_server(&id, req.name.as_deref(), req.hostname.as_deref(), req.ip_address.as_deref(), req.environment.as_ref(), req.status.as_ref(), req.k3s_version.as_deref(), req.specs.as_ref(), req.metadata.as_ref()).await?))
}

pub async fn delete_server(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<SuccessResponse>> {
    ServerManagementService::new(state.db.pool()).delete_server(&id).await?; Ok(Json(SuccessResponse::new("Server deleted")))
}

pub async fn list_servers(State(state): State<AppState>, Query(p): Query<ServerListParams>) -> AppResult<Json<PaginatedResponse<ServerResponse>>> {
    let s = ServerManagementService::new(state.db.pool());
    let servers = s.list_servers(p.environment.clone(), p.status.clone(), p.per_page as i64, ((p.page - 1) * p.per_page) as i64).await?;
    let total = s.count_servers(p.environment, p.status).await?;
    Ok(Json(PaginatedResponse::new(servers, p.page, p.per_page, total as u64)))
}

pub async fn server_heartbeat(State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<ServerHeartbeatRequest>) -> AppResult<Json<ServerHeartbeatResponse>> {
    Ok(Json(ServerManagementService::new(state.db.pool()).record_heartbeat(&id, &req).await?))
}

pub async fn update_service_statuses(State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<BulkServiceStatusRequest>) -> AppResult<Json<Vec<ServerServiceResponse>>> {
    Ok(Json(ServerManagementService::new(state.db.pool()).update_service_statuses(&id, &req).await?))
}

pub async fn create_service(State(state): State<AppState>, Path(server_id): Path<String>, Json(mut req): Json<CreateServerServiceRequest>) -> AppResult<(StatusCode, Json<ServerServiceResponse>)> {
    req.server_id = server_id;
    Ok((StatusCode::CREATED, Json(ServerManagementService::new(state.db.pool()).create_service(&req.server_id, &req.service_name, &req.namespace, req.replicas_desired, req.image_tag.as_deref(), req.health_endpoint.as_deref(), req.metadata.as_ref()).await?)))
}

pub async fn list_server_services(State(state): State<AppState>, Path(server_id): Path<String>) -> AppResult<Json<Vec<ServerServiceResponse>>> {
    Ok(Json(ServerManagementService::new(state.db.pool()).list_services_by_server(&server_id).await?))
}

pub async fn get_service(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<ServerServiceResponse>> {
    Ok(Json(ServerManagementService::new(state.db.pool()).get_service(&id).await?))
}

pub async fn update_service(State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<UpdateServerServiceRequest>) -> AppResult<Json<ServerServiceResponse>> {
    Ok(Json(ServerManagementService::new(state.db.pool()).update_service(&id, req.status.as_ref(), req.replicas_desired, req.replicas_ready, req.image_tag.as_deref(), req.health_endpoint.as_deref(), req.metadata.as_ref()).await?))
}

pub async fn delete_service(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<SuccessResponse>> {
    ServerManagementService::new(state.db.pool()).delete_service(&id).await?; Ok(Json(SuccessResponse::new("Service deleted")))
}

pub async fn list_services(State(state): State<AppState>, Query(p): Query<ServiceListParams>) -> AppResult<Json<PaginatedResponse<ServerServiceResponse>>> {
    let s = ServerManagementService::new(state.db.pool());
    let services = if let Some(ns) = &p.namespace { s.list_services_by_namespace(ns).await? } else { s.list_services(p.per_page as i64, ((p.page - 1) * p.per_page) as i64).await? };
    Ok(Json(PaginatedResponse::new(services, p.page, p.per_page, s.count_services().await? as u64)))
}

pub async fn list_unhealthy_services(State(state): State<AppState>) -> AppResult<Json<Vec<ServerServiceResponse>>> {
    Ok(Json(ServerManagementService::new(state.db.pool()).find_unhealthy_services().await?))
}
