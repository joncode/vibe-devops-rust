use axum::{routing::{get, post}, Router};
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};
use crate::AppState;
use super::handlers::{admin, agent, auth, health, user};

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let api_v1 = Router::new()
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .route("/users/me", get(user::get_me).put(user::update_me).delete(user::delete_me))
        .route("/users/me/sessions", get(user::get_sessions))
        .route("/agent/deploy", post(agent::deploy))
        .route("/agent/tasks", get(agent::list_tasks))
        .route("/agent/tools", get(agent::list_tools))
        .route("/agent/status/{task_id}", get(agent::get_status))
        .route("/agent/logs/{task_id}", get(agent::get_logs))
        .route("/agent/tasks/{task_id}/cancel", post(agent::cancel_task))
        .route("/agent/tasks/{task_id}/execute", post(agent::execute_tool))
        .route("/agent/tasks/{task_id}/complete", post(agent::complete_task))
        .route("/agent/tasks/{task_id}/fail", post(agent::fail_task));
    let admin_api = Router::new()
        .route("/stats", get(admin::get_stats))
        .route("/users", get(admin::list_users))
        .route("/social-identifiers", get(admin::list_social_identifiers))
        .route("/user-passwords", get(admin::list_user_passwords))
        .route("/session-tokens", get(admin::list_session_tokens))
        .route("/refresh-tokens", get(admin::list_refresh_tokens));
    Router::new()
        .route("/admin", get(admin::admin_panel))
        .nest("/api/v1", api_v1)
        .nest("/api/admin", admin_api)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
