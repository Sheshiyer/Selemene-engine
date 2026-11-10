//! Durable today-only request queue. Report bodies stay in the private catalog store.
use super::shared_identity::{
    active, db, fail, private, storage, subject_access, transaction, verified_identity,
    HandlerError,
};
use crate::{
    cf_access::CfIdentity,
    desktop_auth::{hash_secret, new_secret},
    middleware::ApiKeyPrincipal,
    AppState,
};
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use chrono::{DateTime, NaiveDate, Utc};
use noesis_auth::AuthUser;
use noesis_data::models::witness_catalog::WitnessPublish;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(crate) const WORKER_PERMISSION: &str = "witness:requests:worker";
const ENGINES: [&str; 5] = [
    "panchanga",
    "vimshottari",
    "transits",
    "biorhythm",
    "vedic-clock",
];
const HOLD_REASONS: [&str; 8] = [
    "access_changed",
    "source_unavailable",
    "source_changed",
    "provider_unavailable",
    "invalid_report",
    "publication_unconfirmed",
    "request_expired",
    "generation_failed",
];

fn enabled() -> bool {
    std::env::var("NOESIS_WITNESS_ON_DEMAND_ENABLED").as_deref() == Ok("true")
}
pub(crate) fn worker_permission(u: &AuthUser) -> bool {
    u.permissions.iter().any(|value| value == WORKER_PERMISSION)
}
fn today(now: DateTime<Utc>) -> NaiveDate {
    now.with_timezone(&chrono_tz::Asia::Kolkata).date_naive()
}
fn anchor(date: NaiveDate) -> DateTime<Utc> {
    date.and_hms_opt(0, 30, 0)
        .expect("constant clock time")
        .and_utc()
}
fn denied() -> HandlerError {
    fail(StatusCode::FORBIDDEN, "WITNESS_WORKER_DENIED")
}
fn stale() -> HandlerError {
    fail(StatusCode::CONFLICT, "WITNESS_REQUEST_STALE")
}
fn missing() -> HandlerError {
    fail(StatusCode::NOT_FOUND, "WITNESS_REQUEST_NOT_FOUND")
}
fn private_result(result: Result<Response, HandlerError>) -> Response {
    private(result.unwrap_or_else(IntoResponse::into_response))
}
async fn worker_tx<'a>(
    s: &'a AppState,
    u: &AuthUser,
) -> Result<(Uuid, Transaction<'a, Postgres>), HandlerError> {
    if !enabled() || !worker_permission(u) {
        return Err(denied());
    }
    let pool = db(s)?;
    let worker = active(pool, u).await?;
    let mut tx = transaction(pool, worker).await?;
    mark_worker(&mut tx).await?;
    Ok((worker, tx))
}
pub(crate) async fn mark_worker(tx: &mut Transaction<'_, Postgres>) -> Result<(), HandlerError> {
    sqlx::query("SELECT set_config('app.witness_worker','yes',true)")
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
struct Admission {
    id: Uuid,
    subject_id: Uuid,
    worker_user_id: Uuid,
    profile: String,
    profile_revision: i64,
    profile_source_hash: String,
    admission_sha256: String,
    contract_sha256: String,
}
async fn admission(
    tx: &mut Transaction<'_, Postgres>,
    subject: Uuid,
    requester: Uuid,
    id: Option<Uuid>,
    identity: Option<&CfIdentity>,
) -> Result<Option<Admission>, HandlerError> {
    sqlx::query_as(
        "SELECT a.id,a.subject_id,a.worker_user_id,s.subject_key AS profile,a.profile_revision,a.profile_source_hash,a.admission_sha256,a.contract_sha256
         FROM witness_request_admissions a
         JOIN corpus_subjects s ON s.id=a.subject_id AND s.owner_user_id=a.owner_user_id
         JOIN account_identities i ON i.issuer=a.identity_issuer AND i.provider_subject=a.identity_subject
           AND i.user_id=a.requester_user_id AND i.verified_email=a.verified_email
         JOIN user_account_state st ON st.user_id=a.requester_user_id
         JOIN subject_grants g ON g.subject_id=a.subject_id AND g.owner_user_id=a.owner_user_id AND g.grantee_user_id=a.requester_user_id
         WHERE a.subject_id=$1 AND a.requester_user_id=$2 AND ($3::uuid IS NULL OR a.id=$3)
           AND ($4::text IS NULL OR (a.identity_issuer=$4 AND a.identity_subject=$5 AND a.verified_email=$6))
           AND a.revoked_at IS NULL AND s.archived_at IS NULL AND s.subject_key<>'shesh'
           AND i.revoked_at IS NULL AND st.state='active' AND g.revoked_at IS NULL
           AND NOT EXISTS (SELECT 1 FROM subject_account_links l WHERE l.revoked_at IS NULL AND
             ((l.subject_id=a.subject_id AND l.user_id<>a.requester_user_id) OR
              (l.user_id=a.requester_user_id AND l.subject_id<>a.subject_id)))
         FOR SHARE OF a,s,i,st,g",
    )
    .bind(subject).bind(requester).bind(id)
    .bind(identity.map(|i| i.issuer.as_str())).bind(identity.map(|i| i.sub.as_str()))
    .bind(identity.map(|i| i.email.as_str())).fetch_optional(&mut **tx).await.map_err(storage)
}
async fn source_current(
    tx: &mut Transaction<'_, Postgres>,
    a: &Admission,
) -> Result<bool, HandlerError> {
    let profile = sqlx::query_as::<_, (i64, String)>(
        "SELECT revision,source_hash FROM subject_profile_revisions WHERE subject_id=$1 ORDER BY revision DESC LIMIT 1",
    ).bind(a.subject_id).fetch_optional(&mut **tx).await.map_err(storage)?;
    Ok(profile.is_some_and(|(revision, hash)| {
        revision == a.profile_revision && hash == a.profile_source_hash
    }))
}
async fn worker_grants(
    tx: &mut Transaction<'_, Postgres>,
    a: &Admission,
) -> Result<bool, HandlerError> {
    // The scoped engine route and catalog repeat their own permission checks.
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM subject_grants g JOIN user_account_state st ON st.user_id=g.grantee_user_id
          WHERE g.subject_id=$1 AND g.grantee_user_id=$2 AND g.revoked_at IS NULL AND g.can_calculate AND st.state='active'
          FOR SHARE OF g,st) AND EXISTS (SELECT 1 FROM witness_publish_grants
          WHERE subject_id=$1 AND grantee_user_id=$2 AND revoked_at IS NULL FOR SHARE)",
    ).bind(a.subject_id).bind(a.worker_user_id).fetch_one(&mut **tx).await.map_err(storage)
}

