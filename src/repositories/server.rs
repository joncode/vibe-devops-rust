//! Server repository
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{HexId, Server, ServerEnvironment, ServerSpecs, ServerStatus};

pub struct ServerRepository<'a> { pool: &'a PgPool }

impl<'a> ServerRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self { Self { pool } }

    pub async fn create(&self, name: &str, hostname: &str, ip_address: &str, environment: ServerEnvironment, k3s_version: Option<&str>, specs: Option<&ServerSpecs>, metadata: Option<&serde_json::Value>) -> Result<Server> {
        let hex_id = Server::generate_hex_id();
        let specs_json = specs.map(|s| serde_json::to_value(s).unwrap_or_default()).unwrap_or_else(|| serde_json::json!({}));
        let metadata_json = metadata.cloned().unwrap_or_else(|| serde_json::json!({}));
        let server = sqlx::query_as::<_, Server>(
            "INSERT INTO servers (hex_id, name, hostname, ip_address, environment, k3s_version, specs, metadata) VALUES ($1, $2, $3, $4::inet, $5, $6, $7, $8) RETURNING id, hex_id, name, hostname, ip_address::text, environment, status, k3s_version, specs, metadata, last_heartbeat_at, created_at, updated_at, deleted_at")
            .bind(&hex_id).bind(name).bind(hostname).bind(ip_address).bind(&environment).bind(k3s_version).bind(&specs_json).bind(&metadata_json)
            .fetch_one(self.pool).await?;
        Ok(server)
    }

    pub async fn find_by_hex_id(&self, hex_id: &str) -> Result<Option<Server>> {
        Ok(sqlx::query_as::<_, Server>("SELECT id, hex_id, name, hostname, ip_address::text, environment, status, k3s_version, specs, metadata, last_heartbeat_at, created_at, updated_at, deleted_at FROM servers WHERE hex_id = $1 AND deleted_at IS NULL")
            .bind(hex_id).fetch_optional(self.pool).await?)
    }

    pub async fn find_by_hostname(&self, hostname: &str) -> Result<Option<Server>> {
        Ok(sqlx::query_as::<_, Server>("SELECT id, hex_id, name, hostname, ip_address::text, environment, status, k3s_version, specs, metadata, last_heartbeat_at, created_at, updated_at, deleted_at FROM servers WHERE hostname = $1 AND deleted_at IS NULL")
            .bind(hostname).fetch_optional(self.pool).await?)
    }

    pub async fn update(&self, id: Uuid, name: Option<&str>, hostname: Option<&str>, ip_address: Option<&str>, environment: Option<&ServerEnvironment>, status: Option<&ServerStatus>, k3s_version: Option<&str>, specs: Option<&ServerSpecs>, metadata: Option<&serde_json::Value>) -> Result<Option<Server>> {
        let specs_json = specs.map(|s| serde_json::to_value(s).unwrap_or_default());
        Ok(sqlx::query_as::<_, Server>(
            "UPDATE servers SET name = COALESCE($2, name), hostname = COALESCE($3, hostname), ip_address = COALESCE($4::inet, ip_address), environment = COALESCE($5, environment), status = COALESCE($6, status), k3s_version = COALESCE($7, k3s_version), specs = COALESCE($8, specs), metadata = COALESCE($9, metadata), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL RETURNING id, hex_id, name, hostname, ip_address::text, environment, status, k3s_version, specs, metadata, last_heartbeat_at, created_at, updated_at, deleted_at")
            .bind(id).bind(name).bind(hostname).bind(ip_address).bind(environment).bind(status).bind(k3s_version).bind(specs_json).bind(metadata)
            .fetch_optional(self.pool).await?)
    }

    pub async fn record_heartbeat(&self, id: Uuid, k3s_version: Option<&str>, specs: Option<&ServerSpecs>, metadata: Option<&serde_json::Value>) -> Result<Option<Server>> {
        let specs_json = specs.map(|s| serde_json::to_value(s).unwrap_or_default());
        Ok(sqlx::query_as::<_, Server>(
            "UPDATE servers SET last_heartbeat_at = NOW(), status = CASE WHEN status IN ('offline', 'provisioning') THEN 'active'::server_status ELSE status END, k3s_version = COALESCE($2, k3s_version), specs = COALESCE($3, specs), metadata = COALESCE($4, metadata), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL RETURNING id, hex_id, name, hostname, ip_address::text, environment, status, k3s_version, specs, metadata, last_heartbeat_at, created_at, updated_at, deleted_at")
            .bind(id).bind(k3s_version).bind(specs_json).bind(metadata).fetch_optional(self.pool).await?)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        Ok(sqlx::query("UPDATE servers SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL").bind(id).execute(self.pool).await?.rows_affected() > 0)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Server>> {
        Ok(sqlx::query_as::<_, Server>("SELECT id, hex_id, name, hostname, ip_address::text, environment, status, k3s_version, specs, metadata, last_heartbeat_at, created_at, updated_at, deleted_at FROM servers WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(limit).bind(offset).fetch_all(self.pool).await?)
    }

    pub async fn list_by_environment(&self, environment: ServerEnvironment, limit: i64, offset: i64) -> Result<Vec<Server>> {
        Ok(sqlx::query_as::<_, Server>("SELECT id, hex_id, name, hostname, ip_address::text, environment, status, k3s_version, specs, metadata, last_heartbeat_at, created_at, updated_at, deleted_at FROM servers WHERE environment = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind(environment).bind(limit).bind(offset).fetch_all(self.pool).await?)
    }

    pub async fn list_by_status(&self, status: ServerStatus, limit: i64, offset: i64) -> Result<Vec<Server>> {
        Ok(sqlx::query_as::<_, Server>("SELECT id, hex_id, name, hostname, ip_address::text, environment, status, k3s_version, specs, metadata, last_heartbeat_at, created_at, updated_at, deleted_at FROM servers WHERE status = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind(status).bind(limit).bind(offset).fetch_all(self.pool).await?)
    }

    pub async fn count(&self) -> Result<i64> { Ok(sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM servers WHERE deleted_at IS NULL").fetch_one(self.pool).await?.0) }
    pub async fn count_by_environment(&self, environment: ServerEnvironment) -> Result<i64> { Ok(sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM servers WHERE environment = $1 AND deleted_at IS NULL").bind(environment).fetch_one(self.pool).await?.0) }
    pub async fn count_by_status(&self, status: ServerStatus) -> Result<i64> { Ok(sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM servers WHERE status = $1 AND deleted_at IS NULL").bind(status).fetch_one(self.pool).await?.0) }
}
