use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use noesis_auth::AuthUser;
use noesis_data::models::biofield::{
    BIOFIELD_CAPTURE_ARTIFACT_ANALYSIS_OVERLAY, BIOFIELD_CAPTURE_ARTIFACT_SEGMENTATION_MASK,
    BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE, BIOFIELD_CAPTURE_ARTIFACT_THUMBNAIL,
    BIOFIELD_SESSION_STATUS_ACTIVE, BIOFIELD_SESSION_STATUS_CLOSED,
};
use noesis_data::models::reading::NewReading;
use noesis_data::models::reading::Reading;
use noesis_data::models::biofield::{NewBiofieldCaptureArtifact, NewBiofieldSession};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    config::{DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS, DEFAULT_PYTHON_BIOFIELD_URL},
    AppState, BiofieldAnalyzeRequest, BiofieldClient, ErrorMapper,
};

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

fn parse_user_id(user: &AuthUser) -> Result<Uuid, (StatusCode, Json<crate::ErrorResponse>)> {
    Uuid::parse_str(&user.user_id).map_err(|_| {
        ErrorMapper::response(
            StatusCode::BAD_REQUEST,
            "INVALID_USER_ID",
            "Invalid authenticated user id",
            None,
        )
    })
}

fn map_session(session: noesis_data::models::biofield::BiofieldSession) -> BiofieldSessionResource {
    BiofieldSessionResource {
        id: session.id.to_string(),
        status: session.status,
        started_at: session.started_at,
        closed_at: session.closed_at,
        client_device_id: session.client_device_id,
        viewer_version: session.viewer_version,
    }
}

fn sidecar_client_from_env() -> BiofieldClient {
    let base = std::env::var("PYTHON_BIOFIELD_URL")
        .unwrap_or_else(|_| DEFAULT_PYTHON_BIOFIELD_URL.to_string());
    let timeout_ms = std::env::var("PYTHON_BIOFIELD_TIMEOUT_MS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS);
    BiofieldClient::new(base, Duration::from_millis(timeout_ms))
}

fn artifact_storage_path(
    user_id: Uuid,
    session_id: Uuid,
    reading_id: Uuid,
    kind: &str,
    ext: &str,
) -> String {
    format!(
        "biofield/{}/{}/{}/{}-{}.{}",
        user_id,
        session_id,
        reading_id,
        Utc::now().timestamp_millis(),
        kind,
        ext
    )
}

fn infer_extension(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/webp" => "webp",
        _ => "bin",
    }
}

