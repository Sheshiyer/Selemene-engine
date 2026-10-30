//! Daily field — the day's engines and Witness Dyad, computed by the engine and kept
//! in `readings`.
//!
//! For one user and one IST calendar date the job runs the five day-scoped engines
//! (panchanga in daily mode, transits, vimshottari, biorhythm, vedic-clock) ten minutes
//! after that date's sunrise where the day is witnessed (the traditional Panchanga moment;
//! a fixed clock time can fall before sunrise and record the previous Vedic day), from the
//! user's own profile, then the Witness Dyad for that moment with no biofield
//! capture. Each result is stored as a reading with `workflow_id = "daily-field"` and the
//! date in `input_data.daily_date`, so a rerun for the same date computes only what is
//! missing. Runs for one user's day take turns in each process, and a unique index
//! (migration 044) keeps one row per part across processes.
//!
//! - `GET  /api/v1/daily?date=YYYY-MM-DD` returns the stored bundle (no computation).
//! - `POST /api/v1/daily/refresh` computes what is missing for a date (default today, IST).
//! - A background task runs the refresh at 05:30 IST for users whose profile
//!   preferences carry `daily_field: true` (set through `PATCH /api/v1/users/me`).

use crate::handlers::witness::{interpret_grounded, WitnessInterpretRequest};
use crate::AppState;
use axum::{
    extract::{Extension, Json, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use noesis_auth::AuthUser;
use noesis_core::{BirthData, EngineInput, Precision};
use noesis_data::models::reading::{NewReading, Reading};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, Weak};
use utoipa::ToSchema;
use uuid::Uuid;

/// `workflow_id` carried by every daily-field reading.
pub const DAILY_WORKFLOW: &str = "daily-field";
/// The engines whose results change with the day.
pub const DAY_ENGINES: [&str; 5] = ["panchanga", "transits", "vimshottari", "biorhythm", "vedic-clock"];
/// The day engines the Witness Dyad reads. A dyad is stored only when it was given all
/// of them, so one computed without its day context stays missing and is recomputed by
/// the next refresh that can reach them.
pub const WITNESS_GROUNDING: [&str; 4] = ["panchanga", "transits", "vimshottari", "biorhythm"];
/// `engine_id` under which the day's Witness Dyad is stored.
pub const WITNESS_ID: &str = "witness-dyad";
/// Bump when the inputs a daily reading is computed from change meaning.
const METHOD: &str = "daily-field/v1";
/// India Standard Time, the field's calendar.
const IST_OFFSET_SECS: i32 = 5 * 3600 + 30 * 60;
/// How far back and forward a refresh may reach.
const MAX_DAYS_BACK: i64 = 31;
const MAX_DAYS_AHEAD: i64 = 1;

type ApiError = (StatusCode, Json<crate::error_mapper::ErrorResponse>);

fn err(status: StatusCode, code: &str, message: impl Into<String>) -> ApiError {
    crate::error_mapper::ErrorMapper::response(status, code.to_string(), message.into(), None)
}

#[derive(Deserialize, ToSchema, Default)]
pub struct DailyQuery {
    /// IST calendar date, YYYY-MM-DD. Defaults to today in IST.
    pub date: Option<String>,
}

#[derive(Deserialize, ToSchema, Default)]
pub struct DailyRefreshRequest {
    /// IST calendar date, YYYY-MM-DD. Defaults to today in IST.
    pub date: Option<String>,
}

/// One day's field as the engine holds it.
#[derive(Serialize, ToSchema, Debug)]
pub struct DailyBundle {
    /// IST calendar date.
    pub date: String,
    /// The instant the day's engines were computed for, RFC 3339.
    pub moment: String,
    /// How the moment was chosen: `sunrise+10m`, or `fixed-06:00-ist` when sunrise was unreadable.
    pub moment_anchor: String,
    /// Method tag of the stored rows.
    pub method: String,
    /// True when every day engine and the witness dyad are present.
    pub complete: bool,
    /// engine_id -> { reading_id, created_at, result } for each stored day engine.
    #[schema(value_type = Object)]
    pub engines: Map<String, Value>,
    /// The day's Witness Dyad, when stored.
    #[schema(value_type = Object, nullable = true)]
    pub witness: Option<Value>,
    /// What is not stored yet, with the reason when it was skipped this run.
    pub missing: Vec<MissingPart>,
}

#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct MissingPart {
    pub part: String,
    pub reason: String,
}

