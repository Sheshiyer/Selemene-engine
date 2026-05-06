//! Admin billing dashboard endpoints.
//!
//! Surfaces a read-only operator view over the Dodo Payments integration:
//! global subscription counts, paginated subscription list, single
//! subscription detail, processed webhook event log, latest reconcile drift,
//! and plan catalog. One write endpoint exists (`subscriptions/:id/cancel`)
//! and is gated by an explicit `admin:billing:subscriptions:cancel`
//! permission.
//!
//! All endpoints require the auth middleware to have populated `AuthUser`
//! with permissions; the permission check is repeated per-endpoint so that
//! granting `admin:billing:read` doesn't grant cancel/trigger.

use crate::handlers::admin::{
    effective_permissions, forbidden_response, has_permission, json_error_response,
    normalize_limit_offset, service_unavailable_response,
};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use noesis_auth::AuthUser;
use noesis_data::models::subscription::{BillingSubscription, PlanCatalogEntry};
use noesis_data::repositories::billing_repository::{
    ProcessedWebhookEventRow, ReconcileRunRow, SubscriptionStatusCount,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Permission constants
// ---------------------------------------------------------------------------

pub const PERM_BILLING_READ: &str = "admin:billing:read";
pub const PERM_BILLING_SUBSCRIPTIONS_CANCEL: &str = "admin:billing:subscriptions:cancel";
pub const PERM_BILLING_RECONCILE_TRIGGER: &str = "admin:billing:reconcile:trigger";

// ---------------------------------------------------------------------------
// Response shapes
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct AdminBillingOverviewResponse {
    pub status_counts: Vec<StatusCountEntry>,
    pub free_users: i64,
    pub mrr_usd_estimate: f64,
}

#[derive(Serialize)]
pub struct StatusCountEntry {
    pub status: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct AdminSubscriptionItem {
    pub id: String,
    pub user_id: String,
    pub plan_id: String,
    pub provider: String,
    pub provider_customer_id: Option<String>,
    pub provider_subscription_id: Option<String>,
    pub status: String,
    pub cancel_at_period_end: bool,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AdminSubscriptionsResponse {
    pub items: Vec<AdminSubscriptionItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize)]
pub struct AdminSubscriptionDetailResponse {
    pub subscription: AdminSubscriptionItem,
}

#[derive(Serialize)]
pub struct AdminWebhookEventItem {
    pub webhook_id: String,
    pub provider: String,
    pub event_type: String,
    pub processed_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AdminWebhookEventsResponse {
    pub items: Vec<AdminWebhookEventItem>,
    pub limit: i64,
}

#[derive(Serialize)]
pub struct AdminReconcileRunItem {
    pub id: i64,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub force_cancel: bool,
    pub drift_json: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct AdminReconcileDriftResponse {
    pub latest: Option<AdminReconcileRunItem>,
}

#[derive(Serialize)]
pub struct AdminReconcileTriggerResponse {
    /// One-line shell command the operator can run to drive a reconcile.
    /// We deliberately do NOT spawn a job in-process — reconcile is a
    /// separate binary owned by the cron infrastructure (k8s CronJob /
    /// Railway cron). This response keeps the admin UI honest about that
    /// boundary.
    pub command: String,
    pub note: String,
}

#[derive(Serialize)]
pub struct AdminPlanItem {
    pub id: String,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub dodo_product_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AdminPlansResponse {
    pub items: Vec<AdminPlanItem>,
}

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
pub struct ListSubscriptionsQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct ListWebhookEventsQuery {
    pub provider: Option<String>,
    pub limit: Option<i64>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn map_subscription(row: BillingSubscription) -> AdminSubscriptionItem {
    AdminSubscriptionItem {
        id: row.id.to_string(),
        user_id: row.user_id.to_string(),
        plan_id: row.plan_id.to_string(),
        provider: row.provider,
        provider_customer_id: row.provider_customer_id,
        provider_subscription_id: row.provider_subscription_id,
        status: row.status,
        cancel_at_period_end: row.cancel_at_period_end,
        current_period_start: row.current_period_start,
        current_period_end: row.current_period_end,
        canceled_at: row.canceled_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn map_webhook(row: ProcessedWebhookEventRow) -> AdminWebhookEventItem {
    AdminWebhookEventItem {
        webhook_id: row.webhook_id,
        provider: row.provider,
        event_type: row.event_type,
        processed_at: row.processed_at,
    }
}

fn map_reconcile(row: ReconcileRunRow) -> AdminReconcileRunItem {
    AdminReconcileRunItem {
        id: row.id,
        started_at: row.started_at,
        finished_at: row.finished_at,
        force_cancel: row.force_cancel,
        drift_json: row.drift_json,
        error: row.error,
    }
}

fn map_plan(row: PlanCatalogEntry) -> AdminPlanItem {
    AdminPlanItem {
        id: row.id.to_string(),
        code: row.code,
        display_name: row.display_name,
        description: row.description,
        is_active: row.is_active,
        dodo_product_id: row.dodo_product_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

/// Crude MRR estimator: counts active subs by plan code via the loaded plan
/// catalog metadata `monthly_price_usd` field. Returns 0.0 if the field is
/// missing on every plan — the dashboard treats this as "estimate
/// unavailable" and surfaces it accordingly.
fn estimate_mrr(plans: &[PlanCatalogEntry], counts: &[SubscriptionStatusCount]) -> f64 {
    // We want active count per plan, not status; this estimator uses the
    // total active count multiplied by an average price. That's deliberately
    // conservative — a precise MRR needs a per-plan join we can add later.
    let active: f64 = counts
        .iter()
        .filter(|c| c.status == "active")
        .map(|c| c.count as f64)
        .sum();

    let avg_price: f64 = {
        let prices: Vec<f64> = plans
            .iter()
            .filter(|p| p.is_active)
            .filter_map(|p| {
                p.metadata
                    .get("monthly_price_usd")
                    .and_then(|v| v.as_f64())
                    .filter(|&v| v > 0.0)
            })
            .collect();
        if prices.is_empty() {
            0.0
        } else {
            prices.iter().sum::<f64>() / prices.len() as f64
        }
    };

    active * avg_price
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

async fn require_billing_perm(
    state: &AppState,
    auth_user: &AuthUser,
    required: &str,
) -> Result<(), Response> {
    let permissions = match effective_permissions(state, auth_user).await {
        Ok(p) => p,
        Err(_) => {
            return Err(json_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to resolve permissions",
                "PERMISSION_RESOLVE_FAILED",
                None,
            ));
        }
    };
    if !has_permission(&permissions, required) {
        return Err(forbidden_response(required));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Endpoints
// ---------------------------------------------------------------------------

/// GET /api/v1/admin/billing/overview
pub async fn overview(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Response {
    if let Err(resp) = require_billing_perm(&state, &auth_user, PERM_BILLING_READ).await {
        return resp;
    }
    let Some(repo) = state.billing_repository.as_ref() else {
        return service_unavailable_response();
    };

    let counts = match repo.count_subscriptions_by_status().await {
        Ok(rows) => rows,
        Err(e) => {
            return json_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to count subscriptions: {e}"),
                "BILLING_QUERY_FAILED",
                None,
            );
        }
    };
    let free_users = repo.count_free_users().await.unwrap_or(0);
    let plans = repo.list_plan_catalog().await.unwrap_or_default();

    let resp = AdminBillingOverviewResponse {
        mrr_usd_estimate: estimate_mrr(&plans, &counts),
        status_counts: counts
            .into_iter()
            .map(|c| StatusCountEntry {
                status: c.status,
                count: c.count,
            })
            .collect(),
        free_users,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

/// GET /api/v1/admin/billing/subscriptions
pub async fn list_subscriptions(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<ListSubscriptionsQuery>,
) -> Response {
    if let Err(resp) = require_billing_perm(&state, &auth_user, PERM_BILLING_READ).await {
        return resp;
    }
    let Some(repo) = state.billing_repository.as_ref() else {
        return service_unavailable_response();
    };

    let (limit, offset) = normalize_limit_offset(q.limit, q.offset, 50, 200);
    let (rows, total) = match repo
        .list_subscriptions_paginated(q.status.as_deref(), limit, offset)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            return json_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to list subscriptions: {e}"),
                "BILLING_QUERY_FAILED",
                None,
            );
        }
    };

    let resp = AdminSubscriptionsResponse {
        items: rows.into_iter().map(map_subscription).collect(),
        total,
        limit,
        offset,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

/// GET /api/v1/admin/billing/subscriptions/:id
pub async fn get_subscription(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Response {
    if let Err(resp) = require_billing_perm(&state, &auth_user, PERM_BILLING_READ).await {
        return resp;
    }
    let Some(repo) = state.billing_repository.as_ref() else {
        return service_unavailable_response();
    };

    let uuid = match parse_uuid_or_422(&id, "subscription_id") {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    match repo.find_subscription_by_id(uuid).await {
        Ok(Some(row)) => (
            StatusCode::OK,
            Json(AdminSubscriptionDetailResponse {
                subscription: map_subscription(row),
            }),
        )
            .into_response(),
        Ok(None) => json_error_response(
            StatusCode::NOT_FOUND,
            "Subscription not found",
            "SUBSCRIPTION_NOT_FOUND",
            None,
        ),
        Err(e) => json_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch subscription: {e}"),
            "BILLING_QUERY_FAILED",
            None,
        ),
    }
}

/// POST /api/v1/admin/billing/subscriptions/:id/cancel
///
/// Hard-cancels a subscription locally. Useful when Dodo's cancel webhook
/// was missed and reconcile is in read-only mode. The Dodo dashboard
/// should be cancelled separately by the operator — this endpoint only
/// fixes local state.
pub async fn cancel_subscription(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Response {
    if let Err(resp) =
        require_billing_perm(&state, &auth_user, PERM_BILLING_SUBSCRIPTIONS_CANCEL).await
    {
        return resp;
    }
    let Some(repo) = state.billing_repository.as_ref() else {
        return service_unavailable_response();
    };

    let uuid = match parse_uuid_or_422(&id, "subscription_id") {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let row = match repo.find_subscription_by_id(uuid).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return json_error_response(
                StatusCode::NOT_FOUND,
                "Subscription not found",
                "SUBSCRIPTION_NOT_FOUND",
                None,
            );
        }
        Err(e) => {
            return json_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch subscription: {e}"),
                "BILLING_QUERY_FAILED",
                None,
            );
        }
    };

    let provider_sub_id = match row.provider_subscription_id.as_deref() {
        Some(s) if !s.is_empty() => s,
        _ => {
            return json_error_response(
                StatusCode::CONFLICT,
                "Subscription has no provider_subscription_id; cannot force-cancel",
                "INVALID_STATE",
                None,
            );
        }
    };

    match repo.force_cancel_subscription(provider_sub_id).await {
        Ok(Some(_updated)) => {
            // Mirror tier to free.
            let _ = repo.set_user_tier(row.user_id, "free").await;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "ok": true,
                    "subscription_id": id,
                })),
            )
                .into_response()
        }
        Ok(None) => json_error_response(
            StatusCode::NOT_FOUND,
            "Subscription disappeared during cancel",
            "SUBSCRIPTION_NOT_FOUND",
            None,
        ),
        Err(e) => json_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cancel failed: {e}"),
            "BILLING_QUERY_FAILED",
            None,
        ),
    }
}

/// GET /api/v1/admin/billing/webhook-events
pub async fn list_webhook_events(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<ListWebhookEventsQuery>,
) -> Response {
    if let Err(resp) = require_billing_perm(&state, &auth_user, PERM_BILLING_READ).await {
        return resp;
    }
    let Some(repo) = state.billing_repository.as_ref() else {
        return service_unavailable_response();
    };

    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    match repo
        .list_processed_webhook_events(q.provider.as_deref(), limit)
        .await
    {
        Ok(rows) => (
            StatusCode::OK,
            Json(AdminWebhookEventsResponse {
                items: rows.into_iter().map(map_webhook).collect(),
                limit,
            }),
        )
            .into_response(),
        Err(e) => json_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list webhook events: {e}"),
            "BILLING_QUERY_FAILED",
            None,
        ),
    }
}

/// GET /api/v1/admin/billing/reconcile/drift
pub async fn reconcile_drift(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Response {
    if let Err(resp) = require_billing_perm(&state, &auth_user, PERM_BILLING_READ).await {
        return resp;
    }
    let Some(repo) = state.billing_repository.as_ref() else {
        return service_unavailable_response();
    };

    match repo.latest_reconcile_run().await {
        Ok(latest) => (
            StatusCode::OK,
            Json(AdminReconcileDriftResponse {
                latest: latest.map(map_reconcile),
            }),
        )
            .into_response(),
        Err(e) => json_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch reconcile run: {e}"),
            "BILLING_QUERY_FAILED",
            None,
        ),
    }
}

/// POST /api/v1/admin/billing/reconcile/run
pub async fn reconcile_trigger(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Response {
    if let Err(resp) =
        require_billing_perm(&state, &auth_user, PERM_BILLING_RECONCILE_TRIGGER).await
    {
        return resp;
    }
    // No actual job spawn — see struct doc.
    let resp = AdminReconcileTriggerResponse {
        command: "kubectl create job --from=cronjob/dodo-reconcile dodo-reconcile-manual-$(date +%s)".to_string(),
        note: "Reconcile is a separate binary; run the command above (or the Railway/systemd equivalent). The next finished run will be visible at /admin/billing/reconcile/drift.".to_string(),
    };
    (StatusCode::ACCEPTED, Json(resp)).into_response()
}

/// GET /api/v1/admin/billing/plans
pub async fn list_plans(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Response {
    if let Err(resp) = require_billing_perm(&state, &auth_user, PERM_BILLING_READ).await {
        return resp;
    }
    let Some(repo) = state.billing_repository.as_ref() else {
        return service_unavailable_response();
    };

    match repo.list_plan_catalog().await {
        Ok(rows) => (
            StatusCode::OK,
            Json(AdminPlansResponse {
                items: rows.into_iter().map(map_plan).collect(),
            }),
        )
            .into_response(),
        Err(e) => json_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list plans: {e}"),
            "BILLING_QUERY_FAILED",
            None,
        ),
    }
}
