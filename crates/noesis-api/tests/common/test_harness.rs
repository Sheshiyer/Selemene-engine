use super::{create_test_birth_input, TEST_JWT_SECRET};
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use chrono::Utc;
use noesis_api::{create_router, shared_metrics, ApiConfig, AppState};
use noesis_auth::AuthService;
use noesis_cache::CacheManager;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use noesis_data::repositories::user_repository::UserRepository;
use noesis_orchestrator::WorkflowOrchestrator;
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower::ServiceExt;

#[derive(Clone, Default)]
pub struct ProbeExecutionLog {
    counts: Arc<Mutex<HashMap<String, u64>>>,
}

impl ProbeExecutionLog {
    pub fn record(&self, engine_id: &str) {
        let mut counts = self
            .counts
            .lock()
            .expect("probe counts lock should not poison");
        *counts.entry(engine_id.to_string()).or_insert(0) += 1;
    }

    pub fn count(&self, engine_id: &str) -> u64 {
        let counts = self
            .counts
            .lock()
            .expect("probe counts lock should not poison");
        counts.get(engine_id).copied().unwrap_or(0)
    }
}

struct ProbeEngine {
    id: String,
    required_phase: u8,
    log: ProbeExecutionLog,
}

impl ProbeEngine {
    fn new(id: &str, required_phase: u8, log: ProbeExecutionLog) -> Self {
        Self {
            id: id.to_string(),
            required_phase,
            log,
        }
    }
}

#[async_trait]
impl ConsciousnessEngine for ProbeEngine {
    fn engine_id(&self) -> &str {
        &self.id
    }

    fn engine_name(&self) -> &str {
        &self.id
    }

    fn required_phase(&self) -> u8 {
        self.required_phase
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        self.log.record(&self.id);

        Ok(EngineOutput {
            engine_id: self.id.clone(),
            result: json!({
                "route_marker": format!("probe::{}", self.id),
                "birth_data_present": input.birth_data.is_some(),
            }),
            witness_prompt: format!("probe prompt for {}", self.id),
            consciousness_level: self.required_phase,
            metadata: CalculationMetadata {
                calculation_time_ms: 0.1,
                backend: "test-harness".to_string(),
                precision_achieved: "standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: "test".to_string(),
            },
        })
    }

    async fn validate(&self, _output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        Ok(ValidationResult {
            valid: true,
            confidence: 1.0,
            messages: vec!["probe".to_string()],
        })
    }

    fn cache_key(&self, _input: &EngineInput) -> String {
        format!("probe-{}", self.id)
    }
}

pub struct RoutingHarness {
    router: Router,
    token: String,
    pub log: ProbeExecutionLog,
}

impl RoutingHarness {
    pub async fn with_probe_engines(engine_ids: &[&str]) -> Self {
        let log = ProbeExecutionLog::default();
        let mut orchestrator = WorkflowOrchestrator::new();

        for engine_id in engine_ids {
            orchestrator.register_engine(Arc::new(ProbeEngine::new(engine_id, 0, log.clone())));
        }

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/noesis_test".to_string());
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            jwt_secret: TEST_JWT_SECRET.to_string(),
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
            cf_access_issuer: None,
            cf_access_audience: None,
            cf_dev_bypass_token: None,
            dodo_payments_api_key: None,
            dodo_payments_webhook_key: None,
            dodo_payments_env: None,
            python_biofield_url: "http://localhost:8002".to_string(),
            python_biofield_timeout_ms: 10_000,
            gateway_url: None,
            gateway_token: None,
        };

        let cache = CacheManager::new(String::new(), 100, Duration::from_secs(3600), false);
        let auth = AuthService::new(TEST_JWT_SECRET.to_string());
        let user_repository = Arc::new(UserRepository::new(
            PgPoolOptions::new()
                .max_connections(1)
                .connect_lazy(&database_url)
                .expect("invalid DATABASE_URL for routing harness"),
        ));
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
            db_available: false,
            discord_client_id: None,
            discord_client_secret: None,
            discord_redirect_uri: None,
            cf_access_validator: None,
            cf_dev_bypass_token: None,
            startup_time: Instant::now(),
            ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
        };

        let router = create_router(state, &config);
        let token = AuthService::new(TEST_JWT_SECRET.to_string())
            .generate_jwt_token(
                "routing-harness-user",
                "premium",
                &["read".to_string(), "write".to_string()],
                5,
            )
            .expect("routing harness token should generate");

        Self { router, token, log }
    }

    pub async fn assert_engine_calculate_routed(&self, engine_id: &str) -> Value {
        let before = self.log.count(engine_id);
        let response = self
            .send_authenticated_json(
                "POST",
                &format!("/api/v1/engines/{}/calculate", engine_id),
                Some(
                    serde_json::to_value(create_test_birth_input())
                        .expect("test input should serialize"),
                ),
            )
            .await;

        assert_eq!(response.0, StatusCode::OK);
        assert_eq!(response.1["engine_id"], engine_id);
        assert_eq!(
            response.1["result"]["route_marker"],
            json!(format!("probe::{}", engine_id))
        );
        assert_eq!(self.log.count(engine_id), before + 1);

        response.1
    }

    pub async fn assert_workflow_execute_routed(
        &self,
        workflow_id: &str,
        expected_engine_ids: &[&str],
    ) -> Value {
        let before: HashMap<&str, u64> = expected_engine_ids
            .iter()
            .map(|engine_id| (*engine_id, self.log.count(engine_id)))
            .collect();

        let response = self
            .send_authenticated_json(
                "POST",
                &format!("/api/v1/workflows/{}/execute", workflow_id),
                Some(
                    serde_json::to_value(create_test_birth_input())
                        .expect("test input should serialize"),
                ),
            )
            .await;

        assert_eq!(response.0, StatusCode::OK);
        assert_eq!(response.1["workflow_id"], workflow_id);

        for engine_id in expected_engine_ids {
            assert_eq!(
                response.1["engine_outputs"][engine_id]["result"]["route_marker"],
                json!(format!("probe::{}", engine_id))
            );
            assert_eq!(self.log.count(engine_id), before[engine_id] + 1);
        }

        response.1
    }

    async fn send_authenticated_json(
        &self,
        method: &str,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let request_builder = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::CONTENT_TYPE, "application/json");

        let body = if let Some(json_body) = body {
            Body::from(serde_json::to_vec(&json_body).expect("json body should serialize"))
        } else {
            Body::empty()
        };

        let response = self
            .router
            .clone()
            .oneshot(request_builder.body(body).expect("request should build"))
            .await
            .expect("router request should succeed");

        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should read");
        let json = if body_bytes.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(&body_bytes).expect("response should be valid json")
        };

        (status, json)
    }
}