/// 06:00 IST on `date`, as UTC: the fallback moment when sunrise cannot be read.
pub fn day_moment(date: NaiveDate) -> DateTime<Utc> {
    let ist = chrono::FixedOffset::east_opt(IST_OFFSET_SECS).expect("valid offset");
    ist.from_local_datetime(&date.and_time(NaiveTime::from_hms_opt(6, 0, 0).expect("valid time")))
        .single()
        .expect("fixed offsets are unambiguous")
        .with_timezone(&Utc)
}

/// Minutes after sunrise at which the day is read.
const AFTER_SUNRISE_MINUTES: i64 = 10;

/// Noon IST on `date`, inside that date's Vedic day in every season.
fn noon_ist(date: NaiveDate) -> DateTime<Utc> {
    day_moment(date) + Duration::hours(6)
}

/// Sunrise for `date` where the day is witnessed, read from the vedic-clock engine's own
/// solar calculation (NOAA, -0.833 deg), then `AFTER_SUNRISE_MINUTES` later. Falls back to
/// 06:00 IST when the engine is not available or its answer cannot be read, and without
/// running it at all when the caller's key may not use vedic-clock.
async fn sunrise_moment(
    state: &AppState,
    user: &AuthUser,
    is_api_key: bool,
    birth: &BirthData,
    date: NaiveDate,
    lat: f64,
    lng: f64,
) -> (DateTime<Utc>, &'static str) {
    if !crate::engine_scope_allows(user, is_api_key, "vedic-clock", "daily-field") {
        return (day_moment(date), "fixed-06:00-ist");
    }
    let probe = engine_input("vedic-clock", birth, noon_ist(date), lat, lng, user);
    match state.orchestrator.execute_engine("vedic-clock", probe, user.consciousness_level).await {
        Ok(out) => match sunrise_from_clock(&out.result, date) {
            Some(rise) => (rise + Duration::minutes(AFTER_SUNRISE_MINUTES), "sunrise+10m"),
            None => (day_moment(date), "fixed-06:00-ist"),
        },
        Err(_) => (day_moment(date), "fixed-06:00-ist"),
    }
}

/// The sunrise instant in a vedic-clock result, when its Vedic day is `date`.
fn sunrise_from_clock(result: &Value, date: NaiveDate) -> Option<DateTime<Utc>> {
    let day = result.get("vedic_day")?;
    if day.get("date")?.as_str()? != date.format("%Y-%m-%d").to_string() {
        return None;
    }
    let rise = day.get("sunrise")?.as_str()?;
    DateTime::parse_from_rfc3339(rise).ok().map(|t| t.with_timezone(&Utc))
}

/// Today's calendar date in IST.
pub fn today_ist(now: DateTime<Utc>) -> NaiveDate {
    (now + Duration::seconds(IST_OFFSET_SECS as i64)).date_naive()
}

fn parse_date(raw: Option<&str>, now: DateTime<Utc>) -> Result<NaiveDate, ApiError> {
    let today = today_ist(now);
    let date = match raw {
        None => today,
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
            err(StatusCode::BAD_REQUEST, "INVALID_DATE", "date must be YYYY-MM-DD")
        })?,
    };
    if date < today - Duration::days(MAX_DAYS_BACK) || date > today + Duration::days(MAX_DAYS_AHEAD) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "DATE_OUT_OF_RANGE",
            format!("date must be within {MAX_DAYS_BACK} days before and {MAX_DAYS_AHEAD} day after today (IST)"),
        ));
    }
    Ok(date)
}

/// Stable key for one day's reading of one part, so reruns find it.
fn input_hash(user_id: Uuid, part: &str, date: NaiveDate) -> String {
    format!("{:x}", Sha256::digest(format!("{METHOD}:{user_id}:{part}:{date}").as_bytes()))
}

