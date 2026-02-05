//! Application model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// App model - represents a deployable application
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct App {
    pub id: Uuid,
    pub hex_id: String,
    pub name: String,
    pub repo_url: Option<String>,
    pub default_stack: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// App hex_id prefix
impl HexId for App {
    const PREFIX: &'static str = "app";
}

/// App for API responses (excludes internal fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppResponse {
    pub id: String, // hex_id
    pub name: String,
    pub repo_url: Option<String>,
    pub default_stack: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<App> for AppResponse {
    fn from(app: App) -> Self {
        Self {
            id: app.hex_id,
            name: app.name,
            repo_url: app.repo_url,
            default_stack: app.default_stack,
            created_at: app.created_at,
            updated_at: app.updated_at,
        }
    }
}

/// Request to create a new app
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAppRequest {
    pub name: String,
    pub repo_url: Option<String>,
    pub default_stack: Option<String>,
}

/// Request to update an app
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAppRequest {
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub default_stack: Option<String>,
}
