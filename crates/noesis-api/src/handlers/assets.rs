//! POST /api/v1/assets/generate — Premium asset generation (additive only)
//!
//! New surface for premium integrated readings and source-pack generation.
//! Uses the existing orchestrator to gather engine context, then returns a
//! structure aligned with witness-pipeline (passes, assembled, source-pack).
//!
//! This file and route are purely additive. No existing route, handler,
//! or response shape is modified.

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
    let (panchanga, numerology, human_design, gene_keys, vimshottari, biorhythm) = tokio::join!(
        run_engine(&orch, "panchanga", input.clone(), level),
        run_engine(&orch, "numerology", input.clone(), level),
        run_engine(&orch, "human-design", input.clone(), level),
        run_engine(&orch, "gene-keys", input.clone(), level),
        run_engine(&orch, "vimshottari", input.clone(), level),
        run_engine(&orch, "biorhythm", input.clone(), level),
    );

    let mut engines_used: Vec<String> = vec![];
    let mut passes: Vec<AssetPass> = vec![];
    let mut assembled_parts: Vec<String> = vec![];

    let mut push = |id: &str, title: &str, v: Option<Value>| {
        if let Some(val) = v {
            engines_used.push(id.to_string());
            let output = serde_json::to_string(&val).unwrap_or_default();
            passes.push(AssetPass {
                id: id.to_string(),
                title: title.to_string(),
                output: output.clone(),
            });
            assembled_parts.push(format!("{title}: {output}"));
        }
    };

    push("panchanga", "Panchanga Context", panchanga);
    push("numerology", "Numerology", numerology);
    push("human-design", "Human Design", human_design);
    push("gene-keys", "Gene Keys", gene_keys);
    push("vimshottari", "Vimshottari Dasha", vimshottari);
    push("biorhythm", "Biorhythm", biorhythm);

    let register = if consciousness_level <= 3 {
        "l1_l3"
    } else {
        "l4_l5"
    };

    let assembled = assembled_parts.join("\n\n");

    let source_pack = Some(serde_json::json!({
        "person_id": user.user_id,
        "created_at": now.to_rfc3339(),
        "mode": req.mode,
        "register": register,
        "engines": engines_used,
        "quality": {
            "facts_count": engines_used.len(),
            "gate_status": if engines_used.len() >= 3 { "ready" } else { "partial" }
        }
    }));

    Ok(Json(AssetGenerateResponse {
        mode: req.mode,
        register: register.to_string(),
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
