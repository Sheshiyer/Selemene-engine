use crate::{error::ApiError, AppState, ErrorMapper};
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use noesis_auth::{sha256_hex, ApiKey, AuthUser};
use noesis_bridge;
use noesis_core::EngineError;
use noesis_data::models::living_reading::LivingReadingListFilters;
use noesis_data::models::reading::AdminReadingRecord;
use noesis_data::models::witness_dyad::WitnessDyadExecutionAdminRecord;
use noesis_data::repositories::admin_repository::{
    AdminApiKeyRecord, AdminRepository, AdminUserRecord, AuditEventRecord,
    SystemWorkflowSnapshotRecord,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeSet, HashMap},
    time::Instant,
};
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

#[derive(Serialize, ToSchema)]
pub struct AdminUsageWindowSummary {
    pub total: i64,
    pub success: i64,
    pub failure: i64,
    pub active_users: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminUsageEngineEntry {
    pub engine_id: String,
    pub request_count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminUsageTopUserEntry {
    pub user_id: String,
    pub user_email: String,
    pub request_count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminUsageDailyPoint {
    pub day: String,
    pub request_count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminUsageTierEntry {
    pub tier: String,
    pub request_count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminUsageSummaryResponse {
    pub daily: AdminUsageWindowSummary,
    pub monthly: AdminUsageWindowSummary,
    pub daily_requests: Vec<AdminUsageDailyPoint>,
    pub engine_breakdown: Vec<AdminUsageEngineEntry>,
    pub tier_distribution: Vec<AdminUsageTierEntry>,
    pub top_users: Vec<AdminUsageTopUserEntry>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSystemHealthResponse {
    pub checked_at: DateTime<Utc>,
    pub overall_status: String,
    pub uptime_seconds: u64,
    pub subsystems: Vec<AdminSystemSubsystemStatus>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSystemSubsystemStatus {
    pub name: String,
    pub status: String,
    pub detail: String,
    pub latency_ms: Option<f64>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSystemServicesResponse {
    pub items: Vec<AdminSystemServiceItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct AdminSystemServiceItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub status: String,
    pub detail: String,
    pub latency_ms: Option<f64>,
    pub error_rate_pct: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSystemWorkflowsResponse {
    pub window_hours: i64,
    pub items: Vec<AdminSystemWorkflowItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct AdminSystemWorkflowItem {
    pub workflow_id: String,
    pub name: String,
    pub engine_count: i32,
    pub recent_runs: i64,
    pub failure_runs: i64,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synthesis_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_phase: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hits: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_entries: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminBiofieldSessionsResponse {
    pub items: Vec<AdminBiofieldSessionItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminBiofieldSessionItem {
    pub session_id: String,
    pub user_id: String,
    pub user_email: String,
    pub status: String,
    pub client_device_id: Option<String>,
    pub viewer_version: Option<String>,
    pub started_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub artifact_count: i64,
    pub reading_count: i64,
    pub latest_reading_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSystemEnginesResponse {
    pub items: Vec<AdminSystemEngineItem>,
    pub total: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSystemEngineItem {
    pub engine_id: String,
    pub engine_name: String,
    pub required_phase: i32,
    pub category: String,
    pub recent_runs: i64,
    pub failure_runs: i64,
    pub avg_duration_ms: f64,
    pub status: String,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSystemEngineDetailResponse {
    pub engine: AdminSystemEngineItem,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSystemWorkflowDetailResponse {
    pub workflow: AdminSystemWorkflowDetailItem,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSystemWorkflowDetailItem {
    pub workflow_id: String,
    pub name: String,
    pub description: String,
    pub engine_count: i32,
    pub engine_ids: Vec<String>,
    pub synthesis_type: Option<String>,
    pub required_phase: Option<i32>,
    pub recent_runs: i64,
    pub failure_runs: i64,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub status: String,
    pub cache_hits: Option<i64>,
    pub cache_entries: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSystemCacheResponse {
    pub checked_at: DateTime<Utc>,
    pub redis_available: bool,
    pub l1_entries: i32,
    pub total_requests: i64,
    pub l1_hits: i64,
    pub l2_hits: i64,
    pub l3_hits: i64,
    pub cache_misses: i64,
    pub hit_rate_pct: f64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAuditEventsResponse {
    pub items: Vec<AdminAuditEventItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAuditEventItem {
    pub event_id: String,
    // Note: request_id is not stored separately in the DB — reserved for future
    // HTTP request correlation. Consumers should use event_id as the stable identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub actor_user_id: String,
    pub actor_email: String,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub result: String,
    pub duration_ms: i32,
    pub metadata: Value,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAuditEventDetailResponse {
    pub event: AdminAuditEventItem,
}

#[derive(Serialize, ToSchema)]
pub struct AdminAuditActionsResponse {
    pub actions: Vec<String>,
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

#[derive(Deserialize, Default)]
pub struct AdminUsageSummaryQuery {
    pub engine_limit: Option<i64>,
    pub top_users_limit: Option<i64>,
    pub range_days: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct SystemServicesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct SystemWorkflowsQuery {
    pub window_hours: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct AuditEventsQuery {
    pub actor: Option<String>,
    pub action: Option<String>,
    pub result: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct BiofieldSessionsQuery {
    pub status: Option<String>,
    pub user_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct EngineDetailQuery {
    pub window_hours: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminWitnessDyadExecutionsResponse {
    pub items: Vec<AdminWitnessDyadExecutionItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminWitnessDyadExecutionItem {
    pub id: String,
    pub user_id: String,
    pub user_email: String,
    pub tier: String,
    pub consciousness_level: i32,
    pub live_scores: Value,
    pub relationship_mode: String,
    pub engines_available: Value,
    pub aletheios: Option<String>,
    pub pichet: Option<String>,
    pub synthesis: Option<String>,
    pub witness_question: Option<String>,
    pub engines_used: Value,
    pub llm_powered: bool,
    pub llm_provider: Option<String>,
    pub llm_model_aletheios: Option<String>,
    pub llm_model_pichet: Option<String>,
    pub llm_model_synthesis: Option<String>,
    pub llm_duration_ms: Option<f64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminWitnessDyadExecutionDetailResponse {
    pub id: String,
    pub user_id: String,
    pub tier: String,
    pub consciousness_level: i32,
    pub live_scores: Value,
    pub relationship_mode: String,
    pub engines_available: Value,
    pub aletheios: Option<String>,
    pub pichet: Option<String>,
    pub synthesis: Option<String>,
    pub witness_question: Option<String>,
    pub engines_used: Value,
    pub llm_powered: bool,
    pub llm_provider: Option<String>,
    pub llm_model_aletheios: Option<String>,
    pub llm_model_pichet: Option<String>,
    pub llm_model_synthesis: Option<String>,
    pub llm_duration_ms: Option<f64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminWitnessDyadAnalyticsResponse {
    pub window_hours: i64,
    pub llm_vs_rule_based: Vec<AdminWitnessDyadModeEntry>,
    pub llm_rate_pct: f64,
    pub engine_coverage: Vec<AdminAnalyticsBreakdownEntry>,
    pub tier_breakdown: Vec<AdminWitnessDyadTierEntry>,
    pub avg_llm_duration_ms: f64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminWitnessDyadModeEntry {
    pub llm_powered: bool,
    pub count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminWitnessDyadTierEntry {
    pub tier: String,
    pub llm_count: i64,
    pub rule_count: i64,
}

#[derive(Deserialize, Default)]
pub struct WitnessDyadQuery {
    pub user_id: Option<String>,
    pub tier: Option<String>,
    pub llm_powered: Option<bool>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminReadingsResponse {
    pub items: Vec<AdminReadingItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminReadingItem {
    pub id: String,
    pub user_id: String,
    pub user_email: String,
    pub engine_id: String,
    pub workflow_id: Option<String>,
    pub input_hash: String,
    pub input_data: Value,
    pub result_data: Value,
    pub witness_prompt: Option<String>,
    pub consciousness_level: i16,
    pub calculation_time_ms: Option<f64>,
    /// Validated but self-asserted first-party client hint. Never an auth signal.
    pub claimed_source_client: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminReadingsEngineBreakdownResponse {
    pub window_hours: i64,
    pub engines: Vec<AdminAnalyticsBreakdownEntry>,
}

#[derive(Deserialize, Default)]
pub struct AdminReadingsQuery {
    pub user_id: Option<String>,
    pub engine_id: Option<String>,
    pub claimed_source_client: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct AdminLivingReadingsQuery {
    pub owner_user_id: Option<String>,
    pub subject_id: Option<String>,
    pub relationship_id: Option<String>,
    pub source_id: Option<String>,
    pub import_run_id: Option<String>,
    pub editorial_state: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub(crate) fn has_permission(permissions: &[String], required: &str) -> bool {
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

    // Parent grant: holding `admin:billing:read` does NOT grant cancel/trigger
    // (those are explicit destructive perms). But holding the explicit perm
    // for any billing sub-resource grants read.
    if required == "admin:billing:read"
        && permissions
            .iter()
            .any(|perm| perm.starts_with("admin:billing:"))
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
        "admin:keys:delete",
        "admin:users:tier:update",
        "admin:history-sync:retry",
    ];
    if admin_signals.iter().any(|perm| has(perm)) {
        roles.insert("admin".to_string());
    }

    if has("admin:users:roles:update") {
        roles.insert("platform-admin".to_string());
    }

    let billing_signals = [
        "admin:billing:read",
        "admin:billing:subscriptions:cancel",
        "admin:billing:reconcile:trigger",
    ];
    if billing_signals.iter().any(|perm| has(perm)) {
        roles.insert("billing-admin".to_string());
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
                permissions.insert("admin:keys:delete".to_string());
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
                permissions.insert("admin:keys:delete".to_string());
                permissions.insert("admin:users:tier:update".to_string());
            }
            // Billing admin: payment-system surface only. Deliberately disjoint
            // from `admin` so granting billing access doesn't grant user admin
            // (and vice versa). `platform-admin` retains everything via
            // `admin:*` wildcard.
            "billing-admin" => {
                permissions.insert("admin:billing:read".to_string());
                permissions.insert("admin:billing:subscriptions:cancel".to_string());
                permissions.insert("admin:billing:reconcile:trigger".to_string());
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
        "admin:keys:delete",
        "admin:history-sync:read",
        "admin:history-sync:retry",
        "admin:billing:read",
        "admin:billing:subscriptions:cancel",
        "admin:billing:reconcile:trigger",
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

pub(crate) fn normalize_limit_offset(
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

pub(crate) fn default_rate_limit_for_tier(tier: &str) -> i32 {
    match tier.to_ascii_lowercase().as_str() {
        "free" => 60,
        "premium" => 1_000,
        "enterprise" => 10_000,
        _ => 100,
    }
}

pub(crate) fn generate_secret_api_key() -> String {
    format!("nk_{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

pub(crate) fn json_error_response(
    status: StatusCode,
    error: impl Into<String>,
    error_code: &str,
    details: Option<Value>,
) -> Response {
    ErrorMapper::response(status, error_code, error.into(), details).into_response()
}

pub(crate) fn service_unavailable_response() -> Response {
    json_error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "Admin APIs require a configured database connection",
        "ADMIN_DB_UNAVAILABLE",
        None,
    )
}

pub(crate) fn forbidden_response(required_permission: &str) -> Response {
    tracing::warn!(
        required_permission = %required_permission,
        "Authorization denied — insufficient permissions"
    );
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

pub(crate) async fn effective_permissions(
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

fn map_audit_event_record(record: AuditEventRecord) -> AdminAuditEventItem {
    AdminAuditEventItem {
        event_id: record.event_id.to_string(),
        request_id: None, // not stored in DB; event_id is the stable trace identifier
        occurred_at: record.occurred_at,
        actor_user_id: record.actor_user_id.to_string(),
        actor_email: record.actor_email,
        action: record.action,
        target_type: record.target_type,
        target_id: record.target_id,
        result: record.result,
        duration_ms: record.duration_ms,
        metadata: serde_json::json!({
            "engine_id": record.engine_id,
            "workflow_id": record.workflow_id,
        }),
    }
}

#[derive(Clone)]
struct RuntimeProbe {
    status: String,
    detail: String,
    latency_ms: Option<f64>,
}

struct SystemRuntimeSnapshot {
    checked_at: DateTime<Utc>,
    api: RuntimeProbe,
    database: RuntimeProbe,
    cache: RuntimeProbe,
    orchestrator: RuntimeProbe,
    error_rate_pct: Option<f64>,
}

fn status_priority(status: &str) -> i32 {
    match status {
        "unavailable" => 3,
        "degraded" => 2,
        _ => 1,
    }
}

fn summarize_overall_status(statuses: &[String]) -> String {
    if statuses.iter().any(|status| status == "unavailable") {
        "unavailable".to_string()
    } else if statuses.iter().any(|status| status == "degraded") {
        "degraded".to_string()
    } else {
        "healthy".to_string()
    }
}

async fn collect_system_runtime_snapshot(state: &AppState) -> SystemRuntimeSnapshot {
    let checked_at = Utc::now();

    let api = RuntimeProbe {
        status: "healthy".to_string(),
        detail: "API process reachable".to_string(),
        latency_ms: Some(0.0),
    };

    let database = if let Some(repo) = state.admin_repository.as_ref() {
        let started = Instant::now();
        match repo.ping().await {
            Ok(true) => RuntimeProbe {
                status: "healthy".to_string(),
                detail: "Database responded to ping".to_string(),
                latency_ms: Some(started.elapsed().as_secs_f64() * 1000.0),
            },
            Ok(false) => RuntimeProbe {
                status: "degraded".to_string(),
                detail: "Database ping returned unexpected response".to_string(),
                latency_ms: Some(started.elapsed().as_secs_f64() * 1000.0),
            },
            Err(err) => RuntimeProbe {
                status: "unavailable".to_string(),
                detail: format!("Database ping failed: {err}"),
                latency_ms: Some(started.elapsed().as_secs_f64() * 1000.0),
            },
        }
    } else {
        RuntimeProbe {
            status: "unavailable".to_string(),
            detail: "Admin repository not configured".to_string(),
            latency_ms: None,
        }
    };

    let cache_started = Instant::now();
    let cache = match state.cache.health_check().await {
        Ok(true) => RuntimeProbe {
            status: "healthy".to_string(),
            detail: "Redis cache available".to_string(),
            latency_ms: Some(cache_started.elapsed().as_secs_f64() * 1000.0),
        },
        Ok(false) => RuntimeProbe {
            status: "degraded".to_string(),
            detail: "Redis cache unavailable".to_string(),
            latency_ms: Some(cache_started.elapsed().as_secs_f64() * 1000.0),
        },
        Err(err) => RuntimeProbe {
            status: "unavailable".to_string(),
            detail: format!("Cache health check failed: {err}"),
            latency_ms: Some(cache_started.elapsed().as_secs_f64() * 1000.0),
        },
    };

    let orchestrator_started = Instant::now();
    let orchestrator = match state.orchestrator.is_ready().await {
        Ok(true) => RuntimeProbe {
            status: "healthy".to_string(),
            detail: "Workflow orchestrator ready".to_string(),
            latency_ms: Some(orchestrator_started.elapsed().as_secs_f64() * 1000.0),
        },
        Ok(false) => RuntimeProbe {
            status: "degraded".to_string(),
            detail: "Workflow orchestrator not ready".to_string(),
            latency_ms: Some(orchestrator_started.elapsed().as_secs_f64() * 1000.0),
        },
        Err(err) => RuntimeProbe {
            status: "unavailable".to_string(),
            detail: format!("Workflow orchestrator check failed: {err}"),
            latency_ms: Some(orchestrator_started.elapsed().as_secs_f64() * 1000.0),
        },
    };

    let error_rate_pct = if let Some(repo) = state.admin_repository.as_ref() {
        repo.analytics_summary(24)
            .await
            .map(|summary| summary.error_rate_pct)
            .ok()
    } else {
        None
    };

    SystemRuntimeSnapshot {
        checked_at,
        api,
        database,
        cache,
        orchestrator,
        error_rate_pct,
    }
}

fn require_permission_or_forbidden(permissions: &[String], required: &str) -> Option<Response> {
    if has_permission(permissions, required) {
        None
    } else {
        tracing::warn!(required_permission = %required, "Authorization denied — insufficient permissions");
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
    const VALID_TIERS: &[&str] = &["free", "basic", "premium", "enterprise"];
    if !VALID_TIERS.contains(&tier.as_str()) {
        return Ok(json_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!(
                "Invalid tier '{}'. Must be one of: {}",
                tier,
                VALID_TIERS.join(", ")
            ),
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
        const VALID_TIERS: &[&str] = &["free", "basic", "premium", "enterprise"];
        if !VALID_TIERS.contains(&tier.as_str()) {
            return Ok(json_error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!(
                    "Invalid tier '{}'. Must be one of: {}",
                    tier,
                    VALID_TIERS.join(", ")
                ),
                "VALIDATION_ERROR",
                None,
            ));
        }
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

    let rate_limit = {
        let rl = payload
            .rate_limit
            .unwrap_or_else(|| default_rate_limit_for_tier(&tier));
        // Enforce sane bounds: 0 disables rate limiting (allowed for enterprise),
        // 100_000 is a hard ceiling to prevent runaway values.
        if !(0..=100_000).contains(&rl) {
            return Ok(json_error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("rate_limit must be between 0 and 100000, got {rl}"),
                "VALIDATION_ERROR",
                None,
            ));
        }
        rl
    };

    let consciousness_level = payload.consciousness_level.unwrap_or(0).clamp(0, 5);

    let secret_key = generate_secret_api_key();
    let key_hash = sha256_hex(&secret_key);
    let key_prefix = secret_key[..12.min(secret_key.len())].to_string();
    let actor_user_id = Uuid::parse_str(&auth_user.user_id).ok();

    let created = repo
        .create_api_key(
            noesis_data::repositories::admin_repository::NewApiKeyRecord {
                key_hash,
                name: payload.name,
                key_prefix,
                user_id: user_uuid,
                created_by_user_id: actor_user_id,
                tier: tier.clone(),
                permissions: serde_json::json!(permissions),
                consciousness_level,
                rate_limit,
                expires_at: payload.expires_at,
                rotated_from_key_id: None,
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
    let actor_user_id = Uuid::parse_str(&auth_user.user_id).ok();

    let revoked = repo
        .revoke_api_key(key_uuid, actor_user_id)
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
    let actor_user_id = Uuid::parse_str(&auth_user.user_id).ok();

    let rotated = repo
        .rotate_api_key(key_uuid, &key_hash, &key_prefix, actor_user_id)
        .await
        .map_err(|e| {
            tracing::error!(
                error = %e,
                key_id = %key_id,
                actor_user_id = ?actor_user_id,
                "rotate_api_key DB error"
            );
            EngineError::InternalError(format!("Failed to rotate api key: {e}"))
        })?;

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

/// DELETE /api/v1/admin/api-keys/{key_id} -- permanently delete an API key
#[utoipa::path(
    delete,
    path = "/api/v1/admin/api-keys/{key_id}",
    tag = "admin",
    params(("key_id" = String, Path, description = "API key UUID")),
    responses(
        (status = 200, description = "API key deleted"),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 404, description = "Key not found", body = crate::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn delete_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(key_id): Path<String>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:keys:delete")
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

    let deleted = repo
        .delete_api_key(key_uuid)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to delete api key: {e}")))?;

    if !deleted {
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
            "message": "API key deleted",
            "key_id": key_id,
        })),
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

/// GET /api/v1/admin/usage/summary -- usage summary for dashboard views
#[utoipa::path(
    get,
    path = "/api/v1/admin/usage/summary",
    tag = "admin",
    params(
        ("engine_limit" = Option<i64>, Query, description = "Max engine breakdown rows (default 10, max 50)"),
        ("top_users_limit" = Option<i64>, Query, description = "Max top users rows (default 10, max 100)"),
        ("range_days" = Option<i64>, Query, description = "Daily chart range in days (7-90, default 30)")
    ),
    responses(
        (status = 200, description = "Usage summary", body = AdminUsageSummaryResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 503, description = "Usage repository unavailable", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn usage_summary(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AdminUsageSummaryQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) =
        require_permission_or_forbidden(&effective_permissions, "admin:analytics:read")
    {
        return Ok(resp);
    }

    let usage_repo = state
        .usage_repository
        .as_ref()
        .ok_or_else(|| EngineError::InternalError("Usage repository not configured".to_string()))?;

    let engine_limit = query.engine_limit.unwrap_or(10).clamp(1, 50);
    let top_users_limit = query.top_users_limit.unwrap_or(10).clamp(1, 100);
    let range_days = query.range_days.unwrap_or(30).clamp(7, 90);

    let daily = usage_repo.admin_usage_summary(24).await.map_err(|e| {
        EngineError::InternalError(format!("Failed to fetch daily usage summary: {e}"))
    })?;

    let monthly = usage_repo.admin_usage_summary(24 * 30).await.map_err(|e| {
        EngineError::InternalError(format!("Failed to fetch monthly usage summary: {e}"))
    })?;

    let daily_requests = usage_repo
        .admin_daily_series(range_days)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch daily usage series: {e}"))
        })?
        .into_iter()
        .map(|point| AdminUsageDailyPoint {
            day: point.day,
            request_count: point.request_count,
        })
        .collect::<Vec<_>>();

    let engine_breakdown = usage_repo
        .admin_engine_breakdown(24 * 30, engine_limit)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch admin engine breakdown: {e}"))
        })?
        .into_iter()
        .map(|row| AdminUsageEngineEntry {
            engine_id: row.engine_id,
            request_count: row.request_count,
        })
        .collect::<Vec<_>>();

    let tier_distribution = usage_repo
        .admin_tier_distribution(24 * 30)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to fetch tier distribution: {e}")))?
        .into_iter()
        .map(|row| AdminUsageTierEntry {
            tier: row.tier,
            request_count: row.request_count,
        })
        .collect::<Vec<_>>();

    let top_users = usage_repo
        .admin_top_users(24 * 30, top_users_limit)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to fetch top users: {e}")))?
        .into_iter()
        .map(|row| AdminUsageTopUserEntry {
            user_id: row.user_id.to_string(),
            user_email: row.user_email,
            request_count: row.request_count,
        })
        .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(AdminUsageSummaryResponse {
            daily: AdminUsageWindowSummary {
                total: daily.total,
                success: daily.success,
                failure: daily.failure,
                active_users: daily.active_users,
            },
            monthly: AdminUsageWindowSummary {
                total: monthly.total,
                success: monthly.success,
                failure: monthly.failure,
                active_users: monthly.active_users,
            },
            daily_requests,
            engine_breakdown,
            tier_distribution,
            top_users,
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

/// GET /api/v1/admin/system/health -- runtime health view for core subsystems
#[utoipa::path(
    get,
    path = "/api/v1/admin/system/health",
    tag = "admin",
    responses(
        (status = 200, description = "System health snapshot", body = AdminSystemHealthResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn system_health(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let snapshot = collect_system_runtime_snapshot(&state).await;
    let subsystem_items = vec![
        AdminSystemSubsystemStatus {
            name: "api".to_string(),
            status: snapshot.api.status.clone(),
            detail: snapshot.api.detail,
            latency_ms: snapshot.api.latency_ms,
        },
        AdminSystemSubsystemStatus {
            name: "database".to_string(),
            status: snapshot.database.status.clone(),
            detail: snapshot.database.detail,
            latency_ms: snapshot.database.latency_ms,
        },
        AdminSystemSubsystemStatus {
            name: "cache".to_string(),
            status: snapshot.cache.status.clone(),
            detail: snapshot.cache.detail,
            latency_ms: snapshot.cache.latency_ms,
        },
        AdminSystemSubsystemStatus {
            name: "orchestrator".to_string(),
            status: snapshot.orchestrator.status.clone(),
            detail: snapshot.orchestrator.detail,
            latency_ms: snapshot.orchestrator.latency_ms,
        },
    ];

    let overall_status = summarize_overall_status(
        &subsystem_items
            .iter()
            .map(|item| item.status.clone())
            .collect::<Vec<_>>(),
    );

    Ok((
        StatusCode::OK,
        Json(AdminSystemHealthResponse {
            checked_at: snapshot.checked_at,
            overall_status,
            uptime_seconds: state.startup_time.elapsed().as_secs(),
            subsystems: subsystem_items,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/system/services -- service-level operational statuses
#[utoipa::path(
    get,
    path = "/api/v1/admin/system/services",
    tag = "admin",
    params(
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "System services snapshot", body = AdminSystemServicesResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn system_services(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<SystemServicesQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let snapshot = collect_system_runtime_snapshot(&state).await;
    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 25, 200);

    let mut services = [
        AdminSystemServiceItem {
            id: "api".to_string(),
            name: "Noesis API".to_string(),
            category: "application".to_string(),
            status: snapshot.api.status,
            detail: snapshot.api.detail,
            latency_ms: snapshot.api.latency_ms,
            error_rate_pct: snapshot.error_rate_pct,
            updated_at: snapshot.checked_at,
        },
        AdminSystemServiceItem {
            id: "database".to_string(),
            name: "PostgreSQL".to_string(),
            category: "database".to_string(),
            status: snapshot.database.status,
            detail: snapshot.database.detail,
            latency_ms: snapshot.database.latency_ms,
            error_rate_pct: None,
            updated_at: snapshot.checked_at,
        },
        AdminSystemServiceItem {
            id: "cache".to_string(),
            name: "Redis Cache".to_string(),
            category: "cache".to_string(),
            status: snapshot.cache.status,
            detail: snapshot.cache.detail,
            latency_ms: snapshot.cache.latency_ms,
            error_rate_pct: None,
            updated_at: snapshot.checked_at,
        },
        AdminSystemServiceItem {
            id: "orchestrator".to_string(),
            name: "Workflow Orchestrator".to_string(),
            category: "compute".to_string(),
            status: snapshot.orchestrator.status,
            detail: snapshot.orchestrator.detail,
            latency_ms: snapshot.orchestrator.latency_ms,
            error_rate_pct: None,
            updated_at: snapshot.checked_at,
        },
    ];

    services.sort_by(|a, b| {
        status_priority(&b.status)
            .cmp(&status_priority(&a.status))
            .then_with(|| a.name.cmp(&b.name))
    });

    let total = services.len() as i64;
    let start = offset as usize;
    let end = (offset + limit) as usize;
    let items = if start >= services.len() {
        Vec::new()
    } else {
        services[start..services.len().min(end)].to_vec()
    };

    Ok((
        StatusCode::OK,
        Json(AdminSystemServicesResponse {
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/system/workflows -- workflow runtime snapshots
#[utoipa::path(
    get,
    path = "/api/v1/admin/system/workflows",
    tag = "admin",
    params(
        ("window_hours" = Option<i64>, Query, description = "Lookback window in hours"),
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "Workflow runtime snapshots", body = AdminSystemWorkflowsResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 503, description = "Database unavailable", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn system_workflows(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<SystemWorkflowsQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let window_hours = query.window_hours.unwrap_or(24).clamp(1, 24 * 30);
    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 25, 200);

    let snapshots = repo
        .system_workflow_snapshots(window_hours)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch workflow snapshots: {e}"))
        })?;

    let snapshot_by_workflow: HashMap<String, SystemWorkflowSnapshotRecord> = snapshots
        .iter()
        .map(|snapshot| (snapshot.workflow_id.clone(), snapshot.clone()))
        .collect();

    let mut workflow_items = state
        .orchestrator
        .list_workflows()
        .iter()
        .map(|workflow| {
            let usage = snapshot_by_workflow.get(&workflow.id);
            let recent_runs = usage.map(|entry| entry.request_count).unwrap_or(0);
            let failure_runs = usage.map(|entry| entry.failure_count).unwrap_or(0);
            let status = if recent_runs == 0 {
                "idle".to_string()
            } else if failure_runs * 100 >= recent_runs * 20 {
                "degraded".to_string()
            } else {
                "healthy".to_string()
            };

            let (synthesis_type, engine_ids, required_phase, cache_hits, cache_entries) = state
                .workflow_registry
                .as_ref()
                .and_then(|reg| reg.get(&workflow.id))
                .map(|ext| {
                    (
                        Some(format!("{:?}", ext.synthesis_type)),
                        Some(ext.engine_ids.clone()),
                        Some(ext.required_phase as i32),
                        None::<i64>,
                        None::<i64>,
                    )
                })
                .unwrap_or((None, None, None, None, None));

            AdminSystemWorkflowItem {
                workflow_id: workflow.id.clone(),
                name: workflow.name.clone(),
                engine_count: workflow.engine_ids.len() as i32,
                recent_runs,
                failure_runs,
                last_seen_at: usage.and_then(|entry| entry.last_seen_at),
                status,
                synthesis_type,
                engine_ids,
                required_phase,
                cache_hits,
                cache_entries,
            }
        })
        .collect::<Vec<_>>();

    let known_workflows = workflow_items
        .iter()
        .map(|item| item.workflow_id.clone())
        .collect::<BTreeSet<_>>();

    for snapshot in snapshots {
        if known_workflows.contains(&snapshot.workflow_id) {
            continue;
        }

        let status = if snapshot.request_count == 0 {
            "idle".to_string()
        } else if snapshot.failure_count * 100 >= snapshot.request_count * 20 {
            "degraded".to_string()
        } else {
            "healthy".to_string()
        };

        workflow_items.push(AdminSystemWorkflowItem {
            workflow_id: snapshot.workflow_id.clone(),
            name: format!("{} (legacy)", snapshot.workflow_id),
            engine_count: 0,
            recent_runs: snapshot.request_count,
            failure_runs: snapshot.failure_count,
            last_seen_at: snapshot.last_seen_at,
            status,
            synthesis_type: None,
            engine_ids: None,
            required_phase: None,
            cache_hits: None,
            cache_entries: None,
        });
    }

    workflow_items.sort_by(|a, b| {
        b.recent_runs
            .cmp(&a.recent_runs)
            .then_with(|| b.last_seen_at.cmp(&a.last_seen_at))
            .then_with(|| a.workflow_id.cmp(&b.workflow_id))
    });

    let total = workflow_items.len() as i64;
    let start = offset as usize;
    let end = (offset + limit) as usize;
    let items = if start >= workflow_items.len() {
        Vec::new()
    } else {
        workflow_items[start..workflow_items.len().min(end)].to_vec()
    };

    Ok((
        StatusCode::OK,
        Json(AdminSystemWorkflowsResponse {
            window_hours,
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/system/cache -- cache hit/miss and availability metrics
#[utoipa::path(
    get,
    path = "/api/v1/admin/system/cache",
    tag = "admin",
    responses(
        (status = 200, description = "Cache snapshot", body = AdminSystemCacheResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn system_cache(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let cache_stats = state.cache.get_stats().await;
    let redis_available = state.cache.health_check().await.unwrap_or(false);

    Ok((
        StatusCode::OK,
        Json(AdminSystemCacheResponse {
            checked_at: Utc::now(),
            redis_available,
            l1_entries: state.cache.l1_entry_count() as i32,
            total_requests: cache_stats.total_requests as i64,
            l1_hits: cache_stats.l1_hits as i64,
            l2_hits: cache_stats.l2_hits as i64,
            l3_hits: cache_stats.l3_hits as i64,
            cache_misses: cache_stats.cache_misses as i64,
            hit_rate_pct: (cache_stats.hit_rate() * 100.0 * 100.0).round() / 100.0,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/audit-events -- immutable usage/audit stream
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit-events",
    tag = "admin",
    params(
        ("actor" = Option<String>, Query, description = "Filter by actor email or UUID"),
        ("action" = Option<String>, Query, description = "Filter by action name"),
        ("result" = Option<String>, Query, description = "Filter by result status"),
        ("from" = Option<String>, Query, description = "ISO start timestamp"),
        ("to" = Option<String>, Query, description = "ISO end timestamp"),
        ("limit" = Option<i64>, Query, description = "Pagination limit"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "Audit events", body = AdminAuditEventsResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 503, description = "Database unavailable", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn list_audit_events(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AuditEventsQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:audit:list")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 50, 200);

    let action_filter = query
        .action
        .as_deref()
        .map(|action| action.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());

    let result_filter = query
        .result
        .as_deref()
        .map(|result| result.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());

    let events = repo
        .list_audit_events(
            query.actor.as_deref(),
            action_filter.as_deref(),
            result_filter.as_deref(),
            query.from,
            query.to,
            limit,
            offset,
        )
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to list audit events: {e}")))?
        .into_iter()
        .map(map_audit_event_record)
        .collect::<Vec<_>>();

    let total = repo
        .count_audit_events(
            query.actor.as_deref(),
            action_filter.as_deref(),
            result_filter.as_deref(),
            query.from,
            query.to,
        )
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to count audit events: {e}")))?;

    Ok((
        StatusCode::OK,
        Json(AdminAuditEventsResponse {
            items: events,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/audit-events/{event_id} -- single audit event detail
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit-events/{event_id}",
    tag = "admin",
    params(("event_id" = String, Path, description = "Audit event UUID")),
    responses(
        (status = 200, description = "Audit event detail", body = AdminAuditEventDetailResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 404, description = "Event not found", body = crate::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::ErrorResponse),
        (status = 503, description = "Database unavailable", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn get_audit_event(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(event_id): Path<String>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    let can_read = has_permission(&effective_permissions, "admin:audit:read")
        || has_permission(&effective_permissions, "admin:audit:list");
    if !can_read {
        return Ok(forbidden_response("admin:audit:read"));
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let event_uuid = match parse_uuid_or_422(&event_id, "event_id") {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    let event = repo
        .get_audit_event(event_uuid)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to fetch audit event: {e}")))?;

    let Some(event) = event else {
        return Ok(json_error_response(
            StatusCode::NOT_FOUND,
            "Audit event not found",
            "NOT_FOUND",
            Some(serde_json::json!({ "event_id": event_id })),
        ));
    };

    Ok((
        StatusCode::OK,
        Json(AdminAuditEventDetailResponse {
            event: map_audit_event_record(event),
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/audit-events/actions -- distinct action names for filter autocomplete
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit-events/actions",
    tag = "admin",
    responses(
        (status = 200, description = "Audit action values", body = AdminAuditActionsResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
        (status = 503, description = "Database unavailable", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn list_audit_actions(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:audit:list")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let actions = repo
        .list_audit_actions()
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to list audit actions: {e}")))?;

    Ok((StatusCode::OK, Json(AdminAuditActionsResponse { actions })).into_response())
}

// ── Witness Dyad admin handlers ──────────────────────────────────────────────

fn map_witness_dyad_admin_record(
    record: WitnessDyadExecutionAdminRecord,
) -> AdminWitnessDyadExecutionItem {
    AdminWitnessDyadExecutionItem {
        id: record.id.to_string(),
        user_id: record.user_id.to_string(),
        user_email: record.user_email,
        tier: record.tier,
        consciousness_level: record.consciousness_level,
        live_scores: record.live_scores,
        relationship_mode: record.relationship_mode,
        engines_available: record.engines_available,
        aletheios: record.aletheios,
        pichet: record.pichet,
        synthesis: record.synthesis,
        witness_question: record.witness_question,
        engines_used: record.engines_used,
        llm_powered: record.llm_powered,
        llm_provider: record.llm_provider,
        llm_model_aletheios: record.llm_model_aletheios,
        llm_model_pichet: record.llm_model_pichet,
        llm_model_synthesis: record.llm_model_synthesis,
        llm_duration_ms: record.llm_duration_ms,
        error_message: record.error_message,
        created_at: record.created_at,
    }
}

/// GET /api/v1/admin/witness-dyad/executions -- list witness dyad execution records
pub async fn list_witness_dyad_executions(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<WitnessDyadQuery>,
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

    let user_filter = match query.user_id.as_deref() {
        Some(uid) => match parse_uuid_or_422(uid, "user_id") {
            Ok(parsed) => Some(parsed),
            Err(resp) => return Ok(resp),
        },
        None => None,
    };

    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 50, 200);

    let items = repo
        .list_witness_dyad_executions(
            user_filter,
            query.tier.as_deref(),
            query.llm_powered,
            query.from,
            query.to,
            limit,
            offset,
        )
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to list witness dyad executions: {e}"))
        })?
        .into_iter()
        .map(map_witness_dyad_admin_record)
        .collect::<Vec<_>>();

    let total = repo
        .count_witness_dyad_executions(
            user_filter,
            query.tier.as_deref(),
            query.llm_powered,
            query.from,
            query.to,
        )
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to count witness dyad executions: {e}"))
        })?;

    Ok((
        StatusCode::OK,
        Json(AdminWitnessDyadExecutionsResponse {
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/witness-dyad/executions/{id} -- single execution detail
pub async fn get_witness_dyad_execution(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(execution_id): Path<String>,
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

    let exec_uuid = match parse_uuid_or_422(&execution_id, "execution_id") {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    let execution = repo
        .get_witness_dyad_execution(exec_uuid)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch witness dyad execution: {e}"))
        })?;

    let Some(execution) = execution else {
        return Ok(json_error_response(
            StatusCode::NOT_FOUND,
            "Witness dyad execution not found",
            "NOT_FOUND",
            Some(serde_json::json!({ "execution_id": execution_id })),
        ));
    };

    Ok((
        StatusCode::OK,
        Json(AdminWitnessDyadExecutionDetailResponse {
            id: execution.id.to_string(),
            user_id: execution.user_id.to_string(),
            tier: execution.tier,
            consciousness_level: execution.consciousness_level,
            live_scores: execution.live_scores,
            relationship_mode: execution.relationship_mode,
            engines_available: execution.engines_available,
            aletheios: execution.aletheios,
            pichet: execution.pichet,
            synthesis: execution.synthesis,
            witness_question: execution.witness_question,
            engines_used: execution.engines_used,
            llm_powered: execution.llm_powered,
            llm_provider: execution.llm_provider,
            llm_model_aletheios: execution.llm_model_aletheios,
            llm_model_pichet: execution.llm_model_pichet,
            llm_model_synthesis: execution.llm_model_synthesis,
            llm_duration_ms: execution.llm_duration_ms,
            error_message: execution.error_message,
            created_at: execution.created_at,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/witness-dyad/analytics -- dyad analytics summary
pub async fn witness_dyad_analytics(
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

    let mode_breakdown = repo
        .witness_dyad_mode_breakdown(window_hours)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch witness dyad mode breakdown: {e}"))
        })?;

    let engine_coverage = repo
        .witness_engine_coverage(window_hours)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch witness engine coverage: {e}"))
        })?
        .into_iter()
        .map(|row| AdminAnalyticsBreakdownEntry {
            label: row.label,
            request_count: row.request_count,
        })
        .collect::<Vec<_>>();

    let llm_total = mode_breakdown
        .iter()
        .find(|m| m.llm_powered)
        .map(|m| m.count)
        .unwrap_or(0);
    let rule_total = mode_breakdown
        .iter()
        .find(|m| !m.llm_powered)
        .map(|m| m.count)
        .unwrap_or(0);
    let grand_total = llm_total + rule_total;
    let llm_rate_pct = if grand_total > 0 {
        (llm_total as f64 / grand_total as f64) * 100.0
    } else {
        0.0
    };

    let llm_vs_rule_based = mode_breakdown
        .iter()
        .map(|m| AdminWitnessDyadModeEntry {
            llm_powered: m.llm_powered,
            count: m.count,
        })
        .collect::<Vec<_>>();

    let tier_breakdown = repo
        .list_witness_dyad_executions(None, None, None, None, None, 500, 0)
        .await
        .map(|executions| {
            let mut tier_map: HashMap<String, (i64, i64)> = HashMap::new();
            for e in &executions {
                let (llm, rule) = tier_map.entry(e.tier.clone()).or_insert((0, 0));
                if e.llm_powered {
                    *llm += 1;
                } else {
                    *rule += 1;
                }
            }
            tier_map
                .into_iter()
                .map(
                    |(tier, (llm_count, rule_count))| AdminWitnessDyadTierEntry {
                        tier,
                        llm_count,
                        rule_count,
                    },
                )
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let avg_llm_duration_ms = repo
        .witness_dyad_avg_llm_duration_ms(window_hours)
        .await
        .unwrap_or(0.0);

    Ok((
        StatusCode::OK,
        Json(AdminWitnessDyadAnalyticsResponse {
            window_hours,
            llm_vs_rule_based,
            llm_rate_pct: (llm_rate_pct * 100.0).round() / 100.0,
            engine_coverage,
            avg_llm_duration_ms,
            tier_breakdown: {
                let mut tb = tier_breakdown;
                tb.sort_by(|a, b| {
                    (b.llm_count + b.rule_count)
                        .cmp(&(a.llm_count + a.rule_count))
                        .then_with(|| a.tier.cmp(&b.tier))
                });
                tb
            },
        }),
    )
        .into_response())
}

// ── Admin readings handlers ──────────────────────────────────────────────────

const LIVING_READING_EDITORIAL_STATES: &[&str] = &[
    "imported",
    "needs_review",
    "in_review",
    "approved",
    "rejected",
    "published",
    "archived",
];

#[allow(clippy::result_large_err)]
fn parse_optional_uuid_filter(value: Option<&str>, field: &str) -> Result<Option<Uuid>, Response> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| parse_uuid_or_422(value, field))
        .transpose()
}

/// GET /api/v1/admin/living-readings -- canonical archive browser
pub async fn list_living_readings(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AdminLivingReadingsQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(response) =
        require_permission_or_forbidden(&effective_permissions, "admin:analytics:read")
    {
        return Ok(response);
    }

    let admin_repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(response) => return Ok(response),
    };

    let owner_user_id =
        match parse_optional_uuid_filter(query.owner_user_id.as_deref(), "owner_user_id") {
            Ok(value) => value,
            Err(response) => return Ok(response),
        };
    let subject_id = match parse_optional_uuid_filter(query.subject_id.as_deref(), "subject_id") {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let relationship_id =
        match parse_optional_uuid_filter(query.relationship_id.as_deref(), "relationship_id") {
            Ok(value) => value,
            Err(response) => return Ok(response),
        };
    let source_id = match parse_optional_uuid_filter(query.source_id.as_deref(), "source_id") {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let import_run_id =
        match parse_optional_uuid_filter(query.import_run_id.as_deref(), "import_run_id") {
            Ok(value) => value,
            Err(response) => return Ok(response),
        };

    let editorial_state = query
        .editorial_state
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    if let Some(editorial_state) = editorial_state.as_deref() {
        if !LIVING_READING_EDITORIAL_STATES.contains(&editorial_state) {
            return Ok(json_error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid editorial_state",
                "INVALID_INPUT",
                Some(serde_json::json!({
                    "editorial_state": editorial_state,
                    "allowed": LIVING_READING_EDITORIAL_STATES,
                })),
            ));
        }
    }

    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 25, 100);
    let filters = LivingReadingListFilters {
        owner_user_id,
        subject_id,
        relationship_id,
        source_id,
        import_run_id,
        editorial_state,
    };
    let page = admin_repo
        .living_readings()
        .list(&filters, limit, offset)
        .await
        .map_err(|error| {
            EngineError::InternalError(format!("Failed to list living readings: {error}"))
        })?;

    Ok((StatusCode::OK, Json(page)).into_response())
}

/// GET /api/v1/admin/living-readings/:reading_id -- canonical archive detail
pub async fn get_living_reading(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(reading_id): Path<String>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(response) =
        require_permission_or_forbidden(&effective_permissions, "admin:analytics:read")
    {
        return Ok(response);
    }

    let admin_repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(response) => return Ok(response),
    };
    let reading_id = match parse_uuid_or_422(&reading_id, "reading_id") {
        Ok(reading_id) => reading_id,
        Err(response) => return Ok(response),
    };

    match admin_repo
        .living_readings()
        .get(reading_id)
        .await
        .map_err(|error| {
            EngineError::InternalError(format!("Failed to fetch living reading: {error}"))
        })? {
        Some(detail) => Ok((StatusCode::OK, Json(detail)).into_response()),
        None => Ok(json_error_response(
            StatusCode::NOT_FOUND,
            "Living reading not found",
            "LIVING_READING_NOT_FOUND",
            Some(serde_json::json!({ "reading_id": reading_id })),
        )),
    }
}

fn map_admin_reading_record(record: AdminReadingRecord) -> AdminReadingItem {
    AdminReadingItem {
        id: record.id.to_string(),
        user_id: record.user_id.to_string(),
        user_email: record.user_email,
        engine_id: record.engine_id,
        workflow_id: record.workflow_id,
        input_hash: record.input_hash,
        input_data: record.input_data,
        result_data: record.result_data,
        witness_prompt: record.witness_prompt,
        consciousness_level: record.consciousness_level,
        calculation_time_ms: record.calculation_time_ms,
        claimed_source_client: record.claimed_source_client,
        created_at: record.created_at,
    }
}

/// GET /api/v1/admin/readings -- cross-user readings list
pub async fn list_all_readings(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AdminReadingsQuery>,
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

    let user_filter = match query.user_id.as_deref() {
        Some(uid) => match parse_uuid_or_422(uid, "user_id") {
            Ok(parsed) => Some(parsed),
            Err(resp) => return Ok(resp),
        },
        None => None,
    };

    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 50, 200);

    let items = repo
        .list_all_readings(
            user_filter,
            query.engine_id.as_deref(),
            query.claimed_source_client.as_deref(),
            limit,
            offset,
        )
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to list readings: {e}")))?
        .into_iter()
        .map(map_admin_reading_record)
        .collect::<Vec<_>>();

    let total = repo
        .count_all_readings(
            user_filter,
            query.engine_id.as_deref(),
            query.claimed_source_client.as_deref(),
        )
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to count readings: {e}")))?;

    Ok((
        StatusCode::OK,
        Json(AdminReadingsResponse {
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/readings/engine-breakdown -- readings engine breakdown
pub async fn readings_engine_breakdown(
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

    let engines = repo
        .readings_platform_engine_breakdown(window_hours)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch readings engine breakdown: {e}"))
        })?
        .into_iter()
        .map(|row| AdminAnalyticsBreakdownEntry {
            label: row.label,
            request_count: row.request_count,
        })
        .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(AdminReadingsEngineBreakdownResponse {
            window_hours,
            engines,
        }),
    )
        .into_response())
}

/// Response body for the ephemeris checksums endpoint.
#[derive(Serialize, ToSchema)]
pub struct EphemerisChecksumsResponse {
    /// Map of `.se1` filename to its hex-encoded SHA256 digest.
    pub checksums: HashMap<String, String>,
}

/// GET /api/v1/admin/ephemeris/checksums — SHA256 checksums of ephemeris files
#[utoipa::path(
    get,
    path = "/api/v1/admin/ephemeris/checksums",
    tag = "admin",
    responses(
        (status = 200, description = "Ephemeris file checksums", body = EphemerisChecksumsResponse),
        (status = 403, description = "Forbidden", body = crate::ErrorResponse),
    ),
    security(("bearer_auth" = []), ("api_key" = []))
)]
pub async fn ephemeris_checksums(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    // Serialize directly from the Arc to avoid cloning the underlying HashMap.
    let checksums = &*state.ephemeris_checksums;
    Ok((
        StatusCode::OK,
        Json(EphemerisChecksumsResponse {
            checksums: checksums.clone(),
        }),
    )
        .into_response())
}

// ── Biofield Session Admin ───────────────────────────────────────────────────

/// GET /api/v1/admin/biofield/sessions — list sessions with artifact/reading aggregations
pub async fn list_biofield_sessions(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<BiofieldSessionsQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let (limit, offset) = normalize_limit_offset(query.limit, query.offset, 25, 200);

    let user_id_filter = match query.user_id.as_deref() {
        Some(uid) => match parse_uuid_or_422(uid, "user_id") {
            Ok(parsed) => Some(parsed),
            Err(resp) => return Ok(resp),
        },
        None => None,
    };

    let status_filter = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let records = repo
        .list_biofield_sessions(status_filter, user_id_filter, limit, offset)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to list biofield sessions: {e}"))
        })?;

    let items = records
        .into_iter()
        .map(|r| AdminBiofieldSessionItem {
            session_id: r.id.to_string(),
            user_id: r.user_id.to_string(),
            user_email: r.user_email,
            status: r.status,
            client_device_id: r.client_device_id,
            viewer_version: r.viewer_version,
            started_at: r.started_at,
            closed_at: r.closed_at,
            artifact_count: r.artifact_count,
            reading_count: r.reading_count,
            latest_reading_at: r.latest_reading_at,
        })
        .collect::<Vec<_>>();

    let total = repo
        .count_biofield_sessions(status_filter, user_id_filter)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to count biofield sessions: {e}"))
        })?;

    Ok((
        StatusCode::OK,
        Json(AdminBiofieldSessionsResponse {
            items,
            total,
            limit,
            offset,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/biofield/sessions/:session_id — single session detail
pub async fn get_biofield_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(session_id): Path<String>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let session_uuid = match parse_uuid_or_422(&session_id, "session_id") {
        Ok(id) => id,
        Err(resp) => return Ok(resp),
    };

    let sessions = repo
        .list_biofield_sessions(None, None, 500, 0)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to fetch session: {e}")))?;

    let Some(record) = sessions.into_iter().find(|s| s.id == session_uuid) else {
        return Ok(json_error_response(
            StatusCode::NOT_FOUND,
            "Biofield session not found",
            "NOT_FOUND",
            Some(serde_json::json!({ "session_id": session_id })),
        ));
    };

    Ok((
        StatusCode::OK,
        Json(AdminBiofieldSessionItem {
            session_id: record.id.to_string(),
            user_id: record.user_id.to_string(),
            user_email: record.user_email,
            status: record.status,
            client_device_id: record.client_device_id,
            viewer_version: record.viewer_version,
            started_at: record.started_at,
            closed_at: record.closed_at,
            artifact_count: record.artifact_count,
            reading_count: record.reading_count,
            latest_reading_at: record.latest_reading_at,
        }),
    )
        .into_response())
}

// ── Engine Registry Admin ────────────────────────────────────────────────────

fn classify_engine_category(engine_id: &str, bridge_engine_ids: &BTreeSet<String>) -> String {
    if engine_id == "biofield-capture" {
        "python-sidecar".to_string()
    } else if bridge_engine_ids.contains(engine_id) {
        "ts-bridge".to_string()
    } else {
        "rust-native".to_string()
    }
}

/// GET /api/v1/admin/system/engines — all engines with analytics and categorization
pub async fn list_engines_admin(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<EngineDetailQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let window_hours = query.window_hours.unwrap_or(24).clamp(1, 24 * 30);

    let bridge_engine_ids: BTreeSet<String> = state
        .bridge()
        .engines()
        .iter()
        .map(|e| e.engine_id().to_string())
        .collect();

    let analytics_detail = repo
        .analytics_engine_detail(window_hours)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch engine detail analytics: {e}"))
        })?;

    let analytics_by_engine: HashMap<String, (i64, i64, f64)> = analytics_detail
        .iter()
        .map(|a| {
            (
                a.engine_id.clone(),
                (a.request_count, a.failure_count, a.avg_duration_ms),
            )
        })
        .collect();

    let registry_engines = state.orchestrator.registry();

    let mut items = Vec::new();
    let mut seen = BTreeSet::new();

    for engine_id in registry_engines.list() {
        if seen.contains(engine_id) {
            continue;
        }
        seen.insert(engine_id.to_string());

        let (engine_name, required_phase) = match registry_engines.get(engine_id) {
            Some(engine) => (
                engine.engine_name().to_string(),
                engine.required_phase() as i32,
            ),
            None => (engine_id.to_string(), 0),
        };
        let category = classify_engine_category(engine_id, &bridge_engine_ids);
        let analytics = analytics_by_engine.get(engine_id);
        let recent_runs = analytics.map(|a| a.0).unwrap_or(0);
        let failure_runs = analytics.map(|a| a.1).unwrap_or(0);
        let avg_duration = analytics.map(|a| a.2).unwrap_or(0.0);

        let status = if recent_runs == 0 {
            "unknown".to_string()
        } else if failure_runs * 100 >= recent_runs * 20 {
            "degraded".to_string()
        } else {
            "healthy".to_string()
        };

        items.push(AdminSystemEngineItem {
            engine_id: engine_id.to_string(),
            engine_name,
            required_phase,
            category,
            recent_runs,
            failure_runs,
            avg_duration_ms: avg_duration,
            status,
        });
    }

    let total = items.len() as i64;

    Ok((
        StatusCode::OK,
        Json(AdminSystemEnginesResponse { items, total }),
    )
        .into_response())
}

/// GET /api/v1/admin/system/engines/:id — single engine detail
pub async fn get_engine_admin(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(engine_id): Path<String>,
    Query(query): Query<EngineDetailQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let window_hours = query.window_hours.unwrap_or(24).clamp(1, 24 * 30);

    let registry_engines = state.orchestrator.registry();
    let engine = registry_engines.get(&engine_id).ok_or_else(|| {
        ApiError(EngineError::EngineNotFound(format!(
            "Engine '{}' not found",
            engine_id
        )))
    })?;

    let bridge_engine_ids: BTreeSet<String> = state
        .bridge()
        .engines()
        .iter()
        .map(|e| e.engine_id().to_string())
        .collect();

    let category = classify_engine_category(&engine_id, &bridge_engine_ids);

    let analytics_detail = repo
        .analytics_engine_detail(window_hours)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch engine detail analytics: {e}"))
        })?;

    let engine_analytics = analytics_detail.iter().find(|a| a.engine_id == engine_id);

    let recent_runs = engine_analytics.map(|a| a.request_count).unwrap_or(0);
    let failure_runs = engine_analytics.map(|a| a.failure_count).unwrap_or(0);
    let avg_duration = engine_analytics.map(|a| a.avg_duration_ms).unwrap_or(0.0);

    let status = if recent_runs == 0 {
        "unknown".to_string()
    } else if failure_runs * 100 >= recent_runs * 20 {
        "degraded".to_string()
    } else {
        "healthy".to_string()
    };

    let item = AdminSystemEngineItem {
        engine_id: engine.engine_id().to_string(),
        engine_name: engine.engine_name().to_string(),
        required_phase: engine.required_phase() as i32,
        category,
        recent_runs,
        failure_runs,
        avg_duration_ms: avg_duration,
        status,
    };

    Ok((
        StatusCode::OK,
        Json(AdminSystemEngineDetailResponse { engine: item }),
    )
        .into_response())
}

/// GET /api/v1/admin/system/workflows/:id — single workflow detail
pub async fn get_workflow_admin(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(workflow_id): Path<String>,
    Query(query): Query<SystemWorkflowsQuery>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let repo = match admin_repo_or_503(&state) {
        Ok(repo) => repo,
        Err(resp) => return Ok(resp),
    };

    let window_hours = query.window_hours.unwrap_or(24).clamp(1, 24 * 30);

    let base = state
        .orchestrator
        .get_workflow(&workflow_id)
        .ok_or_else(|| {
            ApiError(EngineError::WorkflowNotFound(format!(
                "Workflow '{}' not found",
                workflow_id
            )))
        })?;

    let extended = state
        .workflow_registry
        .as_ref()
        .and_then(|reg| reg.get(&workflow_id).cloned());

    let snapshots = repo
        .system_workflow_snapshots(window_hours)
        .await
        .map_err(|e| {
            EngineError::InternalError(format!("Failed to fetch workflow snapshots: {e}"))
        })?;

    let snapshot = snapshots.iter().find(|s| s.workflow_id == workflow_id);

    let recent_runs = snapshot.map(|s| s.request_count).unwrap_or(0);
    let failure_runs = snapshot.map(|s| s.failure_count).unwrap_or(0);

    let status = if recent_runs == 0 {
        "idle".to_string()
    } else if failure_runs * 100 >= recent_runs * 20 {
        "degraded".to_string()
    } else {
        "healthy".to_string()
    };

    let (synthesis_type, required_phase, cache_hits, cache_entries) =
        if let Some(ref ext) = extended {
            (
                Some(format!("{:?}", ext.synthesis_type)),
                Some(ext.required_phase as i32),
                None,
                None,
            )
        } else {
            (None, None, None, None)
        };

    let item = AdminSystemWorkflowDetailItem {
        workflow_id: base.id.clone(),
        name: base.name.clone(),
        description: base.description.clone(),
        engine_count: base.engine_ids.len() as i32,
        engine_ids: base.engine_ids.clone(),
        synthesis_type,
        required_phase,
        recent_runs,
        failure_runs,
        last_seen_at: snapshot.and_then(|s| s.last_seen_at),
        status,
        cache_hits,
        cache_entries,
    };

    Ok((
        StatusCode::OK,
        Json(AdminSystemWorkflowDetailResponse { workflow: item }),
    )
        .into_response())
}

// ── Bridge Health Admin (T-019, T-020) ──────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct AdminBridgeHealthResponse {
    pub base_url: String,
    pub sidecar_reachable: bool,
    pub overall_status: String,
    pub total_engines: i32,
    pub healthy_engines: i32,
    pub degraded_engines: i32,
    pub engines: Vec<AdminBridgeEngineHealth>,
    pub failed_engines: Vec<String>,
    pub config: AdminBridgeConfig,
}

#[derive(Serialize, ToSchema)]
pub struct AdminBridgeEngineHealth {
    pub engine_id: String,
    pub engine_name: String,
    pub healthy: bool,
    pub detail: String,
    pub latency_ms: u64,
    pub circuit_state: String,
    pub circuit_failures: u32,
    pub circuit_last_failure: Option<String>,
    pub required_phase: u8,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSidecarEngineHealth {
    pub engine_id: String,
    pub healthy: bool,
    pub detail: String,
    pub latency_ms: u64,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSidecarCircuitBreaker {
    pub state: String,
    pub failures: u32,
    pub last_failure_ts: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSidecarDetail {
    pub base_url: String,
    pub status: String,
    pub engines: Vec<AdminSidecarEngineHealth>,
    pub failed_engines: Vec<String>,
    pub circuit_breakers: HashMap<String, AdminSidecarCircuitBreaker>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminBridgeConfig {
    pub timeout_secs: u32,
    pub cb_threshold: u32,
    pub cb_reset_secs: u32,
}

fn read_env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

/// GET /api/v1/admin/bridge/health — bridge circuit breaker + sidecar readiness
pub async fn bridge_health(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let base_url = state.bridge().base_url().to_string();
    let timeout_secs = read_env_u32("TS_BRIDGE_TIMEOUT", 30);
    let cb_threshold = read_env_u32("TS_BRIDGE_CB_THRESHOLD", 5);
    let cb_reset_secs = read_env_u32("TS_BRIDGE_CB_RESET_SECS", 30);

    let config = AdminBridgeConfig {
        timeout_secs,
        cb_threshold,
        cb_reset_secs,
    };

    let sidecar_reachable = state.bridge().is_available().await;

    let readiness = state.bridge().readiness_status().await;

    let (engines, failed_engines, _sidecar_engines) = match readiness {
        Ok(status) => {
            let sidecar_map: HashMap<String, noesis_bridge::SidecarEngineHealth> = status
                .engines
                .clone()
                .into_iter()
                .map(|e| (e.engine_id.clone(), e))
                .collect();

            let engines: Vec<AdminBridgeEngineHealth> = state
                .bridge()
                .engines()
                .iter()
                .map(|engine| {
                    let eid = engine.engine_id().to_string();
                    let sidecar = sidecar_map.get(&eid);
                    // Try to get underlying BridgeEngine to read circuit breaker
                    let (circuit_state, circuit_failures, circuit_last_failure) = {
                        let circuit = engine
                            .as_ref()
                            .as_any()
                            .downcast_ref::<noesis_bridge::BridgeEngine>()
                            .map(|be| be.circuit_breaker());
                        match circuit {
                            Some(cb) => (
                                cb.state().as_str().to_string(),
                                cb.failure_count(),
                                cb.last_failure_at().map(|dt| dt.to_rfc3339()),
                            ),
                            None => ("unknown".to_string(), 0u32, None),
                        }
                    };

                    AdminBridgeEngineHealth {
                        engine_id: eid.clone(),
                        engine_name: engine.engine_name().to_string(),
                        healthy: sidecar.map(|s| s.healthy).unwrap_or(false),
                        detail: sidecar
                            .map(|s| s.detail.clone())
                            .unwrap_or_else(|| "no sidecar data".to_string()),
                        latency_ms: sidecar.map(|s| s.latency_ms).unwrap_or(0),
                        circuit_state,
                        circuit_failures,
                        circuit_last_failure,
                        required_phase: engine.required_phase(),
                    }
                })
                .collect();

            (
                engines,
                status.failed_engines.clone(),
                status.engines.clone(),
            )
        }
        Err(e) => {
            let engines: Vec<AdminBridgeEngineHealth> = state
                .bridge()
                .engines()
                .iter()
                .map(|engine| {
                    let eid = engine.engine_id().to_string();
                    let (circuit_state, circuit_failures, circuit_last_failure) = {
                        let circuit = engine
                            .as_ref()
                            .as_any()
                            .downcast_ref::<noesis_bridge::BridgeEngine>()
                            .map(|be| be.circuit_breaker());
                        match circuit {
                            Some(cb) => (
                                cb.state().as_str().to_string(),
                                cb.failure_count(),
                                cb.last_failure_at().map(|dt| dt.to_rfc3339()),
                            ),
                            None => ("unknown".to_string(), 0u32, None),
                        }
                    };

                    AdminBridgeEngineHealth {
                        engine_id: eid,
                        engine_name: engine.engine_name().to_string(),
                        healthy: false,
                        detail: format!("sidecar unreachable: {e}"),
                        latency_ms: 0,
                        circuit_state,
                        circuit_failures,
                        circuit_last_failure,
                        required_phase: engine.required_phase(),
                    }
                })
                .collect();

            (engines, vec!["ts-sidecar".to_string()], vec![])
        }
    };

    let healthy_count = engines.iter().filter(|e| e.healthy).count() as i32;
    let degraded_count = engines.len() as i32 - healthy_count;
    let overall_status = if !sidecar_reachable {
        "unreachable"
    } else if failed_engines.is_empty() {
        "healthy"
    } else {
        "degraded"
    };

    Ok((
        StatusCode::OK,
        Json(AdminBridgeHealthResponse {
            base_url,
            sidecar_reachable,
            overall_status: overall_status.to_string(),
            total_engines: engines.len() as i32,
            healthy_engines: healthy_count,
            degraded_engines: degraded_count,
            engines,
            failed_engines,
            config,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/bridge/sidecar — sidecar detail with circuit breakers
pub async fn sidecar_detail(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let readiness = state.bridge().readiness_status().await;
    let base_url = state.bridge().base_url().to_string();
    let sidecar_reachable = state.bridge().is_available().await;

    let engines: Vec<AdminSidecarEngineHealth> = state
        .bridge()
        .engines()
        .iter()
        .map(|engine| {
            let eid = engine.engine_id().to_string();
            let sidecar_data = readiness
                .as_ref()
                .ok()
                .and_then(|s| s.engines.iter().find(|e| e.engine_id == eid));

            AdminSidecarEngineHealth {
                engine_id: eid,
                healthy: sidecar_data.map(|s| s.healthy).unwrap_or(false),
                detail: sidecar_data
                    .map(|s| s.detail.clone())
                    .unwrap_or_else(|| "unreachable".to_string()),
                latency_ms: sidecar_data.map(|s| s.latency_ms).unwrap_or(0),
            }
        })
        .collect();

    let failed_engines: Vec<String> = engines
        .iter()
        .filter(|e| !e.healthy)
        .map(|e| e.engine_id.clone())
        .collect();

    let circuit_breakers: HashMap<String, AdminSidecarCircuitBreaker> = state
        .bridge()
        .engines()
        .iter()
        .filter_map(|engine| {
            let eid = engine.engine_id().to_string();
            let circuit = engine
                .as_ref()
                .as_any()
                .downcast_ref::<noesis_bridge::BridgeEngine>()
                .map(|be| be.circuit_breaker())?;
            Some((
                eid,
                AdminSidecarCircuitBreaker {
                    state: circuit.state().as_str().to_string(),
                    failures: circuit.failure_count(),
                    last_failure_ts: circuit.last_failure_at().map(|dt| dt.to_rfc3339()),
                },
            ))
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(AdminSidecarDetail {
            base_url,
            status: if sidecar_reachable {
                "healthy".to_string()
            } else {
                "unavailable".to_string()
            },
            engines,
            failed_engines,
            circuit_breakers,
        }),
    )
        .into_response())
}

// ── Hermes Bridge Status (T-021) ──────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct AdminHermesBridgeStatus {
    pub configured: bool,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub mode: Option<String>,
    pub noesis_api_key_configured: bool,
    pub tools_available: i32,
    pub health: Option<AdminBridgeProbe>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminBridgeProbe {
    pub reachable: bool,
    pub status: Option<i32>,
    pub detail: String,
}

/// GET /api/v1/admin/bridge/hermes/status — Hermes LLM bridge status
pub async fn hermes_bridge_status(
    State(_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&_state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let base_url = std::env::var("HERMES_BASE_URL").ok();
    let configured = base_url.is_some();
    let model = std::env::var("HERMES_MODEL").ok();
    let noesis_api_key_configured = std::env::var("NOESIS_API_KEY").is_ok();

    let mode = model.as_ref().map(|m| {
        if m.contains("xml") || m.contains("claude") || m.contains("sonnet") {
            "xml-tool-calling"
        } else if m.contains("gpt") || m.contains("command") || m.contains("o3") || m.contains("o4")
        {
            "openai-tool-calling"
        } else {
            "unknown"
        }
        .to_string()
    });

    let health = if let Some(ref url) = base_url {
        match reqwest::get(format!("{url}/models")).await {
            Ok(resp) => Some(AdminBridgeProbe {
                reachable: resp.status().is_success(),
                status: Some(resp.status().as_u16() as i32),
                detail: if resp.status().is_success() {
                    "Hermes models endpoint reachable".to_string()
                } else {
                    format!("Hermes returned status {}", resp.status())
                },
            }),
            Err(e) => Some(AdminBridgeProbe {
                reachable: false,
                status: None,
                detail: format!("Probe failed: {e}"),
            }),
        }
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(AdminHermesBridgeStatus {
            configured,
            base_url,
            model,
            mode,
            noesis_api_key_configured,
            tools_available: 26,
            health,
        }),
    )
        .into_response())
}

// ── Suno Bridge Status (T-022) ────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct AdminSunoBridgeStatus {
    pub configured: bool,
    pub base_url: Option<String>,
    pub health: Option<AdminBridgeProbe>,
    pub credit_info: Option<Value>,
}

/// GET /api/v1/admin/bridge/suno/status — Suno music bridge status
pub async fn suno_bridge_status(
    State(_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&_state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let base_url = std::env::var("SUNO_BASE_URL").ok();
    let session_id_set = std::env::var("SUNO_SESSION_ID").is_ok();
    let configured = base_url.is_some() || session_id_set;

    let (health, credit_info) = if let Some(ref url) = base_url {
        let probe = match reqwest::get(format!("{url}/api/get_limit")).await {
            Ok(resp) => {
                let status = resp.status();
                let reachable = status.is_success();
                let detail = if reachable {
                    "Suno get_limit endpoint reachable".to_string()
                } else {
                    format!("Suno returned status {}", status)
                };
                let credit = if reachable {
                    resp.json::<Value>().await.ok()
                } else {
                    None
                };
                (
                    Some(AdminBridgeProbe {
                        reachable,
                        status: Some(status.as_u16() as i32),
                        detail,
                    }),
                    credit,
                )
            }
            Err(e) => (
                Some(AdminBridgeProbe {
                    reachable: false,
                    status: None,
                    detail: format!("Probe failed: {e}"),
                }),
                None,
            ),
        };
        probe
    } else {
        (None, None)
    };

    Ok((
        StatusCode::OK,
        Json(AdminSunoBridgeStatus {
            configured,
            base_url,
            health,
            credit_info,
        }),
    )
        .into_response())
}

// ── LLM Proxy Status (T-023) ──────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct AdminLlmProxyStatus {
    pub deployed: bool,
    pub endpoint: Option<String>,
    pub active_provider: Option<String>,
    pub providers: Vec<String>,
    pub health: Option<AdminBridgeProbe>,
}

/// GET /api/v1/admin/bridge/llm-proxy/status — LLM proxy Cloudflare Worker status
pub async fn llm_proxy_status(
    State(_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&_state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let endpoint = std::env::var("LLM_PROXY_URL").ok();
    let active_provider = std::env::var("LLM_PROXY_ACTIVE_PROVIDER").ok();
    let providers = vec![
        "command-code".to_string(),
        "nvidia".to_string(),
        "openrouter".to_string(),
        "openai".to_string(),
    ];
    let deployed = endpoint.is_some();

    let health = if let Some(ref url) = endpoint {
        match reqwest::get(format!("{url}/health")).await {
            Ok(resp) => Some(AdminBridgeProbe {
                reachable: resp.status().is_success(),
                status: Some(resp.status().as_u16() as i32),
                detail: if resp.status().is_success() {
                    "LLM proxy healthy".to_string()
                } else {
                    format!("LLM proxy returned status {}", resp.status())
                },
            }),
            Err(e) => Some(AdminBridgeProbe {
                reachable: false,
                status: None,
                detail: format!("Probe failed: {e}"),
            }),
        }
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(AdminLlmProxyStatus {
            deployed,
            endpoint,
            active_provider,
            providers,
            health,
        }),
    )
        .into_response())
}

// ── Observability Summary (T-024) ─────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct AdminObservabilitySummary {
    pub prometheus: Option<AdminObservabilityService>,
    pub alertmanager: Option<AdminObservabilityService>,
    pub grafana: Option<AdminObservabilityService>,
    pub loki: Option<AdminObservabilityService>,
    pub jaeger: Option<AdminObservabilityService>,
    pub active_alerts: Vec<AdminActiveAlert>,
    pub grafana_dashboards: Vec<String>,
    pub uptime_seconds: u64,
    pub metrics_endpoint: String,
}

#[derive(Serialize, ToSchema)]
pub struct AdminObservabilityService {
    pub configured: bool,
    pub url: Option<String>,
    pub reachable: bool,
}

#[derive(Serialize, ToSchema)]
pub struct AdminActiveAlert {
    pub name: String,
    pub severity: String,
    pub summary: String,
    pub since: Option<String>,
    pub service: String,
}

async fn probe_service(url: &str) -> AdminObservabilityService {
    let reachable = match reqwest::get(url).await {
        Ok(resp) => resp.status().is_success() || resp.status().as_u16() == 302,
        Err(_) => false,
    };
    AdminObservabilityService {
        configured: true,
        url: Some(url.to_string()),
        reachable,
    }
}

/// GET /api/v1/admin/observability/summary — observability stack status
pub async fn observability_summary(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let prometheus_url = std::env::var("PROMETHEUS_URL").ok();
    let alertmanager_url = std::env::var("ALERTMANAGER_URL").ok();
    let grafana_url = std::env::var("GRAFANA_URL").ok();
    let loki_url = std::env::var("LOKI_URL").ok();
    let jaeger_url = std::env::var("JAEGER_URL").ok();

    let prometheus = if let Some(ref url) = prometheus_url {
        Some(probe_service(url).await)
    } else {
        None
    };

    let alertmanager = if let Some(ref url) = alertmanager_url {
        Some(probe_service(url).await)
    } else {
        None
    };

    let grafana = if let Some(ref url) = grafana_url {
        Some(probe_service(url).await)
    } else {
        None
    };

    let loki = if let Some(ref url) = loki_url {
        Some(probe_service(url).await)
    } else {
        None
    };

    let jaeger = if let Some(ref url) = jaeger_url {
        Some(probe_service(url).await)
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(AdminObservabilitySummary {
            prometheus,
            alertmanager,
            grafana,
            loki,
            jaeger,
            active_alerts: Vec::new(),
            grafana_dashboards: vec![
                "api-overview".to_string(),
                "engine-performance".to_string(),
                "cache-efficiency".to_string(),
            ],
            uptime_seconds: state.startup_time.elapsed().as_secs(),
            metrics_endpoint: "/metrics".to_string(),
        }),
    )
        .into_response())
}

// ── Skills Ecosystem Status (T-025) ───────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct AdminSelemeneSkill {
    pub name: String,
    pub description: String,
    pub origin: String,
    pub location: String,
    pub status: String,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSelemeneReportMode {
    pub mode: String,
    pub report_level: String,
    pub subject_count_min: i32,
    pub subject_count_max: i32,
    pub roles: Vec<String>,
    pub target_words_min: i32,
    pub target_words_max: i32,
    pub architecture: String,
    pub pass_count: usize,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSelemeneAutoresearch {
    pub enabled: bool,
    pub testing_grounds: String,
    pub description: String,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSelemeneVectorize {
    pub index_name: String,
    pub binding: String,
    pub ai_binding: String,
    pub r2_bucket: String,
    pub d1_database: String,
    pub status: String,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSelemeneMcp {
    pub configured: bool,
    pub base_url: String,
    pub auth_methods: Vec<String>,
    pub tool_count: usize,
    pub tools: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AdminSkillsEcosystemStatus {
    pub selemene_skills: Vec<AdminSelemeneSkill>,
    pub autoresearch: AdminSelemeneAutoresearch,
    pub vectorize: AdminSelemeneVectorize,
    pub report_modes: Vec<AdminSelemeneReportMode>,
    pub mcp: AdminSelemeneMcp,
}

/// GET /api/v1/admin/skills/status — Selemene skills, report modes, Vectorize, MCP, and autoresearch
pub async fn skills_ecosystem_status(
    State(_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let effective_permissions = effective_permissions(&_state, &auth_user).await?;
    if let Some(resp) = require_permission_or_forbidden(&effective_permissions, "admin:system:read")
    {
        return Ok(resp);
    }

    let selemene_skills = vec![
        AdminSelemeneSkill {
            name: "selemene-bridge".to_string(),
            description: "Query 16 consciousness engines and 6 workflows for witnessing cosmic patterns. Non-prescriptive mirror — generates inquiry, not advice.".to_string(),
            origin: "project-local".to_string(),
            location: ".claude/skills/selemene-bridge".to_string(),
            status: "active".to_string(),
        },
        AdminSelemeneSkill {
            name: "selemene-report".to_string(),
            description: "Generate Selemene narrative witness readings and route deterministic reports.".to_string(),
            origin: "agents".to_string(),
            location: "~/.agents/skills/selemene-report".to_string(),
            status: "active".to_string(),
        },
        AdminSelemeneSkill {
            name: "selemene-notebooklm".to_string(),
            description: "Turn a Selemene witness-pipeline OrchestratorOutput into a ready-to-paste NotebookLM prompt.".to_string(),
            origin: "agents".to_string(),
            location: "~/.agents/skills/selemene-notebooklm".to_string(),
            status: "active".to_string(),
        },
        AdminSelemeneSkill {
            name: "task-master-planner".to_string(),
            description: "Generate system-engineering task plans from repo specs and .context docs.".to_string(),
            origin: "project-local".to_string(),
            location: ".claude/skills/task-master-planner".to_string(),
            status: "active".to_string(),
        },
        AdminSelemeneSkill {
            name: "dispatching-parallel-agents".to_string(),
            description: "Dispatch multiple Claude agents to investigate and fix independent problems concurrently.".to_string(),
            origin: "project-local".to_string(),
            location: ".claude/skills/dispatching-parallel-agents".to_string(),
            status: "active".to_string(),
        },
    ];

    let autoresearch = AdminSelemeneAutoresearch {
        enabled: std::env::var("AUTORESEARCH_ENABLED")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(true),
        testing_grounds: std::env::var("AUTORESEARCH_GROUNDS")
            .unwrap_or_else(|_| "local".to_string()),
        description: "Autoresearch loop for skills, prompts, and agent evaluation. Operates locally with LLM-as-judge.".to_string(),
    };

    let vectorize = AdminSelemeneVectorize {
        index_name: std::env::var("VECTORIZE_INDEX_NAME")
            .unwrap_or_else(|_| "selemene-report-patterns".to_string()),
        binding: std::env::var("VECTORIZE_BINDING")
            .unwrap_or_else(|_| "REPORT_PATTERNS".to_string()),
        ai_binding: std::env::var("AI_BINDING").unwrap_or_else(|_| "AI".to_string()),
        r2_bucket: std::env::var("PATTERNS_R2_BUCKET")
            .unwrap_or_else(|_| "selemene-patterns".to_string()),
        d1_database: std::env::var("PATTERNS_D1_DATABASE")
            .unwrap_or_else(|_| "selemene-patterns".to_string()),
        status: std::env::var("VECTORIZE_STATUS").unwrap_or_else(|_| "configured".to_string()),
    };

    let report_modes = vec![
        AdminSelemeneReportMode {
            mode: "birth-blueprint".to_string(),
            report_level: "L1".to_string(),
            subject_count_min: 1,
            subject_count_max: 1,
            roles: vec!["subject".to_string()],
            target_words_min: 1600,
            target_words_max: 2400,
            architecture: "linear".to_string(),
            pass_count: 2,
        },
        AdminSelemeneReportMode {
            mode: "integrated-kundali-l0".to_string(),
            report_level: "L0".to_string(),
            subject_count_min: 1,
            subject_count_max: 1,
            roles: vec!["subject".to_string()],
            target_words_min: 15000,
            target_words_max: 21000,
            architecture: "linear".to_string(),
            pass_count: 11,
        },
        AdminSelemeneReportMode {
            mode: "integrated-reading".to_string(),
            report_level: "L3".to_string(),
            subject_count_min: 1,
            subject_count_max: 2,
            roles: vec!["subject".to_string()],
            target_words_min: 4200,
            target_words_max: 5800,
            architecture: "linear".to_string(),
            pass_count: 3,
        },
        AdminSelemeneReportMode {
            mode: "integrated-reading-l4".to_string(),
            report_level: "L4".to_string(),
            subject_count_min: 1,
            subject_count_max: 2,
            roles: vec!["subject".to_string()],
            target_words_min: 4800,
            target_words_max: 6500,
            architecture: "linear".to_string(),
            pass_count: 2,
        },
        AdminSelemeneReportMode {
            mode: "family-penta".to_string(),
            report_level: "—".to_string(),
            subject_count_min: 3,
            subject_count_max: 7,
            roles: vec![
                "mother".to_string(),
                "father".to_string(),
                "child1".to_string(),
                "child2".to_string(),
                "child3".to_string(),
            ],
            target_words_min: 6000,
            target_words_max: 9000,
            architecture: "hierarchical".to_string(),
            pass_count: 5,
        },
        AdminSelemeneReportMode {
            mode: "mother-son-lineage".to_string(),
            report_level: "—".to_string(),
            subject_count_min: 2,
            subject_count_max: 2,
            roles: vec!["mother".to_string(), "son".to_string()],
            target_words_min: 4000,
            target_words_max: 6000,
            architecture: "linear".to_string(),
            pass_count: 4,
        },
        AdminSelemeneReportMode {
            mode: "partner-synastry".to_string(),
            report_level: "—".to_string(),
            subject_count_min: 2,
            subject_count_max: 2,
            roles: vec!["partner".to_string(), "partner".to_string()],
            target_words_min: 3500,
            target_words_max: 5500,
            architecture: "linear".to_string(),
            pass_count: 4,
        },
        AdminSelemeneReportMode {
            mode: "unmarried-partners".to_string(),
            report_level: "—".to_string(),
            subject_count_min: 2,
            subject_count_max: 2,
            roles: vec!["partner".to_string(), "partner".to_string()],
            target_words_min: 3500,
            target_words_max: 5500,
            architecture: "linear".to_string(),
            pass_count: 4,
        },
        AdminSelemeneReportMode {
            mode: "married-partners".to_string(),
            report_level: "—".to_string(),
            subject_count_min: 2,
            subject_count_max: 2,
            roles: vec!["partner".to_string(), "partner".to_string()],
            target_words_min: 3500,
            target_words_max: 5500,
            architecture: "linear".to_string(),
            pass_count: 4,
        },
        AdminSelemeneReportMode {
            mode: "business-partners".to_string(),
            report_level: "—".to_string(),
            subject_count_min: 2,
            subject_count_max: 2,
            roles: vec![
                "business-partner".to_string(),
                "business-partner".to_string(),
            ],
            target_words_min: 3500,
            target_words_max: 5500,
            architecture: "linear".to_string(),
            pass_count: 4,
        },
    ];

    let mcp = AdminSelemeneMcp {
        configured: std::env::var("MCP_CONFIGURED")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(true),
        base_url: std::env::var("MCP_BASE_URL")
            .unwrap_or_else(|_| "https://selemene.tryambakam.space".to_string()),
        auth_methods: vec!["X-API-Key".to_string(), "Bearer JWT".to_string()],
        tool_count: 6,
        tools: vec![
            "noesis_list_engines".to_string(),
            "noesis_calculate_engine".to_string(),
            "noesis_validate_engine".to_string(),
            "noesis_list_workflows".to_string(),
            "noesis_workflow_info".to_string(),
            "noesis_execute_workflow".to_string(),
        ],
    };

    Ok((
        StatusCode::OK,
        Json(AdminSkillsEcosystemStatus {
            selemene_skills,
            autoresearch,
            vectorize,
            report_modes,
            mcp,
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

    #[test]
    fn admin_role_includes_api_key_delete_permission() {
        let roles = vec!["admin".to_string()];
        let permissions = permissions_for_roles(&roles);
        assert!(permissions.iter().any(|perm| perm == "admin:keys:delete"));
    }

    #[test]
    fn legacy_users_permission_normalizes_api_key_delete() {
        let permissions = vec!["admin:users".to_string()];
        let normalized = normalize_effective_permissions(&permissions);
        assert!(normalized.iter().any(|perm| perm == "admin:keys:delete"));
    }

    #[test]
    fn living_readings_require_analytics_read_permission() {
        assert!(!has_permission(
            &["basic:access".to_string()],
            "admin:analytics:read"
        ));
        assert!(has_permission(
            &["admin:analytics:read".to_string()],
            "admin:analytics:read"
        ));
    }
}
