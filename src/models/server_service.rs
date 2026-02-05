//! Server service model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// Service status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "service_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Unknown,
    Running,
    Degraded,
    Stopped,
    Failed,
    Pending,
}

impl Default for ServiceStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Server service model - represents a service deployed on a server
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServerService {
    pub id: Uuid,
    pub hex_id: String,
    pub server_id: Uuid,
    pub service_name: String,
    pub namespace: String,
    pub status: ServiceStatus,
    pub replicas_desired: Option<i32>,
    pub replicas_ready: Option<i32>,
    pub image_tag: Option<String>,
    pub port: Option<i32>,
    pub health_endpoint: Option<String>,
    pub metadata: serde_json::Value,
    pub last_deployed_at: Option<DateTime<Utc>>,
    pub last_health_check_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Server service hex_id prefix
impl HexId for ServerService {
    const PREFIX: &'static str = "svc";
}

/// Server service API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerServiceResponse {
    pub id: String,  // hex_id
    pub server_id: String,  // server's hex_id
    pub service_name: String,
    pub namespace: String,
    pub status: ServiceStatus,
    pub replicas_desired: Option<i32>,
    pub replicas_ready: Option<i32>,
    pub image_tag: Option<String>,
    pub port: Option<i32>,
    pub health_endpoint: Option<String>,
    pub last_deployed_at: Option<DateTime<Utc>>,
    pub last_health_check_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ServerServiceResponse {
    /// Create response with server's hex_id
    pub fn from_service_with_server_hex_id(service: ServerService, server_hex_id: String) -> Self {
        Self {
            id: service.hex_id,
            server_id: server_hex_id,
            service_name: service.service_name,
            namespace: service.namespace,
            status: service.status,
            replicas_desired: service.replicas_desired,
            replicas_ready: service.replicas_ready,
            image_tag: service.image_tag,
            port: service.port,
            health_endpoint: service.health_endpoint,
            last_deployed_at: service.last_deployed_at,
            last_health_check_at: service.last_health_check_at,
            created_at: service.created_at,
        }
    }
}

/// Request to create a new server service
#[derive(Debug, Clone, Deserialize)]
pub struct CreateServerServiceRequest {
    pub service_name: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub replicas_desired: Option<i32>,
    pub image_tag: Option<String>,
    pub port: Option<i32>,
    pub health_endpoint: Option<String>,
}

fn default_namespace() -> String {
    "default".to_string()
}

/// Request to update a server service
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateServerServiceRequest {
    pub status: Option<ServiceStatus>,
    pub replicas_desired: Option<i32>,
    pub replicas_ready: Option<i32>,
    pub image_tag: Option<String>,
    pub port: Option<i32>,
    pub health_endpoint: Option<String>,
}

/// Request to report service status (from server heartbeat)
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceStatusReport {
    pub service_name: String,
    pub namespace: String,
    pub status: ServiceStatus,
    pub replicas_ready: Option<i32>,
    pub image_tag: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_service_hex_id_prefix() {
        let hex_id = ServerService::generate_hex_id();
        assert!(hex_id.starts_with("svc_"));
        assert_eq!(hex_id.len(), 14); // "svc_" (4) + 10 random chars
    }

    #[test]
    fn test_service_status_default() {
        let status = ServiceStatus::default();
        assert_eq!(status, ServiceStatus::Unknown);
    }

    #[test]
    fn test_create_service_request_deserialization() {
        let json = r#"{
            "service_name": "manifest-api",
            "namespace": "manifest",
            "replicas_desired": 3,
            "image_tag": "v1.0.0",
            "port": 8080,
            "health_endpoint": "/health"
        }"#;

        let req: CreateServerServiceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.service_name, "manifest-api");
        assert_eq!(req.namespace, "manifest");
        assert_eq!(req.replicas_desired, Some(3));
        assert_eq!(req.image_tag, Some("v1.0.0".to_string()));
        assert_eq!(req.port, Some(8080));
        assert_eq!(req.health_endpoint, Some("/health".to_string()));
    }

    #[test]
    fn test_create_service_request_minimal() {
        let json = r#"{
            "service_name": "my-service"
        }"#;

        let req: CreateServerServiceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.service_name, "my-service");
        assert_eq!(req.namespace, "default"); // default
        assert!(req.replicas_desired.is_none());
        assert!(req.image_tag.is_none());
    }

    #[test]
    fn test_service_status_report_deserialization() {
        let json = r#"{
            "service_name": "web",
            "namespace": "production",
            "status": "running",
            "replicas_ready": 2,
            "image_tag": "v2.0.0"
        }"#;

        let report: ServiceStatusReport = serde_json::from_str(json).unwrap();
        assert_eq!(report.service_name, "web");
        assert_eq!(report.namespace, "production");
        assert_eq!(report.status, ServiceStatus::Running);
        assert_eq!(report.replicas_ready, Some(2));
    }

    #[test]
    fn test_service_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ServiceStatus::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&ServiceStatus::Failed).unwrap(),
            "\"failed\""
        );
        assert_eq!(
            serde_json::to_string(&ServiceStatus::Pending).unwrap(),
            "\"pending\""
        );
    }
}
