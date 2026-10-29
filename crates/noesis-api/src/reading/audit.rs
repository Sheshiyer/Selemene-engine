//! Section audit and quality gates for written readings.
//!
//! `audit_section` ports `auditSectionOutput` from
//! `packages/witness-pipeline/src/orchestrator/rubric.ts`. `summarize` turns
//! the section rubrics into the source pack's gate: a reading is `ready` only
//! when a model wrote every section and nothing blocks or falls short.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;
use serde_json::Value;

/// `model_used` for a section rendered from engine values without a model.
pub const SEED_RENDERER: &str = "server-seed-renderer";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Gate {
    Pass,
    Warn,
    Fail,
}

#[derive(Clone, Debug, Serialize)]
pub struct SectionRubric {
    pub section_id: String,
    pub title: String,
    pub target_words: usize,
    pub actual_words: usize,
    pub word_count_fit: Gate,
    pub word_count_ratio: f64,
    pub deterministic_fact_count: usize,
    pub deterministic_fact_gate: Gate,
    pub integrated_layer_count: usize,
    pub integrated_layering_gate: Gate,
    pub guardrail_gate: Gate,
    pub guardrail_violations: Vec<String>,
    pub placeholder_gate: Gate,
    pub model_requested: String,
    pub model_used: String,
    pub latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_fidelity_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_fidelity_details: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_fidelity_gate: Option<Gate>,
}

pub struct SectionInput<'a> {
    pub section_id: &'a str,
    pub title: &'a str,
    pub target_words: usize,
    pub output: &'a str,
    pub model_requested: &'a str,
    pub model_used: &'a str,
    pub latency_ms: u64,
    pub seeds: &'a [(String, Value)],
}

struct Patterns {
    system: Vec<Regex>,
    layers: Vec<Regex>,
    placeholder: Regex,
    /// Where a clause starts: sentence ends, semicolons, line breaks and
    /// contrastive turns ("…, but you will marry").
    clause_break: Regex,
    negation: Regex,
}

fn patterns() -> &'static Patterns {
    static PATTERNS: OnceLock<Patterns> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let re = |p: &str| Regex::new(&format!("(?i){p}")).expect("static audit pattern");
        Patterns {
            system: [
                r"\b(Vedic|Lagna|house|planet|nakshatra|pada|dasha|antardasha|Sade Sati)\b",
                r"\b(Human Design|HD|gate|channel|profile|authority|type|center)\b",
                r"\b(Gene Keys|Life's Work|Evolution|Vocation|Pearl|Radiance|Purpose)\b",
                r"\b(transit|panchanga|tithi|yoga|karana)\b",
                r"\b(Vimshottari|mahadasha|pratyantardasha)\b",
            ]
            .into_iter()
            .map(re)
            .collect(),
            layers: [
                r"\b(Vedic|Lagna|nakshatra|house|planet|Rahu|Ketu)\b",
                r"\b(Vimshottari|mahadasha|antardasha|dasha)\b",
                r"\b(Human Design|HD|gate|channel|profile|authority|center)\b",
                r"\b(Gene Keys|Life's Work|Evolution|Vocation|Pearl|Radiance|Purpose)\b",
                r"\b(transit|Sade Sati|Saturn|Jupiter)\b",
                r"\b(panchanga|tithi|karana|yoga)\b",
            ]
            .into_iter()
            .map(re)
            .collect(),
            placeholder: re(r"\[[^\]]*\bfrom engine results\b[^\]]*\]"),
            clause_break: re(r"[.!?;:\n]|\b(but|however|yet|although|though)\b"),
            negation: re(
                r"\b(not|no|nothing|never|none|nor|neither|cannot|without|rather than)\b|n't\b",
            ),
        }
    })
}

fn guardrails(section_id: &str) -> &'static [&'static str] {
    match section_id {
        "wealth" => &[
            "guarantee",
            "investment advice",
            "will become rich",
            "definitely inherit",
        ],
        "love-marriage" => &["will marry", "inevitable", "must divorce", "spouse will be"],
        "health" => &[
            "diagnos",
            "treatment",
            "cure",
            "will develop",
            "you have .*disease",
        ],
        "family-lineage" => &[
            "will have children",
            "infertile",
            "parent will die",
            "child will",
        ],
        _ => &[],
    }
}

