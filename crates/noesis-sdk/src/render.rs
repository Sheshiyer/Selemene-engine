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

        // Witness prompt
        writeln!(md, "## ✦ Witness Prompt").unwrap();
        writeln!(md).unwrap();
        writeln!(md, "> {}", output.witness_prompt).unwrap();
        writeln!(md).unwrap();

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

        writeln!(text).unwrap();
        writeln!(text, "WITNESS PROMPT:").unwrap();
        writeln!(text, "  {}", output.witness_prompt).unwrap();

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
                engine_version: "0.1.0".into(),
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
}
