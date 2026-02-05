//! ServerService repository
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{HexId, ServerService, ServerServiceWithServerHexId, ServiceStatus};

pub struct ServerServiceRepository<'a> { pool: &'a PgPool }

impl<'a> ServerServiceRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self { Self { pool } }

    pub async fn create(&self, server_id: Uuid, service_name: &str, namespace: &str, replicas_desired: Option<i32>, image_tag: Option<&str>, health_endpoint: Option<&str>, metadata: Option<&serde_json::Value>) -> Result<ServerService> {
        let hex_id = ServerService::generate_hex_id();
        let metadata_json = metadata.cloned().unwrap_or_else(|| serde_json::json!({}));
        Ok(sqlx::query_as::<_, ServerService>("INSERT INTO server_services (hex_id, server_id, service_name, namespace, replicas_desired, image_tag, health_endpoint, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *")
            .bind(&hex_id).bind(server_id).bind(service_name).bind(namespace).bind(replicas_desired).bind(image_tag).bind(health_endpoint).bind(&metadata_json).fetch_one(self.pool).await?)
    }

    pub async fn find_by_hex_id(&self, hex_id: &str) -> Result<Option<ServerService>> {
        Ok(sqlx::query_as::<_, ServerService>("SELECT * FROM server_services WHERE hex_id = $1 AND deleted_at IS NULL").bind(hex_id).fetch_optional(self.pool).await?)
    }

    pub async fn find_by_hex_id_with_server(&self, hex_id: &str) -> Result<Option<ServerServiceWithServerHexId>> {
        Ok(sqlx::query_as::<_, ServerServiceWithServerHexId>("SELECT ss.*, s.hex_id as server_hex_id FROM server_services ss JOIN servers s ON s.id = ss.server_id WHERE ss.hex_id = $1 AND ss.deleted_at IS NULL")
            .bind(hex_id).fetch_optional(self.pool).await?)
    }

    pub async fn find_by_server_namespace_name(&self, server_id: Uuid, namespace: &str, service_name: &str) -> Result<Option<ServerService>> {
        Ok(sqlx::query_as::<_, ServerService>("SELECT * FROM server_services WHERE server_id = $1 AND namespace = $2 AND service_name = $3 AND deleted_at IS NULL")
            .bind(server_id).bind(namespace).bind(service_name).fetch_optional(self.pool).await?)
    }

    pub async fn update(&self, id: Uuid, status: Option<&ServiceStatus>, replicas_desired: Option<i32>, replicas_ready: Option<i32>, image_tag: Option<&str>, health_endpoint: Option<&str>, metadata: Option<&serde_json::Value>) -> Result<Option<ServerService>> {
        Ok(sqlx::query_as::<_, ServerService>("UPDATE server_services SET status = COALESCE($2, status), replicas_desired = COALESCE($3, replicas_desired), replicas_ready = COALESCE($4, replicas_ready), image_tag = COALESCE($5, image_tag), health_endpoint = COALESCE($6, health_endpoint), metadata = COALESCE($7, metadata), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL RETURNING *")
            .bind(id).bind(status).bind(replicas_desired).bind(replicas_ready).bind(image_tag).bind(health_endpoint).bind(metadata).fetch_optional(self.pool).await?)
    }

    pub async fn upsert_from_report(&self, server_id: Uuid, service_name: &str, namespace: &str, status: &ServiceStatus, replicas_desired: Option<i32>, replicas_ready: Option<i32>, image_tag: Option<&str>) -> Result<ServerService> {
        let hex_id = ServerService::generate_hex_id();
        Ok(sqlx::query_as::<_, ServerService>("INSERT INTO server_services (hex_id, server_id, service_name, namespace, status, replicas_desired, replicas_ready, image_tag, last_health_check_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW()) ON CONFLICT (server_id, namespace, service_name, deleted_at) WHERE deleted_at IS NULL DO UPDATE SET status = $5, replicas_desired = COALESCE($6, server_services.replicas_desired), replicas_ready = COALESCE($7, server_services.replicas_ready), image_tag = COALESCE($8, server_services.image_tag), last_health_check_at = NOW(), updated_at = NOW() RETURNING *")
            .bind(&hex_id).bind(server_id).bind(service_name).bind(namespace).bind(status).bind(replicas_desired).bind(replicas_ready).bind(image_tag).fetch_one(self.pool).await?)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        Ok(sqlx::query("UPDATE server_services SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL").bind(id).execute(self.pool).await?.rows_affected() > 0)
    }

    pub async fn list_by_server_with_hex_id(&self, server_id: Uuid) -> Result<Vec<ServerServiceWithServerHexId>> {
        Ok(sqlx::query_as::<_, ServerServiceWithServerHexId>("SELECT ss.*, s.hex_id as server_hex_id FROM server_services ss JOIN servers s ON s.id = ss.server_id WHERE ss.server_id = $1 AND ss.deleted_at IS NULL ORDER BY ss.namespace, ss.service_name")
            .bind(server_id).fetch_all(self.pool).await?)
    }

    pub async fn list_by_namespace(&self, namespace: &str) -> Result<Vec<ServerServiceWithServerHexId>> {
        Ok(sqlx::query_as::<_, ServerServiceWithServerHexId>("SELECT ss.*, s.hex_id as server_hex_id FROM server_services ss JOIN servers s ON s.id = ss.server_id WHERE ss.namespace = $1 AND ss.deleted_at IS NULL ORDER BY s.name, ss.service_name")
            .bind(namespace).fetch_all(self.pool).await?)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<ServerServiceWithServerHexId>> {
        Ok(sqlx::query_as::<_, ServerServiceWithServerHexId>("SELECT ss.*, s.hex_id as server_hex_id FROM server_services ss JOIN servers s ON s.id = ss.server_id WHERE ss.deleted_at IS NULL ORDER BY ss.created_at DESC LIMIT $1 OFFSET $2")
            .bind(limit).bind(offset).fetch_all(self.pool).await?)
    }

    pub async fn count(&self) -> Result<i64> { Ok(sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM server_services WHERE deleted_at IS NULL").fetch_one(self.pool).await?.0) }

    pub async fn find_unhealthy_services(&self) -> Result<Vec<ServerServiceWithServerHexId>> {
        Ok(sqlx::query_as::<_, ServerServiceWithServerHexId>("SELECT ss.*, s.hex_id as server_hex_id FROM server_services ss JOIN servers s ON s.id = ss.server_id WHERE ss.deleted_at IS NULL AND (ss.status IN ('failed', 'degraded', 'stopped') OR (ss.replicas_ready IS NOT NULL AND ss.replicas_desired IS NOT NULL AND ss.replicas_ready < ss.replicas_desired)) ORDER BY ss.updated_at DESC")
            .fetch_all(self.pool).await?)
    }
}
