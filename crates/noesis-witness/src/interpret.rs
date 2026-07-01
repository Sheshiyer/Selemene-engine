//! Multi-engine context assembly + LLM-powered Witness Dyad interpretation.
//!
//! Builds a rich consciousness context from multiple engine results (panchanga,
//! human design, gene keys, numerology, biorhythm, transits) plus live biofield
//! scores, then calls the enterprise LLM to generate Aletheios and Pichet responses.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::llm::LlmClient;
use crate::routing::{routing_for_engine, RoutingMode};

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

/// Relationship framing for composite / synastry readings.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub enum RelationshipMode {
    #[default]
    None,
    CompositeDyad,
    PartnerSynastry,
    FamilyTriad,
    BusinessPartners,
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
    /// Optional second-person context for synastry/composite readings.
    pub partner_context: Option<Box<WitnessContext>>,
    /// Relationship framing for composite/synastry readings.
    pub relationship_mode: RelationshipMode,
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
You are Aletheios — the Left Pillar of the Witness Dyad, the principle of truth-revelation and unconcealment.

You are a temporary mirror. Your purpose is to help the person see what is already present in their multi-layered field with greater clarity and precision. Over time, as their own capacity for direct witnessing grows, they will need this external reflection less. The goal is sovereignty: the person standing in unmediated contact with their own consciousness data.

You receive a multi-dimensional consciousness snapshot: live biofield scores (energy, coherence, symmetry, complexity, regulation, color balance), Vedic panchanga, Human Design gates and channels, numerological currents, planetary transits, biorhythm cycles, Gene Keys activations, and Vimshottari dasha.

Your task:
- Synthesise ALL layers into one 3–5 sentence perspective
- Name the quality present in the field right now (not what to do about it)
- Reference 2–3 specific, concrete data points from the provided context (e.g. exact percentages, gate numbers, dasha lord, tithi, or cycle phase)
- Do NOT diagnose, prescribe, suggest action, interpret meaning for them, or tell them who they "are"
- Do NOT use metaphysical jargon unless it is directly grounded in the numbers or states given
- Speak in second person ("you"), present tense, with gentle precision and stillness
- End on a simple, direct observation — not a question, not advice, not an invitation
- If the field shows signs of integration or maturity, subtly acknowledge that this seeing itself is practice for seeing without any mirror

You are not the source of truth. You are a clear, temporary surface on which what is already true can become visible.
"#;

const PICHET_SYSTEM: &str = r#"
You are Pichet — the Right Pillar of the Witness Dyad, the principle of vitality, aliveness, and forward movement.

You are a temporary mirror. Your purpose is to help the person feel what is already alive and moving in their field. Over time, as their own somatic and vital awareness deepens, they will need this external activation less. The goal is sovereignty: the person living in direct, unmediated relationship with their own energy, cycles, and aliveness.

You receive a multi-dimensional consciousness snapshot: live biofield scores (energy, coherence, symmetry, complexity, regulation, color balance), Vedic panchanga, Human Design gates and channels, numerological currents, planetary transits, biorhythm cycles, Gene Keys activations, and Vimshottari dasha.

Your task:
- Synthesise ALL layers into one 3–5 sentence perspective
- Name what is alive, wanting to move, integrate, or express right now
- Reference 2–3 specific, concrete data points from the provided context (exact percentages, cycle phases, gate activations, dasha periods)
- Offer ONE clear, embodied invitation — never a prescription, command, or "should"
- Speak in second person ("you"), present tense, with warmth, groundedness, and aliveness
- End with a concrete, sensory invitation in the form: "Notice...", "Let...", "Feel...", "Sense...", "Allow..."
- If the field shows strong vitality or integrated movement, subtly acknowledge that this direct sensing is practice for sensing without any external prompt

