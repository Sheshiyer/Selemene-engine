//! Report Rendering — Convert engine outputs to Markdown/JSON reports
//!
//! Generates beautifully formatted reports from EngineOutput and WorkflowResult.

use crate::{Error, Result};
use chrono::Utc;
use noesis_core::{EngineOutput, WorkflowResult};
use serde_json::Value;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

/// Output format for reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Markdown,
    Json,
    Text,
}

/// Renderer for engine outputs and workflow results.
pub struct MarkdownRenderer {
    /// Include metadata section in output
    pub include_metadata: bool,
    /// Include consciousness level indicator
    pub include_consciousness_level: bool,
    /// Include timestamp
    pub include_timestamp: bool,
}

impl MarkdownRenderer {
    /// Create a new renderer with default settings.
    pub fn new() -> Self {
        Self {
            include_metadata: true,
            include_consciousness_level: true,
            include_timestamp: true,
        }
    }

    /// Create a minimal renderer (no metadata).
    pub fn minimal() -> Self {
        Self {
            include_metadata: false,
            include_consciousness_level: false,
            include_timestamp: false,
        }
    }

    /// Render an engine output to Markdown.
    pub fn render_engine_output(&self, output: &EngineOutput) -> String {
        let mut md = String::new();

        // Title
        let engine_name = format_engine_name(&output.engine_id);
        writeln!(md, "# {} Reading", engine_name).unwrap();
        writeln!(md).unwrap();

        // Timestamp
        if self.include_timestamp {
            writeln!(
                md,
                "_Generated: {}_",
                output.metadata.timestamp.format("%Y-%m-%d %H:%M UTC")
            )
            .unwrap();
            writeln!(md).unwrap();
        }

        // Result section
        writeln!(md, "## Results").unwrap();
        writeln!(md).unwrap();
        self.render_result_value(&mut md, &output.result, 0);
        writeln!(md).unwrap();

        // Witness prompts: the full contract list when the engine supplied one
        // (GH #1411), otherwise the single prompt.
        let prompts = witness_prompt_texts(output);
        let heading = if prompts.len() > 1 {
            "Witness Prompts"
        } else {
            "Witness Prompt"
        };
        writeln!(md, "## ✦ {}", heading).unwrap();
        writeln!(md).unwrap();
        for prompt in &prompts {
            writeln!(md, "> {}", prompt).unwrap();
            writeln!(md).unwrap();
        }

        // Consciousness level
        if self.include_consciousness_level {
            writeln!(md, "## Consciousness Level").unwrap();
            writeln!(md).unwrap();
            writeln!(
                md,
                "Phase: **{}** {}",
                output.consciousness_level,
                phase_indicator(output.consciousness_level)
            )
            .unwrap();
            writeln!(md).unwrap();
        }

        // Metadata
        if self.include_metadata {
            writeln!(md, "---").unwrap();
            writeln!(md).unwrap();
            writeln!(md, "<details>").unwrap();
            writeln!(md, "<summary>Calculation Metadata</summary>").unwrap();
            writeln!(md).unwrap();
            writeln!(
                md,
                "- **Calculation Time**: {:.2}ms",
                output.metadata.calculation_time_ms
            )
            .unwrap();
            writeln!(md, "- **Backend**: {}", output.metadata.backend).unwrap();
            writeln!(
                md,
                "- **Precision**: {}",
                output.metadata.precision_achieved
            )
            .unwrap();
            writeln!(
                md,
                "- **Cached**: {}",
                if output.metadata.cached { "Yes" } else { "No" }
            )
            .unwrap();
            if !output.metadata.engine_version.is_empty() {
                writeln!(md, "- **Version**: {}", output.metadata.engine_version).unwrap();
            }
            if let Some(provenance) = output.contract_provenance() {
                writeln!(md, "- **Runtime**: {:?}", provenance.runtime_kind).unwrap();
                writeln!(
                    md,
                    "- **Implementation**: {}",
                    provenance.implementation_version
                )
                .unwrap();
                writeln!(md, "- **Contract cached**: {}", provenance.cached).unwrap();
                writeln!(md, "- **Fallback used**: {}", provenance.fallback_used).unwrap();
                if let Some(backend) = provenance.backend_id {
                    writeln!(md, "- **Contract backend**: {backend}").unwrap();
                }
                if let Some(provider) = provenance.provider_id {
                    writeln!(md, "- **Provider**: {provider}").unwrap();
                }
                if let Some(confidence) = provenance.confidence {
                    writeln!(md, "- **Confidence**: {confidence}").unwrap();
                }
            }
            writeln!(md).unwrap();
            writeln!(md, "</details>").unwrap();
        }

        md
    }

