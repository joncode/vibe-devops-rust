//! Server management service
use sqlx::PgPool;
use crate::errors::AppError;
use crate::models::{BulkServiceStatusRequest, Server, ServerEnvironment, ServerHeartbeatRequest, ServerHeartbeatResponse, ServerResponse, ServerServiceResponse, ServerSpecs, ServerStatus, ServiceStatus};
use crate::repositories::{ServerRepository, ServerServiceRepository};

pub struct ServerManagementService<'a> { pool: &'a PgPool }

impl<'a> ServerManagementService<'a> {
    pub fn new(pool: &'a PgPool) -> Self { Self { pool } }

    pub async fn create_server(&self, name: &str, hostname: &str, ip_address: &str, environment: ServerEnvironment, k3s_version: Option<&str>, specs: Option<&ServerSpecs>, metadata: Option<&serde_json::Value>) -> Result<ServerResponse, AppError> {
        if ip_address.parse::<std::net::IpAddr>().is_err() { return Err(AppError::Validation(format!("Invalid IP: {}", ip_address))); }
        let repo = ServerRepository::new(self.pool);
        if repo.find_by_hostname(hostname).await?.is_some() { return Err(AppError::Conflict(format!("Hostname '{}' exists", hostname))); }
        let server = repo.create(name, hostname, ip_address, environment, k3s_version, specs, metadata).await.map_err(AppError::Internal)?;
        Ok(ServerResponse::from(server))
    }

    pub async fn get_server(&self, hex_id: &str) -> Result<ServerResponse, AppError> {
        let repo = ServerRepository::new(self.pool);
        let server = repo.find_by_hex_id(hex_id).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)?;
        Ok(ServerResponse::from(server))
    }