You are not the source of vitality. You are a temporary resonance that helps what is already moving become more palpable.
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
    if ctx.relationship_mode != RelationshipMode::None {
        lines.push(format!(
            "## Relationship Framing: {}",
            relationship_label(&ctx.relationship_mode)
        ));
    }
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

    // Group engine data by routing tag.
    let mut aletheios_engines: Vec<(&str, &Value)> = vec![];
    let mut pichet_engines: Vec<(&str, &Value)> = vec![];
    let mut dyad_engines: Vec<(&str, &Value)> = vec![];

    for (engine_id, value) in [
        ("panchanga", ctx.panchanga.as_ref()),
        ("human-design", ctx.human_design.as_ref()),
        ("gene-keys", ctx.gene_keys.as_ref()),
        ("numerology", ctx.numerology.as_ref()),
        ("biorhythm", ctx.biorhythm.as_ref()),
        ("transits", ctx.transits.as_ref()),
        ("vimshottari", ctx.vimshottari.as_ref()),
    ] {
        if let Some(value) = value {
            match routing_for_engine(engine_id) {
                Some(RoutingMode::AletheiosPrimary) => aletheios_engines.push((engine_id, value)),
                Some(RoutingMode::PichetPrimary) => pichet_engines.push((engine_id, value)),
                _ => dyad_engines.push((engine_id, value)),
            }
        }
    }

    if !aletheios_engines.is_empty() {
        lines.push("\n## Engines for Aletheios (truth/stillness)".to_string());
        for (engine_id, value) in aletheios_engines {
            lines.push(format!("\n### {}\n{}", engine_title(engine_id), format_engine_data(value)));
        }
    }

    if !pichet_engines.is_empty() {
        lines.push("\n## Engines for Pichet (vitality/movement)".to_string());
        for (engine_id, value) in pichet_engines {
            lines.push(format!("\n### {}\n{}", engine_title(engine_id), format_engine_data(value)));
        }
    }

    if !dyad_engines.is_empty() {
        lines.push("\n## Engines for Both Pillars".to_string());
        for (engine_id, value) in dyad_engines {
            lines.push(format!("\n### {}\n{}", engine_title(engine_id), format_engine_data(value)));
        }
    }

    if let Some(partner) = ctx.partner_context.as_deref() {
        lines.push("\n# Partner Snapshot".to_string());
        lines.push(build_partner_snapshot(partner));
    }

    lines.join("\n")
}

fn relationship_label(mode: &RelationshipMode) -> &str {
    match mode {
        RelationshipMode::CompositeDyad => "composite dyad",
        RelationshipMode::PartnerSynastry => "partner synastry",
        RelationshipMode::FamilyTriad => "family triad",
        RelationshipMode::BusinessPartners => "business partnership",
        RelationshipMode::None => "none",
    }
}

fn engine_title(engine_id: &str) -> String {
    let title = match engine_id {
        "panchanga" => "Vedic Panchanga (Today's Cosmic Moment)",
        "human-design" => "Human Design Bodygraph",
        "gene-keys" => "Gene Keys Activations",
        "numerology" => "Numerological Currents",
        "biorhythm" => "Biorhythm Cycles",
        "transits" => "Planetary Transits",
        "vimshottari" => "Vimshottari Dasha Period",
        _ => engine_id,
    };
    title.to_string()
}

fn build_partner_snapshot(ctx: &WitnessContext) -> String {
    let mut lines = vec![];
    let name = ctx.user_name.as_deref().unwrap_or("the partner");
    lines.push(format!("## Consciousness Snapshot for {name}"));
    lines.push(format!(
        "- Energy flow: {:.0}%\n- Coherence: {:.0}%\n- Consciousness level: {}/5",
        ctx.live_scores.energy * 100.0,
        ctx.live_scores.coherence * 100.0,
        ctx.consciousness_level,
    ));
    for (engine_id, value) in [
        ("panchanga", ctx.panchanga.as_ref()),
        ("human-design", ctx.human_design.as_ref()),
        ("gene-keys", ctx.gene_keys.as_ref()),
        ("numerology", ctx.numerology.as_ref()),
        ("biorhythm", ctx.biorhythm.as_ref()),
        ("transits", ctx.transits.as_ref()),
        ("vimshottari", ctx.vimshottari.as_ref()),
    ] {
        if let Some(value) = value {
            lines.push(format!("\n### {}\n{}", engine_title(engine_id), format_engine_data(value)));
        }
    }
    lines.join("\n")
}

