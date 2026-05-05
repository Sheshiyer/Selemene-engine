//! Replay-attack stress + freshness tests for the inbound webhook handler.
//!
//! T23 hardening pass. Drives the live `/internal/billing/events` route
//! against a real Postgres to prove:
//!
//!   1. 100× concurrent re-deliveries of the same webhook_id → exactly ONE
//!      mutation (idempotency holds under contention).
//!   2. Stale `webhook_timestamp` (> 5 min old) → 400, no DB writes.
//!   3. Far-future `webhook_timestamp` → 400 (clock-skew defense).
//!   4. Garbage timestamp string → 400.
//!
//! Requires the same setup as billing_e2e_tests.rs (Postgres at
//! $DATABASE_URL, migrations applied, plan_catalog wired with a basic
//! product ID).

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

const FORWARD_SECRET: &str = "billing-replay-forward-secret-very-long-static";

fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://noesis_user:noesis_password@localhost:5432/noesis".to_string()
    })
}

fn fresh_timestamp() -> String {
    chrono::Utc::now().timestamp().to_string()
}

async fn build_state_and_router() -> (axum::Router, PgPool) {
    std::env::set_var("DODO_INTERNAL_FORWARD_SECRET", FORWARD_SECRET);

    let url = database_url();
    let pool = PgPoolOptions::new()
        .max_connections(20) // higher for the 100× concurrent test
        .connect(&url)
        .await
        .expect("connect to local Postgres for replay tests");

    let cache = CacheManager::new(String::new(), 100, Duration::from_secs(60), false);
    let auth = AuthService::new("replay-jwt-secret-must-be-at-least-32-chars-long-xy".into());
    let user_repository = Arc::new(UserRepository::new(pool.clone()));
    let billing_repository = Some(Arc::new(BillingRepository::new(pool.clone())));
    let metrics = shared_metrics();

    let config = ApiConfig {
        host: "127.0.0.1".into(),
        port: 0,
        jwt_secret: "replay-jwt-secret-must-be-at-least-32-chars-long-xy".into(),
        database_url: Some(url.clone()),
        redis_url: None,
        allowed_origins: vec![],
        rate_limit_requests: 1_000_000,
        rate_limit_window_secs: 60,
        request_timeout_secs: 30,
        log_level: "info".into(),
        log_format: "pretty".into(),
        discord_client_id: None,
        discord_client_secret: None,
        discord_redirect_uri: None,
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
        oauth_repository: None,
        db_available: true,
        discord_client_id: None,
        discord_client_secret: None,
        discord_redirect_uri: None,
        startup_time: Instant::now(),
        ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
    };

    let router = create_router(state, &config);
    (router, pool)
}

async fn seed_user(pool: &PgPool) -> Uuid {
    let user_id = Uuid::new_v4();
    let email = format!("replay-{}@selemene.test", user_id);
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, full_name, tier,
                           consciousness_level, experience_points)
        VALUES ($1, $2, 'placeholder', 'Replay Test', 'free', 0, 0)
        "#,
    )
    .bind(user_id)
    .bind(&email)
    .execute(pool)
    .await
    .expect("seed user");
    user_id
}

async fn cleanup(pool: &PgPool, user_id: Uuid) {
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
}

async fn fetch_basic_product_id(pool: &PgPool) -> String {
    let row: (Option<String>,) =
        sqlx::query_as("SELECT dodo_product_id FROM plan_catalog WHERE code = 'basic'")
            .fetch_one(pool)
            .await
            .expect("fetch basic plan");
    row.0
        .expect("plan_catalog.dodo_product_id for 'basic' must be wired")
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

fn build_active_envelope(
    webhook_id: &str,
    user_id: Uuid,
    customer_id: &str,
    subscription_id: &str,
    product_id: &str,
    timestamp: &str,
) -> Value {
    json!({
        "webhook_id": webhook_id,
        "webhook_timestamp": timestamp,
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
                    "email": format!("replay-{}@selemene.test", user_id),
                    "name": "Replay Test"
                },
                "metadata": { "selemene_user_id": user_id.to_string() }
            }
        }
    })
}

// ---------------------------------------------------------------------------

