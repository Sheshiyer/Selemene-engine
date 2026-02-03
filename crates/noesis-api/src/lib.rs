//! Noesis API -- Axum HTTP server for the Tryambakam Noesis platform
//!
//! Wires up the orchestrator, cache, auth, and metrics into a unified REST API.
//! All engine calculations and workflow executions are exposed through versioned
//! JSON endpoints under `/api/v1/`.

mod config;
mod logging;
mod middleware;
mod handlers;

// Re-export configuration and logging for main.rs
pub use config::ApiConfig;
pub use logging::{init_tracing, init_tracing_json};

use axum::{
    extract::{Json, Path, State},
    http::{HeaderValue, Method, StatusCode},
    middleware as axum_middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension,
    Router,
};
use chrono::Timelike;
use noesis_auth::{AuthService, AuthUser};
use noesis_cache::CacheManager;
use noesis_data::repositories::user_repository::UserRepository;
use noesis_core::{EngineError, EngineInput, EngineOutput, ValidationResult, WorkflowResult};
use noesis_metrics::NoesisMetrics;
use noesis_orchestrator::WorkflowOrchestrator;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use sqlx::postgres::PgPoolOptions;

// ---------------------------------------------------------------------------
// OpenAPI documentation
// ---------------------------------------------------------------------------

#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        readiness_handler,
        status_handler,
        list_engines_handler,
        calculate_handler,
        validate_handler,
        engine_info_handler,
        list_workflows_handler,
        workflow_execute_handler,
        workflow_info_handler,
    ),
    components(
        schemas(
            EngineInput,
            EngineOutput,
            ValidationResult,
            WorkflowResult,
            HealthResponse,
            ReadinessResponse,
            StatusResponse,
            WorkflowSummary,
            EngineInfoResponse,
            EngineListResponse,
            WorkflowListResponse,
            WorkflowInfoResponse,
            ErrorResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check and monitoring endpoints"),
        (name = "engines", description = "Single engine calculation endpoints"),
        (name = "workflows", description = "Multi-engine workflow execution endpoints"),
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Noesis API",
        version = "0.1.0",
        description = "HTTP API for the Tryambakam Noesis consciousness engine platform. Provides endpoints for astrological calculations (Panchanga), numerology, biorhythms, and multi-engine workflows.",
        contact(
            name = "Tryambakam Team",
        )
    ),
)]
struct ApiDoc;

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme, ApiKey, ApiKeyValue};
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
                        .build()
                ),
            );
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(
                    ApiKey::Header(
                        ApiKeyValue::new("X-API-Key")
                    )
                ),
            );
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
    pub cache: Arc<CacheManager>,
    pub auth: Arc<AuthService>,
    pub metrics: Arc<NoesisMetrics>,
    pub user_repository: Arc<UserRepository>,
    pub startup_time: Instant,
}

// ---------------------------------------------------------------------------
// CORS configuration
// ---------------------------------------------------------------------------

