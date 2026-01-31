//! Noesis API -- Axum HTTP server for the Tryambakam Noesis platform
//!
//! Wires up the orchestrator, cache, auth, and metrics into a unified REST API.
//! All engine calculations and workflow executions are exposed through versioned
//! JSON endpoints under `/api/v1/`.

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use noesis_auth::AuthService;
use noesis_cache::CacheManager;
use noesis_core::{EngineError, EngineInput, EngineOutput, WorkflowDefinition};
use noesis_metrics::NoesisMetrics;
use noesis_orchestrator::WorkflowOrchestrator;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

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
}

// ---------------------------------------------------------------------------
// Router construction
// ---------------------------------------------------------------------------

/// Build the Axum router with all API routes and middleware.
pub fn create_router(state: AppState) -> Router {
    let api_v1 = Router::new()
        .route("/status", get(status_handler))
        .route("/engines", get(list_engines_handler))
        .route("/engines/{engine_id}/calculate", post(calculate_handler))
        .route("/engines/{engine_id}/validate", post(validate_handler))
        .route("/engines/{engine_id}/info", get(engine_info_handler))
        .route("/workflows", get(list_workflows_handler))
        .route(
            "/workflows/{workflow_id}/execute",
            post(workflow_execute_handler),
        )
        .route("/workflows/{workflow_id}/info", get(workflow_info_handler));

    Router::new()
        .route("/health", get(health_handler))
        .nest("/api/v1", api_v1)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct StatusResponse {
    engines: Vec<String>,
    workflows: Vec<WorkflowSummary>,
}

#[derive(Serialize)]
struct WorkflowSummary {
    id: String,
    name: String,
    description: String,
    engine_count: usize,
}

#[derive(Serialize)]
struct EngineInfoResponse {
    engine_id: String,
    engine_name: String,
    required_phase: u8,
}

#[derive(Serialize)]
struct EngineListResponse {
    engines: Vec<String>,
}

#[derive(Serialize)]
struct WorkflowListResponse {
    workflows: Vec<WorkflowSummary>,
}

#[derive(Serialize)]
struct WorkflowInfoResponse {
    id: String,
    name: String,
    description: String,
    engine_ids: Vec<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /health -- liveness probe
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
    })
}

/// GET /api/v1/status -- list registered engines and workflows
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
async fn calculate_handler(
    State(state): State<AppState>,
    Path(engine_id): Path<String>,
    Json(input): Json<EngineInput>,
) -> Result<Json<EngineOutput>, (StatusCode, Json<ErrorResponse>)> {
    state
        .orchestrator
        .execute_engine(&engine_id, input, 0)
        .await
        .map(Json)
        .map_err(|e| engine_error_to_response(e))
}

/// POST /api/v1/engines/:engine_id/validate -- validate an engine output
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
async fn list_engines_handler(State(state): State<AppState>) -> Json<EngineListResponse> {
    Json(EngineListResponse {
        engines: state.orchestrator.list_engines(),
    })
}

/// POST /api/v1/workflows/:workflow_id/execute -- execute a workflow
async fn workflow_execute_handler(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(input): Json<EngineInput>,
) -> Result<Json<noesis_core::WorkflowResult>, (StatusCode, Json<ErrorResponse>)> {
    state
        .orchestrator
        .execute_workflow(&workflow_id, input, 0)
        .await
        .map(Json)
        .map_err(|e| engine_error_to_response(e))
}

/// GET /api/v1/workflows -- list all workflow IDs
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
    let (status, message) = match &err {
        EngineError::EngineNotFound(_) | EngineError::WorkflowNotFound(_) => {
            (StatusCode::NOT_FOUND, err.to_string())
        }
        EngineError::PhaseAccessDenied { .. } => (StatusCode::FORBIDDEN, err.to_string()),
        EngineError::AuthError(_) => (StatusCode::UNAUTHORIZED, err.to_string()),
        EngineError::RateLimitExceeded => {
            (StatusCode::TOO_MANY_REQUESTS, err.to_string())
        }
        EngineError::ValidationError(_) => {
            (StatusCode::UNPROCESSABLE_ENTITY, err.to_string())
        }
        _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    };

    (status, Json(ErrorResponse { error: message }))
}

// ---------------------------------------------------------------------------
// Application state builder
// ---------------------------------------------------------------------------

/// Build the default `AppState` with all engines registered.
///
/// Reads `JWT_SECRET` from the environment (falls back to a dev-only default).
pub fn build_app_state() -> AppState {
    // -- Orchestrator with engines --
    let mut orchestrator = WorkflowOrchestrator::new();
    orchestrator.register_engine(Arc::new(engine_panchanga::PanchangaEngine::new()));
    orchestrator.register_engine(Arc::new(engine_numerology::NumerologyEngine::new()));
    orchestrator.register_engine(Arc::new(engine_biorhythm::BiorhythmEngine::new()));

    // -- Cache --
    let cache = CacheManager::new(
        String::new(),        // no Redis URL in dev
        100,                  // L1: 100 MB
        Duration::from_secs(3600), // L2 TTL: 1 hour
        false,                // L3 disabled
    );

    // -- Auth --
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    let auth = AuthService::new(jwt_secret);

    // -- Metrics --
    let metrics = NoesisMetrics::new().expect("Failed to initialise NoesisMetrics");

    AppState {
        orchestrator: Arc::new(orchestrator),
        cache: Arc::new(cache),
        auth: Arc::new(auth),
        metrics: Arc::new(metrics),
    }
}
