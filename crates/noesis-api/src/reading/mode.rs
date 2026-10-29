//! Mode documents for written readings.
//!
//! The L0 document is embedded from `crates/noesis-api/modes/`, a copy of
//! `packages/witness-pipeline/modes/` (production images build from `crates/`
//! only). A test keeps the two identical.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use yaml_rust2::{Yaml, YamlLoader};

const L0_SOURCE: &str = include_str!("../../modes/integrated-kundali-l0.md");

#[derive(Clone, Debug, PartialEq)]
pub struct PassPlan {
    pub id: String,
    pub title: String,
    pub target_words: usize,
    pub template: String,
}

#[derive(Clone, Debug)]
pub struct ModeDocument {
    pub mode: String,
    pub pass_plan: Vec<PassPlan>,
    /// Engine weights in document order.
    engine_overlay_weights: Vec<(String, f64)>,
    house_overlay: Vec<i64>,
    bridge_mandates: Vec<String>,
    /// (register, pass id) → template name.
    overrides: BTreeMap<(String, String), String>,
    sections: BTreeMap<String, String>,
    lessons: Vec<Lesson>,
}

#[derive(Clone, Debug)]
struct Lesson {
    date: String,
    title: String,
    adopted: Option<String>,
}

fn yaml_str(value: &Yaml) -> Option<String> {
    match value {
        Yaml::String(s) => Some(s.clone()),
        Yaml::Integer(i) => Some(i.to_string()),
        Yaml::Real(r) => Some(r.clone()),
        _ => None,
    }
}

fn yaml_f64(value: &Yaml) -> Option<f64> {
    match value {
        Yaml::Real(r) => r.parse().ok(),
        Yaml::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

/// `## name` sections of the body, in order, with their trimmed content.
fn parse_sections(body: &str) -> BTreeMap<String, String> {
    let mut sections = BTreeMap::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    for line in body.lines() {
        if let Some(name) = line.strip_prefix("## ") {
            if let Some((name, lines)) = current.take() {
                sections.insert(name, lines.join("\n").trim().to_string());
            }
            current = Some((name.trim().to_string(), Vec::new()));
        } else if let Some((_, lines)) = current.as_mut() {
            lines.push(line);
        }
    }
    if let Some((name, lines)) = current {
        sections.insert(name, lines.join("\n").trim().to_string());
    }
    sections
}

/// `### <date> — <title>` entries with an optional `**Adopted:**` line.
fn parse_lessons(section: Option<&String>) -> Vec<Lesson> {
    let mut lessons: Vec<Lesson> = Vec::new();
    for line in section.map(String::as_str).unwrap_or("").lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            let (date, title) = heading.split_once(" — ").unwrap_or(("", heading));
            lessons.push(Lesson {
                date: date.trim().to_string(),
                title: title.trim().to_string(),
                adopted: None,
            });
        } else if let Some(adopted) = line.trim().strip_prefix("**Adopted:**") {
            if let Some(lesson) = lessons.last_mut() {
                lesson.adopted = Some(adopted.trim().to_string());
            }
        }
    }
    lessons
}

impl ModeDocument {
    pub fn parse(source: &str) -> Result<Self, String> {
        let rest = source
            .strip_prefix("---\n")
            .ok_or("mode document must start with front matter")?;
        let (front, body) = rest
            .split_once("\n---\n")
            .ok_or("mode document front matter is not closed")?;
        let docs = YamlLoader::load_from_str(front).map_err(|e| format!("front matter: {e}"))?;
        let fm = docs.first().ok_or("front matter is empty")?;

        let mode = yaml_str(&fm["mode"]).ok_or("front matter has no mode")?;
        let pass_plan = fm["pass_plan"]
            .as_vec()
            .ok_or("front matter has no pass_plan")?
            .iter()
            .map(|pass| {
                Ok(PassPlan {
                    id: yaml_str(&pass["id"]).ok_or("pass without id")?,
                    title: yaml_str(&pass["title"]).ok_or("pass without title")?,
                    target_words: pass["target_words"].as_i64().unwrap_or(0).max(0) as usize,
                    template: yaml_str(&pass["template"]).ok_or("pass without template")?,
                })
            })
            .collect::<Result<Vec<_>, &str>>()?;
        let engine_overlay_weights = fm["engine_overlay_weights"]
            .as_hash()
            .map(|weights| {
                weights
                    .iter()
                    .filter_map(|(k, v)| Some((yaml_str(k)?, yaml_f64(v)?)))
                    .collect()
            })
            .unwrap_or_default();
        let house_overlay = fm["house_overlay"]
            .as_vec()
            .map(|houses| houses.iter().filter_map(Yaml::as_i64).collect())
            .unwrap_or_default();
        let bridge_mandates = fm["bridge_mandates"]
            .as_vec()
            .map(|mandates| mandates.iter().filter_map(yaml_str).collect())
            .unwrap_or_default();
        let mut overrides = BTreeMap::new();
        if let Some(variants) = fm["register_variants"].as_hash() {
            for (register, variant) in variants {
                let Some(register) = yaml_str(register) else {
                    continue;
                };
                for item in variant["overrides"].as_vec().into_iter().flatten() {
                    if let (Some(pass_id), Some(template)) =
                        (yaml_str(&item["pass_id"]), yaml_str(&item["template"]))
                    {
                        overrides.insert((register.clone(), pass_id), template);
                    }
                }
            }
        }
        let sections = parse_sections(body);
        let lessons = parse_lessons(sections.get("lessons"));

        let doc = Self {
            mode,
            pass_plan,
            engine_overlay_weights,
            house_overlay,
            bridge_mandates,
            overrides,
            sections,
            lessons,
        };
        for pass in &doc.pass_plan {
            for register in ["l1_l3", "l4_l5"] {
                doc.template(&pass.id, register)?;
            }
        }
        Ok(doc)
    }

