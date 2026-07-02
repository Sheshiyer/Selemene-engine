//! Billing webhook end-to-end tests.
//!
//! Drives the full inbound pipeline against a real Postgres + the live
//! `/internal/billing/events` route:
//!   POST envelope → idempotency check → dispatch → DB mutation
//!
//! Requires:
//!   • Postgres reachable at $DATABASE_URL (defaults to localhost:5432 noesis)
//!   • Migrations 001..022 applied
//!   • plan_catalog wired with a Dodo product ID for the `basic` code
//!     (UPDATE plan_catalog SET dodo_product_id='pdt_…' WHERE code='basic')
//!
//! These tests are #[serial] because they share rows in the local DB, but
//! each test seeds a unique random UUID user so cross-test collision is
//! impossible. Cleanup via CASCADE delete on user_id.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use noesis_api::{create_router, shared_metrics, ApiConfig, AppState};
use noesis_auth::AuthService;
use noesis_cache::CacheManager;
use noesis_data::repositories::{
    billing_repository::BillingRepository, user_repository::UserRepository,
};
use noesis_orchestrator::WorkflowOrchestrator;
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceExt;
use uuid::Uuid;

const FORWARD_SECRET: &str = "billing-e2e-forward-secret-very-long-static";
const E2E_BASIC_DODO_PRODUCT_ID: &str = "pdt_e2e_basic";

fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://noesis_user:noesis_password@localhost:5432/noesis".to_string()
    })
}

async fn build_state_and_router() -> (AppState, axum::Router, PgPool) {
    // Reset the forward secret to a known value so the auth check passes.
    std::env::set_var("DODO_INTERNAL_FORWARD_SECRET", FORWARD_SECRET);

    let url = database_url();
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect to local Postgres for E2E");
    ensure_basic_plan_dodo_product_id(&pool).await;

    let cache = CacheManager::new(String::new(), 100, Duration::from_secs(60), false);
    let auth = AuthService::new("e2e-jwt-secret-must-be-at-least-32-chars-long-xyz".into());
    let user_repository = Arc::new(UserRepository::new(pool.clone()));
    let billing_repository = Some(Arc::new(BillingRepository::new(pool.clone())));
    let metrics = shared_metrics();

    let config = ApiConfig {
        host: "127.0.0.1".into(),
        port: 0,
        jwt_secret: "e2e-jwt-secret-must-be-at-least-32-chars-long-xyz".into(),
        database_url: Some(url.clone()),
        redis_url: None,
        allowed_origins: vec![],
        rate_limit_requests: 1000,
        rate_limit_window_secs: 60,
        request_timeout_secs: 30,
        log_level: "info".into(),
        log_format: "pretty".into(),
        discord_client_id: None,
        discord_client_secret: None,
        discord_redirect_uri: None,
        cf_access_issuer: None,
        cf_access_audience: None,
        cf_dev_bypass_token: None,
        dodo_payments_api_key: None,
        dodo_payments_webhook_key: None,
        dodo_payments_env: None,
        python_biofield_url: "http://localhost:8002".into(),
        python_biofield_timeout_ms: 10_000,
        gateway_url: None,
        gateway_token: None,
    };

    let state = AppState {
        orchestrator: Arc::new(WorkflowOrchestrator::new()),
        bridge_manager: Arc::new(noesis_bridge::BridgeManager::from_env()),
        cache: Arc::new(cache),
        auth: Arc::new(auth),
        metrics,
        user_repository,
        admin_repository: None,
        billing_repository,
        biofield_repository: None,
        readings_repository: None,
        usage_repository: None,
        db_available: true,
        discord_client_id: None,
        discord_client_secret: None,
        discord_redirect_uri: None,
        cf_access_validator: None,
        cf_dev_bypass_token: None,
        startup_time: Instant::now(),
        ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
    };

    let router = create_router(state.clone(), &config);
    (state, router, pool)
}

/// Insert a fresh test user with a random UUID. Returns the user_id + email.
async fn seed_test_user(pool: &PgPool) -> (Uuid, String) {
    let user_id = Uuid::new_v4();
    let email = format!("e2e-{}@selemene.test", user_id);
    sqlx::query(
        r#"
        INSERT INTO users (
            id, email, password_hash, full_name, tier,
            consciousness_level, experience_points
        ) VALUES ($1, $2, $3, $4, 'free', 0, 0)
        "#,
    )
    .bind(user_id)
    .bind(&email)
    .bind("placeholder-hash")
    .bind("E2E Test User")
    .execute(pool)
    .await
    .expect("seed user");
    (user_id, email)
}

