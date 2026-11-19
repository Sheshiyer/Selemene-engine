//! Real PostgreSQL/router boundary tests. DATABASE_URL must be an isolated test database.
use axum::{
    body::Body,
    http::{Request, StatusCode},
};

use http_body_util::BodyExt;
use noesis_api::desktop_auth::{hash_secret, new_secret, s256_challenge};
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
use uuid::Uuid;

const JWT_SECRET: &str = "desktop-auth-jwt-secret-at-least-32-chars-long";
const DEV_TOKEN: &str = "desktop-auth-dev-bypass-token";
#[allow(dead_code)]
const REDIRECT: &str = "http://127.0.0.1:53682/callback";
#[allow(dead_code)]
const STATE: &str = "state-0123456789abcdef";
#[allow(dead_code)]
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
#[allow(dead_code)]
const DEV_USER: &str = "00000000-0000-0000-0000-000000000001";

/// Sets an environment variable for the guard's lifetime and restores the
/// prior value on drop, so a failing assertion cannot leak it into later tests.
struct EnvVarGuard {
    key: &'static str,
    prior: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let prior = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, prior }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match self.prior.take() {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

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

    let fixture_pool = pool.clone();
    let pool = if let Ok(url) = std::env::var("TEST_SHARED_RUNTIME_URL") {
        PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .unwrap()
    } else {
        pool
    };
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
    orchestrator.register_engine(Arc::new(engine_biorhythm::BiorhythmEngine::new()));
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
        readings_repository: None,
        usage_repository: None,
        db_available: true,
        cf_access_validator: std::env::var("TEST_SHARED_CF_ISSUER").ok().map(|issuer| {
            Arc::new(noesis_api::cf_access::CfAccessValidator::new(
                issuer,
                "shared-test-audience".into(),
            ))
        }),
        cf_dev_bypass_token: Some(DEV_TOKEN.into()),
        startup_time: Instant::now(),
        ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
    };
    (create_router(state, &config), fixture_pool)
}

