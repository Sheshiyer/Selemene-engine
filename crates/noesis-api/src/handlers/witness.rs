//! POST /api/v1/witness/interpret — multi-engine LLM Witness Dyad
//!
//! Accepts birth data + live biofield scores, runs all available engines in parallel,
//! then calls the enterprise LLM (OpenAI-compatible, incl. NVIDIA NIM) to produce
//! Aletheios + Pichet perspectives. Falls back to rule-based dyad if no API key.

use crate::{AppState};
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use chrono::Utc;
use noesis_auth::AuthUser;
use noesis_core::{BirthData, EngineInput};
use noesis_witness::{
    interpret::{interpret_with_llm, LiveBiofieldScores, WitnessContext},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

/// Request body for the witness interpret endpoint.
#[derive(Deserialize, ToSchema)]
pub struct WitnessInterpretRequest {
    /// Birth data for engine calculations (optional — skips birth-dependent engines if absent).
    pub birth_data: Option<BirthData>,
    /// Live biofield composite scores from PIP camera analysis.
    pub live_scores: LiveBiofieldScores,
    /// User's consciousness level (0–5).
    #[serde(default)]
    pub consciousness_level: u8,
    /// Optional display name for personalised language.
    pub user_name: Option<String>,
}

/// Witness Dyad interpretation response.
#[derive(Serialize, ToSchema)]
pub struct WitnessInterpretResponse {
    /// Aletheios — Left Pillar: truth-revealing, contemplative.
    pub aletheios: String,
    /// Pichet — Right Pillar: vitalizing, action-oriented.
    pub pichet: String,
    /// Synthesis of both pillars.
    pub synthesis: String,
    /// Open somatic inquiry question.
    pub witness_question: String,
    /// Which engine results contributed to this interpretation.
    pub engines_used: Vec<String>,
    /// Whether an LLM was used (true) or rule-based fallback (false).
    pub llm_powered: bool,
}

/// POST /api/v1/witness/interpret
///
/// Runs all available consciousness engines in parallel, then calls the
/// enterprise LLM to synthesise a personalised Witness Dyad interpretation.
pub async fn interpret(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<WitnessInterpretRequest>,
) -> Result<Json<WitnessInterpretResponse>, (StatusCode, Json<crate::error_mapper::ErrorResponse>)> {
    let now = Utc::now();
    let consciousness_level = req.consciousness_level.max(user.consciousness_level);
    let user_name = req.user_name;

    // ── Run engines in parallel ───────────────────────────────────────────────
    // Only birth-dependent engines run when birth_data is present.
    // Biorhythm and panchanga always run (they use current time).
    let bd = req.birth_data.clone();
    let level = consciousness_level;

    let make_input = |bd_override: Option<BirthData>| EngineInput {
        birth_data: bd_override,
        current_time: now,
        location: None,
        precision: noesis_core::Precision::Standard,
        options: {
            let mut m = std::collections::HashMap::new();
            m.insert("user_id".to_string(), serde_json::json!(user.user_id));
            m
        },
    };

    let panchanga_input = make_input(bd.clone());
    let biorhythm_input = make_input(bd.clone());
    let hd_input = make_input(bd.clone());
    let numerology_input = make_input(bd.clone());
    let transits_input = make_input(bd.clone());
    let gene_keys_input = make_input(bd.clone());
    let vimshottari_input = make_input(bd.clone());

    let orch = state.orchestrator.clone();

    // Run all engines concurrently; ignore individual failures (partial context is fine)
    let (panchanga, biorhythm, human_design, numerology, transits, gene_keys, vimshottari) = tokio::join!(
        run_engine(&orch, "panchanga", panchanga_input, level),
        run_engine(&orch, "biorhythm", biorhythm_input, level),
        run_engine(&orch, "human-design", hd_input, level),
        run_engine(&orch, "numerology", numerology_input, level),
        run_engine(&orch, "transits", transits_input, level),
        run_engine(&orch, "gene-keys", gene_keys_input, level),
        run_engine(&orch, "vimshottari", vimshottari_input, level),
    );

    // ── Build witness context ─────────────────────────────────────────────────
    let ctx = WitnessContext {
        live_scores: req.live_scores,
        consciousness_level,
        user_name,
        panchanga,
        human_design,
        numerology,
        biorhythm,
        transits,
        gene_keys,
        vimshottari,
    };

    // ── LLM interpretation (with rule-based fallback) ─────────────────────────
    if let Some(llm_result) = interpret_with_llm(&ctx).await {
        return Ok(Json(WitnessInterpretResponse {
            aletheios: llm_result.aletheios,
            pichet: llm_result.pichet,
            synthesis: llm_result.synthesis,
            witness_question: llm_result.witness_question,
            engines_used: llm_result.engines_used,
            llm_powered: true,
        }));
    }

    // Fallback: rule-based dyad from biofield metrics
    let (aletheios, pichet, synthesis, witness_question) =
        rule_based_dyad_from_scores(&ctx.live_scores, consciousness_level);

    Ok(Json(WitnessInterpretResponse {
        aletheios,
        pichet,
        synthesis,
        witness_question,
        engines_used: vec!["biofield".to_string()],
        llm_powered: false,
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
            tracing::debug!("[witness] engine {} skipped: {}", engine_id, e);
            None
        }
    }
}

/// Rule-based fallback dyad — used when no OPENAI_API_KEY is set.
fn rule_based_dyad_from_scores(
    scores: &LiveBiofieldScores,
    level: u8,
) -> (String, String, String, String) {
    let aletheios = if scores.coherence > 0.72 {
        format!(
            "Coherence is at {:.0}% — the field has found a temporary equilibrium. \
             What is the quality of this stillness? Not what it means, not what to do with it — \
             simply: what is it?",
            scores.coherence * 100.0
        )
    } else if scores.symmetry < 0.48 {
        "One half of your biofield is quieter than the other. That quieter half is not broken — \
         it is waiting. What has it been waiting to say?"
            .to_string()
    } else {
        format!(
            "Energy at {:.0}%, regulation at {:.0}%. The field is in motion. \
             What does your body already know that your attention has not yet reached?",
            scores.energy * 100.0,
            scores.regulation * 100.0
        )
    };

    let pichet = if scores.energy > 0.72 {
        format!(
            "Your biofield is at {:.0}% energy — there is aliveness present. \
             Let it move. Notice what your hands want to do right now. \
             Feel the edge where stillness meets motion.",
            scores.energy * 100.0
        )
    } else {
        format!(
            "Regulation at {:.0}% — your system is recalibrating. \
             What one small action would feel like a genuine yes? \
             Not a should. A yes.",
            scores.regulation * 100.0
        )
    };

    let synthesis = "The field holds both truth and aliveness simultaneously. \
                     Neither is more important than the other right now."
        .to_string();

    let question = match level {
        0 | 1 => "What are you noticing right now, without adding a story?".to_string(),
        2 => "Who is the one watching all of this?".to_string(),
        _ => "What wants to emerge through you right now?".to_string(),
    };

    (aletheios, pichet, synthesis, question)
}
