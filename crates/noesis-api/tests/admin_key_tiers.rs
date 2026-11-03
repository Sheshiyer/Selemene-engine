use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::Utc;
use noesis_api::{build_app_state_lazy_db, create_router, ApiConfig};
use noesis_auth::{
    key_tiers::{KeyTier, KEY_ENGINE_IDS},
    ApiKey,
};
use noesis_data::repositories::admin_repository::AdminRepository;
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn request(
    router: &Router,
    method: &str,
    path: &str,
    key: &str,
    body: Value,
) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("X-API-Key", key)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[test]
fn tier_catalog_matches_reviewed_runtime_catalog() {
    assert_eq!(KEY_ENGINE_IDS, noesis_orchestrator::SUPPORTED_ENGINE_IDS);
    let mut runtime = noesis_orchestrator::WorkflowOrchestrator::new();
    runtime.register_native_runtime_engines();
    for id in runtime.list_engines() {
        let required = runtime.registry().get(&id).unwrap().required_phase();
        assert!(
            required <= KeyTier::Pro.entitlements().consciousness_level,
            "{id} needs higher Pro access"
        );
    }
}

#[tokio::test]
async fn create_all_tiers_with_server_scopes_and_reject_escalation() {
    let Ok(url) = std::env::var("ADMIN_KEY_TEST_DATABASE_URL") else {
        eprintln!("not run: ADMIN_KEY_TEST_DATABASE_URL is required");
        return;
    };
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    let schema = format!("api_key_test_{}", Uuid::new_v4().simple());
    sqlx::raw_sql(&format!("CREATE SCHEMA {schema}; SET search_path TO {schema};
        CREATE TABLE users (id uuid PRIMARY KEY, email text NOT NULL, tier text NOT NULL);
        CREATE TABLE api_keys (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), key_hash text UNIQUE NOT NULL,
        name text, key_prefix text, user_id uuid REFERENCES users(id), created_by_user_id uuid,
        tier text, permissions jsonb, consciousness_level int, rate_limit int, expires_at timestamptz,
        rotated_from_key_id uuid, is_active bool, created_at timestamptz DEFAULT now(), last_used timestamptz);
        CREATE TABLE api_key_events (id uuid DEFAULT gen_random_uuid(), key_id uuid REFERENCES api_keys(id),
        actor_user_id uuid, event_type text, metadata jsonb, created_at timestamptz);"))
        .execute(&pool).await.unwrap();
    let owner = Uuid::new_v4();
    sqlx::query("INSERT INTO users VALUES ($1,'synthetic@example.test','enterprise')")
        .bind(owner)
        .execute(&pool)
        .await
        .unwrap();
    let mut config = ApiConfig::from_env().unwrap();
    config.database_url = None;
    config.redis_url = None;
    let mut state = build_app_state_lazy_db(&config).await;
    state.admin_repository = Some(Arc::new(AdminRepository::new(pool.clone())));
    let admin_key = format!("test-admin-{}", Uuid::new_v4());
    state
        .auth
        .add_api_key(ApiKey {
            key: admin_key.clone(),
            user_id: "synthetic-admin".into(),
            tier: "enterprise".into(),
            permissions: vec!["admin:*".into()],
            created_at: Utc::now(),
            expires_at: None,
            last_used: None,
            rate_limit: 10000,
            consciousness_level: 5,
        })
        .await
        .unwrap();
    state
        .auth
        .add_api_key(ApiKey {
            key: "list-only-admin-key".into(),
            user_id: "synthetic-viewer".into(),
            tier: "free".into(),
            permissions: vec!["admin:keys:list".into()],
            created_at: Utc::now(),
            expires_at: None,
            last_used: None,
            rate_limit: 60,
            consciousness_level: 0,
        })
        .await
        .unwrap();
    let auth = state.auth.clone();
    let router = create_router(state, &config);
    let (status, catalog) = request(
        &router,
        "GET",
        "/api/v1/admin/api-key-tiers",
        &admin_key,
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        catalog
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["label"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["Basic", "Pro", "Enterprise"]
    );
    let (status, _) = request(
        &router,
        "GET",
        "/api/v1/admin/api-key-tiers",
        "list-only-admin-key",
        Value::Null,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "key viewers need the nonsecret tier catalog"
    );
    for tier in KeyTier::ALL {
        let (status, response) = request(
            &router,
            "POST",
            "/api/v1/admin/api-keys",
            &admin_key,
            json!({"user_id":owner,"name":"synthetic tier test","tier":tier.label()}),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{response}");
        assert_eq!(response["key"]["tier"], tier.code());
        assert_eq!(
            response["key"]["permissions"],
            json!(tier.entitlements().permissions)
        );
        assert!(response["key"].get("secret_key").is_none());
        assert!(response["key"].get("key_hash").is_none());
        let principal = auth
            .validate_api_key(response["secret_key"].as_str().unwrap())
            .await
            .unwrap();
        assert_eq!(
            principal.consciousness_level,
            tier.entitlements().consciousness_level
        );
        assert_eq!(principal.permissions, tier.entitlements().permissions);
    }
    for extra in [
        json!({"permissions":["admin:*"]}),
        json!({"consciousness_level":5}),
        json!({"rate_limit":0}),
    ] {
        let mut payload = json!({"user_id":owner,"tier":"basic"});
        payload
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let (status, _) = request(
            &router,
            "POST",
            "/api/v1/admin/api-keys",
            &admin_key,
            payload,
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
    let (status, response) = request(
        &router,
        "POST",
        "/api/v1/admin/api-keys",
        &admin_key,
        json!({"user_id":owner,"tier":"standard"}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{response}");
    sqlx::raw_sql(&format!(
        "SET search_path TO public; DROP SCHEMA {schema} CASCADE"
    ))
    .execute(&pool)
    .await
    .unwrap();
}
