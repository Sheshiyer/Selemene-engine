//! POST /api/v1/assets/generate — Premium asset generation (additive only)
//!
//! New surface for premium integrated readings and source-pack generation.
//! Fetches minimal engine seeds via the existing orchestrator (partial-results
//! tolerant), loads the mode doc, then drives generation through a Rust mirror
//! of @noesis/witness-pipeline's IntegratedReadingOrchestrator + factory + audit.
//!
//! This file and route are purely additive. No existing route, handler,
//! or response shape (including /witness/interpret) is modified.

use std::sync::Arc;

use crate::reading::audit::{summarize, GateSummary, SectionRubric};
use crate::reading::mode::{l0_mode, PassPlan};
use crate::reading::writer::{
    write_passes, NoProgress, PassWriter, ReadingInputs, ReadingPlan, ReadingProgress, WrittenPass,
};
use crate::AppState;
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use chrono::Utc;
use noesis_auth::AuthUser;
use noesis_core::EngineInput;
use noesis_data::repositories::asset_job_repository::{
    AssetJob, AssetJobRepository, StartJobError,
};
use noesis_witness::llm::LlmClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

/// Internal helper: convert legacy flat BirthData to the new subjects[] shape.
/// Used only inside the handler to support the old path while the rich contract
/// (subjects + normalized_location) is the preferred form.
fn legacy_birth_to_subjects(
    bd: &noesis_core::BirthData,
) -> Vec<noesis_core::intake::ReportSubjectInput> {
    vec![noesis_core::intake::ReportSubjectInput {
        role: "primary".to_string(),
        name: bd.name.clone(),
        gender: None,
        sex_for_external_chart_source: None,
        birth_date: bd.date.clone(),
        birth_time: bd.time.clone(),
        birth_time_confidence: None,
        birth_location_query: None,
        normalized_location: Some(noesis_core::intake::NormalizedLocation {
            display_name: "legacy".to_string(),
            latitude: bd.latitude,
            longitude: bd.longitude,
            timezone: bd.timezone.clone(),
            provider: "legacy".to_string(),
            confidence: "legacy".to_string(),
        }),
        relationship_label: None,
    }]
}

/// Rust equivalent of TS `isCompleteReportRequest`.
/// Requires non-empty subjects AND every subject to have a `normalized_location`.
fn has_complete_locations(subjects: &[noesis_core::intake::ReportSubjectInput]) -> bool {
    !subjects.is_empty() && subjects.iter().all(|s| s.normalized_location.is_some())
}

/// Internal bridge: when rich subjects are provided, synthesize a minimal BirthData
/// from the first subject's normalized_location + birth fields so the engine layer
/// (which still expects legacy BirthData) can execute and populate engines_used.
/// This is additive support only — the rich path is the contract of record.
fn first_subject_to_birth_data(
    sub: &noesis_core::intake::ReportSubjectInput,
) -> Option<noesis_core::BirthData> {
    sub.normalized_location
        .as_ref()
        .map(|loc| noesis_core::BirthData {
            name: sub.name.clone(),
            date: sub.birth_date.clone(),
            time: sub.birth_time.clone(),
            latitude: loc.latitude,
            longitude: loc.longitude,
            timezone: loc.timezone.clone(),
        })
}

#[derive(Deserialize, ToSchema)]
#[schema(example = r#"{
  "mode": "integrated-kundali-l0",
  "report_level": "L0",
  "subjects": [{
    "role": "primary",
    "name": "Aarav Sharma",
    "birth_date": "1992-03-14",
    "birth_time": "06:45",
    "normalized_location": {
      "display_name": "Bengaluru, Karnataka, India",
      "latitude": 12.9716,
      "longitude": 77.5946,
      "timezone": "Asia/Kolkata",
      "provider": "manual",
      "confidence": "exact"
    }
  }]
}"#)]
pub struct AssetGenerateRequest {
    /// Legacy flat birth data. Still accepted for backward compatibility.
    /// Prefer `subjects[]` + `report_level` for L0-L5 and multi-subject flows.
    pub birth_data: Option<noesis_core::BirthData>,
    pub mode: String,
    /// Legacy compatibility field; ignored. Access uses the authenticated user phase.
    #[serde(default)]
    #[allow(dead_code)]
    pub consciousness_level: u8,
    #[allow(dead_code)]
    pub options: Option<Value>,