async fn cleanup_user(pool: &PgPool, user_id: Uuid) {
    // CASCADE wipes billing_subscriptions etc.
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    // processed_webhook_events does NOT cascade — clean explicitly by the
    // webhook IDs we generated. We leave them; the test's webhook IDs are
    // unique per run and the table is bounded by the prune cron.
}

async fn fetch_user_tier(pool: &PgPool, user_id: Uuid) -> String {
    let row: (String,) = sqlx::query_as("SELECT tier FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("fetch tier");
    row.0
}

async fn fetch_subscription(
    pool: &PgPool,
    provider_subscription_id: &str,
) -> Option<(String, bool)> {
    sqlx::query_as::<_, (String, bool)>(
        "SELECT status, cancel_at_period_end FROM billing_subscriptions WHERE provider_subscription_id = $1",
    )
    .bind(provider_subscription_id)
    .fetch_optional(pool)
    .await
    .expect("fetch subscription")
}

async fn fetch_basic_plan_dodo_product_id(pool: &PgPool) -> String {
    let row: (Option<String>,) =
        sqlx::query_as("SELECT dodo_product_id FROM plan_catalog WHERE code = 'basic'")
            .fetch_one(pool)
            .await
            .expect("fetch basic plan");
    row.0
        .expect("plan_catalog.dodo_product_id for 'basic' must be wired (run runbooks/dodo-dashboard-setup.md §10)")
}

async fn ensure_basic_plan_dodo_product_id(pool: &PgPool) {
    sqlx::query(
        "UPDATE plan_catalog SET dodo_product_id = COALESCE(dodo_product_id, $1) WHERE code = 'basic'",
    )
    .bind(E2E_BASIC_DODO_PRODUCT_ID)
    .execute(pool)
    .await
    .expect("seed basic plan Dodo product ID for billing E2E");
}

async fn post_envelope(
    router: &axum::Router,
    envelope: Value,
    forward_secret: &str,
) -> (StatusCode, Value) {
    let body = serde_json::to_vec(&envelope).expect("serialise envelope");
    let request = Request::builder()
        .method("POST")
        .uri("/internal/billing/events")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Forward-Secret", forward_secret)
        .body(Body::from(body))
        .expect("build request");

    let response = router.clone().oneshot(request).await.expect("oneshot");
    let status = response.status();
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap_or(Value::Null);
    (status, parsed)
}

fn fresh_timestamp() -> String {
    chrono::Utc::now().timestamp().to_string()
}

fn build_subscription_active_envelope(
    webhook_id: &str,
    user_id: Uuid,
    customer_id: &str,
    subscription_id: &str,
    product_id: &str,
) -> Value {
    json!({
        "webhook_id": webhook_id,
        "webhook_timestamp": fresh_timestamp(),
        "event_type": "subscription.active",
        "payload": {
            "type": "subscription.active",
            "data": {
                "subscription_id": subscription_id,
                "product_id": product_id,
                "status": "active",
                "cancel_at_period_end": false,
                "current_period_start": "2026-05-05T00:00:00Z",
                "current_period_end":   "2026-06-05T00:00:00Z",
                "customer": {
                    "customer_id": customer_id,
                    "email": format!("e2e-{}@selemene.test", user_id),
                    "name": "E2E Test User"
                },
                "metadata": { "selemene_user_id": user_id.to_string() }
            }
        }
    })
}

// ---------------------------------------------------------------------------

#[tokio::test]
#[serial_test::serial]
async fn webhook_subscription_active_creates_subscription_and_flips_tier() {
    let (_state, router, pool) = build_state_and_router().await;
    let product_id = fetch_basic_plan_dodo_product_id(&pool).await;
    let (user_id, _email) = seed_test_user(&pool).await;

    let webhook_id = format!("msg_e2e_{}", Uuid::new_v4());
    let subscription_id = format!("sub_e2e_{}", Uuid::new_v4());
    let customer_id = format!("cus_e2e_{}", Uuid::new_v4());

    let envelope = build_subscription_active_envelope(
        &webhook_id,
        user_id,
        &customer_id,
        &subscription_id,
        &product_id,
    );
    let (status, body) = post_envelope(&router, envelope, FORWARD_SECRET).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "first delivery should 200; got {} body={}",
        status,
        body
    );
    assert_eq!(body["status"], "ok");

    // tier flipped to basic
    assert_eq!(fetch_user_tier(&pool, user_id).await, "basic");

    // subscription row exists, status=active, not flagged for cancel
    let sub = fetch_subscription(&pool, &subscription_id)
        .await
        .expect("subscription row should exist");
    assert_eq!(sub.0, "active");
    assert!(!sub.1, "cancel_at_period_end should be false on active");

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
#[serial_test::serial]
async fn webhook_replay_with_same_id_returns_dedup() {
    let (_state, router, pool) = build_state_and_router().await;
    let product_id = fetch_basic_plan_dodo_product_id(&pool).await;
    let (user_id, _) = seed_test_user(&pool).await;

    let webhook_id = format!("msg_dedup_{}", Uuid::new_v4());
    let subscription_id = format!("sub_dedup_{}", Uuid::new_v4());
    let customer_id = format!("cus_dedup_{}", Uuid::new_v4());

    let envelope = build_subscription_active_envelope(
        &webhook_id,
        user_id,
        &customer_id,
        &subscription_id,
        &product_id,
    );

    let (s1, b1) = post_envelope(&router, envelope.clone(), FORWARD_SECRET).await;
    assert_eq!(s1, StatusCode::OK);
    assert_eq!(b1["status"], "ok", "first delivery: {}", b1);

    let (s2, b2) = post_envelope(&router, envelope, FORWARD_SECRET).await;
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(b2["status"], "dedup", "second delivery: {}", b2);

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
#[serial_test::serial]
async fn webhook_subscription_cancelled_immediate_downgrades_tier() {
    let (_state, router, pool) = build_state_and_router().await;
    let product_id = fetch_basic_plan_dodo_product_id(&pool).await;
    let (user_id, _) = seed_test_user(&pool).await;

    // 1. Activate the user
    let active_webhook_id = format!("msg_active_{}", Uuid::new_v4());
    let subscription_id = format!("sub_cancel_{}", Uuid::new_v4());
    let customer_id = format!("cus_cancel_{}", Uuid::new_v4());

    let active_envelope = build_subscription_active_envelope(
        &active_webhook_id,
        user_id,
        &customer_id,
        &subscription_id,
        &product_id,
    );
    let (s1, _) = post_envelope(&router, active_envelope, FORWARD_SECRET).await;
    assert_eq!(s1, StatusCode::OK);
    assert_eq!(fetch_user_tier(&pool, user_id).await, "basic");

    // 2. Immediate cancel (cancel_at_period_end=false)
    let cancel_webhook_id = format!("msg_cancel_{}", Uuid::new_v4());
    let cancel_envelope = json!({
        "webhook_id": cancel_webhook_id,
        "webhook_timestamp": fresh_timestamp(),
        "event_type": "subscription.cancelled",
        "payload": {
            "type": "subscription.cancelled",
            "data": {
                "subscription_id": subscription_id,
                "cancel_at_period_end": false
            }
        }
    });

    let (s2, b2) = post_envelope(&router, cancel_envelope, FORWARD_SECRET).await;
    assert_eq!(s2, StatusCode::OK, "cancel response: {}", b2);

    // tier downgrades to free immediately
    assert_eq!(fetch_user_tier(&pool, user_id).await, "free");

    // subscription row reflects canceled status
    let sub = fetch_subscription(&pool, &subscription_id)
        .await
        .expect("subscription row should still exist");
    assert_eq!(sub.0, "canceled");

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
#[serial_test::serial]
async fn webhook_rejects_request_with_wrong_forward_secret() {
    let (_state, router, _pool) = build_state_and_router().await;
    let envelope = json!({
        "webhook_id": "msg_unauth",
        "webhook_timestamp": "1714867200",
        "event_type": "subscription.active",
        "payload": {}
    });

    let (status, _) = post_envelope(&router, envelope, "wrong-secret").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
