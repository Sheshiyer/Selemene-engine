//! Workflow data structures for multi-engine synthesis
//!
//! Defines the core types used across all workflows:
//! - WorkflowOutput: Combined results from workflow execution
//! - SynthesisResult: Cross-engine pattern analysis
//! - Theme, Alignment, Tension: Pattern matching types

use chrono::{DateTime, Utc};
use noesis_core::EngineOutput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete output from workflow execution including synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOutput {
    /// Which workflow was executed
    pub workflow_id: String,
    /// Individual engine results keyed by engine_id
    pub engine_results: HashMap<String, EngineOutput>,
    /// Cross-engine synthesis
    pub synthesis: SynthesisResult,
    /// Self-inquiry prompts at the synthesis level
    pub witness_prompts: Vec<WitnessPrompt>,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    /// When the workflow was executed
    pub timestamp: DateTime<Utc>,
}

/// Cross-engine pattern analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisResult {
    /// Cross-engine patterns identified
    pub themes: Vec<Theme>,
    /// Where engines agree
    pub alignments: Vec<Alignment>,
    /// Where engines differ or create tension
    pub tensions: Vec<Tension>,
    /// Human-readable summary
    pub summary: String,
}

impl Default for SynthesisResult {
    fn default() -> Self {
        Self {
            themes: Vec::new(),
            alignments: Vec::new(),
            tensions: Vec::new(),
            summary: String::new(),
        }
    }
}

/// A cross-engine theme or pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name (e.g., "Leadership", "Introspection", "Transformation")
    pub name: String,
    /// Description of how this theme manifests
    pub description: String,
    /// Engine IDs that contribute to this theme
    pub sources: Vec<String>,
    /// How strongly supported (0.0-1.0, based on how many engines)
    pub strength: f32,
}

impl Theme {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            sources: Vec::new(),
            strength: 0.0,
        }
    }

    pub fn with_sources(mut self, sources: Vec<String>) -> Self {
        let count = sources.len();
        self.sources = sources;
        // Strength based on number of sources (normalized to max of 5 engines)
        self.strength = (count as f32 / 5.0).min(1.0);
        self
    }

    pub fn add_source(&mut self, source: impl Into<String>) {
        self.sources.push(source.into());
        self.strength = (self.sources.len() as f32 / 5.0).min(1.0);
    }
}

/// Where multiple engines agree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alignment {
    /// What the engines agree on
    pub aspect: String,
    /// Human-readable description
    pub description: String,
    /// Engines that agree
    pub engines: Vec<String>,
    /// Confidence in the alignment (0.0-1.0)
    pub confidence: f32,
}

impl Alignment {
    pub fn new(aspect: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            aspect: aspect.into(),
            description: description.into(),
            engines: Vec::new(),
            confidence: 0.0,
        }
    }

    pub fn with_engines(mut self, engines: Vec<String>) -> Self {
        self.engines = engines;
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}

/// Where engines differ or create productive tension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tension {
    /// The tension aspect
    pub aspect: String,
    /// Description of the tension
    pub description: String,
    /// First perspective (engine_id, view)
    pub perspective_a: (String, String),
    /// Second perspective (engine_id, view)
    pub perspective_b: (String, String),
    /// Suggested integration or resolution
    pub integration_hint: String,
}

impl Tension {
    pub fn new(aspect: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            aspect: aspect.into(),
            description: description.into(),
            perspective_a: (String::new(), String::new()),
            perspective_b: (String::new(), String::new()),
            integration_hint: String::new(),
        }
    }

    pub fn with_perspectives(
        mut self,
        engine_a: impl Into<String>,
        view_a: impl Into<String>,
        engine_b: impl Into<String>,
        view_b: impl Into<String>,
    ) -> Self {
        self.perspective_a = (engine_a.into(), view_a.into());
        self.perspective_b = (engine_b.into(), view_b.into());
        self
    }

    pub fn with_integration_hint(mut self, hint: impl Into<String>) -> Self {
        self.integration_hint = hint.into();
        self
    }
}

/// A witness prompt generated from synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessPrompt {
    /// The prompt text
    pub text: String,
    /// What type of inquiry this invites
    pub inquiry_type: InquiryType,
    /// Related theme or tension (if any)
    pub context: Option<String>,
}

impl WitnessPrompt {
    pub fn new(text: impl Into<String>, inquiry_type: InquiryType) -> Self {
        Self {
            text: text.into(),
            inquiry_type,
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Types of witness inquiry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InquiryType {
    /// Observing cross-system patterns
    PatternNoticing,
    /// Exploring tensions between systems
    TensionExploration,
    /// Shifting perspective relationship
    PerspectiveShift,
    /// Deepening understanding
    Understanding,
    /// Integration and action
    Integration,
}

/// Temporal recommendation from daily practice synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalWindow {
    /// Start time
    pub start: DateTime<Utc>,
    /// End time  
    pub end: DateTime<Utc>,
    /// Quality of this window (0.0-1.0)
    pub quality: f32,
    /// Recommended activities
    pub activities: Vec<String>,
    /// Systems that support this window
    pub supporting_systems: Vec<String>,
    /// Description
    pub description: String,
}

impl TemporalWindow {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            start,
            end,
            quality: 0.5,
            activities: Vec::new(),
            supporting_systems: Vec::new(),
            description: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_strength_calculation() {
        let theme = Theme::new("Leadership", "Natural leadership abilities")
            .with_sources(vec!["numerology".into(), "human-design".into(), "vimshottari".into()]);
        
        assert_eq!(theme.strength, 0.6); // 3/5
    }

    #[test]
    fn theme_add_source() {
        let mut theme = Theme::new("Creativity", "Creative expression");
        theme.add_source("numerology");
        theme.add_source("human-design");
        
        assert_eq!(theme.sources.len(), 2);
        assert_eq!(theme.strength, 0.4); // 2/5
    }

    #[test]
    fn witness_prompt_with_context() {
        let prompt = WitnessPrompt::new(
            "What do you notice about how Leadership appears across these lenses?",
            InquiryType::PatternNoticing
        ).with_context("Leadership theme");
        
        assert!(prompt.context.is_some());
    }
}