/// Birth data and the place the day is witnessed from, read from the user's profile.
async fn profile_inputs(
    state: &AppState,
    user_id: Uuid,
) -> Result<(BirthData, f64, f64), ApiError> {
    let profile = state
        .user_repository
        .get_profile(user_id)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "PROFILE_READ_FAILED", e.to_string()))?
        .ok_or_else(|| err(StatusCode::UNPROCESSABLE_ENTITY, "PROFILE_INCOMPLETE", "no profile with birth data"))?;
    let (Some(date), Some(lat), Some(lng)) =
        (profile.birth_date, profile.birth_location_lat, profile.birth_location_lng)
    else {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "PROFILE_INCOMPLETE",
            "profile needs birth date and birth location",
        ));
    };
    let birth = BirthData {
        name: None,
        date: date.format("%Y-%m-%d").to_string(),
        time: profile.birth_time.map(|t| t.format("%H:%M").to_string()),
        latitude: lat,
        longitude: lng,
        timezone: profile.timezone.clone().unwrap_or_else(|| "Asia/Kolkata".to_string()),
    };
    // Where the day is witnessed from: preferences.daily_field_location {lat, lng}, else birth place.
    let here = profile.preferences.get("daily_field_location");
    let wlat = here.and_then(|v| v.get("lat")).and_then(Value::as_f64).unwrap_or(lat);
    let wlng = here.and_then(|v| v.get("lng")).and_then(Value::as_f64).unwrap_or(lng);
    Ok((birth, wlat, wlng))
}

fn engine_input(engine_id: &str, birth: &BirthData, moment: DateTime<Utc>, lat: f64, lng: f64, user: &AuthUser) -> EngineInput {
    let mut options: HashMap<String, Value> = HashMap::new();
    options.insert("user_id".to_string(), json!(user.user_id));
    options.insert("consciousness_level".to_string(), json!(user.consciousness_level));
    match engine_id {
        // Panchanga for the day, not for birth.
        "panchanga" => {
            options.insert("mode".to_string(), json!("daily"));
        }
        // The clock is read where the day is witnessed, in IST.
        "vedic-clock" => {
            options.insert("latitude".to_string(), json!(lat));
            options.insert("longitude".to_string(), json!(lng));
            options.insert("timezone_offset".to_string(), json!(IST_OFFSET_SECS / 60));
        }
        _ => {}
    }
    EngineInput {
        birth_data: Some(birth.clone()),
        current_time: moment,
        location: None,
        precision: Precision::Standard,
        options,
    }
}

#[allow(clippy::too_many_arguments)]
fn new_reading(
    user_id: Uuid,
    part: &str,
    date: NaiveDate,
    moment: DateTime<Utc>,
    anchor: &str,
    lat: f64,
    lng: f64,
    result: Value,
    witness_prompt: Option<String>,
    level: i16,
    ms: Option<f64>,
) -> NewReading {
    NewReading {
        user_id,
        engine_id: part.to_string(),
        workflow_id: Some(DAILY_WORKFLOW.to_string()),
        input_hash: input_hash(user_id, part, date),
        input_data: json!({
            "daily_date": date.format("%Y-%m-%d").to_string(),
            "moment": moment.to_rfc3339(),
            "moment_anchor": anchor,
            "method": METHOD,
            "witnessed_at": { "lat": lat, "lng": lng },
        }),
        result_data: result,
        witness_prompt,
        consciousness_level: level,
        calculation_time_ms: ms,
        client_event_id: None,
        client_device_id: None,
        device_platform: None,
        device_app_version: None,
    }
}

/// The lock for one user's day. Runs for the same user and date take turns in this
/// process, so a refresh that overlaps the scheduler waits, then finds the parts stored
/// and computes nothing. Across processes, the unique index on daily-field readings
/// (migration 044) keeps one row per part.
fn day_lock(uid: Uuid, date: NaiveDate) -> Arc<tokio::sync::Mutex<()>> {
    type Locks = std::sync::Mutex<HashMap<(Uuid, NaiveDate), Weak<tokio::sync::Mutex<()>>>>;
    static LOCKS: OnceLock<Locks> = OnceLock::new();
    let mut locks = LOCKS.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
    locks.retain(|_, held| held.strong_count() > 0);
    if let Some(lock) = locks.get(&(uid, date)).and_then(Weak::upgrade) {
        return lock;
    }
    let lock = Arc::new(tokio::sync::Mutex::new(()));
    locks.insert((uid, date), Arc::downgrade(&lock));
    lock
}

