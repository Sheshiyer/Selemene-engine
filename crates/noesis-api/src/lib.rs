//! Noesis API -- Axum HTTP server for the Tryambakam Noesis platform
//!
//! Wires up the orchestrator, cache, auth, and metrics into a unified REST API.
//! All engine calculations and workflow executions are exposed through versioned
//! JSON endpoints under `/api/v1/`.

mod billing;
mod biofield_client;
mod config;
pub mod error;
pub mod error_mapper;
mod handlers;
mod logging;
mod middleware;
pub mod workflow_parity;

pub use billing::{
    reset_billing_emitter, set_billing_emitter, BillingEventEmitter, DodoWebhookEmitter,
    NoopBillingEmitter, StripeWebhookEmitter,
};
pub use biofield_client::{BiofieldAnalyzeRequest, BiofieldClient};

// Re-export configuration and logging for main.rs
pub use config::ApiConfig;
pub use error_mapper::{ErrorMapper, ErrorResponse};
pub use logging::{init_tracing, init_tracing_json};

use axum::{
    extract::{DefaultBodyLimit, Json, Multipart, Path, State},
    http::{HeaderValue, Method, StatusCode},
    middleware as axum_middleware,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Extension, Router,
};
use chrono::{
    Datelike, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Timelike,
};
use chrono_tz::Tz;
use engine_human_design::ephemeris::{EphemerisCalculator, HDPlanet};
use noesis_auth::{AuthService, AuthUser};
use noesis_cache::CacheManager;
use noesis_core::{
    BiofieldResultSchema, BiorhythmResultSchema, EngineError, EngineInput, EngineOutput,
    EngineResultData, EnneagramResultSchema, FaceReadingResultSchema, GeneKeysResultSchema,
    HumanDesignResultSchema, IChingResultSchema, NadabrahmanResultSchema, NumerologyResultSchema,
    PanchangaResultSchema, Precision, SacredGeometryResultSchema, SigilForgeResultSchema,
    TarotResultSchema, TransitsResultSchema, ValidationResult, VedicClockResultSchema,
    VimshottariResultSchema, WorkflowResult,
};
use noesis_data::models::reading::NewReading;
use noesis_data::repositories::admin_repository::AdminRepository;
use noesis_data::repositories::biofield_repository::BiofieldRepository;
use noesis_data::repositories::readings_repository::ReadingsRepository;
use noesis_data::repositories::usage_repository::UsageRepository;
use noesis_data::repositories::user_repository::UserRepository;
use noesis_metrics::NoesisMetrics;
use noesis_orchestrator::WorkflowOrchestrator;
use sentry_tower::{NewSentryLayer, SentryHttpLayer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use workflow_parity::log_workflow_registry_parity;

// ---------------------------------------------------------------------------
// OpenAPI documentation
// ---------------------------------------------------------------------------

#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        readiness_handler,
        status_handler,
        vedic_chart_handler,
        list_engines_handler,
        calculate_handler,
        face_reading_upload_handler,
        validate_handler,
        engine_info_handler,
        list_workflows_handler,
        workflow_execute_handler,
        birth_blueprint_execute_doc,
        daily_practice_execute_doc,
        decision_support_execute_doc,
        self_inquiry_execute_doc,
        creative_expression_execute_doc,
        full_spectrum_execute_doc,
        workflow_info_handler,
        handlers::users::get_me,
        handlers::users::get_my_usage,
        handlers::users::update_me,
        handlers::admin::get_session,
        handlers::admin::list_users,
        handlers::admin::update_user_state,
        handlers::admin::update_user_tier,
        handlers::admin::update_user_roles,
        handlers::admin::list_api_keys,
        handlers::admin::create_api_key,
        handlers::admin::revoke_api_key,
        handlers::admin::rotate_api_key,
        handlers::admin::delete_api_key,
        handlers::admin::history_sync_users,
        handlers::admin::history_sync_devices,
        handlers::admin::history_sync_events,
        handlers::admin::usage_summary,
        handlers::admin::analytics_summary,
        handlers::admin::analytics_timeseries,
        handlers::admin::analytics_breakdown,
        handlers::admin::analytics_top_consumers,
        handlers::admin::system_health,
        handlers::admin::system_services,
        handlers::admin::system_workflows,
        handlers::admin::system_cache,
        handlers::admin::list_audit_events,
        handlers::admin::get_audit_event,
        handlers::admin::list_audit_actions,
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::forgot_password,
        handlers::auth::reset_password,
        handlers::auth::change_password,
        handlers::biofield::create_session,
        handlers::biofield::close_session,
        handlers::biofield::get_session,
        handlers::biofield::create_capture,
        handlers::biofield::list_readings,
        handlers::biofield::get_reading,
        handlers::biofield::reprocess_reading,
        handlers::biofield::list_baselines,
        handlers::biofield::create_baseline,
    ),
    components(
        schemas(
            EngineInput,
            EngineOutput,
            EngineResultData,
            PanchangaResultSchema,
            NumerologyResultSchema,
            BiorhythmResultSchema,
            HumanDesignResultSchema,
            GeneKeysResultSchema,
            VimshottariResultSchema,
            BiofieldResultSchema,
            VedicClockResultSchema,
            FaceReadingResultSchema,
            NadabrahmanResultSchema,
            TransitsResultSchema,
            EnneagramResultSchema,
            TarotResultSchema,
            IChingResultSchema,
            SacredGeometryResultSchema,
            SigilForgeResultSchema,
            ValidationResult,
            WorkflowResult,
            ApiEngineOutputResponse,
            ApiWorkflowResultResponse,
            HealthResponse,
            ReadinessResponse,
            StatusResponse,
            WorkflowSummary,
            EngineInfoResponse,
            EngineListResponse,
            WorkflowListResponse,
            WorkflowInfoResponse,
            VedicChartBundleResponseSchema,
            BirthBlueprintSynthesisSchema,
            DailyPracticeSynthesisSchema,
            DecisionSupportSynthesisSchema,
            SelfInquirySynthesisSchema,
            CreativeExpressionSynthesisSchema,
            FullSpectrumSynthesisSchema,
            BirthBlueprintWorkflowResultSchema,
            DailyPracticeWorkflowResultSchema,
            DecisionSupportWorkflowResultSchema,
            SelfInquiryWorkflowResultSchema,
            CreativeExpressionWorkflowResultSchema,
            FullSpectrumWorkflowResultSchema,
            FaceUploadResponse,
            ErrorResponse,
            handlers::users::UserResponse,
            handlers::users::LocationResponse,
            handlers::users::UpdateUserRequest,
            handlers::users::UserUsageWindowSummary,
            handlers::users::UserUsageEngineEntry,
            handlers::users::UserUsageResponse,
            handlers::admin::AdminSessionResponse,
            handlers::admin::AdminUsersResponse,
            handlers::admin::AdminUserItem,
            handlers::admin::UpdateUserStateRequest,
            handlers::admin::UpdateUserStateResponse,
            handlers::admin::UpdateUserTierRequest,
            handlers::admin::UpdateUserTierResponse,
            handlers::admin::UpdateUserRolesRequest,
            handlers::admin::UpdateUserRolesResponse,
            handlers::admin::AdminApiKeysResponse,
            handlers::admin::AdminApiKeyItem,
            handlers::admin::CreateApiKeyRequest,
            handlers::admin::CreateApiKeyResponse,
            handlers::admin::RotateApiKeyResponse,
            handlers::admin::AdminHistorySyncUsersResponse,
            handlers::admin::AdminHistorySyncUserItem,
            handlers::admin::AdminHistorySyncDevicesResponse,
            handlers::admin::AdminHistorySyncDeviceItem,
            handlers::admin::AdminHistorySyncEventsResponse,
            handlers::admin::AdminHistorySyncEventItem,
            handlers::admin::AdminUsageWindowSummary,
            handlers::admin::AdminUsageEngineEntry,
            handlers::admin::AdminUsageTopUserEntry,
            handlers::admin::AdminUsageDailyPoint,
            handlers::admin::AdminUsageTierEntry,
            handlers::admin::AdminUsageSummaryResponse,
            handlers::admin::AdminAnalyticsSummaryResponse,
            handlers::admin::AdminAnalyticsTimeseriesResponse,
            handlers::admin::AdminAnalyticsTimeseriesPoint,
            handlers::admin::AdminAnalyticsBreakdownResponse,
            handlers::admin::AdminAnalyticsBreakdownEntry,
            handlers::admin::AdminAnalyticsTopConsumersResponse,
            handlers::admin::AdminAnalyticsTopConsumerItem,
            handlers::admin::AdminSystemHealthResponse,
            handlers::admin::AdminSystemSubsystemStatus,
            handlers::admin::AdminSystemServicesResponse,
            handlers::admin::AdminSystemServiceItem,
            handlers::admin::AdminSystemWorkflowsResponse,
            handlers::admin::AdminSystemWorkflowItem,
            handlers::admin::AdminSystemCacheResponse,
            handlers::admin::AdminAuditEventsResponse,
            handlers::admin::AdminAuditEventItem,
            handlers::admin::AdminAuditEventDetailResponse,
            handlers::admin::AdminAuditActionsResponse,
            handlers::auth::RegisterRequest,
            handlers::auth::RegisterResponse,
            handlers::auth::LoginRequest,
            handlers::auth::LoginResponse,
            handlers::auth::ForgotPasswordRequest,
            handlers::auth::ForgotPasswordResponse,
            handlers::auth::ResetPasswordRequest,
            handlers::auth::ResetPasswordResponse,
            handlers::auth::ChangePasswordRequest,
            handlers::auth::ChangePasswordResponse,
            handlers::biofield::CreateBiofieldSessionRequest,
            handlers::biofield::CloseBiofieldSessionRequest,
            handlers::biofield::ListBiofieldReadingsQuery,
            handlers::biofield::BiofieldSessionResource,
            handlers::biofield::BiofieldQualitySummary,
            handlers::biofield::BiofieldArtifactSummary,
            handlers::biofield::BiofieldCaptureResponse,
            handlers::biofield::BiofieldReprocessResponse,
            handlers::biofield::BiofieldReadingSummary,
            handlers::biofield::ListBiofieldReadingsResponse,
            handlers::biofield::CreateBiofieldBaselineRequest,
            handlers::biofield::BiofieldBaselineSummary,
            handlers::biofield::ListBiofieldBaselinesResponse,
            handlers::biofield::ReprocessBiofieldReadingRequest,
            handlers::biofield::BiofieldReadingDetail,
        )
    ),
    tags(
        (name = "health", description = "Health check and monitoring endpoints"),
        (name = "charts", description = "Vedic chart geometry endpoints"),
        (name = "engines", description = "Single engine calculation endpoints"),
        (name = "workflows", description = "Multi-engine workflow execution endpoints"),
        (name = "users", description = "User profile management endpoints"),
        (name = "admin", description = "Admin session and management surfaces"),
        (name = "auth", description = "Authentication endpoints (register, login, password reset)"),
        (name = "biofield", description = "Biofield capture session and reading endpoints"),
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Noesis API",
        version = "3.0.0",
        description = "HTTP API for the Tryambakam Noesis consciousness engine platform. Provides endpoints for astrological calculations (Panchanga), numerology, biorhythms, and multi-engine workflows.",
        contact(
            name = "Tryambakam Team",
        )
    ),
)]
struct ApiDoc;

