use axum::{
    Router,
    routing::{get, post, put, delete},
};
use crate::SelemeneEngine;
use std::sync::Arc;

use super::{ghati_handlers, ghati_panchanga_handlers};

/// Create v1 API routes
pub fn create_v1_routes() -> Router<Arc<SelemeneEngine>> {
    Router::new()
        // Basic Panchanga calculation routes (working)
        .route("/panchanga/calculate", post(crate::api::handlers::calculate_panchanga))
        .route("/panchanga/batch", post(crate::api::handlers::calculate_batch_panchanga))
        .route("/panchanga/validate", post(crate::api::handlers::validate_panchanga))

        // Ghati calculation routes
        .route("/ghati/calculate", post(ghati_handlers::calculate_ghati_time))
        .route("/ghati/current", post(ghati_handlers::get_current_ghati_time))
        .route("/ghati/boundaries", post(ghati_handlers::get_ghati_boundaries))
        .route("/ghati/utc-to-ghati", post(ghati_handlers::utc_to_ghati))
        .route("/ghati/ghati-to-utc", post(ghati_handlers::ghati_to_utc))
        .route("/ghati/range", post(ghati_handlers::get_ghati_time_range))
        .route("/ghati/methods", get(ghati_handlers::get_ghati_methods))

        // Ghati-Panchanga integration routes
        .route(
            "/ghati-panchanga/calculate",
            post(ghati_panchanga_handlers::calculate_ghati_panchanga),
        )
        .route(
            "/ghati-panchanga/current",
            post(ghati_panchanga_handlers::get_current_ghati_panchanga),
        )
        .route(
            "/ghati-panchanga/daily",
            post(ghati_panchanga_handlers::get_daily_ghati_panchanga),
        )
        .route(
            "/ghati-panchanga/next-change",
            post(ghati_panchanga_handlers::find_next_panchanga_change),
        )
        .route(
            "/ghati-panchanga/element-changes",
            post(ghati_panchanga_handlers::get_ghati_timing_for_panchanga_changes),
        )
        .route(
            "/ghati-panchanga/precision",
            post(ghati_panchanga_handlers::calculate_panchanga_with_ghati_precision),
        )
        .route(
            "/ghati-panchanga/elements",
            get(ghati_panchanga_handlers::get_panchanga_elements),
        )
        
        // Real-time tracking routes
        // .route("/realtime/tracker/create", post(realtime_handlers::create_tracker))
        // .route("/realtime/tracker/start", post(realtime_handlers::start_tracker))
        // .route("/realtime/tracker/stop", post(realtime_handlers::stop_tracker))
        // .route("/realtime/tracker/state", post(realtime_handlers::get_tracker_state))
        // .route("/realtime/tracker/current-ghati", post(realtime_handlers::get_current_ghati_from_tracker))
        // .route("/realtime/tracker/current-ghati-panchanga", post(realtime_handlers::get_current_ghati_panchanga_from_tracker))
        // .route("/realtime/tracker/next-transition", post(realtime_handlers::get_next_ghati_transition_from_tracker))
        // .route("/realtime/tracker/time-until-next", post(realtime_handlers::get_time_until_next_ghati))
        // .route("/realtime/tracker/update-location", post(realtime_handlers::update_tracker_location))
        // .route("/realtime/tracker/remove", post(realtime_handlers::remove_tracker))
        // .route("/realtime/tracker/list", get(realtime_handlers::list_trackers))
        // .route("/realtime/tracker/config", post(realtime_handlers::get_tracker_config))
        // .route("/realtime/tracker/config/update", post(realtime_handlers::update_tracker_config))
        // .route("/realtime/tracker/events", post(realtime_handlers::get_tracking_events))
        // .route("/realtime/stats", get(realtime_handlers::get_tracking_stats))
}

/// Create admin routes
pub fn create_admin_routes() -> Router<Arc<SelemeneEngine>> {
    Router::new()
        .route("/admin/users", get(crate::api::handlers::list_users))
        .route("/admin/users", post(crate::api::handlers::create_user))
        .route("/admin/users/:id", get(crate::api::handlers::get_user))
        .route("/admin/users/:id", put(crate::api::handlers::update_user))
        .route("/admin/users/:id", delete(crate::api::handlers::delete_user))
        .route("/admin/analytics", get(crate::api::handlers::get_analytics))
        .route("/admin/maintenance", post(crate::api::handlers::trigger_maintenance))
}

/// Create WebSocket routes for real-time updates
pub fn create_websocket_routes() -> Router<Arc<SelemeneEngine>> {
    Router::new()
        .route("/ws/panchanga", get(crate::api::handlers::panchanga_websocket))
        .route("/ws/notifications", get(crate::api::handlers::notifications_websocket))
}