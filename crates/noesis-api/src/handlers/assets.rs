//! POST /api/v1/assets/generate — Premium asset generation (additive only)
//!
//! New surface for premium integrated readings and source-pack generation.
//! Fetches minimal engine seeds via the existing orchestrator (partial-results
//! tolerant), loads the mode doc, then drives generation through a Rust mirror
//! of @noesis/witness-pipeline's IntegratedReadingOrchestrator + factory + audit.
//!
//! This file and route are purely additive. No existing route, handler,
//! or response shape (including /witness/interpret) is modified.

use crate::AppState;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use chrono::Utc;
use noesis_auth::AuthUser;
use noesis_core::{BirthData, EngineInput};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct AssetGenerateRequest {
    /// Birth data for engine calculations.
    pub birth_data: Option<BirthData>,
    /// Mode key, e.g. "integrated-reading", "birth-blueprint".
    pub mode: String,
    /// User's consciousness level (0–5). Server will max with auth level.
    #[serde(default)]
    pub consciousness_level: u8,
    /// Free-form options (passed through to pipeline).
    pub options: Option<Value>,
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

/// POST /api/v1/assets/generate
///
/// Additive premium asset endpoint. Gathers engine results and returns
/// a witness-pipeline-compatible shape (mode, register, passes, assembled).
/// Full multi-pass LLM orchestration is provided by the witness-pipeline package.
#[utoipa::path(
    post,
    path = "/api/v1/assets/generate",
    tag = "assets",
    request_body = AssetGenerateRequest,
    responses(
        (status = 200, description = "Premium asset generated (additive surface)", body = AssetGenerateResponse),
    )
)]
pub async fn generate(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<AssetGenerateRequest>,
) -> Result<Json<AssetGenerateResponse>, (StatusCode, Json<crate::error_mapper::ErrorResponse>)> {
    let consciousness_level = req.consciousness_level.max(user.consciousness_level);
    let now = Utc::now();

    let make_input = || EngineInput {
        birth_data: req.birth_data.clone(),
        current_time: now,
        location: None,
        precision: noesis_core::Precision::Standard,
        options: {
            let mut m = std::collections::HashMap::new();
            m.insert("user_id".to_string(), serde_json::json!(user.user_id));
            m.insert("mode".to_string(), serde_json::json!(req.mode.clone()));
            m
        },
    };

    let input = make_input();
    let level = consciousness_level;
    let orch = state.orchestrator.clone();

    // Gather a focused set of engines to seed the asset pipeline.
    // (Partial results are acceptable — mirrors existing witness behavior.)
    // These seeds are the input to the canonical pipeline (same as SelemeneClient in @noesis/witness-pipeline).
    let (panchanga, numerology, human_design, gene_keys, vimshottari, biorhythm) = tokio::join!(
        run_engine(&orch, "panchanga", input.clone(), level),
        run_engine(&orch, "numerology", input.clone(), level),
        run_engine(&orch, "human-design", input.clone(), level),
        run_engine(&orch, "gene-keys", input.clone(), level),
        run_engine(&orch, "vimshottari", input.clone(), level),
        run_engine(&orch, "biorhythm", input.clone(), level),
    );

    // Collect seeds (id, value) preserving partial-results tolerance.
    let mut engines_used: Vec<String> = vec![];
    let mut engine_seeds: Vec<(String, Value)> = vec![];
    let mut add_seed = |id: &str, v: Option<Value>| {
        if let Some(val) = v {
            engines_used.push(id.to_string());
            engine_seeds.push((id.to_string(), val));
        }
    };
    add_seed("panchanga", panchanga);
    add_seed("numerology", numerology);
    add_seed("human-design", human_design);
    add_seed("gene-keys", gene_keys);
    add_seed("vimshottari", vimshottari);
    add_seed("biorhythm", biorhythm);

    // --- Canonical pipeline path (mirrors @noesis/witness-pipeline) ---
    // 1. Load mode document (embed strategy for self-contained crate; same content as packages/witness-pipeline/modes).
    // 2. Drive IntegratedReadingOrchestrator-equivalent: walk pass_plan to produce passes + assembled.
    //    (LLM call per pass is stubbed with seed rendering for this additive surface; real LLM injection is future additive work.)
    // 3. Populate source_pack via factory + audit logic.
    let mode_doc = load_mode_document(&req.mode);
    let register = if consciousness_level <= 3 {
        "l1_l3".to_string()
    } else {
        "l4_l5".to_string()
    };

    let mut passes: Vec<AssetPass> = vec![];
    let mut assembled_parts: Vec<String> = vec![];
    for pass in &mode_doc.pass_plan {
        let output = render_pass_output(pass, &engine_seeds);
        passes.push(AssetPass {
            id: pass.id.clone(),
            title: pass.title.clone(),
            output: output.clone(),
        });
        assembled_parts.push(format!("## {}\n\n{}", pass.title, output));
    }
    let assembled = assembled_parts.join("\n\n");

    // Factory + audit (deterministic facts gate, manifest shape)
    let facts_count = engine_seeds.len();
    let source_pack = Some(build_source_pack_with_audit(
        &user.user_id,
        &assembled,
        &engines_used,
        facts_count,
    ));

    Ok(Json(AssetGenerateResponse {
        mode: req.mode,
        register,
        passes,
        assembled,
        engines_used,
        source_pack,
    }))
}