/// Create production-ready CORS layer with environment-based origin allowlist.
///
/// # Arguments
/// * `allowed_origins` - List of allowed origins (e.g., ["http://localhost:3000"])
///
/// Configuration:
/// - Methods: GET, POST, OPTIONS
/// - Headers: Content-Type, Authorization, X-API-Key
/// - Credentials: true (for cookie/auth workflows)
/// - Max Age: 3600 seconds (1 hour)
fn create_cors_layer(allowed_origins: Vec<String>) -> CorsLayer {
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse().ok()
            }
        })
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderName::from_static("x-api-key"),
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
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
    ));
    
    let auth_routes = Router::new()
         .route("/auth/register", post(handlers::auth::register));

    let api_v1 = Router::new()
        .route("/status", get(status_handler))
        .route("/engines", get(list_engines_handler))
        .route("/engines/:engine_id/calculate", post(calculate_handler))
        .route("/engines/:engine_id/validate", post(validate_handler))
        .route("/engines/:engine_id/info", get(engine_info_handler))
        .route("/workflows", get(list_workflows_handler))
        .route(
            "/workflows/:workflow_id/execute",
            post(workflow_execute_handler),
        )
        .route("/workflows/:workflow_id/info", get(workflow_info_handler))
        // Layers are applied bottom-to-top, so rate_limit runs AFTER auth
        .layer(axum_middleware::from_fn_with_state(
            rate_limiter,
            middleware::rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            auth_state,
            middleware::auth_middleware,
        ))
        .merge(auth_routes);

    // Legacy endpoints for backward compatibility with old Selemene API
    let legacy = Router::new()
        .route("/panchanga/calculate", post(legacy_panchanga_handler))
        .route("/ghati/current", get(legacy_ghati_current_handler));

    // Start with a base router and merge docs first (both have () state)
    let base = Router::new().merge(
        SwaggerUi::new("/api/docs")
            .url("/api/openapi.json", ApiDoc::openapi())
    );

    // Now add stateful routes
    base
        .route("/health", get(health_handler))
        .route("/health/live", get(health_handler))  // Kubernetes liveness probe
        .route("/health/ready", get(readiness_handler))  // Kubernetes readiness probe
        .route("/ready", get(readiness_handler))
        .route("/metrics", get(metrics_handler))
        .nest("/api/v1", api_v1)
        .nest("/api/legacy", legacy)
        .layer(axum_middleware::from_fn(middleware::request_logging_middleware))
        .layer(TraceLayer::new_for_http())
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
    orchestrator: String,
    overall_status: String,
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

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
    error_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
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
        version: "0.1.0".to_string(),
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

    // Check orchestrator readiness
    let orchestrator_status = match state.orchestrator.is_ready().await {
        Ok(true) => "ready",
        _ => "not_ready",
    };

    let overall_ready = redis_status == "ok" && orchestrator_status == "ready";
    let overall_status = if overall_ready { "ready" } else { "not_ready" };

    let response = ReadinessResponse {
        redis: redis_status.to_string(),
        orchestrator: orchestrator_status.to_string(),
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
    match state.metrics.get_metrics_text() {
        Ok(text) => (StatusCode::OK, text).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode metrics: {}", e),
        )
            .into_response(),
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
        (status = 200, description = "Calculation successful", body = EngineOutput),
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
) -> Result<Json<EngineOutput>, (StatusCode, Json<ErrorResponse>)> {
    let start = Instant::now();
    
    // Execute engine with user's consciousness level
    let result = state
        .orchestrator
        .execute_engine(&engine_id, input, user.consciousness_level)
        .await;
    
    let duration_secs = start.elapsed().as_secs_f64();
    
    match result {
        Ok(output) => {
            state.metrics.record_engine_calculation_with_status(&engine_id, "success", duration_secs);
            Ok(Json(output))
        }
        Err(e) => {
            state.metrics.record_engine_calculation_with_status(&engine_id, "failure", duration_secs);
            
            let error_type = match &e {
                EngineError::EngineNotFound(_) => "not_found",
                EngineError::PhaseAccessDenied { .. } => "forbidden",
                EngineError::AuthError(_) => "unauthorized",
                EngineError::RateLimitExceeded => "rate_limit",
                EngineError::ValidationError(_) => "validation_error",
                _ => "internal_error",
            };
            
            state.metrics.record_engine_calculation_error(&engine_id, error_type);
            Err(engine_error_to_response(e))
        }
    }
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
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Engine '{}' not found", engine_id),
                    error_code: "ENGINE_NOT_FOUND".to_string(),
                    details: Some(serde_json::json!({ "engine_id": engine_id })),
                }),
            )
        })?;

    engine
        .validate(&output)
        .await
        .map(Json)
        .map_err(|e| engine_error_to_response(e))
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
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Engine '{}' not found", engine_id),
                    error_code: "ENGINE_NOT_FOUND".to_string(),
                    details: Some(serde_json::json!({ "engine_id": engine_id })),
                }),
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
        (status = 200, description = "Workflow execution successful", body = WorkflowResult),
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
) -> Result<Json<noesis_core::WorkflowResult>, (StatusCode, Json<ErrorResponse>)> {
    let start = Instant::now();
    
    // Execute workflow with user's consciousness level
    let result = state
        .orchestrator
        .execute_workflow(&workflow_id, input, user.consciousness_level)
        .await;
    
    let duration_secs = start.elapsed().as_secs_f64();
    
    // Use workflow_id prefixed to distinguish from engine calculations
    let workflow_label = format!("workflow:{}", workflow_id);
    
    match result {
        Ok(workflow_result) => {
            state.metrics.record_engine_calculation_with_status(&workflow_label, "success", duration_secs);
            Ok(Json(workflow_result))
        }
        Err(e) => {
            state.metrics.record_engine_calculation_with_status(&workflow_label, "failure", duration_secs);
            
            let error_type = match &e {
                EngineError::WorkflowNotFound(_) => "not_found",
                EngineError::PhaseAccessDenied { .. } => "forbidden",
                EngineError::AuthError(_) => "unauthorized",
                EngineError::RateLimitExceeded => "rate_limit",
                EngineError::ValidationError(_) => "validation_error",
                _ => "internal_error",
            };
            
            state.metrics.record_engine_calculation_error(&workflow_label, error_type);
            Err(engine_error_to_response(e))
        }
    }
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
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Workflow '{}' not found", workflow_id),
                    error_code: "WORKFLOW_NOT_FOUND".to_string(),
                    details: Some(serde_json::json!({ "workflow_id": workflow_id })),
                }),
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
// Error mapping
// ---------------------------------------------------------------------------

