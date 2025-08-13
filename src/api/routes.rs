use axum::{
    Router,
    routing::{get, post},
    extract::State,
};
use crate::SelemeneEngine;
use std::sync::Arc;

/// Create v1 API routes
pub fn create_v1_routes(engine: Arc<SelemeneEngine>) -> Router {
    Router::new()
        .route("/panchanga", post(handlers::calculate_panchanga))
        .route("/panchanga/batch", post(handlers::calculate_batch_panchanga))
        .route("/panchanga/range", post(handlers::calculate_panchanga_range))
        .route("/solar/position", post(handlers::calculate_solar_position))
        .route("/lunar/position", post(handlers::calculate_lunar_position))
        .route("/tithi", post(handlers::calculate_tithi))
        .route("/nakshatra", post(handlers::calculate_nakshatra))
        .route("/yoga", post(handlers::calculate_yoga))
        .route("/karana", post(handlers::calculate_karana))
        .route("/vara", post(handlers::calculate_vara))
        .route("/houses", post(handlers::calculate_houses))
        .route("/planets", post(handlers::calculate_planetary_positions))
        .route("/cache/stats", get(handlers::get_cache_stats))
        .route("/cache/clear", post(handlers::clear_cache))
                        .route("/engine/stats", get(handlers::get_engine_stats))
                .route("/engine/config", get(handlers::get_engine_config))
                .route("/engine/config", post(handlers::update_engine_config))
                .route("/performance/optimize", post(handlers::optimize_performance))
                .route("/performance/benchmark", post(handlers::run_benchmarks))
                .with_state(engine)
}

/// Create admin routes (protected)
pub fn create_admin_routes(engine: Arc<SelemeneEngine>) -> Router {
    Router::new()
        .route("/admin/users", get(handlers::list_users))
        .route("/admin/users", post(handlers::create_user))
        .route("/admin/users/:id", get(handlers::get_user))
        .route("/admin/users/:id", put(handlers::update_user))
        .route("/admin/users/:id", delete(handlers::delete_user))
        .route("/admin/analytics", get(handlers::get_analytics))
        .route("/admin/maintenance", post(handlers::trigger_maintenance))
        .with_state(engine)
}

/// Create WebSocket routes for real-time updates
pub fn create_websocket_routes(engine: Arc<SelemeneEngine>) -> Router {
    Router::new()
        .route("/ws/panchanga", get(handlers::panchanga_websocket))
        .route("/ws/notifications", get(handlers::notifications_websocket))
        .with_state(engine)
}