#[derive(sqlx::FromRow)]
struct Job {
    id: Uuid,
    admission_id: Uuid,
    subject_id: Uuid,
    requester_user_id: Uuid,
    report_date: NaiveDate,
    status: String,
    reason: Option<String>,
    lease_expires_at: Option<DateTime<Utc>>,
    inference_started_at: Option<DateTime<Utc>>,
}
const JOB_COLS: &str = "id,admission_id,subject_id,requester_user_id,report_date,status,reason,lease_expires_at,inference_started_at";
async fn job_for_lease(
    tx: &mut Transaction<'_, Postgres>,
    worker: Uuid,
    id: Uuid,
    token: &str,
) -> Result<Job, HandlerError> {
    if !valid_token(token) {
        return Err(missing());
    }
    sqlx::query_as(&format!(
        "SELECT {JOB_COLS} FROM witness_requests WHERE id=$1 AND worker_user_id=$2 AND lease_sha256=$3 FOR UPDATE"
    )).bind(id).bind(worker).bind(hash_secret(token)).fetch_optional(&mut **tx).await
        .map_err(storage)?.ok_or_else(missing)
}
fn valid_token(token: &str) -> bool {
    token.len() == 43
        && token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}
fn lease_live(job: &Job, now: DateTime<Utc>) -> bool {
    job.status == "running"
        && job.report_date == today(now)
        && job.lease_expires_at.is_some_and(|expires| expires > now)
}
async fn head(
    tx: &mut Transaction<'_, Postgres>,
    subject: Uuid,
    date: NaiveDate,
) -> Result<Option<String>, HandlerError> {
    sqlx::query_scalar(
        "SELECT v.sha256 FROM witness_heads h JOIN witness_versions v ON v.id=h.version_id
         WHERE h.subject_id=$1 AND h.kind='daily' AND h.report_date=$2",
    )
    .bind(subject)
    .bind(date)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}
