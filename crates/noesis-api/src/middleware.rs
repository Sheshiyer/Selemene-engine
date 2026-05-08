//! Middleware components for request logging, tracing, and response standardization

use crate::billing;
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use noesis_auth::{AuthService, AuthUser};
use noesis_metrics::NoesisMetrics;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, info_span, Instrument};

use crate::{ErrorMapper, ErrorResponse};

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
    let trace_id = uuid::Uuid::new_v4().to_string();

    // Extract user_id from AuthUser extension (injected by auth middleware after auth check).
    // Reading from the X-User-Id header would always be "anonymous" since auth middleware
    // uses request extensions, not response headers.
    let user_id = req
        .extensions()
        .get::<AuthUser>()
        .map(|u| u.user_id.clone());

    // Create a span for this request - this automatically generates trace_id and span_id
    let span = info_span!(
        "http_request",
        method = %method,
        path = %path,
        trace_id = %trace_id,
        user_id = user_id.as_deref().unwrap_or("anonymous")
    );

    // Execute the request within the span
    async move {
        let trace_id_for_scope = trace_id.clone();
        let response =
            ErrorMapper::with_request_trace_id(
                trace_id_for_scope,
                async move { next.run(req).await },
            )
            .await;

        // Calculate duration
        let duration_ms = start.elapsed().as_millis() as u64;
        let status = response.status().as_u16();

        // Log within span for automatic context injection
        info!(
            status = status,
            duration_ms = duration_ms,
            trace_id = %trace_id,
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
#[allow(dead_code)]
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
                        return Err(ErrorMapper::response(
                            StatusCode::UNAUTHORIZED,
                            "UNAUTHORIZED",
                            "Invalid or expired JWT token",
                            Some(serde_json::json!({ "auth_method": "jwt" })),
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
                Err(e) => {
                    tracing::warn!(error = %e, "API key validation failed");
                    return Err(ErrorMapper::response(
                        StatusCode::UNAUTHORIZED,
                        "UNAUTHORIZED",
                        "Invalid or expired API key",
                        Some(serde_json::json!({ "auth_method": "api_key" })),
                    ));
                }
            }
        }
    }

    // No valid authentication found
    Err(ErrorMapper::response(
        StatusCode::UNAUTHORIZED,
        "UNAUTHORIZED",
        "Authentication required. Provide JWT token via 'Authorization: Bearer <token>' or API key via 'X-API-Key' header",
        None,
    ))
}

// ---------------------------------------------------------------------------
// Rate limiting middleware
// ---------------------------------------------------------------------------

use redis::AsyncCommands;

/// Per-tier request-per-minute and request-per-day limits.
#[derive(Debug, Clone, Copy)]
struct TierLimits {
    per_minute: u32,
    per_day: u32, // 0 means unlimited
}

/// Tracks daily quota counters, preferring Redis and falling back to in-memory DashMap.
#[derive(Clone)]
struct DailyQuotaTracker {
    redis_client: Option<redis::Client>,
    fallback_counts: Arc<DashMap<String, (u32, i64)>>, // key -> (count, reset_ts)
}

impl DailyQuotaTracker {
    fn new(redis_url: Option<&str>) -> Self {
        let redis_client = redis_url.and_then(|url| redis::Client::open(url).ok());
        Self {
            redis_client,
            fallback_counts: Arc::new(DashMap::new()),
        }
    }

    fn key_and_reset(user_id: &str) -> (String, i64) {
        let now = Utc::now();
        let next_midnight_naive = (now.date_naive() + Duration::days(1))
            .and_hms_opt(0, 0, 0)
            .expect("valid midnight");
        let next_midnight = DateTime::<Utc>::from_naive_utc_and_offset(next_midnight_naive, Utc);
        let date = now.format("%Y-%m-%d").to_string();
        (
            format!("quota:daily:{}:{}", user_id, date),
            next_midnight.timestamp(),
        )
    }

    async fn current_count(&self, user_id: &str) -> u32 {
        let (key, reset_ts) = Self::key_and_reset(user_id);

        // Try Redis first
        if let Some(client) = &self.redis_client {
            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                let count: redis::RedisResult<u32> = conn.get(&key).await;
                if let Ok(v) = count {
                    return v;
                }
            }
        }

