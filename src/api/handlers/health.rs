//! Health check handlers

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis: Option<&'static str>,
}

/// Basic health check - always returns OK if server is running
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        database: None,
        redis: None,
    })
}

/// Liveness probe - indicates if the service is alive
pub async fn liveness_check() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "alive",
            version: env!("CARGO_PKG_VERSION"),
            database: None,
            redis: None,
        }),
    )
}

/// Readiness probe - checks if the service can handle requests
pub async fn readiness_check(
    State(state): State<AppState>,
) -> (StatusCode, Json<HealthResponse>) {
    let db_ok = state.db.health_check().await.unwrap_or(false);
    let redis_ok = state.redis.health_check().await.unwrap_or(false);

    let status = if db_ok && redis_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(HealthResponse {
            status: if db_ok && redis_ok { "ready" } else { "not_ready" },
            version: env!("CARGO_PKG_VERSION"),
            database: Some(if db_ok { "ok" } else { "error" }),
            redis: Some(if redis_ok { "ok" } else { "error" }),
        }),
    )
}
