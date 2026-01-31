//! Middleware components for request logging, tracing, and response standardization

use axum::{
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, info_span, Instrument};
use noesis_metrics::NoesisMetrics;
use noesis_auth::{AuthService, AuthUser};
use serde::Serialize;
use dashmap::DashMap;
use chrono::{DateTime, Utc, Duration};

/// Request logging middleware that captures timing and structured request metadata.
///
/// Logs each request with:
/// - HTTP method
/// - Request path
/// - Response status code
/// - Duration in milliseconds
/// - User ID (if available from request extensions)
///
/// All logs are wrapped in a tracing span with trace_id and span_id automatically injected.
pub async fn request_logging_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    
    // Extract user_id from request extensions if authentication middleware set it
    // For now, we'll check if it's in headers (e.g., X-User-Id set by upstream auth)
    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Create a span for this request - this automatically generates trace_id and span_id
    let span = info_span!(
        "http_request",
        method = %method,
        path = %path,
        user_id = user_id.as_deref().unwrap_or("anonymous")
    );

    // Execute the request within the span
    async move {
        // Process the request
        let response = next.run(req).await;
        
        // Calculate duration
        let duration_ms = start.elapsed().as_millis() as u64;
        let status = response.status().as_u16();

        // Log within span for automatic context injection
        info!(
            status = status,
            duration_ms = duration_ms,
            "request completed"
        );

        response
    }
    .instrument(span)
    .await
}

/// Metrics middleware for recording engine calculation metrics.
///
/// Records:
/// - `engine_calculation_duration_seconds` histogram (labeled by engine_id)
/// - `engine_calculation_status_total` counter (labeled by engine_id, status)
/// - `engine_calculation_errors_total` counter (labeled by engine_id, error_type)
///
/// This middleware should wrap handlers that extract engine_id from path or state.
pub async fn metrics_middleware(
    metrics: Arc<NoesisMetrics>,
    engine_id: String,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    
    // Process the request
    let response = next.run(req).await;
    
    // Calculate duration
    let duration_secs = start.elapsed().as_secs_f64();
    let status = response.status();
    
    // Determine status label (success/failure)
    let status_label = if status.is_success() {
        "success"
    } else {
        "failure"
    };
    
    // Record metrics
    metrics.record_engine_calculation_with_status(&engine_id, status_label, duration_secs);
    
    // If it's an error, record error type
    if !status.is_success() {
        let error_type = match status {
            StatusCode::BAD_REQUEST => "bad_request",
            StatusCode::UNAUTHORIZED => "unauthorized",
            StatusCode::FORBIDDEN => "forbidden",
            StatusCode::NOT_FOUND => "not_found",
            StatusCode::UNPROCESSABLE_ENTITY => "validation_error",
            StatusCode::TOO_MANY_REQUESTS => "rate_limit",
            _ => "internal_error",
        };
        
        metrics.record_engine_calculation_error(&engine_id, error_type);
    }
    
    response
}

/// Error response structure for authentication failures
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Authentication middleware that validates JWT tokens or API keys.
///
/// Extracts:
/// - JWT from `Authorization: Bearer <token>` header
/// - OR API key from `X-API-Key` header
///
/// Validates using `AuthService::validate_jwt_token()` or `validate_api_key()`.
/// Injects `AuthUser` into request extensions for handler access.
///
/// Returns 401 UNAUTHORIZED if authentication fails.
pub async fn auth_middleware(
    State(auth): State<Arc<AuthService>>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Try JWT token first
    if let Some(auth_header) = req.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                match auth.validate_jwt_token(token).await {
                    Ok(user) => {
                        // Insert authenticated user into request extensions
                        req.extensions_mut().insert(user);
                        return Ok(next.run(req).await);
                    }
                    Err(_) => {
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            Json(ErrorResponse {
                                error: "Invalid or expired JWT token".to_string(),
                                error_code: "UNAUTHORIZED".to_string(),
                                details: Some(serde_json::json!({ "auth_method": "jwt" })),
                            }),
                        ));
                    }
                }
            }
        }
    }

    // Try API key next
    if let Some(api_key_header) = req.headers().get("X-API-Key") {
        if let Ok(api_key) = api_key_header.to_str() {
            match auth.validate_api_key(api_key).await {
                Ok(user) => {
                    // Insert authenticated user into request extensions
                    req.extensions_mut().insert(user);
                    return Ok(next.run(req).await);
                }
                Err(_) => {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(ErrorResponse {
                            error: "Invalid or expired API key".to_string(),
                            error_code: "UNAUTHORIZED".to_string(),
                            details: Some(serde_json::json!({ "auth_method": "api_key" })),
                        }),
                    ));
                }
            }
        }
    }

    // No valid authentication found
    Err((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Authentication required. Provide JWT token via 'Authorization: Bearer <token>' or API key via 'X-API-Key' header".to_string(),
            error_code: "UNAUTHORIZED".to_string(),
            details: None,
        }),
    ))
}

