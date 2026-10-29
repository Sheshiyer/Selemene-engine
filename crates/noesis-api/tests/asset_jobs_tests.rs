//! Background reading jobs (`/api/v1/assets/jobs`) against a real Postgres,
//! with a stand-in OmniRoute gateway so no model is called.
//!
//! Requires Postgres at $DATABASE_URL with migrations through 043 applied
//! (`bash scripts/cloud/test-db.sh`).

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    routing::post,
    Json, Router,
};
use http_body_util::BodyExt;
use noesis_api::{create_router, shared_metrics, ApiConfig, AppState};
use noesis_auth::AuthService;
use noesis_cache::CacheManager;
use noesis_data::repositories::user_repository::UserRepository;
use noesis_orchestrator::WorkflowOrchestrator;
use serde_json::{json, Value};
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceExt;

const JWT_SECRET: &str = "asset-jobs-jwt-secret-at-least-32-chars-long";
const DEV_TOKEN: &str = "asset-jobs-dev-bypass-token";
const DEV_USER: &str = "00000000-0000-0000-0000-000000000001";

fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@localhost/noesis_test".to_string())
}

async fn app() -> (axum::Router, PgPool) {
    std::env::set_var("RUST_ENV", "development");
    let pool = PgPoolOptions::new()
        .max_connections(2)
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
    let state = AppState {
        orchestrator: Arc::new(WorkflowOrchestrator::new()),
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
        readings_repository: None,
        usage_repository: None,
        db_available: true,
        cf_access_validator: None,
        cf_dev_bypass_token: Some(DEV_TOKEN.into()),
        startup_time: Instant::now(),
        ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
    };
    (create_router(state, &config), pool)
}

/// A stand-in OmniRoute that answers each pass after `delay`.
async fn fake_gateway(delay: Duration) -> String {
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move |Json(body): Json<Value>| async move {
            tokio::time::sleep(delay).await;
            let user = body["messages"][1]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let pass = user
                .split("Pass: ")
                .nth(1)
                .and_then(|rest| rest.split('.').next())
                .unwrap_or("?")
                .to_string();
            let chunk = json!({
                "model": "claude-test",
                "choices": [{ "delta": { "content": format!("Written {pass}.") } }]
            });
            (
                [(header::CONTENT_TYPE, "text/event-stream")],
                format!("data: {chunk}\n\ndata: [DONE]\n\n"),
            )
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}/v1")
}

fn use_model(base_url: Option<&str>) {
    for key in ["LLM_PROVIDER", "NVIDIA_API_KEY", "OPENROUTER_API_KEY"] {
        std::env::remove_var(key);
    }
    match base_url {
        Some(url) => {
            std::env::set_var("OMNIROUTE_BASE_URL", url);
            std::env::set_var("OMNIROUTE_API_KEY", "test-key");
            std::env::set_var("OMNIROUTE_MODEL", "noesis-review");
        }
        None => {
            std::env::remove_var("OMNIROUTE_API_KEY");
            std::env::remove_var("OMNIROUTE_MODEL");
        }
    }
}

