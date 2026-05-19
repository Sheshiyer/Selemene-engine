//! Raga clip endpoints — SUNO-03 + SUNO-05.
//!
//! Public (auth-gated): `GET /api/v1/raga/:num/clip`
//! Internal (service-key): `POST /internal/raga/clip`

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AppState;
const INTERNAL_KEY_HEADER: &str = "x-internal-key";

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct RagaClipRow {
    pub melakarta_num: i32,
    pub style: String,
    pub suno_song_id: String,
    pub cdn_url: String,
    pub duration_sec: Option<f32>,
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RagaClipResponse {
    pub melakarta_num: i32,
    pub style: String,
    pub suno_song_id: String,
    pub cdn_url: String,
    pub duration_sec: Option<f64>,
    pub status: String,
}

// ---------------------------------------------------------------------------
// GET /api/v1/raga/:num/clip — SUNO-03
// ---------------------------------------------------------------------------

/// Return the approved Suno-generated clip for a melakarta raga.
///
/// `num` is 1-72 (the Carnatic melakarta index).
/// Query parameter `style` defaults to `ambient`; valid values: `ambient`, `traditional`.
/// Returns 404 when no approved clip exists yet (UI shows Strudel fallback).
#[utoipa::path(
    get,
    path = "/api/v1/raga/{num}/clip",
    params(
        ("num" = i32, Path, description = "Melakarta raga number (1-72)"),
        ("style" = Option<String>, Query, description = "Clip style: ambient | traditional"),
    ),
    responses(
        (status = 200, description = "Approved clip metadata", body = RagaClipResponse),
        (status = 404, description = "No approved clip for this raga yet"),
        (status = 422, description = "Invalid melakarta number"),
    )
)]
pub async fn get_raga_clip(
    State(state): State<AppState>,
    Path(num): Path<i32>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    if !(1..=72).contains(&num) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "melakarta_num must be 1-72" })),
        )
            .into_response();
    }
    let style = params
        .get("style")
        .cloned()
        .unwrap_or_else(|| "ambient".to_string());

    let pool = match state.auth.pool() {
        Some(p) => p,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "database unavailable" })),
            )
                .into_response()
        }
    };

    let row: Result<Option<RagaClipRow>, _> = sqlx::query_as(
        r#"
        SELECT melakarta_num, style, suno_song_id, cdn_url, duration_sec, status
          FROM raga_clips
         WHERE melakarta_num = $1 AND style = $2 AND status = 'approved'
         LIMIT 1
        "#,
    )
    .bind(num)
    .bind(&style)
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some(r)) => (
            StatusCode::OK,
            Json(serde_json::json!(RagaClipResponse {
                melakarta_num: r.melakarta_num,
                style: r.style,
                suno_song_id: r.suno_song_id,
                cdn_url: r.cdn_url,
                duration_sec: r.duration_sec.map(|d| d as f64),
                status: r.status,
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "no approved clip for this raga" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(melakarta_num = num, style = %style, error = %e, "raga_clips query failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "database error" })),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// POST /internal/raga/clip — SUNO-05
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertRagaClipRequest {
    pub melakarta_num: i32,
    pub style: String,
    pub suno_song_id: String,
    pub cdn_url: String,
    pub duration_sec: Option<f64>,
    /// status: "pending" | "approved" | "rejected"
    pub status: Option<String>,
}

/// Internal upsert — used by bulk-gen script running in ts-engines to persist
/// generated clips. Protected by `INTERNAL_SERVICE_KEY` shared secret.
#[utoipa::path(
    post,
    path = "/internal/raga/clip",
    request_body = UpsertRagaClipRequest,
    responses(
        (status = 201, description = "Clip upserted"),
        (status = 401, description = "Missing or invalid service key"),
        (status = 422, description = "Invalid request"),
    )
)]
pub async fn upsert_raga_clip(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpsertRagaClipRequest>,
) -> impl IntoResponse {
    if !internal_key_ok(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "forbidden" })),
        )
            .into_response();
    }
    if !(1..=72).contains(&body.melakarta_num) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "melakarta_num must be 1-72" })),
        )
            .into_response();
    }

    let status = body.status.as_deref().unwrap_or("pending");
    let duration = body.duration_sec.map(|d| d as f32);

    let pool = match state.auth.pool() {
        Some(p) => p,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "database unavailable" })),
            )
                .into_response()
        }
    };

    let result = sqlx::query(
        r#"
        INSERT INTO raga_clips (melakarta_num, style, suno_song_id, cdn_url, duration_sec, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (melakarta_num, style)
        DO UPDATE SET
          suno_song_id = EXCLUDED.suno_song_id,
          cdn_url      = EXCLUDED.cdn_url,
          duration_sec = EXCLUDED.duration_sec,
          status       = EXCLUDED.status,
          updated_at   = now()
        "#,
    )
    .bind(body.melakarta_num)
    .bind(&body.style)
    .bind(&body.suno_song_id)
    .bind(&body.cdn_url)
    .bind(duration)
    .bind(status)
    .execute(pool)
    .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({ "ok": true }))).into_response(),
        Err(e) => {
            tracing::error!(
                melakarta_num = body.melakarta_num,
                suno_song_id = %body.suno_song_id,
                error = %e,
                "raga_clips upsert failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "database error" })),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

fn internal_key_ok(headers: &HeaderMap) -> bool {
    let expected = std::env::var("INTERNAL_SERVICE_KEY").unwrap_or_default();
    if expected.is_empty() {
        tracing::warn!("INTERNAL_SERVICE_KEY not set — rejecting all /internal/raga/clip calls");
        return false;
    }
    headers
        .get(INTERNAL_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|v| v == expected)
        .unwrap_or(false)
}
