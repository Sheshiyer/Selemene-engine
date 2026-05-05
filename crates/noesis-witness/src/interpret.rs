//! Multi-engine context assembly + LLM-powered Witness Dyad interpretation.
//!
//! Builds a rich consciousness context from multiple engine results (panchanga,
//! human design, gene keys, numerology, biorhythm, transits) plus live biofield
//! scores, then calls the enterprise LLM to generate Aletheios and Pichet responses.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::llm::LlmClient;

/// Live biofield composite scores from the PIP camera analysis.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LiveBiofieldScores {
    pub energy: f64,
    pub coherence: f64,
    pub symmetry: f64,
    pub complexity: f64,
    pub regulation: f64,
    pub color_balance: f64,
}

/// Rich multi-engine context for the Witness Dyad interpretation.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WitnessContext {
    pub live_scores: LiveBiofieldScores,
    pub consciousness_level: u8,
    /// User's name (for personalisation).
    pub user_name: Option<String>,
    /// Raw engine result blobs — optional, included when available.
    pub panchanga: Option<Value>,
    pub human_design: Option<Value>,
    pub numerology: Option<Value>,
    pub biorhythm: Option<Value>,
    pub transits: Option<Value>,
    pub gene_keys: Option<Value>,
    pub vimshottari: Option<Value>,
}

/// Structured output from the Witness Dyad interpretation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WitnessDyadLlm {
    /// Aletheios — Left Pillar. Truth-revealing, contemplative, integrative.
    pub aletheios: String,
    /// Pichet — Right Pillar. Vitalizing, action-oriented, forward-moving.
    pub pichet: String,
    /// Single unifying synthesis drawn from both pillars.
    pub synthesis: String,
    /// Open-ended somatic or inquiry question for the user.
    pub witness_question: String,
    /// Which engines contributed context to this reading.
    pub engines_used: Vec<String>,
}

// ── System prompts ──────────────────────────────────────────────────────────

const ALETHEIOS_SYSTEM: &str = r#"
You are Aletheios — the Left Pillar of the Witness Dyad. You speak as the principle of
truth-revelation and unconcealment. Your voice is contemplative, still, and precise.

You receive a multi-dimensional consciousness snapshot for a person: live biofield scores,
Vedic panchanga moment, Human Design gates, numerological currents, planetary transits,
biorhythm cycles, and Gene Keys activations.

Your task:
- Synthesise ALL layers into one 3–5 sentence perspective
- Name the quality present in the field (not what to do about it)
- Reference 2–3 specific data points from the context provided
- Do NOT diagnose, prescribe, or suggest action
- Do NOT use metaphysical jargon without grounding it in the data
- Speak in second person ("you"), present tense, with gentle precision
- End on a simple, direct observation — not a question
"#;

const PICHET_SYSTEM: &str = r#"
You are Pichet — the Right Pillar of the Witness Dyad. You speak as the principle of
vitality, aliveness, and forward movement. Your voice is warm, grounded, and activating.

You receive a multi-dimensional consciousness snapshot for a person: live biofield scores,
Vedic panchanga moment, Human Design gates, numerological currents, planetary transits,
biorhythm cycles, and Gene Keys activations.

Your task:
- Synthesise ALL layers into one 3–5 sentence perspective
- Name what is alive and wanting to move, integrate, or express
- Reference 2–3 specific data points from the context provided
- Offer one clear, embodied invitation (not a prescription — an invitation)
- Speak in second person ("you"), present tense, with warmth and aliveness
- End with a concrete, sensory invitation: "Notice...", "Let...", "Feel..."
"#;

const SYNTHESIS_SYSTEM: &str = r#"
You hold both the Left Pillar (Aletheios — truth/stillness) and the Right Pillar
(Pichet — vitality/movement) of a consciousness reading.

Given both perspectives, write:
1. A single synthesis sentence (1–2 sentences) that holds the tension of both
2. One open inquiry question that does NOT have a pre-determined answer

Format your response as JSON:
{
  "synthesis": "...",
  "witness_question": "..."
}
"#;