fn engine_error_to_response(err: EngineError) -> (StatusCode, Json<ErrorResponse>) {
    let (status, error_code, message, details) = match &err {
        EngineError::EngineNotFound(id) => (
            StatusCode::NOT_FOUND,
            "ENGINE_NOT_FOUND".to_string(),
            err.to_string(),
            Some(serde_json::json!({ "engine_id": id })),
        ),
        EngineError::WorkflowNotFound(id) => (
            StatusCode::NOT_FOUND,
            "WORKFLOW_NOT_FOUND".to_string(),
            err.to_string(),
            Some(serde_json::json!({ "workflow_id": id })),
        ),
        EngineError::PhaseAccessDenied { required, current } => (
            StatusCode::FORBIDDEN,
            "PHASE_ACCESS_DENIED".to_string(),
            err.to_string(),
            Some(serde_json::json!({
                "required_phase": required,
                "current_phase": current
            })),
        ),
        EngineError::AuthError(msg) => (
            StatusCode::UNAUTHORIZED,
            "AUTH_ERROR".to_string(),
            err.to_string(),
            Some(serde_json::json!({ "reason": msg })),
        ),
        EngineError::RateLimitExceeded => (
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMIT_EXCEEDED".to_string(),
            err.to_string(),
            None,
        ),
        EngineError::ValidationError(msg) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "VALIDATION_ERROR".to_string(),
            err.to_string(),
            Some(serde_json::json!({ "validation_message": msg })),
        ),
        EngineError::CalculationError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "CALCULATION_ERROR".to_string(),
            err.to_string(),
            Some(serde_json::json!({ "calculation_message": msg })),
        ),
        EngineError::CacheError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "CACHE_ERROR".to_string(),
            err.to_string(),
            Some(serde_json::json!({ "cache_message": msg })),
        ),
        EngineError::ConfigError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONFIG_ERROR".to_string(),
            err.to_string(),
            Some(serde_json::json!({ "config_message": msg })),
        ),
        EngineError::BridgeError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "BRIDGE_ERROR".to_string(),
            err.to_string(),
            Some(serde_json::json!({ "bridge_message": msg })),
        ),
        EngineError::SwissEphemerisError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "SWISS_EPHEMERIS_ERROR".to_string(),
            err.to_string(),
            Some(serde_json::json!({ "ephemeris_message": msg })),
        ),
        EngineError::InternalError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR".to_string(),
            err.to_string(),
            Some(serde_json::json!({ "internal_message": msg })),
        ),
    };

    (
        status,
        Json(ErrorResponse {
            error: message,
            error_code,
            details,
        }),
    )
}

// ---------------------------------------------------------------------------
// Legacy API handlers (backward compatibility with old Selemene API)
// ---------------------------------------------------------------------------

/// Legacy request format for Panchanga calculations from old Selemene API
#[derive(Deserialize)]
struct LegacyPanchangaRequest {
    date: String,        // YYYY-MM-DD
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
        .map_err(engine_error_to_response)?;