    // === Rich intake (preferred contract) ===
    /// Report depth: "L0" | "L1" | "L2" | "L3" | "L4" | "L5".
    #[schema(example = "L0")]
    pub report_level: Option<String>,

    /// One or more subjects (primary + optional partners).
    /// Every subject MUST carry a `normalized_location` for the rich path
    /// (enforced by the isComplete gate).
    pub subjects: Option<Vec<noesis_core::intake::ReportSubjectInput>>,

    /// Rich relationship framing for multi-subject reports (e.g. synastry, family, business).
    pub relationship_context: Option<noesis_core::intake::RelationshipContext>,

    /// Optional language code for orchestrator/prompt selection (e.g. "hi", "en").
    /// Additive first-class field. Top-level (not inside relationship_context).
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AssetPass {
    pub id: String,
    pub title: String,
    pub output: String,
}

#[derive(Serialize, ToSchema)]
pub struct AssetGenerateResponse {
    pub mode: String,
    pub register: String,
    pub passes: Vec<AssetPass>,
    pub assembled: String,
    pub engines_used: Vec<String>,
    /// Minimal source-pack manifest (full factory lives in witness-pipeline).
    pub source_pack: Option<Value>,
}

/// Engines every integrated reading gathers.
const READING_ENGINES: [&str; 6] = [
    "panchanga",
    "numerology",
    "human-design",
    "gene-keys",
    "vimshottari",
    "biorhythm",
];

type HandlerError = (StatusCode, Json<crate::error_mapper::ErrorResponse>);

fn error(status: StatusCode, code: &str, message: &str, details: Option<Value>) -> HandlerError {
    crate::ErrorMapper::response(status, code, message, details)
}

/// A validated request with its engine results, ready to be written.
struct PreparedReading {
    mode: String,
    register: String,
    plan: ReadingPlan,
    subject_names: Vec<String>,
    seeds: Vec<(String, Value)>,
    engines_used: Vec<String>,
    person_id: String,
    report_level: Option<String>,
    language: Option<String>,
    subjects: Vec<noesis_core::intake::ReportSubjectInput>,
    relationship_context: Option<noesis_core::intake::RelationshipContext>,
}

impl PreparedReading {
    fn inputs(&self) -> ReadingInputs<'_> {
        ReadingInputs {
            subject_names: &self.subject_names,
            seeds: &self.seeds,
            register: &self.register,
            language: self.language.as_deref(),
        }
    }
}

