use crate::{error::ApiError, AppState};
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use noesis_auth::{sha256_hex, ApiKey, AuthUser};
use noesis_core::EngineError;
use noesis_data::repositories::admin_repository::{
    AdminApiKeyRecord, AdminRepository, AdminUserRecord,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct AdminSessionResponse {
    pub user_id: String,
    pub email: String,
    pub tier: String,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
    pub has_admin_access: bool,
}

#[derive(Serialize, ToSchema)]
pub struct AdminUsersResponse {
    pub items: Vec<AdminUserItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminUserItem {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub tier: String,
    pub consciousness_level: i32,
    pub experience_points: i32,
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub state: String,
    pub active_key_count: i64,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserStateRequest {
    pub state: String,
    pub lock_minutes: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct UpdateUserStateResponse {
    pub user_id: String,
    pub state: String,
    pub locked_until: Option<DateTime<Utc>>,
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserTierRequest {
    pub tier: String,
}

#[derive(Serialize, ToSchema)]
pub struct UpdateUserTierResponse {
    pub user_id: String,
    pub tier: String,
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRolesRequest {
    pub roles: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct UpdateUserRolesResponse {
    pub user_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub affected_api_keys: i64,
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct AdminApiKeysResponse {
    pub items: Vec<AdminApiKeyItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminApiKeyItem {
    pub id: String,
    pub name: Option<String>,
    pub key_prefix: Option<String>,
    pub user_id: String,
    pub user_email: String,
    pub tier: String,
    pub permissions: Vec<String>,
    pub consciousness_level: i32,
    pub rate_limit: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub user_id: String,
    pub name: Option<String>,
    pub tier: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub consciousness_level: Option<i32>,
    pub rate_limit: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct CreateApiKeyResponse {
    pub key: AdminApiKeyItem,
    pub secret_key: String,
}

#[derive(Serialize, ToSchema)]
pub struct RotateApiKeyResponse {
    pub old_key_id: String,
    pub key: AdminApiKeyItem,
    pub secret_key: String,
}

#[derive(Serialize, ToSchema)]
pub struct AdminHistorySyncUsersResponse {
    pub items: Vec<AdminHistorySyncUserItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminHistorySyncUserItem {
    pub user_id: String,
    pub email: String,
    pub readings_count: i64,
    pub usage_events_count: i64,
    pub drift_count: i64,
    pub status: String,
    pub last_reading_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminHistorySyncDevicesResponse {
    pub items: Vec<AdminHistorySyncDeviceItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminHistorySyncDeviceItem {
    pub device_id: String,
    pub user_id: String,
    pub user_email: String,
    pub tier: String,
    pub status: String,
    pub permission_count: i32,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminHistorySyncEventsResponse {
    pub items: Vec<AdminHistorySyncEventItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminHistorySyncEventItem {
    pub event_id: String,
    pub user_id: String,
    pub user_email: String,
    pub engine_id: Option<String>,
    pub workflow_id: Option<String>,
    pub status: String,
    pub duration_ms: i32,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAnalyticsSummaryResponse {
    pub window_hours: i64,
    pub requests_total: i64,
    pub success_total: i64,
    pub failure_total: i64,
    pub error_rate_pct: f64,
    pub p95_duration_ms: f64,
    pub avg_duration_ms: f64,
    pub active_users: i64,
    pub unique_keys: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAnalyticsTimeseriesResponse {
    pub window_hours: i64,
    pub bucket: String,
    pub points: Vec<AdminAnalyticsTimeseriesPoint>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAnalyticsTimeseriesPoint {
    pub bucket_start: DateTime<Utc>,
    pub request_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub avg_duration_ms: f64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAnalyticsBreakdownResponse {
    pub window_hours: i64,
    pub engines: Vec<AdminAnalyticsBreakdownEntry>,
    pub workflows: Vec<AdminAnalyticsBreakdownEntry>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAnalyticsBreakdownEntry {
    pub label: String,
    pub request_count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAnalyticsTopConsumersResponse {
    pub window_hours: i64,
    pub items: Vec<AdminAnalyticsTopConsumerItem>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAnalyticsTopConsumerItem {
    pub user_id: String,
    pub user_email: String,
    pub request_count: i64,
    pub failure_count: i64,
    pub avg_duration_ms: f64,
}

#[derive(Deserialize, Default)]
pub struct ListUsersQuery {
    pub query: Option<String>,
    pub tier: Option<String>,
    pub state: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct ListApiKeysQuery {
    pub query: Option<String>,
    pub user_id: Option<String>,
    pub active_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct HistoryEventsQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct AnalyticsQuery {
    pub window_hours: Option<i64>,
    pub bucket: Option<String>,
    pub limit: Option<i64>,
}

fn has_permission(permissions: &[String], required: &str) -> bool {
    if permissions
        .iter()
        .any(|perm| perm == required || perm == "admin:*")
    {
        return true;
    }

    if required.starts_with("admin:users:") && permissions.iter().any(|perm| perm == "admin:users")
    {
        return true;
    }

    if (required.starts_with("admin:analytics:")
        || required.starts_with("admin:system:")
        || required.starts_with("admin:audit:"))
        && permissions.iter().any(|perm| perm == "admin:analytics")
    {
        return true;
    }

    if (required.starts_with("admin:keys:") || required.starts_with("admin:history-sync:"))
        && permissions.iter().any(|perm| perm == "admin:users")
    {
        return true;
    }

    false
}

fn has_admin_access(permissions: &[String]) -> bool {
    permissions.iter().any(|p| p.starts_with("admin:"))
        || permissions
            .iter()
            .any(|p| p == "admin:users" || p == "admin:analytics")
}

fn derive_roles(permissions: &[String]) -> Vec<String> {
    let mut roles: BTreeSet<String> = BTreeSet::new();
    let has = |perm: &str| has_permission(permissions, perm);

    let viewer_signals = [
        "admin:analytics:read",
        "admin:system:read",
        "admin:audit:list",
        "admin:audit:read",
        "admin:analytics",
    ];
    if viewer_signals.iter().any(|perm| has(perm)) {
        roles.insert("viewer".to_string());
    }

    let support_signals = [
        "admin:users:list",
        "admin:users:read",
        "admin:users:suspend",
        "admin:history-sync:read",
        "admin:users",
    ];
    if support_signals.iter().any(|perm| has(perm)) {
        roles.insert("support".to_string());
    }

    let admin_signals = [
        "admin:keys:list",
        "admin:keys:create",
        "admin:keys:revoke",
        "admin:keys:rotate",
        "admin:users:tier:update",
        "admin:history-sync:retry",
    ];
    if admin_signals.iter().any(|perm| has(perm)) {
        roles.insert("admin".to_string());
    }

    if has("admin:users:roles:update") {
        roles.insert("platform-admin".to_string());
    }

    if roles.is_empty() && has_admin_access(permissions) {
        roles.insert("viewer".to_string());
    }

    roles.into_iter().collect()
}

fn permissions_for_roles(roles: &[String]) -> Vec<String> {
    let mut permissions: BTreeSet<String> = BTreeSet::new();
    permissions.insert("basic:access".to_string());

    for role in roles {
        match role.as_str() {
            "viewer" => {
                permissions.insert("admin:analytics:read".to_string());
                permissions.insert("admin:system:read".to_string());
                permissions.insert("admin:audit:list".to_string());
            }
            "support" => {
                permissions.insert("admin:analytics:read".to_string());
                permissions.insert("admin:users:list".to_string());
                permissions.insert("admin:users:read".to_string());
                permissions.insert("admin:history-sync:read".to_string());
            }
            "admin" => {
                permissions.insert("admin:analytics:read".to_string());
                permissions.insert("admin:users:list".to_string());
                permissions.insert("admin:users:read".to_string());
                permissions.insert("admin:history-sync:read".to_string());
                permissions.insert("admin:keys:list".to_string());
                permissions.insert("admin:keys:create".to_string());
                permissions.insert("admin:keys:revoke".to_string());
                permissions.insert("admin:keys:rotate".to_string());
                permissions.insert("admin:users:tier:update".to_string());
            }
            "platform-admin" => {
                permissions.insert("admin:*".to_string());
                permissions.insert("admin:analytics:read".to_string());
                permissions.insert("admin:users:list".to_string());
                permissions.insert("admin:users:read".to_string());
                permissions.insert("admin:users:roles:update".to_string());
                permissions.insert("admin:history-sync:read".to_string());
                permissions.insert("admin:keys:list".to_string());
                permissions.insert("admin:keys:create".to_string());
                permissions.insert("admin:keys:revoke".to_string());
                permissions.insert("admin:keys:rotate".to_string());
                permissions.insert("admin:users:tier:update".to_string());
            }
            _ => {}
        }
    }

    permissions.into_iter().collect()
}

fn normalize_effective_permissions(permissions: &[String]) -> Vec<String> {
    let mut normalized: BTreeSet<String> = permissions.iter().cloned().collect();

    let canonical_permissions = [
        "admin:analytics:read",
        "admin:system:read",
        "admin:audit:list",
        "admin:users:list",
        "admin:users:read",
        "admin:users:suspend",
        "admin:users:tier:update",
        "admin:users:roles:update",
        "admin:keys:list",
        "admin:keys:create",
        "admin:keys:revoke",
        "admin:keys:rotate",
        "admin:history-sync:read",
        "admin:history-sync:retry",
    ];

    for required in canonical_permissions {
        if has_permission(permissions, required) {
            normalized.insert(required.to_string());
        }
    }

    normalized.into_iter().collect()
}

fn parse_permissions(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn normalize_limit_offset(
    limit: Option<i64>,
    offset: Option<i64>,
    default_limit: i64,
    max_limit: i64,
) -> (i64, i64) {
    let safe_limit = limit.unwrap_or(default_limit).clamp(1, max_limit);
    let safe_offset = offset.unwrap_or(0).max(0);
    (safe_limit, safe_offset)
}

fn user_state(locked_until: Option<DateTime<Utc>>) -> String {
    if locked_until.is_some_and(|ts| ts > Utc::now()) {
        "locked".to_string()
    } else {
        "active".to_string()
    }
}

fn default_rate_limit_for_tier(tier: &str) -> i32 {
    match tier.to_ascii_lowercase().as_str() {
        "free" => 60,
        "premium" => 1_000,
        "enterprise" => 10_000,
        _ => 100,
    }
}

fn generate_secret_api_key() -> String {
    format!("nk_{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn json_error_response(
    status: StatusCode,
    error: impl Into<String>,
    error_code: &str,
    details: Option<Value>,
) -> Response {
    (
        status,
        Json(serde_json::json!({
            "error": error.into(),
            "error_code": error_code,
            "details": details,
        })),
    )
        .into_response()
}

fn service_unavailable_response() -> Response {
    json_error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Admin APIs require a configured database connection",
        "ADMIN_DB_UNAVAILABLE",
        None,
    )
}

fn forbidden_response(required_permission: &str) -> Response {
    json_error_response(
        StatusCode::FORBIDDEN,
        format!("Missing required permission: {required_permission}"),
        "FORBIDDEN",
        Some(serde_json::json!({
            "required_permission": required_permission,
        })),
    )
}

#[allow(clippy::result_large_err)]
fn parse_uuid_or_422(id: &str, field: &str) -> Result<Uuid, Response> {
    Uuid::parse_str(id).map_err(|_| {
        json_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Invalid UUID for {field}"),
            "INVALID_INPUT",
            Some(serde_json::json!({ field: id })),
        )
    })
}

async fn effective_permissions(
    state: &AppState,
    auth_user: &AuthUser,
) -> Result<Vec<String>, ApiError> {
    let mut permissions: BTreeSet<String> = auth_user.permissions.iter().cloned().collect();

    if let Some(repo) = state.admin_repository.as_ref() {
        if let Ok(user_id) = Uuid::parse_str(&auth_user.user_id) {
            let repo_permissions = repo.get_effective_permissions(user_id).await.map_err(|e| {
                EngineError::InternalError(format!("Failed to resolve permissions: {e}"))
            })?;
            permissions.extend(repo_permissions);

            let admin_roles = repo.get_admin_roles(user_id).await.map_err(|e| {
                EngineError::InternalError(format!("Failed to resolve admin roles: {e}"))
            })?;
            if !admin_roles.is_empty() {
                let role_permissions = permissions_for_roles(&admin_roles);
                permissions.extend(role_permissions);
            }
        }
    }

    let collected = permissions.into_iter().collect::<Vec<_>>();
    Ok(normalize_effective_permissions(&collected))
}

fn map_user_record(record: AdminUserRecord) -> AdminUserItem {
    AdminUserItem {
        id: record.id.to_string(),
        email: record.email,
        full_name: record.full_name,
        tier: record.tier,
        consciousness_level: record.consciousness_level,
        experience_points: record.experience_points,
        last_login_at: record.last_login_at,
        failed_login_attempts: record.failed_login_attempts,
        locked_until: record.locked_until,
        state: user_state(record.locked_until),
        active_key_count: record.active_key_count,
        permissions: parse_permissions(&record.permissions),
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn map_api_key_record(record: AdminApiKeyRecord) -> AdminApiKeyItem {
    AdminApiKeyItem {
        id: record.id.to_string(),
        name: record.name,
        key_prefix: record.key_prefix,
        user_id: record.user_id.to_string(),
        user_email: record.user_email,
        tier: record.tier,
        permissions: parse_permissions(&record.permissions),
        consciousness_level: record.consciousness_level,
        rate_limit: record.rate_limit,
        created_at: record.created_at,
        expires_at: record.expires_at,
        last_used: record.last_used,
        is_active: record.is_active,
    }
}

fn require_permission_or_forbidden(permissions: &[String], required: &str) -> Option<Response> {
    if has_permission(permissions, required) {
        None
    } else {
        Some(forbidden_response(required))
    }
}

#[allow(clippy::result_large_err)]
fn admin_repo_or_503(state: &AppState) -> Result<&AdminRepository, Response> {
    state
        .admin_repository
        .as_deref()
        .ok_or_else(service_unavailable_response)
}

/// GET /api/v1/admin/session -- return current authenticated admin session shape
#[utoipa::path(
    get,
    path = "/api/v1/admin/session",
    tag = "admin",
    responses(
        (status = 200, description = "Admin session data", body = AdminSessionResponse),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn get_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let user_uuid = Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| EngineError::AuthError("Invalid user ID in token".to_string()))?;

    let user = state
        .user_repository
        .get_user_by_id(user_uuid)
        .await
        .map_err(|e| EngineError::InternalError(format!("Database error: {e}")))?
        .ok_or_else(|| EngineError::AuthError("User not found".to_string()))?;

    let effective_permissions = effective_permissions(&state, &auth_user).await?;

    let response = AdminSessionResponse {
        user_id: user.id.to_string(),
        email: user.email,
        tier: user.tier,
        roles: derive_roles(&effective_permissions),
        has_admin_access: has_admin_access(&effective_permissions),
        permissions: effective_permissions,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// GET /api/v1/admin/users -- list users with account/admin metadata
#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    tag = "admin",
    params(
        ("query" = Option<String>, Query, description = "Search by email or name"),
        ("tier" = Option<String>, Query, description = "Filter by tier"),
        ("state" = Option<String>, Query, description = "Filter by account state (active|locked)"),
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "Admin users list", body = AdminUsersResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 503, description = "Database unavailable", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn list_users(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:users:list")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 50, 200);

    let items = repo
        .list_users(
            query.query.as_deref(),
            query.tier.as_deref(),
            query.state.as_deref(),
            limit,
            offset,
        )
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to list users: {e}")))?
        .into_iter()
        .map(map_user_record)
        .collect::<Vec<_>>();

    let total = repo
        .count_users(
            query.query.as_deref(),
            query.tier.as_deref(),
            query.state.as_deref(),
        )
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to count users: {e}")))?;

    Ok((
        StatusCode::OK,
        Json(AdminUsersResponse {
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// PATCH /api/v1/admin/users/{user_id}/state -- lock/unlock user account
#[utoipa::path(
    patch,
    path = "/api/v1/admin/users/{user_id}/state",
    tag = "admin",
    params(("user_id" = String, Path, description = "User UUID")),
    request_body = UpdateUserStateRequest,
    responses(
        (status = 200, description = "User state updated", body = UpdateUserStateResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 404, description = "User not found", body = crate::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn update_user_state(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserStateRequest>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:users:suspend")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let user_uuid = match parse_uuid_or_422(&user_id, "user_id") {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    let target_state = payload.state.trim().to_ascii_lowercase();
    let locked_until = match target_state.as_str() {
        "active" => None,
        "locked" => {
            let lock_minutes = payload.lock_minutes.unwrap_or(60).clamp(5, 43_200);
            Some(Utc::now() + Duration::minutes(lock_minutes))
        }
        _ => {
            return Ok(json_error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "State must be either 'active' or 'locked'",
                "VALIDATION_ERROR",
                Some(serde_json::json!({ "state": payload.state })),
            ));
        }
    };

    let updated = repo
        .set_user_lock_state(user_uuid, locked_until)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to update user state: {e}")))?;

    let Some((updated_user_id, updated_locked_until)) = updated else {
        return Ok(json_error_response(
            StatusCode::NOT_FOUND,
            "User not found",
            "NOT_FOUND",
            Some(serde_json::json!({ "user_id": user_id })),
        ));
    };

    Ok((
        StatusCode::OK,
        Json(UpdateUserStateResponse {
            user_id: updated_user_id.to_string(),
            state: user_state(updated_locked_until),
            locked_until: updated_locked_until,
            message: "User state updated".to_string(),
        }),
    )
        .into_response())
}

/// PATCH /api/v1/admin/users/{user_id}/tier -- update user tier and linked key tiers
#[utoipa::path(
    patch,
    path = "/api/v1/admin/users/{user_id}/tier",
    tag = "admin",
    params(("user_id" = String, Path, description = "User UUID")),
    request_body = UpdateUserTierRequest,
    responses(
        (status = 200, description = "User tier updated", body = UpdateUserTierResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 404, description = "User not found", body = crate::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn update_user_tier(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserTierRequest>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:users:tier:update")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let user_uuid = match parse_uuid_or_422(&user_id, "user_id") {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    let tier = payload.tier.trim().to_ascii_lowercase();
    if tier.is_empty() {
        return Ok(json_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Tier cannot be empty",
            "VALIDATION_ERROR",
            None,
        ));
    }

    let updated = repo
        .set_user_tier(user_uuid, &tier)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to update user tier: {e}")))?;

    let Some((updated_user_id, updated_tier)) = updated else {
        return Ok(json_error_response(
            StatusCode::NOT_FOUND,
            "User not found",
            "NOT_FOUND",
            Some(serde_json::json!({ "user_id": user_id })),
        ));
    };

    Ok((
        StatusCode::OK,
        Json(UpdateUserTierResponse {
            user_id: updated_user_id.to_string(),
            tier: updated_tier,
            message: "User tier updated".to_string(),
        }),
    )
        .into_response())
}

/// PUT /api/v1/admin/users/{user_id}/roles -- assign role set and persist derived permissions
#[utoipa::path(
    put,
    path = "/api/v1/admin/users/{user_id}/roles",
    tag = "admin",
    params(("user_id" = String, Path, description = "User UUID")),
    request_body = UpdateUserRolesRequest,
    responses(
        (status = 200, description = "User roles updated", body = UpdateUserRolesResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 404, description = "User not found", body = crate::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn update_user_roles(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserRolesRequest>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:users:roles:update")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let user_uuid = match parse_uuid_or_422(&user_id, "user_id") {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    if payload.roles.is_empty() {
        return Ok(json_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "At least one role must be provided",
            "VALIDATION_ERROR",
            None,
        ));
    }

    let normalized_roles = payload
        .roles
        .iter()
        .map(|role| role.trim().to_ascii_lowercase())
        .filter(|role| !role.is_empty())
        .collect::<Vec<_>>();

    let allowed_roles = ["viewer", "support", "admin", "platform-admin"];
    for role in &normalized_roles {
        if !allowed_roles.contains(&role.as_str()) {
            return Ok(json_error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Unsupported role: {role}"),
                "VALIDATION_ERROR",
                Some(serde_json::json!({ "allowed_roles": allowed_roles })),
            ));
        }
    }

    let derived_permissions = permissions_for_roles(&normalized_roles);

    let affected_api_keys = repo
        .set_user_roles_and_permissions(user_uuid, &normalized_roles, &derived_permissions)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to update user roles: {e}")))?;

    let Some(affected_api_keys) = affected_api_keys else {
        return Ok(json_error_response(
            StatusCode::NOT_FOUND,
            "User not found",
            "NOT_FOUND",
            Some(serde_json::json!({ "user_id": user_id })),
        ));
    };

    Ok((
        StatusCode::OK,
        Json(UpdateUserRolesResponse {
            user_id: user_uuid.to_string(),
            roles: normalized_roles,
            permissions: derived_permissions,
            affected_api_keys,
            message: "User roles and permissions updated".to_string(),
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/api-keys -- list API key records
#[utoipa::path(
    get,
    path = "/api/v1/admin/api-keys",
    tag = "admin",
    params(
        ("query" = Option<String>, Query, description = "Search by key/user/email"),
        ("user_id" = Option<String>, Query, description = "Filter by user UUID"),
        ("active_only" = Option<bool>, Query, description = "Only active keys"),
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "API keys list", body = AdminApiKeysResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn list_api_keys(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListApiKeysQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:keys:list") {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 50, 200);

    let user_filter = match query.user_id.as_deref() {
        Some(uid) => match parse_uuid_or_422(uid, "user_id") {
            Ok(parsed) => Some(parsed),
            Err(resp) => return Ok(resp),
        },
        None => None,
    };

    let active_only = query.active_only.unwrap_or(false);

    let items = repo
        .list_api_keys(
            query.query.as_deref(),
            user_filter,
            active_only,
            limit,
            offset,
        )
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to list api keys: {e}")))?
        .into_iter()
        .map(map_api_key_record)
        .collect::<Vec<_>>();

    let total = repo
        .count_api_keys(query.query.as_deref(), user_filter, active_only)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to count api keys: {e}")))?;

    Ok((
        StatusCode::OK,
        Json(AdminApiKeysResponse {
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// POST /api/v1/admin/api-keys -- create a new API key and return one-time secret
#[utoipa::path(
    post,
    path = "/api/v1/admin/api-keys",
    tag = "admin",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, description = "API key created", body = CreateApiKeyResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 404, description = "User not found", body = crate::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn create_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:keys:create")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let user_uuid = match parse_uuid_or_422(&payload.user_id, "user_id") {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    if !repo
        .user_exists(user_uuid)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed checking user existence: {e}")))?
    {
        return Ok(json_error_response(
            StatusCode::NOT_FOUND,
            "User not found",
            "NOT_FOUND",
            Some(serde_json::json!({ "user_id": payload.user_id })),
        ));
    }

    let tier = if let Some(tier) = payload
        .tier
        .as_deref()
        .map(|t| t.trim().to_ascii_lowercase())
        .filter(|t| !t.is_empty())
    {
        tier
    } else if let Some(existing_tier) = repo
        .get_user_tier(user_uuid)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to read user tier: {e}")))?
    {
        existing_tier.to_ascii_lowercase()
    } else {
        "free".to_string()
    };

    let permissions = payload
        .permissions
        .unwrap_or_else(|| vec!["basic:access".to_string()]);

    let rate_limit = payload
        .rate_limit
        .unwrap_or_else(|| default_rate_limit_for_tier(&tier));

    let consciousness_level = payload.consciousness_level.unwrap_or(0).clamp(0, 5);

    let secret_key = generate_secret_api_key();
    let key_hash = sha256_hex(&secret_key);
    let key_prefix = secret_key[..12.min(secret_key.len())].to_string();

    let created = repo
        .create_api_key(
            noesis_data::repositories::admin_repository::NewApiKeyRecord {
                key_hash,
                name: payload.name,
                key_prefix,
                user_id: user_uuid,
                tier: tier.clone(),
                permissions: serde_json::json!(permissions),
                consciousness_level,
                rate_limit,
                expires_at: payload.expires_at,
            },
        )
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to create api key: {e}")))?;

    let _ = state
        .auth
        .add_api_key(ApiKey {
            key: secret_key.clone(),
            user_id: created.user_id.to_string(),
            tier,
            permissions: parse_permissions(&created.permissions),
            created_at: created.created_at,
            expires_at: created.expires_at,
            last_used: created.last_used,
            rate_limit: created.rate_limit as u32,
            consciousness_level: created.consciousness_level as u8,
        })
        .await;

    Ok((
        StatusCode::CREATED,
        Json(CreateApiKeyResponse {
            key: map_api_key_record(created),
            secret_key,
        }),
    )
        .into_response())
}

/// POST /api/v1/admin/api-keys/{key_id}/revoke -- revoke an API key
#[utoipa::path(
    post,
    path = "/api/v1/admin/api-keys/{key_id}/revoke",
    tag = "admin",
    params(("key_id" = String, Path, description = "API key UUID")),
    responses(
        (status = 200, description = "API key revoked"),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 404, description = "Key not found", body = crate::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn revoke_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(key_id): Path<String>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:keys:revoke")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let key_uuid = match parse_uuid_or_422(&key_id, "key_id") {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    let revoked = repo
        .revoke_api_key(key_uuid)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to revoke api key: {e}")))?;

    if !revoked {
        return Ok(json_error_response(
            StatusCode::NOT_FOUND,
            "API key not found",
            "NOT_FOUND",
            Some(serde_json::json!({ "key_id": key_id })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "API key revoked",
            "key_id": key_id,
        })),
    )
        .into_response())
}

/// POST /api/v1/admin/api-keys/{key_id}/rotate -- rotate an API key
#[utoipa::path(
    post,
    path = "/api/v1/admin/api-keys/{key_id}/rotate",
    tag = "admin",
    params(("key_id" = String, Path, description = "API key UUID")),
    responses(
        (status = 200, description = "API key rotated", body = RotateApiKeyResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 404, description = "Key not found/active", body = crate::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn rotate_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(key_id): Path<String>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:keys:rotate")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let key_uuid = match parse_uuid_or_422(&key_id, "key_id") {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    let secret_key = generate_secret_api_key();
    let key_hash = sha256_hex(&secret_key);
    let key_prefix = secret_key[..12.min(secret_key.len())].to_string();

    let rotated = repo
        .rotate_api_key(key_uuid, &key_hash, &key_prefix)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to rotate api key: {e}")))?;

    let Some(rotated) = rotated else {
        return Ok(json_error_response(
            StatusCode::NOT_FOUND,
            "Active API key not found",
            "NOT_FOUND",
            Some(serde_json::json!({ "key_id": key_id })),
        ));
    };

    let _ = state
        .auth
        .add_api_key(ApiKey {
            key: secret_key.clone(),
            user_id: rotated.user_id.to_string(),
            tier: rotated.tier.clone(),
            permissions: parse_permissions(&rotated.permissions),
            created_at: rotated.created_at,
            expires_at: rotated.expires_at,
            last_used: rotated.last_used,
            rate_limit: rotated.rate_limit as u32,
            consciousness_level: rotated.consciousness_level as u8,
        })
        .await;

    Ok((
        StatusCode::OK,
        Json(RotateApiKeyResponse {
            old_key_id: key_id,
            key: map_api_key_record(rotated),
            secret_key,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/history-sync/users -- user-level sync drift view
#[utoipa::path(
    get,
    path = "/api/v1/admin/history-sync/users",
    tag = "admin",
    params(
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "History sync users", body = AdminHistorySyncUsersResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn history_sync_users(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<PaginationQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:history-sync:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 50, 200);

    let items = repo
        .list_history_sync_users(limit, offset)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to list history-sync users: {e}")))?
        .into_iter()
        .map(|row| AdminHistorySyncUserItem {
            user_id: row.user_id.to_string(),
            email: row.email,
            readings_count: row.readings_count,
            usage_events_count: row.usage_events_count,
            drift_count: row.drift_count,
            status: if row.drift_count > 0 {
                "lagging".to_string()
            } else if row.drift_count < 0 {
                "ahead".to_string()
            } else {
                "synced".to_string()
            },
            last_reading_at: row.last_reading_at,
            last_event_at: row.last_event_at,
        })
        .collect::<Vec<_>>();

    let total = repo.count_history_sync_users().await.map_err(|e| {
        EngineError::InternalError(format!("Failed to count history-sync users: {e}"))
    })?;

    Ok((
        StatusCode::OK,
        Json(AdminHistorySyncUsersResponse {
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/history-sync/devices -- device/key-level sync source view
#[utoipa::path(
    get,
    path = "/api/v1/admin/history-sync/devices",
    tag = "admin",
    params(
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "History sync devices", body = AdminHistorySyncDevicesResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn history_sync_devices(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<PaginationQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:history-sync:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 50, 200);

    let items = repo
        .list_history_sync_devices(limit, offset)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to list history-sync devices: {e}"))
        })?
        .into_iter()
        .map(|row| AdminHistorySyncDeviceItem {
            device_id: row.device_id.to_string(),
            user_id: row.user_id.to_string(),
            user_email: row.user_email,
            tier: row.tier,
            status: if row.is_active {
                "active".to_string()
            } else {
                "revoked".to_string()
            },
            permission_count: row.permission_count,
            last_seen_at: row.last_seen_at,
            created_at: row.created_at,
        })
        .collect::<Vec<_>>();

    let total = repo.count_history_sync_devices().await.map_err(|e| {
        EngineError::InternalError(format!("Failed to count history-sync devices: {e}"))
    })?;

    Ok((
        StatusCode::OK,
        Json(AdminHistorySyncDevicesResponse {
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/history-sync/events -- recent ingestion/usage events
#[utoipa::path(
    get,
    path = "/api/v1/admin/history-sync/events",
    tag = "admin",
    params(
        ("status" = Option<String>, Query, description = "Filter by status (success|failure)"),
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "History sync events", body = AdminHistorySyncEventsResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn history_sync_events(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<HistoryEventsQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:history-sync:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 50, 200);

    let status_filter = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let items = repo
        .list_history_sync_events(status_filter, limit, offset)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to list history-sync events: {e}"))
        })?
        .into_iter()
        .map(|row| AdminHistorySyncEventItem {
            event_id: row.event_id.to_string(),
            user_id: row.user_id.to_string(),
            user_email: row.user_email,
            engine_id: row.engine_id,
            workflow_id: row.workflow_id,
            status: row.status,
            duration_ms: row.duration_ms,
            occurred_at: row.occurred_at,
        })
        .collect::<Vec<_>>();

    let total = repo
        .count_history_sync_events(status_filter)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to count history-sync events: {e}"))
        })?;

    Ok((
        StatusCode::OK,
        Json(AdminHistorySyncEventsResponse {
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/analytics/summary -- aggregate usage metrics
#[utoipa::path(
    get,
    path = "/api/v1/admin/analytics/summary",
    tag = "admin",
    params(("window_hours" = Option<i64>, Query, description = "Lookback window in hours")),
    responses(
        (status = 200, description = "Analytics summary", body = AdminAnalyticsSummaryResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn analytics_summary(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:analytics:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let window_hours = query.window_hours.unwrap_or(24).clamp(1, 24 * 30);

    let summary = repo.analytics_summary(window_hours).await.map_err(|e| {
        EngineError::InternalError(format!("Failed to fetch analytics summary: {e}"))
    })?;

    let unique_keys = repo
        .analytics_unique_keys(window_hours)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch unique key analytics: {e}"))
        })?;

    Ok((
        StatusCode::OK,
        Json(AdminAnalyticsSummaryResponse {
            window_hours,
            requests_total: summary.requests_total,
            success_total: summary.success_total,
            failure_total: summary.failure_total,
            error_rate_pct: summary.error_rate_pct,
            p95_duration_ms: summary.p95_duration_ms,
            avg_duration_ms: summary.avg_duration_ms,
            active_users: summary.active_users,
            unique_keys,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/analytics/usage-timeseries -- time series usage metrics
#[utoipa::path(
    get,
    path = "/api/v1/admin/analytics/usage-timeseries",
    tag = "admin",
    params(
        ("window_hours" = Option<i64>, Query, description = "Lookback window in hours"),
        ("bucket" = Option<String>, Query, description = "Bucket granularity: hour|day"),
    ),
    responses(
        (status = 200, description = "Analytics time series", body = AdminAnalyticsTimeseriesResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn analytics_timeseries(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:analytics:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let window_hours = query.window_hours.unwrap_or(24).clamp(1, 24 * 30);
    let bucket = query
        .bucket
        .as_deref()
        .map(|b| b.trim().to_ascii_lowercase())
        .filter(|b| b == "hour" || b == "day")
        .unwrap_or_else(|| {
            if window_hours > 72 {
                "day".to_string()
            } else {
                "hour".to_string()
            }
        });

    let points = repo
        .analytics_timeseries(window_hours, &bucket)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch analytics timeseries: {e}"))
        })?
        .into_iter()
        .map(|point| AdminAnalyticsTimeseriesPoint {
            bucket_start: point.bucket_start,
            request_count: point.request_count,
            success_count: point.success_count,
            failure_count: point.failure_count,
            avg_duration_ms: point.avg_duration_ms,
        })
        .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(AdminAnalyticsTimeseriesResponse {
            window_hours,
            bucket,
            points,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/analytics/usage-breakdown -- engine/workflow segmented usage
#[utoipa::path(
    get,
    path = "/api/v1/admin/analytics/usage-breakdown",
    tag = "admin",
    params(
        ("window_hours" = Option<i64>, Query, description = "Lookback window in hours"),
        ("limit" = Option<i64>, Query, description = "Max rows per segment"),
    ),
    responses(
        (status = 200, description = "Analytics breakdown", body = AdminAnalyticsBreakdownResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn analytics_breakdown(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:analytics:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let window_hours = query.window_hours.unwrap_or(24).clamp(1, 24 * 30);
    let limit = query.limit.unwrap_or(10).clamp(1, 50);

    let engines = repo
        .analytics_engine_breakdown(window_hours, limit)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch analytics engine breakdown: {e}"))
        })?
        .into_iter()
        .map(|row| AdminAnalyticsBreakdownEntry {
            label: row.label,
            request_count: row.request_count,
        })
        .collect::<Vec<_>>();

    let workflows = repo
        .analytics_workflow_breakdown(window_hours, limit)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch analytics workflow breakdown: {e}"))
        })?
        .into_iter()
        .map(|row| AdminAnalyticsBreakdownEntry {
            label: row.label,
            request_count: row.request_count,
        })
        .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(AdminAnalyticsBreakdownResponse {
            window_hours,
            engines,
            workflows,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/analytics/top-consumers -- top traffic consumers
#[utoipa::path(
    get,
    path = "/api/v1/admin/analytics/top-consumers",
    tag = "admin",
    params(
        ("window_hours" = Option<i64>, Query, description = "Lookback window in hours"),
        ("limit" = Option<i64>, Query, description = "Max number of users"),
    ),
    responses(
        (status = 200, description = "Top consumer analytics", body = AdminAnalyticsTopConsumersResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn analytics_top_consumers(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:analytics:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let window_hours = query.window_hours.unwrap_or(24).clamp(1, 24 * 30);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);

    let items = repo
        .analytics_top_consumers(window_hours, limit)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch top consumers analytics: {e}"))
        })?
        .into_iter()
        .map(|row| AdminAnalyticsTopConsumerItem {
            user_id: row.user_id.to_string(),
            user_email: row.user_email,
            request_count: row.request_count,
            failure_count: row.failure_count,
            avg_duration_ms: row.avg_duration_ms,
        })
        .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(AdminAnalyticsTopConsumersResponse {
            window_hours,
            items,
        }),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_platform_admin_role() {
        let permissions = vec![
            "admin:users:roles:update".to_string(),
            "admin:keys:create".to_string(),
        ];
        let roles = derive_roles(&permissions);
        assert!(roles.iter().any(|r| r == "platform-admin"));
        assert!(roles.iter().any(|r| r == "admin"));
    }

    #[test]
    fn derives_legacy_alias_roles() {
        let permissions = vec!["admin:users".to_string(), "admin:analytics".to_string()];
        let roles = derive_roles(&permissions);
        assert!(roles.iter().any(|r| r == "support"));
        assert!(roles.iter().any(|r| r == "viewer"));
    }

    #[test]
    fn legacy_users_permission_unlocks_keys_and_history() {
        let permissions = vec!["admin:users".to_string()];
        assert!(has_permission(&permissions, "admin:keys:list"));
        assert!(has_permission(&permissions, "admin:history-sync:read"));
    }

    #[test]
    fn legacy_analytics_permission_unlocks_system_and_audit() {
        let permissions = vec!["admin:analytics".to_string()];
        let normalized = normalize_effective_permissions(&permissions);
        assert!(normalized.iter().any(|perm| perm == "admin:analytics:read"));
        assert!(normalized.iter().any(|perm| perm == "admin:system:read"));
        assert!(normalized.iter().any(|perm| perm == "admin:audit:list"));
    }

    #[test]
    fn legacy_users_permission_normalizes_keys_and_history() {
        let permissions = vec!["admin:users".to_string()];
        let normalized = normalize_effective_permissions(&permissions);
        assert!(normalized.iter().any(|perm| perm == "admin:users:list"));
        assert!(normalized.iter().any(|perm| perm == "admin:keys:list"));
        assert!(normalized
            .iter()
            .any(|perm| perm == "admin:history-sync:read"));
    }

    #[test]
    fn role_to_permissions_maps_platform_admin() {
        let roles = vec!["platform-admin".to_string()];
        let permissions = permissions_for_roles(&roles);
        assert!(permissions.iter().any(|perm| perm == "admin:*"));
        assert!(permissions
            .iter()
            .any(|perm| perm == "admin:users:roles:update"));
    }
}
