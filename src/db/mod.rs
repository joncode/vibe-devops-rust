//! Database module - PostgreSQL connection and migrations

use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

use crate::config::DatabaseConfig;

/// Database connection wrapper
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn connect(config: &DatabaseConfig) -> Result<Self> {
        info!("Connecting to database...");
        
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(&config.url)
            .await?;

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        
        // Run migrations from the migrations directory
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;

        Ok(())
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<bool> {
        let result: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(result.0 == 1)
    }
}