/// Compute and store whatever of `date` is not stored yet. Returns the parts skipped
/// this run with their reasons (scope, phase, engine error, storage error).
pub(crate) async fn run_daily(
    state: &AppState,
    user: &AuthUser,
    is_api_key: bool,
    date: NaiveDate,
) -> Result<Vec<MissingPart>, ApiError> {
    let uid = Uuid::parse_str(&user.user_id)
        .map_err(|_| err(StatusCode::UNAUTHORIZED, "INVALID_USER", "user id is not a UUID"))?;
    let repo = state
        .readings_repository
        .clone()
        .ok_or_else(|| err(StatusCode::SERVICE_UNAVAILABLE, "READINGS_UNAVAILABLE", "readings store not configured"))?;
    let (birth, lat, lng) = profile_inputs(state, uid).await?;
    let date_s = date.format("%Y-%m-%d").to_string();

    let lock = day_lock(uid, date);
    let _turn = lock.lock().await;

    let stored_rows = repo
        .list_daily_field(uid, &date_s)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "READINGS_READ_FAILED", e.to_string()))?;
    // A partly stored day keeps the moment it was started with, so every part agrees.
    let (moment, anchor) = match stored_rows.iter().find_map(|r| stored_moment(&r.input_data)) {
        Some(m) => m,
        None => sunrise_moment(state, user, is_api_key, &birth, date, lat, lng).await,
    };
    let stored: Vec<String> = stored_rows.into_iter().map(|r| r.engine_id).collect();

    let mut skipped: Vec<MissingPart> = vec![];
    let mut computed = 0usize;
    let level = user.consciousness_level;

    for engine_id in DAY_ENGINES {
        if stored.iter().any(|s| s == engine_id) {
            continue;
        }
        if !crate::engine_scope_allows(user, is_api_key, engine_id, "daily-field") {
            skipped.push(MissingPart { part: engine_id.into(), reason: "engine not in this key's scope".into() });
            continue;
        }
        let started = std::time::Instant::now();
        match state
            .orchestrator
            .execute_engine(engine_id, engine_input(engine_id, &birth, moment, lat, lng, user), level)
            .await
        {
            Ok(output) => {
                let prompt = Some(output.witness_prompt.clone());
                let envelope = crate::ApiEngineOutputResponse::from_engine_output(output, state.is_native_engine(engine_id));
                let ms = Some(started.elapsed().as_secs_f64() * 1000.0);
                let reading = new_reading(uid, engine_id, date, moment, anchor, lat, lng, serde_json::to_value(&envelope).unwrap_or_default(), prompt, level as i16, ms);
                match repo.save_daily_reading(&reading).await {
                    Ok(Some(_)) => computed += 1,
                    // Another process stored this part first; its row stands.
                    Ok(None) => {}
                    Err(e) => skipped.push(MissingPart { part: engine_id.into(), reason: format!("storage: {e}") }),
                }
            }
            Err(e) => skipped.push(MissingPart { part: engine_id.into(), reason: format!("engine: {e}") }),
        }
    }

    if !stored.iter().any(|s| s == WITNESS_ID) {
        let out_of_scope: Vec<&str> = WITNESS_GROUNDING
            .into_iter()
            .filter(|e| !crate::engine_scope_allows(user, is_api_key, e, "daily-field"))
            .collect();
        if !out_of_scope.is_empty() {
            skipped.push(MissingPart {
                part: WITNESS_ID.into(),
                reason: format!("witness needs {} in this key's scope", out_of_scope.join(", ")),
            });
        } else {
            let started = std::time::Instant::now();
            let (dyad, available) = interpret_grounded(
                state,
                user,
                is_api_key,
                WitnessInterpretRequest {
                    birth_data: Some(birth.clone()),
                    live_scores: None,
                    current_time: Some(moment),
                    ..Default::default()
                },
            )
            .await;
            let lacking = missing_grounding(&available);
            if !lacking.is_empty() {
                skipped.push(MissingPart {
                    part: WITNESS_ID.into(),
                    reason: format!("witness ran without {}; not stored, retried on the next refresh", lacking.join(", ")),
                });
            } else {
                let ms = Some(started.elapsed().as_secs_f64() * 1000.0);
                let question = Some(dyad.witness_question.clone());
                let reading = new_reading(uid, WITNESS_ID, date, moment, anchor, lat, lng, serde_json::to_value(&dyad).unwrap_or_default(), question, level as i16, ms);
                match repo.save_daily_reading(&reading).await {
                    Ok(Some(_)) => computed += 1,
                    Ok(None) => {}
                    Err(e) => skipped.push(MissingPart { part: WITNESS_ID.into(), reason: format!("storage: {e}") }),
                }
            }
        }
    }

    // One usage event per run that computed anything: the day counts as one call.
    if computed > 0 {
        crate::billing::emit_usage_event(&user.user_id, "workflow:daily-field", &user.tier);
    }
    Ok(skipped)
}

