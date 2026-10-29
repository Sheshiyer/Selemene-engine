//! Prompts for one pass, as `packages/witness-pipeline` builds them
//! (`orchestrator/integrated.ts` and `orchestrator/engine-formatter.ts`).

use serde_json::Value;

use super::mode::{ModeDocument, PassPlan};

/// What a pass prompt is built from.
pub struct PassContext<'a> {
    pub subject_names: &'a [String],
    pub seeds: &'a [(String, Value)],
    /// The tail of the reading so far.
    pub prior: &'a str,
    pub register: &'a str,
    pub language: Option<&'a str>,
}

fn result_of<'a>(seeds: &'a [(String, Value)], id: &str) -> Option<&'a Value> {
    seeds
        .iter()
        .find(|(engine, _)| engine == id)
        .map(|(_, v)| v)
}

fn text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn named_list(value: &Value, keys: &[&str]) -> Option<String> {
    let items: Vec<String> = value
        .as_array()?
        .iter()
        .filter_map(|item| text(item).or_else(|| keys.iter().find_map(|k| text(&item[*k]))))
        .collect();
    (!items.is_empty()).then(|| items.join(", "))
}

fn key_facts(seeds: &[(String, Value)]) -> String {
    let null = Value::Null;
    let get = |id: &str| result_of(seeds, id).unwrap_or(&null);
    let (panchanga, vimshottari, human_design, gene_keys, biofield) = (
        get("panchanga"),
        get("vimshottari"),
        get("human-design"),
        get("gene-keys"),
        get("biofield"),
    );
    let mut facts = Vec::new();
    for (label, key) in [
        ("nakshatra", "nakshatra_name"),
        ("tithi", "tithi_name"),
        ("vara", "vara_name"),
        ("yoga", "yoga_name"),
    ] {
        if let Some(v) = text(&panchanga[key]) {
            facts.push(format!("{label} {v}"));
        }
    }
    for (label, level) in [("dasha", "mahadasha"), ("antardasha", "antardasha")] {
        if let Some(planet) = text(&vimshottari["current_period"][level]["planet"]) {
            facts.push(format!("{label} {planet}"));
        }
    }
    for (label, key) in [
        ("HD type", "hd_type"),
        ("profile", "profile"),
        ("authority", "authority"),
    ] {
        if let Some(v) = text(&human_design[key]) {
            facts.push(format!("{label} {v}"));
        }
    }
    if let Some(channels) = named_list(&human_design["active_channels"], &["channel", "name"]) {
        facts.push(format!("active channels {channels}"));
    }
    if let Some(keys) = named_list(&gene_keys["active_keys"], &["key", "name"]) {
        facts.push(format!("gene keys {keys}"));
    }
    if let Some(areas) = named_list(&biofield["areas_of_attention"], &["area", "name"]) {
        facts.push(format!("biofield attention {areas}"));
    }
    if facts.is_empty() {
        "No key facts extracted.".to_string()
    } else {
        format!("{}.", facts.join("; "))
    }
}

