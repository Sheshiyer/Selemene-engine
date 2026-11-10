//! Synthetic HTTP coverage using only an explicitly provisioned loopback database.
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use chrono::{Duration, NaiveDate, Utc};
use noesis_api::{build_app_state_lazy_db, create_router, ApiConfig, AppState};
use noesis_auth::ApiKey;
use serde_json::{json, Value};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use tower::ServiceExt;
use uuid::Uuid;

async fn account(db: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO users(id,email,password_hash) VALUES($1,$2,'synthetic')")
        .bind(id)
        .bind(format!("{id}@example.test"))
        .execute(db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO user_account_state(user_id,state) VALUES($1,'active') ON CONFLICT(user_id) DO UPDATE SET state='active'")
        .bind(id).execute(db).await.unwrap();
    id
}
async fn browser(db: &PgPool, state: &AppState, id: Uuid) -> String {
    let provider_subject = format!("verified-{id}");
    sqlx::query("INSERT INTO account_identities(issuer,provider_subject,user_id,verified_email) VALUES('https://fixture.example.test',$1,$2,$3)")
        .bind(&provider_subject).bind(id).bind(format!("{id}@example.test")).execute(db).await.unwrap();
    let token = state
        .auth
        .generate_jwt_token(
            &id.to_string(),
            "enterprise",
            &["session:registered".into()],
            5,
        )
        .unwrap();
    let jti = state
        .auth
        .validate_jwt_token(&token)
        .await
        .unwrap()
        .jti
        .unwrap();
    let family:Uuid = sqlx::query_scalar(
        "INSERT INTO registered_session_families(client_id,user_id,identity_issuer,identity_subject,expires_at)
         VALUES('noesis-daily',$1,'https://fixture.example.test',$2,now()+interval '1 hour') RETURNING id")
        .bind(id).bind(provider_subject).fetch_one(db).await.unwrap();
    sqlx::query("INSERT INTO registered_access_tokens(jti,family_id,expires_at) VALUES($1,$2,now()+interval '1 hour')")
        .bind(jti).bind(family).execute(db).await.unwrap();
    token
}
async fn grant(db: &PgPool, subject: Uuid, owner: Uuid, grantee: Uuid, calculate: bool) {
    sqlx::query("INSERT INTO subject_grants(subject_id,owner_user_id,grantee_user_id,grantor_user_id,can_calculate) VALUES($1,$2,$3,$2,$4)")
        .bind(subject).bind(owner).bind(grantee).bind(calculate).execute(db).await.unwrap();
}
async fn subject(db: &PgPool, owner: Uuid, recipient: Uuid, worker: Uuid) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let profile = format!("fixture-{}", id.simple());
    sqlx::query("INSERT INTO corpus_subjects(id,owner_user_id,subject_key,canonical_name) VALUES($1,$2,$3,'Witness fixture')")
        .bind(id).bind(owner).bind(&profile).execute(db).await.unwrap();
    for (grantee, calculate) in [(owner, false), (recipient, false), (worker, true)] {
        grant(db, id, owner, grantee, calculate).await;
    }
    sqlx::query("INSERT INTO subject_profile_revisions(subject_id,owner_user_id,revision,profile,source_hash,changed_by) VALUES($1,$2,1,$3,repeat('a',64),$2)")
        .bind(id).bind(owner).bind(json!({"birth_date":"1990-01-01","birth_time":"12:00:00","timezone":"Asia/Kolkata","birth_location":{"lat":19.0,"lng":72.0}}))
        .execute(db).await.unwrap();
    sqlx::query("INSERT INTO witness_publish_grants(subject_id,owner_user_id,grantee_user_id,grantor_user_id) VALUES($1,$2,$3,$2)")
        .bind(id).bind(owner).bind(worker).execute(db).await.unwrap();
    sqlx::query(
        "INSERT INTO witness_request_admissions(subject_id,owner_user_id,requester_user_id,worker_user_id,identity_issuer,identity_subject,verified_email,profile_revision,profile_source_hash,admission_sha256,contract_sha256)
         VALUES($1,$2,$3,$4,'https://fixture.example.test',$5,$6,1,repeat('a',64),repeat('b',64),repeat('c',64))",
    ).bind(id).bind(owner).bind(recipient).bind(worker)
        .bind(format!("verified-{recipient}")).bind(format!("{recipient}@example.test")).execute(db).await.unwrap();
    (id, profile)
}
async fn http(
    router: &Router,
    method: &str,
    path: &str,
    credential: (&str, &str),
    body: Value,
    lease: Option<&Value>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(credential.0, credential.1)
        .header("Content-Type", "application/json");
    if let Some(job) = lease {
        request = request
            .header("X-Witness-Request", job["id"].as_str().unwrap())
            .header("X-Witness-Lease", job["lease_token"].as_str().unwrap());
    }
    let response = router
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 100_000).await.unwrap();
    let body =
        serde_json::from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes)));
    (status, body)
}
fn artifact(subject: Uuid, profile: &str, date: NaiveDate, digit: &str) -> Value {
    let hash = digit.repeat(64);
    json!({"expected_head":null,"approval":"automatic","artifact":{
        "profile":profile,"kind":"daily","date":date,"sha256":hash,
        "object_key":format!("witness-v2/{subject}/{profile}/daily/{date}/{hash}.json"),
        "report_schema":"noesis.witness-report.v2","packet_sha256":"d".repeat(64),
        "contract_sha256":"c".repeat(64),"effective_at":format!("{date}T00:30:00Z")
    }})
}