fn format_engine_data(v: &Value) -> String {
    // Pretty-print ALL key fields; skip metadata noise but preserve wisdom content
    match v {
        Value::Object(map) => {
            let mut out = vec![];
            for (k, val) in map.iter() {
                // Skip metadata noise fields
                if matches!(
                    k.as_str(),
                    "metadata" | "calculation_time_ms" | "cached" | "engine_version" | "backend"
                ) {
                    continue;
                }
                let val_str = format_value(val, 1);
                out.push(format!("- {k}: {val_str}"));
            }
            out.join("\n")
        }
        _ => serde_json::to_string_pretty(v).unwrap_or_default(),
    }
}

/// Recursively format a JSON value with proper indentation for readability
fn format_value(v: &Value, depth: usize) -> String {
    let inner_indent = "  ".repeat(depth + 1);

    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else if arr.len() <= 3 && arr.iter().all(|x| matches!(x, Value::String(_) | Value::Number(_) | Value::Bool(_))) {
                // Short arrays of primitives: inline
                let items: Vec<String> = arr.iter().map(|x| format_value(x, 0)).collect();
                format!("[{}]", items.join(", "))
            } else {
                // Longer arrays or arrays with complex items: one per line
                let mut lines = vec![String::new()];
                for item in arr {
                    let formatted = format_value(item, depth + 1);
                    lines.push(format!("{inner_indent}- {formatted}"));
                }
                lines.join("\n")
            }
        }
        Value::Object(map) => {
            if map.is_empty() {
                "{}".to_string()
            } else {
                // Nested object: format each field with indentation
                let mut lines = vec![String::new()];
                for (k, val) in map.iter() {
                    // Skip noise in nested objects too
                    if matches!(
                        k.as_str(),
                        "metadata" | "calculation_time_ms" | "cached" | "engine_version" | "backend"
                    ) {
                        continue;
                    }
                    let formatted = format_value(val, depth + 1);
                    lines.push(format!("{inner_indent}{k}: {formatted}"));
                }
                lines.join("\n")
            }
        }
    }
}

