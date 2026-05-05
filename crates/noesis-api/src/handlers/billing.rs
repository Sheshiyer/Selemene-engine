//! Internal billing webhook ingest endpoint.
//!
//! Authenticated by shared secret in `X-Forward-Secret`. Called by the
//! Next.js webhook adaptor (T11) after Standard Webhooks signature
//! verification. Idempotent via `processed_webhook_events` PK constraint.
//!
//! Contract: `.context/billing/contracts.md` § API.

use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Extension,
};
use chrono::{DateTime, Utc};
use noesis_auth::AuthUser;
use noesis_data::repositories::billing_repository::{BillingRepository, PROVIDER_DODO};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::billing::{BillingForwardRequest, BillingForwardResponse, DodoInboundEventType};
use crate::AppState;

const SHARED_SECRET_HEADER: &str = "x-forward-secret";

/// Maximum age (seconds) accepted for webhook_timestamp before we reject as
/// stale. Mirrors the Standard Webhooks library default. Defense-in-depth on
/// top of Next.js's signature verification — anyone replaying old payloads
/// to /internal/billing/events with a leaked forward secret hits this gate.
const WEBHOOK_MAX_AGE_SECS: i64 = 300;

pub async fn events_forward(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(envelope): Json<BillingForwardRequest>,
) -> impl IntoResponse {
    // -- 1. Auth (shared secret) --
    if !shared_secret_ok(&headers) {
        noesis_metrics::record_dodo_webhook("unknown", "unauthorized");
        return (StatusCode::UNAUTHORIZED, "forbidden").into_response();
    }

    // -- 2. Freshness — reject replayed/old envelopes --
    if let Err(err_msg) = validate_timestamp_freshness(&envelope.webhook_timestamp) {
        tracing::warn!(
            webhook_id = %envelope.webhook_id,
            timestamp = %envelope.webhook_timestamp,
            error = %err_msg,
            "rejecting stale or unparseable webhook timestamp"
        );
        let event_type_label = serde_json::to_value(envelope.event_type)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string());
        noesis_metrics::record_dodo_webhook(&event_type_label, "stale");
        return (StatusCode::BAD_REQUEST, format!("stale timestamp: {}", err_msg))
            .into_response();
    }

    let Some(repo) = state.billing_repository.clone() else {
        tracing::error!("billing webhook received but billing_repository unavailable (db not configured)");
        return (StatusCode::SERVICE_UNAVAILABLE, "billing not configured").into_response();
    };

    let event_type_str = serde_json::to_value(envelope.event_type)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| format!("{:?}", envelope.event_type));

    // -- 2. Idempotency --
    match repo
        .try_record_webhook_event(&envelope.webhook_id, PROVIDER_DODO, &event_type_str)
        .await
    {
        Ok(false) => {
            tracing::info!(
                webhook_id = %envelope.webhook_id,
                event_type = %event_type_str,
                "duplicate webhook delivery — already processed"
            );
            noesis_metrics::record_dodo_webhook(&event_type_str, "dedup");
            return (StatusCode::OK, Json(BillingForwardResponse::Dedup)).into_response();
        }
        Ok(true) => {
            tracing::info!(
                webhook_id = %envelope.webhook_id,
                event_type = %event_type_str,
                "processing webhook"
            );
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to record webhook event for idempotency");
            return (StatusCode::INTERNAL_SERVER_ERROR, "idempotency check failed")
                .into_response();
        }
    }

    // -- 3. Dispatch --
    let result = match envelope.event_type {
        DodoInboundEventType::SubscriptionActive => {
            handle_subscription_active(&repo, &envelope.payload).await
        }
        DodoInboundEventType::SubscriptionUpdated => {
            handle_subscription_updated(&repo, &envelope.payload).await
        }
        DodoInboundEventType::SubscriptionOnHold => {
            handle_subscription_on_hold(&repo, &envelope.payload).await
        }
        DodoInboundEventType::SubscriptionCancelled => {
            handle_subscription_cancelled(&repo, &envelope.payload).await
        }
        DodoInboundEventType::SubscriptionFailed
        | DodoInboundEventType::PaymentSucceeded
        | DodoInboundEventType::PaymentFailed
        | DodoInboundEventType::CreditAdded
        | DodoInboundEventType::CreditDeducted
        | DodoInboundEventType::CreditBalanceLow
        | DodoInboundEventType::CreditOverageCharged => {
            // Log-only events for v1. Cache invalidation for credit.* lands
            // in T17 alongside the balance proxy.
            tracing::info!(
                event_type = %event_type_str,
                "log-only event acknowledged"
            );
            Ok(())
        }
    };

    match result {
        Ok(()) => {
            noesis_metrics::record_dodo_webhook(&event_type_str, "ok");
            (StatusCode::OK, Json(BillingForwardResponse::Ok)).into_response()
        }
        Err(HandlerError::MissingField(path)) => {
            noesis_metrics::record_dodo_webhook(&event_type_str, "bad_request");
            tracing::warn!(path = path, "missing field in webhook payload");
            (StatusCode::UNPROCESSABLE_ENTITY, format!("missing field: {}", path)).into_response()
        }
        Err(HandlerError::UnknownProduct(product_id)) => {
            tracing::error!(
                product_id = %product_id,
                "subscription references a product_id with no plan_catalog mapping — run the post-provisioning UPDATE on plan_catalog"
            );
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                "unknown product_id — plan_catalog not yet wired",
            )
                .into_response()
        }
        Err(HandlerError::UnknownUser(detail)) => {
            tracing::error!(detail = %detail, "could not resolve subscription to a user");
            (StatusCode::UNPROCESSABLE_ENTITY, format!("unknown user: {}", detail))
                .into_response()
        }
        Err(HandlerError::Db(e)) => {
            tracing::error!(error = %e, "database error during webhook dispatch");
            (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

/// Validates that `webhook_timestamp` (Standard Webhooks unix-seconds string)
/// is within `WEBHOOK_MAX_AGE_SECS` of now. Accepts mild future skew (+30s)
/// to tolerate clock drift between Dodo and our servers. Returns Err with
/// a short human reason on failure.
fn validate_timestamp_freshness(ts: &str) -> Result<(), String> {
    let parsed: i64 = ts
        .parse()
        .map_err(|_| format!("not a unix-seconds integer ('{}')", ts))?;
    let now = chrono::Utc::now().timestamp();
    let age = now - parsed;
    if age > WEBHOOK_MAX_AGE_SECS {
        return Err(format!("{}s old (max {}s)", age, WEBHOOK_MAX_AGE_SECS));
    }
    if age < -30 {
        return Err(format!("{}s in future — clock skew suspected", -age));
    }
    Ok(())
}

fn shared_secret_ok(headers: &HeaderMap) -> bool {
    let provided = headers
        .get(SHARED_SECRET_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let expected = std::env::var("DODO_INTERNAL_FORWARD_SECRET").unwrap_or_default();
    if expected.is_empty() || provided.is_empty() {
        return false;
    }
    constant_time_eq(provided.as_bytes(), expected.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ---------------------------------------------------------------------------
// Payload extraction
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum HandlerError {
    MissingField(&'static str),
    UnknownProduct(String),
    UnknownUser(String),
    Db(sqlx::Error),
}

impl From<sqlx::Error> for HandlerError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}

fn extract_str<'a>(payload: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut cur = payload;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_str()
}

fn extract_bool(payload: &Value, path: &[&str]) -> Option<bool> {
    let mut cur = payload;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_bool()
}

fn extract_datetime(payload: &Value, path: &[&str]) -> Option<DateTime<Utc>> {
    let s = extract_str(payload, path)?;
    DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))
}

/// Customer ID comes either as `data.customer.customer_id` (rich object) or
/// `data.customer_id` (flat string). We accept both.
fn extract_customer_id(payload: &Value) -> Option<&str> {
    extract_str(payload, &["data", "customer", "customer_id"])
        .or_else(|| extract_str(payload, &["data", "customer_id"]))
}

fn extract_product_id(payload: &Value) -> Option<&str> {
    extract_str(payload, &["data", "product_id"])
        .or_else(|| extract_str(payload, &["data", "product", "product_id"]))
}

fn extract_subscription_id(payload: &Value) -> Option<&str> {
    extract_str(payload, &["data", "subscription_id"])
}

fn extract_selemene_user_id(payload: &Value) -> Option<Uuid> {
    let s = extract_str(payload, &["data", "metadata", "selemene_user_id"])?;
    Uuid::parse_str(s).ok()
}

// ---------------------------------------------------------------------------
// subscription.active — first-payment landed; create/refresh row, set tier
// ---------------------------------------------------------------------------

async fn handle_subscription_active(
    repo: &Arc<BillingRepository>,
    payload: &Value,
) -> Result<(), HandlerError> {
    let subscription_id = extract_subscription_id(payload)
        .ok_or(HandlerError::MissingField("data.subscription_id"))?;
    let customer_id = extract_customer_id(payload)
        .ok_or(HandlerError::MissingField("data.customer.customer_id"))?;
    let product_id = extract_product_id(payload)
        .ok_or(HandlerError::MissingField("data.product_id"))?;

    let plan = repo
        .find_plan_by_dodo_product_id(product_id)
        .await?
        .ok_or_else(|| HandlerError::UnknownProduct(product_id.to_string()))?;

    // Resolve the user. Prefer metadata.selemene_user_id (set by checkout
    // route in T12). Fall back to existing dodo_customer_id mapping for users
    // who already had a Dodo customer (e.g. re-subscriptions).
    let user_id = if let Some(uid) = extract_selemene_user_id(payload) {
        uid
    } else if let Some(uid) = repo.find_user_id_by_dodo_customer_id(customer_id).await? {
        uid
    } else {
        return Err(HandlerError::UnknownUser(format!(
            "no metadata.selemene_user_id and customer {customer_id} not yet mapped"
        )));
    };

    repo.set_user_dodo_customer_id(user_id, customer_id).await?;

    let cancel_at_period_end =
        extract_bool(payload, &["data", "cancel_at_period_end"]).unwrap_or(false);
    let current_period_start = extract_datetime(payload, &["data", "current_period_start"]);
    let current_period_end = extract_datetime(payload, &["data", "current_period_end"]);

    let metadata = serde_json::json!({
        "last_event_type": "subscription.active",
        "last_payload_data": payload.get("data").cloned().unwrap_or(Value::Null),
    });

    repo.upsert_subscription(
        user_id,
        plan.id,
        customer_id,
        subscription_id,
        "active",
        cancel_at_period_end,
        current_period_start,
        current_period_end,
        metadata,
    )
    .await?;

    repo.set_user_tier(user_id, &plan.code).await?;

    tracing::info!(
        user_id = %user_id,
        plan = %plan.code,
        subscription_id = subscription_id,
        "subscription activated"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// subscription.updated — re-sync from current payload
// ---------------------------------------------------------------------------

async fn handle_subscription_updated(
    repo: &Arc<BillingRepository>,
    payload: &Value,
) -> Result<(), HandlerError> {
    let subscription_id = extract_subscription_id(payload)
        .ok_or(HandlerError::MissingField("data.subscription_id"))?;
    let customer_id = extract_customer_id(payload)
        .ok_or(HandlerError::MissingField("data.customer.customer_id"))?;
    let product_id = extract_product_id(payload)
        .ok_or(HandlerError::MissingField("data.product_id"))?;
    let status = extract_str(payload, &["data", "status"])
        .ok_or(HandlerError::MissingField("data.status"))?;

    let plan = repo
        .find_plan_by_dodo_product_id(product_id)
        .await?
        .ok_or_else(|| HandlerError::UnknownProduct(product_id.to_string()))?;

    let user_id = repo
        .find_user_id_by_dodo_customer_id(customer_id)
        .await?
        .ok_or_else(|| HandlerError::UnknownUser(format!("customer {customer_id} not mapped")))?;

    let cancel_at_period_end =
        extract_bool(payload, &["data", "cancel_at_period_end"]).unwrap_or(false);
    let current_period_start = extract_datetime(payload, &["data", "current_period_start"]);
    let current_period_end = extract_datetime(payload, &["data", "current_period_end"]);

    let metadata = serde_json::json!({
        "last_event_type": "subscription.updated",
        "last_payload_data": payload.get("data").cloned().unwrap_or(Value::Null),
    });

    repo.upsert_subscription(
        user_id,
        plan.id,
        customer_id,
        subscription_id,
        status,
        cancel_at_period_end,
        current_period_start,
        current_period_end,
        metadata,
    )
    .await?;

    // Mirror the (possibly new) plan onto users.tier whenever the active
    // subscription state changes. Past_due and active both keep tier set;
    // canceled handled by the cancel branch.
    if matches!(status, "active" | "trialing" | "past_due") {
        repo.set_user_tier(user_id, &plan.code).await?;
    }

    tracing::info!(
        user_id = %user_id,
        plan = %plan.code,
        subscription_id = subscription_id,
        status = status,
        "subscription updated"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// subscription.on_hold — renewal failure, in dunning window
// ---------------------------------------------------------------------------

async fn handle_subscription_on_hold(
    repo: &Arc<BillingRepository>,
    payload: &Value,
) -> Result<(), HandlerError> {
    let subscription_id = extract_subscription_id(payload)
        .ok_or(HandlerError::MissingField("data.subscription_id"))?;

    let updated = repo.set_subscription_past_due(subscription_id).await?;
    if updated.is_none() {
        tracing::warn!(
            subscription_id = subscription_id,
            "on_hold for unknown subscription — ignoring"
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// subscription.cancelled
// ---------------------------------------------------------------------------

async fn handle_subscription_cancelled(
    repo: &Arc<BillingRepository>,
    payload: &Value,
) -> Result<(), HandlerError> {
    let subscription_id = extract_subscription_id(payload)
        .ok_or(HandlerError::MissingField("data.subscription_id"))?;
    let cancel_at_period_end =
        extract_bool(payload, &["data", "cancel_at_period_end"]).unwrap_or(false);

    let cancelled = repo
        .cancel_subscription(subscription_id, cancel_at_period_end)
        .await?;

    if let Some(sub) = cancelled {
        // Immediate cancellations downgrade the user to free now. Cancel-at-
        // period-end leaves tier intact until the period boundary; that's the
        // point of the "cancel later" UX.
        if !cancel_at_period_end {
            repo.set_user_tier(sub.user_id, "free").await?;
        }
        tracing::info!(
            subscription_id = subscription_id,
            cancel_at_period_end = cancel_at_period_end,
            "subscription cancelled"
        );
    } else {
        tracing::warn!(
            subscription_id = subscription_id,
            "cancellation for unknown subscription — ignoring"
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Checkout creation — POST /api/v1/billing/checkout (JWT-authenticated)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateCheckoutRequest {
    pub plan_code: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCheckoutResponse {
    pub checkout_url: String,
}

pub async fn create_checkout_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CreateCheckoutRequest>,
) -> impl IntoResponse {
    let Some(repo) = state.billing_repository.clone() else {
        return (StatusCode::SERVICE_UNAVAILABLE, "billing not configured").into_response();
    };

    let api_key = match std::env::var("DODO_PAYMENTS_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "DODO_PAYMENTS_API_KEY not set",
            )
                .into_response();
        }
    };

    let dodo_env = std::env::var("DODO_PAYMENTS_ENV").unwrap_or_else(|_| "test".to_string());
    let api_base = if dodo_env == "live" {
        "https://live.dodopayments.com"
    } else {
        "https://test.dodopayments.com"
    };
    let return_url = std::env::var("DODO_PAYMENTS_RETURN_URL")
        .unwrap_or_else(|_| "http://localhost:3000/billing?status=success".to_string());

    let plan = match repo.find_plan_by_code(&body.plan_code).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                format!("unknown plan_code: {}", body.plan_code),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "find_plan_by_code failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
    };

    let Some(product_id) = plan.dodo_product_id.as_deref() else {
        return (
            StatusCode::FAILED_DEPENDENCY,
            format!(
                "plan_catalog row for '{}' has no dodo_product_id — run the post-provisioning UPDATE",
                plan.code
            ),
        )
            .into_response();
    };

    let user_id = match Uuid::parse_str(&auth_user.user_id) {
        Ok(u) => u,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, "invalid user_id in token").into_response();
        }
    };

    let (email, _existing_customer_id) = match repo.get_user_for_checkout(user_id).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, "user not found").into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "get_user_for_checkout failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
    };

    let request_body = serde_json::json!({
        "product_cart": [{ "product_id": product_id, "quantity": 1 }],
        "customer": { "email": email },
        "return_url": return_url,
        "metadata": { "selemene_user_id": user_id.to_string() },
    });

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "reqwest client build failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "http client error").into_response();
        }
    };

    let resp = match client
        .post(format!("{}/checkouts", api_base))
        .bearer_auth(&api_key)
        .json(&request_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Dodo checkout POST failed");
            return (StatusCode::BAD_GATEWAY, "upstream unreachable").into_response();
        }
    };

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        tracing::error!(
            status = %status,
            body = %body_text,
            "Dodo checkout returned non-success"
        );
        return (StatusCode::BAD_GATEWAY, format!("dodo {}: {}", status, body_text))
            .into_response();
    }

    let parsed: serde_json::Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, body = %body_text, "Dodo response not JSON");
            return (StatusCode::BAD_GATEWAY, "malformed dodo response").into_response();
        }
    };

    let checkout_url = parsed
        .get("checkout_url")
        .and_then(|v| v.as_str())
        .map(String::from);

    let Some(checkout_url) = checkout_url else {
        tracing::error!(body = %body_text, "checkout_url missing in Dodo response");
        return (StatusCode::BAD_GATEWAY, "no checkout_url").into_response();
    };

    (StatusCode::OK, Json(CreateCheckoutResponse { checkout_url })).into_response()
}