/// Validate the request, resolve its mode, and run the seed engines.
async fn prepare_reading(
    state: &AppState,
    user: &AuthUser,
    is_api_key: bool,
    req: AssetGenerateRequest,
) -> Result<PreparedReading, HandlerError> {
    // Engine permission scopes (API-key principals): an engine the key is not
    // entitled to is not executed and does not appear in engines_used.
    let allows = |engine_id: &str| {
        crate::engine_scope_allows(user, is_api_key, engine_id, "assets/generate")
    };
    // Engine authorization must come only from the authenticated identity.
    // The legacy request field remains accepted but cannot raise or lower access.
    let consciousness_level = user.consciousness_level;
    let now = Utc::now();

    // Derive effective subjects: prefer req.subjects when provided and non-empty,
    // else fall back to legacy birth_data conversion (Task 5 helper).
    let effective_subjects: Vec<noesis_core::intake::ReportSubjectInput> =
        if let Some(subs) = &req.subjects {
            if !subs.is_empty() {
                subs.clone()
            } else if let Some(bd) = &req.birth_data {
                legacy_birth_to_subjects(bd)
            } else {
                vec![]
            }
        } else if let Some(bd) = &req.birth_data {
            legacy_birth_to_subjects(bd)
        } else {
            vec![]
        };

    // is_complete gate (Rust equivalent of TS isCompleteReportRequest)
    // If client sends the rich subjects form, every subject must have normalized_location.
    if let Some(subs) = &req.subjects {
        if !subs.is_empty() && !has_complete_locations(&effective_subjects) {
            return Err(error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "INCOMPLETE_SUBJECTS",
                "Every subject must include a normalized_location (isCompleteReportRequest contract)",
                Some(serde_json::json!({
                    "subject_count": subs.len(),
                    "subjects_missing_location": subs.iter().filter(|s| s.normalized_location.is_none()).count()
                })),
            ));
        }
    }

    let plan = load_mode_document(&req.mode)
        .map_err(|msg| error(StatusCode::BAD_REQUEST, "UNKNOWN_MODE", &msg, None))?;

    // Prefer explicit report_level when provided.
    let effective_report_level: Option<String> = req.report_level.clone();

    // Bridge rich subjects → birth_data for engine execution.
    // Prefer explicit birth_data (legacy), else synthesize from first rich subject.
    let bridged_birth_data: Option<noesis_core::BirthData> = if req.birth_data.is_some() {
        req.birth_data.clone()
    } else {
        effective_subjects
            .first()
            .and_then(first_subject_to_birth_data)
    };

    let input = EngineInput {
        birth_data: bridged_birth_data,
        current_time: now,
        location: None,
        precision: noesis_core::Precision::Standard,
        options: {
            let mut m = std::collections::HashMap::new();
            m.insert("user_id".to_string(), serde_json::json!(user.user_id));
            m.insert("mode".to_string(), serde_json::json!(req.mode.clone()));
            if let Some(rl) = &effective_report_level {
                m.insert("report_level".to_string(), serde_json::json!(rl));
            }
            if let Some(lang) = &req.language {
                m.insert("language".to_string(), serde_json::json!(lang));
            }
            m
        },
    };
    let level = consciousness_level;
    let orch = state.orchestrator.clone();

    // Gather a focused set of engines to seed the asset pipeline.
    // (Partial results are acceptable — mirrors existing witness behavior.)
    let (panchanga, numerology, human_design, gene_keys, vimshottari, biorhythm) = tokio::join!(
        run_engine(
            &orch,
            allows("panchanga"),
            "panchanga",
            input.clone(),
            level
        ),
        run_engine(
            &orch,
            allows("numerology"),
            "numerology",
            input.clone(),
            level
        ),
        run_engine(
            &orch,
            allows("human-design"),
            "human-design",
            input.clone(),
            level
        ),
        run_engine(
            &orch,
            allows("gene-keys"),
            "gene-keys",
            input.clone(),
            level
        ),
        run_engine(
            &orch,
            allows("vimshottari"),
            "vimshottari",
            input.clone(),
            level
        ),
        run_engine(
            &orch,
            allows("biorhythm"),
            "biorhythm",
            input.clone(),
            level
        ),
    );

    // Collect seeds (id, value) preserving partial-results tolerance.
    let mut engines_used: Vec<String> = vec![];
    let mut seeds: Vec<(String, Value)> = vec![];
    for (id, value) in READING_ENGINES.iter().zip([
        panchanga,
        numerology,
        human_design,
        gene_keys,
        vimshottari,
        biorhythm,
    ]) {
        if let Some(value) = value {
            engines_used.push(id.to_string());
            seeds.push((id.to_string(), value));
        }
    }

    let subject_names = effective_subjects
        .iter()
        .map(|s| {
            s.name
                .clone()
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(|| s.role.clone())
        })
        .collect::<Vec<_>>();

    Ok(PreparedReading {
        mode: req.mode,
        register: if consciousness_level <= 3 {
            "l1_l3"
        } else {
            "l4_l5"
        }
        .to_string(),
        plan,
        subject_names,
        seeds,
        engines_used,
        person_id: user.user_id.clone(),
        report_level: effective_report_level,
        language: req.language,
        subjects: effective_subjects,
        relationship_context: req.relationship_context,
    })
}

