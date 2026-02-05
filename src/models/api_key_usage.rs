//! API Key Usage model for tracking service account activity
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use super::hex_id::HexId;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKeyUsage {
    pub id: Uuid,
    pub hex_id: String,
    pub service_account_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: Option<i32>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl HexId for ApiKeyUsage {
    const PREFIX: &'static str = "aku";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_requests: i64,
    pub requests_last_hour: i64,
    pub requests_last_24h: i64,
    pub avg_response_time_ms: Option<f64>,
    pub error_rate: f64,
    pub most_used_endpoints: Vec<EndpointStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStats {
    pub endpoint: String,
    pub method: String,
    pub request_count: i64,
    pub avg_response_time_ms: Option<f64>,
    pub error_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_id() {
        assert!(ApiKeyUsage::generate_hex_id().starts_with("aku_"));
    }
}
