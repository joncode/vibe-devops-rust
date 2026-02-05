//! Vibe DevOps Server - Production-ready backend server in Rust
//!
//! A generic DevOps server providing:
//! - REST API with JWT authentication
//! - PostgreSQL database with migrations
//! - Redis for caching and sessions
//! - Health checks and observability

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod api;
mod config;
mod db;
mod errors;
mod models;
mod redis;
mod repositories;
mod services;
mod utils;

use crate::config::AppConfig;
use crate::db::Database;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: Database,
    pub redis: redis::RedisPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    init_tracing();

    info!("Starting Vibe DevOps Server...");

    // Load configuration
    let config = AppConfig::load()?;
    info!("Configuration loaded");

    // Initialize database connection pool
    let db = Database::connect(&config.database).await?;
    info!("Database connected");

    // Run migrations
    db.migrate().await?;
    info!("Database migrations completed");

    // Initialize Redis connection
    let redis_pool = redis::RedisPool::connect(&config.redis).await?;
    info!("Redis connected");

    // Create application state
    let state = AppState {
        config: Arc::new(config.clone()),
        db,
        redis: redis_pool,
    };

    // Build the application router
    let app = api::router::create_router(state);

    // Start the server
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Invalid server address");

    info!("Server listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true))
        .init();
}
