//! Request/response logging helpers for FreeAstrologyAPI.com

use std::time::Duration;
use reqwest::{Method, StatusCode};
use tracing::{debug, info, warn};

pub fn log_request(method: &Method, url: &str, masked_key: &str) {
    info!(
        api.method = %method,
        api.url = %url,
        api.key = %masked_key,
        "FreeAstrologyAPI request"
    );
}

pub fn log_response(url: &str, status: StatusCode, duration: Duration) {
    info!(
        api.url = %url,
        api.status = %status.as_u16(),
        api.duration_ms = %duration.as_millis(),
        "FreeAstrologyAPI response"
    );
}

pub fn log_error(url: &str, status: Option<StatusCode>, duration: Duration, body: &str) {
    warn!(
        api.url = %url,
        api.status = %status.map(|s| s.as_u16()).unwrap_or(0),
        api.duration_ms = %duration.as_millis(),
        api.body = %body,
        "FreeAstrologyAPI error response"
    );
}

pub fn log_request_build_failure(url: &str) {
    debug!(api.url = %url, "Failed to build request for logging");
}
