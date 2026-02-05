//! Application configuration module

use anyhow::Result;
use serde::Deserialize;
use std::env;

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
}

/// HTTP server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
}

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

/// Redis configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub key_prefix: String,
}

/// Authentication configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub refresh_token_expiry_days: i64,
    pub session_expiry_days: i64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            environment: "development".to_string(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://postgres:postgres@localhost:5432/vibe_devops".to_string(),
            max_connections: 10,
            min_connections: 2,
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            key_prefix: "vibe:dev".to_string(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "CHANGE_ME_IN_PRODUCTION_32_CHARS_MIN".to_string(),
            jwt_expiry_hours: 1,
            refresh_token_expiry_days: 7,
            session_expiry_days: 30,
        }
    }
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn load() -> Result<Self> {
        let server = ServerConfig {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        };

        let database = DatabaseConfig {
            url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| DatabaseConfig::default().url),
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .unwrap_or(2),
        };

        let redis = RedisConfig {
            url: env::var("REDIS_URL")
                .unwrap_or_else(|_| RedisConfig::default().url),
            key_prefix: env::var("REDIS_KEY_PREFIX")
                .unwrap_or_else(|_| format!("vibe:{}", server.environment)),
        };

        let auth = AuthConfig {
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| AuthConfig::default().jwt_secret),
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            refresh_token_expiry_days: env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .unwrap_or(7),
            session_expiry_days: env::var("SESSION_EXPIRY_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        };

        Ok(Self {
            server,
            database,
            redis,
            auth,
        })
    }
}