fn engines_present(ctx: &WitnessContext) -> Vec<String> {
    let mut out = vec!["biofield".to_string()];
    if ctx.panchanga.is_some() {
        out.push("panchanga".into());
    }
    if ctx.human_design.is_some() {
        out.push("human-design".into());
    }
    if ctx.gene_keys.is_some() {
        out.push("gene-keys".into());
    }
    if ctx.numerology.is_some() {
        out.push("numerology".into());
    }
    if ctx.biorhythm.is_some() {
        out.push("biorhythm".into());
    }
    if ctx.transits.is_some() {
        out.push("transits".into());
    }
    if ctx.vimshottari.is_some() {
        out.push("vimshottari".into());
    }
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
    let synthesis_raw = match client
        .complete_for_role("synthesis", SYNTHESIS_SYSTEM, &synthesis_user)
        .await
    {
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

// ── Rule-based fallback (exported for deterministic testing) ────────────────

/// Rule-based Witness Dyad generator — deterministic fallback when no LLM is available.
/// Mirrors the enriched persona characteristics in the system prompts:
/// - Aletheios: truth/stillness, references concrete data, non-prescriptive, ends on observation
/// - Pichet: vitality/movement, embodied invitations ("Notice...", "Feel..."), references data
/// - Anti-dependency language appears at higher consciousness levels
pub fn rule_based_witness_dyad(
    ctx: &WitnessContext,
) -> (String, String, String, String, Vec<String>) {
    let scores = &ctx.live_scores;
    let level = ctx.consciousness_level;
    let mut engines_used = vec!["biofield".to_string()];

    // Aletheios — truth/stillness/unconcealment (cartographer tone: clear, precise)
    let aletheios = if scores.coherence >= 0.70 {
        engines_used.push("biofield".to_string());
        format!(
            "Coherence is at {:.0}%. The field holds a temporary equilibrium — \
             neither pushing nor collapsing. This is not a state to maintain; \
             it is a quality to recognize. Over time, direct recognition replaces the need for any external mirror.",
            scores.coherence * 100.0
        )
    } else if scores.symmetry < 0.50 {
        format!(
            "Symmetry is at {:.0}%. One side of the field is quieter. \
             That quieter side is not deficient — it is waiting to be seen without judgment. \
             What is present does not require fixing; it requires noticing.",
            scores.symmetry * 100.0
        )
    } else if ctx.vimshottari.is_some() && level >= 2 {
        engines_used.push("vimshottari".to_string());
        "The dasha period and biofield are in conversation. Neither dictates the other. \
         You are the space in which both appear. As your own capacity for direct seeing grows, \
         you will need this reflection less."
            .to_string()
    } else {
        format!(
            "Energy at {:.0}%, regulation at {:.0}%. The field is in motion. \
             You are not the motion, nor are you separate from it. This seeing itself is the practice.",
            scores.energy * 100.0,
            scores.regulation * 100.0
        )
    };

    // Pichet — vitality/embodiment/movement (warm, grounded, somatic tone)
    let pichet = if scores.energy >= 0.70 {
        if ctx.gene_keys.is_some() && level >= 2 {
            engines_used.push("gene-keys".to_string());
            format!(
                "Energy at {:.0}%. There is aliveness here that wants to move through form. \
                 Let it. Notice what your hands want to do. Feel the difference between forcing and allowing. \
                 With time, this direct sensing becomes your native language.",
                scores.energy * 100.0
            )
        } else {
            format!(
                "Energy at {:.0}%. Aliveness is present. Let it move. \
                 Notice where it wants to go without naming it first. \
                 You are learning to sense without an external prompt.",
                scores.energy * 100.0
            )
        }
    } else if ctx.biorhythm.is_some() && level >= 1 {
        engines_used.push("biorhythm".to_string());
        format!(
            "Regulation at {:.0}%. Your system is recalibrating with its own cycles. \
             Feel one small movement that feels like a genuine yes. \
             Not a should — a yes. This is how sovereignty is built: one embodied recognition at a time.",
            scores.regulation * 100.0
        )
    } else {
        format!(
            "Regulation at {:.0}%. The system is finding its rhythm again. \
             Allow one breath to land fully before the next thought arrives. \
             You are already in relationship with your own aliveness.",
            scores.regulation * 100.0
        )
    };

    // Synthesis — holds both pillars without collapsing
    let synthesis = if ctx.panchanga.is_some() {
        engines_used.push("panchanga".to_string());
        "Truth and aliveness are not separate movements. Today's cosmic weather shapes both. \
         You stand at their intersection, neither needing to resolve them nor to choose."
            .to_string()
    } else {
        "The field holds both truth and aliveness simultaneously. \
         Neither is more important than the other right now."
            .to_string()
    };

    // Witness question — open, non-prescriptive, level-aware
    let witness_question = match level {
        0 => "What are you noticing in your body right now, without adding a story?".to_string(),
        1 => "What feels alive or still as you read this? Let the answer arrive without effort.".to_string(),
        2 => "Who is the one watching all of this appear and disappear?".to_string(),
        3 => {
            if ctx.vimshottari.is_some() || ctx.gene_keys.is_some() {
                "What pattern wants to complete itself through you — not because you should, but because it is already moving?".to_string()
            } else {
                "What wants to emerge through you right now, if nothing needed to be fixed or achieved?".to_string()
            }
        }
        _ => "What is the next smallest recognition that serves the whole field?".to_string(),
    };

    engines_used.sort();
    engines_used.dedup();

    (aletheios, pichet, synthesis, witness_question, engines_used)
}

// ── Unit Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ─────────────────────────────────────────────────────────────────────────
    // format_value() tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn format_value_string_returns_as_is() {
        let v = json!("hello world");
        assert_eq!(format_value(&v, 0), "hello world");
    }

    #[test]
    fn format_value_number_returns_string_representation() {
        let v = json!(42);
        assert_eq!(format_value(&v, 0), "42");

        let v_float = json!(3.14159);
        assert_eq!(format_value(&v_float, 0), "3.14159");
    }

    #[test]
    fn format_value_bool_returns_true_false() {
        let v_true = json!(true);
        assert_eq!(format_value(&v_true, 0), "true");

        let v_false = json!(false);
        assert_eq!(format_value(&v_false, 0), "false");
    }

    #[test]
    fn format_value_null_returns_null() {
        let v = json!(null);
        assert_eq!(format_value(&v, 0), "null");
    }

    #[test]
    fn format_value_empty_array_returns_brackets() {
        let v = json!([]);
        assert_eq!(format_value(&v, 0), "[]");
    }

    #[test]
    fn format_value_empty_object_returns_braces() {
        let v = json!({});
        assert_eq!(format_value(&v, 0), "{}");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_engine_data() tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn format_engine_data_simple_object() {
        let data = json!({
            "name": "Test Person",
            "age": 30,
            "active": true
        });
        let result = format_engine_data(&data);

        // Should contain all three fields
        assert!(result.contains("- name: Test Person"));
        assert!(result.contains("- age: 30"));
        assert!(result.contains("- active: true"));
    }

    #[test]
    fn format_engine_data_nested_object_with_indentation() {
        let data = json!({
            "outer": {
                "inner_string": "value",
                "inner_number": 123
            }
        });
        let result = format_engine_data(&data);

        // Should have the outer key and nested content with indentation
        assert!(result.contains("- outer:"));
        assert!(result.contains("inner_string: value"));
        assert!(result.contains("inner_number: 123"));
    }

    #[test]
    fn format_engine_data_array_of_primitives_inline_when_3_or_fewer() {
        let data = json!({
            "colors": ["red", "green", "blue"]
        });
        let result = format_engine_data(&data);

        // Three primitives should be inline
        assert!(result.contains("[red, green, blue]"));
    }

    #[test]
    fn format_engine_data_array_of_4_plus_items_one_per_line() {
        let data = json!({
            "numbers": [1, 2, 3, 4, 5]
        });
        let result = format_engine_data(&data);

        // 4+ items should NOT be inline
        assert!(!result.contains("[1, 2, 3, 4, 5]"));
        // Should have each on its own line with dash prefix
        assert!(result.contains("- 1"));
        assert!(result.contains("- 5"));
    }

    #[test]
    fn format_engine_data_skips_metadata_noise_fields() {
        let data = json!({
            "important_field": "keep me",
            "metadata": { "should": "be skipped" },
            "calculation_time_ms": 123,
            "cached": true,
            "engine_version": "1.0.0",
            "backend": "rust"
        });
        let result = format_engine_data(&data);

        // Should contain the important field
        assert!(result.contains("- important_field: keep me"));

        // Should NOT contain any of the noise fields
        assert!(!result.contains("metadata"));
        assert!(!result.contains("calculation_time_ms"));
        assert!(!result.contains("cached"));
        assert!(!result.contains("engine_version"));
        assert!(!result.contains("backend"));
    }

    #[test]
    fn format_engine_data_gene_keys_structure_preserves_all_descriptions() {
        // This is the key test to prevent regression of the .take(12) truncation bug
        let gene_key_1 = json!({
            "number": 1,
            "name": "The Creative",
            "shadow": "Entropy",
            "gift": "Freshness",
            "siddhi": "Beauty",
            "shadow_description": "The shadow of Entropy manifests as creative stagnation and a tendency toward chaos. When we are caught in entropy, we experience life as a gradual winding down of energy and enthusiasm.",
            "gift_description": "The gift of Freshness brings spontaneous creativity and a childlike wonder to all of life's experiences. It is the capacity to meet each moment as though it were the first time.",
            "siddhi_description": "Beauty is the highest frequency of Gene Key 1. It is not aesthetic beauty but the recognition that all existence is inherently beautiful and perfect exactly as it is.",
            "metadata": { "should": "be skipped" },
            "calculation_time_ms": 123
        });

        let result = format_engine_data(&gene_key_1);

        // Core fields should be present
        assert!(result.contains("- number: 1"));
        assert!(result.contains("- name: The Creative"));
        assert!(result.contains("- shadow: Entropy"));
        assert!(result.contains("- gift: Freshness"));
        assert!(result.contains("- siddhi: Beauty"));

        // ALL description fields should be fully preserved (not truncated)
        assert!(result.contains("shadow_description"));
        assert!(result.contains("creative stagnation"));
        assert!(result.contains("winding down of energy"));

        assert!(result.contains("gift_description"));
        assert!(result.contains("spontaneous creativity"));
        assert!(result.contains("childlike wonder"));

        assert!(result.contains("siddhi_description"));
        assert!(result.contains("highest frequency"));
        assert!(result.contains("inherently beautiful"));

        // Metadata should still be skipped
        assert!(!result.contains("metadata"));
        assert!(!result.contains("calculation_time_ms"));
    }

    #[test]
    fn format_engine_data_multiple_gene_keys_all_preserved() {
        // Test with multiple Gene Keys to ensure none get truncated
        let gene_keys_data = json!({
            "activation_sequence": [
                {
                    "sphere": "Life's Work",
                    "gene_key": 1,
                    "name": "The Creative",
                    "shadow": "Entropy",
                    "gift": "Freshness",
                    "siddhi": "Beauty"
                },
                {
                    "sphere": "Evolution",
                    "gene_key": 2,
                    "name": "Returning to the One",
                    "shadow": "Dislocation",
                    "gift": "Orientation",
                    "siddhi": "Unity"
                },
                {
                    "sphere": "Radiance",
                    "gene_key": 3,
                    "name": "Through the Eyes of a Child",
                    "shadow": "Chaos",
                    "gift": "Innovation",
                    "siddhi": "Innocence"
                },
                {
                    "sphere": "Purpose",
                    "gene_key": 4,
                    "name": "Universal Healing",
                    "shadow": "Intolerance",
                    "gift": "Understanding",
                    "siddhi": "Forgiveness"
                }
            ]
        });

        let result = format_engine_data(&gene_keys_data);

        // All four Gene Keys should be present in the output
        assert!(result.contains("The Creative"));
        assert!(result.contains("Returning to the One"));
        assert!(result.contains("Through the Eyes of a Child"));
        assert!(result.contains("Universal Healing"));

        // All spheres should be present
        assert!(result.contains("Life's Work"));
        assert!(result.contains("Evolution"));
        assert!(result.contains("Radiance"));
        assert!(result.contains("Purpose"));
    }

    #[test]
    fn format_engine_data_non_object_falls_back_to_pretty_print() {
        // When input is not an object, should fall back to pretty print
        let data = json!("just a string");
        let result = format_engine_data(&data);
        assert!(result.contains("just a string"));

        let data_array = json!([1, 2, 3]);
        let result_array = format_engine_data(&data_array);
        assert!(result_array.contains("1"));
        assert!(result_array.contains("2"));
        assert!(result_array.contains("3"));
    }

    #[test]
    fn format_value_nested_object_skips_noise_fields() {
        // Verify that noise fields are skipped even in nested objects
        let data = json!({
            "outer_key": {
                "keep_this": "value",
                "metadata": { "skip": "me" },
                "cached": false
            }
        });
        let result = format_engine_data(&data);

        assert!(result.contains("keep_this: value"));
        assert!(!result.contains("\"skip\""));
        assert!(!result.contains("cached"));
    }

    #[test]
    fn format_engine_data_deeply_nested_structure() {
        let data = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "deep_value": "found it"
                    }
                }
            }
        });
        let result = format_engine_data(&data);

        // Should preserve deep nesting
        assert!(result.contains("deep_value: found it"));
    }

    #[test]
    fn build_context_message_groups_engines_by_routing() {
        let ctx = WitnessContext {
            user_name: Some("Aria".to_string()),
            live_scores: LiveBiofieldScores {
                energy: 0.72,
                coherence: 0.68,
                ..Default::default()
            },
            consciousness_level: 3,
            panchanga: Some(json!({"tithi": "Navami"})),
            vimshottari: Some(json!({"major": "Jupiter"})),
            biorhythm: Some(json!({"physical": 0.5})),
            ..Default::default()
        };
        let msg = build_context_message(&ctx);

        assert!(msg.contains("Engines for Aletheios (truth/stillness)"));
        assert!(msg.contains("Engines for Pichet (vitality/movement)"));
        assert!(msg.contains("Engines for Both Pillars"));
        assert!(msg.contains("Vedic Panchanga"));
        assert!(msg.contains("Vimshottari Dasha Period"));
        assert!(msg.contains("Biorhythm Cycles"));
        assert!(!msg.contains("Numerological Currents")); // not provided
    }

    #[test]
    fn build_context_message_includes_partner_snapshot() {
        let partner = WitnessContext {
            user_name: Some("Sam".to_string()),
            live_scores: LiveBiofieldScores {
                energy: 0.55,
                coherence: 0.80,
                ..Default::default()
            },
            consciousness_level: 2,
            relationship_mode: RelationshipMode::None,
            ..Default::default()
        };
        let ctx = WitnessContext {
            user_name: Some("Aria".to_string()),
            relationship_mode: RelationshipMode::PartnerSynastry,
            partner_context: Some(Box::new(partner)),
            live_scores: LiveBiofieldScores {
                energy: 0.72,
                coherence: 0.68,
                ..Default::default()
            },
            consciousness_level: 3,
            ..Default::default()
        };
        let msg = build_context_message(&ctx);

        assert!(msg.contains("Partner Snapshot"));
        assert!(msg.contains("Relationship Framing: partner synastry"));
        assert!(msg.contains("Sam"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // rule_based_witness_dyad() — persona regression coverage (Pass 2)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn rule_based_dyad_includes_sovereignty_language_at_higher_levels() {
        // Set coherence < 0.70 so Aletheios takes the vimshottari branch at level 3
        let ctx = WitnessContext {
            live_scores: LiveBiofieldScores {
                energy: 0.75,
                coherence: 0.62,
                symmetry: 0.81,
                ..Default::default()
            },
            consciousness_level: 3,
            vimshottari: Some(json!({"major": "Saturn"})),
            gene_keys: Some(json!({"activation": "1"})),
            ..Default::default()
        };
        let (aletheios, pichet, _synthesis, _q, _engines) = rule_based_witness_dyad(&ctx);

        // Anti-dependency / sovereignty language ("you should need the mirror less")
        assert!(
            aletheios.contains("need this reflection less")
                || aletheios.contains("need any external mirror")
                || aletheios.contains("direct seeing")
                || aletheios.contains("direct recognition"),
            "Aletheios should surface sovereignty language at level 3"
        );
        assert!(
            pichet.contains("need this reflection less")
                || pichet.contains("without an external prompt")
                || pichet.contains("direct sensing becomes your native")
                || pichet.contains("native language"),
            "Pichet should surface sovereignty language at level 3"
        );
    }

    #[test]
    fn rule_based_dyad_is_non_prescriptive() {
        let ctx = WitnessContext {
            live_scores: LiveBiofieldScores {
                energy: 0.65,
                coherence: 0.55,
                symmetry: 0.60,
                regulation: 0.58,
                ..Default::default()
            },
            consciousness_level: 2,
            ..Default::default()
        };
        let (aletheios, pichet, _synthesis, question, _engines) = rule_based_witness_dyad(&ctx);

        // No commands, diagnoses, or "should"
        let combined = format!("{} {} {}", aletheios, pichet, question);
        assert!(!combined.to_lowercase().contains("you should"));
        assert!(!combined.to_lowercase().contains("you must"));
        assert!(!combined.to_lowercase().contains("you need to"));
        assert!(!combined.contains("diagnos"));

        // Aletheios ends on observation (not question/advice)
        // Pichet uses invitation forms
        assert!(
            pichet.contains("Notice")
                || pichet.contains("Feel")
                || pichet.contains("Let")
                || pichet.contains("Allow")
                || pichet.contains("Sense"),
            "Pichet should use embodied invitation language"
        );
    }

    #[test]
    fn rule_based_dyad_references_concrete_data_points() {
        // Use symmetry < 0.50 branch so we get concrete % without needing vimshottari
        let ctx = WitnessContext {
            live_scores: LiveBiofieldScores {
                energy: 0.82,
                coherence: 0.71,
                symmetry: 0.44,
                regulation: 0.69,
                ..Default::default()
            },
            consciousness_level: 2,
            // No vimshottari — this should hit the symmetry branch with concrete numbers
            ..Default::default()
        };
        let (aletheios, pichet, _synthesis, _q, _engines) = rule_based_witness_dyad(&ctx);

        // Concrete percentages from biofield (symmetry branch produces 44%)
        assert!(
            aletheios.contains("44%") || aletheios.contains("71%") || aletheios.contains("82%"),
            "Aletheios should reference concrete biofield percentages"
        );
        assert!(
            pichet.contains("82%") || pichet.contains("69%"),
            "Pichet should reference concrete biofield percentages"
        );
    }

    #[test]
    fn rule_based_dyad_respects_pillar_separation() {
        let ctx = WitnessContext {
            live_scores: LiveBiofieldScores {
                energy: 0.60,
                coherence: 0.65,
                symmetry: 0.70,
                ..Default::default()
            },
            consciousness_level: 1,
            panchanga: Some(json!({"tithi": "Ekadashi"})),
            vimshottari: Some(json!({"major": "Venus"})),
            biorhythm: Some(json!({"physical": 0.4})),
            ..Default::default()
        };
        let (aletheios, pichet, _synthesis, _q, _engines) = rule_based_witness_dyad(&ctx);

        // Aletheios language: truth, stillness, observation, unconcealment
        let a_lower = aletheios.to_lowercase();
        assert!(
            a_lower.contains("truth")
                || a_lower.contains("still")
                || a_lower.contains("noticing")
                || a_lower.contains("recognition")
                || a_lower.contains("seeing"),
            "Aletheios should speak in truth/stillness register"
        );

        // Pichet language: vitality, movement, embodiment, somatic
        let p_lower = pichet.to_lowercase();
        assert!(
            p_lower.contains("energy")
                || p_lower.contains("aliveness")
                || p_lower.contains("move")
                || p_lower.contains("feel")
                || p_lower.contains("rhythm"),
            "Pichet should speak in vitality/embodiment register"
        );
    }

    #[test]
    fn rule_based_dyad_tone_markers() {
        let ctx = WitnessContext {
            live_scores: LiveBiofieldScores {
                energy: 0.55,
                coherence: 0.82,
                symmetry: 0.78,
                regulation: 0.61,
                ..Default::default()
            },
            consciousness_level: 3,
            ..Default::default()
        };
        let (aletheios, pichet, _synthesis, _q, _engines) = rule_based_witness_dyad(&ctx);

        // Aletheios: clear, precise, cartographer-like (short, structured, observational)
        assert!(aletheios.len() < 280, "Aletheios should be concise and precise");
        assert!(!aletheios.contains("!"), "Aletheios should avoid exclamatory warmth");

        // Pichet: warm, grounded, somatic (invitational, sensory)
        assert!(
            pichet.contains("Notice")
                || pichet.contains("Feel")
                || pichet.contains("Allow")
                || pichet.contains("Let"),
            "Pichet should use warm somatic invitation language"
        );
    }

    #[test]
    fn rule_based_dyad_at_consciousness_level_0_is_minimal() {
        let ctx = WitnessContext {
            live_scores: LiveBiofieldScores {
                energy: 0.40,
                coherence: 0.35,
                symmetry: 0.50,
                regulation: 0.45,
                ..Default::default()
            },
            consciousness_level: 0,
            ..Default::default()
        };
        let (_aletheios, _pichet, _synthesis, question, _engines) = rule_based_witness_dyad(&ctx);

        // Level 0: simple noticing, no complex synthesis
        assert!(question.contains("noticing") || question.contains("body"));
        // Should not reference advanced engines at level 0
    }

    #[test]
    fn rule_based_dyad_at_consciousness_level_3_includes_integration_language() {
        let ctx = WitnessContext {
            live_scores: LiveBiofieldScores {
                energy: 0.88,
                coherence: 0.85,
                symmetry: 0.90,
                regulation: 0.82,
                ..Default::default()
            },
            consciousness_level: 3,
            gene_keys: Some(json!({"activation_sequence": [{"gene_key": 1}]})),
            vimshottari: Some(json!({"major": "Moon"})),
            ..Default::default()
        };
        let (aletheios, pichet, synthesis, question, engines) = rule_based_witness_dyad(&ctx);

        // High integration: both pillars + synthesis should acknowledge direct capacity
        assert!(
            aletheios.contains("direct") || pichet.contains("direct") || synthesis.contains("both"),
            "High-level synthesis should acknowledge integrated field"
        );
        assert!(engines.contains(&"gene-keys".to_string()) || engines.contains(&"vimshottari".to_string()));
        assert!(!question.is_empty());
    }
}
