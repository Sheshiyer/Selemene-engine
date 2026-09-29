//! Exercise the real JWT middleware, report handlers and orchestrator phase gate.
//! Synthetic engines count executions; no DB, provider or ephemeris is contacted.

use async_trait::async_trait;
use axum::{body::Body, http::Request};
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
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tower::ServiceExt;

const JWT_SECRET: &str = "report-phase-tests-only-not-production-secret";

struct PhaseProbe {
    id: &'static str,
    phase: u8,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl ConsciousnessEngine for PhaseProbe {
    fn engine_id(&self) -> &str {
        self.id
    }
    fn engine_name(&self) -> &str {
        self.id
    }
    fn required_phase(&self) -> u8 {
        self.phase
    }
    fn cache_key(&self, _: &EngineInput) -> String {
        self.id.to_string()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn calculate(&self, _: EngineInput) -> Result<EngineOutput, EngineError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(EngineOutput {
            engine_id: self.id.to_string(),
            result: json!({"synthetic": true}),
            witness_prompt: "Synthetic phase probe".to_string(),
            consciousness_level: self.phase,
            metadata: CalculationMetadata {
                calculation_time_ms: 0.0,
                backend: "phase-test".to_string(),
                precision_achieved: "standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: "test".to_string(),
            },
        })
    }

    async fn validate(&self, _: &EngineOutput) -> Result<ValidationResult, EngineError> {
        Ok(ValidationResult {
            valid: true,
            confidence: 1.0,
            messages: vec![],
        })
    }
}

#[tokio::test]
async fn reports_gate_primary_and_partner_engines_only_by_authenticated_phase() {
    // This isolated test binary has one test. Prevent any ambient provider keys
    // from turning its synthetic witness requests into billable network calls.
    std::env::remove_var("NVIDIA_API_KEY");
    std::env::remove_var("OPENROUTER_API_KEY");
    let public_calls = Arc::new(AtomicUsize::new(0));
    let gated_calls = Arc::new(AtomicUsize::new(0));
    let mut orchestrator = WorkflowOrchestrator::new();
    orchestrator.register_engine(Arc::new(PhaseProbe {
        id: "panchanga",
        phase: 0,
        calls: public_calls.clone(),
    }));
    orchestrator.register_engine(Arc::new(PhaseProbe {
        id: "numerology",
        phase: 3,
        calls: gated_calls.clone(),
    }));
    let database_url = "postgres://localhost:1/phase_tests_no_connection";
    let config = ApiConfig {
        host: "127.0.0.1".into(),
        port: 0,
        jwt_secret: JWT_SECRET.into(),
        database_url: Some(database_url.into()),
        redis_url: None,
        allowed_origins: vec![],
        rate_limit_requests: 100,
        rate_limit_window_secs: 60,
        request_timeout_secs: 30,
        log_level: "info".into(),
        log_format: "pretty".into(),
        cf_access_issuer: None,
        cf_access_audience: None,
        cf_dev_bypass_token: None,
        dodo_payments_api_key: None,
        dodo_payments_webhook_key: None,
        dodo_payments_env: None,
        python_biofield_url: "http://localhost:1".into(),
        python_biofield_timeout_ms: 100,
        gateway_url: None,
        gateway_token: None,
    };
    let state = AppState {
        orchestrator: Arc::new(orchestrator),
        bridge_manager: Arc::new(noesis_bridge::BridgeManager::from_env()),
        workflow_registry: None,
        cache: Arc::new(CacheManager::new(
            String::new(),
            10,
            Duration::from_secs(60),
            false,
        )),
        auth: Arc::new(AuthService::new(JWT_SECRET.into())),
        metrics: shared_metrics(),
        user_repository: Arc::new(UserRepository::new(
            PgPoolOptions::new()
                .max_connections(1)
                .connect_lazy(database_url)
                .unwrap(),
        )),
        admin_repository: None,
        billing_repository: None,
        biofield_repository: None,
        readings_repository: None,
        usage_repository: None,
        db_available: false,
        cf_access_validator: None,
        cf_dev_bypass_token: None,
        startup_time: Instant::now(),
        ephemeris_checksums: Arc::new(Default::default()),
    };
    let router = create_router(state, &config);
    let auth = AuthService::new(JWT_SECRET.into());
    let birth_data = json!({
        "date": "1990-01-15", "time": "14:30", "latitude": 12.97,
        "longitude": 77.59, "timezone": "Asia/Kolkata", "name": "Synthetic",
    });

    for endpoint in ["assets/generate", "witness/interpret"] {
        // Includes omitted legacy field, both valid boundaries, and an ignored
        // out-of-range value. A lower requested phase cannot demote identity.
        for (auth_phase, requested) in [
            (0, Some(5)),
            (0, Some(255)),
            (0, None),
            (5, Some(0)),
            (5, None),
        ] {
            let token = auth
                .generate_jwt_token(
                    "synthetic-report-user",
                    "premium",
                    &["read".into(), "write".into()],
                    auth_phase,
                )
                .unwrap();
            let mut body = if endpoint == "assets/generate" {
                json!({"mode": "integrated-reading", "birth_data": birth_data})
            } else {
                json!({
                    "birth_data": birth_data, "partner_birth_data": birth_data,
                    "live_scores": {"energy": 0.5, "coherence": 0.5, "symmetry": 0.5,
                        "complexity": 0.5, "regulation": 0.5, "color_balance": 0.5}
                })
            };
            if let Some(requested) = requested {
                body["consciousness_level"] = json!(requested);
            }
            let public_before = public_calls.load(Ordering::SeqCst);
            let gated_before = gated_calls.load(Ordering::SeqCst);
            let request = Request::builder()
                .method("POST")
                .uri(format!("/api/v1/{endpoint}"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap();
            let response = router.clone().oneshot(request).await.unwrap();
            assert_eq!(
                response.status(),
                200,
                "{endpoint}, auth={auth_phase}, request={requested:?}"
            );
            let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let output: Value = serde_json::from_slice(&bytes).unwrap();

            assert_eq!(
                public_calls.load(Ordering::SeqCst),
                public_before + 1,
                "public engine must still execute for {endpoint}"
            );
            let allowed_calls = if auth_phase < 3 {
                0
            } else if endpoint == "witness/interpret" {
                2
            } else {
                1
            };
            assert_eq!(gated_calls.load(Ordering::SeqCst), gated_before + allowed_calls,
                "{endpoint} must gate primary and partner execution by auth={auth_phase}, not request={requested:?}");
            if endpoint == "assets/generate" {
                assert_eq!(
                    output["register"],
                    if auth_phase <= 3 { "l1_l3" } else { "l4_l5" }
                );
                assert_eq!(
                    output["engines_used"]
                        .as_array()
                        .unwrap()
                        .contains(&json!("numerology")),
                    auth_phase >= 3
                );
            } else {
                assert_eq!(output["llm_powered"], false);
            }
        }
    }
}