    /// Render a workflow result to Markdown.
    pub fn render_workflow_result(&self, result: &WorkflowResult) -> String {
        let mut md = String::new();

        // Title
        let workflow_name = format_workflow_name(&result.workflow_id);
        writeln!(md, "# {} Workflow", workflow_name).unwrap();
        writeln!(md).unwrap();

        // Timestamp
        if self.include_timestamp {
            writeln!(
                md,
                "_Generated: {}_",
                result.timestamp.format("%Y-%m-%d %H:%M UTC")
            )
            .unwrap();
            writeln!(md).unwrap();
        }

        // Each engine output
        for (engine_id, output) in &result.engine_outputs {
            let engine_name = format_engine_name(engine_id);
            writeln!(md, "## {}", engine_name).unwrap();
            writeln!(md).unwrap();
            self.render_result_value(&mut md, &output.result, 0);
            writeln!(md).unwrap();
            writeln!(md, "> **Witness**: {}", output.witness_prompt).unwrap();
            writeln!(md).unwrap();
        }

        // Synthesis (if present)
        if let Some(ref synthesis) = result.synthesis {
            writeln!(md, "## ✦ Synthesis").unwrap();
            writeln!(md).unwrap();
            self.render_result_value(&mut md, synthesis, 0);
            writeln!(md).unwrap();
        }

        // Timing
        if self.include_metadata {
            writeln!(md, "---").unwrap();
            writeln!(md).unwrap();
            writeln!(md, "_Total execution time: {:.2}ms_", result.total_time_ms).unwrap();
        }

        md
    }

    /// Render a JSON value to Markdown recursively.
    fn render_result_value(&self, md: &mut String, value: &Value, depth: usize) {
        match value {
            Value::Object(obj) => {
                for (key, val) in obj {
                    if depth == 0
                        && (key == noesis_core::CONTRACT_PROVENANCE_KEY
                            || key == noesis_core::CONTRACT_WITNESS_PROMPTS_KEY)
                    {
                        continue;
                    }
                    let formatted_key = format_key(key);
                    match val {
                        Value::Object(_) | Value::Array(_) => {
                            writeln!(md, "{}**{}**:", indent(depth), formatted_key).unwrap();
                            self.render_result_value(md, val, depth + 1);
                        }
                        _ => {
                            writeln!(
                                md,
                                "{}- **{}**: {}",
                                indent(depth),
                                formatted_key,
                                format_value(val)
                            )
                            .unwrap();
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    if let Value::Object(_) = item {
                        self.render_result_value(md, item, depth);
                        writeln!(md).unwrap();
                    } else {
                        writeln!(md, "{}- {}", indent(depth), format_value(item)).unwrap();
                    }
                }
            }
            _ => {
                writeln!(md, "{}{}", indent(depth), format_value(value)).unwrap();
            }
        }
    }

    /// Render to a specific format.
    pub fn render(&self, output: &EngineOutput, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Markdown => Ok(self.render_engine_output(output)),
            ReportFormat::Json => {
                serde_json::to_string_pretty(output).map_err(|e| Error::Render(e.to_string()))
            }
            ReportFormat::Text => Ok(self.render_plain_text(output)),
        }
    }

    /// Render workflow to a specific format.
    pub fn render_workflow(&self, result: &WorkflowResult, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Markdown => Ok(self.render_workflow_result(result)),
            ReportFormat::Json => {
                serde_json::to_string_pretty(result).map_err(|e| Error::Render(e.to_string()))
            }
            ReportFormat::Text => Ok(self.render_workflow_plain_text(result)),
        }
    }