/// The day engines a witness was not given, out of the ones it reads.
fn missing_grounding(available: &[String]) -> Vec<&'static str> {
    WITNESS_GROUNDING.into_iter().filter(|e| !available.iter().any(|a| a == e)).collect()
}

/// The moment and anchor a stored daily row was computed for.
fn stored_moment(input: &Value) -> Option<(DateTime<Utc>, &'static str)> {
    let m = DateTime::parse_from_rfc3339(input.get("moment")?.as_str()?).ok()?.with_timezone(&Utc);
    let anchor = match input.get("moment_anchor").and_then(Value::as_str) {
        Some("sunrise+10m") => "sunrise+10m",
        _ => "fixed-06:00-ist",
    };
    Some((m, anchor))
}

/// Assemble the stored bundle for a date from `readings`.
pub(crate) async fn load_bundle(
    state: &AppState,
    user_id: Uuid,
    date: NaiveDate,
    skipped: Vec<MissingPart>,
) -> Result<DailyBundle, ApiError> {
    let repo = state
        .readings_repository
        .clone()
        .ok_or_else(|| err(StatusCode::SERVICE_UNAVAILABLE, "READINGS_UNAVAILABLE", "readings store not configured"))?;
    let date_s = date.format("%Y-%m-%d").to_string();
    let rows: Vec<Reading> = repo
        .list_daily_field(user_id, &date_s)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "READINGS_READ_FAILED", e.to_string()))?;
    Ok(bundle_from_rows(date, rows, skipped))
}

fn bundle_from_rows(date: NaiveDate, rows: Vec<Reading>, skipped: Vec<MissingPart>) -> DailyBundle {
    let (moment, anchor) = rows
        .iter()
        .find_map(|r| stored_moment(&r.input_data))
        .unwrap_or((day_moment(date), "fixed-06:00-ist"));
    let mut engines = Map::new();
    let mut witness = None;
    for r in rows {
        let entry = json!({
            "reading_id": r.id,
            "created_at": r.created_at.to_rfc3339(),
            "result": r.result_data,
        });
        if r.engine_id == WITNESS_ID {
            witness = Some(entry);
        } else if DAY_ENGINES.contains(&r.engine_id.as_str()) {
            engines.insert(r.engine_id.clone(), entry);
        }
    }
    let mut missing: Vec<MissingPart> = DAY_ENGINES
        .iter()
        .filter(|e| !engines.contains_key(**e))
        .map(|e| {
            skipped
                .iter()
                .find(|s| s.part == *e)
                .cloned()
                .unwrap_or_else(|| MissingPart { part: (*e).into(), reason: "not computed yet".into() })
        })
        .collect();
    if witness.is_none() {
        missing.push(
            skipped
                .iter()
                .find(|s| s.part == WITNESS_ID)
                .cloned()
                .unwrap_or_else(|| MissingPart { part: WITNESS_ID.into(), reason: "not computed yet".into() }),
        );
    }
    DailyBundle {
        date: date.format("%Y-%m-%d").to_string(),
        moment: moment.to_rfc3339(),
        moment_anchor: anchor.to_string(),
        method: METHOD.to_string(),
        complete: missing.is_empty(),
        engines,
        witness,
        missing,
    }
}

/// GET /api/v1/daily — the stored daily field for a date. Never computes.
pub async fn get_daily(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(q): Query<DailyQuery>,
) -> Result<Json<DailyBundle>, ApiError> {
    let uid = Uuid::parse_str(&user.user_id)
        .map_err(|_| err(StatusCode::UNAUTHORIZED, "INVALID_USER", "user id is not a UUID"))?;
    let date = parse_date(q.date.as_deref(), Utc::now())?;
    Ok(Json(load_bundle(&state, uid, date, vec![]).await?))
}