/// The deterministic engine results block every pass template embeds.
pub fn format_engine_results(seeds: &[(String, Value)]) -> String {
    let blocks = seeds
        .iter()
        .map(|(id, result)| {
            let json = serde_json::to_string_pretty(result).unwrap_or_default();
            format!("### {id}\n\n```json\n{json}\n```")
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    format!(
        "## Deterministic Engine Results\n\n### Key facts\n\n{}\n\n{blocks}",
        key_facts(seeds)
    )
}

/// Acceptable length range for a pass: 90–100% of its target.
pub fn target_range(pass: &PassPlan) -> (usize, usize) {
    (
        ((pass.target_words as f64) * 0.9).round() as usize,
        pass.target_words,
    )
}

/// Output budget for a pass, as the TypeScript orchestrator sets it.
pub fn max_tokens(pass: &PassPlan) -> u32 {
    (target_range(pass).1 * 2) as u32
}

pub fn render_user_prompt(
    doc: &ModeDocument,
    pass: &PassPlan,
    template: &str,
    ctx: &PassContext<'_>,
) -> String {
    let names = ctx.subject_names.join(", ");
    // Engine results go in last so `{{…}}` text inside the JSON is never expanded.
    let mut prompt = template
        .replace("{{subject_names}}", &names)
        .replace("{{subject_roles}}", &names)
        .replace("{{relationship_header}}", "")
        .replace("{{relationship_context}}", "")
        .replace("{{mapping_goal}}", "")
        .replace("{{prior_pass}}", ctx.prior)
        .replace("{{overlay_summary}}", &doc.overlay_summary())
        .replace("{{bridge_mandates}}", &doc.bridge_mandates())
        .replace("{{lessons_summary}}", &doc.lessons_summary(5))
        .replace("{{register}}", ctx.register)
        .replace("{{pass_id}}", &pass.id)
        .replace("{{target_words}}", &pass.target_words.to_string())
        .replace("{{language}}", ctx.language.unwrap_or("en"));
    prompt = prompt.replace("{{engine_results}}", &format_engine_results(ctx.seeds));
    prompt
}

pub fn render_system_prompt(doc: &ModeDocument, pass: &PassPlan, ctx: &PassContext<'_>) -> String {
    let (min, max) = target_range(pass);
    let language = ctx
        .language
        .map(|l| format!("Language: {l}."))
        .unwrap_or_default();
    format!(
        "You are writing pass \"{title}\" (id: {id}) for the {mode} reading mode.\n\
         Register band: {register}.\n\
         Target length: ~{target} words (acceptable range {min}-{max}).\n\
         Subjects: {names}\n\
         \n\
         {language}\n\
         {rules}",
        title = pass.title,
        id = pass.id,
        mode = doc.mode,
        register = ctx.register,
        target = pass.target_words,
        names = ctx.subject_names.join(", "),
        rules = doc.section("overlay-rules").unwrap_or(""),
    )
}

/// The last `max_chars` characters of the reading so far.
pub fn prior_tail(assembled: &str, max_chars: usize) -> &str {
    let count = assembled.chars().count();
    if count <= max_chars {
        return assembled;
    }
    let start = assembled
        .char_indices()
        .nth(count - max_chars)
        .map(|(i, _)| i)
        .unwrap_or(0);
    &assembled[start..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reading::mode::l0_mode;
    use serde_json::json;

    fn seeds() -> Vec<(String, Value)> {
        vec![
            (
                "panchanga".into(),
                json!({ "tithi_name": "Shukla Dashami", "nakshatra_name": "Purva Phalguni" }),
            ),
            (
                "human-design".into(),
                json!({ "hd_type": "Generator", "profile": "5/1", "active_channels": ["5-15", "3-60"] }),
            ),
            (
                "vimshottari".into(),
                json!({ "current_period": { "mahadasha": { "planet": "Rahu" }, "antardasha": { "planet": "Jupiter" } } }),
            ),
            (
                "numerology".into(),
                json!({ "note": "{{prior_pass}} $1 stays literal" }),
            ),
        ]
    }

    #[test]
    fn engine_results_lead_with_key_facts_then_pretty_json() {
        let block = format_engine_results(&seeds());
        assert!(block.starts_with("## Deterministic Engine Results\n\n### Key facts\n\n"));
        assert!(block.contains(
            "nakshatra Purva Phalguni; tithi Shukla Dashami; dasha Rahu; antardasha Jupiter; HD type Generator; profile 5/1; active channels 5-15, 3-60."
        ));
        assert!(block.contains("### human-design\n\n```json\n{\n  \"active_channels\""));
        assert!(format_engine_results(&[]).contains("No key facts extracted."));
    }

    #[test]
    fn user_prompt_fills_every_slot_and_keeps_engine_text_literal() {
        let doc = l0_mode();
        let pass = &doc.pass_plan[0];
        let names = vec!["Test Seeker".to_string()];
        let seeds = seeds();
        let ctx = PassContext {
            subject_names: &names,
            seeds: &seeds,
            prior: "earlier text",
            register: "l4_l5",
            language: None,
        };
        let prompt = render_user_prompt(doc, pass, doc.template(&pass.id, "l4_l5").unwrap(), &ctx);
        assert!(prompt.contains("Test Seeker's premium integrated kundali report"));
        assert!(prompt.contains("Prior pass context:\nearlier text"));
        assert!(prompt.contains("Target ~450 words. Pass: opening."));
        assert!(
            prompt.contains("{{prior_pass}} $1 stays literal"),
            "engine JSON must not be expanded"
        );
        assert!(!prompt.contains("{{engine_results}}") && !prompt.contains("{{register}}"));
    }

    #[test]
    fn system_prompt_states_range_and_rules() {
        let doc = l0_mode();
        let pass = &doc.pass_plan[2];
        let names = vec!["Test Seeker".to_string()];
        let ctx = PassContext {
            subject_names: &names,
            seeds: &[],
            prior: "",
            register: "l1_l3",
            language: Some("hi"),
        };
        let system = render_system_prompt(doc, pass, &ctx);
        assert!(system.contains("Target length: ~3800 words (acceptable range 3420-3800)."));
        assert!(system.contains("Language: hi."));
        assert!(system.contains("Grounding rule:"));
        assert_eq!(max_tokens(pass), 7600);
    }

    #[test]
    fn prior_tail_is_char_safe() {
        assert_eq!(prior_tail("abc", 10), "abc");
        assert_eq!(prior_tail("ॐabcdef", 3), "def");
        assert_eq!(prior_tail("abॐ", 1), "ॐ");
    }
}