async fn held(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    reason: &str,
) -> Result<(), HandlerError> {
    sqlx::query("UPDATE witness_requests SET status='held',reason=$2,updated_at=now() WHERE id=$1 AND status IN ('queued','running')")
        .bind(id).bind(reason).execute(&mut **tx).await.map_err(storage)?;
    Ok(())
}
pub(crate) async fn record_accepted(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    sha256: &str,
) -> Result<(), HandlerError> {
    sqlx::query("UPDATE witness_requests SET status='accepted',sha256=$2,reason=NULL,updated_at=now() WHERE id=$1 AND status IN ('queued','running','held')")
        .bind(id).bind(sha256).execute(&mut **tx).await.map_err(storage)?;
    Ok(())
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RequestStatus {
    status: &'static str,
    date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
}
impl RequestStatus {
    fn status(mut self, status: &'static str, reason: Option<&str>) -> Self {
        self.status = status;
        self.reason = reason.map(str::to_owned);
        self
    }
}
async fn request_status(
    tx: &mut Transaction<'_, Postgres>,
    account: Uuid,
    subject: Uuid,
    identity: &CfIdentity,
    submit: bool,
    now: DateTime<Utc>,
) -> Result<RequestStatus, HandlerError> {
    subject_access(tx, account, subject, false).await?;
    let date = today(now);
    let latest_date = sqlx::query_scalar(
        "SELECT max(report_date) FROM witness_heads WHERE subject_id=$1 AND kind='daily'",
    )
    .bind(subject)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    let mut response = RequestStatus {
        status: "not_requested",
        date,
        latest_date,
        reason: None,
        id: None,
        sha256: None,
    };
    if let Some(hash) = head(tx, subject, date).await? {
        response.sha256 = Some(hash);
        return Ok(response.status("accepted", None));
    }
    let scheduled: bool =
        sqlx::query_scalar("SELECT subject_key='shesh' FROM corpus_subjects WHERE id=$1")
            .bind(subject)
            .fetch_one(&mut **tx)
            .await
            .map_err(storage)?;
    if scheduled {
        return Ok(response.status("unavailable", Some("scheduled")));
    }
    if !enabled() {
        return Ok(response.status("unavailable", Some("service_unavailable")));
    }
    let Some(admission) = admission(tx, subject, account, None, Some(identity)).await? else {
        let linked: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM subject_account_links WHERE subject_id=$1 AND user_id=$2 AND revoked_at IS NULL)",
        ).bind(subject).bind(account).fetch_one(&mut **tx).await.map_err(storage)?;
        return Ok(response.status(
            "unavailable",
            Some(if linked {
                "not_admitted"
            } else {
                "not_eligible"
            }),
        ));
    };
    if !source_current(tx, &admission).await? {
        return Ok(response.status("unavailable", Some("source_changed")));
    }
    if now < anchor(date) {
        return Ok(response.status("before_anchor", None));
    }
    if submit {
        sqlx::query(
            "INSERT INTO witness_requests(admission_id,subject_id,requester_user_id,worker_user_id,report_date)
             VALUES($1,$2,$3,$4,$5) ON CONFLICT(subject_id,report_date) DO NOTHING",
        ).bind(admission.id).bind(subject).bind(account).bind(admission.worker_user_id).bind(date)
            .execute(&mut **tx).await.map_err(storage)?;
    }
    let job: Option<Job> = sqlx::query_as(&format!(
        "SELECT {JOB_COLS} FROM witness_requests WHERE subject_id=$1 AND requester_user_id=$2 AND report_date=$3"
    )).bind(subject).bind(account).bind(date).fetch_optional(&mut **tx).await.map_err(storage)?;
    let Some(job) = job else {
        return Ok(response);
    };
    response.id = Some(job.id);
    Ok(match job.status.as_str() {
        "queued" => response.status("queued", None),
        "running" if lease_live(&job, now) => response.status("running", None),
        "running" => response.status("held", Some("request_expired")),
        "held" => response.status("held", job.reason.as_deref()),
        // The live catalog is authoritative if an operator rolled back the head.
        "accepted" => response.status("held", Some("publication_unconfirmed")),
        _ => return Err(stale()),
    })
}
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Consent {
    consent: bool,
}
async fn human(
    s: AppState,
    u: AuthUser,
    identity: Option<Extension<CfIdentity>>,
    subject: Uuid,
    submit: bool,
) -> Result<Response, HandlerError> {
    let identity = verified_identity(&s, &u, identity).await?;
    let pool = db(&s)?;
    let account = active(pool, &u).await?;
    let mut tx = transaction(pool, account).await?;
    let status = request_status(&mut tx, account, subject, &identity, submit, Utc::now()).await?;
    tx.commit().await.map_err(storage)?;
    Ok(Json(status).into_response())
}
#[utoipa::path(get, path="/api/v1/subjects/{id}/witness/requests", tag="shared-identity",
    responses((status=200,description="Private current IST date request status",body=RequestStatus),
              (status=403,description="Verified identity required"),(status=404,description="Subject unavailable")))]