    /// Render engine output as plain text.
    fn render_plain_text(&self, output: &EngineOutput) -> String {
        let mut text = String::new();

        writeln!(
            text,
            "{} READING",
            format_engine_name(&output.engine_id).to_uppercase()
        )
        .unwrap();
        writeln!(text, "{}", "=".repeat(40)).unwrap();
        writeln!(text).unwrap();

        self.render_value_plain(&mut text, &output.result, 0);

        if self.include_metadata {
            if let Some(provenance) = output.contract_provenance() {
                writeln!(
                    text,
                    "Contract provenance: {}",
                    serde_json::to_string(&provenance).unwrap()
                )
                .unwrap();
            }
        }

        writeln!(text).unwrap();
        let prompts = witness_prompt_texts(output);
        let heading = if prompts.len() > 1 {
            "WITNESS PROMPTS:"
        } else {
            "WITNESS PROMPT:"
        };
        writeln!(text, "{}", heading).unwrap();
        for prompt in &prompts {
            writeln!(text, "  {}", prompt).unwrap();
        }

        text
    }

    /// Render workflow result as plain text.
    fn render_workflow_plain_text(&self, result: &WorkflowResult) -> String {
        let mut text = String::new();

        writeln!(
            text,
            "{} WORKFLOW",
            format_workflow_name(&result.workflow_id).to_uppercase()
        )
        .unwrap();
        writeln!(text, "{}", "=".repeat(40)).unwrap();
        writeln!(text).unwrap();

        for (engine_id, output) in &result.engine_outputs {
            writeln!(text, "--- {} ---", format_engine_name(engine_id)).unwrap();
            self.render_value_plain(&mut text, &output.result, 0);
            writeln!(text, "  Witness: {}", output.witness_prompt).unwrap();
            writeln!(text).unwrap();
        }

        text
    }