        // Fallback in-memory
        if let Some(entry) = self.fallback_counts.get(&key) {
            let (count, stored_reset) = *entry.value();
            if Utc::now().timestamp() <= stored_reset {
                return count;
            }
        }
        // purge stale
        self.fallback_counts.remove(&key);
        self.fallback_counts.insert(key, (0, reset_ts));
        0
    }

    async fn check_and_increment(&self, user_id: &str, daily_limit: u32) -> (bool, u32, i64, bool) {
        let (_key, reset_ts) = Self::key_and_reset(user_id);

        // Unlimited daily quota
        if daily_limit == 0 {
            return (true, u32::MAX, reset_ts, false);
        }

        let (key, reset_ts) = Self::key_and_reset(user_id);

        // Prefer Redis for distributed consistency
        if let Some(client) = &self.redis_client {
            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                let incr_result: redis::RedisResult<u32> = conn.incr(&key, 1).await;
                if let Ok(count) = incr_result {
                    if count == 1 {
                        let _: redis::RedisResult<()> = redis::cmd("EXPIREAT")
                            .arg(&key)
                            .arg(reset_ts)
                            .query_async(&mut conn)
                            .await;
                    }

                    if count > daily_limit {
                        return (false, 0, reset_ts, false);
                    }

                    return (true, daily_limit.saturating_sub(count), reset_ts, false);
                }
            }
        }

        // Fallback (single-process) using DashMap
        let now_ts = Utc::now().timestamp();
        let mut entry = self.fallback_counts.entry(key).or_insert((0, reset_ts));

        // Reset if stale
        if now_ts > entry.value().1 {
            *entry.value_mut() = (0, reset_ts);
        }

        let (count, stored_reset) = entry.value_mut();
        *count += 1;

        if *count > daily_limit {
            return (false, 0, *stored_reset, true);
        }

        (
            true,
            daily_limit.saturating_sub(*count),
            *stored_reset,
            true,
        )
    }
}

/// Rate limiter tracking per-user request counts in a sliding window
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of key -> (request_count, window_start_time)
    /// Keys are either user IDs (authenticated) or "ip:<addr>" (anonymous)
    user_windows: Arc<DashMap<String, (u32, DateTime<Utc>)>>,
    /// Default fallback rate limit for authenticated users: requests per window
    default_limit: u32,
    /// Rate limit for unauthenticated (IP-based) requests per window
    anonymous_limit: u32,
    /// Window duration in seconds
    window_seconds: i64,
    /// Tiered limits (W2-S3-01)
    free_limits: TierLimits,
    pro_limits: TierLimits,
    enterprise_limits: TierLimits,
    /// Daily quota tracking (W2-S3-02)
    daily_quota: DailyQuotaTracker,
}

impl RateLimiter {
    /// Create a new rate limiter with default 100 req/min and 60 second window
    pub fn new() -> Self {
        Self::new_with_config(100, 60, None)
    }

    /// Create a new rate limiter with custom config.
    ///
    /// Tier defaults (env-overridable):
    /// - free: 30/min, 500/day
    /// - pro: 200/min, 10_000/day
    /// - enterprise: 1000/min, unlimited/day (0)
    pub fn new_with_config(
        default_limit: u32,
        window_seconds: u64,
        redis_url: Option<&str>,
    ) -> Self {
        let read_env = |key: &str, default: u32| {
            std::env::var(key)
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(default)
        };

        let free_limits = TierLimits {
            per_minute: read_env("RATE_LIMIT_FREE_PER_MINUTE", 30),
            per_day: read_env("RATE_LIMIT_FREE_PER_DAY", 500),
        };
        let pro_limits = TierLimits {
            per_minute: read_env("RATE_LIMIT_PRO_PER_MINUTE", 200),
            per_day: read_env("RATE_LIMIT_PRO_PER_DAY", 10_000),
        };
        let enterprise_limits = TierLimits {
            per_minute: read_env("RATE_LIMIT_ENTERPRISE_PER_MINUTE", 1_000),
            per_day: read_env("RATE_LIMIT_ENTERPRISE_PER_DAY", 0),
        };

        Self {
            user_windows: Arc::new(DashMap::new()),
            default_limit,
            anonymous_limit: free_limits.per_minute.min(30),
            window_seconds: window_seconds as i64,
            free_limits,
            pro_limits,
            enterprise_limits,
            daily_quota: DailyQuotaTracker::new(redis_url),
        }
    }