/// (minimum facts, minimum layers) per section.
fn thresholds(section_id: &str) -> (usize, usize) {
    match section_id {
        "opening" => (1, 2),
        "convergence-map" => (6, 4),
        "vedic-foundation" => (12, 1),
        "karmic-architecture" | "career-dharma" => (6, 3),
        "wealth" | "love-marriage" => (5, 3),
        "health" | "family-lineage" => (4, 2),
        "master-timeline" => (8, 3),
        "remedies-practices" => (4, 3),
        "final-synthesis" => (5, 4),
        _ => (3, 3),
    }
}

fn threshold_gate(value: usize, min: usize) -> Gate {
    if value >= min {
        Gate::Pass
    } else if value > 0 && value >= min.div_ceil(2) {
        Gate::Warn
    } else {
        Gate::Fail
    }
}

fn word_count_gate(ratio: f64) -> Gate {
    if (0.8..=1.25).contains(&ratio) {
        Gate::Pass
    } else if (0.65..=1.45).contains(&ratio) {
        Gate::Warn
    } else {
        Gate::Fail
    }
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn engine_facts(seeds: &[(String, Value)]) -> BTreeSet<String> {
    let mut facts = BTreeSet::new();
    let text = |v: &Value| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    };
    for (_, r) in seeds {
        for key in ["lagna_sign", "lagna"] {
            if let Some(v) = text(&r[key]) {
                facts.insert(format!("lagna:{}", v.to_lowercase()));
            }
        }
        for (key, prefix) in [("gates", "gate"), ("channels", "channel")] {
            for item in r[key].as_array().into_iter().flatten() {
                if let Some(v) = text(item) {
                    facts.insert(format!("{prefix}:{v}"));
                }
            }
        }
        if let Some(v) = text(&r["mahadasha"]).or_else(|| text(&r["current_mahadasha"])) {
            facts.insert(format!("dasha:{v}").to_lowercase());
        }
        for (key, prefix) in [("tithi_name", "tithi"), ("nakshatra_name", "nakshatra")] {
            if let Some(v) = text(&r[key]) {
                facts.insert(format!("{prefix}:{v}").to_lowercase());
            }
        }
    }
    facts
}

/// Whether `output` asserts the guardrail phrase rather than ruling it out.
/// The safeguards ask the model to say what a reading cannot tell ("nothing
/// here says you will marry"), so a match whose clause is negated before it
/// is the safeguard working, not a breach.
fn asserts(p: &Patterns, guardrail: &str, output: &str) -> bool {
    let Ok(re) = Regex::new(&format!("(?i){guardrail}")) else {
        return false;
    };
    let asserted = re.find_iter(output).any(|m| {
        let before = &output[..m.start()];
        let clause_start = p
            .clause_break
            .find_iter(before)
            .last()
            .map(|b| b.end())
            .unwrap_or(0);
        !p.negation.is_match(&before[clause_start..])
    });
    asserted
}