use utoipa::openapi::path::PathItemType;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::{HeaderBuilder, ObjectBuilder, Ref, RefOr, ResponseBuilder, SchemaType};
use utoipa::Modify;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("JWT token obtained from authentication endpoint"))
                        .build(),
                ),
            );
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );
        }

        let integer_header_schema = ObjectBuilder::new()
            .schema_type(SchemaType::Integer)
            .build();

        let rate_limited_response = ResponseBuilder::new()
            .description("Rate limit exceeded")
            .header(
                "X-RateLimit-Limit",
                HeaderBuilder::new()
                    .description(Some("Per-minute quota limit."))
                    .schema(integer_header_schema.clone())
                    .build(),
            )
            .header(
                "X-RateLimit-Remaining",
                HeaderBuilder::new()
                    .description(Some("Remaining requests in current minute window."))
                    .schema(integer_header_schema.clone())
                    .build(),
            )
            .header(
                "X-RateLimit-Reset",
                HeaderBuilder::new()
                    .description(Some("UNIX timestamp when minute window resets."))
                    .schema(integer_header_schema.clone())
                    .build(),
            )
            .header(
                "X-RateLimit-Daily-Remaining",
                HeaderBuilder::new()
                    .description(Some("Remaining requests in current daily window."))
                    .schema(integer_header_schema.clone())
                    .build(),
            )
            .header(
                "X-RateLimit-Daily-Reset",
                HeaderBuilder::new()
                    .description(Some("UNIX timestamp when daily window resets."))
                    .schema(integer_header_schema)
                    .build(),
            )
            .content(
                "application/json",
                utoipa::openapi::ContentBuilder::new()
                    .schema(RefOr::Ref(Ref::from_schema_name("ErrorResponse")))
                    .build(),
            )
            .build();

        for path_item in openapi.paths.paths.values_mut() {
            for method in [
                PathItemType::Get,
                PathItemType::Post,
                PathItemType::Put,
                PathItemType::Patch,
                PathItemType::Delete,
                PathItemType::Head,
                PathItemType::Options,
                PathItemType::Trace,
                PathItemType::Connect,
            ] {
                if let Some(operation) = path_item.operations.get_mut(&method) {
                    let is_secured = operation
                        .security
                        .as_ref()
                        .map(|security| !security.is_empty())
                        .unwrap_or(false);

                    if !is_secured {
                        continue;
                    }

                    operation
                        .responses
                        .responses
                        .insert("429".to_string(), RefOr::T(rate_limited_response.clone()));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

/// Shared application state threaded through all Axum handlers via `State`.
#[derive(Clone)]
pub struct AppState {
    pub orchestrator: Arc<WorkflowOrchestrator>,
    pub bridge_manager: Arc<noesis_bridge::BridgeManager>,
    pub cache: Arc<CacheManager>,
    pub auth: Arc<AuthService>,
    pub metrics: Arc<NoesisMetrics>,
    pub user_repository: Arc<UserRepository>,
    pub admin_repository: Option<Arc<AdminRepository>>,
    pub biofield_repository: Option<Arc<BiofieldRepository>>,
    pub readings_repository: Option<Arc<ReadingsRepository>>,
    pub usage_repository: Option<Arc<UsageRepository>>,
    pub oauth_repository: Option<Arc<noesis_data::repositories::oauth_repository::OAuthRepository>>,
    pub startup_time: Instant,
    pub db_available: bool,
    pub discord_client_id: Option<String>,
    pub discord_client_secret: Option<String>,
    pub discord_redirect_uri: Option<String>,
}

pub fn shared_metrics() -> Arc<NoesisMetrics> {
    static SHARED_METRICS: std::sync::OnceLock<Arc<NoesisMetrics>> = std::sync::OnceLock::new();

    SHARED_METRICS
        .get_or_init(|| Arc::new(NoesisMetrics::new().expect("Failed to initialise NoesisMetrics")))
        .clone()
}

// ---------------------------------------------------------------------------
// CORS configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct WildcardOriginPattern {
    scheme: String,
    host_suffix: String,
}

fn parse_wildcard_origin_pattern(origin: &str) -> Option<WildcardOriginPattern> {
    let trimmed = origin.trim();
    let (scheme, host_suffix) = trimmed.split_once("://*.")?;

    if scheme.is_empty() || host_suffix.is_empty() {
        return None;
    }

    Some(WildcardOriginPattern {
        scheme: scheme.to_ascii_lowercase(),
        host_suffix: format!(".{}", host_suffix.to_ascii_lowercase()),
    })
}

fn matches_wildcard_origin(origin: &str, pattern: &WildcardOriginPattern) -> bool {
    let Some((scheme, authority)) = origin.split_once("://") else {
        return false;
    };

    if !scheme.eq_ignore_ascii_case(&pattern.scheme) {
        return false;
    }

    let host = authority
        .split('/')
        .next()
        .unwrap_or(authority)
        .split(':')
        .next()
        .unwrap_or(authority)
        .trim()
        .to_ascii_lowercase();

    if !host.ends_with(&pattern.host_suffix) {
        return false;
    }

    let prefix = &host[..host.len() - pattern.host_suffix.len()];
    !prefix.is_empty()
}

fn origin_is_allowed(
    origin: &HeaderValue,
    exact_origins: &[HeaderValue],
    wildcard_patterns: &[WildcardOriginPattern],
) -> bool {
    if exact_origins.contains(origin) {
        return true;
    }

    let Ok(origin_str) = origin.to_str() else {
        return false;
    };

    wildcard_patterns
        .iter()
        .any(|pattern| matches_wildcard_origin(origin_str, pattern))
}

/// Create production-ready CORS layer with environment-based origin allowlist.
///
/// # Arguments
/// * `allowed_origins` - List of allowed origins (e.g., ["http://localhost:3000"])
///
/// Configuration:
/// - Methods: GET, POST, PATCH, PUT, DELETE, OPTIONS
/// - Headers: Content-Type, Authorization, X-API-Key
/// - Credentials: true (for cookie/auth workflows)
/// - Max Age: 3600 seconds (1 hour)
fn create_cors_layer(allowed_origins: Vec<String>) -> CorsLayer {
    let wildcard_patterns: Vec<WildcardOriginPattern> = allowed_origins
        .iter()
        .filter_map(|origin| parse_wildcard_origin_pattern(origin))
        .collect();

    let exact_origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter(|origin| parse_wildcard_origin_pattern(origin).is_none())
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse().ok()
            }
        })
        .collect();

    let allow_origin = if wildcard_patterns.is_empty() {
        AllowOrigin::list(exact_origins.clone())
    } else {
        let exact_origins = exact_origins.clone();
        let wildcard_patterns = wildcard_patterns.clone();
        AllowOrigin::predicate(move |origin, _parts| {
            origin_is_allowed(origin, &exact_origins, &wildcard_patterns)
        })
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderName::from_static("x-api-key"),
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

#[cfg(test)]
mod cors_tests {
    use super::*;

    #[test]
    fn parses_wildcard_origin_pattern() {
        let pattern = parse_wildcard_origin_pattern("https://*.railway.app")
            .expect("wildcard pattern should parse");

        assert_eq!(
            pattern,
            WildcardOriginPattern {
                scheme: "https".to_string(),
                host_suffix: ".railway.app".to_string(),
            }
        );
    }

    #[test]
    fn wildcard_origin_matches_subdomain_only() {
        let pattern = parse_wildcard_origin_pattern("https://*.railway.app")
            .expect("wildcard pattern should parse");

        assert!(matches_wildcard_origin(
            "https://selemene-engine-production.up.railway.app",
            &pattern
        ));
        assert!(!matches_wildcard_origin("https://railway.app", &pattern));
        assert!(!matches_wildcard_origin(
            "http://selemene-engine-production.up.railway.app",
            &pattern
        ));
        assert!(!matches_wildcard_origin(
            "https://railway.app.evil.com",
            &pattern
        ));
    }

    #[test]
    fn origin_allowlist_supports_exact_and_wildcard_matches() {
        let exact_origins = vec!["http://localhost:5173"
            .parse::<HeaderValue>()
            .expect("localhost origin should parse")];
        let wildcard_patterns = vec![parse_wildcard_origin_pattern("https://*.railway.app")
            .expect("wildcard pattern should parse")];

        assert!(origin_is_allowed(
            &"http://localhost:5173"
                .parse::<HeaderValue>()
                .expect("localhost origin should parse"),
            &exact_origins,
            &wildcard_patterns,
        ));
        assert!(origin_is_allowed(
            &"https://selemene-engine-production.up.railway.app"
                .parse::<HeaderValue>()
                .expect("preview origin should parse"),
            &exact_origins,
            &wildcard_patterns,
        ));
        assert!(!origin_is_allowed(
            &"https://evil.com"
                .parse::<HeaderValue>()
                .expect("origin should parse"),
            &exact_origins,
            &wildcard_patterns,
        ));
    }
}

// ---------------------------------------------------------------------------
// Router construction
// ---------------------------------------------------------------------------

/// Build the Axum router with all API routes and middleware.
///
/// # Arguments
/// * `state` - Application state with orchestrator, cache, auth, metrics
/// * `config` - API configuration with CORS, rate limiting, etc.
pub fn create_router(state: AppState, config: &ApiConfig) -> Router {
    let auth_state = state.auth.clone();

    // Create rate limiter with config values
    let rate_limiter = Arc::new(middleware::RateLimiter::new_with_config(
        config.rate_limit_requests,
        config.rate_limit_window_secs,
        config.redis_url.as_deref(),
    ));

    let auth_routes = Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route(
            "/auth/forgot-password",
            post(handlers::auth::forgot_password),
        )
        .route("/auth/reset-password", post(handlers::auth::reset_password))
        .route(
            "/auth/discord/authorize",
            get(handlers::oauth::discord_authorize),
        )
        .route(
            "/auth/discord/callback",
            post(handlers::oauth::discord_callback),
        );

    let api_v1 = Router::new()
        .route(
            "/auth/change-password",
            post(handlers::auth::change_password),
        )
        .route(
            "/users/me",
            get(handlers::users::get_me).patch(handlers::users::update_me),
        )
        .route("/users/me/usage", get(handlers::users::get_my_usage))
        .route(
            "/biofield/sessions",
            post(handlers::biofield::create_session),
        )
        .route(
            "/biofield/sessions/:session_id/close",
            post(handlers::biofield::close_session),
        )
        .route(
            "/biofield/sessions/:session_id",
            get(handlers::biofield::get_session),
        )
        .route(
            "/biofield/sessions/:session_id/captures",
            post(handlers::biofield::create_capture),
        )
        .route("/biofield/readings", get(handlers::biofield::list_readings))
        .route(
            "/biofield/readings/:reading_id",
            get(handlers::biofield::get_reading),
        )
        .route(
            "/biofield/readings/:reading_id/reprocess",
            post(handlers::biofield::reprocess_reading),
        )
        .route(
            "/biofield/baselines",
            get(handlers::biofield::list_baselines).post(handlers::biofield::create_baseline),
        )
        .route("/admin/session", get(handlers::admin::get_session))
        .route("/admin/users", get(handlers::admin::list_users))
        .route(
            "/admin/users/:user_id/state",
            patch(handlers::admin::update_user_state),
        )
        .route(
            "/admin/users/:user_id/tier",
            patch(handlers::admin::update_user_tier),
        )
        .route(
            "/admin/users/:user_id/roles",
            put(handlers::admin::update_user_roles),
        )
        .route(
            "/admin/api-keys",
            get(handlers::admin::list_api_keys).post(handlers::admin::create_api_key),
        )
        .route(
            "/admin/api-keys/:key_id",
            delete(handlers::admin::delete_api_key),
        )
        .route(
            "/admin/api-keys/:key_id/revoke",
            post(handlers::admin::revoke_api_key),
        )
        .route(
            "/admin/api-keys/:key_id/rotate",
            post(handlers::admin::rotate_api_key),
        )
        .route(
            "/admin/history-sync/users",
            get(handlers::admin::history_sync_users),
        )
        .route(
            "/admin/history-sync/devices",
            get(handlers::admin::history_sync_devices),
        )
        .route(
            "/admin/history-sync/events",
            get(handlers::admin::history_sync_events),
        )
        .route("/admin/usage/summary", get(handlers::admin::usage_summary))
        .route(
            "/admin/analytics/summary",
            get(handlers::admin::analytics_summary),
        )
        .route(
            "/admin/analytics/usage-timeseries",
            get(handlers::admin::analytics_timeseries),
        )
        .route(
            "/admin/analytics/usage-breakdown",
            get(handlers::admin::analytics_breakdown),
        )
        .route(
            "/admin/analytics/top-consumers",
            get(handlers::admin::analytics_top_consumers),
        )
        .route("/admin/system/health", get(handlers::admin::system_health))
        .route(
            "/admin/system/services",
            get(handlers::admin::system_services),
        )
        .route(
            "/admin/system/workflows",
            get(handlers::admin::system_workflows),
        )
        .route("/admin/system/cache", get(handlers::admin::system_cache))
        .route(
            "/admin/audit-events",
            get(handlers::admin::list_audit_events),
        )
        .route(
            "/admin/audit-events/actions",
            get(handlers::admin::list_audit_actions),
        )
        .route(
            "/admin/audit-events/:event_id",
            get(handlers::admin::get_audit_event),
        )
        .route("/status", get(status_handler))
        .route("/charts/vedic", post(vedic_chart_handler))
        .route("/engines", get(list_engines_handler))
        .route("/engines/:engine_id/calculate", post(calculate_handler))
        .route(
            "/engines/face-reading/upload",
            post(face_reading_upload_handler),
        )
        .route("/engines/:engine_id/validate", post(validate_handler))
        .route("/engines/:engine_id/info", get(engine_info_handler))
        .route("/workflows", get(list_workflows_handler))
        .route(
            "/workflows/:workflow_id",
            get(workflow_info_handler).post(workflow_execute_handler),
        )
        .route(
            "/workflows/:workflow_id/execute",
            post(workflow_execute_handler),
        )
        .route("/workflows/:workflow_id/info", get(workflow_info_handler))
        .route("/readings", get(list_readings_handler))
        .route("/readings/stats", get(readings_stats_handler))
        .route("/readings/:reading_id", get(get_reading_handler))
        // OpenClaw onboarding routes
        .route(
            "/onboarding/invite",
            post(handlers::onboarding::create_invite),
        )
        .route(
            "/onboarding/invites",
            get(handlers::onboarding::list_invites),
        )
        .route(
            "/onboarding/:code/openclaw.txt",
            get(handlers::onboarding::get_onboarding),
        )
        // Layers are applied bottom-to-top, so rate_limit runs AFTER auth
        .layer(axum_middleware::from_fn_with_state(
            rate_limiter.clone(),
            middleware::rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            auth_state,
            middleware::auth_middleware,
        ))
        .merge(auth_routes);

    // Legacy endpoints for backward compatibility with old Selemene API
    // Rate limited by IP (no auth required, but protected against abuse)
    let legacy = Router::new()
        .route("/panchanga/calculate", post(legacy_panchanga_handler))
        .route("/ghati/current", get(legacy_ghati_current_handler))
        .layer(axum_middleware::from_fn_with_state(
            rate_limiter,
            middleware::rate_limit_middleware,
        ));

    // Start with a base router and merge docs first (both have () state)
    let base = Router::new()
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()));

    // Now add stateful routes
    base.route("/health", get(health_handler))
        .route("/health/live", get(health_handler)) // Kubernetes liveness probe
        .route("/health/ready", get(readiness_handler)) // Kubernetes readiness probe
        .route("/ready", get(readiness_handler))
        .route("/metrics", get(metrics_handler))
        .nest("/api/v1", api_v1)
        .nest("/api/legacy", legacy)
        .layer(axum_middleware::from_fn(
            middleware::request_logging_middleware,
        ))
        .layer(SentryHttpLayer::with_transaction())
        .layer(NewSentryLayer::new_from_top())
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024)) // 2 MB max body size
        .layer(create_cors_layer(config.allowed_origins.clone()))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    engines_loaded: usize,
    workflows_loaded: usize,
}