/// Assemble written passes into the response, with the audited source pack.
fn finish_reading(prepared: PreparedReading, written: Vec<WrittenPass>) -> AssetGenerateResponse {
    let assembled = written
        .iter()
        .map(|pass| format!("## {}\n\n{}", pass.title, pass.output))
        .collect::<Vec<_>>()
        .join("\n\n");
    let rubrics: Vec<_> = written.iter().map(|pass| pass.rubric.clone()).collect();
    let gate = summarize(
        &rubrics,
        &prepared.engines_used,
        &READING_ENGINES,
        prepared.seeds.len(),
    );
    let source_pack = Some(build_source_pack_with_audit(SourcePackInput {
        person_id: &prepared.person_id,
        register: &prepared.register,
        engines: &prepared.engines_used,
        facts_count: prepared.seeds.len(),
        section_rubrics: &rubrics,
        gate: &gate,
        report_level: prepared.report_level.as_deref(),
        language: prepared.language.as_deref(),
        subjects: &prepared.subjects,
        relationship_context: prepared.relationship_context.as_ref(),
    }));
    AssetGenerateResponse {
        mode: prepared.mode,
        register: prepared.register,
        passes: written
            .into_iter()
            .map(|pass| AssetPass {
                id: pass.id,
                title: pass.title,
                output: pass.output,
            })
            .collect(),
        assembled,
        engines_used: prepared.engines_used,
        source_pack,
    }
}

/// POST /api/v1/assets/generate
///
/// Gathers engine results and returns a witness-pipeline-compatible shape
/// (mode, register, passes, assembled) at once. Sections are rendered from
/// engine values without a language model, so the source pack's gate reports
/// `blocked` with `sections_not_model_written`. For a model-written reading,
/// start a job with `POST /api/v1/assets/jobs`.
#[utoipa::path(
    post,
    path = "/api/v1/assets/generate",
    tag = "assets",
    request_body = AssetGenerateRequest,
    responses(
        (status = 200, description = "Engine-seeded reading (not model-written)", body = AssetGenerateResponse),
    )
)]
pub async fn generate(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    api_key_principal: Option<Extension<crate::ApiKeyPrincipal>>,
    Json(req): Json<AssetGenerateRequest>,
) -> Result<Json<AssetGenerateResponse>, HandlerError> {
    let prepared = prepare_reading(&state, &user, api_key_principal.is_some(), req).await?;
    let written = write_passes(
        &prepared.plan,
        &prepared.inputs(),
        &PassWriter::Seed,
        &NoProgress,
    )
    .await
    .map_err(|msg| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "READING_FAILED",
            &msg,
            None,
        )
    })?;
    Ok(Json(finish_reading(prepared, written)))
}

/// A job that has not reported progress for this long has lost its writer:
/// a pass is two attempts at most, each capped at 20 minutes of streaming.
const JOB_STALE_AFTER_SECS: i64 = 45 * 60;

