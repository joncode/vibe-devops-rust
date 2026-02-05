//! Social identifier model (email, phone, wallet, OAuth)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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
    pub user_id: Uuid,
    pub identifier_type: IdentifierType,
    pub identifier_value: String,
    pub verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub is_primary: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Identifier for API responses
#[derive(Debug, Clone, Serialize)]
pub struct IdentifierResponse {
    pub id: Uuid,
    pub identifier_type: IdentifierType,
    pub identifier_value: String,
    pub verified: bool,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

impl From<SocialIdentifier> for IdentifierResponse {
    fn from(id: SocialIdentifier) -> Self {
        Self {
            id: id.id,
            identifier_type: id.identifier_type,
            identifier_value: mask_identifier(&id.identifier_value, &id.identifier_type),
            verified: id.verified,
            is_primary: id.is_primary,
            created_at: id.created_at,
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

/// Password hash stored alongside email identifier
#[derive(Debug, Clone, FromRow)]
pub struct UserCredentials {
    pub user_id: Uuid,
    pub identifier_id: Uuid,
    pub password_hash: String,
}