/// POST /api/v1/daily/refresh — compute what is missing for a date, then return the bundle.
/// Idempotent: a date already complete is returned without computing anything.
pub async fn refresh_daily(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    api_key_principal: Option<Extension<crate::ApiKeyPrincipal>>,
    body: Option<Json<DailyRefreshRequest>>,
) -> Result<Json<DailyBundle>, ApiError> {
    let uid = Uuid::parse_str(&user.user_id)
        .map_err(|_| err(StatusCode::UNAUTHORIZED, "INVALID_USER", "user id is not a UUID"))?;
    let req = body.map(|Json(b)| b).unwrap_or_default();
    let date = parse_date(req.date.as_deref(), Utc::now())?;
    let skipped = run_daily(&state, &user, api_key_principal.is_some(), date).await?;
    Ok(Json(load_bundle(&state, uid, date, skipped).await?))
}

/// The scheduled run: every day at 00:00 UTC (05:30 IST) for opted-in users.
/// Disabled with `DAILY_FIELD_SCHEDULER=off`.
pub(crate) fn spawn_scheduler(state: AppState) {
    if std::env::var("DAILY_FIELD_SCHEDULER").map(|v| v.eq_ignore_ascii_case("off")).unwrap_or(false) {
        tracing::info!("Daily field scheduler disabled (DAILY_FIELD_SCHEDULER=off)");
        return;
    }
    tokio::spawn(async move {
        loop {
            let now = Utc::now();
            let next = (now.date_naive() + Duration::days(1)).and_hms_opt(0, 0, 0).expect("midnight").and_utc();
            let wait = (next - now).to_std().unwrap_or(std::time::Duration::from_secs(60));
            tokio::time::sleep(wait).await;
            run_scheduled(&state).await;
        }
    });
    tracing::info!("Daily field scheduler armed (00:00 UTC, 05:30 IST)");
}