/// The state of a reading job; `result` is set once it has succeeded.
#[derive(Serialize, ToSchema)]
pub struct AssetJobResponse {
    pub job_id: String,
    /// `running`, `succeeded` or `failed`.
    pub status: String,
    pub mode: String,
    pub completed_passes: i32,
    pub total_passes: i32,
    /// The pass being written, while running.
    pub current_pass: Option<String>,
    /// The full reading (an `AssetGenerateResponse`) once succeeded.
    #[schema(value_type = Option<Object>)]
    pub result: Option<Value>,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<AssetJob> for AssetJobResponse {
    fn from(job: AssetJob) -> Self {
        Self {
            job_id: job.id.to_string(),
            status: job.status,
            mode: job.mode,
            completed_passes: job.completed_passes,
            total_passes: job.total_passes,
            current_pass: job.current_pass,
            result: job.result,
            error: job.error,
            created_at: job.created_at.to_rfc3339(),
            updated_at: job.updated_at.to_rfc3339(),
        }
    }
}

fn job_repository(state: &AppState) -> Result<Arc<AssetJobRepository>, HandlerError> {
    state
        .auth
        .pool()
        .filter(|_| state.db_available)
        .map(|pool| Arc::new(AssetJobRepository::new(pool.clone())))
        .ok_or_else(|| {
            error(
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                "Reading jobs require a database connection",
                None,
            )
        })
}

fn database_error(e: impl std::fmt::Display) -> HandlerError {
    tracing::error!("[assets] job store error: {e}");
    error(
        StatusCode::SERVICE_UNAVAILABLE,
        "SERVICE_UNAVAILABLE",
        "The reading job store is unavailable",
        None,
    )
}

struct JobProgress {
    repo: Arc<AssetJobRepository>,
    id: uuid::Uuid,
}

#[async_trait::async_trait]
impl ReadingProgress for JobProgress {
    async fn pass_started(&self, completed: usize, pass_id: &str) {
        if let Err(e) = self
            .repo
            .progress(self.id, completed as i32, Some(pass_id))
            .await
        {
            tracing::warn!("[assets] job {} progress not saved: {e}", self.id);
        }
    }
}

async fn run_job(
    repo: Arc<AssetJobRepository>,
    id: uuid::Uuid,
    prepared: PreparedReading,
    writer: PassWriter,
) {
    let progress = JobProgress {
        repo: repo.clone(),
        id,
    };
    let outcome = write_passes(&prepared.plan, &prepared.inputs(), &writer, &progress).await;
    let saved = match outcome {
        Ok(written) => match serde_json::to_value(finish_reading(prepared, written)) {
            Ok(result) => repo.succeed(id, &result).await,
            Err(e) => {
                repo.fail(id, &format!("The reading could not be saved: {e}"))
                    .await
            }
        },
        Err(message) => repo.fail(id, &message).await,
    };
    if let Err(e) = saved {
        tracing::error!("[assets] job {id} outcome not saved: {e}");
    }
}

/// POST /api/v1/assets/jobs
///
/// Start a model-written reading in the background. A written L0 reading is
/// twelve sequential model passes and takes several minutes, longer than one
/// request may last behind the edge proxy, so the client polls
/// `GET /api/v1/assets/jobs/{job_id}`. One job runs per user at a time.
#[utoipa::path(
    post,
    path = "/api/v1/assets/jobs",
    tag = "assets",
    request_body = AssetGenerateRequest,
    responses(
        (status = 202, description = "Reading job started", body = AssetJobResponse),
        (status = 409, description = "A reading job is already running for this user"),
        (status = 422, description = "This mode cannot be written by a model yet"),
        (status = 503, description = "No language model or job store is available"),
    )
)]
pub async fn start_job(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    api_key_principal: Option<Extension<crate::ApiKeyPrincipal>>,
    Json(req): Json<AssetGenerateRequest>,
) -> Result<(StatusCode, Json<AssetJobResponse>), HandlerError> {
    let repo = job_repository(&state)?;
    // Check what a job needs before running any engines.
    let llm = LlmClient::for_reading(&user.tier).ok_or_else(|| {
        error(
            StatusCode::SERVICE_UNAVAILABLE,
            "MODEL_UNAVAILABLE",
            "No language model is configured for written readings",
            None,
        )
    })?;
    let prepared = prepare_reading(&state, &user, api_key_principal.is_some(), req).await?;
    if prepared.plan.document.is_none() {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "MODE_NOT_WRITABLE",
            "Only integrated-kundali-l0 has an authored mode document for model-written readings",
            Some(serde_json::json!({ "mode": prepared.mode })),
        ));
    }
    let total = prepared.plan.passes.len() as i32;
    let job = match repo
        .start(&user.user_id, &prepared.mode, total, JOB_STALE_AFTER_SECS)
        .await
    {
        Ok(job) => job,
        Err(StartJobError::AlreadyRunning(id)) => {
            return Err(error(
                StatusCode::CONFLICT,
                "JOB_IN_PROGRESS",
                "A reading is already being written; wait for it to finish",
                Some(serde_json::json!({ "job_id": id.to_string() })),
            ))
        }
        Err(StartJobError::Database(e)) => return Err(database_error(e)),
    };
    tokio::spawn(run_job(
        repo,
        job.id,
        prepared,
        PassWriter::Model(Arc::new(llm)),
    ));
    Ok((StatusCode::ACCEPTED, Json(job.into())))
}

