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
    (create_router(state, &config), pool)
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
    let browser = app
        .clone()
        .layer(axum::Extension(noesis_api::cf_access::CfIdentity {
            issuer: issuer.into(),
            sub,
            email,
            groups: vec![],
        }));
    let subject:Uuid=sqlx::query_scalar("INSERT INTO corpus_subjects(owner_user_id,subject_key,canonical_name) VALUES($1,$2,'Synthetic enrollment') RETURNING id").bind(owner).bind(Uuid::new_v4().to_string()).fetch_one(&pool).await.unwrap();
    let owner_token = jwt(&pool, owner);
    let claim_token = jwt(&pool, claimant);
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
            Some(&jwt(&pool, outsider)),
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
        Some(&jwt(&pool, outsider)),
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
        Some(&jwt(&pool, outsider)),
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
    async fn finish(app: &axum::Router, challenge: &Value, proof: String) -> (StatusCode, Value) {
        let req = Request::post("/api/v1/account/email-links/complete")
            .header("content-type", "application/json")
            .header("cf-access-jwt-assertion", proof)
            .body(Body::from(
                json!({"challenge":challenge["challenge"]}).to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }
    assert_eq!(
        finish(&app, &challenge, proof(&desired, "wrong-audience"))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let (status, result) = finish(&app, &challenge, proof(&desired, "shared-test-audience")).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["account_id"], uid.to_string());
    assert_eq!(result["preferred_email"], desired);
    assert_eq!(
        finish(&app, &challenge, proof(&desired, "shared-test-audience"))
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
    assert_eq!(
        finish(&app, &challenge, proof(&taken, "shared-test-audience"))
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