// ---------------------------------------------------------------------------
// Rate limiting middleware
// ---------------------------------------------------------------------------

/// Rate limiter tracking per-user request counts in a sliding window
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of user_id -> (request_count, window_start_time)
    user_windows: Arc<DashMap<String, (u32, DateTime<Utc>)>>,
    /// Default rate limit: requests per minute
    default_limit: u32,
    /// Window duration in seconds
    window_seconds: i64,
}

impl RateLimiter {
    /// Create a new rate limiter with default 100 req/min and 60 second window
    pub fn new() -> Self {
        Self {
            user_windows: Arc::new(DashMap::new()),
            default_limit: 100,
            window_seconds: 60,
        }
    }
    
    /// Create a new rate limiter with custom config
    pub fn new_with_config(default_limit: u32, window_seconds: u64) -> Self {
        Self {
            user_windows: Arc::new(DashMap::new()),
            default_limit,
            window_seconds: window_seconds as i64,
        }
    }

    /// Check if request is allowed and update counter
    /// Returns (is_allowed, remaining, reset_timestamp)
    fn check_and_update(&self, user_id: &str, rate_limit: u32) -> (bool, u32, i64) {
        let now = Utc::now();
        
        // Use entry API for atomic check-and-update
        let mut entry = self.user_windows.entry(user_id.to_string()).or_insert((0, now));
        let (count, window_start) = entry.value_mut();
        
        // Check if window has expired (1 minute sliding window)
        if now - *window_start > Duration::seconds(self.window_seconds) {
            // Reset window
            *count = 1;
            *window_start = now;
            let reset_timestamp = (*window_start + Duration::seconds(self.window_seconds)).timestamp();
            (true, rate_limit.saturating_sub(1), reset_timestamp)
        } else if *count < rate_limit {
            // Within window and under limit
            *count += 1;
            let remaining = rate_limit.saturating_sub(*count);
            let reset_timestamp = (*window_start + Duration::seconds(self.window_seconds)).timestamp();
            (true, remaining, reset_timestamp)
        } else {
            // Rate limit exceeded
            let reset_timestamp = (*window_start + Duration::seconds(self.window_seconds)).timestamp();
            (false, 0, reset_timestamp)
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiting middleware that enforces per-user request limits.
///
/// Configuration:
/// - Rate limit: 100 requests per minute (or user-specific from AuthUser)
/// - Window: 60 seconds sliding window
///
/// Behavior:
/// - Extracts user_id from AuthUser extension (set by auth_middleware)
/// - Tracks requests per user in sliding time window
/// - Returns 429 Too Many Requests when limit exceeded
/// - Skips rate limiting if no AuthUser present (public routes)
///
/// Response headers:
/// - X-RateLimit-Limit: Maximum requests per minute
/// - X-RateLimit-Remaining: Remaining requests in current window
/// - X-RateLimit-Reset: Unix timestamp when window resets
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Check if user is authenticated (AuthUser extension present)
    let user = req.extensions().get::<AuthUser>().cloned();
    
    // Skip rate limiting for unauthenticated requests (public routes)
    let Some(auth_user) = user else {
        return Ok(next.run(req).await);
    };
    
    // Get rate limit (from user or use default 100)
    let rate_limit = if auth_user.rate_limit > 0 {
        auth_user.rate_limit
    } else {
        limiter.default_limit
    };
    
    // Check rate limit
    let (allowed, remaining, reset_timestamp) = limiter.check_and_update(&auth_user.user_id, rate_limit);
    
    if !allowed {
        // Rate limit exceeded - return 429 with headers
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                error: format!("Rate limit exceeded. Maximum {} requests per minute allowed.", rate_limit),
                error_code: "RATE_LIMIT_EXCEEDED".to_string(),
                details: Some(serde_json::json!({
                    "limit": rate_limit,
                    "window_seconds": limiter.window_seconds,
                    "reset_at": reset_timestamp,
                })),
            }),
        ).into_response();
        
        // Add rate limit headers
        let headers = response.headers_mut();
        headers.insert("X-RateLimit-Limit", rate_limit.to_string().parse().unwrap());
        headers.insert("X-RateLimit-Remaining", "0".parse().unwrap());
        headers.insert("X-RateLimit-Reset", reset_timestamp.to_string().parse().unwrap());
        
        return Ok(response);
    }
    
    // Request allowed - process and add rate limit headers to response
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("X-RateLimit-Limit", rate_limit.to_string().parse().unwrap());
    headers.insert("X-RateLimit-Remaining", remaining.to_string().parse().unwrap());
    headers.insert("X-RateLimit-Reset", reset_timestamp.to_string().parse().unwrap());
    
    Ok(response)
}
