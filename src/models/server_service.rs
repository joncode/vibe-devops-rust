//! ServerService model
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use super::hex_id::HexId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "service_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus { Unknown, Running, Degraded, Stopped, Failed, Deploying, Scaling }
impl Default for ServiceStatus { fn default() -> Self { Self::Unknown } }

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServerService {
    pub id: Uuid, pub hex_id: String, pub server_id: Uuid, pub service_name: String, pub namespace: String,
    pub status: ServiceStatus, pub replicas_desired: Option<i32>, pub replicas_ready: Option<i32>,
    pub image_tag: Option<String>, pub health_endpoint: Option<String>, pub metadata: serde_json::Value,
    pub last_deployed_at: Option<DateTime<Utc>>, pub last_health_check_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>, pub deleted_at: Option<DateTime<Utc>>,
}
impl HexId for ServerService { const PREFIX: &'static str = "svc"; }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerServiceResponse {
    pub id: String, pub server_id: String, pub service_name: String, pub namespace: String,
    pub status: ServiceStatus, pub replicas_desired: Option<i32>, pub replicas_ready: Option<i32>,
    pub image_tag: Option<String>, pub health_endpoint: Option<String>,
    pub last_deployed_at: Option<DateTime<Utc>>, pub last_health_check_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServerServiceWithServerHexId {
    pub id: Uuid, pub hex_id: String, pub server_id: Uuid, pub server_hex_id: String,
    pub service_name: String, pub namespace: String, pub status: ServiceStatus,
    pub replicas_desired: Option<i32>, pub replicas_ready: Option<i32>, pub image_tag: Option<String>,
    pub health_endpoint: Option<String>, pub metadata: serde_json::Value,
    pub last_deployed_at: Option<DateTime<Utc>>, pub last_health_check_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>, pub deleted_at: Option<DateTime<Utc>>,
}
impl From<ServerServiceWithServerHexId> for ServerServiceResponse {
    fn from(s: ServerServiceWithServerHexId) -> Self {
        Self { id: s.hex_id, server_id: s.server_hex_id, service_name: s.service_name, namespace: s.namespace,
            status: s.status, replicas_desired: s.replicas_desired, replicas_ready: s.replicas_ready,
            image_tag: s.image_tag, health_endpoint: s.health_endpoint, last_deployed_at: s.last_deployed_at,
            last_health_check_at: s.last_health_check_at, created_at: s.created_at, updated_at: s.updated_at }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateServerServiceRequest {
    pub server_id: String, pub service_name: String, pub namespace: String,
    #[serde(default)] pub replicas_desired: Option<i32>, #[serde(default)] pub image_tag: Option<String>,
    #[serde(default)] pub health_endpoint: Option<String>, #[serde(default)] pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateServerServiceRequest {
    #[serde(default)] pub status: Option<ServiceStatus>, #[serde(default)] pub replicas_desired: Option<i32>,
    #[serde(default)] pub replicas_ready: Option<i32>, #[serde(default)] pub image_tag: Option<String>,
    #[serde(default)] pub health_endpoint: Option<String>, #[serde(default)] pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceStatusReport {
    pub service_name: String, pub namespace: String, pub status: ServiceStatus,
    #[serde(default)] pub replicas_desired: Option<i32>, #[serde(default)] pub replicas_ready: Option<i32>,
    #[serde(default)] pub image_tag: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkServiceStatusRequest { pub services: Vec<ServiceStatusReport> }