async fn run_engine(
    orch: &noesis_orchestrator::WorkflowOrchestrator,
    engine_id: &str,
    input: EngineInput,
    level: u8,
) -> Option<Value> {
    match orch.execute_engine(engine_id, input, level).await {
        Ok(output) => Some(output.result),
        Err(e) => {
            tracing::debug!("[assets] engine {} skipped: {}", engine_id, e);
            None
        }
    }
}

// --- Minimal embedded mode documents (canonical content, self-contained crate) ---

#[derive(Clone, Debug)]
struct PassDef {
    id: String,
    title: String,
}

#[derive(Clone, Debug)]
struct ModeDoc {
    pass_plan: Vec<PassDef>,
}

fn load_mode_document(mode: &str) -> ModeDoc {
    // Strategy: embed a small canonical set (same structure as packages/witness-pipeline/modes).
    // This keeps crates/noesis-api self-contained for the additive surface.
    // Full external load (from known path or bundle) is additive later.
    match mode {
        "integrated-reading" | "composite-dyad" => ModeDoc {
            pass_plan: vec![
                PassDef {
                    id: "alpha".to_string(),
                    title: "Structural Field".to_string(),
                },
                PassDef {
                    id: "beta".to_string(),
                    title: "Somatic Field".to_string(),
                },
            ],
        },
        _ => ModeDoc {
            pass_plan: vec![PassDef {
                id: "default".to_string(),
                title: "Reading".to_string(),
            }],
        },
    }
}

// Deterministic render for a pass using collected engine seeds (no LLM in this crate).
// Mirrors the pipeline shape: prior context + engine seeds → pass text.
fn render_pass_output(pass: &PassDef, seeds: &[(String, Value)]) -> String {
    let mut lines: Vec<String> = vec![format!("Pass {} — {}", pass.id, pass.title)];
    for (id, val) in seeds.iter().take(6) {
        let snippet = serde_json::to_string(val).unwrap_or_default();
        let short = if snippet.len() > 240 {
            format!("{}…", &snippet[..240])
        } else {
            snippet
        };
        lines.push(format!("- {}: {}", id, short));
    }
    if lines.len() == 1 {
        lines.push("(partial seed context)".to_string());
    }
    lines.join("\n")
}

// Factory + audit shape (mirrors packages/witness-pipeline/src/assets/{factory,audit}.ts)
fn build_source_pack_with_audit(
    person_id: &str,
    reading_markdown: &str,
    engines: &[String],
    facts_count: usize,
) -> Value {
    let now = Utc::now().to_rfc3339();
    let gate = if facts_count >= 3 { "ready" } else { "partial" };
    let mut warnings: Vec<&str> = vec![];
    if reading_markdown.len() < 100 {
        warnings.push("reading_short");
    }
    serde_json::json!({
        "person_id": person_id,
        "created_at": now,
        "mode": "asset",
        "register": "l1_l3", // caller can override in response
        "engines": engines,
        "quality": {
            "facts_count": facts_count,
            "gate_status": gate,
        },
        "audit": {
            "blockers": [],
            "warnings": warnings,
            "passed": facts_count >= 3,
        }
    })
}
