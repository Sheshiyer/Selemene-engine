//! POST /api/v1/witness/interpret — multi-engine LLM Witness Dyad
//!
//! Accepts birth data + live biofield scores, runs all available engines in parallel,
//! then calls the enterprise LLM (OpenAI-compatible, incl. NVIDIA NIM) to produce
//! Aletheios + Pichet perspectives. Falls back to rule-based dyad if no API key.

use crate::AppState;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use chrono::Utc;
use noesis_auth::AuthUser;
use noesis_core::{BirthData, EngineInput, intake};
use noesis_data::models::witness_dyad::NewWitnessDyadExecution;
use noesis_witness::{
    interpret_with_llm, LiveBiofieldScores, RelationshipMode, WitnessContext,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Instant;
use utoipa::ToSchema;
use uuid::Uuid;

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
    /// Optional second-person birth data for synastry / composite readings.
    pub partner_birth_data: Option<BirthData>,
    /// Relationship framing for synastry / composite readings (narrow enum).
    #[serde(default)]
    pub relationship_mode: RelationshipMode,
    /// Optional richer relationship context (preferred for parity with assets path).
    pub relationship_context: Option<intake::RelationshipContext>,

    /// Optional language code (additive for future orchestrator/prompt parity with assets path).
    /// Not used by the narrow RelationshipMode dyad path; language is orchestrator concern.
    #[serde(default)]
    pub language: Option<String>,
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
) -> Result<Json<WitnessInterpretResponse>, (StatusCode, Json<crate::error_mapper::ErrorResponse>)>
{
    let now = Utc::now();
    let consciousness_level = req.consciousness_level.max(user.consciousness_level);
    let user_name = req.user_name;
    let start = Instant::now();

    // Map rich relationship_context to narrow RelationshipMode for WitnessContext parity.
    let relationship_mode = if req.relationship_mode != RelationshipMode::None {
        req.relationship_mode
    } else if let Some(rc) = &req.relationship_context {
        map_relationship_context_to_mode(rc)
    } else if req.partner_birth_data.is_some() {
        // Sensible default for narrow dyad path when partner present but no explicit mode.
        RelationshipMode::CompositeDyad
    } else {
        RelationshipMode::None
    };

    // ── Run engines in parallel ───────────────────────────────────────────────
    // Only birth-dependent engines run when birth_data is present.
    // Biorhythm and panchanga always run (they use current time).
    let bd = req.birth_data.clone();
    let partner_bd = req.partner_birth_data.clone();
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

    // Panchanga in the witness always reflects TODAY's cosmic weather, not birth.
    // Pass mode=daily so the engine uses current_time date instead of birth_data.date.
    let panchanga_input = {
        let mut inp = make_input(bd.clone());
        inp.options
            .insert("mode".to_string(), serde_json::json!("daily"));
        inp
    };
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

    // ── Partner context (if synastry/composite) ───────────────────────────────
    let partner_context = if let Some(partner_bd) = partner_bd {
        let partner_input = |bd: Option<BirthData>| EngineInput {
            birth_data: bd,
            current_time: now,
            location: None,
            precision: noesis_core::Precision::Standard,
            options: std::collections::HashMap::new(),
        };
        let (p_hd, p_num, p_gk, p_vim) = tokio::join!(
            run_engine(&orch, "human-design", partner_input(Some(partner_bd.clone())), level),
            run_engine(&orch, "numerology", partner_input(Some(partner_bd.clone())), level),
            run_engine(&orch, "gene-keys", partner_input(Some(partner_bd.clone())), level),
            run_engine(&orch, "vimshottari", partner_input(Some(partner_bd.clone())), level),
        );
        Some(Box::new(WitnessContext {
            user_name: partner_bd.name,
            human_design: p_hd,
            numerology: p_num,
            gene_keys: p_gk,
            vimshottari: p_vim,
            ..Default::default()
        }))
    } else {
        None
    };

    // ── Build witness context ─────────────────────────────────────────────────
    let ctx = WitnessContext {
        live_scores: req.live_scores.clone(),
        consciousness_level,
        user_name,
        panchanga,
        human_design,
        numerology,
        biorhythm,
        transits,
        gene_keys,
        vimshottari,
        partner_context,
        relationship_mode: relationship_mode.clone(),
    };

    // ── Compute engines_available for persistence ─────────────────────────────
    let mut engines_available = vec!["biofield".to_string()];
    if ctx.panchanga.is_some() { engines_available.push("panchanga".into()); }
    if ctx.human_design.is_some() { engines_available.push("human-design".into()); }
    if ctx.numerology.is_some() { engines_available.push("numerology".into()); }
    if ctx.biorhythm.is_some() { engines_available.push("biorhythm".into()); }
    if ctx.transits.is_some() { engines_available.push("transits".into()); }
    if ctx.gene_keys.is_some() { engines_available.push("gene-keys".into()); }
    if ctx.vimshottari.is_some() { engines_available.push("vimshottari".into()); }
    engines_available.sort();
    engines_available.dedup();

    // ── LLM interpretation (with rule-based fallback) ─────────────────────────
    let user_id_for_persist = Uuid::parse_str(&user.user_id).ok();
    let tier_for_persist = user.tier.clone();
    let admin_repo = state.admin_repository.clone();

    if let Some(llm_result) = interpret_with_llm(&ctx, &user.tier).await {
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Fire-and-forget persistence
        if let (Some(uid), Some(repo)) = (user_id_for_persist, admin_repo) {
            let persist_record = NewWitnessDyadExecution {
                user_id: uid,
                tier: tier_for_persist,
                consciousness_level: consciousness_level as i16,
                live_scores: serde_json::to_value(&req.live_scores).unwrap_or_default(),
                relationship_mode: serde_json::to_string(&relationship_mode)
                    .unwrap_or_else(|_| "None".into())
                    .trim_matches('"')
                    .to_string(),
                engines_available: engines_available.clone(),
                aletheios: Some(llm_result.aletheios.clone()),
                pichet: Some(llm_result.pichet.clone()),
                synthesis: Some(llm_result.synthesis.clone()),
                witness_question: Some(llm_result.witness_question.clone()),
                engines_used: llm_result.engines_used.clone(),
                llm_powered: true,
                llm_provider: Some("nvidia/openrouter".into()),
                llm_model_aletheios: None,
                llm_model_pichet: None,
                llm_model_synthesis: None,
                llm_duration_ms: Some(duration_ms),
                error_message: None,
                request_ip_hash: None,
            };
            tokio::spawn(async move {
                let _ = repo.save_witness_dyad_execution(&persist_record).await;
            });
        }

        return Ok(Json(WitnessInterpretResponse {
            aletheios: llm_result.aletheios,
            pichet: llm_result.pichet,
            synthesis: llm_result.synthesis,
            witness_question: llm_result.witness_question,
            engines_used: llm_result.engines_used,
            llm_powered: true,
        }));
    }

    // Fallback: rule-based dyad from biofield metrics and available engine context
    let (aletheios, pichet, synthesis, witness_question, engines_used) =
        rule_based_dyad(&ctx, &user.tier);

    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Fire-and-forget persistence for rule-based fallback
    if let (Some(uid), Some(repo)) = (user_id_for_persist, admin_repo) {
        let persist_record = NewWitnessDyadExecution {
            user_id: uid,
            tier: tier_for_persist,
            consciousness_level: consciousness_level as i16,
            live_scores: serde_json::to_value(&req.live_scores).unwrap_or_default(),
            relationship_mode: serde_json::to_string(&relationship_mode)
                .unwrap_or_else(|_| "None".into())
                .trim_matches('"')
                .to_string(),
            engines_available: engines_available.clone(),
            aletheios: Some(aletheios.clone()),
            pichet: Some(pichet.clone()),
            synthesis: Some(synthesis.clone()),
            witness_question: Some(witness_question.clone()),
            engines_used: engines_used.clone(),
            llm_powered: false,
            llm_provider: None,
            llm_model_aletheios: None,
            llm_model_pichet: None,
            llm_model_synthesis: None,
            llm_duration_ms: Some(duration_ms),
            error_message: None,
            request_ip_hash: None,
        };
        tokio::spawn(async move {
            let _ = repo.save_witness_dyad_execution(&persist_record).await;
        });
    }

    Ok(Json(WitnessInterpretResponse {
        aletheios,
        pichet,
        synthesis,
        witness_question,
        engines_used,
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

/// Rule-based fallback dyad — used when no OPENAI_API_KEY is set or LLM call fails.
/// Takes full context and tier for richer, tier-gated responses.
fn rule_based_dyad(
    ctx: &WitnessContext,
    tier: &str,
) -> (String, String, String, String, Vec<String>) {
    let scores = &ctx.live_scores;
    let level = ctx.consciousness_level;
    let mut engines_used = vec!["biofield".to_string()];

    // Tier gating: free/basic gets simpler responses
    let is_premium = matches!(tier, "premium" | "enterprise" | "founder");

    // Build Aletheios response (truth-revealing, still)
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
    } else if is_premium && ctx.vimshottari.is_some() {
        engines_used.push("vimshottari".to_string());
        format!(
            "Energy at {:.0}%, regulation at {:.0}%. Your dasha period colors how this energy \
             expresses. What does your body already know that your attention has not yet reached?",
            scores.energy * 100.0,
            scores.regulation * 100.0
        )
    } else {
        format!(
            "Energy at {:.0}%, regulation at {:.0}%. The field is in motion. \
             What does your body already know that your attention has not yet reached?",
            scores.energy * 100.0,
            scores.regulation * 100.0
        )
    };

    // Build Pichet response (vitalizing, action-oriented)
    let pichet = if scores.energy > 0.72 {
        if is_premium && ctx.gene_keys.is_some() {
            engines_used.push("gene-keys".to_string());
            format!(
                "Your biofield is at {:.0}% energy — there is aliveness present. \
                 Your Gene Keys point to where this energy wants to express. \
                 Let it move. Notice what your hands want to do right now.",
                scores.energy * 100.0
            )
        } else {
            format!(
                "Your biofield is at {:.0}% energy — there is aliveness present. \
                 Let it move. Notice what your hands want to do right now. \
                 Feel the edge where stillness meets motion.",
                scores.energy * 100.0
            )
        }
    } else if is_premium && ctx.biorhythm.is_some() {
        engines_used.push("biorhythm".to_string());
        format!(
            "Regulation at {:.0}% — your system is recalibrating with your biorhythm cycles. \
             What one small action would feel like a genuine yes? Not a should. A yes.",
            scores.regulation * 100.0
        )
    } else {
        format!(
            "Regulation at {:.0}% — your system is recalibrating. \
             What one small action would feel like a genuine yes? \
             Not a should. A yes.",
            scores.regulation * 100.0
        )
    };

    // Synthesis varies by tier
    let synthesis = if is_premium && ctx.panchanga.is_some() {
        engines_used.push("panchanga".to_string());
        "The field holds both truth and aliveness simultaneously, shaped by today's \
         cosmic weather. Neither is more important than the other right now."
            .to_string()
    } else {
        "The field holds both truth and aliveness simultaneously. \
         Neither is more important than the other right now."
            .to_string()
    };

    // Witness question based on consciousness level
    let question = match level {
        0 | 1 => "What are you noticing right now, without adding a story?".to_string(),
        2 => "Who is the one watching all of this?".to_string(),
        3 => {
            if is_premium {
                "What pattern wants to complete itself through you today?".to_string()
            } else {
                "What wants to emerge through you right now?".to_string()
            }
        }
        _ => {
            if is_premium {
                "What is the next smallest step that serves the whole?".to_string()
            } else {
                "What wants to emerge through you right now?".to_string()
            }
        }
    };

    // Deduplicate engines_used
    engines_used.sort();
    engines_used.dedup();

    (aletheios, pichet, synthesis, question, engines_used)
}

/// Map rich relationship_context.type (or mapping_goal) strings to the narrow RelationshipMode.
/// Used to keep the narrow dyad path in parity with rich multi-subject requests.
fn map_relationship_context_to_mode(rc: &intake::RelationshipContext) -> RelationshipMode {
    // Prefer explicit type if present.
    if let Some(t) = rc.r#type.as_deref() {
        let t = t.to_ascii_lowercase();
        return match t.as_str() {
            "family" | "family-triad" => RelationshipMode::FamilyTriad,
            "business-partners" => RelationshipMode::BusinessPartners,
            "unmarried-partners" | "married-partners" | "partner-synastry" => RelationshipMode::PartnerSynastry,
            "composite-dyad" | "composite" => RelationshipMode::CompositeDyad,
            _ => RelationshipMode::CompositeDyad,
        };
    }
    // Fallback: inspect mapping_goal for keywords.
    if let Some(goal) = rc.mapping_goal.as_deref() {
        let g = goal.to_ascii_lowercase();
        if g.contains("family") || g.contains("mother") || g.contains("father") || g.contains("son") || g.contains("daughter") {
            return RelationshipMode::FamilyTriad;
        }
        if g.contains("business") || g.contains("partner") {
            return RelationshipMode::BusinessPartners;
        }
        if g.contains("partner") || g.contains("spouse") || g.contains("husband") || g.contains("wife") {
            return RelationshipMode::PartnerSynastry;
        }
    }
    RelationshipMode::CompositeDyad
}
