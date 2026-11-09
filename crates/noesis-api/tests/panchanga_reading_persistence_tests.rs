//! GH #1191 persistence probe: a Panchanga reading saved by the calculate
//! route and read back through `/api/v1/readings/:id` keeps its provenance
//! and method fields, and an older stored reading is returned as stored.
//!
//! Requires Postgres at $DATABASE_URL with migrations applied
//! (`bash scripts/cloud/test-db.sh`). Synthetic data only.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use noesis_api::{create_router, shared_metrics, ApiConfig, AppState};
use noesis_auth::AuthService;
use noesis_cache::CacheManager;
use noesis_data::repositories::readings_repository::ReadingsRepository;
use noesis_data::repositories::user_repository::UserRepository;
use noesis_orchestrator::WorkflowOrchestrator;
use serde_json::{json, Value};
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "panchanga-persistence-jwt-secret-32-chars-min";
const DEV_TOKEN: &str = "panchanga-persistence-dev-bypass-token";
// The development-bypass principal (`cf_access::dev_user`).
const DEV_USER: &str = "00000000-0000-0000-0000-000000000001";

fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@localhost/noesis_test".to_string())
}

async fn app() -> (Router, PgPool) {
    std::env::set_var("RUST_ENV", "development");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url())
        .await
        .expect("connect to local Postgres (run scripts/cloud/test-db.sh)");

    let config = ApiConfig {
        host: "127.0.0.1".into(),
        port: 0,
        jwt_secret: JWT_SECRET.into(),
        database_url: Some(database_url()),
        redis_url: None,
        allowed_origins: vec![],
        rate_limit_requests: 1000,
        rate_limit_window_secs: 60,
        request_timeout_secs: 30,
        log_level: "info".into(),
        log_format: "pretty".into(),
        cf_access_issuer: None,
        cf_access_audience: None,
        cf_dev_bypass_token: Some(DEV_TOKEN.into()),
        dodo_payments_api_key: None,
        dodo_payments_webhook_key: None,
        dodo_payments_env: None,
        python_biofield_url: "http://localhost:8002".into(),
        python_biofield_timeout_ms: 10_000,
        gateway_url: None,
        gateway_token: None,
    };
    let mut orchestrator = WorkflowOrchestrator::new();
    orchestrator.register_native_runtime_engines();
    let state = AppState {
        orchestrator: Arc::new(orchestrator),
        bridge_manager: Arc::new(noesis_bridge::BridgeManager::from_env()),
        workflow_registry: None,
        cache: Arc::new(CacheManager::new(
            String::new(),
            100,
            Duration::from_secs(60),
            false,
        )),
        auth: Arc::new(AuthService::with_pool(
            JWT_SECRET.into(),
            Some(pool.clone()),
        )),
        metrics: shared_metrics(),
        user_repository: Arc::new(UserRepository::new(pool.clone())),
        admin_repository: None,
        billing_repository: None,
        biofield_repository: None,
        readings_repository: Some(Arc::new(ReadingsRepository::new(pool.clone()))),
        usage_repository: None,
        db_available: true,
        cf_access_validator: None,
        cf_dev_bypass_token: Some(DEV_TOKEN.into()),
        startup_time: Instant::now(),
        ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
    };
    (create_router(state, &config), pool)
}

/// A synthetic user row for the dev principal, with no prior readings.
async fn fresh_dev_user(pool: &PgPool) -> Uuid {
    let uid = Uuid::parse_str(DEV_USER).unwrap();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, full_name, tier, consciousness_level, experience_points)
         VALUES ($1, 'panchanga-persistence@example.test', 'x', 'Synthetic Seeker', 'free', 0, 0)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(uid)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("DELETE FROM readings WHERE user_id = $1")
        .bind(uid)
        .execute(pool)
        .await
        .unwrap();
    uid
}