/// GET /api/v1/assets/jobs/{job_id}
///
/// Progress of a reading job, and the reading once it has succeeded. Only
/// the user who started the job can see it.
#[utoipa::path(
    get,
    path = "/api/v1/assets/jobs/{job_id}",
    tag = "assets",
    params(("job_id" = String, Path, description = "Job id from POST /api/v1/assets/jobs")),
    responses(
        (status = 200, description = "Job state", body = AssetJobResponse),
        (status = 404, description = "No such job for this user"),
    )
)]
pub async fn get_job(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(job_id): Path<String>,
) -> Result<Json<AssetJobResponse>, HandlerError> {
    let not_found = || {
        error(
            StatusCode::NOT_FOUND,
            "JOB_NOT_FOUND",
            "No such reading job",
            None,
        )
    };
    let id = uuid::Uuid::parse_str(&job_id).map_err(|_| not_found())?;
    let repo = job_repository(&state)?;
    repo.get_for_user(id, &user.user_id, JOB_STALE_AFTER_SECS)
        .await
        .map_err(database_error)?
        .map(|job| Json(job.into()))
        .ok_or_else(not_found)
}

async fn run_engine(
    orch: &noesis_orchestrator::WorkflowOrchestrator,
    allowed: bool,
    engine_id: &str,
    input: EngineInput,
    level: u8,
) -> Option<Value> {
    if !allowed {
        return None;
    }
    match orch.execute_engine(engine_id, input, level).await {
        Ok(output) => Some(output.result),
        Err(e) => {
            tracing::debug!("[assets] engine {} skipped: {}", engine_id, e);
            None
        }
    }
}

// --- Mode documents ---

fn load_mode_document(mode: &str) -> Result<ReadingPlan, String> {
    // Strategy: embed a small canonical set (same structure as packages/witness-pipeline/modes).
    // This keeps crates/noesis-api self-contained for the additive surface.
    // Full external load (from known path or bundle) is additive later.
    //
    // FIX (using-superpowers + real data case study): unknown modes now error instead of
    // silently falling to "default:Reading". This was the exact lie exposed when
    // THIS-MODE-DOES-NOT-EXIST-xyz, synastry, bridge-query etc. all produced identical output.
    match mode {
        "integrated-reading" | "composite-dyad" => Ok(seed_only_plan(&[
            ("alpha", "Structural Field"),
            ("beta", "Somatic Field"),
        ])),
        "integrated-kundali-l0" | "kundali-l0" | "kundali" => {
            let doc = l0_mode();
            Ok(ReadingPlan {
                passes: doc.pass_plan.clone(),
                document: Some(doc),
            })
        }
        // Other real authored modes from packages/witness-pipeline/modes/
        // For the additive Rust surface we whitelist them (full rich content lives in the TS pipeline + md docs).
        // The critical fix is: they no longer collapse to the same 1-pass "default:Reading".
        "birth-blueprint" | "partner-synastry" | "business-partners" | "family-penta"
        | "married-partners" | "mother-son-lineage" | "unmarried-partners"
        | "integrated-reading-l4" => Ok(seed_only_plan(&[("core", "Core Reading")])),
        _ => Err(format!("unknown mode '{}'; only integrated-reading, integrated-kundali-l0 and the 8 relationship/lineage modes are currently supported on this surface", mode)),
    }
}

