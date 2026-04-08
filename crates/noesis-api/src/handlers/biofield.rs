use crate::{AppState, ErrorMapper};
use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use noesis_auth::AuthUser;
use noesis_data::models::biofield::{
    BiofieldSession, NewBiofieldSession, BIOFIELD_SESSION_STATUS_ACTIVE,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBiofieldSessionRequest {
    pub client_device_id: Option<String>,
    pub viewer_version: Option<String>,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CloseBiofieldSessionRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct ListBiofieldReadingsQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BiofieldSessionResource {
    pub id: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub client_device_id: Option<String>,
    pub viewer_version: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BiofieldQualitySummary {
    pub sufficient_quality: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BiofieldArtifactSummary {
    pub kind: String,
    pub mime_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BiofieldReadingSummary {
    pub reading_id: String,
    pub session_id: String,
    pub engine_id: String,
    pub created_at: DateTime<Utc>,
    pub quality: BiofieldQualitySummary,
    pub artifact: BiofieldArtifactSummary,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListBiofieldReadingsResponse {
    pub items: Vec<BiofieldReadingSummary>,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BiofieldReadingDetail {
    pub reading_id: String,
    pub session_id: String,
    pub engine_id: String,
    pub created_at: DateTime<Utc>,
    pub input: serde_json::Value,
    pub result: serde_json::Value,
    pub quality: serde_json::Value,
    pub artifacts: Vec<BiofieldArtifactSummary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BiofieldPlaceholderResponse {
    pub status: String,
    pub message: String,
}

fn not_implemented_response(route: &str) -> (StatusCode, Json<BiofieldPlaceholderResponse>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(BiofieldPlaceholderResponse {
            status: "not_implemented".to_string(),
            message: format!("{route} is scaffolded but not implemented yet"),
        }),
    )
}

fn json_error_response(
    status: StatusCode,
    message: impl Into<String>,
    error_code: &str,
    details: Option<serde_json::Value>,
) -> Response {
    ErrorMapper::response(status, error_code, message.into(), details).into_response()
}

fn biofield_db_unavailable_response() -> Response {
    json_error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Biofield APIs require a configured database connection",
        "BIOFIELD_DB_UNAVAILABLE",
        None,
    )
}

fn invalid_uuid_response(field: &str, value: &str) -> Response {
    json_error_response(
        StatusCode::UNPROCESSABLE_ENTITY,
        format!("Invalid UUID for {field}"),
        "INVALID_INPUT",
        Some(serde_json::json!({ field: value })),
    )
}

fn session_not_found_response(session_id: &str) -> Response {
    json_error_response(
        StatusCode::NOT_FOUND,
        "Biofield session not found",
        "BIOFIELD_SESSION_NOT_FOUND",
        Some(serde_json::json!({ "session_id": session_id })),
    )
}

fn session_not_active_response(session: &BiofieldSession) -> Response {
    json_error_response(
        StatusCode::CONFLICT,
        "Biofield session is not active",
        "BIOFIELD_SESSION_NOT_ACTIVE",
        Some(serde_json::json!({
            "session_id": session.id,
            "status": session.status,
        })),
    )
}

fn biofield_db_error_response(action: &str, error: &sqlx::Error) -> Response {
    tracing::error!("biofield database error during {action}: {error}");
    json_error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Failed to {action}"),
        "BIOFIELD_DB_ERROR",
        Some(serde_json::json!({ "action": action })),
    )
}

fn parse_auth_user_uuid(auth_user: &AuthUser) -> Result<Uuid, Response> {
    Uuid::parse_str(&auth_user.user_id).map_err(|_| {
        json_error_response(
            StatusCode::UNAUTHORIZED,
            "Invalid user ID in token",
            "AUTH_ERROR",
            None,
        )
    })
}

fn parse_uuid_or_422(value: &str, field: &str) -> Result<Uuid, Response> {
    Uuid::parse_str(value).map_err(|_| invalid_uuid_response(field, value))
}

fn session_to_resource(session: &BiofieldSession) -> BiofieldSessionResource {
    BiofieldSessionResource {
        id: session.id.to_string(),
        status: session.status.clone(),
        started_at: session.started_at,
        closed_at: session.closed_at,
        client_device_id: session.client_device_id.clone(),
        viewer_version: session.viewer_version.clone(),
    }
}

/// POST /api/v1/biofield/sessions
#[utoipa::path(
    post,
    path = "/api/v1/biofield/sessions",
    tag = "biofield",
    request_body = CreateBiofieldSessionRequest,
    responses(
        (status = 201, description = "Biofield session created", body = BiofieldSessionResource),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 503, description = "Biofield DB unavailable", body = crate::ErrorResponse),
        (status = 500, description = "Internal biofield persistence error", body = crate::ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn create_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateBiofieldSessionRequest>,
) -> Response {
    let Some(repo) = state.biofield_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };

    let user_id = match parse_auth_user_uuid(&auth_user) {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let mut new_session = NewBiofieldSession::new(user_id);
    new_session.client_device_id = request.client_device_id;
    new_session.viewer_version = request.viewer_version;
    new_session.notes = request.context.map(|context| context.to_string());

    match repo.create_session(&new_session).await {
        Ok(session) => (StatusCode::CREATED, Json(session_to_resource(&session))).into_response(),
        Err(error) => biofield_db_error_response("create biofield session", &error),
    }
}

/// POST /api/v1/biofield/sessions/:session_id/close
#[utoipa::path(
    post,
    path = "/api/v1/biofield/sessions/{session_id}/close",
    tag = "biofield",
    params(
        ("session_id" = String, Path, description = "Biofield session identifier")
    ),
    request_body = CloseBiofieldSessionRequest,
    responses(
        (status = 200, description = "Biofield session closed", body = BiofieldSessionResource),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::ErrorResponse),
        (status = 409, description = "Session not active", body = crate::ErrorResponse),
        (status = 422, description = "Invalid session identifier", body = crate::ErrorResponse),
        (status = 503, description = "Biofield DB unavailable", body = crate::ErrorResponse),
        (status = 500, description = "Internal biofield persistence error", body = crate::ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn close_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(session_id): Path<String>,
    Json(request): Json<CloseBiofieldSessionRequest>,
) -> Response {
    let Some(repo) = state.biofield_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };

    let user_id = match parse_auth_user_uuid(&auth_user) {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let session_uuid = match parse_uuid_or_422(&session_id, "session_id") {
        Ok(session_uuid) => session_uuid,
        Err(response) => return response,
    };

    let close_reason = request.reason.as_deref();
    tracing::debug!(session_id = %session_id, close_reason = ?close_reason, "closing biofield session");

    let existing = match repo.get_session(session_uuid, user_id).await {
        Ok(Some(session)) => session,
        Ok(None) => return session_not_found_response(&session_id),
        Err(error) => return biofield_db_error_response("fetch biofield session", &error),
    };

    if existing.status != BIOFIELD_SESSION_STATUS_ACTIVE {
        return session_not_active_response(&existing);
    }

    match repo.close_session(session_uuid, user_id).await {
        Ok(Some(session)) => (StatusCode::OK, Json(session_to_resource(&session))).into_response(),
        Ok(None) => session_not_active_response(&existing),
        Err(error) => biofield_db_error_response("close biofield session", &error),
    }
}

/// GET /api/v1/biofield/sessions/:session_id
#[utoipa::path(
    get,
    path = "/api/v1/biofield/sessions/{session_id}",
    tag = "biofield",
    params(
        ("session_id" = String, Path, description = "Biofield session identifier")
    ),
    responses(
        (status = 200, description = "Biofield session retrieved", body = BiofieldSessionResource),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::ErrorResponse),
        (status = 422, description = "Invalid session identifier", body = crate::ErrorResponse),
        (status = 503, description = "Biofield DB unavailable", body = crate::ErrorResponse),
        (status = 500, description = "Internal biofield persistence error", body = crate::ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn get_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(session_id): Path<String>,
) -> Response {
    let Some(repo) = state.biofield_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };

    let user_id = match parse_auth_user_uuid(&auth_user) {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let session_uuid = match parse_uuid_or_422(&session_id, "session_id") {
        Ok(session_uuid) => session_uuid,
        Err(response) => return response,
    };

    match repo.get_session(session_uuid, user_id).await {
        Ok(Some(session)) => (StatusCode::OK, Json(session_to_resource(&session))).into_response(),
        Ok(None) => session_not_found_response(&session_id),
        Err(error) => biofield_db_error_response("fetch biofield session", &error),
    }
}

/// POST /api/v1/biofield/sessions/:session_id/captures
#[utoipa::path(
    post,
    path = "/api/v1/biofield/sessions/{session_id}/captures",
    tag = "biofield",
    params(
        ("session_id" = String, Path, description = "Biofield session identifier")
    ),
    responses(
        (status = 201, description = "Capture accepted and reading created", body = BiofieldReadingDetail),
        (status = 400, description = "Invalid multipart payload", body = crate::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::ErrorResponse),
        (status = 413, description = "Payload too large", body = crate::ErrorResponse),
        (status = 422, description = "Invalid or rejected capture", body = crate::ErrorResponse),
        (status = 501, description = "Scaffolded route placeholder", body = BiofieldPlaceholderResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn create_capture(
    State(_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(_session_id): Path<String>,
    _multipart: Multipart,
) -> impl IntoResponse {
    not_implemented_response("POST /api/v1/biofield/sessions/:session_id/captures")
}

/// GET /api/v1/biofield/readings
#[utoipa::path(
    get,
    path = "/api/v1/biofield/readings",
    tag = "biofield",
    params(
        ("limit" = Option<u32>, Query, description = "Page size"),
        ("offset" = Option<u32>, Query, description = "Pagination offset")
    ),
    responses(
        (status = 200, description = "Biofield readings listed", body = ListBiofieldReadingsResponse),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 501, description = "Scaffolded route placeholder", body = BiofieldPlaceholderResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn list_readings(
    State(_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Query(query): Query<ListBiofieldReadingsQuery>,
) -> impl IntoResponse {
    let _ = (query.limit, query.offset);
    not_implemented_response("GET /api/v1/biofield/readings")
}

/// GET /api/v1/biofield/readings/:reading_id
#[utoipa::path(
    get,
    path = "/api/v1/biofield/readings/{reading_id}",
    tag = "biofield",
    params(
        ("reading_id" = String, Path, description = "Biofield reading identifier")
    ),
    responses(
        (status = 200, description = "Biofield reading detail", body = BiofieldReadingDetail),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 404, description = "Reading not found", body = crate::ErrorResponse),
        (status = 501, description = "Scaffolded route placeholder", body = BiofieldPlaceholderResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn get_reading(
    State(_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(_reading_id): Path<String>,
) -> impl IntoResponse {
    not_implemented_response("GET /api/v1/biofield/readings/:reading_id")
}