async fn send(router: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let res = router.clone().oneshot(req).await.expect("request");
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn get_reading(id: Uuid) -> Request<Body> {
    Request::get(format!("/api/v1/readings/{id}"))
        .header("x-noesis-dev-auth", DEV_TOKEN)
        .body(Body::empty())
        .unwrap()
}

/// The calculate route persists fire-and-forget, so wait for the row.
async fn wait_for_reading(pool: &PgPool, uid: Uuid) -> Uuid {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM readings WHERE user_id = $1 AND engine_id = 'panchanga'",
        )
        .bind(uid)
        .fetch_optional(pool)
        .await
        .unwrap();
        if let Some(id) = id {
            return id;
        }
        assert!(
            Instant::now() < deadline,
            "panchanga reading was never persisted"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
#[serial]
async fn calculated_panchanga_reading_persists_provenance_fields() {
    let (router, pool) = app().await;
    let uid = fresh_dev_user(&pool).await;

    // Synthetic birth data with no time, so `birth_time_assumed` must be true.
    let body = json!({
        "birth_data": {
            "date": "2000-01-01",
            "latitude": 12.9716,
            "longitude": 77.5946,
            "timezone": "Asia/Kolkata"
        }
    });
    let (status, calculated) = send(
        &router,
        Request::post("/api/v1/engines/panchanga/calculate")
            .header("x-noesis-dev-auth", DEV_TOKEN)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "calculate: {calculated}");
    assert_eq!(calculated["result"]["birth_time_assumed"], true);

    let id = wait_for_reading(&pool, uid).await;
    let (status, reading) = send(&router, get_reading(id)).await;
    assert_eq!(status, StatusCode::OK, "get reading: {reading}");
    let stored = &reading["result_data"];

    for field in [
        "position_source",
        "birth_time_assumed",
        "vara_basis",
        "ayanamsa_degrees",
        "tithi_index",
        "julian_day",
    ] {
        assert!(
            !calculated["result"][field].is_null(),
            "calculate response lacks result.{field}"
        );
        assert_eq!(
            stored["result"][field], calculated["result"][field],
            "stored result.{field} differs from the calculate response"
        );
    }
    // The reading stores the engine output, not the API envelope, so the
    // envelope's `provenance` block is not stored. For a native engine that
    // block is derived from `metadata` alone (`native_contract_provenance`),
    // and the stored metadata carries every field it is built from.
    let provenance = &calculated["provenance"];
    assert_eq!(provenance["runtime_kind"], "native");
    assert_eq!(
        stored["metadata"]["fallback_used"],
        provenance["fallback_used"]
    );
    assert_eq!(stored["metadata"]["backend"], provenance["backend_id"]);
    assert_eq!(
        stored["metadata"]["engine_version"],
        provenance["implementation_version"]
    );
    assert_eq!(stored["metadata"]["cached"], provenance["cached"]);
    assert_eq!(stored["metadata"]["fallback_used"], false);
    // The bridge's internal scaffolding key never reaches the stored result.
    assert!(stored["result"].get("__contract_provenance").is_none());
}

#[tokio::test]
#[serial]
async fn pre_provenance_panchanga_reading_is_returned_as_stored() {
    let (router, pool) = app().await;
    let uid = fresh_dev_user(&pool).await;

    // A reading stored before `position_source`, `vara_basis` and
    // `ayanamsa_degrees` existed. Synthetic values.
    let old_result = json!({
        "engine_id": "panchanga",
        "result": {
            "tithi_index": 3, "tithi_name": "Chaturthi (Shukla)", "tithi_value": 3.5,
            "nakshatra_index": 11, "nakshatra_name": "Uttara Phalguni", "nakshatra_value": 11.9,
            "yoga_index": 20, "yoga_name": "Siddha", "yoga_value": 20.4,
            "karana_index": 6, "karana_name": "Vishti", "karana_value": 8.0,
            "vara_index": 2, "vara_name": "Mangalavara (Tuesday)",
            "solar_longitude": 116.0, "lunar_longitude": 158.0, "julian_day": 2448482.0
        },
        "witness_prompt": "synthetic",
        "consciousness_level": 0
    });
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO readings (user_id, engine_id, input_hash, input_data, result_data, witness_prompt, consciousness_level)
         VALUES ($1, 'panchanga', 'synthetic-hash', '{}'::jsonb, $2, 'synthetic', 0)
         RETURNING id",
    )
    .bind(uid)
    .bind(&old_result)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (status, reading) = send(&router, get_reading(id)).await;
    assert_eq!(status, StatusCode::OK, "get reading: {reading}");
    // Returned exactly as stored: nothing is relabelled as newly calculated.
    assert_eq!(reading["result_data"], old_result);
    for field in ["position_source", "vara_basis", "ayanamsa_degrees"] {
        assert!(reading["result_data"]["result"].get(field).is_none());
    }
}
