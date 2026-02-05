//! Main application router

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::AppState;
use super::handlers::{admin, auth, health, server, user};

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API v1 routes
    let api_v1 = Router::new()
        // Health
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        // Auth
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        // Users
        .route("/users/me", get(user::get_me).put(user::update_me).delete(user::delete_me))
        .route("/users/me/sessions", get(user::get_sessions))
        // Servers
        .route("/servers", get(server::list_servers).post(server::create_server))
        .route("/servers/{id}", get(server::get_server).put(server::update_server).delete(server::delete_server))
        .route("/servers/{id}/heartbeat", post(server::server_heartbeat))
        .route("/servers/{id}/services", get(server::list_server_services).post(server::create_service))
        .route("/servers/{id}/services/status", post(server::update_service_statuses))
        // Services (global)
        .route("/services", get(server::list_services))
        .route("/services/unhealthy", get(server::list_unhealthy_services))
        .route("/services/{id}", get(server::get_service).put(server::update_service).delete(server::delete_service));

    // Admin API routes
    let admin_api = Router::new()
        .route("/stats", get(admin::get_stats))
        .route("/users", get(admin::list_users))
        .route("/social-identifiers", get(admin::list_social_identifiers))
        .route("/user-passwords", get(admin::list_user_passwords))
        .route("/session-tokens", get(admin::list_session_tokens))
        .route("/refresh-tokens", get(admin::list_refresh_tokens));

    // Combine all routes
    Router::new()
        .route("/admin", get(admin::admin_panel))
        .nest("/api/v1", api_v1)
        .nest("/api/admin", admin_api)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