    // Extract PanchangaResult from engine output
    let panchanga_result: serde_json::Value = output.result;
    
    // Convert to legacy response format
    let legacy_response = LegacyPanchangaResponse {
        tithi_index: panchanga_result["tithi_index"].as_u64().unwrap_or(0) as u8,
        tithi_name: panchanga_result["tithi_name"].as_str().unwrap_or("").to_string(),
        tithi_value: panchanga_result["tithi_value"].as_f64().unwrap_or(0.0),
        nakshatra_index: panchanga_result["nakshatra_index"].as_u64().unwrap_or(0) as u8,
        nakshatra_name: panchanga_result["nakshatra_name"].as_str().unwrap_or("").to_string(),
        nakshatra_value: panchanga_result["nakshatra_value"].as_f64().unwrap_or(0.0),
        yoga_index: panchanga_result["yoga_index"].as_u64().unwrap_or(0) as u8,
        yoga_name: panchanga_result["yoga_name"].as_str().unwrap_or("").to_string(),
        yoga_value: panchanga_result["yoga_value"].as_f64().unwrap_or(0.0),
        karana_index: panchanga_result["karana_index"].as_u64().unwrap_or(0) as u8,
        karana_name: panchanga_result["karana_name"].as_str().unwrap_or("").to_string(),
        karana_value: panchanga_result["karana_value"].as_f64().unwrap_or(0.0),
        vara_index: panchanga_result["vara_index"].as_u64().unwrap_or(0) as u8,
        vara_name: panchanga_result["vara_name"].as_str().unwrap_or("").to_string(),
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
        .map_err(engine_error_to_response)?;

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

/// Build the default `AppState` with all engines registered.
///
/// # Arguments
/// * `config` - API configuration with JWT secret, Redis URL, cache settings, etc.
///
/// # Returns
/// Configured `AppState` with orchestrator, cache, auth, and metrics
pub async fn build_app_state(config: &ApiConfig) -> AppState {
    // -- Orchestrator with engines --
    let mut orchestrator = WorkflowOrchestrator::new();
    orchestrator.register_engine(Arc::new(engine_panchanga::PanchangaEngine::new()));
    orchestrator.register_engine(Arc::new(engine_numerology::NumerologyEngine::new()));
    orchestrator.register_engine(Arc::new(engine_biorhythm::BiorhythmEngine::new()));
    
    // Register HD engine (Phase 1)
    let hd_engine = Arc::new(engine_human_design::HumanDesignEngine::new());
    orchestrator.register_engine(hd_engine.clone());
    
    // Register Gene Keys engine with HD dependency (Phase 2)
    let gk_engine = Arc::new(engine_gene_keys::GeneKeysEngine::with_hd_engine(hd_engine.clone()));
    orchestrator.register_engine(gk_engine);

    // Register Vimshottari Dasha engine with HD dependency (Phase 2)
    let vim_engine = Arc::new(engine_vimshottari::VimshottariEngine::with_hd_engine(hd_engine));
    orchestrator.register_engine(vim_engine);

    // Register Biofield engine (Phase 1 - somatic awareness) - returns mock data
    orchestrator.register_engine(Arc::new(engine_biofield::BiofieldEngine::new()));

    // Register VedicClock-TCM engine (Phase 0 - available to all)
    orchestrator.register_engine(Arc::new(engine_vedic_clock::VedicClockEngine::new()));

    // -- Cache --
    let redis_url = config.redis_url.clone().unwrap_or_else(|| String::new());
    let cache = CacheManager::new(
        redis_url,               // Redis URL from config
        100,                     // L1: 100 MB
        Duration::from_secs(3600), // L2 TTL: 1 hour
        false,                   // L3 disabled
    );

    // -- Auth --
    let auth = AuthService::new(config.jwt_secret.clone());

    // -- Database --
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database pool");

    let user_repository = Arc::new(UserRepository::new(pool));

    // -- Metrics --
    let metrics = NoesisMetrics::new().expect("Failed to initialise NoesisMetrics");

    AppState {
        orchestrator: Arc::new(orchestrator),
        cache: Arc::new(cache),
        auth: Arc::new(auth),
        metrics: Arc::new(metrics),
        user_repository,
        startup_time: Instant::now(),
    }
}