// ---------------------------------------------------------------------------
// Customer portal — POST /api/v1/billing/portal (JWT-authenticated)
// ---------------------------------------------------------------------------
//
// Returns a Dodo-hosted customer-portal URL where the user can manage
// their subscription, payment method, and download invoices. Requires
// the user to have an existing dodo_customer_id (404 otherwise — they
// have never checked out).

#[derive(Debug, Serialize)]
pub struct CreatePortalResponse {
    pub portal_url: String,
}

pub async fn create_portal_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    let Some(repo) = state.billing_repository.clone() else {
        return (StatusCode::SERVICE_UNAVAILABLE, "billing not configured").into_response();
    };

    let api_key = match std::env::var("DODO_PAYMENTS_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "DODO_PAYMENTS_API_KEY not set",
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&auth_user.user_id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::UNAUTHORIZED, "invalid user_id").into_response(),
    };

    let customer_id = match repo.find_user_dodo_customer_id(user_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                "no Dodo customer for this user — complete a checkout first",
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "find_user_dodo_customer_id failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
    };

    let dodo_env = std::env::var("DODO_PAYMENTS_ENV").unwrap_or_else(|_| "test".to_string());
    let api_base = if dodo_env == "live" {
        "https://live.dodopayments.com"
    } else {
        "https://test.dodopayments.com"
    };
    let url = format!(
        "{}/customers/{}/customer-portal/session",
        api_base, customer_id
    );

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "reqwest build failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "http client error").into_response();
        }
    };

    let resp = match client.post(&url).bearer_auth(&api_key).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Dodo portal POST failed");
            return (StatusCode::BAD_GATEWAY, "upstream unreachable").into_response();
        }
    };

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        tracing::error!(status = %status, body = %body_text, "Dodo portal non-success");
        return (StatusCode::BAD_GATEWAY, format!("dodo {}: {}", status, body_text))
            .into_response();
    }

    let parsed: Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_GATEWAY, "malformed dodo response").into_response(),
    };
    let portal_url = parsed
        .get("link")
        .and_then(|v| v.as_str())
        .map(String::from);
    let Some(portal_url) = portal_url else {
        tracing::error!(body = %body_text, "link missing in Dodo portal response");
        return (StatusCode::BAD_GATEWAY, "no portal link").into_response();
    };

    (StatusCode::OK, Json(CreatePortalResponse { portal_url })).into_response()
}