async fn call(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
async fn user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users(id,email,password_hash,full_name) VALUES($1,$2,'disabled','Synthetic')",
    )
    .bind(id)
    .bind(format!("{id}@example.test"))
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO user_account_state(user_id,state) VALUES($1,'active')")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    id
}
fn jwt(pool: &PgPool, id: Uuid) -> String {
    AuthService::with_pool(JWT_SECRET.into(), Some(pool.clone()))
        .generate_jwt_token(&id.to_string(), "free", &["basic:access".into()], 0)
        .unwrap()
}
async fn registered_jwt(pool: &PgPool, id: Uuid) -> String {
    let identity=sqlx::query_as::<_,(String,String)>("SELECT issuer,provider_subject FROM account_identities WHERE user_id=$1 AND revoked_at IS NULL LIMIT 1").bind(id).fetch_optional(pool).await.unwrap();
    let (issuer, sub) =
        identity.unwrap_or_else(|| ("https://registered.example.test".into(), id.to_string()));
    sqlx::query("INSERT INTO account_identities(issuer,provider_subject,user_id,verified_email) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING").bind(&issuer).bind(&sub).bind(id).bind(format!("{id}@example.test")).execute(pool).await.unwrap();
    let family = Uuid::new_v4();
    sqlx::query("INSERT INTO registered_session_families(id,client_id,user_id,identity_issuer,identity_subject,expires_at) VALUES($1,'noesis-daily',$2,$3,$4,now()+interval '1 hour')").bind(family).bind(id).bind(issuer).bind(sub).execute(pool).await.unwrap();
    let auth = AuthService::with_pool(JWT_SECRET.into(), Some(pool.clone()));
    let token = auth
        .generate_jwt_token(
            &id.to_string(),
            "free",
            &["basic:access".into(), "session:registered".into()],
            0,
        )
        .unwrap();
    let u = auth.validate_jwt_token(&token).await.unwrap();
    sqlx::query("INSERT INTO registered_access_tokens(jti,family_id,expires_at) VALUES($1,$2,now()+interval '1 hour')").bind(u.jti).bind(family).execute(pool).await.unwrap();
    token
}
#[tokio::test]
#[serial]
async fn real_reader_matrix_revision_conflict_rls_and_revocation() {
    let (app, pool) = app().await;
    let owner = user(&pool).await;
    let own = user(&pool).await;
    let denied = user(&pool).await;
    let other_owner = user(&pool).await;
    let subject:Uuid=sqlx::query_scalar("INSERT INTO corpus_subjects(owner_user_id,subject_key,canonical_name) VALUES($1,$2,'Synthetic subject') RETURNING id").bind(owner).bind(Uuid::new_v4().to_string()).fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO subject_grants(subject_id,owner_user_id,grantee_user_id,grantor_user_id,can_edit,can_calculate) VALUES($1,$2,$2,$2,false,false),($1,$2,$3,$2,true,true)").bind(subject).bind(owner).bind(own).execute(&pool).await.unwrap();
    // Composite FK prevents fabricated cross-owner grants.
    assert!(sqlx::query("INSERT INTO subject_grants(subject_id,owner_user_id,grantee_user_id,grantor_user_id) VALUES($1,$2,$3,$2)").bind(subject).bind(other_owner).bind(denied).execute(&pool).await.is_err());
    let own_token = jwt(&pool, own);
    let owner_token = jwt(&pool, owner);
    let denied_token = jwt(&pool, denied);
    let path = format!("/api/v1/subjects/{subject}/profile");
    let p = json!({"expected_revision":0,"patch":{"full_name":"Synthetic","birth_date":"1990-01-01","birth_location":{"lat":0,"lng":0,"name":"Synthetic"},"timezone":"UTC","birth_time_confidence":"unknown"}});
    assert_eq!(
        call(&app, "PATCH", &path, Some(&owner_token), p.clone())
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&app, "PATCH", &path, Some(&own_token), p.clone())
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, "PATCH", &path, Some(&own_token), p).await.0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(&app, "GET", &path, Some(&owner_token), Value::Null)
            .await
            .1["revision"],
        1
    );
    assert_eq!(
        call(&app, "GET", &path, Some(&denied_token), Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &app,
            "PATCH",
            &path,
            Some(&own_token),
            json!({"expected_revision":1,"patch":{"email":"fake@example.test"}})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let ctx = call(
        &app,
        "GET",
        "/api/v1/users/me/context",
        Some(&denied_token),
        Value::Null,
    )
    .await
    .1;
    assert_eq!(ctx["subjects"], json!([]));
    assert_eq!(
        call(
            &app,
            "PATCH",
            "/api/v1/users/me/context",
            Some(&denied_token),
            json!({"default_subject_id":subject})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &app,
            "PATCH",
            "/api/v1/users/me/context",
            Some(&own_token),
            json!({"default_subject_id":subject})
        )
        .await
        .0,
        StatusCode::OK
    );
    // A read grant alone cannot compute for another person.
    assert_eq!(
        call(
            &app,
            "POST",
            &format!("/api/v1/subjects/{subject}/engines/biorhythm/calculate"),
            Some(&owner_token),
            json!({"expected_revision":1})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    // Explicit calculation grant records actor/subject separately and never fills actor defaults.
    sqlx::query(
        "UPDATE subject_grants SET can_calculate=true WHERE subject_id=$1 AND grantee_user_id=$2",
    )
    .bind(subject)
    .bind(owner)
    .execute(&pool)
    .await
    .unwrap();
    let (status, run) = call(
        &app,
        "POST",
        &format!("/api/v1/subjects/{subject}/engines/biorhythm/calculate"),
        Some(&owner_token),
        json!({"expected_revision":1,"current_time":"2026-11-07T00:30:00Z"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{run}");
    assert_eq!(run["actor_account_id"], owner.to_string());
    assert_eq!(run["subject_id"], subject.to_string());
    let defaults: i64 = sqlx::query_scalar("SELECT count(*) FROM user_profiles WHERE user_id=$1")
        .bind(owner)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(defaults, 0);
    let reading = format!(
        "/api/v1/subjects/{subject}/readings/{}",
        run["id"].as_str().unwrap()
    );
    assert_eq!(
        call(&app, "GET", &reading, Some(&denied_token), Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&app, "GET", &reading, Some(&own_token), Value::Null)
            .await
            .0,
        StatusCode::OK
    );
    let a = json!({"expected_revision":1,"patch":{"full_name":"Concurrent A"}});
    let b = json!({"expected_revision":1,"patch":{"full_name":"Concurrent B"}});
    let (a, b) = tokio::join!(
        call(&app, "PATCH", &path, Some(&own_token), a),
        call(&app, "PATCH", &path, Some(&own_token), b)
    );
    assert!(
        (a.0 == StatusCode::OK && b.0 == StatusCode::CONFLICT)
            || (b.0 == StatusCode::OK && a.0 == StatusCode::CONFLICT)
    );
    // FORCE RLS tested as non-superuser, including a pooled transaction without context.
    sqlx::query("DO $$ BEGIN IF NOT EXISTS(SELECT 1 FROM pg_roles WHERE rolname='identity_test_reader') THEN CREATE ROLE identity_test_reader NOLOGIN; END IF; END $$").execute(&pool).await.unwrap();
    sqlx::query("GRANT SELECT ON subject_profile_revisions,subject_grants,user_account_state TO identity_test_reader").execute(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE identity_test_reader")
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM subject_profile_revisions WHERE subject_id=$1")
            .bind(subject)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(count, 0);
    sqlx::query("SELECT set_config('app.account_id',$1,true)")
        .bind(own.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM subject_profile_revisions WHERE subject_id=$1")
            .bind(subject)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(count, 2);
    tx.commit().await.unwrap();
    sqlx::query(
        "UPDATE subject_grants SET revoked_at=now() WHERE subject_id=$1 AND grantee_user_id=$2",
    )
    .bind(subject)
    .bind(owner)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        call(&app, "GET", &path, Some(&owner_token), Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    sqlx::query("UPDATE user_account_state SET state='locked',locked_until=now()+interval '1 hour' WHERE user_id=$1").bind(own).execute(&pool).await.unwrap();
    assert_eq!(
        call(&app, "GET", &path, Some(&own_token), Value::Null)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
}
#[tokio::test]
#[serial]
async fn registered_sessions_bind_client_rotate_revoke_and_block_password_email() {
    let (app, pool) = app().await;
    let uid = user(&pool).await;
    let issuer = "https://synthetic.cloudflareaccess.com";
    let sub = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO account_identities(issuer,provider_subject,user_id,verified_email) VALUES($1,$2,$3,$4)").bind(issuer).bind(&sub).bind(uid).bind(format!("{uid}@example.test")).execute(&pool).await.unwrap();
    let code = new_secret();
    sqlx::query("INSERT INTO registered_auth_codes(code_hash,client_id,user_id,code_challenge,redirect_uri,expires_at,identity_issuer,identity_subject) VALUES($1,'antahkarana',$2,$3,$4,now()+interval '1 minute',$5,$6)").bind(hash_secret(&code)).bind(uid).bind(s256_challenge(VERIFIER)).bind(REDIRECT).bind(issuer).bind(&sub).execute(&pool).await.unwrap();
    let exchange = json!({"grant_type":"authorization_code","client_id":"antahkarana","code":code,"redirect_uri":REDIRECT,"code_verifier":VERIFIER});
    let mut wrong = exchange.clone();
    wrong["client_id"] = json!("noesis-raycast");
    assert_eq!(
        call(&app, "POST", "/api/v1/sessions/token", None, wrong)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let (status, tokens) = call(
        &app,
        "POST",
        "/api/v1/sessions/token",
        None,
        exchange.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{tokens}");
    assert_eq!(
        call(&app, "POST", "/api/v1/sessions/token", None, exchange)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let access = tokens["access_token"].as_str().unwrap();
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/users/me/context",
            Some(access),
            Value::Null
        )
        .await
        .0,
        StatusCode::OK
    );
    // A password JWT cannot manufacture verified email authority.
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/account/email-links",
            Some(&jwt(&pool, uid)),
            json!({"email":"new@example.test"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/account/email-links",
            Some(access),
            json!({"email":"new@example.test"})
        )
        .await
        .0,
        StatusCode::OK
    );
    let refresh = json!({"grant_type":"refresh_token","client_id":"antahkarana","refresh_token":tokens["refresh_token"]});
    let (status, next) = call(
        &app,
        "POST",
        "/api/v1/sessions/token",
        None,
        refresh.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{next}");
    assert_eq!(
        call(&app, "POST", "/api/v1/sessions/token", None, refresh)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/users/me/context",
            Some(next["access_token"].as_str().unwrap()),
            Value::Null
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/users/me/context",
            Some(access),
            Value::Null
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test]
#[serial]
async fn verified_provider_subject_stays_stable_and_legacy_email_is_preserved() {
    let (_, pool) = app().await;
    let uid = user(&pool).await;
    let repo = UserRepository::new(pool.clone());
    let sub = Uuid::new_v4().to_string();
    let issuer = "https://stable.cloudflareaccess.com";
    let first = repo
        .find_or_create_verified_user(issuer, &format!("{uid}@example.test"), &sub)
        .await
        .unwrap();
    assert_eq!(first.id, uid);
    let changed = repo
        .find_or_create_verified_user(issuer, "changed@example.test", &sub)
        .await
        .unwrap();
    assert_eq!(changed.id, uid);
    assert_eq!(changed.email, first.email);
    sqlx::query(
        "UPDATE account_identities SET revoked_at=now() WHERE issuer=$1 AND provider_subject=$2",
    )
    .bind(issuer)
    .bind(&sub)
    .execute(&pool)
    .await
    .unwrap();
    assert!(repo
        .find_or_create_verified_user(issuer, &first.email, &sub)
        .await
        .is_err());
}
#[tokio::test]
#[serial]
async fn enrollment_pending_confirmation_replay_and_independent_relationships() {
    let (app, pool) = app().await;
    let owner = user(&pool).await;
    let claimant = user(&pool).await;
    let outsider = user(&pool).await;
    let issuer = "https://enroll.cloudflareaccess.com";
    let sub = Uuid::new_v4().to_string();
    let email = format!("{claimant}@example.test");
    sqlx::query("INSERT INTO account_identities(issuer,provider_subject,user_id,verified_email) VALUES($1,$2,$3,$4)").bind(issuer).bind(&sub).bind(claimant).bind(&email).execute(&pool).await.unwrap();
    let browser = app.clone();
    let subject:Uuid=sqlx::query_scalar("INSERT INTO corpus_subjects(owner_user_id,subject_key,canonical_name) VALUES($1,$2,'Synthetic enrollment') RETURNING id").bind(owner).bind(Uuid::new_v4().to_string()).fetch_one(&pool).await.unwrap();
    let owner_token = registered_jwt(&pool, owner).await;
    let claim_token = registered_jwt(&pool, claimant).await;
    let (status, invite) = call(
        &browser,
        "POST",
        "/api/v1/account/enrollments",
        Some(&owner_token),
        json!({"subject_id":subject,"sharing_reader_id":owner}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{invite}");
    let claim = json!({"token":invite["token"],"accept_sharing":true});
    let (status, pending) = call(
        &browser,
        "POST",
        "/api/v1/account/enrollments/claim",
        Some(&claim_token),
        claim.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{pending}");
    assert_eq!(pending["status"], "pending_owner_confirmation");
    let context = call(
        &app,
        "GET",
        "/api/v1/users/me/context",
        Some(&claim_token),
        Value::Null,
    )
    .await
    .1;
    assert_eq!(context["subjects"], json!([]));
    assert_eq!(
        call(
            &browser,
            "POST",
            "/api/v1/account/enrollments/claim",
            Some(&claim_token),
            claim
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let confirm = format!(
        "/api/v1/account/enrollments/{}/confirm",
        invite["id"].as_str().unwrap()
    );
    assert_eq!(
        call(
            &browser,
            "POST",
            &confirm,
            Some(&registered_jwt(&pool, outsider).await),
            json!({})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&browser, "POST", &confirm, Some(&owner_token), json!({}))
            .await
            .0,
        StatusCode::OK
    );
    let ctx = call(
        &app,
        "GET",
        "/api/v1/users/me/context",
        Some(&claim_token),
        Value::Null,
    )
    .await
    .1;
    assert_eq!(ctx["subjects"][0]["can_edit"], true);
    let ctx = call(
        &app,
        "GET",
        "/api/v1/users/me/context",
        Some(&owner_token),
        Value::Null,
    )
    .await
    .1;
    assert_eq!(ctx["subjects"][0]["can_edit"], false);
    assert_eq!(ctx["subjects"][0]["can_calculate"], false);
    let relationship:Uuid=sqlx::query_scalar("INSERT INTO corpus_relationships(owner_user_id,relationship_key,relationship_kind,label) VALUES($1,$2,'pair','Synthetic pair') RETURNING id").bind(owner).bind(Uuid::new_v4().to_string()).fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO relationship_grants(relationship_id,owner_user_id,grantee_user_id,grantor_user_id) VALUES($1,$2,$3,$2)").bind(relationship).bind(owner).bind(outsider).execute(&pool).await.unwrap();
    let ctx = call(
        &app,
        "GET",
        "/api/v1/users/me/context",
        Some(&registered_jwt(&pool, outsider).await),
        Value::Null,
    )
    .await
    .1;
    assert_eq!(ctx["subjects"], json!([]));
    assert_eq!(ctx["relationships"].as_array().unwrap().len(), 1);
    sqlx::query("UPDATE relationship_grants SET revoked_at=now() WHERE relationship_id=$1")
        .bind(relationship)
        .execute(&pool)
        .await
        .unwrap();
    let ctx = call(
        &app,
        "GET",
        "/api/v1/users/me/context",
        Some(&registered_jwt(&pool, outsider).await),
        Value::Null,
    )
    .await
    .1;
    assert_eq!(ctx["relationships"], json!([]));
}
#[tokio::test]
#[serial]
async fn email_link_verifies_signature_audience_and_single_use_without_merging_accounts() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };
    let dir = tempfile::tempdir().unwrap();
    let key = dir.path().join("synthetic.pem");
    assert!(std::process::Command::new("openssl")
        .args(["genrsa", "-out"])
        .arg(&key)
        .arg("2048")
        .output()
        .unwrap()
        .status
        .success());
    let modulus = std::process::Command::new("openssl")
        .args(["rsa", "-in"])
        .arg(&key)
        .args(["-modulus", "-noout"])
        .output()
        .unwrap();
    assert!(modulus.status.success());
    let text = String::from_utf8(modulus.stdout).unwrap();
    let hex = text.trim().strip_prefix("Modulus=").unwrap();
    let n: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    let server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/cdn-cgi/access/certs")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"keys":[{"kid":"synthetic","kty":"RSA","n":URL_SAFE_NO_PAD.encode(n),"e":"AQAB"}]}))).mount(&server).await;
    std::env::set_var("TEST_SHARED_CF_ISSUER", server.uri());
    let (app, pool) = app().await;
    std::env::remove_var("TEST_SHARED_CF_ISSUER");
    let uid = user(&pool).await;
    let original = format!("{uid}@example.test");
    let desired = format!("new-{uid}@example.test");
    let browser = app
        .clone()
        .layer(axum::Extension(noesis_api::cf_access::CfIdentity {
            issuer: server.uri(),
            sub: Uuid::new_v4().to_string(),
            email: original.clone(),
            groups: vec![],
        }));
    let (_, challenge) = call(
        &browser,
        "POST",
        "/api/v1/account/email-links",
        Some(&jwt(&pool, uid)),
        json!({"email":desired}),
    )
    .await;
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(&std::fs::read(key).unwrap()).unwrap();
    let proof = |email: &str, audience: &str| {
        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some("synthetic".into());
        jsonwebtoken::encode(&header,&json!({"iss":server.uri(),"sub":format!("sub-{email}"),"email":email,"aud":[audience],"iat":chrono::Utc::now().timestamp(),"exp":chrono::Utc::now().timestamp()+300}),&key).unwrap()
    };
    std::env::set_var("NOESIS_ACCOUNT_ORIGIN", "https://account.example.test");
    sqlx::query("INSERT INTO account_identities(issuer,provider_subject,user_id,verified_email) VALUES($1,$2,$3,$4)").bind(server.uri()).bind(format!("sub-{original}")).bind(uid).bind(&original).execute(&pool).await.unwrap();
    async fn browser_post(
        app: &axum::Router,
        path: &str,
        challenge: &Value,
        proof: String,
        cookie: Option<&str>,
        origin: &str,
    ) -> (StatusCode, Value, Option<String>) {
        let mut req = Request::post(path)
            .header("content-type", "application/json")
            .header("origin", origin)
            .header("cf-access-jwt-assertion", proof);
        if let Some(cookie) = cookie {
            req = req.header("cookie", cookie);
        }
        let response = app
            .clone()
            .oneshot(
                req.body(Body::from(
                    json!({"challenge":challenge["challenge"],"confirm":true}).to_string(),
                ))
                .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let cookie = response
            .headers()
            .get("set-cookie")
            .map(|h| h.to_str().unwrap().split(';').next().unwrap().to_string());
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
            cookie,
        )
    }
    let origin = "https://account.example.test";
    // The original attack: an unregistered victim's fresh identity plus stolen URL
    // cannot bind the browser or complete the attacker's pending email change.
    assert_eq!(
        browser_post(
            &app,
            "/api/v1/account/email-links/begin",
            &challenge,
            proof(&desired, "shared-test-audience"),
            None,
            origin
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        browser_post(
            &app,
            "/api/v1/account/email-links/complete",
            &challenge,
            proof(&desired, "shared-test-audience"),
            None,
            origin
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    for page in ["begin", "verify"] {
        let uri = format!(
            "/api/v1/account/email-links/{page}?challenge={}",
            challenge["challenge"].as_str().unwrap()
        );
        assert_eq!(
            call(&app, "GET", &uri, None, Value::Null).await.0,
            StatusCode::OK
        );
    }
    let bound:bool=sqlx::query_scalar("SELECT browser_bound_at IS NOT NULL OR consumed_at IS NOT NULL FROM account_email_links WHERE token_hash=$1").bind(hash_secret(challenge["challenge"].as_str().unwrap())).fetch_one(&pool).await.unwrap();
    assert!(!bound);
    let (status, _, cookie) = browser_post(
        &app,
        "/api/v1/account/email-links/begin",
        &challenge,
        proof(&original, "shared-test-audience"),
        None,
        origin,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let cookie = cookie.unwrap();
    assert_eq!(
        browser_post(
            &app,
            "/api/v1/account/email-links/complete",
            &challenge,
            proof(&desired, "shared-test-audience"),
            Some(&cookie),
            "https://attacker.example.test"
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        browser_post(
            &app,
            "/api/v1/account/email-links/complete",
            &challenge,
            proof(&desired, "wrong-audience"),
            Some(&cookie),
            origin
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let (status, result, _) = browser_post(
        &app,
        "/api/v1/account/email-links/complete",
        &challenge,
        proof(&desired, "shared-test-audience"),
        Some(&cookie),
        origin,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["account_id"], uid.to_string());
    assert_eq!(result["preferred_email"], desired);
    assert_eq!(
        browser_post(
            &app,
            "/api/v1/account/email-links/complete",
            &challenge,
            proof(&desired, "shared-test-audience"),
            Some(&cookie),
            origin
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let stored: String = sqlx::query_scalar("SELECT email FROM users WHERE id=$1")
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, original);
    let other = user(&pool).await;
    let taken = format!("{other}@example.test");
    let (_, challenge) = call(
        &browser,
        "POST",
        "/api/v1/account/email-links",
        Some(&jwt(&pool, uid)),
        json!({"email":taken}),
    )
    .await;
    let (status, _, cookie) = browser_post(
        &app,
        "/api/v1/account/email-links/begin",
        &challenge,
        proof(&original, "shared-test-audience"),
        None,
        origin,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let cookie = cookie.unwrap();
    assert_eq!(
        browser_post(
            &app,
            "/api/v1/account/email-links/complete",
            &challenge,
            proof(&taken, "shared-test-audience"),
            Some(&cookie),
            origin
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let preferred: String =
        sqlx::query_scalar("SELECT preferred_email FROM account_preferences WHERE user_id=$1")
            .bind(uid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(preferred, desired);
}
#[tokio::test]
#[serial]
async fn bounded_runtime_role_enforces_rls_and_resets_pooled_context() {
    let runtime = std::env::var("TEST_SHARED_RUNTIME_URL").expect(
        "bounded runtime URL required for role verification \
         (scripts/cloud/test-db.sh provisions one and prints it)",
    );
    let runtime = PgPoolOptions::new()
        .max_connections(1)
        .connect(&runtime)
        .await
        .unwrap();
    let flags: (bool, bool) =
        sqlx::query_as("SELECT rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user")
            .fetch_one(&runtime)
            .await
            .unwrap();
    assert_eq!(flags, (false, false));
    let (_, pool) = app().await;
    let migration_role: String = sqlx::query_scalar("SELECT current_user::text")
        .fetch_one(&pool)
        .await
        .unwrap();
    let switch = format!("SET ROLE \"{}\"", migration_role.replace('"', "\"\""));
    assert!(sqlx::query(&switch).execute(&runtime).await.is_err());
    assert!(
        sqlx::query("CREATE TABLE public.runtime_escape_probe(id integer)")
            .execute(&runtime)
            .await
            .is_err()
    );
    let uid = user(&pool).await;
    let subject:Uuid=sqlx::query_scalar("INSERT INTO corpus_subjects(owner_user_id,subject_key,canonical_name) VALUES($1,$2,'Runtime RLS synthetic') RETURNING id").bind(uid).bind(Uuid::new_v4().to_string()).fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO subject_grants(subject_id,owner_user_id,grantee_user_id,grantor_user_id,can_edit) VALUES($1,$2,$2,$2,true)").bind(subject).bind(uid).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO subject_profile_revisions(subject_id,owner_user_id,revision,profile,source_hash,changed_by) VALUES($1,$2,1,'{}',$3,$2)").bind(subject).bind(uid).bind("a".repeat(64)).execute(&pool).await.unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM subject_profile_revisions WHERE subject_id=$1")
            .bind(subject)
            .fetch_one(&runtime)
            .await
            .unwrap();
    assert_eq!(count, 0);
    let mut tx = runtime.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.account_id',$1,true)")
        .bind(uid.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM subject_profile_revisions WHERE subject_id=$1")
            .bind(subject)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(count, 1);
    tx.commit().await.unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM subject_profile_revisions WHERE subject_id=$1")
            .bind(subject)
            .fetch_one(&runtime)
            .await
            .unwrap();
    assert_eq!(count, 0);
    let configured = std::env::var("TEST_SHARED_RUNTIME_URL").unwrap();
    std::env::remove_var("TEST_SHARED_RUNTIME_URL");
    let (unbounded, _) = app().await;
    std::env::set_var("TEST_SHARED_RUNTIME_URL", configured);
    // `app()` runs with RUST_ENV=development, where the privileged-connection
    // guard is opt-in (docs/shared-identity/ROLLOUT.md). Turn it on explicitly
    // so this proves a superuser/BYPASSRLS pool is refused when enforced.
    let _enforce = EnvVarGuard::set("NOESIS_REQUIRE_BOUNDED_DB_ROLE", "true");
    assert_eq!(
        call(
            &unbounded,
            "GET",
            "/api/v1/users/me/context",
            Some(&jwt(&pool, uid)),
            Value::Null
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
}
#[tokio::test]
#[serial]
async fn intended_email_gates_preview_and_claim_without_changing_the_owner_grant() {
    let (app, pool) = app().await;
    let owner = user(&pool).await;
    let recipient = user(&pool).await;
    let outsider = user(&pool).await;
    let owner_token = registered_jwt(&pool, owner).await;
    let recipient_token = registered_jwt(&pool, recipient).await;
    let outsider_token = registered_jwt(&pool, outsider).await;
    let subject: Uuid = sqlx::query_scalar(
        "INSERT INTO corpus_subjects(owner_user_id,subject_key,canonical_name) VALUES($1,$2,'Private enrollment subject') RETURNING id",
    )
    .bind(owner)
    .bind(Uuid::new_v4().to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO subject_grants(subject_id,owner_user_id,grantee_user_id,grantor_user_id) VALUES($1,$2,$2,$2)")
        .bind(subject).bind(owner).execute(&pool).await.unwrap();
    let (status, invite) = call(
        &app,
        "POST",
        "/api/v1/account/enrollments",
        Some(&owner_token),
        json!({"subject_id":subject,"intended_email":format!("{recipient}@example.test")}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let token = json!({"token":invite["token"]});
    let preview_path = "/api/v1/account/enrollments/preview";
    let mut denied = call(
        &app,
        "POST",
        preview_path,
        Some(&outsider_token),
        token.clone(),
    )
    .await;
    let mut absent = call(
        &app,
        "POST",
        preview_path,
        Some(&outsider_token),
        json!({"token":new_secret()}),
    )
    .await;
    assert_eq!(denied.0, StatusCode::NOT_FOUND);
    denied.1.as_object_mut().unwrap().remove("trace_id");
    absent.1.as_object_mut().unwrap().remove("trace_id");
    assert_eq!(denied, absent);
    let (status, preview) = call(&app, "POST", preview_path, Some(&recipient_token), token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(preview["display_name"], "Private enrollment subject");
    assert_eq!(
        preview["owner_reader"],
        json!({"id":owner,"name":"Synthetic"})
    );
    assert!(preview["sharing_reader_id"].is_null());
    assert_eq!(preview["relationships"], json!([]));
    assert_eq!(preview["requires_owner_confirmation"], false);

    let claim_path = "/api/v1/account/enrollments/claim";
    for (token, accepts, expected) in [
        (&outsider_token, true, StatusCode::FORBIDDEN),
        (&recipient_token, false, StatusCode::BAD_REQUEST),
    ] {
        assert_eq!(
            call(
                &app,
                "POST",
                claim_path,
                Some(token),
                json!({"token":invite["token"],"accept_sharing":accepts})
            )
            .await
            .0,
            expected
        );
    }
    let ctx = "/api/v1/users/me/context";
    assert_eq!(
        call(&app, "GET", ctx, Some(&recipient_token), Value::Null)
            .await
            .1["subjects"],
        json!([])
    );
    let claim = json!({"token":invite["token"],"accept_sharing":true});
    let (status, active) = call(
        &app,
        "POST",
        claim_path,
        Some(&recipient_token),
        claim.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(active["status"], "active");
    assert_eq!(
        call(&app, "POST", claim_path, Some(&recipient_token), claim)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );

    let (status, _) = call(
        &app,
        "PATCH",
        ctx,
        Some(&recipient_token),
        json!({"default_subject_id":subject}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, persisted) = call(&app, "GET", ctx, Some(&recipient_token), Value::Null).await;
    assert_eq!(persisted["default_subject_id"], subject.to_string());
    assert_eq!(persisted["subjects"].as_array().unwrap().len(), 1);
    assert_eq!(persisted["subjects"][0]["id"], subject.to_string());
    assert_eq!(persisted["subjects"][0]["can_edit"], true);
    assert_eq!(persisted["relationships"], json!([]));
    let (_, owner_context) = call(&app, "GET", ctx, Some(&owner_token), Value::Null).await;
    assert_eq!(owner_context["subjects"][0]["can_edit"], false);
    assert_eq!(owner_context["subjects"][0]["can_calculate"], false);
}
#[tokio::test]
#[serial]
async fn enrollment_pins_only_owned_member_relationships_and_grants_after_confirmation() {
    let (app, pool) = app().await;
    let owner = user(&pool).await;
    let recipient = user(&pool).await;
    let other = user(&pool).await;
    let owner_token = registered_jwt(&pool, owner).await;
    let recipient_token = registered_jwt(&pool, recipient).await;
    async fn subject(pool: &PgPool, owner: Uuid) -> Uuid {
        sqlx::query_scalar("INSERT INTO corpus_subjects(owner_user_id,subject_key,canonical_name) VALUES($1,$2,'Pinned pair synthetic') RETURNING id").bind(owner).bind(Uuid::new_v4().to_string()).fetch_one(pool).await.unwrap()
    }
    let selected = subject(&pool, owner).await;
    let nonmember = subject(&pool, owner).await;
    let pair:Uuid=sqlx::query_scalar("INSERT INTO corpus_relationships(owner_user_id,relationship_key,relationship_kind,label) VALUES($1,$2,'pair','Explicit joint projection') RETURNING id").bind(owner).bind(Uuid::new_v4().to_string()).fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO corpus_relationship_members(relationship_id,subject_id,owner_user_id,member_role,position) VALUES($1,$2,$3,'member',1)").bind(pair).bind(selected).bind(owner).execute(&pool).await.unwrap();
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/account/enrollments",
            Some(&owner_token),
            json!({"subject_id":nonmember,"relationship_ids":[pair]})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let other_pair:Uuid=sqlx::query_scalar("INSERT INTO corpus_relationships(owner_user_id,relationship_key,relationship_kind,label) VALUES($1,$2,'pair','Other owner') RETURNING id").bind(other).bind(Uuid::new_v4().to_string()).fetch_one(&pool).await.unwrap();
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/account/enrollments",
            Some(&owner_token),
            json!({"subject_id":selected,"relationship_ids":[other_pair]})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (status, invite) = call(
        &app,
        "POST",
        "/api/v1/account/enrollments",
        Some(&owner_token),
        json!({"subject_id":selected,"relationship_ids":[pair],"sharing_reader_id":owner}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{invite}");
    let (status, preview) = call(
        &app,
        "POST",
        "/api/v1/account/enrollments/preview",
        Some(&recipient_token),
        json!({"token":invite["token"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        preview["relationships"],
        json!([{"id":pair,"label":"Explicit joint projection"}])
    );
    let (status, claim) = call(
        &app,
        "POST",
        "/api/v1/account/enrollments/claim",
        Some(&recipient_token),
        json!({"token":invite["token"],"accept_sharing":true}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(claim["status"], "pending_owner_confirmation");
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/users/me/context",
            Some(&recipient_token),
            Value::Null
        )
        .await
        .1["relationships"],
        json!([])
    );
    sqlx::query("UPDATE corpus_relationships SET reconciliation_state='archived',archived_at=now() WHERE id=$1").bind(pair).execute(&pool).await.unwrap();
    let path = format!(
        "/api/v1/account/enrollments/{}/confirm",
        invite["id"].as_str().unwrap()
    );
    assert_eq!(
        call(&app, "POST", &path, Some(&owner_token), json!({}))
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/account/enrollments",
            Some(&owner_token),
            json!({"subject_id":selected,"relationship_ids":[pair]})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    sqlx::query("UPDATE corpus_relationships SET reconciliation_state='verified',archived_at=NULL WHERE id=$1").bind(pair).execute(&pool).await.unwrap();
    assert_eq!(
        call(&app, "POST", &path, Some(&owner_token), json!({}))
            .await
            .0,
        StatusCode::OK
    );
    let ctx = call(
        &app,
        "GET",
        "/api/v1/users/me/context",
        Some(&recipient_token),
        Value::Null,
    )
    .await
    .1;
    assert_eq!(ctx["relationships"][0]["id"], pair.to_string());
    assert_eq!(ctx["subjects"][0]["can_edit"], true);
    assert_eq!(ctx["subjects"][0]["can_manage_enrollment"], false);
    sqlx::query("UPDATE relationship_grants SET revoked_at=now() WHERE relationship_id=$1 AND grantee_user_id=$2").bind(pair).bind(recipient).execute(&pool).await.unwrap();
    let ctx = call(
        &app,
        "GET",
        "/api/v1/users/me/context",
        Some(&recipient_token),
        Value::Null,
    )
    .await
    .1;
    assert_eq!(ctx["relationships"], json!([]));
    assert_eq!(ctx["subjects"].as_array().unwrap().len(), 1);
}
#[tokio::test]
#[serial]
async fn service_api_key_reads_owned_context_but_cannot_operate_enrollment_or_email() {
    let (app, pool) = app().await;
    let owner = user(&pool).await;
    let key = new_secret();
    AuthService::with_pool(JWT_SECRET.into(), Some(pool.clone()))
        .add_api_key(noesis_auth::ApiKey {
            key: key.clone(),
            user_id: owner.to_string(),
            tier: "free".into(),
            permissions: vec!["basic:access".into()],
            created_at: chrono::Utc::now(),
            expires_at: None,
            last_used: None,
            rate_limit: 1000,
            consciousness_level: 0,
        })
        .await
        .unwrap();
    async fn request(
        app: &axum::Router,
        key: &str,
        method: &str,
        path: &str,
        body: Value,
    ) -> StatusCode {
        app.clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("x-api-key", key)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }
    assert_eq!(
        request(&app, &key, "GET", "/api/v1/users/me/context", Value::Null).await,
        StatusCode::OK
    );
    for (method, path, body) in [
        (
            "GET",
            "/api/v1/account/enrollments".to_string(),
            Value::Null,
        ),
        (
            "POST",
            "/api/v1/account/enrollments".to_string(),
            json!({"subject_id":Uuid::new_v4()}),
        ),
        (
            "POST",
            "/api/v1/account/enrollments/preview".to_string(),
            json!({"token":new_secret()}),
        ),
        (
            "POST",
            "/api/v1/account/enrollments/claim".to_string(),
            json!({"token":new_secret(),"accept_sharing":true}),
        ),
        (
            "POST",
            format!("/api/v1/account/enrollments/{}/confirm", Uuid::new_v4()),
            json!({}),
        ),
        (
            "DELETE",
            format!("/api/v1/account/enrollments/{}", Uuid::new_v4()),
            Value::Null,
        ),
        (
            "POST",
            "/api/v1/account/email-links".to_string(),
            json!({"email":"synthetic@example.test"}),
        ),
    ] {
        assert_eq!(
            request(&app, &key, method, &path, body).await,
            StatusCode::FORBIDDEN,
            "{method} {path}"
        );
    }
}

const DAILY_REDIRECT: &str = "https://daily.example.test/auth/callback";
const ENROLLMENTS: &str = "/api/v1/account/enrollments";
const ACCOUNT_CONTEXT: &str = "/api/v1/users/me/context";

async fn daily_exchange(pool: &PgPool, uid: Uuid, asserted_email: &str) -> Value {
    let code = new_secret();
    sqlx::query(
        "INSERT INTO registered_auth_codes(code_hash,client_id,user_id,code_challenge,redirect_uri,
             expires_at,identity_issuer,identity_subject,enrollment_email)
         SELECT $1,'noesis-daily',user_id,$3,$4,now()+interval '1 minute',issuer,provider_subject,$5
         FROM account_identities WHERE user_id=$2 AND revoked_at IS NULL",
    )
    .bind(hash_secret(&code))
    .bind(uid)
    .bind(s256_challenge(VERIFIER))
    .bind(DAILY_REDIRECT)
    .bind(asserted_email)
    .execute(pool)
    .await
    .unwrap();
    json!({"grant_type":"authorization_code","client_id":"noesis-daily",
           "code":code,"redirect_uri":DAILY_REDIRECT,"code_verifier":VERIFIER})
}

#[tokio::test]
#[serial]
async fn verified_daily_sign_in_reuses_prepared_profile_and_keeps_sharing_optional() {
    let _redirect = EnvVarGuard::set("NOESIS_DAILY_REDIRECT_URIS", DAILY_REDIRECT);
    let (app, pool) = app().await;
    let owner = user(&pool).await;
    let recipient = user(&pool).await;
    let outsider = user(&pool).await;
    let reader = user(&pool).await;
    let owner_token = registered_jwt(&pool, owner).await;
    let prior_token = registered_jwt(&pool, recipient).await;
    let outsider_token = registered_jwt(&pool, outsider).await;
    let recipient_email = format!("{recipient}@example.test");
    let subject: Uuid = sqlx::query_scalar(
        "INSERT INTO corpus_subjects(owner_user_id,subject_key,canonical_name)
         VALUES($1,$2,'Prepared profile') RETURNING id",
    )
    .bind(owner)
    .bind(Uuid::new_v4().to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO subject_grants(subject_id,owner_user_id,grantee_user_id,grantor_user_id)
         VALUES($1,$2,$2,$2),($1,$2,$3,$2)",
    )
    .bind(subject)
    .bind(owner)
    .bind(recipient)
    .execute(&pool)
    .await
    .unwrap();
    let prepared = json!({"full_name":"Prepared profile","birth_date":"1990-01-01",
                          "timezone":"Asia/Kolkata","preferences":{}});
    let prepared_hash = hash_secret(&prepared.to_string());
    sqlx::query(
        "INSERT INTO subject_profile_revisions(subject_id,owner_user_id,revision,profile,source_hash,changed_by)
         VALUES($1,$2,1,$3,$4,$2)",
    )
    .bind(subject)
    .bind(owner)
    .bind(&prepared)
    .bind(&prepared_hash)
    .execute(&pool)
    .await
    .unwrap();
    let pair: Uuid = sqlx::query_scalar(
        "INSERT INTO corpus_relationships(owner_user_id,relationship_key,relationship_kind,label)
         VALUES($1,$2,'pair','Approved joint view') RETURNING id",
    )
    .bind(owner)
    .bind(Uuid::new_v4().to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO corpus_relationship_members(relationship_id,subject_id,owner_user_id,member_role,position)
         VALUES($1,$2,$3,'member',1)",
    )
    .bind(pair)
    .bind(subject)
    .bind(owner)
    .execute(&pool)
    .await
    .unwrap();
    let request = json!({"mode":"sign_in","subject_id":subject,
        "intended_email":format!(" {} ", recipient_email.to_ascii_uppercase()),
        "sharing_reader_id":reader,"relationship_ids":[pair]});
    assert_eq!(
        call(
            &app,
            "POST",
            ENROLLMENTS,
            Some(&prior_token),
            request.clone()
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (status, invite) = call(
        &app,
        "POST",
        ENROLLMENTS,
        Some(&owner_token),
        request.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{invite}");
    assert_eq!(invite["status"], "pending_sign_in");
    assert!(invite.get("token").is_none());
    assert_eq!(
        call(&app, "POST", ENROLLMENTS, Some(&owner_token), request)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let (_, before) = call(
        &app,
        "GET",
        ACCOUNT_CONTEXT,
        Some(&prior_token),
        Value::Null,
    )
    .await;
    assert!(before["self_subject_id"].is_null());
    assert_eq!(before["subjects"][0]["can_edit"], false);

    let mut wrong_pkce = daily_exchange(&pool, recipient, &recipient_email).await;
    wrong_pkce["code_verifier"] = json!(new_secret());
    assert_eq!(
        call(&app, "POST", "/api/v1/sessions/token", None, wrong_pkce)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let (status, wrong_address) = call(
        &app,
        "POST",
        "/api/v1/sessions/token",
        None,
        daily_exchange(&pool, outsider, &recipient_email).await,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, denied) = call(
        &app,
        "GET",
        ACCOUNT_CONTEXT,
        wrong_address["access_token"].as_str(),
        Value::Null,
    )
    .await;
    assert_eq!(denied["subjects"], json!([]));
    assert!(denied["self_subject_id"].is_null());

    let (status, tokens) = call(
        &app,
        "POST",
        "/api/v1/sessions/token",
        None,
        daily_exchange(&pool, recipient, &recipient_email).await,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let access = tokens["access_token"].as_str();
    let (_, opened) = call(&app, "GET", ACCOUNT_CONTEXT, access, Value::Null).await;
    assert_eq!(opened["self_subject_id"], json!(subject));
    assert_eq!(opened["default_subject_id"], json!(subject));
    assert_eq!(opened["subjects"][0]["can_edit"], true);
    assert_eq!(opened["subjects"][0]["profile_revision"], 1);
    assert_eq!(opened["relationships"], json!([]));
    let path = format!("/api/v1/subjects/{subject}/profile");
    let (_, saved) = call(&app, "GET", &path, access, Value::Null).await;
    assert_eq!(saved["profile"], prepared);
    let stored: (String, String) = sqlx::query_as(
        "SELECT p.source_hash,u.birth_date::text FROM subject_profile_revisions p
         JOIN user_profiles u ON u.user_id=$2 WHERE p.subject_id=$1",
    )
    .bind(subject)
    .bind(recipient)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stored, (prepared_hash, "1990-01-01".into()));
    let (_, sharing) = call(&app, "GET", "/api/v1/account/sharing", access, Value::Null).await;
    assert_eq!(sharing["readers"], json!(["Synthetic"]));
    assert_eq!(sharing["offers"][0]["id"], invite["id"]);
    assert_eq!(sharing["offers"][0]["relationships"][0]["id"], json!(pair));
    let accept = format!("{ENROLLMENTS}/{}/sharing", invite["id"].as_str().unwrap());
    let joint = json!({"relationship_ids":[pair]});
    assert_eq!(
        call(&app, "POST", &accept, Some(&outsider_token), joint.clone())
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &app,
            "POST",
            &accept,
            access,
            json!({"reader":true,"relationship_ids":[Uuid::new_v4()]})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&app, "POST", &accept, access, joint.clone()).await.0,
        StatusCode::OK
    );
    let (_, opened) = call(&app, "GET", ACCOUNT_CONTEXT, access, Value::Null).await;
    assert_eq!(opened["relationships"][0]["id"], json!(pair));
    let (_, offer) = call(&app, "GET", "/api/v1/account/sharing", access, Value::Null).await;
    assert_eq!(offer["offers"][0]["reader"], "Synthetic");
    assert_eq!(offer["offers"][0]["relationships"], json!([]));
    assert_eq!(
        call(&app, "POST", &accept, access, json!({"reader":true}))
            .await
            .0,
        StatusCode::OK
    );
    let new_grant: (bool, bool) = sqlx::query_as(
        "SELECT can_edit,can_calculate FROM subject_grants WHERE subject_id=$1 AND grantee_user_id=$2",
    ).bind(subject).bind(reader).fetch_one(&pool).await.unwrap();
    assert_eq!(new_grant, (false, false));
    sqlx::query("UPDATE relationship_grants SET revoked_at=now() WHERE relationship_id=$1 AND grantee_user_id=$2")
        .bind(pair).bind(recipient).execute(&pool).await.unwrap();
    assert_eq!(
        call(&app, "POST", &accept, access, joint).await.0,
        StatusCode::OK
    );
    let (_, final_context) = call(&app, "GET", ACCOUNT_CONTEXT, access, Value::Null).await;
    assert_eq!(final_context["relationships"], json!([]));
}

#[tokio::test]
#[serial]
async fn verified_daily_sign_in_creates_one_private_profile_after_owner_authorization() {
    let _redirect = EnvVarGuard::set("NOESIS_DAILY_REDIRECT_URIS", DAILY_REDIRECT);
    let (app, pool) = app().await;
    let admin = user(&pool).await;
    let invitee = user(&pool).await;
    let admin_token = registered_jwt(&pool, admin).await;
    let _invitee_token = registered_jwt(&pool, invitee).await;
    let email = format!("{invitee}@example.test");
    let request = json!({"mode":"sign_in","subject_id":null,"intended_email":email});
    assert_eq!(
        call(
            &app,
            "POST",
            ENROLLMENTS,
            Some(&admin_token),
            request.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    sqlx::query("INSERT INTO user_roles(user_id,role) VALUES($1,'platform-admin')")
        .bind(admin)
        .execute(&pool)
        .await
        .unwrap();
    let (status, first) = call(
        &app,
        "POST",
        ENROLLMENTS,
        Some(&admin_token),
        request.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let withdraw = format!("{ENROLLMENTS}/{}", first["id"].as_str().unwrap());
    assert_eq!(
        call(&app, "DELETE", &withdraw, Some(&admin_token), Value::Null)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    let (status, unassigned) = call(
        &app,
        "POST",
        "/api/v1/sessions/token",
        None,
        daily_exchange(&pool, invitee, &email).await,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, unassigned) = call(
        &app,
        "GET",
        ACCOUNT_CONTEXT,
        unassigned["access_token"].as_str(),
        Value::Null,
    )
    .await;
    assert_eq!(unassigned["subjects"], json!([]));
    assert!(unassigned["self_subject_id"].is_null());

    assert_eq!(
        call(
            &app,
            "POST",
            ENROLLMENTS,
            Some(&admin_token),
            request.clone()
        )
        .await
        .0,
        StatusCode::OK
    );
    let (one, two) = tokio::join!(
        daily_exchange(&pool, invitee, &email),
        daily_exchange(&pool, invitee, &email),
    );
    let ((status_one, tokens_one), (status_two, tokens_two)) = tokio::join!(
        call(&app, "POST", "/api/v1/sessions/token", None, one),
        call(&app, "POST", "/api/v1/sessions/token", None, two),
    );
    assert_eq!(status_one, StatusCode::OK);
    assert_eq!(status_two, StatusCode::OK);
    let access = tokens_one["access_token"].as_str();
    let (_, first_context) = call(&app, "GET", ACCOUNT_CONTEXT, access, Value::Null).await;
    let subject = first_context["self_subject_id"].as_str().unwrap();
    assert_eq!(first_context["default_subject_id"], json!(subject));
    let (_, second_context) = call(
        &app,
        "GET",
        ACCOUNT_CONTEXT,
        tokens_two["access_token"].as_str(),
        Value::Null,
    )
    .await;
    assert_eq!(first_context, second_context);
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM corpus_subjects WHERE owner_user_id=$1")
            .bind(invitee)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
    let path = format!("/api/v1/subjects/{subject}/profile");
    let (_, profile) = call(&app, "GET", &path, access, Value::Null).await;
    assert_eq!(profile["revision"], 1);
    assert!(profile["profile"]["full_name"].is_null());
    assert_eq!(
        call(&app, "GET", &path, Some(&admin_token), Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let (status, _) = call(
        &app,
        "PATCH",
        &path,
        access,
        json!({"expected_revision":1,"patch":{"full_name":"Saved name"}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // A second authorization for an already linked account cannot create another subject.
    assert_eq!(
        call(&app, "POST", ENROLLMENTS, Some(&admin_token), request)
            .await
            .0,
        StatusCode::OK
    );
    let (status, fresh) = call(
        &app,
        "POST",
        "/api/v1/sessions/token",
        None,
        daily_exchange(&pool, invitee, &email).await,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, profile) = call(
        &app,
        "GET",
        &path,
        fresh["access_token"].as_str(),
        Value::Null,
    )
    .await;
    assert_eq!(profile["revision"], 2);
    assert_eq!(profile["profile"]["full_name"], "Saved name");
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM corpus_subjects WHERE owner_user_id=$1")
            .bind(invitee)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
}
