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
        biofield::{
            BiofieldBaselineSummaryRecord, BiofieldCaptureArtifact, BiofieldSession,
            NewBiofieldBaseline, NewBiofieldCaptureArtifact, NewBiofieldSession,
            BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE, BIOFIELD_SESSION_STATUS_ACTIVE,
        },
        reading::{NewReading, Reading},
    },
    repositories::{
        biofield_repository::BiofieldRepository, readings_repository::ReadingsRepository,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{path::PathBuf, time::Duration};
use tokio::fs as tokio_fs;
use utoipa::ToSchema;
use uuid::Uuid;

const BIOFIELD_ENGINE_ID: &str = "biofield-capture";
const DEFAULT_BIOFIELD_READINGS_LIMIT: u32 = 20;
const MAX_BIOFIELD_READINGS_LIMIT: u32 = 100;

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

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct ReprocessBiofieldReadingRequest {
    pub algorithms: Option<Vec<String>>,
    #[schema(value_type = Object)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBiofieldBaselineRequest {
    pub name: String,
    pub notes: Option<String>,
    pub reading_ids: Vec<String>,
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
pub struct BiofieldReprocessResponse {
    pub source_reading_id: String,
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
pub struct BiofieldBaselineSummary {
    pub baseline_id: String,
    pub name: String,
    pub notes: Option<String>,
    pub reading_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListBiofieldBaselinesResponse {
    pub items: Vec<BiofieldBaselineSummary>,
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

#[derive(Debug, Default)]
struct ParsedBiofieldCapture {
    image_bytes: Option<Vec<u8>>,
    image_content_type: Option<String>,
    image_file_name: Option<String>,
    algorithms: Option<Vec<String>>,
    options: Option<Value>,
    capture_metadata: Option<Value>,
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

fn reading_not_found_response(reading_id: &str) -> Response {
    json_error_response(
        StatusCode::NOT_FOUND,
        "Biofield reading not found",
        "BIOFIELD_READING_NOT_FOUND",
        Some(serde_json::json!({ "reading_id": reading_id })),
    )
}

fn missing_reading_session_link_response(reading_id: Uuid) -> Response {
    json_error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Biofield reading is missing session linkage",
        "BIOFIELD_DB_ERROR",
        Some(serde_json::json!({ "reading_id": reading_id })),
    )
}

fn artifact_not_found_response(reading_id: &str) -> Response {
    json_error_response(
        StatusCode::NOT_FOUND,
        "Biofield source artifact not found",
        "BIOFIELD_ARTIFACT_NOT_FOUND",
        Some(serde_json::json!({ "reading_id": reading_id })),
    )
}

fn baseline_invalid_response(message: impl Into<String>, details: Option<Value>) -> Response {
    json_error_response(
        StatusCode::UNPROCESSABLE_ENTITY,
        message,
        "BIOFIELD_BASELINE_INVALID",
        details,
    )
}

fn biofield_storage_error_response(
    action: &str,
    storage_path: &str,
    error: &std::io::Error,
) -> Response {
    tracing::error!(
        storage_path = storage_path,
        "biofield storage error during {action}: {error}"
    );
    json_error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Failed to {action}"),
        "BIOFIELD_STORAGE_ERROR",
        Some(serde_json::json!({
            "action": action,
            "storage_path": storage_path,
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

fn artifact_to_summary(artifact: &BiofieldCaptureArtifact) -> BiofieldArtifactSummary {
    BiofieldArtifactSummary {
        id: Some(artifact.id.to_string()),
        kind: artifact.artifact_kind.clone(),
        mime_type: artifact.mime_type.clone(),
        storage_path: Some(artifact.storage_path.clone()),
        byte_size: Some(artifact.byte_size as u64),
    }
}

fn fallback_artifact_summary(reading: &Reading) -> BiofieldArtifactSummary {
    let byte_size = reading
        .input_data
        .get("capture_metadata")
        .and_then(|value| value.get("file_size"))
        .and_then(Value::as_u64);

    BiofieldArtifactSummary {
        id: None,
        kind: BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE.to_string(),
        mime_type: reading
            .input_data
            .get("content_type")
            .and_then(Value::as_str)
            .unwrap_or("application/octet-stream")
            .to_string(),
        storage_path: None,
        byte_size,
    }
}

fn extract_reading_session_id(
    reading: &Reading,
    artifacts: &[BiofieldCaptureArtifact],
) -> Option<String> {
    artifacts
        .first()
        .map(|artifact| artifact.session_id.to_string())
        .or_else(|| {
            reading
                .input_data
                .get("session_id")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

fn build_reading_summary(
    reading: &Reading,
    artifacts: &[BiofieldCaptureArtifact],
) -> Result<BiofieldReadingSummary, Response> {
    let session_id = extract_reading_session_id(reading, artifacts)
        .ok_or_else(|| missing_reading_session_link_response(reading.id))?;
    let sufficient_quality = reading
        .result_data
        .get("quality_assessment")
        .and_then(|value| value.get("sufficient_quality"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let artifact = artifacts
        .first()
        .map(artifact_to_summary)
        .unwrap_or_else(|| fallback_artifact_summary(reading));

    Ok(BiofieldReadingSummary {
        reading_id: reading.id.to_string(),
        session_id,
        engine_id: BIOFIELD_ENGINE_ID.to_string(),
        created_at: reading.created_at,
        quality: BiofieldQualitySummary { sufficient_quality },
        artifact,
    })
}

fn build_reading_detail_resource(
    reading: &Reading,
    artifacts: &[BiofieldCaptureArtifact],
) -> Result<BiofieldReadingDetail, Response> {
    let session_id = extract_reading_session_id(reading, artifacts)
        .ok_or_else(|| missing_reading_session_link_response(reading.id))?;
    let artifact_summaries = if artifacts.is_empty() {
        vec![fallback_artifact_summary(reading)]
    } else {
        artifacts.iter().map(artifact_to_summary).collect()
    };

    Ok(BiofieldReadingDetail {
        reading_id: reading.id.to_string(),
        session_id,
        engine_id: BIOFIELD_ENGINE_ID.to_string(),
        created_at: reading.created_at,
        input: reading.input_data.clone(),
        result: reading.result_data.clone(),
        quality: reading
            .result_data
            .get("quality_assessment")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        artifacts: artifact_summaries,
    })
}

fn normalize_readings_page(query: &ListBiofieldReadingsQuery) -> (u32, u32) {
    let limit = query
        .limit
        .unwrap_or(DEFAULT_BIOFIELD_READINGS_LIMIT)
        .clamp(1, MAX_BIOFIELD_READINGS_LIMIT);
    let offset = query.offset.unwrap_or(0);
    (limit, offset)
}

fn infer_image_extension(file_name: Option<&str>, content_type: &str) -> String {
    if let Some(file_name) = file_name {
        if let Some((_, extension)) = file_name.rsplit_once('.') {
            if !extension.is_empty()
                && extension.len() <= 8
                && extension
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
            {
                return extension.to_ascii_lowercase();
            }
        }
    }

    match content_type {
        "image/jpeg" => "jpg".to_string(),
        "image/png" => "png".to_string(),
        "image/webp" => "webp".to_string(),
        "image/gif" => "gif".to_string(),
        "image/bmp" => "bmp".to_string(),
        _ => "bin".to_string(),
    }
}

fn build_source_artifact_storage_path(
    session_id: Uuid,
    file_name: Option<&str>,
    content_type: &str,
) -> String {
    let extension = infer_image_extension(file_name, content_type);
    format!(
        "biofield/{session_id}/source-{}.{}",
        Uuid::new_v4(),
        extension
    )
}

fn build_biofield_artifacts_root() -> PathBuf {
    std::env::var("BIOFIELD_ARTIFACTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(config::DEFAULT_BIOFIELD_ARTIFACTS_DIR))
}

fn resolve_artifact_file_path(storage_path: &str) -> PathBuf {
    build_biofield_artifacts_root().join(storage_path)
}

async fn persist_artifact_bytes(storage_path: &str, bytes: &[u8]) -> Result<(), std::io::Error> {
    let absolute_path = resolve_artifact_file_path(storage_path);
    if let Some(parent) = absolute_path.parent() {
        tokio_fs::create_dir_all(parent).await?;
    }
    tokio_fs::write(absolute_path, bytes).await
}

async fn load_artifact_bytes(storage_path: &str) -> Result<Vec<u8>, std::io::Error> {
    tokio_fs::read(resolve_artifact_file_path(storage_path)).await
}

fn build_artifact_capture_metadata(
    parsed_capture: &ParsedBiofieldCapture,
    content_type: &str,
) -> Value {
    serde_json::json!({
        "file_name": parsed_capture.image_file_name,
        "content_type": content_type,
        "algorithms": parsed_capture.algorithms,
        "options": parsed_capture.options,
        "capture_metadata": parsed_capture.capture_metadata,
    })
}

async fn create_source_artifact(
    biofield_repository: &BiofieldRepository,
    session: &BiofieldSession,
    parsed_capture: &ParsedBiofieldCapture,
    content_type: &str,
    image_bytes: &[u8],
) -> Result<BiofieldCaptureArtifact, Response> {
    let storage_path = build_source_artifact_storage_path(
        session.id,
        parsed_capture.image_file_name.as_deref(),
        content_type,
    );

    if let Err(error) = persist_artifact_bytes(&storage_path, image_bytes).await {
        return Err(biofield_storage_error_response(
            "persist biofield source artifact bytes",
            &storage_path,
            &error,
        ));
    }

    match biofield_repository
        .create_artifact(&NewBiofieldCaptureArtifact {
            session_id: session.id,
            reading_id: None,
            artifact_kind: BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE.to_string(),
            storage_path: storage_path.clone(),
            mime_type: content_type.to_string(),
            byte_size: image_bytes.len() as i64,
            capture_metadata: build_artifact_capture_metadata(parsed_capture, content_type),
        })
        .await
    {
        Ok(artifact) => Ok(artifact),
        Err(error) => {
            let _ = tokio_fs::remove_file(resolve_artifact_file_path(&storage_path)).await;
            Err(biofield_db_error_response(
                "persist biofield capture artifact",
                &error,
            ))
        }
    }
}

async fn link_source_artifact_to_reading(
    biofield_repository: &BiofieldRepository,
    artifact: &BiofieldCaptureArtifact,
    reading_id: Uuid,
    user_id: Uuid,
) -> Result<BiofieldCaptureArtifact, Response> {
    match biofield_repository
        .link_artifact_to_reading(artifact.id, reading_id, user_id)
        .await
    {
        Ok(Some(linked_artifact)) => Ok(linked_artifact),
        Ok(None) => Err(json_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to link biofield artifact to reading",
            "BIOFIELD_DB_ERROR",
            Some(serde_json::json!({
                "artifact_id": artifact.id,
                "reading_id": reading_id,
            })),
        )),
        Err(error) => Err(biofield_db_error_response(
            "link biofield capture artifact to reading",
            &error,
        )),
    }
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

fn build_reprocess_response(
    source_reading_id: Uuid,
    reading_id: Uuid,
    session_id: Uuid,
    analysis_version: &str,
    metrics: Value,
    quality_assessment: Value,
    artifacts: Vec<BiofieldArtifactSummary>,
) -> BiofieldReprocessResponse {
    BiofieldReprocessResponse {
        source_reading_id: source_reading_id.to_string(),
        reading_id: reading_id.to_string(),
        session_id: session_id.to_string(),
        analysis_version: analysis_version.to_string(),
        metrics,
        quality_assessment,
        artifacts,
    }
}

fn extract_analysis_payload(sidecar_response: &Value) -> Result<(String, Value, Value), Response> {
    let analysis_version = match sidecar_response
        .get("analysis_version")
        .and_then(|value| value.as_str())
    {
        Some(analysis_version) => analysis_version.to_string(),
        None => {
            return Err(biofield_analysis_unavailable_response(Some(
                serde_json::json!({
                    "reason": "missing analysis_version in sidecar response",
                }),
            )))
        }
    };

    let metrics = match sidecar_response.get("metrics").cloned() {
        Some(metrics) => metrics,
        None => {
            return Err(biofield_analysis_unavailable_response(Some(
                serde_json::json!({
                    "reason": "missing metrics in sidecar response",
                }),
            )))
        }
    };

    let quality_assessment = match sidecar_response.get("quality_assessment").cloned() {
        Some(quality_assessment) => quality_assessment,
        None => {
            return Err(biofield_analysis_unavailable_response(Some(
                serde_json::json!({
                    "reason": "missing quality_assessment in sidecar response",
                }),
            )))
        }
    };

    let sufficient_quality = match quality_assessment
        .get("sufficient_quality")
        .and_then(|value| value.as_bool())
    {
        Some(sufficient_quality) => sufficient_quality,
        None => {
            return Err(biofield_analysis_unavailable_response(Some(
                serde_json::json!({
                    "reason": "missing sufficient_quality in sidecar response",
                }),
            )))
        }
    };

    if !sufficient_quality {
        return Err(capture_rejected_quality_response(
            "Biofield capture rejected for insufficient quality",
            quality_assessment,
            Some(&analysis_version),
        ));
    }

    Ok((analysis_version, metrics, quality_assessment))
}

fn reading_algorithms(reading: &Reading) -> Option<Vec<String>> {
    reading
        .input_data
        .get("algorithms")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
}

fn reading_options(reading: &Reading) -> Option<Value> {
    reading
        .input_data
        .get("options")
        .filter(|value| value.is_object())
        .cloned()
}

fn source_artifact_for_reading<'a>(
    artifacts: &'a [BiofieldCaptureArtifact],
) -> Option<&'a BiofieldCaptureArtifact> {
    artifacts
        .iter()
        .find(|artifact| artifact.artifact_kind == BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE)
        .or_else(|| artifacts.first())
}

fn parsed_capture_from_artifact(
    source_artifact: &BiofieldCaptureArtifact,
    source_reading: &Reading,
) -> ParsedBiofieldCapture {
    let file_name = source_artifact
        .capture_metadata
        .get("file_name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            source_reading
                .input_data
                .get("file_name")
                .and_then(Value::as_str)
                .map(str::to_string)
        });

    let capture_metadata = source_artifact
        .capture_metadata
        .get("capture_metadata")
        .filter(|value| value.is_object())
        .cloned();

    ParsedBiofieldCapture {
        image_bytes: None,
        image_content_type: Some(source_artifact.mime_type.clone()),
        image_file_name: file_name,
        algorithms: reading_algorithms(source_reading),
        options: reading_options(source_reading),
        capture_metadata,
    }
}

fn build_reprocess_input_data(
    source_reading_id: Uuid,
    session_id: Uuid,
    source_artifact: &BiofieldCaptureArtifact,
    parsed_capture: &ParsedBiofieldCapture,
) -> Value {
    serde_json::json!({
        "session_id": session_id,
        "file_name": parsed_capture.image_file_name,
        "content_type": source_artifact.mime_type,
        "algorithms": parsed_capture.algorithms,
        "options": parsed_capture.options,
        "capture_metadata": parsed_capture.capture_metadata,
        "reprocessed_from_reading_id": source_reading_id,
        "source_storage_path": source_artifact.storage_path,
    })
}

async fn save_reprocessed_reading(
    readings_repository: &ReadingsRepository,
    auth_user: &AuthUser,
    user_id: Uuid,
    session: &BiofieldSession,
    parsed_capture: &ParsedBiofieldCapture,
    source_reading_id: Uuid,
    source_artifact: &BiofieldCaptureArtifact,
    sidecar_response: &Value,
    image_bytes: &[u8],
) -> Result<Uuid, sqlx::Error> {
    let input_hash = create_input_hash(
        session.id,
        image_bytes,
        &source_artifact.mime_type,
        parsed_capture.algorithms.as_ref(),
        parsed_capture.options.as_ref(),
        parsed_capture.capture_metadata.as_ref(),
    );

    let processing_time_ms = sidecar_response
        .get("processing_time_ms")
        .and_then(|value| value.as_f64());

    let device_platform = parsed_capture
        .capture_metadata
        .as_ref()
        .and_then(|value| value.get("platform"))
        .and_then(|value| value.as_str())
        .map(str::to_string);

    readings_repository
        .save_reading(&NewReading {
            user_id,
            engine_id: BIOFIELD_ENGINE_ID.to_string(),
            workflow_id: None,
            input_hash,
            input_data: build_reprocess_input_data(
                source_reading_id,
                session.id,
                source_artifact,
                parsed_capture,
            ),
            result_data: sidecar_response.clone(),
            witness_prompt: None,
            consciousness_level: auth_user.consciousness_level as i16,
            calculation_time_ms: processing_time_ms,
            client_event_id: None,
            client_device_id: session.client_device_id.clone(),
            device_platform,
            device_app_version: session.viewer_version.clone(),
        })
        .await
}

fn normalize_baseline_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_baseline_notes(raw: Option<&str>) -> Option<String> {
    raw.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn dedupe_preserve_order(items: &[String]) -> Vec<String> {
    let mut values: Vec<String> = Vec::new();
    for item in items {
        if !values.iter().any(|existing| existing == item) {
            values.push(item.clone());
        }
    }
    values
}

fn baseline_summary_from_record(record: &BiofieldBaselineSummaryRecord) -> BiofieldBaselineSummary {
    BiofieldBaselineSummary {
        baseline_id: record.id.to_string(),
        name: record.name.clone(),
        notes: record.notes.clone(),
        reading_count: record.reading_count.max(0) as u32,
        created_at: record.created_at,
        updated_at: record.updated_at,
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

    let source_artifact = match create_source_artifact(
        biofield_repository,
        &session,
        &parsed_capture,
        &content_type,
        &image_bytes,
    )
    .await
    {
        Ok(artifact) => artifact,
        Err(response) => return response,
    };

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

    let (analysis_version, metrics, quality_assessment) =
        match extract_analysis_payload(&sidecar_response) {
            Ok(payload) => payload,
            Err(response) => return response,
        };

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

    let linked_artifact = match link_source_artifact_to_reading(
        biofield_repository,
        &source_artifact,
        reading_id,
        user_id,
    )
    .await
    {
        Ok(artifact) => artifact,
        Err(response) => return response,
    };

    let response = build_capture_response(
        reading_id,
        session.id,
        &analysis_version,
        metrics,
        quality_assessment,
        vec![artifact_to_summary(&linked_artifact)],
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
        (status = 503, description = "Biofield DB unavailable", body = crate::ErrorResponse),
        (status = 500, description = "Internal biofield persistence error", body = crate::ErrorResponse),
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
) -> Response {
    let Some(readings_repository) = state.readings_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };
    let Some(biofield_repository) = state.biofield_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };

    let user_id = match parse_auth_user_uuid(&auth_user) {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let (limit, offset) = normalize_readings_page(&query);
    let readings = match readings_repository
        .list_readings(
            user_id,
            Some(BIOFIELD_ENGINE_ID),
            limit as i64,
            offset as i64,
        )
        .await
    {
        Ok(readings) => readings,
        Err(error) => return biofield_db_error_response("list biofield readings", &error),
    };

    let mut items = Vec::with_capacity(readings.len());
    for reading in readings {
        let artifacts = match biofield_repository
            .list_reading_artifacts(reading.id, user_id)
            .await
        {
            Ok(artifacts) => artifacts,
            Err(error) => {
                return biofield_db_error_response("list biofield reading artifacts", &error)
            }
        };

        let summary = match build_reading_summary(&reading, &artifacts) {
            Ok(summary) => summary,
            Err(response) => return response,
        };
        items.push(summary);
    }

    (
        StatusCode::OK,
        Json(ListBiofieldReadingsResponse {
            items,
            limit,
            offset,
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
        (status = 422, description = "Invalid reading identifier", body = crate::ErrorResponse),
        (status = 503, description = "Biofield DB unavailable", body = crate::ErrorResponse),
        (status = 500, description = "Internal biofield persistence error", body = crate::ErrorResponse),
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
) -> Response {
    let Some(readings_repository) = state.readings_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };
    let Some(biofield_repository) = state.biofield_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };

    let user_id = match parse_auth_user_uuid(&auth_user) {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let reading_uuid = match parse_uuid_or_422(&reading_id, "reading_id") {
        Ok(reading_uuid) => reading_uuid,
        Err(response) => return response,
    };

    let reading = match readings_repository.get_reading(reading_uuid, user_id).await {
        Ok(Some(reading)) if reading.engine_id == BIOFIELD_ENGINE_ID => reading,
        Ok(Some(_)) | Ok(None) => return reading_not_found_response(&reading_id),
        Err(error) => return biofield_db_error_response("fetch biofield reading", &error),
    };

    let artifacts = match biofield_repository
        .list_reading_artifacts(reading_uuid, user_id)
        .await
    {
        Ok(artifacts) => artifacts,
        Err(error) => {
            return biofield_db_error_response("fetch biofield reading artifacts", &error)
        }
    };

    let response = match build_reading_detail_resource(&reading, &artifacts) {
        Ok(response) => response,
        Err(response) => return response,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// POST /api/v1/biofield/readings/:reading_id/reprocess
#[utoipa::path(
    post,
    path = "/api/v1/biofield/readings/{reading_id}/reprocess",
    tag = "biofield",
    params(
        ("reading_id" = String, Path, description = "Biofield reading identifier")
    ),
    request_body = ReprocessBiofieldReadingRequest,
    responses(
        (status = 201, description = "Biofield reading reprocessed", body = BiofieldReprocessResponse),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 404, description = "Reading or source artifact not found", body = crate::ErrorResponse),
        (status = 422, description = "Invalid reading identifier or rejected reprocess", body = crate::ErrorResponse),
        (status = 503, description = "Biofield dependencies unavailable", body = crate::ErrorResponse),
        (status = 500, description = "Internal biofield persistence or storage error", body = crate::ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn reprocess_reading(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(reading_id): Path<String>,
    Json(request): Json<ReprocessBiofieldReadingRequest>,
) -> Response {
    let Some(readings_repository) = state.readings_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };
    let Some(biofield_repository) = state.biofield_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };

    let user_id = match parse_auth_user_uuid(&auth_user) {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let reading_uuid = match parse_uuid_or_422(&reading_id, "reading_id") {
        Ok(reading_uuid) => reading_uuid,
        Err(response) => return response,
    };

    let source_reading = match readings_repository.get_reading(reading_uuid, user_id).await {
        Ok(Some(reading)) if reading.engine_id == BIOFIELD_ENGINE_ID => reading,
        Ok(Some(_)) | Ok(None) => return reading_not_found_response(&reading_id),
        Err(error) => return biofield_db_error_response("fetch biofield reading", &error),
    };

    let artifacts = match biofield_repository
        .list_reading_artifacts(reading_uuid, user_id)
        .await
    {
        Ok(artifacts) => artifacts,
        Err(error) => {
            return biofield_db_error_response("fetch biofield reading artifacts", &error)
        }
    };

    let Some(source_artifact) = source_artifact_for_reading(&artifacts) else {
        return artifact_not_found_response(&reading_id);
    };

    let image_bytes = match load_artifact_bytes(&source_artifact.storage_path).await {
        Ok(image_bytes) => image_bytes,
        Err(error) => {
            return biofield_storage_error_response(
                "load biofield source artifact bytes",
                &source_artifact.storage_path,
                &error,
            )
        }
    };

    let session = match biofield_repository
        .get_session(source_artifact.session_id, user_id)
        .await
    {
        Ok(Some(session)) => session,
        Ok(None) => return missing_reading_session_link_response(reading_uuid),
        Err(error) => return biofield_db_error_response("fetch biofield session", &error),
    };

    let mut parsed_capture = parsed_capture_from_artifact(source_artifact, &source_reading);
    if request.algorithms.is_some() {
        parsed_capture.algorithms = request.algorithms.clone();
    }
    if request.options.is_some() {
        parsed_capture.options = request.options.clone();
    }

    let sidecar_client = build_biofield_client_from_env();
    let sidecar_response = match sidecar_client
        .analyze_capture(BiofieldAnalyzeRequest {
            image_data: image_bytes.clone(),
            content_type: source_artifact.mime_type.clone(),
            algorithms: parsed_capture.algorithms.clone(),
            options: parsed_capture.options.clone(),
        })
        .await
    {
        Ok(response) => response,
        Err(error) => return map_bridge_error(error),
    };

    let (analysis_version, metrics, quality_assessment) =
        match extract_analysis_payload(&sidecar_response) {
            Ok(payload) => payload,
            Err(response) => return response,
        };

    let reprocessed_reading_id = match save_reprocessed_reading(
        readings_repository,
        &auth_user,
        user_id,
        &session,
        &parsed_capture,
        reading_uuid,
        source_artifact,
        &sidecar_response,
        &image_bytes,
    )
    .await
    {
        Ok(reading_id) => reading_id,
        Err(error) => {
            return biofield_db_error_response("persist reprocessed biofield reading", &error)
        }
    };

    let reprocessed_artifact = match create_source_artifact(
        biofield_repository,
        &session,
        &parsed_capture,
        &source_artifact.mime_type,
        &image_bytes,
    )
    .await
    {
        Ok(artifact) => artifact,
        Err(response) => return response,
    };

    let linked_artifact = match link_source_artifact_to_reading(
        biofield_repository,
        &reprocessed_artifact,
        reprocessed_reading_id,
        user_id,
    )
    .await
    {
        Ok(artifact) => artifact,
        Err(response) => return response,
    };

    (
        StatusCode::CREATED,
        Json(build_reprocess_response(
            reading_uuid,
            reprocessed_reading_id,
            session.id,
            &analysis_version,
            metrics,
            quality_assessment,
            vec![artifact_to_summary(&linked_artifact)],
        )),
    )
        .into_response()
}

/// GET /api/v1/biofield/baselines
#[utoipa::path(
    get,
    path = "/api/v1/biofield/baselines",
    tag = "biofield",
    responses(
        (status = 200, description = "Biofield baselines listed", body = ListBiofieldBaselinesResponse),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 503, description = "Biofield DB unavailable", body = crate::ErrorResponse),
        (status = 500, description = "Internal biofield persistence error", body = crate::ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn list_baselines(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Response {
    let Some(biofield_repository) = state.biofield_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };

    let user_id = match parse_auth_user_uuid(&auth_user) {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let records = match biofield_repository.list_baselines(user_id).await {
        Ok(records) => records,
        Err(error) => return biofield_db_error_response("list biofield baselines", &error),
    };

    (
        StatusCode::OK,
        Json(ListBiofieldBaselinesResponse {
            items: records
                .iter()
                .map(baseline_summary_from_record)
                .collect::<Vec<_>>(),
        }),
    )
        .into_response()
}

/// POST /api/v1/biofield/baselines
#[utoipa::path(
    post,
    path = "/api/v1/biofield/baselines",
    tag = "biofield",
    request_body = CreateBiofieldBaselineRequest,
    responses(
        (status = 201, description = "Biofield baseline created", body = BiofieldBaselineSummary),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 404, description = "Reading not found", body = crate::ErrorResponse),
        (status = 422, description = "Invalid baseline request", body = crate::ErrorResponse),
        (status = 503, description = "Biofield DB unavailable", body = crate::ErrorResponse),
        (status = 500, description = "Internal biofield persistence error", body = crate::ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn create_baseline(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateBiofieldBaselineRequest>,
) -> Response {
    let Some(readings_repository) = state.readings_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };
    let Some(biofield_repository) = state.biofield_repository.as_ref() else {
        return biofield_db_unavailable_response();
    };

    let user_id = match parse_auth_user_uuid(&auth_user) {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let Some(name) = normalize_baseline_name(&request.name) else {
        return baseline_invalid_response("Baseline name is required", None);
    };

    let reading_id_values = dedupe_preserve_order(&request.reading_ids);
    if reading_id_values.is_empty() {
        return baseline_invalid_response(
            "Baseline must include at least one biofield reading",
            None,
        );
    }

    let mut reading_uuids = Vec::with_capacity(reading_id_values.len());
    for reading_id in &reading_id_values {
        let reading_uuid = match parse_uuid_or_422(reading_id, "reading_ids") {
            Ok(reading_uuid) => reading_uuid,
            Err(response) => return response,
        };

        let reading = match readings_repository.get_reading(reading_uuid, user_id).await {
            Ok(Some(reading)) if reading.engine_id == BIOFIELD_ENGINE_ID => reading,
            Ok(Some(_)) | Ok(None) => return reading_not_found_response(reading_id),
            Err(error) => return biofield_db_error_response("fetch baseline reading", &error),
        };
        if reading.engine_id != BIOFIELD_ENGINE_ID {
            return reading_not_found_response(reading_id);
        }
        reading_uuids.push(reading_uuid);
    }

    let baseline = match biofield_repository
        .create_baseline(
            &NewBiofieldBaseline {
                user_id,
                name,
                notes: normalize_baseline_notes(request.notes.as_deref()),
            },
            &reading_uuids,
        )
        .await
    {
        Ok(baseline) => baseline,
        Err(error) => return biofield_db_error_response("create biofield baseline", &error),
    };

    (
        StatusCode::CREATED,
        Json(BiofieldBaselineSummary {
            baseline_id: baseline.id.to_string(),
            name: baseline.name,
            notes: baseline.notes,
            reading_count: reading_uuids.len() as u32,
            created_at: baseline.created_at,
            updated_at: baseline.updated_at,
        }),
    )
        .into_response()
}
