//! Session token model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// Session token model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SessionToken {
    pub id: Uuid,
    pub hex_id: String,
    pub user_id: Uuid,
    pub token_hash: String,
    pub device_info: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Session token hex_id prefix
impl HexId for SessionToken {
    const PREFIX: &'static str = "stk";
}

/// Session for API responses
/// Uses hex_id as the public identifier instead of internal UUID
#[derive(Debug, Clone, Serialize)]
pub struct SessionResponse {
    pub id: String,  // hex_id, not UUID
    pub device_info: serde_json::Value,
    pub ip_address: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_current: bool,
}

impl SessionToken {
    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none() && 
        self.deleted_at.is_none() && 
        self.expires_at > Utc::now()
    }
}

/// Refresh token model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub hex_id: String,
    pub user_id: Uuid,
    pub token_hash: String,
    pub session_id: Option<Uuid>,
    pub family_id: Uuid,
    pub generation: i32,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Refresh token hex_id prefix
impl HexId for RefreshToken {
    const PREFIX: &'static str = "rtk";
}

impl RefreshToken {
    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none() && 
        self.deleted_at.is_none() && 
        self.expires_at > Utc::now()
    }
}