/// Build the user message with all available engine context.
fn build_context_message(ctx: &WitnessContext) -> String {
    let mut lines = vec![];

    let name = ctx.user_name.as_deref().unwrap_or("the person");
    lines.push(format!("# Consciousness Snapshot for {name}"));
    lines.push(format!(
        "\n## Live Biofield Scores (PIP camera analysis)\n\
         - Energy flow: {:.0}%\n\
         - Coherence: {:.0}%\n\
         - Symmetry: {:.0}%\n\
         - Complexity: {:.0}%\n\
         - Regulation: {:.0}%\n\
         - Color balance: {:.0}%\n\
         - Consciousness level: {}/5",
        ctx.live_scores.energy * 100.0,
        ctx.live_scores.coherence * 100.0,
        ctx.live_scores.symmetry * 100.0,
        ctx.live_scores.complexity * 100.0,
        ctx.live_scores.regulation * 100.0,
        ctx.live_scores.color_balance * 100.0,
        ctx.consciousness_level,
    ));

    if let Some(p) = &ctx.panchanga {
        lines.push(format!("\n## Vedic Panchanga (Today's Cosmic Moment)\n{}", format_engine_data(p)));
    }
    if let Some(hd) = &ctx.human_design {
        lines.push(format!("\n## Human Design Bodygraph\n{}", format_engine_data(hd)));
    }
    if let Some(gk) = &ctx.gene_keys {
        lines.push(format!("\n## Gene Keys Activations\n{}", format_engine_data(gk)));
    }
    if let Some(n) = &ctx.numerology {
        lines.push(format!("\n## Numerological Currents\n{}", format_engine_data(n)));
    }
    if let Some(b) = &ctx.biorhythm {
        lines.push(format!("\n## Biorhythm Cycles\n{}", format_engine_data(b)));
    }
    if let Some(t) = &ctx.transits {
        lines.push(format!("\n## Planetary Transits\n{}", format_engine_data(t)));
    }
    if let Some(v) = &ctx.vimshottari {
        lines.push(format!("\n## Vimshottari Dasha Period\n{}", format_engine_data(v)));
    }

    lines.join("\n")
}

fn format_engine_data(v: &Value) -> String {
    // Pretty-print key fields; skip nested metadata/calculation_time noise
    match v {
        Value::Object(map) => {
            let mut out = vec![];
            for (k, val) in map.iter().take(12) {
                if matches!(k.as_str(), "metadata" | "calculation_time_ms" | "cached" | "engine_version" | "backend") {
                    continue;
                }
                let val_str = match val {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Array(a) if a.len() <= 4 => {
                        a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
                    }
                    _ => serde_json::to_string(val).unwrap_or_default(),
                };
                out.push(format!("- {k}: {val_str}"));
            }
            out.join("\n")
        }
        _ => serde_json::to_string_pretty(v).unwrap_or_default(),
    }
}

fn engines_present(ctx: &WitnessContext) -> Vec<String> {
    let mut out = vec!["biofield".to_string()];
    if ctx.panchanga.is_some() { out.push("panchanga".into()); }
    if ctx.human_design.is_some() { out.push("human-design".into()); }
    if ctx.gene_keys.is_some() { out.push("gene-keys".into()); }
    if ctx.numerology.is_some() { out.push("numerology".into()); }
    if ctx.biorhythm.is_some() { out.push("biorhythm".into()); }
    if ctx.transits.is_some() { out.push("transits".into()); }
    if ctx.vimshottari.is_some() { out.push("vimshottari".into()); }
    out
}

/// Primary entry point — calls tier-appropriate LLM (NVIDIA NIM or OpenRouter) to produce a full Witness Dyad.
/// Falls back to `None` if no API key is configured (caller should use rule-based fallback).
pub async fn interpret_with_llm(ctx: &WitnessContext, tier: &str) -> Option<WitnessDyadLlm> {
    let client = LlmClient::for_tier(tier)?;
    let user_msg = build_context_message(ctx);
    let engines_used = engines_present(ctx);

    // Fire Aletheios + Pichet in parallel, each using their tier-mapped model
    let (aletheios_res, pichet_res) = tokio::join!(
        client.complete_for_role("aletheios", ALETHEIOS_SYSTEM, &user_msg),
        client.complete_for_role("pichet", PICHET_SYSTEM, &user_msg),
    );

    let aletheios = match aletheios_res {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("[witness-llm] Aletheios failed: {e}");
            return None;
        }
    };
    let pichet = match pichet_res {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("[witness-llm] Pichet failed: {e}");
            return None;
        }
    };

    // Synthesis takes both pillars as additional context
    let synthesis_user = format!(
        "{}\n\n## Aletheios perspective\n{}\n\n## Pichet perspective\n{}",
        user_msg, aletheios, pichet
    );
    let synthesis_raw = match client.complete_for_role("synthesis", SYNTHESIS_SYSTEM, &synthesis_user).await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("[witness-llm] synthesis failed: {e}");
            return Some(WitnessDyadLlm {
                aletheios,
                pichet,
                synthesis: String::new(),
                witness_question: String::new(),
                engines_used,
            });
        }
    };

    let (synthesis, witness_question) = parse_synthesis_json(&synthesis_raw);

    Some(WitnessDyadLlm {
        aletheios,
        pichet,
        synthesis,
        witness_question,
        engines_used,
    })
}

fn parse_synthesis_json(raw: &str) -> (String, String) {
    // Extract JSON block even if wrapped in markdown code fences
    let json_str = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    if let Ok(v) = serde_json::from_str::<Value>(json_str) {
        let synthesis = v["synthesis"].as_str().unwrap_or("").to_string();
        let question = v["witness_question"].as_str().unwrap_or("").to_string();
        return (synthesis, question);
    }
    // Fallback: return raw text as synthesis
    (raw.to_string(), String::new())
}