#[derive(Serialize, ToSchema)]
struct ReadinessResponse {
    redis: String,
    postgres: String,
    orchestrator: String,
    bridge_status: String,
    bridge_engines: Vec<BridgeEngineStatus>,
    bridge_failed_engines: Vec<String>,
    overall_status: String,
}

#[derive(Serialize, ToSchema)]
struct BridgeEngineStatus {
    engine_id: String,
    healthy: bool,
    detail: String,
    latency_ms: u64,
}

#[derive(Serialize, ToSchema)]
struct StatusResponse {
    engines: Vec<String>,
    workflows: Vec<WorkflowSummary>,
}

#[derive(Serialize, ToSchema)]
struct WorkflowSummary {
    id: String,
    name: String,
    description: String,
    engine_count: usize,
}

#[derive(Serialize, ToSchema)]
struct EngineInfoResponse {
    engine_id: String,
    engine_name: String,
    required_phase: u8,
}

#[derive(Serialize, ToSchema)]
struct EngineListResponse {
    engines: Vec<String>,
}

#[derive(Serialize, ToSchema)]
struct WorkflowListResponse {
    workflows: Vec<WorkflowSummary>,
}

#[derive(Serialize, ToSchema)]
struct WorkflowInfoResponse {
    id: String,
    name: String,
    description: String,
    engine_ids: Vec<String>,
}

#[derive(Serialize)]
struct VedicChartBundleResponse {
    d1: serde_json::Value,
    d9: serde_json::Value,
}

#[derive(Serialize, ToSchema)]
struct VedicChartBundleResponseSchema {
    #[schema(value_type = Object)]
    d1: serde_json::Value,
    #[schema(value_type = Object)]
    d9: serde_json::Value,
}

#[derive(Serialize, ToSchema)]
struct ApiEngineOutputResponse {
    #[serde(flatten)]
    output: EngineOutput,
    envelope_version: String,
}

impl From<EngineOutput> for ApiEngineOutputResponse {
    fn from(output: EngineOutput) -> Self {
        Self {
            output,
            envelope_version: "1".to_string(),
        }
    }
}

#[derive(Serialize, ToSchema)]
struct ApiWorkflowResultResponse {
    #[serde(flatten)]
    workflow: WorkflowResult,
    engine_results: HashMap<String, EngineOutput>,
}

impl From<WorkflowResult> for ApiWorkflowResultResponse {
    fn from(workflow: WorkflowResult) -> Self {
        let engine_results = workflow.engine_outputs.clone();
        Self {
            workflow,
            engine_results,
        }
    }
}