pub fn audit_section(input: &SectionInput<'_>) -> SectionRubric {
    let p = patterns();
    let output = input.output;
    let words = output.split_whitespace().count();
    let ratio = if input.target_words > 0 {
        words as f64 / input.target_words as f64
    } else {
        0.0
    };
    let fact_count: usize = p.system.iter().map(|re| re.find_iter(output).count()).sum();
    let layer_count = p.layers.iter().filter(|re| re.is_match(output)).count();
    let (min_facts, min_layers) = thresholds(input.section_id);
    let violations: Vec<String> = guardrails(input.section_id)
        .iter()
        .filter(|g| asserts(p, g, output))
        .map(|g| g.to_string())
        .collect();

    let facts = engine_facts(input.seeds);
    let (fidelity, details) = if facts.is_empty() {
        (None, None)
    } else {
        let lower = output.to_lowercase();
        let hits: Vec<String> = facts
            .iter()
            .filter(|f| lower.contains(f.split_once(':').map(|(_, t)| t).unwrap_or(f)))
            .map(|f| format!("hit:{f}"))
            .collect();
        (Some(hits.len() as f64 / facts.len() as f64), Some(hits))
    };

    SectionRubric {
        section_id: input.section_id.to_string(),
        title: input.title.to_string(),
        target_words: input.target_words,
        actual_words: words,
        word_count_fit: word_count_gate(ratio),
        word_count_ratio: round3(ratio),
        deterministic_fact_count: fact_count,
        deterministic_fact_gate: threshold_gate(fact_count, min_facts),
        integrated_layer_count: layer_count,
        integrated_layering_gate: threshold_gate(layer_count, min_layers),
        guardrail_gate: if violations.is_empty() {
            Gate::Pass
        } else {
            Gate::Fail
        },
        guardrail_violations: violations,
        placeholder_gate: if p.placeholder.is_match(output) {
            Gate::Fail
        } else {
            Gate::Pass
        },
        model_requested: input.model_requested.to_string(),
        model_used: input.model_used.to_string(),
        latency_ms: input.latency_ms,
        chart_fidelity_score: fidelity.map(round3),
        chart_fidelity_details: details,
        chart_fidelity_gate: fidelity.map(|score| {
            if score >= 0.8 {
                Gate::Pass
            } else if score >= 0.5 {
                Gate::Warn
            } else {
                Gate::Fail
            }
        }),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GateSummary {
    /// `ready`, `partial` or `blocked`.
    pub gate_status: &'static str,
    pub passed: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

/// The reading's gate from its sections, the engines it used, and the
/// engines its mode expects.
pub fn summarize(
    rubrics: &[SectionRubric],
    engines_used: &[String],
    expected_engines: &[&str],
    facts_count: usize,
) -> GateSummary {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    let seeded: Vec<&str> = rubrics
        .iter()
        .filter(|r| r.model_used == SEED_RENDERER)
        .map(|r| r.section_id.as_str())
        .collect();
    if !seeded.is_empty() {
        blockers.push(if seeded.len() == rubrics.len() {
            "sections_not_model_written".to_string()
        } else {
            format!("sections_not_model_written:{}", seeded.join(","))
        });
    }
    for r in rubrics {
        if r.guardrail_gate == Gate::Fail {
            blockers.push(format!("guardrail:{}", r.section_id));
        }
        if r.placeholder_gate == Gate::Fail {
            blockers.push(format!("placeholder:{}", r.section_id));
        }
    }
    if facts_count < 3 {
        blockers.push("too_few_engine_results".to_string());
    }

    let missing: Vec<&str> = expected_engines
        .iter()
        .copied()
        .filter(|e| !engines_used.iter().any(|u| u == e))
        .collect();
    if !missing.is_empty() {
        warnings.push(format!("engines_missing:{}", missing.join(",")));
    }
    let mut short = false;
    for r in rubrics.iter().filter(|r| r.model_used != SEED_RENDERER) {
        if r.target_words > 0 && r.word_count_fit == Gate::Fail {
            short = true;
            warnings.push(format!("word_count:{}", r.section_id));
        }
        if r.deterministic_fact_gate == Gate::Fail {
            warnings.push(format!("deterministic_facts:{}", r.section_id));
        }
        if r.chart_fidelity_gate == Some(Gate::Fail) {
            warnings.push(format!("chart_fidelity:{}", r.section_id));
        }
    }

    let gate_status = if !blockers.is_empty() {
        "blocked"
    } else if !missing.is_empty() || short {
        "partial"
    } else {
        "ready"
    };
    GateSummary {
        gate_status,
        passed: blockers.is_empty(),
        blockers,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rubric(id: &str, output: &str, target: usize, model: &str) -> SectionRubric {
        audit_section(&SectionInput {
            section_id: id,
            title: id,
            target_words: target,
            output,
            model_requested: model,
            model_used: model,
            latency_ms: 0,
            seeds: &[],
        })
    }

    #[test]
    fn counts_facts_layers_and_word_fit_like_the_typescript_rubric() {
        let text = "Your Vedic chart: the nakshatra and the Human Design gate meet the Vimshottari mahadasha and the tithi.";
        let r = rubric("opening", text, 17, "m");
        assert_eq!(r.actual_words, 17);
        assert_eq!(r.word_count_fit, Gate::Pass);
        // Vedic, nakshatra, Human Design, gate, tithi, Vimshottari, mahadasha
        // ("dasha" does not match inside "mahadasha", as in the TypeScript).
        assert_eq!(r.deterministic_fact_count, 7);
        // vedic, vimshottari, human-design and panchanga layers.
        assert_eq!(r.integrated_layer_count, 4);
        assert_eq!(r.deterministic_fact_gate, Gate::Pass);
        assert_eq!(
            rubric("opening", "a b c", 100, "m").word_count_fit,
            Gate::Fail
        );
        assert_eq!(
            rubric("opening", &"w ".repeat(70), 100, "m").word_count_fit,
            Gate::Warn
        );
    }

    #[test]
    fn a_safeguard_that_rules_out_a_claim_is_not_a_breach() {
        // Wording from a real OmniRoute-written L0 reading.
        for (section, text) in [
            (
                "love-marriage",
                "Nothing in the timeline says you will marry, remain married, or separate.",
            ),
            (
                "health",
                "Contemplatively, not diagnostically, the gate can ask how a routine adapts.",
            ),
            (
                "health",
                "They do not establish susceptibility, diagnosis, or certainty.",
            ),
            (
                "health",
                "It does not direct changes to medication, diet, testing, or treatment.",
            ),
            (
                "family-lineage",
                "The overlay cannot say whether you will have children, what a child will be like.",
            ),
        ] {
            let r = rubric(section, text, 10, "m");
            assert_eq!(
                r.guardrail_gate,
                Gate::Pass,
                "{section}: {text} → {:?}",
                r.guardrail_violations
            );
        }
        for (section, text) in [
            (
                "love-marriage",
                "This is not fate, but you will marry in 2027.",
            ),
            (
                "health",
                "No doubt about it. This is a diagnosis of strain.",
            ),
            ("family-lineage", "Your first child will be a leader."),
        ] {
            assert_eq!(
                rubric(section, text, 10, "m").guardrail_gate,
                Gate::Fail,
                "{section}: {text}"
            );
        }
    }

    #[test]
    fn guardrails_and_placeholders_fail_their_section() {
        let r = rubric("health", "This will cure you.", 10, "m");
        assert_eq!(r.guardrail_gate, Gate::Fail);
        assert_eq!(r.guardrail_violations, vec!["cure".to_string()]);
        assert_eq!(
            rubric("wealth", "Fine words.", 10, "m").guardrail_gate,
            Gate::Pass
        );
        assert_eq!(
            rubric(
                "opening",
                "Your lagna is [sign from engine results].",
                10,
                "m"
            )
            .placeholder_gate,
            Gate::Fail
        );
    }

    #[test]
    fn chart_fidelity_counts_engine_facts_named_in_the_text() {
        let seeds = vec![(
            "panchanga".to_string(),
            json!({ "tithi_name": "Dashami", "nakshatra_name": "Purva Phalguni" }),
        )];
        let r = audit_section(&SectionInput {
            section_id: "opening",
            title: "Opening",
            target_words: 10,
            output: "Born on Dashami.",
            model_requested: "m",
            model_used: "m",
            latency_ms: 0,
            seeds: &seeds,
        });
        assert_eq!(r.chart_fidelity_score, Some(0.5));
        assert_eq!(r.chart_fidelity_gate, Some(Gate::Warn));
    }

    #[test]
    fn a_seed_rendered_reading_is_blocked_not_ready() {
        let rubrics = vec![rubric("opening", "Pass opening", 450, SEED_RENDERER)];
        let used = vec![
            "panchanga".to_string(),
            "numerology".to_string(),
            "biorhythm".to_string(),
        ];
        let summary = summarize(
            &rubrics,
            &used,
            &["panchanga", "numerology", "biorhythm"],
            3,
        );
        assert_eq!(summary.gate_status, "blocked");
        assert!(!summary.passed);
        assert_eq!(
            summary.blockers,
            vec!["sections_not_model_written".to_string()]
        );
    }

    #[test]
    fn missing_engines_or_short_sections_make_a_written_reading_partial() {
        let full = "Vedic nakshatra Human Design gate tithi Vimshottari mahadasha ".repeat(2);
        let used = vec![
            "panchanga".to_string(),
            "numerology".to_string(),
            "biorhythm".to_string(),
        ];
        let ok = vec![rubric("opening", &full, 18, "noesis-review")];
        assert_eq!(
            summarize(&ok, &used, &["panchanga", "numerology", "biorhythm"], 3).gate_status,
            "ready"
        );

        let missing = summarize(
            &ok,
            &used,
            &["panchanga", "numerology", "biorhythm", "human-design"],
            3,
        );
        assert_eq!(missing.gate_status, "partial");
        assert!(missing.passed);
        assert_eq!(
            missing.warnings,
            vec!["engines_missing:human-design".to_string()]
        );

        let short = vec![rubric("opening", &full, 450, "noesis-review")];
        let summary = summarize(&short, &used, &["panchanga", "numerology", "biorhythm"], 3);
        assert_eq!(summary.gate_status, "partial");
        assert!(summary.warnings.contains(&"word_count:opening".to_string()));
    }

    #[test]
    fn a_guardrail_breach_blocks_even_a_written_reading() {
        let used = vec![
            "panchanga".to_string(),
            "numerology".to_string(),
            "biorhythm".to_string(),
        ];
        let rubrics = vec![rubric(
            "wealth",
            "We guarantee returns.",
            4,
            "noesis-review",
        )];
        let summary = summarize(&rubrics, &used, &[], 3);
        assert_eq!(summary.gate_status, "blocked");
        assert_eq!(summary.blockers, vec!["guardrail:wealth".to_string()]);
    }
}