async fn run_scheduled(state: &AppState) {
    let users = match state.user_repository.list_daily_field_users().await {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!(error = %e, "daily field: could not list opted-in users");
            return;
        }
    };
    let date = today_ist(Utc::now());
    for (uid, tier, level) in users {
        // The job acts for the user on their own profile, like a signed-in session:
        // phase gating applies through the user's level; key scopes do not apply.
        let user = AuthUser {
            user_id: uid.to_string(),
            tier,
            permissions: vec![],
            rate_limit: 0,
            consciousness_level: level.clamp(0, 5) as u8,
            jti: None,
            token_exp: None,
        };
        match run_daily(state, &user, false, date).await {
            Ok(skipped) if skipped.is_empty() => tracing::info!(user = %uid, %date, "daily field stored"),
            Ok(skipped) => tracing::warn!(user = %uid, %date, ?skipped, "daily field stored with gaps"),
            Err((code, body)) => tracing::warn!(user = %uid, %date, status = %code, error = %body.0.message, "daily field failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moment_is_six_in_the_morning_ist() {
        let d = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        assert_eq!(day_moment(d).to_rfc3339(), "2026-10-30T00:30:00+00:00");
    }

    #[test]
    fn sunrise_is_read_only_for_the_matching_vedic_day() {
        let d = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        let ok = json!({"vedic_day": {"date": "2026-10-30", "sunrise": "2026-10-30T06:12:37+05:30"}});
        assert_eq!(sunrise_from_clock(&ok, d).unwrap().to_rfc3339(), "2026-10-30T00:42:37+00:00");
        let previous_day = json!({"vedic_day": {"date": "2026-10-29", "sunrise": "2026-10-29T06:12:22+05:30"}});
        assert!(sunrise_from_clock(&previous_day, d).is_none());
        assert!(sunrise_from_clock(&json!({}), d).is_none());
    }

    #[test]
    fn noon_probe_sits_inside_the_vedic_day() {
        let d = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        assert_eq!(noon_ist(d).to_rfc3339(), "2026-10-30T06:30:00+00:00");
    }

    #[test]
    fn a_stored_moment_is_reused() {
        let input = json!({"moment": "2026-10-30T00:52:37+00:00", "moment_anchor": "sunrise+10m"});
        let (m, a) = stored_moment(&input).unwrap();
        assert_eq!(m.to_rfc3339(), "2026-10-30T00:52:37+00:00");
        assert_eq!(a, "sunrise+10m");
    }

    #[test]
    fn today_follows_the_ist_calendar() {
        // 20:00 UTC on 29 Oct is 01:30 IST on 30 Oct.
        let t = Utc.with_ymd_and_hms(2026, 10, 29, 20, 0, 0).unwrap();
        assert_eq!(today_ist(t), NaiveDate::from_ymd_opt(2026, 10, 30).unwrap());
    }

    #[test]
    fn dates_outside_the_window_are_refused() {
        let now = Utc.with_ymd_and_hms(2026, 10, 30, 0, 0, 0).unwrap();
        assert!(parse_date(Some("2026-10-30"), now).is_ok());
        assert!(parse_date(Some("2026-10-31"), now).is_ok());
        assert!(parse_date(Some("2026-11-02"), now).is_err());
        assert!(parse_date(Some("2026-09-01"), now).is_err());
        assert!(parse_date(Some("30-10-2026"), now).is_err());
        assert_eq!(parse_date(None, now).ok(), NaiveDate::from_ymd_opt(2026, 10, 30));
    }

    #[test]
    fn input_hash_is_stable_per_user_part_and_date() {
        let u = Uuid::nil();
        let d = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        assert_eq!(input_hash(u, "transits", d), input_hash(u, "transits", d));
        assert_ne!(input_hash(u, "transits", d), input_hash(u, "panchanga", d));
        assert_ne!(input_hash(u, "transits", d), input_hash(u, "transits", d.succ_opt().unwrap()));
    }

    fn row(engine: &str) -> Reading {
        Reading {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            engine_id: engine.into(),
            workflow_id: Some(DAILY_WORKFLOW.into()),
            input_hash: String::new(),
            input_data: json!({}),
            result_data: json!({"ok": true}),
            witness_prompt: None,
            consciousness_level: 5,
            calculation_time_ms: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn bundle_reports_what_is_missing_and_why() {
        let d = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        let b = bundle_from_rows(
            d,
            vec![row("panchanga"), row("transits")],
            vec![MissingPart { part: "vedic-clock".into(), reason: "engine not in this key's scope".into() }],
        );
        assert!(!b.complete);
        assert_eq!(b.engines.len(), 2);
        let parts: Vec<&str> = b.missing.iter().map(|m| m.part.as_str()).collect();
        assert_eq!(parts, vec!["vimshottari", "biorhythm", "vedic-clock", WITNESS_ID]);
        assert!(b.missing.iter().any(|m| m.part == "vedic-clock" && m.reason.contains("scope")));
    }

    #[test]
    fn a_witness_is_grounded_only_with_every_day_engine_it_reads() {
        let all: Vec<String> = ["biorhythm", "human-design", "panchanga", "transits", "vimshottari"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(missing_grounding(&all).is_empty());
        let partial: Vec<String> = ["panchanga", "biorhythm"].iter().map(|s| s.to_string()).collect();
        assert_eq!(missing_grounding(&partial), vec!["transits", "vimshottari"]);
        assert_eq!(missing_grounding(&[]), WITNESS_GROUNDING.to_vec());
        assert!(WITNESS_GROUNDING.iter().all(|e| DAY_ENGINES.contains(e)));
    }

    #[tokio::test]
    async fn runs_for_the_same_day_share_one_lock() {
        let u = Uuid::new_v4();
        let d = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        let first = day_lock(u, d);
        assert!(Arc::ptr_eq(&first, &day_lock(u, d)));
        assert!(!Arc::ptr_eq(&first, &day_lock(u, d.succ_opt().unwrap())));
        assert!(!Arc::ptr_eq(&first, &day_lock(Uuid::new_v4(), d)));
        let held = first.lock().await;
        assert!(day_lock(u, d).try_lock().is_err(), "a second run must wait for the first");
        drop(held);
        assert!(day_lock(u, d).try_lock().is_ok());
    }

    #[test]
    fn bundle_is_complete_with_every_part() {
        let d = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        let mut rows: Vec<Reading> = DAY_ENGINES.iter().map(|e| row(e)).collect();
        rows.push(row(WITNESS_ID));
        let b = bundle_from_rows(d, rows, vec![]);
        assert!(b.complete && b.missing.is_empty() && b.witness.is_some());
    }
}
