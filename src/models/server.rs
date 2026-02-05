//! Server model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// Server environment enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "server_environment", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ServerEnvironment {
    Prod,
    Testnet,
    Staging,
    Dev,
}

impl Default for ServerEnvironment {
    fn default() -> Self {
        Self::Dev
    }
}

/// Server status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "server_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Provisioning,
    Active,
    Degraded,
    Maintenance,
    Offline,
    Decommissioned,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self::Provisioning
    }
}

/// Server hardware specifications
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerSpecs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_cores: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_gb: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_gb: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// Server model - represents a physical or virtual server
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Server {
    pub id: Uuid,
    pub hex_id: String,
    pub name: String,
    pub hostname: String,
    pub ip_address: String,  // Stored as string for SQLx compatibility
    pub environment: ServerEnvironment,
    pub status: ServerStatus,
    pub k3s_version: Option<String>,
    pub specs: serde_json::Value,
    pub ssh_user: Option<String>,
    pub ssh_port: Option<i32>,
    pub notes: Option<String>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Server hex_id prefix
impl HexId for Server {
    const PREFIX: &'static str = "srv";
}

/// Server API response (excludes sensitive fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    pub id: String,  // hex_id
    pub name: String,
    pub hostname: String,
    pub ip_address: String,
    pub environment: ServerEnvironment,
    pub status: ServerStatus,
    pub k3s_version: Option<String>,
    pub specs: ServerSpecs,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Server> for ServerResponse {
    fn from(server: Server) -> Self {
        let specs: ServerSpecs = serde_json::from_value(server.specs.clone())
            .unwrap_or_default();
        
        Self {
            id: server.hex_id,
            name: server.name,
            hostname: server.hostname,
            ip_address: server.ip_address,
            environment: server.environment,
            status: server.status,
            k3s_version: server.k3s_version,
            specs,
            last_heartbeat_at: server.last_heartbeat_at,
            created_at: server.created_at,
            updated_at: server.updated_at,
        }
    }
}

/// Request to create a new server
#[derive(Debug, Clone, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub hostname: String,
    pub ip_address: String,
    #[serde(default)]
    pub environment: ServerEnvironment,
    pub k3s_version: Option<String>,
    #[serde(default)]
    pub specs: ServerSpecs,
    pub ssh_user: Option<String>,
    pub ssh_port: Option<i32>,
    pub notes: Option<String>,
}

/// Request to update a server
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub ip_address: Option<String>,
    pub environment: Option<ServerEnvironment>,
    pub status: Option<ServerStatus>,
    pub k3s_version: Option<String>,
    pub specs: Option<ServerSpecs>,
    pub ssh_user: Option<String>,
    pub ssh_port: Option<i32>,
    pub notes: Option<String>,
}

/// Heartbeat request from a server
#[derive(Debug, Clone, Deserialize)]
pub struct ServerHeartbeatRequest {
    pub k3s_version: Option<String>,
    pub status: Option<ServerStatus>,
    #[serde(default)]
    pub specs: Option<ServerSpecs>,
}

/// Heartbeat response
#[derive(Debug, Clone, Serialize)]
pub struct ServerHeartbeatResponse {
    pub acknowledged: bool,
    pub server_time: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_hex_id_prefix() {
        let hex_id = Server::generate_hex_id();
        assert!(hex_id.starts_with("srv_"));
        assert_eq!(hex_id.len(), 14); // "srv_" (4) + 10 random chars
    }

    #[test]
    fn test_server_environment_default() {
        let env = ServerEnvironment::default();
        assert_eq!(env, ServerEnvironment::Dev);
    }

    #[test]
    fn test_server_status_default() {
        let status = ServerStatus::default();
        assert_eq!(status, ServerStatus::Provisioning);
    }

    #[test]
    fn test_server_specs_serialization() {
        let specs = ServerSpecs {
            cpu_cores: Some(4),
            ram_gb: Some(16),
            disk_gb: Some(100),
            provider: Some("hetzner".to_string()),
            region: Some("eu-central".to_string()),
        };

        let json = serde_json::to_value(&specs).unwrap();
        assert_eq!(json["cpu_cores"], 4);
        assert_eq!(json["ram_gb"], 16);
        assert_eq!(json["disk_gb"], 100);
        assert_eq!(json["provider"], "hetzner");
        assert_eq!(json["region"], "eu-central");
    }

    #[test]
    fn test_server_specs_empty() {
        let specs = ServerSpecs::default();
        let json = serde_json::to_value(&specs).unwrap();
        // Empty specs should not include null fields (skip_serializing_if)
        assert!(json.get("cpu_cores").is_none());
    }

    #[test]
    fn test_create_server_request_deserialization() {
        let json = r#"{
            "name": "prod-1",
            "hostname": "prod-1.example.com",
            "ip_address": "192.168.1.100",
            "environment": "prod",
            "k3s_version": "v1.28.0",
            "specs": {
                "cpu_cores": 8,
                "ram_gb": 32
            },
            "ssh_user": "manifest",
            "ssh_port": 22,
            "notes": "Primary production server"
        }"#;

        let req: CreateServerRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "prod-1");
        assert_eq!(req.hostname, "prod-1.example.com");
        assert_eq!(req.ip_address, "192.168.1.100");
        assert_eq!(req.environment, ServerEnvironment::Prod);
        assert_eq!(req.k3s_version, Some("v1.28.0".to_string()));
        assert_eq!(req.specs.cpu_cores, Some(8));
        assert_eq!(req.specs.ram_gb, Some(32));
        assert_eq!(req.ssh_user, Some("manifest".to_string()));
        assert_eq!(req.ssh_port, Some(22));
    }

    #[test]
    fn test_create_server_request_minimal() {
        let json = r#"{
            "name": "dev-1",
            "hostname": "dev-1.local",
            "ip_address": "10.0.0.1"
        }"#;

        let req: CreateServerRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "dev-1");
        assert_eq!(req.environment, ServerEnvironment::Dev); // default
        assert!(req.k3s_version.is_none());
    }
}