/// A pass plan for modes without an authored document; these are seed-rendered only.
fn seed_only_plan(passes: &[(&str, &str)]) -> ReadingPlan {
    ReadingPlan {
        passes: passes
            .iter()
            .map(|(id, title)| PassPlan {
                id: id.to_string(),
                title: title.to_string(),
                target_words: 0,
                template: String::new(),
            })
            .collect(),
        document: None,
    }
}

struct SourcePackInput<'a> {
    person_id: &'a str,
    register: &'a str,
    engines: &'a [String],
    facts_count: usize,
    section_rubrics: &'a [SectionRubric],
    gate: &'a GateSummary,
    report_level: Option<&'a str>,
    language: Option<&'a str>,
    subjects: &'a [noesis_core::intake::ReportSubjectInput],
    relationship_context: Option<&'a noesis_core::intake::RelationshipContext>,
}

// Factory + audit shape (mirrors packages/witness-pipeline/src/assets/{factory,audit}.ts)
fn build_source_pack_with_audit(input: SourcePackInput<'_>) -> Value {
    let SourcePackInput {
        person_id,
        register,
        engines,
        facts_count,
        section_rubrics,
        gate,
        report_level,
        language,
        subjects,
        relationship_context,
    } = input;
    let now = Utc::now().to_rfc3339();
    let mut sp = serde_json::json!({
        "person_id": person_id,
        "created_at": now,
        "mode": "asset",
        "register": register,
        "engines": engines,
        "quality": {
            "facts_count": facts_count,
            "gate_status": gate.gate_status,
            "sections": section_rubrics,
        },
        "audit": {
            "blockers": gate.blockers,
            "warnings": gate.warnings,
            "passed": gate.passed,
        }
    });
    if let Some(rl) = report_level {
        if let Some(obj) = sp.as_object_mut() {
            obj.insert("report_level".to_string(), serde_json::json!(rl));
        }
    }
    if !subjects.is_empty() {
        let subs_json: Vec<Value> = subjects
            .iter()
            .map(|s| {
                let mut subj = serde_json::json!({
                    "role": s.role,
                    "name": s.name,
                    "birth_date": s.birth_date,
                    "birth_time": s.birth_time,
                    "normalized_location": s.normalized_location,
                });
                if let Some(lbl) = &s.relationship_label {
                    if let Some(o) = subj.as_object_mut() {
                        o.insert("relationship_label".to_string(), serde_json::json!(lbl));
                    }
                }
                subj
            })
            .collect();
        if let Some(obj) = sp.as_object_mut() {
            obj.insert("subjects".to_string(), serde_json::json!(subs_json));
            obj.insert(
                "subject_count".to_string(),
                serde_json::json!(subjects.len()),
            );
            if let Some(first) = subjects
                .first()
                .and_then(|s| s.normalized_location.as_ref())
            {
                obj.insert(
                    "first_normalized_location".to_string(),
                    serde_json::json!({
                        "display_name": first.display_name,
                        "latitude": first.latitude,
                        "longitude": first.longitude,
                        "timezone": first.timezone,
                    }),
                );
            }
        }
    }
    if let Some(rc) = relationship_context {
        if let Some(obj) = sp.as_object_mut() {
            obj.insert(
                "relationship_context".to_string(),
                serde_json::to_value(rc).unwrap_or(serde_json::json!({})),
            );
        }
    }
    if let Some(lang) = language {
        if let Some(obj) = sp.as_object_mut() {
            obj.insert("language".to_string(), serde_json::json!(lang));
        }
    }
    sp
}