async fn map_reading_detail(
    reading: Reading,
    user_id: Uuid,
    biofield_repo: &noesis_data::repositories::biofield_repository::BiofieldRepository,
) -> Result<BiofieldReadingDetail, (StatusCode, Json<crate::ErrorResponse>)> {
    let artifacts = biofield_repo
        .list_reading_artifacts(reading.id, user_id)
        .await
        .map_err(|err| {
            tracing::error!("biofield list_reading_artifacts failed: {}", err);
            ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to fetch biofield artifacts",
                None,
            )
        })?
        .into_iter()
        .map(|artifact| BiofieldArtifactSummary {
            kind: artifact.artifact_kind,
            mime_type: artifact.mime_type,
        })
        .collect::<Vec<_>>();

    Ok(BiofieldReadingDetail {
        reading_id: reading.id.to_string(),
        session_id: reading
            .input_data
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        engine_id: reading.engine_id,
        created_at: reading.created_at,
        input: reading.input_data,
        result: reading.result_data.clone(),
        quality: reading
            .result_data
            .get("quality_assessment")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        artifacts,
    })
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
        (status = 501, description = "Scaffolded route placeholder", body = BiofieldPlaceholderResponse),
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
) -> impl IntoResponse {
    let Some(repo) = state.biofield_repository.as_ref() else {
        return ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
        .into_response();
    };

    let user_id = match parse_user_id(&auth_user) {
        Ok(uid) => uid,
        Err(err) => return err.into_response(),
    };

    let mut model = NewBiofieldSession::new(user_id);
    model.client_device_id = request.client_device_id;
    model.viewer_version = request.viewer_version;
    model.notes = request.context.map(|ctx| format!("context={ctx}"));

    match repo.create_session(&model).await {
        Ok(session) => (StatusCode::CREATED, Json(map_session(session))).into_response(),
        Err(err) => {
            tracing::error!("biofield create_session failed: {}", err);
            ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to create biofield session",
                None,
            )
            .into_response()
        }
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
        (status = 501, description = "Scaffolded route placeholder", body = BiofieldPlaceholderResponse),
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
) -> impl IntoResponse {
    let Some(repo) = state.biofield_repository.as_ref() else {
        return ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
        .into_response();
    };

    let user_id = match parse_user_id(&auth_user) {
        Ok(uid) => uid,
        Err(err) => return err.into_response(),
    };

    let session_uuid = match Uuid::parse_str(&session_id) {
        Ok(id) => id,
        Err(_) => {
            return ErrorMapper::response(
                StatusCode::BAD_REQUEST,
                "INVALID_SESSION_ID",
                "Invalid session id format",
                None,
            )
            .into_response()
        }
    };

    match repo
        .update_session_status(
            session_uuid,
            user_id,
            BIOFIELD_SESSION_STATUS_CLOSED,
            request.reason.as_deref(),
        )
        .await
    {
        Ok(Some(session)) => (StatusCode::OK, Json(map_session(session))).into_response(),
        Ok(None) => ErrorMapper::response(
            StatusCode::NOT_FOUND,
            "SESSION_NOT_FOUND",
            "Biofield session not found",
            None,
        )
        .into_response(),
        Err(err) => {
            tracing::error!("biofield close_session failed: {}", err);
            ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to close session",
                None,
            )
            .into_response()
        }
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
        (status = 501, description = "Scaffolded route placeholder", body = BiofieldPlaceholderResponse),
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
) -> impl IntoResponse {
    let Some(repo) = state.biofield_repository.as_ref() else {
        return ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
        .into_response();
    };

    let user_id = match parse_user_id(&auth_user) {
        Ok(uid) => uid,
        Err(err) => return err.into_response(),
    };

    let session_uuid = match Uuid::parse_str(&session_id) {
        Ok(id) => id,
        Err(_) => {
            return ErrorMapper::response(
                StatusCode::BAD_REQUEST,
                "INVALID_SESSION_ID",
                "Invalid session id format",
                None,
            )
            .into_response()
        }
    };

    match repo.get_session(session_uuid, user_id).await {
        Ok(Some(session)) => (StatusCode::OK, Json(map_session(session))).into_response(),
        Ok(None) => ErrorMapper::response(
            StatusCode::NOT_FOUND,
            "SESSION_NOT_FOUND",
            "Biofield session not found",
            None,
        )
        .into_response(),
        Err(err) => {
            tracing::error!("biofield get_session failed: {}", err);
            ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to fetch session",
                None,
            )
            .into_response()
        }
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
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(session_id): Path<String>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let Some(biofield_repo) = state.biofield_repository.as_ref() else {
        return ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
        .into_response();
    };
    let Some(readings_repo) = state.readings_repository.as_ref() else {
        return ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
        .into_response();
    };

    let user_id = match parse_user_id(&auth_user) {
        Ok(uid) => uid,
        Err(err) => return err.into_response(),
    };

    let session_uuid = match Uuid::parse_str(&session_id) {
        Ok(id) => id,
        Err(_) => {
            return ErrorMapper::response(
                StatusCode::BAD_REQUEST,
                "INVALID_SESSION_ID",
                "Invalid session id format",
                None,
            )
            .into_response();
        }
    };

    let session = match biofield_repo.get_session(session_uuid, user_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return ErrorMapper::response(
                StatusCode::NOT_FOUND,
                "SESSION_NOT_FOUND",
                "Biofield session not found",
                None,
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!("biofield create_capture get_session failed: {}", err);
            return ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to fetch session",
                None,
            )
            .into_response();
        }
    };

    if session.status != BIOFIELD_SESSION_STATUS_ACTIVE {
        return ErrorMapper::response(
            StatusCode::CONFLICT,
            "SESSION_NOT_ACTIVE",
            "Capture upload requires an active session",
            Some(serde_json::json!({ "status": session.status })),
        )
        .into_response();
    }

    let mut image_bytes: Option<Vec<u8>> = None;
    let mut image_mime = "image/png".to_string();
    let mut algorithms: Option<Vec<String>> = None;
    let mut options_json: Option<serde_json::Value> = None;
    let mut capture_metadata: serde_json::Value = serde_json::json!({});
    let mut segmentation_mask: Option<(Vec<u8>, String)> = None;
    let mut analysis_overlay: Option<(Vec<u8>, String)> = None;
    let mut thumbnail: Option<(Vec<u8>, String)> = None;

    while let Some(field) = match multipart.next_field().await {
        Ok(next) => next,
        Err(err) => {
            return ErrorMapper::response(
                StatusCode::BAD_REQUEST,
                "INVALID_MULTIPART",
                format!("Invalid multipart payload: {err}"),
                None,
            )
            .into_response();
        }
    } {
        let name = field.name().unwrap_or_default().to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        match name.as_str() {
            "image" | "file" => match field.bytes().await {
                Ok(bytes) => {
                    image_bytes = Some(bytes.to_vec());
                    image_mime = content_type;
                }
                Err(err) => {
                    return ErrorMapper::response(
                        StatusCode::BAD_REQUEST,
                        "INVALID_UPLOAD",
                        format!("Failed to read uploaded image: {err}"),
                        None,
                    )
                    .into_response();
                }
            },
            "algorithms" => {
                if let Ok(raw) = field.text().await {
                    let parsed = serde_json::from_str::<Vec<String>>(&raw).ok();
                    algorithms = parsed;
                }
            }
            "options" => {
                if let Ok(raw) = field.text().await {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) {
                        options_json = Some(parsed);
                    }
                }
            }
            "capture_metadata" => {
                if let Ok(raw) = field.text().await {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) {
                        capture_metadata = parsed;
                    }
                }
            }
            "segmentation_mask" => {
                if let Ok(bytes) = field.bytes().await {
                    segmentation_mask = Some((bytes.to_vec(), content_type));
                }
            }
            "analysis_overlay" => {
                if let Ok(bytes) = field.bytes().await {
                    analysis_overlay = Some((bytes.to_vec(), content_type));
                }
            }
            "thumbnail" => {
                if let Ok(bytes) = field.bytes().await {
                    thumbnail = Some((bytes.to_vec(), content_type));
                }
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let image_bytes = match image_bytes {
        Some(bytes) => bytes,
        None => {
            return ErrorMapper::response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "MISSING_IMAGE_FILE",
                "No image file found. Provide multipart field named `file` or `image`.",
                None,
            )
            .into_response();
        }
    };

    if image_bytes.len() > 10 * 1024 * 1024 {
        return ErrorMapper::response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "PAYLOAD_TOO_LARGE",
            "Capture file exceeds 10MB limit",
            None,
        )
        .into_response();
    }

    let sidecar = sidecar_client_from_env();
    let sidecar_response = match sidecar
        .analyze_capture(BiofieldAnalyzeRequest {
            image_data: image_bytes.clone(),
            content_type: image_mime.clone(),
            algorithms,
            options: options_json.clone(),
        })
        .await
    {
        Ok(payload) => payload,
        Err(err) => {
            tracing::error!("biofield sidecar analyze failed: {}", err);
            return ErrorMapper::response(
                StatusCode::BAD_GATEWAY,
                "SIDECAR_UNAVAILABLE",
                "Biofield analysis sidecar unavailable",
                None,
            )
            .into_response();
        }
    };

    if !sidecar_response
        .get("quality_assessment")
        .and_then(|q| q.get("sufficient_quality"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
    {
        return ErrorMapper::response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "INSUFFICIENT_CAPTURE_QUALITY",
            "Capture rejected due to quality assessment",
            Some(serde_json::json!({ "quality": sidecar_response.get("quality_assessment") })),
        )
        .into_response();
    }

    let mut input_data = serde_json::json!({
        "session_id": session_uuid,
        "mime_type": image_mime,
        "capture_metadata": capture_metadata,
        "sidecar_options": options_json,
    });

    if let Some(metrics) = input_data
        .get_mut("capture_metadata")
        .and_then(|v| v.get("realtime_metrics"))
        .cloned()
    {
        input_data["realtime_metrics"] = metrics;
    }

    let reading = NewReading {
        user_id,
        engine_id: "biofield-capture".to_string(),
        workflow_id: None,
        input_hash: format!("{:x}", Sha256::digest(&image_bytes)),
        input_data,
        result_data: sidecar_response.clone(),
        witness_prompt: None,
        consciousness_level: auth_user.consciousness_level as i16,
        calculation_time_ms: sidecar_response
            .get("processing_time_ms")
            .and_then(|v| v.as_f64()),
        client_event_id: None,
        client_device_id: None,
        device_platform: None,
        device_app_version: None,
    };

    let reading_id = match readings_repo.save_reading(&reading).await {
        Ok(id) => id,
        Err(err) => {
            tracing::error!("biofield create_capture save_reading failed: {}", err);
            return ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to persist reading",
                None,
            )
            .into_response();
        }
    };

    let source_artifact = NewBiofieldCaptureArtifact {
        session_id: session_uuid,
        reading_id: Some(reading_id),
        artifact_kind: BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE.to_string(),
        storage_path: artifact_storage_path(
            user_id,
            session_uuid,
            reading_id,
            BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE,
            infer_extension(&image_mime),
        ),
        mime_type: image_mime.clone(),
        byte_size: image_bytes.len() as i64,
        capture_metadata: serde_json::json!({
            "source": "viewer-upload",
            "capture_metadata": capture_metadata,
            "sidecar_response_summary": {
                "analysis_version": sidecar_response.get("analysis_version"),
                "algorithms_run": sidecar_response.get("algorithms_run")
            }
        }),
    };

    if let Err(err) = biofield_repo.create_artifact(&source_artifact).await {
        tracing::warn!("biofield create_capture source artifact insert failed: {}", err);
    }

    if let Some((bytes, mime)) = segmentation_mask {
        let model = NewBiofieldCaptureArtifact {
            session_id: session_uuid,
            reading_id: Some(reading_id),
            artifact_kind: BIOFIELD_CAPTURE_ARTIFACT_SEGMENTATION_MASK.to_string(),
            storage_path: artifact_storage_path(
                user_id,
                session_uuid,
                reading_id,
                BIOFIELD_CAPTURE_ARTIFACT_SEGMENTATION_MASK,
                infer_extension(&mime),
            ),
            mime_type: mime,
            byte_size: bytes.len() as i64,
            capture_metadata: serde_json::json!({ "source": "viewer-segmentation-mask" }),
        };
        if let Err(err) = biofield_repo.create_artifact(&model).await {
            tracing::warn!("biofield create_capture segmentation artifact insert failed: {}", err);
        }
    }

    if let Some((bytes, mime)) = analysis_overlay {
        let model = NewBiofieldCaptureArtifact {
            session_id: session_uuid,
            reading_id: Some(reading_id),
            artifact_kind: BIOFIELD_CAPTURE_ARTIFACT_ANALYSIS_OVERLAY.to_string(),
            storage_path: artifact_storage_path(
                user_id,
                session_uuid,
                reading_id,
                BIOFIELD_CAPTURE_ARTIFACT_ANALYSIS_OVERLAY,
                infer_extension(&mime),
            ),
            mime_type: mime,
            byte_size: bytes.len() as i64,
            capture_metadata: serde_json::json!({ "source": "viewer-overlay" }),
        };
        if let Err(err) = biofield_repo.create_artifact(&model).await {
            tracing::warn!("biofield create_capture overlay artifact insert failed: {}", err);
        }
    }

    if let Some((bytes, mime)) = thumbnail {
        let model = NewBiofieldCaptureArtifact {
            session_id: session_uuid,
            reading_id: Some(reading_id),
            artifact_kind: BIOFIELD_CAPTURE_ARTIFACT_THUMBNAIL.to_string(),
            storage_path: artifact_storage_path(
                user_id,
                session_uuid,
                reading_id,
                BIOFIELD_CAPTURE_ARTIFACT_THUMBNAIL,
                infer_extension(&mime),
            ),
            mime_type: mime,
            byte_size: bytes.len() as i64,
            capture_metadata: serde_json::json!({ "source": "viewer-thumbnail" }),
        };
        if let Err(err) = biofield_repo.create_artifact(&model).await {
            tracing::warn!("biofield create_capture thumbnail artifact insert failed: {}", err);
        }
    }

    let reading_model = match readings_repo.get_reading(reading_id, user_id).await {
        Ok(Some(model)) => model,
        Ok(None) => {
            return ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "READING_NOT_FOUND",
                "Reading persisted but could not be fetched",
                None,
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!("biofield create_capture get_reading failed: {}", err);
            return ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to fetch persisted reading",
                None,
            )
            .into_response();
        }
    };

    match map_reading_detail(reading_model, user_id, biofield_repo).await {
        Ok(detail) => (StatusCode::CREATED, Json(detail)).into_response(),
        Err(err) => err.into_response(),
    }
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
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListBiofieldReadingsQuery>,
) -> impl IntoResponse {
    let Some(readings_repo) = state.readings_repository.as_ref() else {
        return ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
        .into_response();
    };
    let Some(biofield_repo) = state.biofield_repository.as_ref() else {
        return ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
        .into_response();
    };

    let user_id = match parse_user_id(&auth_user) {
        Ok(uid) => uid,
        Err(err) => return err.into_response(),
    };

    let limit = query.limit.unwrap_or(20).min(100) as i64;
    let offset = query.offset.unwrap_or(0) as i64;

    let readings = match readings_repo
        .list_readings(user_id, Some("biofield-capture"), limit, offset)
        .await
    {
        Ok(items) => items,
        Err(err) => {
            tracing::error!("biofield list_readings failed: {}", err);
            return ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to list readings",
                None,
            )
            .into_response();
        }
    };

    let mut items = Vec::with_capacity(readings.len());
    for reading in readings {
        let artifact = match biofield_repo
            .list_reading_artifacts(reading.id, user_id)
            .await
            .ok()
            .and_then(|v| v.into_iter().next())
        {
            Some(artifact) => BiofieldArtifactSummary {
                kind: artifact.artifact_kind,
                mime_type: artifact.mime_type,
            },
            None => BiofieldArtifactSummary {
                kind: BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE.to_string(),
                mime_type: "image/png".to_string(),
            },
        };

        let sufficient_quality = reading
            .result_data
            .get("quality_assessment")
            .and_then(|q| q.get("sufficient_quality"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let session_id = reading
            .input_data
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        items.push(BiofieldReadingSummary {
            reading_id: reading.id.to_string(),
            session_id,
            engine_id: reading.engine_id,
            created_at: reading.created_at,
            quality: BiofieldQualitySummary { sufficient_quality },
            artifact,
        });
    }

    (
        StatusCode::OK,
        Json(ListBiofieldReadingsResponse {
            items,
            limit: limit as u32,
            offset: offset as u32,
        }),
    )
        .into_response()
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
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(reading_id): Path<String>,
) -> impl IntoResponse {
    let Some(readings_repo) = state.readings_repository.as_ref() else {
        return ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
        .into_response();
    };
    let Some(biofield_repo) = state.biofield_repository.as_ref() else {
        return ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "DB_UNAVAILABLE",
            "Database not available",
            None,
        )
        .into_response();
    };

    let user_id = match parse_user_id(&auth_user) {
        Ok(uid) => uid,
        Err(err) => return err.into_response(),
    };

    let reading_uuid = match Uuid::parse_str(&reading_id) {
        Ok(id) => id,
        Err(_) => {
            return ErrorMapper::response(
                StatusCode::BAD_REQUEST,
                "INVALID_READING_ID",
                "Invalid reading id format",
                None,
            )
            .into_response();
        }
    };

    let reading = match readings_repo.get_reading(reading_uuid, user_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return ErrorMapper::response(
                StatusCode::NOT_FOUND,
                "READING_NOT_FOUND",
                "Biofield reading not found",
                None,
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!("biofield get_reading failed: {}", err);
            return ErrorMapper::response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Failed to fetch reading",
                None,
            )
            .into_response();
        }
    };

    if reading.engine_id != "biofield-capture" {
        return ErrorMapper::response(
            StatusCode::NOT_FOUND,
            "READING_NOT_FOUND",
            "Biofield reading not found",
            None,
        )
        .into_response();
    }

    match map_reading_detail(reading, user_id, biofield_repo).await {
        Ok(detail) => (StatusCode::OK, Json(detail)).into_response(),
        Err(err) => err.into_response(),
    }
}