#[derive(Serialize, ToSchema)]
struct FaceUploadResponse {
    engine_id: String,
    witness_prompt: String,
    analysis: serde_json::Value,
    is_mock_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct BirthBlueprintSynthesisSchema {
    themes: Vec<String>,
    alignments: Vec<String>,
    tensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct DailyPracticeSynthesisSchema {
    practices: Vec<String>,
    timing_notes: Vec<String>,
    cautions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct DecisionSupportSynthesisSchema {
    options_summary: Vec<String>,
    decision_lenses: Vec<String>,
    risks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct SelfInquirySynthesisSchema {
    inquiry_threads: Vec<String>,
    shadow_themes: Vec<String>,
    reflection_prompts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct CreativeExpressionSynthesisSchema {
    creative_seeds: Vec<String>,
    modality_alignment: Vec<String>,
    blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct FullSpectrumSynthesisSchema {
    integrative_themes: Vec<String>,
    cross_system_alignments: Vec<String>,
    developmental_edges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct BirthBlueprintWorkflowResultSchema {
    workflow_id: String,
    engine_outputs: std::collections::HashMap<String, EngineOutput>,
    synthesis: Option<BirthBlueprintSynthesisSchema>,
    total_time_ms: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct DailyPracticeWorkflowResultSchema {
    workflow_id: String,
    engine_outputs: std::collections::HashMap<String, EngineOutput>,
    synthesis: Option<DailyPracticeSynthesisSchema>,
    total_time_ms: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct DecisionSupportWorkflowResultSchema {
    workflow_id: String,
    engine_outputs: std::collections::HashMap<String, EngineOutput>,
    synthesis: Option<DecisionSupportSynthesisSchema>,
    total_time_ms: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct SelfInquiryWorkflowResultSchema {
    workflow_id: String,
    engine_outputs: std::collections::HashMap<String, EngineOutput>,
    synthesis: Option<SelfInquirySynthesisSchema>,
    total_time_ms: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct CreativeExpressionWorkflowResultSchema {
    workflow_id: String,
    engine_outputs: std::collections::HashMap<String, EngineOutput>,
    synthesis: Option<CreativeExpressionSynthesisSchema>,
    total_time_ms: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct FullSpectrumWorkflowResultSchema {
    workflow_id: String,
    engine_outputs: std::collections::HashMap<String, EngineOutput>,
    synthesis: Option<FullSpectrumSynthesisSchema>,
    total_time_ms: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

struct ResolvedBirthDetails {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    latitude: f64,
    longitude: f64,
    timezone_offset_hours: f64,
}

fn chart_validation_error(
    message: impl Into<String>,
    details: Option<serde_json::Value>,
) -> (StatusCode, Json<ErrorResponse>) {
    ErrorMapper::response(
        StatusCode::UNPROCESSABLE_ENTITY,
        "VALIDATION_ERROR",
        message,
        details,
    )
}

fn parse_explicit_timezone_offset(tz: &str) -> Option<f64> {
    if !(tz.starts_with('+') || tz.starts_with('-')) {
        return None;
    }

    let parts: Vec<&str> = tz[1..].split(':').collect();
    let sign = if tz.starts_with('-') { -1.0 } else { 1.0 };
    let hours = parts.first()?.parse::<f64>().ok()?;
    let minutes = parts
        .get(1)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);

    Some(sign * (hours + (minutes / 60.0)))
}

fn resolve_timezone_offset(
    timezone: &str,
    local_datetime: NaiveDateTime,
) -> Result<f64, (StatusCode, Json<ErrorResponse>)> {
    let trimmed = timezone.trim();
    if trimmed.is_empty() {
        return Err(chart_validation_error(
            "birth_data.timezone is required for chart calculations.",
            Some(serde_json::json!({ "field": "birth_data.timezone" })),
        ));
    }

    if trimmed.eq_ignore_ascii_case("utc") || trimmed.eq_ignore_ascii_case("gmt") {
        return Ok(0.0);
    }

    if let Some(offset) = parse_explicit_timezone_offset(trimmed) {
        return Ok(offset);
    }

    let timezone: Tz = trimmed.parse().map_err(|_| {
        chart_validation_error(
            format!("Unsupported IANA timezone '{}'.", trimmed),
            Some(serde_json::json!({ "field": "birth_data.timezone" })),
        )
    })?;

    match timezone.from_local_datetime(&local_datetime) {
        LocalResult::Single(value) | LocalResult::Ambiguous(value, _) => {
            Ok(value.offset().fix().local_minus_utc() as f64 / 3600.0)
        }
        LocalResult::None => Err(chart_validation_error(
            format!(
                "Timezone '{}' does not map to a valid local time for the provided birth datetime.",
                trimmed
            ),
            Some(serde_json::json!({ "field": "birth_data.timezone" })),
        )),
    }
}

fn resolve_birth_details(
    input: &EngineInput,
) -> Result<ResolvedBirthDetails, (StatusCode, Json<ErrorResponse>)> {
    let birth_data = input.birth_data.as_ref().ok_or_else(|| {
        chart_validation_error(
            "birth_data is required to generate a Vedic chart bundle.",
            Some(serde_json::json!({ "field": "birth_data" })),
        )
    })?;

    let time_string = birth_data.time.as_deref().ok_or_else(|| {
        chart_validation_error(
            "birth_data.time is required to generate precise D1 and D9 chart geometry.",
            Some(serde_json::json!({ "field": "birth_data.time" })),
        )
    })?;

    let date = NaiveDate::parse_from_str(&birth_data.date, "%Y-%m-%d").map_err(|error| {
        chart_validation_error(
            format!("birth_data.date must use YYYY-MM-DD: {}", error),
            Some(serde_json::json!({ "field": "birth_data.date" })),
        )
    })?;

    let time = NaiveTime::parse_from_str(time_string, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(time_string, "%H:%M:%S"))
        .map_err(|error| {
            chart_validation_error(
                format!("birth_data.time must use HH:MM or HH:MM:SS: {}", error),
                Some(serde_json::json!({ "field": "birth_data.time" })),
            )
        })?;

    let local_datetime = NaiveDateTime::new(date, time);
    let timezone_offset_hours = resolve_timezone_offset(&birth_data.timezone, local_datetime)?;

    Ok(ResolvedBirthDetails {
        year: date.year(),
        month: date.month(),
        day: date.day(),
        hour: time.hour(),
        minute: time.minute(),
        second: time.second(),
        latitude: birth_data.latitude,
        longitude: birth_data.longitude,
        timezone_offset_hours,
    })
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /health -- Enhanced liveness probe with uptime and resource counts
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let uptime = state.startup_time.elapsed().as_secs();
    let engines_loaded = state.orchestrator.list_engines().len();
    let workflows_loaded = state.orchestrator.list_workflows().len();

    Json(HealthResponse {
        status: "ok".to_string(),
        version: "3.0.0".to_string(),
        uptime_seconds: uptime,
        engines_loaded,
        workflows_loaded,
    })
}

/// GET /ready -- Readiness probe checking dependencies
#[utoipa::path(
    get,
    path = "/ready",
    tag = "health",
    responses(
        (status = 200, description = "Service is ready", body = ReadinessResponse),
        (status = 503, description = "Service is not ready", body = ReadinessResponse),
    )
)]
async fn readiness_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Check Redis/cache health
    let redis_status = match state.cache.health_check().await {
        Ok(true) => "ok",
        _ => "down",
    };

    let postgres_status = match state.auth.pool() {
        Some(pool) => match tokio::time::timeout(
            Duration::from_secs(2),
            sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(pool),
        )
        .await
        {
            Ok(Ok(_)) => "ok",
            _ => "down",
        },
        None => "disabled",
    };

    // Check orchestrator readiness
    let orchestrator_status = match state.orchestrator.is_ready().await {
        Ok(true) => "ready",
        _ => "not_ready",
    };

    let (bridge_status, bridge_engines, bridge_failed_engines) =
        match state.bridge_manager.readiness_status().await {
            Ok(status) => (
                if status.failed_engines.is_empty() {
                    "available".to_string()
                } else {
                    "degraded".to_string()
                },
                status
                    .engines
                    .into_iter()
                    .map(|engine| BridgeEngineStatus {
                        engine_id: engine.engine_id,
                        healthy: engine.healthy,
                        detail: engine.detail,
                        latency_ms: engine.latency_ms,
                    })
                    .collect(),
                status.failed_engines,
            ),
            Err(err) => (
                "unreachable".to_string(),
                vec![BridgeEngineStatus {
                    engine_id: "ts-sidecar".to_string(),
                    healthy: false,
                    detail: err.to_string(),
                    latency_ms: 0,
                }],
                vec!["ts-sidecar".to_string()],
            ),
        };

    let overall_ready = redis_status == "ok"
        && postgres_status == "ok"
        && orchestrator_status == "ready"
        && bridge_status == "available";
    let overall_status = if overall_ready { "ready" } else { "not_ready" };

    let response = ReadinessResponse {
        redis: redis_status.to_string(),
        postgres: postgres_status.to_string(),
        orchestrator: orchestrator_status.to_string(),
        bridge_status,
        bridge_engines,
        bridge_failed_engines,
        overall_status: overall_status.to_string(),
    };

    if overall_ready {
        (StatusCode::OK, Json(response)).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response()
    }
}

/// GET /metrics -- Prometheus metrics endpoint
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Sync per-layer cache stats before encoding so Prometheus sees fresh values
    let cache_stats = state.cache.get_stats().await;
    state.metrics.update_cache_layer_stats(
        cache_stats.l1_hits,
        cache_stats.l2_hits,
        cache_stats.l3_hits,
        cache_stats.cache_misses,
        cache_stats.total_requests,
    );

    match state.metrics.get_metrics_text() {
        Ok(text) => (StatusCode::OK, text).into_response(),
        Err(e) => {
            tracing::error!("Failed to encode metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal metrics error".to_string(),
            )
                .into_response()
        }
    }
}

/// GET /api/v1/status -- list registered engines and workflows
#[utoipa::path(
    get,
    path = "/api/v1/status",
    tag = "health",
    responses(
        (status = 200, description = "List of engines and workflows", body = StatusResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn status_handler(State(state): State<AppState>) -> Json<StatusResponse> {
    let engines = state.orchestrator.list_engines();
    let workflows = state
        .orchestrator
        .list_workflows()
        .iter()
        .map(|w| WorkflowSummary {
            id: w.id.clone(),
            name: w.name.clone(),
            description: w.description.clone(),
            engine_count: w.engine_ids.len(),
        })
        .collect();

    Json(StatusResponse { engines, workflows })
}

// ---------------------------------------------------------------------------
// Native Vedic chart computation (Swiss Ephemeris, Lahiri ayanamsa, whole-sign)
// ---------------------------------------------------------------------------

const NAKSHATRA_NAMES: [&str; 27] = [
    "Ashwini",
    "Bharani",
    "Krittika",
    "Rohini",
    "Mrigashira",
    "Ardra",
    "Punarvasu",
    "Pushya",
    "Ashlesha",
    "Magha",
    "Purva Phalguni",
    "Uttara Phalguni",
    "Hasta",
    "Chitra",
    "Swati",
    "Vishakha",
    "Anuradha",
    "Jyeshtha",
    "Mula",
    "Purva Ashadha",
    "Uttara Ashadha",
    "Shravana",
    "Dhanishtha",
    "Shatabhisha",
    "Purva Bhadrapada",
    "Uttara Bhadrapada",
    "Revati",
];

const VEDIC_SIGN_NAMES: [&str; 12] = [
    "aries",
    "taurus",
    "gemini",
    "cancer",
    "leo",
    "virgo",
    "libra",
    "scorpio",
    "sagittarius",
    "capricorn",
    "aquarius",
    "pisces",
];

const VEDIC_SIGN_LORDS: [&str; 12] = [
    "Mars", "Venus", "Mercury", "Moon", "Sun", "Mercury", "Venus", "Mars", "Jupiter", "Saturn",
    "Saturn", "Jupiter",
];

/// 9 classical Vedic planets mapped to their familiar names.
const VEDIC_PLANETS: &[(HDPlanet, &str)] = &[
    (HDPlanet::Sun, "Sun"),
    (HDPlanet::Moon, "Moon"),
    (HDPlanet::Mars, "Mars"),
    (HDPlanet::Mercury, "Mercury"),
    (HDPlanet::Jupiter, "Jupiter"),
    (HDPlanet::Venus, "Venus"),
    (HDPlanet::Saturn, "Saturn"),
    (HDPlanet::NorthNode, "Rahu"),
    (HDPlanet::SouthNode, "Ketu"),
];

/// Lahiri (Chitrapaksha) ayanamsa in degrees for a given Julian Day.
fn lahiri_ayanamsa(jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    23.85 + t * 1.3968 // ~50.3"/year drift
}

/// Julian Day (UT) from a UTC chrono DateTime.
fn jd_from_utc(dt: &chrono::DateTime<chrono::Utc>) -> f64 {
    // J2000.0 = 2000-01-01 12:00:00 UTC = unix timestamp 946728000
    2451545.0 + (dt.timestamp() - 946728000) as f64 / 86_400.0
}

/// Tropical ascendant in degrees [0,360) for a given Julian Day and location.
fn tropical_ascendant(jd: f64, lat: f64, lng: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    // GMST in degrees
    let gmst = (280.460_618_37 + 360.985_647_366_29 * (jd - 2451545.0) + 0.000_387_933 * t * t
        - t * t * t / 38_710_000.0)
        .rem_euclid(360.0);
    let lst = (gmst + lng).rem_euclid(360.0);
    let ramc = lst.to_radians();
    let eps = (23.439_291_111 - 0.013_004_167 * t).to_radians();
    let lat_r = lat.to_radians();
    // Standard Ascendant: tan(ASC) = cos(RAMC) / -(sin(RAMC)*cos(ε) + tan(φ)*sin(ε))
    let num = ramc.cos();
    let den = -(ramc.sin() * eps.cos() + lat_r.tan() * eps.sin());
    f64::atan2(num, den).to_degrees().rem_euclid(360.0)
}

/// (nakshatra name, pada 1-4) from a sidereal longitude [0,360).
fn nakshatra_and_pada(sidereal: f64) -> (&'static str, u8) {
    let idx = ((sidereal * 27.0 / 360.0) as usize).min(26);
    let frac = sidereal * 27.0 / 360.0 - idx as f64;
    let pada = ((frac * 4.0) as u8 + 1).min(4);
    (NAKSHATRA_NAMES[idx], pada)
}

/// Navamsa (D9) sign index [0,11] from D1 sign index and degree within that sign.
fn navamsa_sign_idx(d1_sign: usize, degree_in_sign: f64) -> usize {
    let nav_num = (degree_in_sign / (30.0 / 9.0)) as usize; // 0..8
    let base = match d1_sign % 3 {
        0 => 0, // movable → starts at Aries
        1 => 4, // fixed   → starts at Leo
        _ => 8, // dual    → starts at Sagittarius
    };
    (base + nav_num) % 12
}

/// Build a JSON D1 (Rashi) chart from Swiss Ephemeris tropical planet positions.
fn build_d1_chart(
    birth: &ResolvedBirthDetails,
    jd: f64,
    ayanamsa: f64,
    planet_positions: &[(HDPlanet, engine_human_design::ephemeris::PlanetPosition)],
) -> serde_json::Value {
    let trop_asc = tropical_ascendant(jd, birth.latitude, birth.longitude);
    let sid_asc = (trop_asc - ayanamsa).rem_euclid(360.0);
    let asc_sign_idx = (sid_asc / 30.0) as usize % 12;
    let asc_degree = sid_asc % 30.0;
    let (asc_nak, asc_pada) = nakshatra_and_pada(sid_asc);

    let mut planets_json: Vec<serde_json::Value> = Vec::new();
    let mut moon_json = serde_json::json!({ "sign": "aries", "degree": 0.0, "nakshatra": "Ashwini", "pada": 1, "rashi_lord": "Mars" });
    let mut sun_sid = 0.0_f64;
    let mut moon_sid = 0.0_f64;

    for (hd_planet, vedic_name) in VEDIC_PLANETS {
        let Some((_, pp)) = planet_positions
            .iter()
            .find(|(p, _)| *p as i32 == *hd_planet as i32)
        else {
            continue;
        };
        let sid_lon = (pp.longitude - ayanamsa).rem_euclid(360.0);
        let sign_idx = (sid_lon / 30.0) as usize % 12;
        let degree = sid_lon % 30.0;
        let (nak, pada) = nakshatra_and_pada(sid_lon);
        let house = ((sign_idx + 12 - asc_sign_idx) % 12 + 1) as u8;

        planets_json.push(serde_json::json!({
            "name": vedic_name,
            "longitude": sid_lon,
            "sign": VEDIC_SIGN_NAMES[sign_idx],
            "degree": degree,
            "minutes": (degree.fract() * 60.0).floor(),
            "house": house,
            "is_retrograde": pp.speed < 0.0,
            "is_combust": false,
            "nakshatra": nak,
            "pada": pada,
            "speed": pp.speed,
            "latitude": pp.latitude,
        }));

        if *vedic_name == "Moon" {
            moon_sid = sid_lon;
            moon_json = serde_json::json!({
                "sign": VEDIC_SIGN_NAMES[sign_idx],
                "degree": degree,
                "nakshatra": nak,
                "pada": pada,
                "rashi_lord": VEDIC_SIGN_LORDS[sign_idx],
            });
        }
        if *vedic_name == "Sun" {
            sun_sid = sid_lon;
        }
    }

    let houses: Vec<serde_json::Value> = (0u8..12).map(|i| {
        let sign = (asc_sign_idx + i as usize) % 12;
        let hn = i + 1;
        serde_json::json!({
            "number": hn,
            "sign": VEDIC_SIGN_NAMES[sign],
            "cusp": sign as f64 * 30.0,
            "degree": 0.0,
            "house_type": match hn { 1|5|9 => "dharma", 2|6|10 => "artha", 3|7|11 => "kama", _ => "moksha" },
            "is_kendra": matches!(hn, 1|4|7|10),
            "is_panapara": matches!(hn, 2|5|8|11),
            "is_apoklima": matches!(hn, 3|6|9|12),
        })
    }).collect();

    let pof = (sid_asc + moon_sid - sun_sid).rem_euclid(360.0);

    serde_json::json!({
        "native": {
            "birth_date": format!("{:04}-{:02}-{:02}", birth.year, birth.month, birth.day),
            "birth_time": format!("{:02}:{:02}:{:02}", birth.hour, birth.minute, birth.second),
            "latitude": birth.latitude,
            "longitude": birth.longitude,
            "timezone": birth.timezone_offset_hours,
        },
        "ayanamsa": ayanamsa,
        "house_system": "whole_sign",
        "planets": planets_json,
        "houses": houses,
        "ascendant": { "sign": VEDIC_SIGN_NAMES[asc_sign_idx], "degree": asc_degree, "nakshatra": asc_nak, "pada": asc_pada },
        "moon": moon_json,
        "special_points": { "lagna": sid_asc, "midheaven": serde_json::Value::Null, "part_of_fortune": pof },
        "notes": ["Native computation via Swiss Ephemeris (Lahiri ayanamsa, whole-sign houses)"],
    })
}

/// Build a JSON D9 (Navamsa) chart from the D1 chart JSON.
fn build_d9_chart(birth: &ResolvedBirthDetails, d1: &serde_json::Value) -> serde_json::Value {
    let asc_sign_name = d1["ascendant"]["sign"].as_str().unwrap_or("aries");
    let asc_sign_idx = VEDIC_SIGN_NAMES
        .iter()
        .position(|&s| s == asc_sign_name)
        .unwrap_or(0);
    let asc_degree = d1["ascendant"]["degree"].as_f64().unwrap_or(0.0);
    let d9_lagna_idx = navamsa_sign_idx(asc_sign_idx, asc_degree);

    let empty = vec![];
    let planets = d1["planets"].as_array().unwrap_or(&empty);

    let mut vargottama: Vec<&str> = Vec::new();
    let mut navamsa_positions: Vec<serde_json::Value> = Vec::new();

    for planet in planets {
        let name = planet["name"].as_str().unwrap_or("");
        let d1_sign = VEDIC_SIGN_NAMES
            .iter()
            .position(|&s| s == planet["sign"].as_str().unwrap_or(""))
            .unwrap_or(0);
        let degree = planet["degree"].as_f64().unwrap_or(0.0);
        let d9_sign = navamsa_sign_idx(d1_sign, degree);
        let nav_deg = (degree % (30.0 / 9.0)) * 9.0;
        let is_varg = d1_sign == d9_sign;
        if is_varg {
            vargottama.push(name);
        }
        navamsa_positions.push(serde_json::json!({
            "planet": name,
            "sign": VEDIC_SIGN_NAMES[d9_sign],
            "degree": nav_deg,
            "is_vargottama": is_varg,
        }));
    }

    serde_json::json!({
        "source": d1["native"].clone(),
        "navamsa_positions": navamsa_positions,
        "vargottama": vargottama,
        "d9_lagna": VEDIC_SIGN_NAMES[d9_lagna_idx],
    })
}

/// POST /api/v1/charts/vedic -- return the D1/D9 chart bundle used by AstroLens
#[utoipa::path(
    post,
    path = "/api/v1/charts/vedic",
    tag = "charts",
    request_body = EngineInput,
    responses(
        (status = 200, description = "Vedic D1 and D9 chart bundle", body = VedicChartBundleResponseSchema),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
        (status = 500, description = "Native ephemeris calculation failed", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn vedic_chart_handler(
    State(state): State<AppState>,
    Json(input): Json<EngineInput>,
) -> Result<Json<VedicChartBundleResponse>, (StatusCode, Json<ErrorResponse>)> {
    let metric_label = "chart:vedic";
    let start = Instant::now();

    let birth = match resolve_birth_details(&input) {
        Ok(v) => v,
        Err(e) => {
            state.metrics.record_engine_calculation_with_status(
                metric_label,
                "failure",
                start.elapsed().as_secs_f64(),
            );
            state
                .metrics
                .record_engine_calculation_error(metric_label, "validation_error");
            return Err(e);
        }
    };

    // Convert local birth time to UTC for Swiss Ephemeris
    let offset_secs = (birth.timezone_offset_hours * 3600.0) as i64;
    let naive_dt = chrono::NaiveDateTime::new(
        chrono::NaiveDate::from_ymd_opt(birth.year, birth.month, birth.day)
            .expect("validated date"),
        chrono::NaiveTime::from_hms_opt(birth.hour, birth.minute, birth.second)
            .expect("validated time"),
    );
    let utc_dt = (naive_dt - chrono::Duration::seconds(offset_secs)).and_utc();

    // Ephemeris calls are synchronous (C library with global mutex); run off the async thread.
    let birth_for_task = ResolvedBirthDetails {
        year: birth.year,
        month: birth.month,
        day: birth.day,
        hour: birth.hour,
        minute: birth.minute,
        second: birth.second,
        latitude: birth.latitude,
        longitude: birth.longitude,
        timezone_offset_hours: birth.timezone_offset_hours,
    };

    let chart_result = tokio::task::spawn_blocking(move || {
        let ephe = EphemerisCalculator::new("");
        let jd = jd_from_utc(&utc_dt);
        let ayanamsa = lahiri_ayanamsa(jd);

        let mut positions: Vec<(HDPlanet, engine_human_design::ephemeris::PlanetPosition)> =
            Vec::new();
        for (planet, _) in VEDIC_PLANETS {
            match ephe.get_planet_position(*planet, &utc_dt) {
                Ok(pos) => positions.push((*planet, pos)),
                Err(e) => return Err(format!("Planet {:?}: {}", planet, e)),
            }
        }

        let d1 = build_d1_chart(&birth_for_task, jd, ayanamsa, &positions);
        let d9 = build_d9_chart(&birth_for_task, &d1);
        Ok((d1, d9))
    })
    .await
    .map_err(|_| {
        state.metrics.record_engine_calculation_with_status(
            metric_label,
            "failure",
            start.elapsed().as_secs_f64(),
        );
        state
            .metrics
            .record_engine_calculation_error(metric_label, "task_panic");
        ErrorMapper::response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CALCULATION_ERROR",
            "Vedic chart task panicked",
            None,
        )
    })?
    .map_err(|e| {
        state.metrics.record_engine_calculation_with_status(
            metric_label,
            "failure",
            start.elapsed().as_secs_f64(),
        );
        state
            .metrics
            .record_engine_calculation_error(metric_label, "ephemeris_error");
        ErrorMapper::response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CALCULATION_ERROR",
            e,
            None,
        )
    })?;

    state.metrics.record_engine_calculation_with_status(
        metric_label,
        "success",
        start.elapsed().as_secs_f64(),
    );
    Ok(Json(VedicChartBundleResponse {
        d1: chart_result.0,
        d9: chart_result.1,
    }))
}

/// POST /api/v1/engines/:engine_id/calculate -- execute a single engine
#[utoipa::path(
    post,
    path = "/api/v1/engines/{engine_id}/calculate",
    tag = "engines",
    params(
        ("engine_id" = String, Path, description = "Engine identifier (e.g., 'panchanga', 'numerology', 'biorhythm')"),
    ),
    request_body = EngineInput,
    responses(
        (status = 200, description = "Calculation successful", body = ApiEngineOutputResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Insufficient consciousness phase", body = ErrorResponse),
        (status = 404, description = "Engine not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn calculate_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(engine_id): Path<String>,
    Json(input): Json<EngineInput>,
) -> Result<Json<ApiEngineOutputResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = Instant::now();

    // Capture input for persistence before it's moved into the engine
    let input_json = serde_json::to_value(&input).unwrap_or_default();

    // Extract birth_data for profile auto-population (before input is moved)
    let birth_data_for_profile = input.birth_data.clone();

    let result = state
        .orchestrator
        .execute_engine(&engine_id, input, user.consciousness_level)
        .await;

    let duration_secs = start.elapsed().as_secs_f64();
    let duration_ms = duration_secs * 1000.0;
    let user_id_str = user.user_id.clone();

    billing::emit_usage_event(&user_id_str, &engine_id, &user.tier);

    match result {
        Ok(output) => {
            state.metrics.record_engine_calculation_with_status(
                &engine_id,
                "success",
                duration_secs,
            );

            // Fire-and-forget: persist reading, log usage, award XP
            if let Ok(uid) = uuid::Uuid::parse_str(&user_id_str) {
                let input_hash = format!("{:x}", Sha256::digest(input_json.to_string().as_bytes()));
                let reading = NewReading {
                    user_id: uid,
                    engine_id: engine_id.clone(),
                    workflow_id: None,
                    input_hash,
                    input_data: input_json,
                    result_data: serde_json::to_value(&output).unwrap_or_default(),
                    witness_prompt: Some(output.witness_prompt.clone()),
                    consciousness_level: output.consciousness_level as i16,
                    calculation_time_ms: Some(duration_ms),
                    client_event_id: None,
                    client_device_id: None,
                    device_platform: None,
                    device_app_version: None,
                };
                let readings_repo = state.readings_repository.clone();
                let usage_repo = state.usage_repository.clone();
                let user_repo = state.user_repository.clone();
                let eid = engine_id.clone();
                let bd = birth_data_for_profile.clone();
                tokio::spawn(async move {
                    if let Some(repo) = readings_repo {
                        if let Err(e) = repo.save_reading(&reading).await {
                            tracing::warn!("Failed to persist reading: {}", e);
                        }
                    }
                    if let Some(repo) = usage_repo {
                        if let Err(e) = repo
                            .log_usage(uid, Some(&eid), None, "success", duration_ms as i32)
                            .await
                        {
                            tracing::warn!("Failed to log usage: {}", e);
                        }
                    }
                    // Auto-populate user profile from birth_data on first use
                    if let Some(ref birth_data) = bd {
                        if let Err(e) = user_repo
                            .ensure_profile_from_birth_data(
                                uid,
                                birth_data.name.as_deref(),
                                &birth_data.date,
                                birth_data.time.as_deref(),
                                birth_data.latitude,
                                birth_data.longitude,
                                &birth_data.timezone,
                            )
                            .await
                        {
                            tracing::warn!("Failed to auto-populate user profile: {}", e);
                        }
                    }
                    if let Err(e) = user_repo
                        .add_experience(uid, 10, "engine_calculation")
                        .await
                    {
                        tracing::warn!("Failed to award XP: {}", e);
                    }
                    // Auto-promote consciousness level based on reading count
                    if let Err(e) = user_repo.promote_consciousness_level(uid).await {
                        tracing::warn!("Failed to check level promotion: {}", e);
                    }
                });
            }

            Ok(Json(output.into()))
        }
        Err(e) => {
            state.metrics.record_engine_calculation_with_status(
                &engine_id,
                "failure",
                duration_secs,
            );

            // Log failed usage
            if let Ok(uid) = uuid::Uuid::parse_str(&user_id_str) {
                let usage_repo = state.usage_repository.clone();
                let eid = engine_id.clone();
                tokio::spawn(async move {
                    if let Some(repo) = usage_repo {
                        let _ = repo
                            .log_usage(uid, Some(&eid), None, "failure", duration_ms as i32)
                            .await;
                    }
                });
            }

            let error_type = match &e {
                EngineError::EngineNotFound(_) => "not_found",
                EngineError::PhaseAccessDenied { .. } => "forbidden",
                EngineError::AuthError(_) => "unauthorized",
                EngineError::RateLimitExceeded => "rate_limit",
                EngineError::ValidationError(_) => "validation_error",
                _ => "internal_error",
            };

            state
                .metrics
                .record_engine_calculation_error(&engine_id, error_type);
            Err(ErrorMapper::map(e))
        }
    }
}

/// POST /api/v1/engines/face-reading/upload -- upload image and run face-reading analysis
#[utoipa::path(
    post,
    path = "/api/v1/engines/face-reading/upload",
    tag = "engines",
    request_body(content = String, content_type = "multipart/form-data", description = "Multipart form-data with file field named `file` or `image`"),
    responses(
        (status = 200, description = "Face analysis successful", body = FaceUploadResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Insufficient consciousness phase", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn face_reading_upload_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<Json<FaceUploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut image_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ErrorMapper::response(
            StatusCode::BAD_REQUEST,
            "INVALID_MULTIPART",
            format!("Invalid multipart payload: {}", e),
            None,
        )
    })? {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" || field_name == "image" {
            let bytes = field.bytes().await.map_err(|e| {
                ErrorMapper::response(
                    StatusCode::BAD_REQUEST,
                    "INVALID_UPLOAD",
                    format!("Failed to read uploaded file: {}", e),
                    None,
                )
            })?;
            image_bytes = Some(bytes.to_vec());
            break;
        }
    }

    let image_bytes = image_bytes.ok_or_else(|| {
        ErrorMapper::response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "MISSING_IMAGE_FILE",
            "No image file found. Provide multipart field named `file` or `image`.",
            None,
        )
    })?;

    let mut options = std::collections::HashMap::new();
    options.insert(
        "image_data".to_string(),
        Value::String(String::from_utf8_lossy(&image_bytes).to_string()),
    );

    let input = EngineInput {
        birth_data: None,
        current_time: chrono::Utc::now(),
        location: None,
        precision: Precision::Standard,
        options,
    };

    let output = state
        .orchestrator
        .execute_engine("face-reading", input, user.consciousness_level)
        .await
        .map_err(ErrorMapper::map)?;

    let analysis = output
        .result
        .get("analysis")
        .cloned()
        .unwrap_or_else(|| Value::Object(Default::default()));

    let is_mock_data = analysis
        .get("is_mock_data")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    Ok(Json(FaceUploadResponse {
        engine_id: "face-reading".to_string(),
        witness_prompt: output.witness_prompt,
        analysis,
        is_mock_data,
    }))
}

/// POST /api/v1/engines/:engine_id/validate -- validate an engine output
#[utoipa::path(
    post,
    path = "/api/v1/engines/{engine_id}/validate",
    tag = "engines",
    params(
        ("engine_id" = String, Path, description = "Engine identifier"),
    ),
    request_body = EngineOutput,
    responses(
        (status = 200, description = "Validation result", body = ValidationResult),
        (status = 404, description = "Engine not found", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn validate_handler(
    State(state): State<AppState>,
    Path(engine_id): Path<String>,
    Json(output): Json<EngineOutput>,
) -> Result<Json<noesis_core::ValidationResult>, (StatusCode, Json<ErrorResponse>)> {
    let engine = state
        .orchestrator
        .registry()
        .get(&engine_id)
        .ok_or_else(|| {
            ErrorMapper::response(
                StatusCode::NOT_FOUND,
                "ENGINE_NOT_FOUND",
                format!("Engine '{}' not found", engine_id),
                Some(serde_json::json!({ "engine_id": engine_id })),
            )
        })?;

    engine
        .validate(&output)
        .await
        .map(Json)
        .map_err(ErrorMapper::map)
}

/// GET /api/v1/engines/:engine_id/info -- engine metadata
#[utoipa::path(
    get,
    path = "/api/v1/engines/{engine_id}/info",
    tag = "engines",
    params(
        ("engine_id" = String, Path, description = "Engine identifier"),
    ),
    responses(
        (status = 200, description = "Engine information", body = EngineInfoResponse),
        (status = 404, description = "Engine not found", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn engine_info_handler(
    State(state): State<AppState>,
    Path(engine_id): Path<String>,
) -> Result<Json<EngineInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    let engine = state
        .orchestrator
        .registry()
        .get(&engine_id)
        .ok_or_else(|| {
            ErrorMapper::response(
                StatusCode::NOT_FOUND,
                "ENGINE_NOT_FOUND",
                format!("Engine '{}' not found", engine_id),
                Some(serde_json::json!({ "engine_id": engine_id })),
            )
        })?;

    Ok(Json(EngineInfoResponse {
        engine_id: engine.engine_id().to_string(),
        engine_name: engine.engine_name().to_string(),
        required_phase: engine.required_phase(),
    }))
}

/// GET /api/v1/engines -- list all engine IDs
#[utoipa::path(
    get,
    path = "/api/v1/engines",
    tag = "engines",
    responses(
        (status = 200, description = "List of available engines", body = EngineListResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn list_engines_handler(State(state): State<AppState>) -> Json<EngineListResponse> {
    Json(EngineListResponse {
        engines: state.orchestrator.list_engines(),
    })
}

/// POST /api/v1/workflows/:workflow_id/execute -- execute a workflow
#[utoipa::path(
    post,
    path = "/api/v1/workflows/{workflow_id}/execute",
    tag = "workflows",
    params(
        ("workflow_id" = String, Path, description = "Workflow identifier"),
    ),
    request_body = EngineInput,
    responses(
        (status = 200, description = "Workflow execution successful", body = ApiWorkflowResultResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Insufficient consciousness phase", body = ErrorResponse),
        (status = 404, description = "Workflow not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn workflow_execute_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(workflow_id): Path<String>,
    Json(input): Json<EngineInput>,
) -> Result<Json<ApiWorkflowResultResponse>, (StatusCode, Json<ErrorResponse>)> {
    execute_workflow_by_id(state, user, workflow_id, input).await
}

async fn execute_workflow_by_id(
    state: AppState,
    user: AuthUser,
    workflow_id: String,
    input: EngineInput,
) -> Result<Json<ApiWorkflowResultResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = Instant::now();

    // Capture input for persistence before it's moved into the workflow
    let input_json = serde_json::to_value(&input).unwrap_or_default();
    let birth_data_for_profile = input.birth_data.clone();

    // Execute workflow with user's consciousness level
    let result = state
        .orchestrator
        .execute_workflow(&workflow_id, input, user.consciousness_level)
        .await;

    let duration_secs = start.elapsed().as_secs_f64();
    let duration_ms = duration_secs * 1000.0;
    let user_id_str = user.user_id.clone();

    // Use workflow_id prefixed to distinguish from engine calculations
    let workflow_label = format!("workflow:{}", workflow_id);

    billing::emit_usage_event(&user_id_str, &workflow_label, &user.tier);

    match result {
        Ok(workflow_result) => {
            state.metrics.record_engine_calculation_with_status(
                &workflow_label,
                "success",
                duration_secs,
            );

            // Fire-and-forget: persist reading, log usage, award XP (25 for workflow)
            if let Ok(uid) = uuid::Uuid::parse_str(&user_id_str) {
                let input_hash = format!("{:x}", Sha256::digest(input_json.to_string().as_bytes()));
                let reading = NewReading {
                    user_id: uid,
                    engine_id: format!("workflow:{}", workflow_id),
                    workflow_id: Some(workflow_id.clone()),
                    input_hash,
                    input_data: input_json,
                    result_data: serde_json::to_value(&workflow_result).unwrap_or_default(),
                    witness_prompt: None,
                    consciousness_level: user.consciousness_level as i16,
                    calculation_time_ms: Some(duration_ms),
                    client_event_id: None,
                    client_device_id: None,
                    device_platform: None,
                    device_app_version: None,
                };
                let readings_repo = state.readings_repository.clone();
                let usage_repo = state.usage_repository.clone();
                let user_repo = state.user_repository.clone();
                let wid = workflow_id.clone();
                let bd = birth_data_for_profile.clone();
                tokio::spawn(async move {
                    if let Some(repo) = readings_repo {
                        if let Err(e) = repo.save_reading(&reading).await {
                            tracing::warn!("Failed to persist workflow reading: {}", e);
                        }
                    }
                    if let Some(repo) = usage_repo {
                        if let Err(e) = repo
                            .log_usage(uid, None, Some(&wid), "success", duration_ms as i32)
                            .await
                        {
                            tracing::warn!("Failed to log workflow usage: {}", e);
                        }
                    }
                    // Auto-populate user profile from birth_data on first use
                    if let Some(ref birth_data) = bd {
                        if let Err(e) = user_repo
                            .ensure_profile_from_birth_data(
                                uid,
                                birth_data.name.as_deref(),
                                &birth_data.date,
                                birth_data.time.as_deref(),
                                birth_data.latitude,
                                birth_data.longitude,
                                &birth_data.timezone,
                            )
                            .await
                        {
                            tracing::warn!("Failed to auto-populate user profile: {}", e);
                        }
                    }
                    if let Err(e) = user_repo
                        .add_experience(uid, 25, "workflow_execution")
                        .await
                    {
                        tracing::warn!("Failed to award workflow XP: {}", e);
                    }
                    // Auto-promote consciousness level based on reading count
                    if let Err(e) = user_repo.promote_consciousness_level(uid).await {
                        tracing::warn!("Failed to check level promotion: {}", e);
                    }
                });
            }

            Ok(Json(workflow_result.into()))
        }
        Err(e) => {
            state.metrics.record_engine_calculation_with_status(
                &workflow_label,
                "failure",
                duration_secs,
            );

            // Log failed usage
            if let Ok(uid) = uuid::Uuid::parse_str(&user_id_str) {
                let usage_repo = state.usage_repository.clone();
                let wid = workflow_id.clone();
                tokio::spawn(async move {
                    if let Some(repo) = usage_repo {
                        let _ = repo
                            .log_usage(uid, None, Some(&wid), "failure", duration_ms as i32)
                            .await;
                    }
                });
            }

            let error_type = match &e {
                EngineError::WorkflowNotFound(_) => "not_found",
                EngineError::PhaseAccessDenied { .. } => "forbidden",
                EngineError::AuthError(_) => "unauthorized",
                EngineError::RateLimitExceeded => "rate_limit",
                EngineError::ValidationError(_) => "validation_error",
                _ => "internal_error",
            };

            state
                .metrics
                .record_engine_calculation_error(&workflow_label, error_type);
            Err(ErrorMapper::map(e))
        }
    }
}

/// POST /api/v1/workflows/birth-blueprint/execute -- workflow-specific OpenAPI shape
#[utoipa::path(
    post,
    path = "/api/v1/workflows/birth-blueprint/execute",
    tag = "workflows",
    request_body = EngineInput,
    responses(
        (status = 200, description = "Birth Blueprint workflow result", body = BirthBlueprintWorkflowResultSchema),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Workflow not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
#[allow(dead_code)]
async fn birth_blueprint_execute_doc(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<EngineInput>,
) -> Result<Json<ApiWorkflowResultResponse>, (StatusCode, Json<ErrorResponse>)> {
    execute_workflow_by_id(state, user, "birth-blueprint".to_string(), input).await
}

/// POST /api/v1/workflows/daily-practice/execute -- workflow-specific OpenAPI shape
#[utoipa::path(
    post,
    path = "/api/v1/workflows/daily-practice/execute",
    tag = "workflows",
    request_body = EngineInput,
    responses(
        (status = 200, description = "Daily Practice workflow result", body = DailyPracticeWorkflowResultSchema),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Workflow not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
#[allow(dead_code)]
async fn daily_practice_execute_doc(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<EngineInput>,
) -> Result<Json<ApiWorkflowResultResponse>, (StatusCode, Json<ErrorResponse>)> {
    execute_workflow_by_id(state, user, "daily-practice".to_string(), input).await
}

/// POST /api/v1/workflows/decision-support/execute -- workflow-specific OpenAPI shape
#[utoipa::path(
    post,
    path = "/api/v1/workflows/decision-support/execute",
    tag = "workflows",
    request_body = EngineInput,
    responses(
        (status = 200, description = "Decision Support workflow result", body = DecisionSupportWorkflowResultSchema),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Workflow not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
#[allow(dead_code)]
async fn decision_support_execute_doc(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<EngineInput>,
) -> Result<Json<ApiWorkflowResultResponse>, (StatusCode, Json<ErrorResponse>)> {
    execute_workflow_by_id(state, user, "decision-support".to_string(), input).await
}

/// POST /api/v1/workflows/self-inquiry/execute -- workflow-specific OpenAPI shape
#[utoipa::path(
    post,
    path = "/api/v1/workflows/self-inquiry/execute",
    tag = "workflows",
    request_body = EngineInput,
    responses(
        (status = 200, description = "Self Inquiry workflow result", body = SelfInquiryWorkflowResultSchema),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Workflow not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
#[allow(dead_code)]
async fn self_inquiry_execute_doc(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<EngineInput>,
) -> Result<Json<ApiWorkflowResultResponse>, (StatusCode, Json<ErrorResponse>)> {
    execute_workflow_by_id(state, user, "self-inquiry".to_string(), input).await
}

/// POST /api/v1/workflows/creative-expression/execute -- workflow-specific OpenAPI shape
#[utoipa::path(
    post,
    path = "/api/v1/workflows/creative-expression/execute",
    tag = "workflows",
    request_body = EngineInput,
    responses(
        (status = 200, description = "Creative Expression workflow result", body = CreativeExpressionWorkflowResultSchema),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Workflow not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
#[allow(dead_code)]
async fn creative_expression_execute_doc(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<EngineInput>,
) -> Result<Json<ApiWorkflowResultResponse>, (StatusCode, Json<ErrorResponse>)> {
    execute_workflow_by_id(state, user, "creative-expression".to_string(), input).await
}

/// POST /api/v1/workflows/full-spectrum/execute -- workflow-specific OpenAPI shape
#[utoipa::path(
    post,
    path = "/api/v1/workflows/full-spectrum/execute",
    tag = "workflows",
    request_body = EngineInput,
    responses(
        (status = 200, description = "Full Spectrum workflow result", body = FullSpectrumWorkflowResultSchema),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Workflow not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
#[allow(dead_code)]
async fn full_spectrum_execute_doc(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<EngineInput>,
) -> Result<Json<ApiWorkflowResultResponse>, (StatusCode, Json<ErrorResponse>)> {
    execute_workflow_by_id(state, user, "full-spectrum".to_string(), input).await
}

/// GET /api/v1/workflows -- list all workflow IDs
#[utoipa::path(
    get,
    path = "/api/v1/workflows",
    tag = "workflows",
    responses(
        (status = 200, description = "List of available workflows", body = WorkflowListResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn list_workflows_handler(State(state): State<AppState>) -> Json<WorkflowListResponse> {
    let workflows = state
        .orchestrator
        .list_workflows()
        .iter()
        .map(|w| WorkflowSummary {
            id: w.id.clone(),
            name: w.name.clone(),
            description: w.description.clone(),
            engine_count: w.engine_ids.len(),
        })
        .collect();

    Json(WorkflowListResponse { workflows })
}

/// GET /api/v1/workflows/:workflow_id/info -- workflow definition details
#[utoipa::path(
    get,
    path = "/api/v1/workflows/{workflow_id}/info",
    tag = "workflows",
    params(
        ("workflow_id" = String, Path, description = "Workflow identifier"),
    ),
    responses(
        (status = 200, description = "Workflow information", body = WorkflowInfoResponse),
        (status = 404, description = "Workflow not found", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
async fn workflow_info_handler(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
) -> Result<Json<WorkflowInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    let workflow = state
        .orchestrator
        .get_workflow(&workflow_id)
        .ok_or_else(|| {
            ErrorMapper::response(
                StatusCode::NOT_FOUND,
                "WORKFLOW_NOT_FOUND",
                format!("Workflow '{}' not found", workflow_id),
                Some(serde_json::json!({ "workflow_id": workflow_id })),
            )
        })?;

    Ok(Json(WorkflowInfoResponse {
        id: workflow.id.clone(),
        name: workflow.name.clone(),
        description: workflow.description.clone(),
        engine_ids: workflow.engine_ids.clone(),
    }))
}

// ---------------------------------------------------------------------------
// Readings / history handlers
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ReadingsQuery {
    engine_id: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize)]
struct ReadingsListResponse {
    readings: Vec<noesis_data::models::reading::Reading>,
    total: i64,
    limit: i64,
    offset: i64,
}

#[derive(Serialize)]
struct ReadingsStatsEntry {
    engine_id: String,
    count: i64,
}

#[derive(Serialize)]
struct ReadingsStatsResponse {
    stats: Vec<ReadingsStatsEntry>,
    total: i64,
}

/// GET /api/v1/readings -- paginated list of user's readings
async fn list_readings_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    axum::extract::Query(params): axum::extract::Query<ReadingsQuery>,
) -> Result<Json<ReadingsListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = state.readings_repository.as_ref().ok_or_else(|| {
        ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
    })?;

    let uid = uuid::Uuid::parse_str(&user.user_id).map_err(|_| {
        ErrorMapper::response(
            StatusCode::BAD_REQUEST,
            "INVALID_USER_ID",
            "Invalid user ID",
            None,
        )
    })?;

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let engine_filter = params.engine_id.as_deref();

    let readings = repo
        .list_readings(uid, engine_filter, limit, offset)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch readings: {}", e);
            ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to fetch readings",
                None,
            )
        })?;

    let total = repo.count_readings(uid, engine_filter).await.map_err(|e| {
        tracing::error!("Failed to count readings: {}", e);
        ErrorMapper::response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Failed to count readings",
            None,
        )
    })?;

    Ok(Json(ReadingsListResponse {
        readings,
        total,
        limit,
        offset,
    }))
}

/// GET /api/v1/readings/:reading_id -- single reading (user-scoped)
async fn get_reading_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(reading_id): Path<uuid::Uuid>,
) -> Result<Json<noesis_data::models::reading::Reading>, (StatusCode, Json<ErrorResponse>)> {
    let repo = state.readings_repository.as_ref().ok_or_else(|| {
        ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
    })?;

    let uid = uuid::Uuid::parse_str(&user.user_id).map_err(|_| {
        ErrorMapper::response(
            StatusCode::BAD_REQUEST,
            "INVALID_USER_ID",
            "Invalid user ID",
            None,
        )
    })?;

    let reading = repo.get_reading(reading_id, uid).await.map_err(|e| {
        tracing::error!("Failed to fetch reading: {}", e);
        ErrorMapper::response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Failed to fetch reading",
            None,
        )
    })?;

    reading
        .ok_or_else(|| {
            ErrorMapper::response(
                StatusCode::NOT_FOUND,
                "READING_NOT_FOUND",
                "Reading not found",
                Some(serde_json::json!({ "reading_id": reading_id })),
            )
        })
        .map(Json)
}

/// GET /api/v1/readings/stats -- count of readings per engine
async fn readings_stats_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ReadingsStatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repo = state.readings_repository.as_ref().ok_or_else(|| {
        ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
    })?;

    let uid = uuid::Uuid::parse_str(&user.user_id).map_err(|_| {
        ErrorMapper::response(
            StatusCode::BAD_REQUEST,
            "INVALID_USER_ID",
            "Invalid user ID",
            None,
        )
    })?;

    // Get total count
    let total = repo.count_readings(uid, None).await.map_err(|e| {
        tracing::error!("Failed to count readings: {}", e);
        ErrorMapper::response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Failed to count readings",
            None,
        )
    })?;

    // Get per-engine stats
    let rows = repo.count_by_engine(uid).await.map_err(|e| {
        tracing::error!("Failed to fetch stats: {}", e);
        ErrorMapper::response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB_ERROR",
            "Failed to fetch stats",
            None,
        )
    })?;

    let stats = rows
        .into_iter()
        .map(|(engine_id, count)| ReadingsStatsEntry { engine_id, count })
        .collect();

    Ok(Json(ReadingsStatsResponse { stats, total }))
}

// ---------------------------------------------------------------------------
// Legacy API handlers (backward compatibility with old Selemene API)
// ---------------------------------------------------------------------------

/// Legacy request format for Panchanga calculations from old Selemene API
#[derive(Deserialize)]
struct LegacyPanchangaRequest {
    date: String,         // YYYY-MM-DD
    time: Option<String>, // HH:MM
    latitude: f64,
    longitude: f64,
    timezone: String,
    #[serde(default)]
    name: Option<String>,
}

/// Legacy response format for Panchanga calculations
#[derive(Serialize)]
struct LegacyPanchangaResponse {
    // Preserve exact field names from old Selemene API
    tithi_index: u8,
    tithi_name: String,
    tithi_value: f64,
    nakshatra_index: u8,
    nakshatra_name: String,
    nakshatra_value: f64,
    yoga_index: u8,
    yoga_name: String,
    yoga_value: f64,
    karana_index: u8,
    karana_name: String,
    karana_value: f64,
    vara_index: u8,
    vara_name: String,
    solar_longitude: f64,
    lunar_longitude: f64,
    julian_day: f64,
}

/// POST /api/legacy/panchanga/calculate -- backward compatible Panchanga endpoint
async fn legacy_panchanga_handler(
    State(state): State<AppState>,
    Json(request): Json<LegacyPanchangaRequest>,
) -> Result<Json<LegacyPanchangaResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Convert legacy request to new EngineInput format
    let input = EngineInput {
        birth_data: Some(noesis_core::BirthData {
            name: request.name,
            date: request.date,
            time: request.time,
            latitude: request.latitude,
            longitude: request.longitude,
            timezone: request.timezone,
        }),
        current_time: chrono::Utc::now(),
        location: Some(noesis_core::Coordinates {
            latitude: request.latitude,
            longitude: request.longitude,
            altitude: None,
        }),
        precision: noesis_core::Precision::Standard,
        options: std::collections::HashMap::new(),
    };

    // Execute Panchanga engine through orchestrator
    let output = state
        .orchestrator
        .execute_engine("panchanga", input, 0)
        .await
        .map_err(ErrorMapper::map)?;

    // Extract PanchangaResult from engine output
    let panchanga_result: serde_json::Value = output.result;

    // Convert to legacy response format
    let legacy_response = LegacyPanchangaResponse {
        tithi_index: panchanga_result["tithi_index"].as_u64().unwrap_or(0) as u8,
        tithi_name: panchanga_result["tithi_name"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        tithi_value: panchanga_result["tithi_value"].as_f64().unwrap_or(0.0),
        nakshatra_index: panchanga_result["nakshatra_index"].as_u64().unwrap_or(0) as u8,
        nakshatra_name: panchanga_result["nakshatra_name"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        nakshatra_value: panchanga_result["nakshatra_value"].as_f64().unwrap_or(0.0),
        yoga_index: panchanga_result["yoga_index"].as_u64().unwrap_or(0) as u8,
        yoga_name: panchanga_result["yoga_name"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        yoga_value: panchanga_result["yoga_value"].as_f64().unwrap_or(0.0),
        karana_index: panchanga_result["karana_index"].as_u64().unwrap_or(0) as u8,
        karana_name: panchanga_result["karana_name"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        karana_value: panchanga_result["karana_value"].as_f64().unwrap_or(0.0),
        vara_index: panchanga_result["vara_index"].as_u64().unwrap_or(0) as u8,
        vara_name: panchanga_result["vara_name"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        solar_longitude: panchanga_result["solar_longitude"].as_f64().unwrap_or(0.0),
        lunar_longitude: panchanga_result["lunar_longitude"].as_f64().unwrap_or(0.0),
        julian_day: panchanga_result["julian_day"].as_f64().unwrap_or(0.0),
    };

    Ok(Json(legacy_response))
}

/// Legacy request format for Ghati time queries
#[derive(Deserialize)]
struct LegacyGhatiRequest {
    #[serde(default)]
    latitude: Option<f64>,
    #[serde(default)]
    longitude: Option<f64>,
}

/// Legacy response format for Ghati time
#[derive(Serialize)]
struct LegacyGhatiResponse {
    ghati: u8,
    pala: u8,
    vipala: u8,
    utc_timestamp: String,
}

/// GET /api/legacy/ghati/current -- backward compatible current Ghati time endpoint
async fn legacy_ghati_current_handler(
    State(state): State<AppState>,
    Json(request): Json<LegacyGhatiRequest>,
) -> Result<Json<LegacyGhatiResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Default to Bangalore coordinates if not provided
    let latitude = request.latitude.unwrap_or(12.9716);
    let longitude = request.longitude.unwrap_or(77.5946);

    // Build input for Panchanga calculation (current time)
    let now = chrono::Utc::now();
    let input = EngineInput {
        birth_data: Some(noesis_core::BirthData {
            name: None,
            date: now.format("%Y-%m-%d").to_string(),
            time: Some(now.format("%H:%M").to_string()),
            latitude,
            longitude,
            timezone: "UTC".to_string(),
        }),
        current_time: now,
        location: Some(noesis_core::Coordinates {
            latitude,
            longitude,
            altitude: None,
        }),
        precision: noesis_core::Precision::Standard,
        options: std::collections::HashMap::new(),
    };

    // Execute Panchanga engine to get current time data
    let output = state
        .orchestrator
        .execute_engine("panchanga", input, 0)
        .await
        .map_err(ErrorMapper::map)?;

    // Extract tithi value to calculate ghati time
    // In Vedic time, 1 day = 60 ghatis, 1 ghati = 24 minutes
    let panchanga_result: serde_json::Value = output.result;
    let _tithi_value = panchanga_result["tithi_value"].as_f64().unwrap_or(0.0);

    // Calculate ghati from time of day (simplified - using hour of day)
    let hour_of_day = now.hour() as f64 + (now.minute() as f64 / 60.0);
    let ghati_value = (hour_of_day / 24.0) * 60.0;

    let ghati = ghati_value.floor() as u8;
    let pala = ((ghati_value.fract() * 60.0).floor()) as u8;
    let vipala = ((ghati_value.fract() * 60.0).fract() * 60.0).floor() as u8;

    Ok(Json(LegacyGhatiResponse {
        ghati,
        pala,
        vipala,
        utc_timestamp: now.to_rfc3339(),
    }))
}

// ---------------------------------------------------------------------------
// Application state builder
// ---------------------------------------------------------------------------

async fn build_runtime_orchestrator_and_bridge(
) -> (WorkflowOrchestrator, Arc<noesis_bridge::BridgeManager>) {
    let mut orchestrator = WorkflowOrchestrator::new();
    orchestrator.register_native_runtime_engines();

    let bridge_manager = Arc::new(noesis_bridge::BridgeManager::from_env());

    // Always register TS engines so they appear in the API regardless of
    // connectivity during startup. They will return a BridgeError if called
    // while the TS server is down.
    for engine in bridge_manager.engines() {
        orchestrator.register_engine(engine);
    }

    let mut ts_connected = false;

    // Optional connectivity check for logging status
    for attempt in 1..=3 {
        if bridge_manager.is_available().await {
            tracing::info!("TS engines verified at {}", bridge_manager.base_url());
            ts_connected = true;
            break;
        }

        if attempt < 3 {
            tracing::info!(
                "Checking TS engine connectivity at {} (attempt {}/3)...",
                bridge_manager.base_url(),
                attempt
            );
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    if !ts_connected {
        tracing::warn!(
            "TS engines registered but unreachable at {} - they will appear in API but return errors if used",
            bridge_manager.base_url()
        );
    }

    log_workflow_registry_parity(&orchestrator);

    (orchestrator, bridge_manager)
}

/// Build the default `AppState` with all engines registered.
///
/// # Arguments
/// * `config` - API configuration with JWT secret, Redis URL, cache settings, etc.
///
/// # Returns
/// Configured `AppState` with orchestrator, cache, auth, and metrics
pub async fn build_app_state(config: &ApiConfig) -> AppState {
    // -- Orchestrator with engines --
    let (orchestrator, bridge_manager) = build_runtime_orchestrator_and_bridge().await;

    // -- Cache --
    let redis_url = config.redis_url.clone().unwrap_or_default();
    let cache = CacheManager::new(
        redis_url,                 // Redis URL from config
        100,                       // L1: 100 MB
        Duration::from_secs(3600), // L2 TTL: 1 hour
        false,                     // L3 disabled
    );

    // -- Database (optional — server runs in degraded mode without it) --
    //
    // Supabase uses PgBouncer in transaction mode, which conflicts with SQLx
    // prepared statement caching. Disable the cache to avoid:
    //   "prepared statement 'sqlx_s_N' already exists"
    let pool = if let Some(ref db_url) = config.database_url {
        let connect_options: PgConnectOptions = db_url
            .parse::<PgConnectOptions>()
            .map(|opts| opts.statement_cache_capacity(0))
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to parse DATABASE_URL ({}), using as-is", e);
                PgConnectOptions::new()
            });

        // Retry startup DB connection to handle transient pooler unavailability.
        // Uses 3 attempts with exponential back-off (2s -> 4s) before giving up.
        const MAX_ATTEMPTS: u32 = 3;
        let mut pool_result: Option<sqlx::PgPool> = None;
        for attempt in 1..=MAX_ATTEMPTS {
            let opts = connect_options.clone();
            match tokio::time::timeout(
                Duration::from_secs(10),
                PgPoolOptions::new()
                    .max_connections(5)
                    .idle_timeout(Duration::from_secs(300)) // drop idle conns after 5min
                    .acquire_timeout(Duration::from_secs(5)) // fail fast on acquire
                    .test_before_acquire(true) // health-check stale conns
                    .connect_with(opts),
            )
            .await
            {
                Ok(Ok(p)) => {
                    tracing::info!(
                        "Database pool connected (attempt {}/{})",
                        attempt,
                        MAX_ATTEMPTS
                    );
                    pool_result = Some(p);
                    break;
                }
                Ok(Err(ref e)) if attempt < MAX_ATTEMPTS => {
                    let delay = Duration::from_secs(2u64.pow(attempt));
                    tracing::warn!(
                        "Database connection attempt {}/{} failed: {} - retrying in {}s",
                        attempt,
                        MAX_ATTEMPTS,
                        e,
                        delay.as_secs()
                    );
                    tokio::time::sleep(delay).await;
                }
                Ok(Err(e)) => {
                    tracing::warn!("Database connection failed (running without DB): {}", e);
                }
                Err(_) if attempt < MAX_ATTEMPTS => {
                    let delay = Duration::from_secs(2u64.pow(attempt));
                    tracing::warn!(
                        "Database connection attempt {}/{} timed out - retrying in {}s",
                        attempt,
                        MAX_ATTEMPTS,
                        delay.as_secs()
                    );
                    tokio::time::sleep(delay).await;
                }
                Err(_) => {
                    tracing::warn!("Database connection timed out after 10s (running without DB)");
                }
            }
        }
        pool_result
    } else {
        tracing::warn!("No DATABASE_URL configured — auth endpoints unavailable");
        None
    };

    // -- Auth (Postgres-backed API key validation, or degraded without DB) --
    let auth = AuthService::with_pool(config.jwt_secret.clone(), pool.clone());

    // -- Persistence repos (only available when DB is connected) --
    let db_available = pool.is_some();
    let admin_repository = pool
        .as_ref()
        .map(|p| Arc::new(AdminRepository::new(p.clone())));
    let biofield_repository = pool
        .as_ref()
        .map(|p| Arc::new(BiofieldRepository::new(p.clone())));
    let readings_repository = pool
        .as_ref()
        .map(|p| Arc::new(ReadingsRepository::new(p.clone())));
    let usage_repository = pool
        .as_ref()
        .map(|p| Arc::new(UsageRepository::new(p.clone())));
    let oauth_repository = pool.as_ref().map(|p| {
        Arc::new(noesis_data::repositories::oauth_repository::OAuthRepository::new(p.clone()))
    });

    let user_repository = Arc::new(UserRepository::new(pool.unwrap_or_else(|| {
        // Create a lazy pool with a dummy URL — queries will fail at runtime,
        // but the server can still boot and serve non-DB endpoints.
        PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://localhost/noesis_unavailable")
            .expect("Failed to create placeholder pool")
    })));

    // -- Metrics --
    let metrics = shared_metrics();

    AppState {
        orchestrator: Arc::new(orchestrator),
        bridge_manager,
        cache: Arc::new(cache),
        auth: Arc::new(auth),
        metrics,
        user_repository,
        admin_repository,
        biofield_repository,
        readings_repository,
        usage_repository,
        oauth_repository,
        startup_time: Instant::now(),
        db_available,
        discord_client_id: config.discord_client_id.clone(),
        discord_client_secret: config.discord_client_secret.clone(),
        discord_redirect_uri: config.discord_redirect_uri.clone(),
    }
}

/// Build `AppState` but create the PostgreSQL pool lazily (no network connection during init).
///
/// This is primarily intended for integration/E2E tests that don't exercise DB-backed
/// endpoints but still need a fully constructed `AppState`.
pub async fn build_app_state_lazy_db(config: &ApiConfig) -> AppState {
    // -- Orchestrator with engines --
    let (orchestrator, bridge_manager) = build_runtime_orchestrator_and_bridge().await;

    // -- Cache --
    let redis_url = config.redis_url.clone().unwrap_or_default();
    let cache = CacheManager::new(
        redis_url,                 // Redis URL from config
        100,                       // L1: 100 MB
        Duration::from_secs(3600), // L2 TTL: 1 hour
        false,                     // L3 disabled
    );

    // -- Database (lazy pool, optional) --
    let pool = config.database_url.as_ref().map(|db_url| {
        PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(db_url)
            .expect("Failed to create lazy database pool")
    });

    // -- Auth (lazy Postgres-backed API key validation, or degraded without DB) --
    let auth = AuthService::with_pool(config.jwt_secret.clone(), pool.clone());

    // -- Persistence repos (only available when DB is configured) --
    let db_available = pool.is_some();
    let admin_repository = pool
        .as_ref()
        .map(|p| Arc::new(AdminRepository::new(p.clone())));
    let biofield_repository = pool
        .as_ref()
        .map(|p| Arc::new(BiofieldRepository::new(p.clone())));
    let readings_repository = pool
        .as_ref()
        .map(|p| Arc::new(ReadingsRepository::new(p.clone())));
    let usage_repository = pool
        .as_ref()
        .map(|p| Arc::new(UsageRepository::new(p.clone())));
    let oauth_repository = pool.as_ref().map(|p| {
        Arc::new(noesis_data::repositories::oauth_repository::OAuthRepository::new(p.clone()))
    });

    let user_repository = Arc::new(UserRepository::new(pool.unwrap_or_else(|| {
        PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://localhost/noesis_unavailable")
            .expect("Failed to create placeholder pool")
    })));

    // -- Metrics --
    let metrics = shared_metrics();

    AppState {
        orchestrator: Arc::new(orchestrator),
        bridge_manager,
        cache: Arc::new(cache),
        auth: Arc::new(auth),
        metrics,
        user_repository,
        admin_repository,
        biofield_repository,
        readings_repository,
        usage_repository,
        oauth_repository,
        startup_time: Instant::now(),
        db_available,
        discord_client_id: config.discord_client_id.clone(),
        discord_client_secret: config.discord_client_secret.clone(),
        discord_redirect_uri: config.discord_redirect_uri.clone(),
    }
}