    fn render_value_plain(&self, text: &mut String, value: &Value, depth: usize) {
        match value {
            Value::Object(obj) => {
                for (key, val) in obj {
                    if depth == 0
                        && (key == noesis_core::CONTRACT_PROVENANCE_KEY
                            || key == noesis_core::CONTRACT_WITNESS_PROMPTS_KEY)
                    {
                        continue;
                    }
                    match val {
                        Value::Object(_) | Value::Array(_) => {
                            writeln!(text, "{}{}:", "  ".repeat(depth), format_key(key)).unwrap();
                            self.render_value_plain(text, val, depth + 1);
                        }
                        _ => {
                            writeln!(
                                text,
                                "{}{}: {}",
                                "  ".repeat(depth),
                                format_key(key),
                                format_value(val)
                            )
                            .unwrap();
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.render_value_plain(text, item, depth);
                }
            }
            _ => {
                writeln!(text, "{}{}", "  ".repeat(depth), format_value(value)).unwrap();
            }
        }
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Witness prompt texts to show: the contract list when present, else the
/// single `witness_prompt`.
fn witness_prompt_texts(output: &EngineOutput) -> Vec<String> {
    match output.contract_witness_prompts() {
        Some(prompts) if !prompts.is_empty() => prompts.into_iter().map(|p| p.prompt).collect(),
        _ => vec![output.witness_prompt.clone()],
    }
}

/// Save a rendered report to a file.
pub fn save_report(content: &str, path: impl AsRef<Path>) -> Result<()> {
    fs::write(path.as_ref(), content)?;
    Ok(())
}

/// Generate a filename for a report.
pub fn report_filename(engine_id: &str, format: ReportFormat) -> String {
    let now = Utc::now();
    let ext = match format {
        ReportFormat::Markdown => "md",
        ReportFormat::Json => "json",
        ReportFormat::Text => "txt",
    };
    format!("{}_{}.{}", engine_id, now.format("%Y%m%d_%H%M%S"), ext)
}

/// Get the reports directory path.
pub fn reports_dir() -> Result<std::path::PathBuf> {
    let dir = dirs::home_dir()
        .ok_or_else(|| Error::Render("Could not determine home directory".into()))?
        .join("noesis-reports");

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

// Helper functions

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

fn format_key(key: &str) -> String {
    key.replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => if *b { "Yes" } else { "No" }.into(),
        Value::Null => "—".into(),
        _ => value.to_string(),
    }
}

fn format_engine_name(engine_id: &str) -> String {
    engine_id
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_workflow_name(workflow_id: &str) -> String {
    format_engine_name(workflow_id)
}

fn phase_indicator(level: u8) -> &'static str {
    match level {
        0 => "○○○○○",
        1 => "●○○○○",
        2 => "●●○○○",
        3 => "●●●○○",
        4 => "●●●●○",
        5 => "●●●●●",
        _ => "?????",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use noesis_core::CalculationMetadata;
    use serde_json::json;

    fn test_output() -> EngineOutput {
        EngineOutput {
            engine_id: "numerology".into(),
            result: json!({
                "life_path": {
                    "value": 5,
                    "meaning": "Freedom, change, adventure"
                },
                "expression": {
                    "value": 1,
                    "meaning": "Leadership, independence"
                }
            }),
            witness_prompt: "What patterns arise when freedom meets discipline?".into(),
            consciousness_level: 2,
            metadata: CalculationMetadata {
                calculation_time_ms: 1.23,
                backend: "native".into(),
                precision_achieved: "Standard".into(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: "3.0.0".into(),
                fallback_used: false,
                provider_id: None,
            },
        }
    }

    #[test]
    fn test_render_markdown() {
        let output = test_output();
        let renderer = MarkdownRenderer::new();
        let md = renderer.render_engine_output(&output);

        assert!(md.contains("# Numerology Reading"));
        assert!(md.contains("Life Path"));
        assert!(md.contains("Freedom, change, adventure"));
        assert!(md.contains("Witness Prompt"));
        assert!(md.contains("What patterns arise"));
        assert!(md.contains("Phase: **2**"));
    }

    #[test]
    fn test_render_minimal() {
        let output = test_output();
        let renderer = MarkdownRenderer::minimal();
        let md = renderer.render_engine_output(&output);

        assert!(md.contains("# Numerology Reading"));
        assert!(!md.contains("Calculation Metadata"));
        assert!(!md.contains("Consciousness Level"));
    }

    #[test]
    fn test_render_json() {
        let output = test_output();
        let renderer = MarkdownRenderer::new();
        let json = renderer.render(&output, ReportFormat::Json).unwrap();

        assert!(json.contains("\"engine_id\": \"numerology\""));
        assert!(json.contains("\"life_path\""));
    }

    #[test]
    fn test_render_text() {
        let output = test_output();
        let renderer = MarkdownRenderer::new();
        let text = renderer.render(&output, ReportFormat::Text).unwrap();

        assert!(text.contains("NUMEROLOGY READING"));
        assert!(text.contains("Life Path"));
        assert!(text.contains("WITNESS PROMPT"));
    }

    #[test]
    fn test_format_key() {
        assert_eq!(format_key("life_path"), "Life Path");
        assert_eq!(format_key("consciousness_level"), "Consciousness Level");
        assert_eq!(format_key("simple"), "Simple");
    }

    #[test]
    fn test_format_engine_name() {
        assert_eq!(format_engine_name("human-design"), "Human Design");
        assert_eq!(format_engine_name("gene-keys"), "Gene Keys");
        assert_eq!(format_engine_name("numerology"), "Numerology");
    }

    #[test]
    fn test_phase_indicator() {
        assert_eq!(phase_indicator(0), "○○○○○");
        assert_eq!(phase_indicator(3), "●●●○○");
        assert_eq!(phase_indicator(5), "●●●●●");
    }

    #[test]
    fn test_report_filename() {
        let filename = report_filename("numerology", ReportFormat::Markdown);
        assert!(filename.starts_with("numerology_"));
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn sacred_geometry_tui_render_keeps_provenance_detail() {
        // GH #1401: the TUI result screen and its exports go through this
        // renderer. Feed it the sacred-geometry API envelope as the SDK
        // deserializes it and check the per-part labels stay visible.
        let envelope: Value = serde_json::from_str(include_str!(
            "../../../ts-engines/tests/fixtures/sacred-geometry/api-envelope-seed-1401.json"
        ))
        .unwrap();
        let output: EngineOutput = serde_json::from_value(envelope.clone()).unwrap();
        let renderer = MarkdownRenderer::new();

        let markdown = renderer.render(&output, ReportFormat::Markdown).unwrap();
        for label in [
            "symbolic_lookup",
            "seeded_random",
            "not_implemented",
            "caller_template_validation",
            "rejected",
        ] {
            assert!(markdown.contains(label), "markdown lost {label}");
        }

        let text = renderer.render(&output, ReportFormat::Text).unwrap();
        assert!(text.contains("not_implemented"), "text export lost labels");

        let json: Value =
            serde_json::from_str(&renderer.render(&output, ReportFormat::Json).unwrap()).unwrap();
        assert_eq!(
            json["result"]["provenance_detail"],
            envelope["result"]["provenance_detail"]
        );
        assert_eq!(json["provenance"], envelope["provenance"]);
        assert!(json["result"]
            .get(noesis_core::CONTRACT_PROVENANCE_KEY)
            .is_none());
        assert!(markdown.contains("**Runtime**: TypeScript"));
        assert!(markdown.contains("**Implementation**: 1.0.0"));
        assert!(text.contains("implementation_version"));
        assert!(!markdown.contains(noesis_core::CONTRACT_PROVENANCE_KEY));

        // GH #1411: every witness prompt reaches the screen and the exports,
        // the derivation labels stay visible, and the scaffolding key does not.
        let prompts = envelope["witness_prompts"].as_array().unwrap();
        assert_eq!(prompts.len(), 3);
        assert!(markdown.contains("## ✦ Witness Prompts"));
        for prompt in prompts {
            let prompt = prompt["prompt"].as_str().unwrap();
            assert!(
                markdown.contains(&format!("> {prompt}")),
                "markdown lost {prompt}"
            );
            assert!(text.contains(prompt), "text export lost {prompt}");
        }
        assert!(text.contains("WITNESS PROMPTS:"));
        for label in ["templated_narrative", "guidance", "static_constant"] {
            assert!(markdown.contains(label), "markdown lost {label}");
        }
        assert_eq!(json["witness_prompts"], envelope["witness_prompts"]);
        assert!(!markdown.contains(noesis_core::CONTRACT_WITNESS_PROMPTS_KEY));
        assert!(!text.contains(noesis_core::CONTRACT_WITNESS_PROMPTS_KEY));
    }

    fn i_ching_output() -> (Value, EngineOutput) {
        // #1341: the real API envelope (pinned by noesis-api), deserialized as the
        // SDK does, so the contract block is held under the reserved key.
        let envelope: Value = serde_json::from_str(include_str!(
            "../../../ts-engines/tests/fixtures/i-ching/api-envelope-seed-42.json"
        ))
        .unwrap();
        let output: EngineOutput = serde_json::from_value(envelope.clone()).unwrap();
        (envelope, output)
    }

    #[test]
    fn i_ching_reports_keep_cast_and_derived_labels() {
        let (envelope, output) = i_ching_output();
        let renderer = MarkdownRenderer::new();

        let markdown = renderer.render(&output, ReportFormat::Markdown).unwrap();
        let text = renderer.render(&output, ReportFormat::Text).unwrap();
        for label in ["seeded_three_coin_simulation", "derived", "unavailable"] {
            assert!(markdown.contains(label), "markdown lost {label}");
            assert!(text.contains(label), "text lost {label}");
        }
        assert!(markdown.contains("**Implementation**: 1.1.0"));
        assert!(markdown.contains("**Fallback used**: false"));
        assert!(!markdown.contains(noesis_core::CONTRACT_PROVENANCE_KEY));
        assert!(!text.contains(noesis_core::CONTRACT_PROVENANCE_KEY));

        let json: Value =
            serde_json::from_str(&renderer.render(&output, ReportFormat::Json).unwrap()).unwrap();
        assert_eq!(json["result"], envelope["result"]);
        assert_eq!(json["provenance"], envelope["provenance"]);
    }

    #[test]
    fn i_ching_workflow_reports_keep_derivation_and_provenance() {
        let (envelope, output) = i_ching_output();
        let workflow = WorkflowResult {
            workflow_id: "decision-support".into(),
            engine_outputs: std::collections::HashMap::from([("i-ching".to_string(), output)]),
            synthesis: None,
            total_time_ms: 0.0,
            timestamp: Utc::now(),
        };
        let renderer = MarkdownRenderer::new();

        for format in [ReportFormat::Markdown, ReportFormat::Text] {
            let report = renderer.render_workflow(&workflow, format).unwrap();
            assert!(report.contains("seeded_three_coin_simulation"));
            assert!(!report.contains(noesis_core::CONTRACT_PROVENANCE_KEY));
        }
        let json: Value = serde_json::from_str(
            &renderer
                .render_workflow(&workflow, ReportFormat::Json)
                .unwrap(),
        )
        .unwrap();
        let exported = &json["engine_outputs"]["i-ching"];
        assert_eq!(exported["result"], envelope["result"]);
        assert_eq!(exported["provenance"], envelope["provenance"]);
    }
}