    async fn get_server_internal(&self, hex_id: &str) -> Result<Server, AppError> {
        let repo = ServerRepository::new(self.pool);
        repo.find_by_hex_id(hex_id).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)
    }

    pub async fn update_server(&self, hex_id: &str, name: Option<&str>, hostname: Option<&str>, ip_address: Option<&str>, environment: Option<&ServerEnvironment>, status: Option<&ServerStatus>, k3s_version: Option<&str>, specs: Option<&ServerSpecs>, metadata: Option<&serde_json::Value>) -> Result<ServerResponse, AppError> {
        let server = self.get_server_internal(hex_id).await?;
        if let Some(ip) = ip_address { if ip.parse::<std::net::IpAddr>().is_err() { return Err(AppError::Validation(format!("Invalid IP: {}", ip))); } }
        let repo = ServerRepository::new(self.pool);
        if let Some(h) = hostname { if h != server.hostname { if let Some(e) = repo.find_by_hostname(h).await? { if e.id != server.id { return Err(AppError::Conflict(format!("Hostname '{}' exists", h))); } } } }
        let updated = repo.update(server.id, name, hostname, ip_address, environment, status, k3s_version, specs, metadata).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)?;
        Ok(ServerResponse::from(updated))
    }

    pub async fn delete_server(&self, hex_id: &str) -> Result<(), AppError> {
        let server = self.get_server_internal(hex_id).await?;
        ServerRepository::new(self.pool).delete(server.id).await.map_err(AppError::Internal)?;
        Ok(())
    }

    pub async fn list_servers(&self, environment: Option<ServerEnvironment>, status: Option<ServerStatus>, limit: i64, offset: i64) -> Result<Vec<ServerResponse>, AppError> {
        let repo = ServerRepository::new(self.pool);
        let servers = match (environment, status) { (Some(e), _) => repo.list_by_environment(e, limit, offset).await, (None, Some(s)) => repo.list_by_status(s, limit, offset).await, _ => repo.list(limit, offset).await }.map_err(AppError::Internal)?;
        Ok(servers.into_iter().map(ServerResponse::from).collect())
    }

    pub async fn count_servers(&self, environment: Option<ServerEnvironment>, status: Option<ServerStatus>) -> Result<i64, AppError> {
        let repo = ServerRepository::new(self.pool);
        match (environment, status) { (Some(e), _) => repo.count_by_environment(e).await, (None, Some(s)) => repo.count_by_status(s).await, _ => repo.count().await }.map_err(AppError::Internal)
    }

    pub async fn record_heartbeat(&self, hex_id: &str, request: &ServerHeartbeatRequest) -> Result<ServerHeartbeatResponse, AppError> {
        let server = self.get_server_internal(hex_id).await?;
        let updated = ServerRepository::new(self.pool).record_heartbeat(server.id, request.k3s_version.as_deref(), request.specs.as_ref(), request.metadata.as_ref()).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)?;
        Ok(ServerHeartbeatResponse { server_id: updated.hex_id, status: updated.status, recorded_at: updated.last_heartbeat_at.unwrap_or(updated.updated_at) })
    }

    pub async fn update_service_statuses(&self, server_hex_id: &str, request: &BulkServiceStatusRequest) -> Result<Vec<ServerServiceResponse>, AppError> {
        let server = self.get_server_internal(server_hex_id).await?;
        let repo = ServerServiceRepository::new(self.pool);
        let mut responses = Vec::with_capacity(request.services.len());
        for r in &request.services {
            let s = repo.upsert_from_report(server.id, &r.service_name, &r.namespace, &r.status, r.replicas_desired, r.replicas_ready, r.image_tag.as_deref()).await.map_err(AppError::Internal)?;
            responses.push(ServerServiceResponse { id: s.hex_id, server_id: server.hex_id.clone(), service_name: s.service_name, namespace: s.namespace, status: s.status, replicas_desired: s.replicas_desired, replicas_ready: s.replicas_ready, image_tag: s.image_tag, health_endpoint: s.health_endpoint, last_deployed_at: s.last_deployed_at, last_health_check_at: s.last_health_check_at, created_at: s.created_at, updated_at: s.updated_at });
        }
        Ok(responses)
    }

    pub async fn create_service(&self, server_hex_id: &str, service_name: &str, namespace: &str, replicas_desired: Option<i32>, image_tag: Option<&str>, health_endpoint: Option<&str>, metadata: Option<&serde_json::Value>) -> Result<ServerServiceResponse, AppError> {
        let server = self.get_server_internal(server_hex_id).await?;
        let repo = ServerServiceRepository::new(self.pool);
        if repo.find_by_server_namespace_name(server.id, namespace, service_name).await?.is_some() { return Err(AppError::Conflict(format!("Service '{}/{}' exists", namespace, service_name))); }
        let s = repo.create(server.id, service_name, namespace, replicas_desired, image_tag, health_endpoint, metadata).await.map_err(AppError::Internal)?;
        Ok(ServerServiceResponse { id: s.hex_id, server_id: server.hex_id, service_name: s.service_name, namespace: s.namespace, status: s.status, replicas_desired: s.replicas_desired, replicas_ready: s.replicas_ready, image_tag: s.image_tag, health_endpoint: s.health_endpoint, last_deployed_at: s.last_deployed_at, last_health_check_at: s.last_health_check_at, created_at: s.created_at, updated_at: s.updated_at })
    }

    pub async fn get_service(&self, hex_id: &str) -> Result<ServerServiceResponse, AppError> {
        let s = ServerServiceRepository::new(self.pool).find_by_hex_id_with_server(hex_id).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)?;
        Ok(ServerServiceResponse::from(s))
    }

    pub async fn update_service(&self, hex_id: &str, status: Option<&ServiceStatus>, replicas_desired: Option<i32>, replicas_ready: Option<i32>, image_tag: Option<&str>, health_endpoint: Option<&str>, metadata: Option<&serde_json::Value>) -> Result<ServerServiceResponse, AppError> {
        let repo = ServerServiceRepository::new(self.pool);
        let existing = repo.find_by_hex_id_with_server(hex_id).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)?;
        let s = repo.update(existing.id, status, replicas_desired, replicas_ready, image_tag, health_endpoint, metadata).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)?;
        Ok(ServerServiceResponse { id: s.hex_id, server_id: existing.server_hex_id, service_name: s.service_name, namespace: s.namespace, status: s.status, replicas_desired: s.replicas_desired, replicas_ready: s.replicas_ready, image_tag: s.image_tag, health_endpoint: s.health_endpoint, last_deployed_at: s.last_deployed_at, last_health_check_at: s.last_health_check_at, created_at: s.created_at, updated_at: s.updated_at })
    }

    pub async fn delete_service(&self, hex_id: &str) -> Result<(), AppError> {
        let repo = ServerServiceRepository::new(self.pool);
        let s = repo.find_by_hex_id(hex_id).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)?;
        repo.delete(s.id).await.map_err(AppError::Internal)?;
        Ok(())
    }

    pub async fn list_services_by_server(&self, server_hex_id: &str) -> Result<Vec<ServerServiceResponse>, AppError> {
        let server = self.get_server_internal(server_hex_id).await?;
        let services = ServerServiceRepository::new(self.pool).list_by_server_with_hex_id(server.id).await.map_err(AppError::Internal)?;
        Ok(services.into_iter().map(ServerServiceResponse::from).collect())
    }

    pub async fn list_services_by_namespace(&self, namespace: &str) -> Result<Vec<ServerServiceResponse>, AppError> {
        let services = ServerServiceRepository::new(self.pool).list_by_namespace(namespace).await.map_err(AppError::Internal)?;
        Ok(services.into_iter().map(ServerServiceResponse::from).collect())
    }

    pub async fn list_services(&self, limit: i64, offset: i64) -> Result<Vec<ServerServiceResponse>, AppError> {
        let services = ServerServiceRepository::new(self.pool).list(limit, offset).await.map_err(AppError::Internal)?;
        Ok(services.into_iter().map(ServerServiceResponse::from).collect())
    }

    pub async fn count_services(&self) -> Result<i64, AppError> { ServerServiceRepository::new(self.pool).count().await.map_err(AppError::Internal) }

    pub async fn find_unhealthy_services(&self) -> Result<Vec<ServerServiceResponse>, AppError> {
        let services = ServerServiceRepository::new(self.pool).find_unhealthy_services().await.map_err(AppError::Internal)?;
        Ok(services.into_iter().map(ServerServiceResponse::from).collect())
    }
}
