//! User model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// User status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Pending,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// App user model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub hex_id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: serde_json::Value,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// User hex_id prefix
impl HexId for User {
    const PREFIX: &'static str = "usr";
}

/// User for API responses (excludes sensitive fields)
/// Uses hex_id as the public identifier instead of internal UUID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,  // hex_id, not UUID
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.hex_id,  // expose hex_id as "id" in API
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            status: user.status,
            created_at: user.created_at,
        }
    }
}

/// Request to create a new user
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

/// Request to update a user
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
