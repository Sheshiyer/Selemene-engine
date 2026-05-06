//! Integration tests for billing hook emission.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::{Duration as ChronoDuration, Utc};
use noesis_api::{
    create_router, reset_billing_emitter, set_billing_emitter, shared_metrics, ApiConfig, AppState,
    BillingEventEmitter, StripeWebhookEmitter,
};
use noesis_auth::{ApiKey, AuthService};
use noesis_cache::CacheManager;
use noesis_data::repositories::user_repository::UserRepository;
use noesis_orchestrator::WorkflowOrchestrator;
use sqlx::postgres::PgPoolOptions;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tower::ServiceExt;

mod common;

#[derive(Clone, Default)]
struct RecordingEmitter {
    usage_events: Arc<Mutex<Vec<(String, String, String)>>>,
    quota_events: Arc<Mutex<Vec<(String, String)>>>,
}

impl BillingEventEmitter for RecordingEmitter {
    fn emit_usage_event(&self, user_id: &str, engine_id: &str, tier: &str) {
        self.usage_events.lock().unwrap().push((
            user_id.to_string(),
            engine_id.to_string(),
            tier.to_string(),
        ));
    }

    fn emit_quota_exceeded(&self, user_id: &str, tier: &str) {
        self.quota_events
            .lock()
            .unwrap()
            .push((user_id.to_string(), tier.to_string()));
    }
}

fn build_test_app_state() -> (AppState, ApiConfig) {
    let mut orchestrator = WorkflowOrchestrator::new();
    orchestrator.register_engine(Arc::new(engine_panchanga::PanchangaEngine::new()));
    orchestrator.register_engine(Arc::new(engine_numerology::NumerologyEngine::new()));
    orchestrator.register_engine(Arc::new(engine_biorhythm::BiorhythmEngine::new()));

    let cache = CacheManager::new(String::new(), 100, Duration::from_secs(3600), false);

    let jwt_secret = common::TEST_JWT_SECRET.to_string();
    let auth = AuthService::new(jwt_secret);

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/noesis_test".to_string());
    let config = ApiConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        jwt_secret: common::TEST_JWT_SECRET.to_string(),
        database_url: Some(database_url.clone()),
        redis_url: None,
        allowed_origins: vec![],
        rate_limit_requests: 100,
        rate_limit_window_secs: 60,
        request_timeout_secs: 30,
        log_level: "info".to_string(),
        log_format: "pretty".to_string(),
        discord_client_id: None,
        discord_client_secret: None,
        discord_redirect_uri: None,
        dodo_payments_api_key: None,
        dodo_payments_webhook_key: None,
        dodo_payments_env: None,
        python_biofield_url: "http://localhost:8002".to_string(),
        python_biofield_timeout_ms: 10_000,
        gateway_url: None,
        gateway_token: None,
    };

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&database_url)
        .expect("Invalid DATABASE_URL");
    let user_repository = Arc::new(UserRepository::new(pool));

    let metrics = shared_metrics();

    let state = AppState {
        orchestrator: Arc::new(orchestrator),
        bridge_manager: Arc::new(noesis_bridge::BridgeManager::from_env()),
        cache: Arc::new(cache),
        auth: Arc::new(auth),
        metrics,
        user_repository,
        admin_repository: None,
        billing_repository: None,
        biofield_repository: None,
        readings_repository: None,
        usage_repository: None,
        oauth_repository: None,
        db_available: false,
        discord_client_id: None,
        discord_client_secret: None,
        discord_redirect_uri: None,
        startup_time: Instant::now(),
        ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
    };

    (state, config)
}

async fn create_test_api_key(auth: &Arc<AuthService>, user_id: &str, rate_limit: u32) -> String {
    let api_key_value = format!("billing-test-key-{}", user_id);

    let api_key = ApiKey {
        key: api_key_value.clone(),
        user_id: user_id.to_string(),
        tier: "free".to_string(),
        permissions: vec!["basic:access".to_string()],
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + ChronoDuration::hours(1)),
        last_used: None,
        rate_limit,
        consciousness_level: 0,
    };

    auth.add_api_key(api_key)
        .await
        .expect("Failed to add API key");
    api_key_value
}

#[tokio::test]
#[serial_test::serial]
async fn test_billing_usage_event_emitted_on_calculation() {
    let emitter = RecordingEmitter::default();
    set_billing_emitter(Arc::new(emitter.clone()));

    let router = common::get_router().await;
    let token = common::generate_test_token(5);
    let input = common::create_test_birth_input();

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/engines/numerology/calculate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Unexpected status: {}",
        response.status()
    );

    let events = emitter.usage_events.lock().unwrap();
    assert!(!events.is_empty(), "Expected usage billing event emission");

    reset_billing_emitter();
}

#[test]
fn test_stripe_webhook_emitter_formats_payload_structure() {
    let emitter = StripeWebhookEmitter::new("https://stripe.example/webhook");

    let usage = emitter.format_usage_payload("user-1", "numerology", "pro");
    assert_eq!(usage["event_type"], "usage_event");
    assert_eq!(usage["provider"], "stripe");
    assert_eq!(usage["user_id"], "user-1");
    assert_eq!(usage["engine_id"], "numerology");
    assert_eq!(usage["tier"], "pro");
    assert!(usage["timestamp"].as_str().is_some());

    let quota = emitter.format_quota_exceeded_payload("user-1", "free");
    assert_eq!(quota["event_type"], "quota_exceeded");
    assert_eq!(quota["provider"], "stripe");
    assert_eq!(quota["user_id"], "user-1");
    assert_eq!(quota["tier"], "free");
    assert!(quota["timestamp"].as_str().is_some());
}

#[tokio::test]
#[serial_test::serial]
async fn test_billing_quota_event_emitted_on_rate_limit() {
    let emitter = RecordingEmitter::default();
    set_billing_emitter(Arc::new(emitter.clone()));

    let (state, config) = build_test_app_state();
    let api_key = create_test_api_key(&state.auth, "quota-user", 1).await;
    let app = create_router(state, &config);

    let first = Request::builder()
        .uri("/api/v1/status")
        .header("X-API-Key", &api_key)
        .body(Body::empty())
        .unwrap();
    let first_resp = app.clone().oneshot(first).await.unwrap();
    assert_eq!(first_resp.status(), StatusCode::OK);

    let second = Request::builder()
        .uri("/api/v1/status")
        .header("X-API-Key", &api_key)
        .body(Body::empty())
        .unwrap();
    let second_resp = app.clone().oneshot(second).await.unwrap();
    assert_eq!(second_resp.status(), StatusCode::TOO_MANY_REQUESTS);

    let quota_events = emitter.quota_events.lock().unwrap();
    assert!(
        !quota_events.is_empty(),
        "Expected quota exceeded billing event emission"
    );

    reset_billing_emitter();
}