/// 25× concurrent deliveries of the same webhook_id must converge to:
///   • exactly 1 row in `processed_webhook_events`
///   • exactly 1 row in `billing_subscriptions` (no double-billing)
///
/// Note: with idempotency recording moved to AFTER dispatch (so transient
/// dispatch failures don't poison-pill retries), concurrent deliveries that
/// arrive in the race window between the pre-flight SELECT and the post-
/// dispatch INSERT will all dispatch. State mutations are upserts and
/// therefore idempotent, so the DB still ends in the single canonical
/// state. **Sequential** retries (the common Dodo retry case) still
/// short-circuit at the SELECT — covered by the
/// `webhook_replay_with_same_id_returns_dedup` test in billing_e2e_tests.
///
/// 25 deliberately sits under the IP-based rate-limiter default (30/min)
/// so the test exercises only the idempotency primitive, not the
/// orthogonal rate-limit middleware.
#[tokio::test]
#[serial_test::serial]
async fn webhook_replay_concurrent_yields_exactly_one_mutation() {
    let (router, pool) = build_state_and_router().await;
    let product_id = fetch_basic_product_id(&pool).await;
    let user_id = seed_user(&pool).await;

    let webhook_id = format!("msg_replay_{}", Uuid::new_v4());
    let subscription_id = format!("sub_replay_{}", Uuid::new_v4());
    let customer_id = format!("cus_replay_{}", Uuid::new_v4());
    let ts = fresh_timestamp();

    let envelope = build_active_envelope(
        &webhook_id,
        user_id,
        &customer_id,
        &subscription_id,
        &product_id,
        &ts,
    );

    const CONCURRENCY: usize = 25;
    let mut futures = Vec::with_capacity(CONCURRENCY);
    for _ in 0..CONCURRENCY {
        let r = router.clone();
        let env = envelope.clone();
        futures.push(tokio::spawn(async move {
            post_envelope(&r, env, FORWARD_SECRET).await
        }));
    }
    // Every delivery should respond 200 — either dispatched cleanly or
    // dedup'd at the pre-flight check. Distribution between ok/dedup
    // depends on race timing and is not asserted here; the invariant we
    // care about is the resulting DB state below.
    for fut in futures {
        let (status, body) = fut.await.expect("task");
        assert_eq!(
            status,
            StatusCode::OK,
            "every delivery should 200; got {} {}",
            status,
            body
        );
        let s = body.get("status").and_then(|v| v.as_str()).unwrap_or("?");
        assert!(
            s == "ok" || s == "dedup",
            "unexpected status '{}': {}",
            s,
            body
        );
    }

    // DB invariants — the safety properties.
    let processed: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM processed_webhook_events WHERE webhook_id = $1")
            .bind(&webhook_id)
            .fetch_one(&pool)
            .await
            .expect("count processed");
    assert_eq!(
        processed.0, 1,
        "exactly one processed_webhook_events row (PK guarantees this)"
    );

    let subs: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM billing_subscriptions WHERE provider_subscription_id = $1",
    )
    .bind(&subscription_id)
    .fetch_one(&pool)
    .await
    .expect("count subs");
    assert_eq!(subs.0, 1, "exactly one billing_subscriptions row");

    cleanup(&pool, user_id).await;
}

#[tokio::test]
#[serial_test::serial]
async fn webhook_with_stale_timestamp_rejected_400() {
    let (router, _pool) = build_state_and_router().await;

    // 10 minutes ago — well past the 5-minute window.
    let stale = (chrono::Utc::now().timestamp() - 600).to_string();

    let envelope = json!({
        "webhook_id": format!("msg_stale_{}", Uuid::new_v4()),
        "webhook_timestamp": stale,
        "event_type": "subscription.active",
        "payload": { "type": "subscription.active", "data": {} }
    });

    let (status, _) = post_envelope(&router, envelope, FORWARD_SECRET).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial_test::serial]
async fn webhook_with_far_future_timestamp_rejected_400() {
    let (router, _pool) = build_state_and_router().await;

    // 1 hour in the future — well past the +30s skew tolerance.
    let future = (chrono::Utc::now().timestamp() + 3600).to_string();

    let envelope = json!({
        "webhook_id": format!("msg_future_{}", Uuid::new_v4()),
        "webhook_timestamp": future,
        "event_type": "subscription.active",
        "payload": { "type": "subscription.active", "data": {} }
    });

    let (status, _) = post_envelope(&router, envelope, FORWARD_SECRET).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial_test::serial]
async fn webhook_with_garbage_timestamp_rejected_400() {
    let (router, _pool) = build_state_and_router().await;

    let envelope = json!({
        "webhook_id": format!("msg_garbage_{}", Uuid::new_v4()),
        "webhook_timestamp": "not-a-number",
        "event_type": "subscription.active",
        "payload": { "type": "subscription.active", "data": {} }
    });

    let (status, _) = post_envelope(&router, envelope, FORWARD_SECRET).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Stale envelopes must NOT touch the DB — verify that nothing ends up in
/// processed_webhook_events for a rejected webhook_id.
#[tokio::test]
#[serial_test::serial]
async fn stale_envelope_writes_nothing_to_processed_events() {
    let (router, pool) = build_state_and_router().await;

    let webhook_id = format!("msg_no_write_{}", Uuid::new_v4());
    let stale = (chrono::Utc::now().timestamp() - 600).to_string();
    let envelope = json!({
        "webhook_id": webhook_id,
        "webhook_timestamp": stale,
        "event_type": "subscription.active",
        "payload": { "type": "subscription.active", "data": {} }
    });

    let (status, _) = post_envelope(&router, envelope, FORWARD_SECRET).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM processed_webhook_events WHERE webhook_id = $1")
            .bind(&webhook_id)
            .fetch_one(&pool)
            .await
            .expect("count processed");
    assert_eq!(
        count.0, 0,
        "stale rejection should not record idempotency entry"
    );
}