#[tokio::test]
async fn verified_requests_reserve_once_and_publish_under_scoped_runtime_role() {
    let Ok(url) = std::env::var("WITNESS_REQUEST_TEST_DATABASE_URL") else {
        eprintln!(
            "Set WITNESS_REQUEST_TEST_DATABASE_URL for the isolated PostgreSQL integration test"
        );
        return;
    };
    let target: PgConnectOptions = url.parse().unwrap();
    assert_eq!(target.get_host(), "127.0.0.1");
    assert_eq!(target.get_database(), Some("noesis_witness_request_test"));
    let db = PgPoolOptions::new()
        .connect_with(target.clone())
        .await
        .unwrap();
    let runtime_url = format!(
        "postgres://noesis_api_runtime@127.0.0.1:{}/noesis_witness_request_test",
        target.get_port()
    );
    let mut config = ApiConfig::from_env().unwrap();
    config.database_url = Some(runtime_url);
    config.redis_url = None;
    config.jwt_secret = "synthetic-witness-request-test-only".into();
    std::env::set_var("NOESIS_REQUIRE_BOUNDED_DB_ROLE", "true");
    std::env::set_var("NOESIS_WITNESS_ON_DEMAND_ENABLED", "true");
    let state = build_app_state_lazy_db(&config).await;
    let owner = account(&db).await;
    let requester = account(&db).await;
    let viewer = account(&db).await;
    let worker = account(&db).await;
    let second_requester = account(&db).await;
    let reader = format!("Bearer {}", browser(&db, &state, requester).await);
    let outsider = format!("Bearer {}", browser(&db, &state, viewer).await);
    let second_reader = format!("Bearer {}", browser(&db, &state, second_requester).await);
    let second_person = ("Authorization", second_reader.as_str());
    let person = ("Authorization", reader.as_str());
    let other = ("Authorization", outsider.as_str());
    let key = format!("test-witness-worker-{}", Uuid::new_v4());
    state
        .auth
        .add_api_key(ApiKey {
            key: key.clone(),
            user_id: worker.to_string(),
            tier: "enterprise".into(),
            permissions: vec!["witness:requests:worker".into(), "biorhythm:read".into()],
            created_at: Utc::now(),
            expires_at: None,
            last_used: None,
            rate_limit: 10000,
            consciousness_level: 5,
        })
        .await
        .unwrap();
    let worker_auth = ("X-API-Key", key.as_str());
    let router = create_router(state, &config);
    let (id, profile) = subject(&db, owner, requester, worker).await;
    let (other_id, other_profile) = subject(&db, owner, second_requester, worker).await;
    grant(&db, id, owner, viewer, false).await;
    let requests = format!("/api/v1/subjects/{id}/witness/requests");
    let catalog = format!("/api/v1/subjects/{id}/witness");
    let claims = "/api/v1/witness/requests/claim";
    let (status, initial) = http(&router, "GET", &requests, person, json!(null), None).await;
    assert_eq!(status, StatusCode::OK, "{initial}");
    assert_eq!(
        initial["status"], "not_requested",
        "run this integration test after 06:00 IST"
    );
    let date: NaiveDate = initial["date"].as_str().unwrap().parse().unwrap();
    assert_eq!(
        http(&router, "GET", &requests, other, json!(null), None)
            .await
            .1["status"],
        "unavailable"
    );
    let unrelated = format!("/api/v1/subjects/{}/witness/requests", Uuid::new_v4());
    assert_eq!(
        http(&router, "GET", &unrelated, person, json!(null), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        http(
            &router,
            "POST",
            &requests,
            person,
            json!({"consent":false}),
            None
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        http(
            &router,
            "POST",
            &requests,
            person,
            json!({"consent":true,"date":date-Duration::days(1)}),
            None
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        http(&router, "POST", claims, person, json!({}), None)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        http(
            &router,
            "GET",
            "/api/v1/users/me/context",
            worker_auth,
            json!(null),
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let accepted = json!({"consent":true});
    let (a, b) = tokio::join!(
        http(&router, "POST", &requests, person, accepted.clone(), None),
        http(&router, "POST", &requests, person, accepted.clone(), None)
    );
    assert_eq!(a.0, StatusCode::OK, "{}", a.1);
    assert_eq!(a.1["status"], "queued");
    assert_eq!(a, b);
    let (status, result) = http(&router, "POST", claims, worker_auth, json!({}), None).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let job = &result["job"];
    assert_eq!(job["id"], a.1["id"]);
    assert_eq!(job["profile_revision"], 1);
    assert_eq!(job["worker_account_id"], worker.to_string());
    assert_eq!(job["admission_sha256"], "b".repeat(64));
    assert_eq!(job["contract_sha256"], "c".repeat(64));
    assert!(http(&router, "POST", claims, worker_auth, json!({}), None)
        .await
        .1["job"]
        .is_null());
    let profile_path = format!("/api/v1/subjects/{id}/profile");
    assert_eq!(
        http(
            &router,
            "GET",
            &profile_path,
            worker_auth,
            json!(null),
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        http(
            &router,
            "GET",
            &profile_path,
            worker_auth,
            json!(null),
            Some(job)
        )
        .await
        .0,
        StatusCode::OK
    );
    let another_profile = format!("/api/v1/subjects/{other_id}/profile");
    assert_eq!(
        http(
            &router,
            "GET",
            &another_profile,
            worker_auth,
            json!(null),
            Some(job)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let capture_path = format!("/api/v1/subjects/{id}/engines/biorhythm/calculate");
    let capture = json!({"expected_revision":1,"options":{"forecast_days":7},"current_time":format!("{date}T00:30:00Z")});
    assert_eq!(
        http(
            &router,
            "POST",
            &capture_path,
            person,
            capture.clone(),
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut wrong = capture.clone();
    wrong["expected_revision"] = json!(2);
    assert_eq!(
        http(
            &router,
            "POST",
            &capture_path,
            worker_auth,
            wrong,
            Some(job)
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (status, result) = http(
        &router,
        "POST",
        &capture_path,
        worker_auth,
        capture,
        Some(job),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["subject_id"], id.to_string());
    let start = format!(
        "/api/v1/witness/requests/{}/start",
        job["id"].as_str().unwrap()
    );
    let lease = json!({"lease_token":job["lease_token"]});
    let mut publication = artifact(id, &profile, date, "e");
    publication["request_id"] = job["id"].clone();
    publication["lease_token"] = job["lease_token"].clone();
    assert_eq!(
        http(
            &router,
            "POST",
            &catalog,
            worker_auth,
            publication.clone(),
            None
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let (first, second) = tokio::join!(
        http(&router, "POST", &start, worker_auth, lease.clone(), None),
        http(&router, "POST", &start, worker_auth, lease.clone(), None)
    );
    assert!(
        matches!(
            (first.0, second.0),
            (StatusCode::OK, StatusCode::CONFLICT) | (StatusCode::CONFLICT, StatusCode::OK)
        ),
        "{first:?} {second:?}"
    );
    let (first, second) = tokio::join!(
        http(
            &router,
            "POST",
            &catalog,
            worker_auth,
            publication.clone(),
            None
        ),
        http(
            &router,
            "POST",
            &catalog,
            worker_auth,
            publication.clone(),
            None
        )
    );
    assert_eq!(first.0, StatusCode::OK, "{}", first.1);
    assert_eq!(second.0, StatusCode::OK, "{}", second.1);
    assert_eq!(first.1["entry"]["head_revision"], 1);
    assert_eq!(second.1["entry"]["head_revision"], 1);
    assert_ne!(first.1["idempotent"], second.1["idempotent"]);
    let (status, result) = http(&router, "GET", &requests, person, json!(null), None).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["status"], "accepted");
    assert_eq!(result["sha256"], "e".repeat(64));
    assert_eq!(result["latest_date"], initial["date"]);

    let mut inconsistent = artifact(id, &profile, date, "f");
    inconsistent["request_id"] = job["id"].clone();
    inconsistent["lease_token"] = job["lease_token"].clone();
    assert_eq!(
        http(&router, "POST", &catalog, worker_auth, inconsistent, None)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let reconcile = format!(
        "/api/v1/witness/requests/{}/hold",
        job["id"].as_str().unwrap()
    );
    assert_eq!(
        http(
            &router,
            "POST",
            &reconcile,
            worker_auth,
            json!({
                "lease_token":job["lease_token"],"reason":"publication_unconfirmed"
            }),
            None
        )
        .await
        .1,
        json!({"status":"accepted","sha256":"e".repeat(64)})
    );

    let more = format!("/api/v1/subjects/{other_id}/witness/requests");
    assert_eq!(
        http(&router, "POST", &more, second_person, accepted, None)
            .await
            .1["status"],
        "queued"
    );
    let (_, result) = http(&router, "POST", claims, worker_auth, json!({}), None).await;
    let job = &result["job"];
    assert_eq!(job["subject_id"], other_id.to_string());
    let start = format!(
        "/api/v1/witness/requests/{}/start",
        job["id"].as_str().unwrap()
    );
    let lease = json!({"lease_token":job["lease_token"]});
    assert_eq!(
        http(&router, "POST", &start, worker_auth, lease.clone(), None)
            .await
            .0,
        StatusCode::OK
    );
    sqlx::query(
        "INSERT INTO subject_profile_revisions(subject_id,owner_user_id,revision,profile,source_hash,changed_by)
         VALUES($1,$2,2,'{}',repeat('f',64),$2)")
        .bind(other_id).bind(owner).execute(&db).await.unwrap();
    let mut stale_publication = artifact(other_id, &other_profile, date, "f");
    stale_publication["request_id"] = job["id"].clone();
    stale_publication["lease_token"] = job["lease_token"].clone();
    let other_catalog = format!("/api/v1/subjects/{other_id}/witness");
    assert_eq!(
        http(
            &router,
            "POST",
            &other_catalog,
            worker_auth,
            stale_publication,
            None
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        http(&router, "GET", &more, second_person, json!(null), None)
            .await
            .1["reason"],
        "source_changed"
    );
    sqlx::query("UPDATE witness_request_admissions SET revoked_at=now() WHERE subject_id=$1")
        .bind(other_id)
        .execute(&db)
        .await
        .unwrap();
    let start = format!(
        "/api/v1/witness/requests/{}/start",
        job["id"].as_str().unwrap()
    );
    let lease = json!({"lease_token":job["lease_token"]});
    assert_eq!(
        http(&router, "POST", &start, worker_auth, lease, None)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let hold = format!(
        "/api/v1/witness/requests/{}/hold",
        job["id"].as_str().unwrap()
    );
    assert_eq!(
        http(
            &router,
            "POST",
            &hold,
            worker_auth,
            json!({"lease_token":job["lease_token"],"reason":"access_changed"}),
            None
        )
        .await
        .1["status"],
        "held"
    );
}
