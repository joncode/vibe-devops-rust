//! Social identifier model (email, phone, wallet, OAuth)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

/// Identifier type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "identifier_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IdentifierType {
    Email,
    Phone,
    Wallet,
    OauthGoogle,
    OauthGithub,
    OauthApple,
    Username,
}

/// Social identifier model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SocialIdentifier {
    pub id: Uuid,
    pub hex_id: String,
    pub user_id: Uuid,
    pub identifier_type: IdentifierType,
    pub identifier_value: String,
    pub verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub is_primary: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Social identifier hex_id prefix
impl HexId for SocialIdentifier {
    const PREFIX: &'static str = "sid";
}

/// Identifier for API responses
/// Uses hex_id as the public identifier instead of internal UUID
#[derive(Debug, Clone, Serialize)]
pub struct IdentifierResponse {
    pub id: String,  // hex_id, not UUID
    pub identifier_type: IdentifierType,
    pub identifier_value: String,
    pub verified: bool,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

impl From<SocialIdentifier> for IdentifierResponse {
    fn from(identifier: SocialIdentifier) -> Self {
        Self {
            id: identifier.hex_id,  // expose hex_id as "id" in API
            identifier_type: identifier.identifier_type,
            identifier_value: mask_identifier(&identifier.identifier_value, &identifier.identifier_type),
            verified: identifier.verified,
            is_primary: identifier.is_primary,
            created_at: identifier.created_at,
        }
    }
}

/// Mask sensitive parts of identifiers
fn mask_identifier(value: &str, id_type: &IdentifierType) -> String {
    match id_type {
        IdentifierType::Email => {
            if let Some(at_pos) = value.find('@') {
                let (local, domain) = value.split_at(at_pos);
                if local.len() > 2 {
                    format!("{}***{}", &local[0..2], domain)
                } else {
                    format!("***{}", domain)
                }
            } else {
                "***".to_string()
            }
        }
        IdentifierType::Phone => {
            if value.len() > 4 {
                format!("***{}", &value[value.len()-4..])
            } else {
                "***".to_string()
            }
        }
        IdentifierType::Wallet => {
            if value.len() > 10 {
                format!("{}...{}", &value[0..6], &value[value.len()-4..])
            } else {
                value.to_string()
            }
        }
        _ => value.to_string(),
    }
}

/// Password hash stored alongside user
#[derive(Debug, Clone, FromRow)]
pub struct UserPassword {
    pub user_id: Uuid,
    pub hex_id: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// User password hex_id prefix
impl HexId for UserPassword {
    const PREFIX: &'static str = "pwd";
}

/// Legacy alias for backward compatibility
pub type UserCredentials = UserPassword;