// ---------------------------------------------------------------------------
// Balance proxy — GET /api/v1/billing/balance (JWT-authenticated)
// ---------------------------------------------------------------------------
//
// Combines local subscription state (period_end, tier) with Dodo's customer
// credit balance. Free-tier users (no dodo_customer_id) get a tier-default
// response without touching Dodo.

const FREE_TIER_DEFAULT_CREDITS: u64 = 50;

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub credits_remaining: u64,
    /// Overage credits charged so far this period (string from Dodo to
    /// preserve precision; 0 for free tier).
    pub overage_charged: String,
    pub period_end: Option<DateTime<Utc>>,
    pub tier: String,
    pub cancel_at_period_end: bool,
    /// Where the response data came from.
    pub source: BalanceSource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BalanceSource {
    Dodo,
    TierDefault,
}

pub async fn get_balance(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    let Some(repo) = state.billing_repository.clone() else {
        return (StatusCode::SERVICE_UNAVAILABLE, "billing not configured").into_response();
    };

    let user_id = match Uuid::parse_str(&auth_user.user_id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::UNAUTHORIZED, "invalid user_id").into_response(),
    };

    // Pull the active plan resolution (tier, period_end, customer_id). If
    // there is no active row the user is on the free tier.
    let active = match repo.find_active_plan_resolution(user_id).await {
        Ok(a) => a,
        Err(e) => {
            tracing::error!(error = %e, "find_active_plan_resolution failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
    };

    let Some(active) = active else {
        // No active subscription → free tier defaults
        return (
            StatusCode::OK,
            Json(BalanceResponse {
                credits_remaining: FREE_TIER_DEFAULT_CREDITS,
                overage_charged: "0".to_string(),
                period_end: None,
                tier: "free".to_string(),
                cancel_at_period_end: false,
                source: BalanceSource::TierDefault,
            }),
        )
            .into_response();
    };

    let Some(customer_id) = active.provider_customer_id.as_deref() else {
        // Active row exists but no customer_id (legacy backfill row from
        // migration 014). Treat as tier_default for this tier.
        return (
            StatusCode::OK,
            Json(BalanceResponse {
                credits_remaining: FREE_TIER_DEFAULT_CREDITS,
                overage_charged: "0".to_string(),
                period_end: active.current_period_end,
                tier: active.plan_code,
                cancel_at_period_end: active.cancel_at_period_end,
                source: BalanceSource::TierDefault,
            }),
        )
            .into_response();
    };

    let entitlement_id = std::env::var("DODO_ENTITLEMENT_WITNESS_CREDITS_ID").unwrap_or_default();
    let api_key = std::env::var("DODO_PAYMENTS_API_KEY").unwrap_or_default();
    if entitlement_id.is_empty() || api_key.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "DODO_ENTITLEMENT_WITNESS_CREDITS_ID or DODO_PAYMENTS_API_KEY not set",
        )
            .into_response();
    }
    let dodo_env = std::env::var("DODO_PAYMENTS_ENV").unwrap_or_else(|_| "test".to_string());
    let api_base = if dodo_env == "live" {
        "https://live.dodopayments.com"
    } else {
        "https://test.dodopayments.com"
    };

    let url = format!(
        "{}/credit-entitlements/{}/balances/{}",
        api_base, entitlement_id, customer_id
    );
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "reqwest build failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "http client error").into_response();
        }
    };

    let resp = match client.get(&url).bearer_auth(&api_key).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Dodo balance fetch failed");
            return (StatusCode::BAD_GATEWAY, "upstream unreachable").into_response();
        }
    };

    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        // Customer never had this entitlement — return tier_default.
        return (
            StatusCode::OK,
            Json(BalanceResponse {
                credits_remaining: FREE_TIER_DEFAULT_CREDITS,
                overage_charged: "0".to_string(),
                period_end: active.current_period_end,
                tier: active.plan_code,
                cancel_at_period_end: active.cancel_at_period_end,
                source: BalanceSource::TierDefault,
            }),
        )
            .into_response();
    }
    let body_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        tracing::error!(status = %status, body = %body_text, "Dodo balance non-success");
        return (StatusCode::BAD_GATEWAY, format!("dodo {}: {}", status, body_text))
            .into_response();
    }

    let parsed: Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(_) => {
            return (StatusCode::BAD_GATEWAY, "malformed dodo response").into_response();
        }
    };
    let balance_str = parsed
        .get("balance")
        .and_then(|v| v.as_str())
        .unwrap_or("0");
    let credits_remaining = balance_str.parse::<u64>().unwrap_or(0);
    let overage_charged = parsed
        .get("overage")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();

    (
        StatusCode::OK,
        Json(BalanceResponse {
            credits_remaining,
            overage_charged,
            period_end: active.current_period_end,
            tier: active.plan_code,
            cancel_at_period_end: active.cancel_at_period_end,
            source: BalanceSource::Dodo,
        }),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Tests — exercise extraction + dispatch routing without needing a DB.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_active_payload() -> Value {
        json!({
            "type": "subscription.active",
            "data": {
                "subscription_id": "sub_test123",
                "product_id": "pdt_basic",
                "status": "active",
                "cancel_at_period_end": false,
                "current_period_start": "2026-05-01T00:00:00Z",
                "current_period_end":   "2026-06-01T00:00:00Z",
                "customer": { "customer_id": "cus_abc", "email": "u@example.com" },
                "metadata": { "selemene_user_id": "11111111-1111-1111-1111-111111111111" }
            }
        })
    }

    #[test]
    fn shared_secret_header_lowercase() {
        assert_eq!(SHARED_SECRET_HEADER, "x-forward-secret");
    }

    #[test]
    fn constant_time_eq_matches_only_when_equal() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }

    #[test]
    fn extract_subscription_id_from_active() {
        let p = sample_active_payload();
        assert_eq!(extract_subscription_id(&p), Some("sub_test123"));
    }

    #[test]
    fn extract_customer_id_handles_rich_and_flat_shapes() {
        let rich = json!({"data": {"customer": {"customer_id": "cus_1"}}});
        assert_eq!(extract_customer_id(&rich), Some("cus_1"));
        let flat = json!({"data": {"customer_id": "cus_2"}});
        assert_eq!(extract_customer_id(&flat), Some("cus_2"));
        let missing = json!({"data": {}});
        assert_eq!(extract_customer_id(&missing), None);
    }

    #[test]
    fn extract_product_id_handles_both_shapes() {
        assert_eq!(
            extract_product_id(&json!({"data": {"product_id": "pdt_x"}})),
            Some("pdt_x")
        );
        assert_eq!(
            extract_product_id(&json!({"data": {"product": {"product_id": "pdt_y"}}})),
            Some("pdt_y")
        );
        assert_eq!(extract_product_id(&json!({"data": {}})), None);
    }

    #[test]
    fn extract_selemene_user_id_parses_uuid() {
        let p = sample_active_payload();
        assert_eq!(
            extract_selemene_user_id(&p),
            Some(Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap())
        );
    }

    #[test]
    fn extract_selemene_user_id_rejects_non_uuid() {
        let p = json!({"data": {"metadata": {"selemene_user_id": "not-a-uuid"}}});
        assert_eq!(extract_selemene_user_id(&p), None);
    }

    #[test]
    fn timestamp_freshness_accepts_now() {
        let now = chrono::Utc::now().timestamp().to_string();
        assert!(validate_timestamp_freshness(&now).is_ok());
    }

    #[test]
    fn timestamp_freshness_rejects_old() {
        let old = (chrono::Utc::now().timestamp() - WEBHOOK_MAX_AGE_SECS - 60).to_string();
        let err = validate_timestamp_freshness(&old).unwrap_err();
        assert!(err.contains("old"), "expected 'old' in error: {}", err);
    }

    #[test]
    fn timestamp_freshness_rejects_far_future() {
        let future = (chrono::Utc::now().timestamp() + 3600).to_string();
        let err = validate_timestamp_freshness(&future).unwrap_err();
        assert!(err.contains("future"), "expected 'future' in error: {}", err);
    }

    #[test]
    fn timestamp_freshness_tolerates_small_skew() {
        // +20s in future — within ±30s tolerance band
        let near_future = (chrono::Utc::now().timestamp() + 20).to_string();
        assert!(validate_timestamp_freshness(&near_future).is_ok());
    }

    #[test]
    fn timestamp_freshness_rejects_garbage() {
        assert!(validate_timestamp_freshness("not-a-number").is_err());
        assert!(validate_timestamp_freshness("").is_err());
    }

    #[test]
    fn extract_datetime_parses_rfc3339() {
        let p = sample_active_payload();
        let dt = extract_datetime(&p, &["data", "current_period_end"]).unwrap();
        assert_eq!(dt.to_rfc3339(), "2026-06-01T00:00:00+00:00");
    }
}