    /// The template for a pass in a register band, honouring overrides.
    pub fn template(&self, pass_id: &str, register: &str) -> Result<&str, String> {
        let pass = self
            .pass_plan
            .iter()
            .find(|p| p.id == pass_id)
            .ok_or_else(|| format!("no pass '{pass_id}' in mode '{}'", self.mode))?;
        let name = self
            .overrides
            .get(&(register.to_string(), pass_id.to_string()))
            .unwrap_or(&pass.template);
        self.sections
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| format!("pass '{pass_id}' uses template '{name}', which has no section"))
    }

    pub fn section(&self, name: &str) -> Option<&str> {
        self.sections.get(name).map(String::as_str)
    }

    pub fn overlay_summary(&self) -> String {
        let weights = self
            .engine_overlay_weights
            .iter()
            .map(|(engine, weight)| format!("{engine}: {weight}"))
            .collect::<Vec<_>>()
            .join(", ");
        let houses = self
            .house_overlay
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        format!("Engine weights: {weights}; Houses: {houses}")
    }

    pub fn bridge_mandates(&self) -> String {
        self.bridge_mandates
            .iter()
            .map(|m| format!("- {m}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn lessons_summary(&self, max_entries: usize) -> String {
        if self.lessons.is_empty() {
            return String::new();
        }
        let start = self.lessons.len().saturating_sub(max_entries);
        let lines = self.lessons[start..]
            .iter()
            .map(|l| {
                let adopted = l
                    .adopted
                    .as_ref()
                    .map(|a| format!(" — Adopted: {a}"))
                    .unwrap_or_default();
                format!("• {} {}{adopted}", l.date, l.title)
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("## Prior Autoresearch Findings\n\n{lines}")
    }
}

/// The integrated L0 kundali mode document, parsed once.
pub fn l0_mode() -> &'static ModeDocument {
    static DOC: OnceLock<ModeDocument> = OnceLock::new();
    DOC.get_or_init(|| ModeDocument::parse(L0_SOURCE).expect("embedded L0 mode document parses"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_l0_document_matches_the_witness_pipeline_copy() {
        let canonical = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../packages/witness-pipeline/modes/integrated-kundali-l0.md"
        ))
        .expect("packages copy is readable");
        assert_eq!(
            L0_SOURCE, canonical,
            "crates/noesis-api/modes/integrated-kundali-l0.md must be a byte copy of packages/witness-pipeline/modes/integrated-kundali-l0.md"
        );
    }

    #[test]
    fn l0_document_parses_with_twelve_passes_and_every_template() {
        let doc = l0_mode();
        assert_eq!(doc.mode, "integrated-kundali-l0");
        assert_eq!(doc.pass_plan.len(), 12);
        assert_eq!(doc.pass_plan[0].id, "opening");
        assert_eq!(doc.pass_plan[2].target_words, 3800);
        assert!(doc
            .template("opening", "l4_l5")
            .unwrap()
            .contains("{{engine_results}}"));
        assert!(doc.section("overlay-rules").is_some());
    }

    #[test]
    fn register_overrides_pick_the_l1_l3_final_synthesis() {
        let doc = l0_mode();
        assert!(doc
            .template("final-synthesis", "l1_l3")
            .unwrap()
            .starts_with("Build the L1-L3 final synthesis"));
        assert!(doc
            .template("final-synthesis", "l4_l5")
            .unwrap()
            .starts_with("Build Part XI"));
    }

    #[test]
    fn overlay_mandates_and_lessons_render_like_the_typescript_parser() {
        let doc = l0_mode();
        assert!(doc
            .overlay_summary()
            .starts_with("Engine weights: panchanga: 1, vimshottari: 1, human-design: 0.9"));
        assert!(doc
            .overlay_summary()
            .ends_with("Houses: 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12"));
        assert!(doc
            .bridge_mandates()
            .starts_with("- Use deterministic chart data first"));
        assert!(doc
            .lessons_summary(5)
            .contains("• 2026-07-04 L0 kundali requires premium interpretation with Witness safeguards — Adopted:"));
    }
}