pub async fn status(
    State(s): State<AppState>,
    Extension(u): Extension<AuthUser>,
    identity: Option<Extension<CfIdentity>>,
    Path(subject): Path<Uuid>,
) -> Response {
    private_result(human(s, u, identity, subject, false).await)
}
#[utoipa::path(post, path="/api/v1/subjects/{id}/witness/requests", tag="shared-identity",
    request_body=Consent,
    responses((status=200,description="Current IST date request status",body=RequestStatus),
              (status=400,description="Explicit consent required"),(status=403,description="Verified identity required"),
              (status=404,description="Subject unavailable")))]
pub async fn enqueue(
    State(s): State<AppState>,
    Extension(u): Extension<AuthUser>,
    identity: Option<Extension<CfIdentity>>,
    Path(subject): Path<Uuid>,
    Json(p): Json<Consent>,
) -> Response {
    if !p.consent {
        return private_result(Err(fail(
            StatusCode::BAD_REQUEST,
            "WITNESS_CONSENT_REQUIRED",
        )));
    }
    private_result(human(s, u, identity, subject, true).await)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {}
async fn claim_job(s: AppState, u: AuthUser) -> Result<Response, HandlerError> {
    let (worker, mut tx) = worker_tx(&s, &u).await?;
    // Serialize concurrent claims by the same bounded worker account.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("witness-request:{worker}"))
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
    let now = Utc::now();
    let date = today(now);
    sqlx::query(
        "UPDATE witness_requests SET status='held',reason='request_expired',updated_at=now()
         WHERE worker_user_id=$1 AND ((status='queued' AND report_date<$2)
           OR (status='running' AND (report_date<>$2 OR lease_expires_at<=$3)))",
    )
    .bind(worker)
    .bind(date)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    let current: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM witness_requests WHERE worker_user_id=$1 AND status='running')")
        .bind(worker).fetch_one(&mut *tx).await.map_err(storage)?;
    let mut claimed = None;
    if !current && now >= anchor(date) {
        let job: Option<Job> = sqlx::query_as(&format!(
            "SELECT {JOB_COLS} FROM witness_requests WHERE worker_user_id=$1 AND report_date=$2 AND status='queued'
             ORDER BY created_at,id LIMIT 1 FOR UPDATE SKIP LOCKED"
        )).bind(worker).bind(date).fetch_optional(&mut *tx).await.map_err(storage)?;
        if let Some(job) = job {
            if let Some(hash) = head(&mut tx, job.subject_id, date).await? {
                record_accepted(&mut tx, job.id, &hash).await?;
            } else if let Some(a) = admission(
                &mut tx,
                job.subject_id,
                job.requester_user_id,
                Some(job.admission_id),
                None,
            )
            .await?
            {
                if !worker_grants(&mut tx, &a).await? {
                    held(&mut tx, job.id, "access_changed").await?;
                } else if !source_current(&mut tx, &a).await? {
                    held(&mut tx, job.id, "source_changed").await?;
                } else {
                    let token = new_secret();
                    sqlx::query("UPDATE witness_requests SET status='running',lease_sha256=$2,lease_expires_at=$3,updated_at=now() WHERE id=$1")
                        .bind(job.id).bind(hash_secret(&token)).bind(now+chrono::Duration::minutes(30)).execute(&mut *tx).await.map_err(storage)?;
                    claimed = Some(json!({
                        "id":job.id,"subject_id":job.subject_id,"profile":a.profile,"date":date,
                        "profile_revision":a.profile_revision,"admission_sha256":a.admission_sha256,
                        "contract_sha256":a.contract_sha256,
                        "worker_account_id":worker,"lease_token":token
                    }));
                }
            } else {
                held(&mut tx, job.id, "access_changed").await?;
            }
        }
    }
    tx.commit().await.map_err(storage)?;
    Ok(Json(json!({"job":claimed})).into_response())
}
pub async fn claim(
    State(s): State<AppState>,
    Extension(u): Extension<AuthUser>,
    key: Option<Extension<ApiKeyPrincipal>>,
    Json(_): Json<Claim>,
) -> Response {
    if key.is_none() {
        return private_result(Err(denied()));
    }
    private_result(claim_job(s, u).await)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lease {
    lease_token: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hold {
    lease_token: String,
    reason: String,
}
async fn current_admission(
    tx: &mut Transaction<'_, Postgres>,
    job: &Job,
) -> Result<Admission, HandlerError> {
    let a = admission(
        tx,
        job.subject_id,
        job.requester_user_id,
        Some(job.admission_id),
        None,
    )
    .await?
    .ok_or_else(stale)?;
    if !worker_grants(tx, &a).await? || !source_current(tx, &a).await? {
        return Err(stale());
    }
    Ok(a)
}
async fn start_job(
    s: AppState,
    u: AuthUser,
    id: Uuid,
    token: &str,
) -> Result<Response, HandlerError> {
    let (worker, mut tx) = worker_tx(&s, &u).await?;
    let job = job_for_lease(&mut tx, worker, id, token).await?;
    let now = Utc::now();
    if !lease_live(&job, now) || job.inference_started_at.is_some() {
        return Err(stale());
    }
    current_admission(&mut tx, &job).await?;
    if head(&mut tx, job.subject_id, job.report_date)
        .await?
        .is_some()
    {
        return Err(stale());
    }
    // This is a one-way reservation. Unknown outcomes are held, never replayed.
    sqlx::query("UPDATE witness_requests SET inference_started_at=$2,updated_at=now() WHERE id=$1")
        .bind(id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
    tx.commit().await.map_err(storage)?;
    Ok(Json(json!({"status":"running"})).into_response())
}
pub async fn start(
    State(s): State<AppState>,
    Extension(u): Extension<AuthUser>,
    key: Option<Extension<ApiKeyPrincipal>>,
    Path(id): Path<Uuid>,
    Json(p): Json<Lease>,
) -> Response {
    if key.is_none() {
        return private_result(Err(denied()));
    }
    private_result(start_job(s, u, id, &p.lease_token).await)
}
async fn hold_job(s: AppState, u: AuthUser, id: Uuid, p: Hold) -> Result<Response, HandlerError> {
    if !HOLD_REASONS.contains(&p.reason.as_str()) {
        return Err(fail(StatusCode::BAD_REQUEST, "INVALID_WITNESS_HOLD_REASON"));
    }
    let (worker, mut tx) = worker_tx(&s, &u).await?;
    let job = job_for_lease(&mut tx, worker, id, &p.lease_token).await?;
    let status = if let Some(hash) = head(&mut tx, job.subject_id, job.report_date).await? {
        record_accepted(&mut tx, id, &hash).await?;
        json!({"status":"accepted","sha256":hash})
    } else {
        if job.status == "accepted" {
            return Err(stale());
        }
        held(&mut tx, id, &p.reason).await?;
        json!({"status":"held"})
    };
    tx.commit().await.map_err(storage)?;
    Ok(Json(status).into_response())
}
pub async fn hold(
    State(s): State<AppState>,
    Extension(u): Extension<AuthUser>,
    key: Option<Extension<ApiKeyPrincipal>>,
    Path(id): Path<Uuid>,
    Json(p): Json<Hold>,
) -> Response {
    if key.is_none() {
        return private_result(Err(denied()));
    }
    private_result(hold_job(s, u, id, p).await)
}

/// Called in the same transaction as the catalog CAS and acceptance record.
pub(crate) async fn check_publish(
    tx: &mut Transaction<'_, Postgres>,
    worker: Uuid,
    subject: Uuid,
    id: Uuid,
    token: &str,
    p: &WitnessPublish,
) -> Result<(), HandlerError> {
    if !enabled() {
        return Err(denied());
    }
    mark_worker(tx).await?;
    let job = job_for_lease(tx, worker, id, token).await?;
    if job.subject_id != subject
        || p.artifact.date != job.report_date.to_string()
        || p.artifact.kind != "daily"
        || p.artifact.report_schema != "noesis.witness-report.v2"
        || p.approval != "automatic"
        || !p.expected_head.is_null()
    {
        return Err(stale());
    }
    let a = current_admission(tx, &job).await?;
    if p.artifact.contract_sha256 != a.contract_sha256 {
        return Err(stale());
    }
    // A retry of an already committed request may only acknowledge the exact
    // current artifact. The repository's expected-head rules also apply.
    if job.status == "accepted" {
        return if head(tx, subject, job.report_date).await?.as_deref()
            == Some(p.artifact.sha256.as_str())
        {
            Ok(())
        } else {
            Err(stale())
        };
    }
    if !lease_live(&job, Utc::now()) || job.inference_started_at.is_none() {
        return Err(stale());
    }
    Ok(())
}

#[derive(Clone)]
pub struct WorkerAccess {
    id: Uuid,
    token: String,
}
impl WorkerAccess {
    pub(crate) async fn recheck(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        worker: Uuid,
        subject: Uuid,
    ) -> Result<(i64, DateTime<Utc>), HandlerError> {
        mark_worker(tx).await?;
        let job = job_for_lease(tx, worker, self.id, &self.token).await?;
        if job.subject_id != subject || !lease_live(&job, Utc::now()) {
            return Err(stale());
        }
        let a = current_admission(tx, &job).await?;
        Ok((a.profile_revision, anchor(job.report_date)))
    }
}

pub(crate) fn capture_options(
    engine: &str,
    options: &std::collections::HashMap<String, serde_json::Value>,
) -> bool {
    match engine {
        "panchanga" => {
            options.len() == 1 && options.get("mode").and_then(|v| v.as_str()) == Some("daily")
        }
        "biorhythm" => {
            options.len() == 1 && options.get("forecast_days").and_then(|v| v.as_u64()) == Some(7)
        }
        _ => options.is_empty(),
    }
}

/// Restrict this dedicated API key to its queue and leased subject.
pub(crate) fn restrict_worker(
    u: &AuthUser,
    method: &Method,
    path: &str,
    headers: &HeaderMap,
) -> Result<Option<WorkerAccess>, HandlerError> {
    if !enabled() {
        return Err(denied());
    }
    if method == Method::POST {
        if path == "/api/v1/witness/requests/claim" {
            return Ok(None);
        }
        if let Some((id, action)) = path
            .strip_prefix("/api/v1/witness/requests/")
            .and_then(|rest| rest.split_once('/'))
        {
            if Uuid::parse_str(id).is_ok() && matches!(action, "start" | "hold") {
                return Ok(None);
            }
        }
    }
    let (subject, operation) = path
        .strip_prefix("/api/v1/subjects/")
        .and_then(|rest| rest.split_once('/'))
        .ok_or_else(denied)?;
    Uuid::parse_str(subject).map_err(|_| denied())?;
    if method == Method::POST && operation == "witness" {
        return Ok(None);
    }
    if !(method == Method::GET && matches!(operation, "profile" | "witness")) {
        let engine = operation
            .strip_prefix("engines/")
            .and_then(|v| v.strip_suffix("/calculate"))
            .ok_or_else(denied)?;
        if method != Method::POST
            || !ENGINES.contains(&engine)
            || !u
                .permissions
                .iter()
                .any(|p| p == &crate::engine_scope_permission(engine))
        {
            return Err(denied());
        }
    }
    let id = headers
        .get("x-witness-request")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or_else(denied)?;
    let token = headers
        .get("x-witness-lease")
        .and_then(|v| v.to_str().ok())
        .filter(|v| valid_token(v))
        .ok_or_else(denied)?;
    Ok(Some(WorkerAccess {
        id,
        token: token.to_owned(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_date_and_06_00_ist_boundary() {
        let parse = |value: &str| value.parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            today(parse("2026-11-09T18:29:59Z")).to_string(),
            "2026-11-09"
        );
        assert_eq!(
            today(parse("2026-11-09T18:30:00Z")).to_string(),
            "2026-11-10"
        );
        let now = parse("2026-11-10T00:29:59Z");
        assert!(now < anchor(today(now)));
        let now = parse("2026-11-10T00:30:00Z");
        assert_eq!(now, anchor(today(now)));
    }
}