    fn limits_for_tier(&self, tier: &str) -> TierLimits {
        match tier.to_ascii_lowercase().as_str() {
            "free" => self.free_limits,
            "pro" | "premium" => self.pro_limits,
            "enterprise" => self.enterprise_limits,
            _ => TierLimits {
                per_minute: self.default_limit,
                per_day: self.free_limits.per_day,
            },
        }
    }

    /// Check if request is allowed and update counter
    /// Returns (is_allowed, remaining, reset_timestamp)
    fn check_and_update(&self, key: &str, rate_limit: u32) -> (bool, u32, i64) {
        let now = Utc::now();

        // Use entry API for atomic check-and-update
        let mut entry = self.user_windows.entry(key.to_string()).or_insert((0, now));
        let (count, window_start) = entry.value_mut();

        // Check if window has expired
        if now - *window_start > Duration::seconds(self.window_seconds) {
            // Reset window
            *count = 1;
            *window_start = now;
            let reset_timestamp =
                (*window_start + Duration::seconds(self.window_seconds)).timestamp();
            (true, rate_limit.saturating_sub(1), reset_timestamp)
        } else if *count < rate_limit {
            // Within window and under limit
            *count += 1;
            let remaining = rate_limit.saturating_sub(*count);
            let reset_timestamp =
                (*window_start + Duration::seconds(self.window_seconds)).timestamp();
            (true, remaining, reset_timestamp)
        } else {
            // Rate limit exceeded
            let reset_timestamp =
                (*window_start + Duration::seconds(self.window_seconds)).timestamp();
            (false, 0, reset_timestamp)
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract client IP address from proxy headers or fall back to "unknown".
///
/// On Railway (and most CDN/proxy setups), the platform appends the real client IP
/// as the LAST entry in X-Forwarded-For. Trusting the FIRST entry is unsafe — an
/// attacker can spoof it by sending `X-Forwarded-For: 1.2.3.4` before the proxy chain.
///
/// Priority order:
/// 1. CF-Connecting-IP (Cloudflare — single hop, can't be spoofed at the edge)
/// 2. Last entry in X-Forwarded-For (added by Railway's own reverse proxy)
/// 3. X-Real-IP (nginx upstream)
fn extract_client_ip(req: &Request) -> String {
    // Cloudflare sets this to the real client IP — single value, not a list.
    if let Some(cf_ip) = req.headers().get("cf-connecting-ip") {
        if let Ok(val) = cf_ip.to_str() {
            let ip = val.trim();
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }

    // X-Forwarded-For: use the LAST (rightmost) IP — it's the one added by Railway's
    // trusted proxy and cannot be forged by the client.
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(val) = forwarded.to_str() {
            if let Some(last_ip) = val.split(',').next_back() {
                let ip = last_ip.trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
    }

    // X-Real-IP (nginx upstream)
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(val) = real_ip.to_str() {
            let ip = val.trim();
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }

    "unknown".to_string()
}

/// Rate limiting middleware that enforces per-user and per-IP request limits.
///
/// Configuration:
/// - Authenticated: tier-configured requests/min + requests/day
/// - Unauthenticated: 30 requests/min per IP address
/// - Window: configurable sliding window (default 60 seconds)
///
/// Behavior:
/// - Authenticated requests: keyed by user_id, tier-based limits
/// - Unauthenticated requests: keyed by IP address, stricter limits
/// - Returns 429 Too Many Requests when minute or daily limit exceeded
///
/// Response headers:
/// - X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset
/// - X-RateLimit-Daily-Remaining, X-RateLimit-Daily-Reset (authenticated)
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Determine rate limit key and limits based on authentication status
    let user = req.extensions().get::<AuthUser>().cloned();

    let (key, per_minute_limit, daily_limit, auth_user_id, auth_tier) = match user {
        Some(auth_user) => {
            let tier_limits = limiter.limits_for_tier(&auth_user.tier);
            // Backward compatibility: explicit per-key rate_limit still overrides if set
            let per_minute_limit = if auth_user.rate_limit > 0 {
                auth_user.rate_limit
            } else {
                tier_limits.per_minute
            };

            (
                auth_user.user_id.clone(),
                per_minute_limit,
                tier_limits.per_day,
                Some(auth_user.user_id),
                Some(auth_user.tier),
            )
        }
        None => {
            // Unauthenticated: rate limit by IP
            let ip = extract_client_ip(&req);
            (format!("ip:{}", ip), limiter.anonymous_limit, 0, None, None)
        }
    };

    // Minute-level check first
    let (allowed_minute, remaining_minute, minute_reset_timestamp) =
        limiter.check_and_update(&key, per_minute_limit);

    // Daily headers (authenticated only)
    let mut daily_remaining: Option<u32> = None;
    let mut daily_reset_timestamp: Option<i64> = None;

    if let Some(user_id) = &auth_user_id {
        let current_count = limiter.daily_quota.current_count(user_id).await;
        let (_, reset_ts) = DailyQuotaTracker::key_and_reset(user_id);
        daily_reset_timestamp = Some(reset_ts);
        daily_remaining = Some(if daily_limit == 0 {
            u32::MAX
        } else {
            daily_limit.saturating_sub(current_count.min(daily_limit))
        });
    }

    if !allowed_minute {
        let mut response = ErrorMapper::response(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMIT_EXCEEDED",
            format!(
                "Rate limit exceeded. Maximum {} requests per {} seconds allowed.",
                per_minute_limit, limiter.window_seconds
            ),
            Some(serde_json::json!({
                "limit": per_minute_limit,
                "window_seconds": limiter.window_seconds,
                "reset_at": minute_reset_timestamp,
            })),
        )
        .into_response();

        let headers = response.headers_mut();
        headers.insert(
            "X-RateLimit-Limit",
            per_minute_limit.to_string().parse().unwrap(),
        );
        headers.insert("X-RateLimit-Remaining", "0".parse().unwrap());
        headers.insert(
            "X-RateLimit-Reset",
            minute_reset_timestamp.to_string().parse().unwrap(),
        );
        if let Some(rem) = daily_remaining {
            headers.insert(
                "X-RateLimit-Daily-Remaining",
                rem.to_string().parse().unwrap(),
            );
        }
        if let Some(reset) = daily_reset_timestamp {
            headers.insert(
                "X-RateLimit-Daily-Reset",
                reset.to_string().parse().unwrap(),
            );
        }

        if let (Some(user_id), Some(tier)) = (&auth_user_id, &auth_tier) {
            billing::emit_quota_exceeded(user_id, tier);
        }

        return Ok(response);
    }

    // Daily check (authenticated users only)
    if let Some(user_id) = &auth_user_id {
        let (allowed_daily, remaining_daily, reset_daily, _used_fallback) = limiter
            .daily_quota
            .check_and_increment(user_id, daily_limit)
            .await;

        daily_remaining = Some(remaining_daily);
        daily_reset_timestamp = Some(reset_daily);

        if !allowed_daily {
            let mut response = ErrorMapper::response(
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                format!(
                    "Daily quota exceeded. Maximum {} requests per day allowed for your tier.",
                    daily_limit
                ),
                Some(serde_json::json!({
                    "daily_limit": daily_limit,
                    "daily_reset_at": reset_daily,
                    "limit_type": "daily",
                })),
            )
            .into_response();

            let headers = response.headers_mut();
            headers.insert(
                "X-RateLimit-Limit",
                per_minute_limit.to_string().parse().unwrap(),
            );
            headers.insert(
                "X-RateLimit-Remaining",
                remaining_minute.to_string().parse().unwrap(),
            );
            headers.insert(
                "X-RateLimit-Reset",
                minute_reset_timestamp.to_string().parse().unwrap(),
            );
            headers.insert("X-RateLimit-Daily-Remaining", "0".parse().unwrap());
            headers.insert(
                "X-RateLimit-Daily-Reset",
                reset_daily.to_string().parse().unwrap(),
            );

            if let Some(tier) = &auth_tier {
                billing::emit_quota_exceeded(user_id, tier);
            }

            return Ok(response);
        }
    }

    // Request allowed - process and add headers
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(
        "X-RateLimit-Limit",
        per_minute_limit.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        remaining_minute.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Reset",
        minute_reset_timestamp.to_string().parse().unwrap(),
    );

    if let Some(rem) = daily_remaining {
        headers.insert(
            "X-RateLimit-Daily-Remaining",
            rem.to_string().parse().unwrap(),
        );
    }
    if let Some(reset) = daily_reset_timestamp {
        headers.insert(
            "X-RateLimit-Daily-Reset",
            reset.to_string().parse().unwrap(),
        );
    }

    Ok(response)
}
