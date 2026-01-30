use axum::{
    middleware::Next,
    response::Response,
    http::{StatusCode, Request},
};
use std::time::Instant;
use tracing::{info, warn, error};
use governor::{Quota, RateLimiter, DefaultDirectRateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use serde_json::json;

/// Logging middleware for request/response logging
pub async fn logging_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    info!(
        "Request started: {} {} (User-Agent: {})",
        method, uri, user_agent
    );

    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status();

    if status.is_success() {
        info!(
            "Request completed: {} {} - {} ({}ms)",
            method, uri, status, duration.as_millis()
        );
    } else {
        warn!(
            "Request failed: {} {} - {} ({}ms)",
            method, uri, status, duration.as_millis()
        );
    }

    Ok(response)
}

/// Authentication middleware
pub async fn auth_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    
    // Skip auth for public endpoints
    if is_public_endpoint(request.uri().path()) {
        return Ok(next.run(request).await);
    }
    
    // Check for API key or JWT token
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                if validate_token(token).await {
                    return Ok(next.run(request).await);
                }
            } else if auth_str.starts_with("ApiKey ") {
                let api_key = &auth_str[7..];
                if validate_api_key(api_key).await {
                    return Ok(next.run(request).await);
                }
            }
        }
    }
    
    // Check for API key in query parameters (for GET requests)
    if let Some(query) = request.uri().query() {
        if let Some((_, v)) = url::form_urlencoded::parse(query.as_bytes())
            .find(|(k, _)| k == "api_key") {
            let api_key = v.to_string();
            if validate_api_key(&api_key).await {
                return Ok(next.run(request).await);
            }
        }
    }
    
    error!("Authentication failed for request: {}", request.uri());
    Err(StatusCode::UNAUTHORIZED)
}

/// Rate limiting middleware
pub async fn rate_limit_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let client_ip = get_client_ip(&request);
    let endpoint = request.uri().path();
    
    // Create rate limiter based on endpoint and client
    let rate_limiter = get_rate_limiter(endpoint);
    
    if rate_limiter.check().is_ok() {
        Ok(next.run(request).await)
    } else {
        warn!("Rate limit exceeded for client: {} at endpoint: {}", client_ip, endpoint);
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}

/// Check if endpoint is public (no auth required)
fn is_public_endpoint(path: &str) -> bool {
    matches!(
        path,
        "/health" | "/metrics" | "/status" | "/docs" | "/openapi.json"
    )
}

/// Validate JWT token
async fn validate_token(token: &str) -> bool {
    // TODO: Use actual AuthService for validation
    // For now, accept any non-empty token
    !token.is_empty()
}

/// Validate API key
async fn validate_api_key(api_key: &str) -> bool {
    // TODO: Use actual AuthService for validation
    // For now, accept any non-empty API key
    !api_key.is_empty()
}

/// Get client IP address
fn get_client_ip<T>(request: &Request<T>) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split(',').next())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
        }
    )
    .unwrap_or("unknown")
    .to_string()
}

/// Get rate limiter for specific endpoint
fn get_rate_limiter(endpoint: &str) -> Arc<DefaultDirectRateLimiter> {
    let requests_per_minute = match endpoint {
        "/api/v1/panchanga" => 60,
        "/api/v1/panchanga/batch" => 10,
        "/api/v1/panchanga/range" => 5,
        "/api/v1/solar/position" => 120,
        "/api/v1/lunar/position" => 120,
        "/api/v1/tithi" => 120,
        "/api/v1/nakshatra" => 120,
        "/api/v1/yoga" => 120,
        "/api/v1/karana" => 120,
        "/api/v1/vara" => 120,
        "/api/v1/houses" => 60,
        "/api/v1/planets" => 60,
        _ => 1000, // Default rate limit
    };
    
    Arc::new(RateLimiter::direct(Quota::per_minute(
        NonZeroU32::new(requests_per_minute).unwrap()
    )))
}

/// Error handling middleware
pub async fn error_handling_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let response = next.run(request).await;
    Ok(response)
}

/// Create error response
#[allow(dead_code)]
fn create_error_response(status: StatusCode) -> Response<axum::body::Body> {
    let error_body = json!({
        "error": {
            "code": status.as_u16(),
            "message": status.canonical_reason().unwrap_or("Unknown error"),
            "timestamp": chrono::Utc::now()
        }
    });
    
    let body = serde_json::to_string(&error_body).unwrap_or_default();
    
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body))
        .unwrap()
}

/// Request validation middleware
pub async fn validation_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // TODO: Implement request validation
    // - Check content length
    // - Validate JSON schema
    // - Sanitize input
    
    Ok(next.run(request).await)
}

/// Compression middleware (handled by tower-http)
pub async fn compression_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Compression is handled by tower-http compression layer
    Ok(next.run(request).await)
}

/// Caching middleware
pub async fn caching_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // TODO: Implement response caching
    // - Check cache headers
    // - Store responses in cache
    // - Return cached responses when appropriate
    
    Ok(next.run(request).await)
}
