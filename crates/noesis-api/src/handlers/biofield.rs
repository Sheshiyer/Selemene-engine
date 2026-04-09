use crate::{config, AppState, BiofieldAnalyzeRequest, BiofieldClient, ErrorMapper};
use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use noesis_auth::AuthUser;
use noesis_bridge::BridgeError;
use noesis_data::{
    models::{
        biofield::{BiofieldSession, NewBiofieldSession, BIOFIELD_SESSION_STATUS_ACTIVE},
        reading::NewReading,
    },
    repositories::readings_repository::ReadingsRepository,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::Duration;
use utoipa::ToSchema;
use uuid::Uuid;

const BIOFIELD_ENGINE_ID: &str = "biofield-capture";

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
    pub id: Option<String>,
    pub kind: String,
    pub mime_type: String,
    pub storage_path: Option<String>,
    pub byte_size: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BiofieldCaptureResponse {
    pub reading_id: String,
    pub session_id: String,
    pub analysis_version: String,
    #[schema(value_type = Object)]
    pub metrics: serde_json::Value,
    #[schema(value_type = Object)]
    pub quality_assessment: serde_json::Value,
    pub artifacts: Vec<BiofieldArtifactSummary>,
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

#[derive(Debug, Default)]
struct ParsedBiofieldCapture {
    image_bytes: Option<Vec<u8>>,
    image_content_type: Option<String>,
    image_file_name: Option<String>,
    algorithms: Option<Vec<String>>,
    options: Option<Value>,
    capture_metadata: Option<Value>,
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

fn biofield_analysis_unavailable_response(details: Option<Value>) -> Response {
    json_error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Biofield analysis service is unavailable",
        "BIOFIELD_ANALYSIS_UNAVAILABLE",
        details,
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

fn capture_invalid_response(message: impl Into<String>, details: Option<Value>) -> Response {
    json_error_response(
        StatusCode::BAD_REQUEST,
        message,
        "BIOFIELD_CAPTURE_INVALID",
        details,
    )
}

fn capture_too_large_response(details: Option<Value>) -> Response {
    json_error_response(
        StatusCode::PAYLOAD_TOO_LARGE,
        "Biofield capture payload exceeds the accepted size limit",
        "BIOFIELD_CAPTURE_TOO_LARGE",
        details,
    )
}

fn capture_rejected_quality_response(
    message: impl Into<String>,
    quality_assessment: Value,
    analysis_version: Option<&str>,
) -> Response {
    json_error_response(
        StatusCode::UNPROCESSABLE_ENTITY,
        message,
        "BIOFIELD_CAPTURE_REJECTED_QUALITY",
        Some(serde_json::json!({
            "quality_assessment": quality_assessment,
            "analysis_version": analysis_version,
        })),
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

fn build_biofield_client_from_env() -> BiofieldClient {
    let base_url = std::env::var("PYTHON_BIOFIELD_URL")
        .unwrap_or_else(|_| config::DEFAULT_PYTHON_BIOFIELD_URL.to_string());
    let timeout_ms = std::env::var("PYTHON_BIOFIELD_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(config::DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS);

    BiofieldClient::new(base_url, Duration::from_millis(timeout_ms))
}

fn parse_json_string_array(raw: &str, field_name: &str) -> Result<Vec<String>, Response> {
    let parsed = serde_json::from_str::<Value>(raw).map_err(|error| {
        capture_invalid_response(
            format!("{field_name} must be valid JSON: {error}"),
            Some(serde_json::json!({ "field": field_name })),
        )
    })?;

    let items = parsed.as_array().ok_or_else(|| {
        capture_invalid_response(
            format!("{field_name} must be a JSON array of strings"),
            Some(serde_json::json!({ "field": field_name })),
        )
    })?;

    let mut values = Vec::with_capacity(items.len());
    for item in items {
        let value = item.as_str().ok_or_else(|| {
            capture_invalid_response(
                format!("{field_name} must be a JSON array of strings"),
                Some(serde_json::json!({ "field": field_name })),
            )
        })?;
        values.push(value.to_string());
    }

    Ok(values)
}

fn parse_json_object(raw: &str, field_name: &str) -> Result<Value, Response> {
    let parsed = serde_json::from_str::<Value>(raw).map_err(|error| {
        capture_invalid_response(
            format!("{field_name} must be valid JSON: {error}"),
            Some(serde_json::json!({ "field": field_name })),
        )
    })?;

    if !parsed.is_object() {
        return Err(capture_invalid_response(
            format!("{field_name} must be a JSON object"),
            Some(serde_json::json!({ "field": field_name })),
        ));
    }

    Ok(parsed)
}

async fn parse_capture_multipart(
    mut multipart: Multipart,
) -> Result<ParsedBiofieldCapture, Response> {
    let mut parsed = ParsedBiofieldCapture::default();

    while let Some(field) = multipart.next_field().await.map_err(|error| {
        capture_invalid_response(
            format!("Invalid multipart payload: {error}"),
            Some(serde_json::json!({ "error": error.to_string() })),
        )
    })? {
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            "image" | "file" => {
                if parsed.image_bytes.is_none() {
                    let content_type = field
                        .content_type()
                        .map(str::to_string)
                        .unwrap_or_else(|| "application/octet-stream".to_string());
                    let file_name = field.file_name().map(str::to_string);
                    let bytes = field.bytes().await.map_err(|error| {
                        capture_invalid_response(
                            format!("Failed to read uploaded file: {error}"),
                            Some(serde_json::json!({ "field": field_name })),
                        )
                    })?;
                    parsed.image_bytes = Some(bytes.to_vec());
                    parsed.image_content_type = Some(content_type);
                    parsed.image_file_name = file_name;
                }
            }
            "algorithms" => {
                let text = field.text().await.map_err(|error| {
                    capture_invalid_response(
                        format!("Failed to read algorithms field: {error}"),
                        Some(serde_json::json!({ "field": field_name })),
                    )
                })?;

                if !text.trim().is_empty() {
                    parsed.algorithms = Some(parse_json_string_array(&text, "algorithms")?);
                }
            }
            "options" => {
                let text = field.text().await.map_err(|error| {
                    capture_invalid_response(
                        format!("Failed to read options field: {error}"),
                        Some(serde_json::json!({ "field": field_name })),
                    )
                })?;

                if !text.trim().is_empty() {
                    parsed.options = Some(parse_json_object(&text, "options")?);
                }
            }
            "capture_metadata" => {
                let text = field.text().await.map_err(|error| {
                    capture_invalid_response(
                        format!("Failed to read capture_metadata field: {error}"),
                        Some(serde_json::json!({ "field": field_name })),
                    )
                })?;

                if !text.trim().is_empty() {
                    parsed.capture_metadata = Some(parse_json_object(&text, "capture_metadata")?);
                }
            }
            _ => {}
        }
    }

    if parsed.image_bytes.is_none() {
        return Err(capture_invalid_response(
            "No image file found. Provide multipart field named `file` or `image`.",
            None,
        ));
    }

    Ok(parsed)
}

fn map_bridge_error(error: BridgeError) -> Response {
    match error {
        BridgeError::Timeout { timeout_secs } => {
            biofield_analysis_unavailable_response(Some(serde_json::json!({
                "reason": "timeout",
                "timeout_secs": timeout_secs,
            })))
        }
        BridgeError::ConnectionRefused { url } => {
            biofield_analysis_unavailable_response(Some(serde_json::json!({
                "reason": "connection_refused",
                "url": url,
            })))
        }
        BridgeError::ServerUnavailable(message)
        | BridgeError::HttpError(message)
        | BridgeError::DeserializationError(message) => {
            biofield_analysis_unavailable_response(Some(serde_json::json!({
                "reason": message,
            })))
        }
        BridgeError::EngineResponse { status, body } => {
            let parsed_body = serde_json::from_str::<Value>(&body).ok();
            let details = parsed_body
                .clone()
                .or_else(|| Some(serde_json::json!({ "upstream_body": body })));

            if status == StatusCode::PAYLOAD_TOO_LARGE.as_u16() {
                return capture_too_large_response(details);
            }

            if status == StatusCode::UNPROCESSABLE_ENTITY.as_u16() {
                let error_code = parsed_body
                    .as_ref()
                    .and_then(|value| value.get("error_code"))
                    .and_then(|value| value.as_str());
                let error_message = parsed_body
                    .as_ref()
                    .and_then(|value| {
                        value
                            .get("error_message")
                            .or_else(|| value.get("message"))
                            .or_else(|| value.get("detail"))
                    })
                    .and_then(|value| value.as_str())
                    .unwrap_or("Biofield capture was rejected by the analysis service");

                if error_code == Some("BIOFIELD_CAPTURE_REJECTED_QUALITY") {
                    let quality = parsed_body
                        .as_ref()
                        .and_then(|value| value.get("quality_assessment"))
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}));
                    let analysis_version = parsed_body
                        .as_ref()
                        .and_then(|value| value.get("analysis_version"))
                        .and_then(|value| value.as_str());
                    return capture_rejected_quality_response(
                        error_message,
                        quality,
                        analysis_version,
                    );
                }

                return json_error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    error_message,
                    "BIOFIELD_CAPTURE_INVALID",
                    details,
                );
            }

            if (400..500).contains(&status) {
                return capture_invalid_response(
                    format!(
                        "Biofield capture request was rejected by the analysis service ({status})"
                    ),
                    details,
                );
            }

            biofield_analysis_unavailable_response(details)
        }
    }
}

fn create_input_hash(
    session_id: Uuid,
    image_bytes: &[u8],
    content_type: &str,
    algorithms: Option<&Vec<String>>,
    options: Option<&Value>,
    capture_metadata: Option<&Value>,
) -> String {
    let metadata = serde_json::json!({
        "session_id": session_id,
        "content_type": content_type,
        "algorithms": algorithms,
        "options": options,
        "capture_metadata": capture_metadata,
    });

    let mut hasher = Sha256::new();
    hasher.update(image_bytes);
    hasher.update(metadata.to_string().as_bytes());
    format!("{:x}", hasher.finalize())
}

fn build_capture_artifacts(content_type: &str, byte_size: usize) -> Vec<BiofieldArtifactSummary> {
    vec![BiofieldArtifactSummary {
        id: None,
        kind: "source-image".to_string(),
        mime_type: content_type.to_string(),
        storage_path: None,
        byte_size: Some(byte_size as u64),
    }]
}

fn build_capture_response(
    reading_id: Uuid,
    session_id: Uuid,
    analysis_version: &str,
    metrics: Value,
    quality_assessment: Value,
    artifacts: Vec<BiofieldArtifactSummary>,
) -> BiofieldCaptureResponse {
    BiofieldCaptureResponse {
        reading_id: reading_id.to_string(),
        session_id: session_id.to_string(),
        analysis_version: analysis_version.to_string(),
        metrics,
        quality_assessment,
        artifacts,
    }
}

async fn save_capture_reading(
    readings_repository: &ReadingsRepository,
    auth_user: &AuthUser,
    user_id: Uuid,
    session: &BiofieldSession,
    parsed_capture: &ParsedBiofieldCapture,
    sidecar_response: &Value,
    content_type: &str,
    image_bytes: &[u8],
) -> Result<Uuid, sqlx::Error> {
    let input_hash = create_input_hash(
        session.id,
        image_bytes,
        content_type,
        parsed_capture.algorithms.as_ref(),
        parsed_capture.options.as_ref(),
        parsed_capture.capture_metadata.as_ref(),
    );

    let input_data = serde_json::json!({
        "session_id": session.id,
        "file_name": parsed_capture.image_file_name,
        "content_type": content_type,
        "algorithms": parsed_capture.algorithms,
        "options": parsed_capture.options,
        "capture_metadata": parsed_capture.capture_metadata,
    });

    let processing_time_ms = sidecar_response
        .get("processing_time_ms")
        .and_then(|value| value.as_f64());

    let device_platform = parsed_capture
        .capture_metadata
        .as_ref()
        .and_then(|value| value.get("platform"))
        .and_then(|value| value.as_str())
        .map(str::to_string);

    let client_device_id = session.client_device_id.clone();
    let device_app_version = session.viewer_version.clone();

    readings_repository
        .save_reading(&NewReading {
            user_id,
            engine_id: BIOFIELD_ENGINE_ID.to_string(),
            workflow_id: None,
            input_hash,
            input_data,
            result_data: sidecar_response.clone(),
            witness_prompt: None,
            consciousness_level: auth_user.consciousness_level as i16,
            calculation_time_ms: processing_time_ms,
            client_event_id: None,
            client_device_id,
            device_platform,
            device_app_version,
        })
        .await
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
        (status = 201, description = "Capture accepted and analysis returned", body = BiofieldCaptureResponse),
        (status = 400, description = "Invalid multipart payload", body = crate::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::ErrorResponse),
        (status = 409, description = "Session not active", body = crate::ErrorResponse),
        (status = 413, description = "Payload too large", body = crate::ErrorResponse),
        (status = 422, description = "Invalid or rejected capture", body = crate::ErrorResponse),
        (status = 503, description = "Biofield analysis unavailable", body = crate::ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn create_capture(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(session_id): Path<String>,
    multipart: Multipart,
) -> Response {
    let Some(biofield_repository) = state.biofield_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };
    let Some(readings_repository) = state.readings_repository.as_ref() else {
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

    let session = match biofield_repository.get_session(session_uuid, user_id).await {
        Ok(Some(session)) => session,
        Ok(None) => return session_not_found_response(&session_id),
        Err(error) => return biofield_db_error_response("fetch biofield session", &error),
    };

    if session.status != BIOFIELD_SESSION_STATUS_ACTIVE {
        return session_not_active_response(&session);
    }

    let parsed_capture = match parse_capture_multipart(multipart).await {
        Ok(parsed_capture) => parsed_capture,
        Err(response) => return response,
    };

    let image_bytes = parsed_capture
        .image_bytes
        .clone()
        .expect("image_bytes should exist after multipart parsing");
    let content_type = parsed_capture
        .image_content_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());

    if !content_type.starts_with("image/") {
        return json_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Uploaded capture must be an image/* content type",
            "BIOFIELD_CAPTURE_INVALID",
            Some(serde_json::json!({ "content_type": content_type })),
        );
    }

    let sidecar_client = build_biofield_client_from_env();
    let sidecar_response = match sidecar_client
        .analyze_capture(BiofieldAnalyzeRequest {
            image_data: image_bytes.clone(),
            content_type: content_type.clone(),
            algorithms: parsed_capture.algorithms.clone(),
            options: parsed_capture.options.clone(),
        })
        .await
    {
        Ok(response) => response,
        Err(error) => return map_bridge_error(error),
    };

    let analysis_version = match sidecar_response
        .get("analysis_version")
        .and_then(|value| value.as_str())
    {
        Some(analysis_version) => analysis_version,
        None => {
            return biofield_analysis_unavailable_response(Some(serde_json::json!({
                "reason": "missing analysis_version in sidecar response",
            })))
        }
    };

    let metrics = match sidecar_response.get("metrics").cloned() {
        Some(metrics) => metrics,
        None => {
            return biofield_analysis_unavailable_response(Some(serde_json::json!({
                "reason": "missing metrics in sidecar response",
            })))
        }
    };

    let quality_assessment = match sidecar_response.get("quality_assessment").cloned() {
        Some(quality_assessment) => quality_assessment,
        None => {
            return biofield_analysis_unavailable_response(Some(serde_json::json!({
                "reason": "missing quality_assessment in sidecar response",
            })))
        }
    };

    let sufficient_quality = match quality_assessment
        .get("sufficient_quality")
        .and_then(|value| value.as_bool())
    {
        Some(sufficient_quality) => sufficient_quality,
        None => {
            return biofield_analysis_unavailable_response(Some(serde_json::json!({
                "reason": "missing sufficient_quality in sidecar response",
            })))
        }
    };

    if !sufficient_quality {
        return capture_rejected_quality_response(
            "Biofield capture rejected for insufficient quality",
            quality_assessment,
            Some(analysis_version),
        );
    }

    let reading_id = match save_capture_reading(
        readings_repository,
        &auth_user,
        user_id,
        &session,
        &parsed_capture,
        &sidecar_response,
        &content_type,
        &image_bytes,
    )
    .await
    {
        Ok(reading_id) => reading_id,
        Err(error) => {
            return biofield_db_error_response("persist biofield capture reading", &error)
        }
    };

    let artifacts = build_capture_artifacts(&content_type, image_bytes.len());
    let response = build_capture_response(
        reading_id,
        session.id,
        analysis_version,
        metrics,
        quality_assessment,
        artifacts,
    );

    (StatusCode::CREATED, Json(response)).into_response()
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
