//! Server model
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use super::hex_id::HexId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "server_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus { Provisioning, Active, Degraded, Offline, Maintenance, Decommissioned }
impl Default for ServerStatus { fn default() -> Self { Self::Provisioning } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "server_environment", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ServerEnvironment { Prod, Testnet, Staging, Dev }
impl Default for ServerEnvironment { fn default() -> Self { Self::Dev } }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerSpecs {
    #[serde(skip_serializing_if = "Option::is_none")] pub cpu_cores: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")] pub ram_gb: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")] pub disk_gb: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")] pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub arch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Server {
    pub id: Uuid, pub hex_id: String, pub name: String, pub hostname: String, pub ip_address: String,
    pub environment: ServerEnvironment, pub status: ServerStatus, pub k3s_version: Option<String>,
    pub specs: serde_json::Value, pub metadata: serde_json::Value, pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>, pub deleted_at: Option<DateTime<Utc>>,
}
impl HexId for Server { const PREFIX: &'static str = "srv"; }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    pub id: String, pub name: String, pub hostname: String, pub ip_address: String,
    pub environment: ServerEnvironment, pub status: ServerStatus, pub k3s_version: Option<String>,
    pub specs: serde_json::Value, pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
}
impl From<Server> for ServerResponse {
    fn from(s: Server) -> Self {
        Self { id: s.hex_id, name: s.name, hostname: s.hostname, ip_address: s.ip_address,
            environment: s.environment, status: s.status, k3s_version: s.k3s_version,
            specs: s.specs, last_heartbeat_at: s.last_heartbeat_at, created_at: s.created_at, updated_at: s.updated_at }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateServerRequest {
    pub name: String, pub hostname: String, pub ip_address: String, pub environment: ServerEnvironment,
    #[serde(default)] pub k3s_version: Option<String>, #[serde(default)] pub specs: Option<ServerSpecs>,
    #[serde(default)] pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateServerRequest {
    #[serde(default)] pub name: Option<String>, #[serde(default)] pub hostname: Option<String>,
    #[serde(default)] pub ip_address: Option<String>, #[serde(default)] pub environment: Option<ServerEnvironment>,
    #[serde(default)] pub status: Option<ServerStatus>, #[serde(default)] pub k3s_version: Option<String>,
    #[serde(default)] pub specs: Option<ServerSpecs>, #[serde(default)] pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerHeartbeatRequest {
    #[serde(default)] pub k3s_version: Option<String>, #[serde(default)] pub specs: Option<ServerSpecs>,
    #[serde(default)] pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerHeartbeatResponse { pub server_id: String, pub status: ServerStatus, pub recorded_at: DateTime<Utc> }