async fn clear_jobs(pool: &PgPool) {
    sqlx::query("DELETE FROM asset_jobs WHERE user_id = $1 OR user_id = 'other-user'")
        .bind(DEV_USER)
        .execute(pool)
        .await
        .unwrap();
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

fn as_dev(req: axum::http::request::Builder) -> axum::http::request::Builder {
    req.header("x-noesis-dev-auth", DEV_TOKEN)
}

fn l0_request(mode: &str) -> Value {
    // Synthetic subject; no real birth data.
    json!({
        "mode": mode,
        "report_level": "L0",
        "subjects": [{
            "role": "primary",
            "name": "Test Seeker",
            "birth_date": "1990-01-15",
            "birth_time": "10:30",
            "normalized_location": {
                "display_name": "Bengaluru, India",
                "latitude": 12.9716,
                "longitude": 77.5946,
                "timezone": "Asia/Kolkata",
                "provider": "manual",
                "confidence": "exact"
            }
        }]
    })
}

fn start(body: Value) -> Request<Body> {
    as_dev(Request::post("/api/v1/assets/jobs"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn poll(job_id: &str) -> Request<Body> {
    as_dev(Request::get(format!("/api/v1/assets/jobs/{job_id}")))
        .body(Body::empty())
        .unwrap()
}

async fn wait_for_finish(router: &Router, job_id: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let (status, job) = send(router, poll(job_id)).await;
        assert_eq!(status, StatusCode::OK, "{job}");
        if job["status"] != "running" {
            return job;
        }
        assert!(Instant::now() < deadline, "job did not finish: {job}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
#[serial]
async fn a_job_writes_every_pass_with_the_model_and_reports_progress() {
    let (router, pool) = app().await;
    clear_jobs(&pool).await;
    use_model(Some(&fake_gateway(Duration::from_millis(20)).await));

    let (status, job) = send(&router, start(l0_request("integrated-kundali-l0"))).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{job}");
    assert_eq!(job["status"], "running");
    assert_eq!(job["total_passes"], 12);
    let job_id = job["job_id"].as_str().unwrap().to_string();

    let done = wait_for_finish(&router, &job_id).await;
    assert_eq!(done["status"], "succeeded", "{done}");
    assert_eq!(done["completed_passes"], 12);
    let result = &done["result"];
    assert_eq!(result["passes"].as_array().unwrap().len(), 12);
    assert_eq!(result["passes"][0]["output"], "Written opening.");
    assert_eq!(result["passes"][11]["output"], "Written final-synthesis.");
    let sections = result["source_pack"]["quality"]["sections"]
        .as_array()
        .unwrap();
    assert!(sections
        .iter()
        .all(|s| s["model_used"] == "claude-test" && s["model_requested"] == "noesis-review"));
    let blockers = result["source_pack"]["audit"]["blockers"]
        .as_array()
        .unwrap();
    assert!(
        !blockers.iter().any(|b| b
            .as_str()
            .unwrap()
            .starts_with("sections_not_model_written")),
        "{blockers:?}"
    );
    assert_eq!(result["source_pack"]["register"], result["register"]);
    clear_jobs(&pool).await;
}

#[tokio::test]
#[serial]
async fn one_job_at_a_time_and_only_the_owner_can_see_it() {
    let (router, pool) = app().await;
    clear_jobs(&pool).await;
    use_model(Some(&fake_gateway(Duration::from_millis(300)).await));

    let (_, job) = send(&router, start(l0_request("integrated-kundali-l0"))).await;
    let job_id = job["job_id"].as_str().unwrap().to_string();
    let (status, second) = send(&router, start(l0_request("integrated-kundali-l0"))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{second}");
    assert_eq!(second["error_code"], "JOB_IN_PROGRESS");
    assert_eq!(second["details"]["job_id"], job_id);

    let auth = AuthService::new(JWT_SECRET.into());
    let other = auth
        .generate_jwt_token("other-user", "free", &["basic:access".to_string()], 3)
        .unwrap();
    let (status, _) = send(
        &router,
        Request::get(format!("/api/v1/assets/jobs/{job_id}"))
            .header(header::AUTHORIZATION, format!("Bearer {other}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = send(&router, poll("not-a-uuid")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    clear_jobs(&pool).await;
}

#[tokio::test]
#[serial]
async fn a_job_needs_a_model_and_an_authored_mode() {
    let (router, pool) = app().await;
    clear_jobs(&pool).await;

    use_model(None);
    let (status, body) = send(&router, start(l0_request("integrated-kundali-l0"))).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{body}");
    assert_eq!(body["error_code"], "MODEL_UNAVAILABLE");

    use_model(Some("http://127.0.0.1:9/v1"));
    let (status, body) = send(&router, start(l0_request("integrated-reading"))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error_code"], "MODE_NOT_WRITABLE");
    let (status, body) = send(&router, start(l0_request("no-such-mode"))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
}

#[tokio::test]
#[serial]
async fn a_failing_model_fails_the_job_with_a_reason() {
    let (router, pool) = app().await;
    clear_jobs(&pool).await;
    use_model(Some("http://127.0.0.1:9/v1"));

    let (_, job) = send(&router, start(l0_request("integrated-kundali-l0"))).await;
    let done = wait_for_finish(&router, job["job_id"].as_str().unwrap()).await;
    assert_eq!(done["status"], "failed");
    assert!(
        done["error"]
            .as_str()
            .unwrap()
            .starts_with("Writing \"Opening\" failed:"),
        "{done}"
    );
    assert!(done["result"].is_null());
    clear_jobs(&pool).await;
}

#[tokio::test]
#[serial]
async fn a_job_that_went_quiet_is_reported_failed_and_frees_the_user() {
    let (router, pool) = app().await;
    clear_jobs(&pool).await;
    let id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO asset_jobs (id, user_id, mode, status, total_passes, updated_at) \
         VALUES ($1, $2, 'integrated-kundali-l0', 'running', 12, NOW() - INTERVAL '1 hour')",
    )
    .bind(id)
    .bind(DEV_USER)
    .execute(&pool)
    .await
    .unwrap();

    let (status, job) = send(&router, poll(&id.to_string())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(job["status"], "failed");
    assert!(job["error"]
        .as_str()
        .unwrap()
        .contains("stopped before it finished"));

    use_model(Some(&fake_gateway(Duration::from_millis(1)).await));
    let (status, body) = send(&router, start(l0_request("integrated-kundali-l0"))).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    wait_for_finish(&router, body["job_id"].as_str().unwrap()).await;
    clear_jobs(&pool).await;
}
