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
        lines.push(format!(
            "\n## Vedic Panchanga (Today's Cosmic Moment)\n{}",
            format_engine_data(p)
        ));
    }
    if let Some(hd) = &ctx.human_design {
        lines.push(format!(
            "\n## Human Design Bodygraph\n{}",
            format_engine_data(hd)
        ));
    }
    if let Some(gk) = &ctx.gene_keys {
        lines.push(format!(
            "\n## Gene Keys Activations\n{}",
            format_engine_data(gk)
        ));
    }
    if let Some(n) = &ctx.numerology {
        lines.push(format!(
            "\n## Numerological Currents\n{}",
            format_engine_data(n)
        ));
    }
    if let Some(b) = &ctx.biorhythm {
        lines.push(format!("\n## Biorhythm Cycles\n{}", format_engine_data(b)));
    }
    if let Some(t) = &ctx.transits {
        lines.push(format!(
            "\n## Planetary Transits\n{}",
            format_engine_data(t)
        ));
    }
    if let Some(v) = &ctx.vimshottari {
        lines.push(format!(
            "\n## Vimshottari Dasha Period\n{}",
            format_engine_data(v)
        ));
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
}
