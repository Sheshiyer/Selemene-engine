//! ConsciousnessEngine implementation for Biofield
//!
//! Routes to Vedic birth-data-driven analysis when birth_data is provided,
//! falls back to mock data when absent.
//! T-026 P2 harden start: Vedic path + capture result mapping to frozen contracts (11 metrics, consent).
//! Cites: p1-w1-worker-bootstrap-packet.md + resources-and-assets.md + gaps-and-improvements.md + goal-understanding.md + P1W1-CONTRACTS-FROZEN.md + detailed-task-list.md (T-026) + EXECUTION-STATUS.md + noesis-core frozen media (worktree). Keep is_mock for now. Minimal no creep.

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::Instant;

use crate::mock::{generate_metrics_for_user, generate_mock_metrics};
use crate::models::{BiofieldAnalysis, BiofieldMetrics};
use crate::vedic::{generate_vedic_witness_prompt, VedicAnalysisResult, VedicBiofieldAnalyzer};
use crate::wisdom::{get_chakra_wisdom, get_metric_interpretation};
use crate::witness::{generate_witness_dyad, generate_witness_prompt};

/// Biofield consciousness engine
///
/// Analyzes biofield energy patterns using Vedic planetary correspondences
/// when birth data is provided, or returns mock data as fallback.
pub struct BiofieldEngine {
    engine_id: String,
    engine_name: String,
    analyzer: VedicBiofieldAnalyzer,
}

impl BiofieldEngine {
    /// Create a new Biofield engine instance
    pub fn new() -> Self {
        Self {
            engine_id: "biofield".to_string(),
            engine_name: "Biofield".to_string(),
            analyzer: VedicBiofieldAnalyzer::new(),
        }
    }

    /// Generate interpretation text from mock metrics
    fn generate_mock_interpretation(metrics: &BiofieldMetrics) -> String {
        let mut parts = Vec::new();

        if metrics.vitality_index > 0.7 {
            parts.push("Overall vitality appears strong".to_string());
        } else if metrics.vitality_index < 0.4 {
            parts.push("Vitality may benefit from attention".to_string());
        } else {
            parts.push("Vitality is in a moderate range".to_string());
        }

        if let Some(interp) = get_metric_interpretation("coherence") {
            if metrics.coherence < interp.optimal_range.0 {
                parts.push("Coherence patterns suggest potential for more integration".to_string());
            } else if metrics.coherence > interp.optimal_range.1 {
                parts.push("Coherence is notably high, suggesting aligned energy flow".to_string());
            }
        }

        if metrics.symmetry < 0.5 {
            parts.push("Left-right balance shows some asymmetry".to_string());
        }

        parts.push("Note: This interpretation is based on simulated data".to_string());

        parts.join(". ") + "."
    }

    /// Identify areas that may benefit from attention (mock path)
    fn identify_mock_areas_of_attention(metrics: &BiofieldMetrics) -> Vec<String> {
        let mut areas = Vec::new();

        for reading in &metrics.chakra_readings {
            if reading.activity_level < 0.4 {
                if let Some(wisdom) = get_chakra_wisdom(reading.chakra) {
                    areas.push(format!(
                        "{} ({}) - lower activity observed",
                        wisdom.name, wisdom.location
                    ));
                }
            }

            if reading.balance.abs() > 0.4 {
                let side = if reading.balance > 0.0 {
                    "right-dominant"
                } else {
                    "left-dominant"
                };
                areas.push(format!(
                    "{} chakra shows {} pattern",
                    reading.chakra.name(),
                    side
                ));
            }
        }

        if metrics.coherence < 0.4 {
            areas.push("Overall coherence may benefit from grounding practices".to_string());
        }

        if metrics.entropy < 0.3 {
            areas.push("Energy patterns appear uniform - gentle movement may help".to_string());
        }

        if areas.is_empty() {
            areas.push("No specific areas require immediate attention".to_string());
        }

        areas
    }

    /// Perform biofield analysis. Three explicit paths:
    ///
    /// * [`Self::analyze_vedic`] — real ephemeris-backed analysis when `birth_data` is present.
    /// * [`Self::analyze_capture`] — capture-shaped request (`image_data` / `consent` /
    ///   `capture` / `biofield_capture` in options). This engine has no readings
    ///   repository, so it always returns the 11-metric mock and declares it; the real
    ///   persisted capture result is served by the `biofield-capture` engine.
    /// * [`Self::analyze_mock`] — plain simulated metrics.
    ///
    /// Every path returns a [`Provenance`] that `calculate` writes to `result.provenance`.
    fn analyze(&self, input: &EngineInput) -> Result<AnalysisOutcome, EngineError> {
        if input.birth_data.is_some() {
            return self.analyze_vedic(input);
        }

        if Self::is_capture_request(input) {
            return Ok(self.analyze_capture(input));
        }

        Ok(self.analyze_mock(input))
    }

    fn is_capture_request(input: &EngineInput) -> bool {
        input.options.contains_key("image_data")
            || input.options.contains_key("consent")
            || input.options.contains_key("capture")
            || input.options.contains_key("biofield_capture")
    }

    fn consent_granted(input: &EngineInput) -> bool {
        input
            .options
            .get("consent")
            .and_then(|c| c.get("granted"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    fn capture_reading_id(input: &EngineInput) -> Option<String> {
        input
            .options
            .get("biofield_capture")
            .and_then(|v| v.get("reading_id"))
            .and_then(Value::as_str)
            .map(str::to_string)
    }

    /// Real path: Vedic planetary analysis from birth data.
    fn analyze_vedic(&self, input: &EngineInput) -> Result<AnalysisOutcome, EngineError> {
        let birth_data = input
            .birth_data
            .as_ref()
            .ok_or_else(|| EngineError::ValidationError("birth_data is required".to_string()))?;
        let result = self.analyzer.analyze(birth_data, input.current_time)?;
        let provenance = Provenance {
            backend: "vedic-ephemeris",
            precision: format!("{:?}", input.precision),
            fallback_used: false,
            fallback_reason: None,
            birth_data_provided: true,
            consent_granted: Self::consent_granted(input),
            capture_reading_id: None,
            mock_reason: None,
        };
        Ok(AnalysisOutcome::Vedic(result, provenance))
    }

    /// Capture-shaped request. Maps the simulated metrics onto the frozen 11-metric
    /// `biofield-capture` shape and echoes consent/quality, but is always mock:
    /// the standalone `biofield` engine has no readings repository, so it cannot
    /// resolve `options.biofield_capture.reading_id`. Callers who want the real
    /// persisted capture result must calculate engine `biofield-capture` with the
    /// same `reading_id`.
    fn analyze_capture(&self, input: &EngineInput) -> AnalysisOutcome {
        let seed = input
            .options
            .get("seed")
            .and_then(|v| v.as_u64())
            .unwrap_or(42);
        let metrics = generate_mock_metrics(Some(seed));
        let consent_val = input.options.get("consent").cloned();
        let quality = input.options.get("quality").cloned();
        let reading_id = Self::capture_reading_id(input);

        let fallback_reason = match &reading_id {
            Some(id) => format!(
                "reading_id {id} supplied, but the `biofield` engine has no readings repository; \
                 the persisted capture result is served by engine `biofield-capture` \
                 (options.biofield_capture.reading_id). This result is simulated."
            ),
            None => "no biofield_capture.reading_id supplied; capture-shaped request served with \
                     simulated metrics"
                .to_string(),
        };
        let mock_reason = if reading_id.is_some() {
            "biofield-engine-cannot-resolve-reading"
        } else {
            "no-reading-id"
        };

        let analysis = BiofieldAnalysis {
            metrics,
            interpretation:
                "Simulated capture-shaped metrics (11-metric frozen contract). Not derived from an image."
                    .to_string(),
            areas_of_attention: vec![
                "Simulated data: calculate engine `biofield-capture` with a reading_id for real metrics"
                    .to_string(),
            ],
            is_mock_data: true,
            consent: consent_val,
            quality,
        };
        let provenance = Provenance {
            backend: "mock",
            precision: "simulated".to_string(),
            fallback_used: true,
            fallback_reason: Some(fallback_reason),
            birth_data_provided: false,
            consent_granted: Self::consent_granted(input),
            capture_reading_id: reading_id,
            mock_reason: Some(mock_reason),
        };
        AnalysisOutcome::Mock(analysis, provenance)
    }

    /// Plain simulated metrics, declared as such.
    fn analyze_mock(&self, input: &EngineInput) -> AnalysisOutcome {
        let seed = input.options.get("seed").and_then(|v| v.as_u64());
        let metrics = if let Some(user_id) = input.options.get("user_id").and_then(|v| v.as_str()) {
            generate_metrics_for_user(user_id)
        } else {
            generate_mock_metrics(seed)
        };

        let interpretation = Self::generate_mock_interpretation(&metrics);
        let areas_of_attention = Self::identify_mock_areas_of_attention(&metrics);

        let analysis = BiofieldAnalysis {
            metrics,
            interpretation,
            areas_of_attention,
            is_mock_data: true,
            consent: None,
            quality: None,
        };
        let provenance = Provenance {
            backend: "mock",
            precision: "simulated".to_string(),
            fallback_used: true,
            fallback_reason: Some(
                "no birth_data provided; biofield metrics are simulated".to_string(),
            ),
            birth_data_provided: false,
            consent_granted: Self::consent_granted(input),
            capture_reading_id: None,
            mock_reason: Some("no-birth-data"),
        };
        AnalysisOutcome::Mock(analysis, provenance)
    }

    /// Serialize analysis result to JSON
    /// T-026: capture result mapping -- when consent present, emit 11-metric frozen shape (see P1W1-CONTRACTS-FROZEN + py 11 keys) + echo consent/quality
    fn serialize_result(analysis: &BiofieldAnalysis) -> Value {
        let chakra_readings: Vec<Value> = analysis
            .metrics
            .chakra_readings
            .iter()
            .map(|r| {
                let mut reading = json!({
                    "chakra": format!("{:?}", r.chakra),
                    "chakra_name": r.chakra.name(),
                    "activity_level": r.activity_level,
                    "balance": r.balance,
                    "color_intensity": r.color_intensity,
                });

                if let Some(wisdom) = get_chakra_wisdom(r.chakra) {
                    reading["location"] = json!(wisdom.location);
                    reading["element"] = json!(wisdom.element);
                }

                reading
            })
            .collect();

        let mut res = json!({
            "metrics": {
                "fractal_dimension": analysis.metrics.fractal_dimension,
                "entropy": analysis.metrics.entropy,
                "coherence": analysis.metrics.coherence,
                "symmetry": analysis.metrics.symmetry,
                "vitality_index": analysis.metrics.vitality_index,
                "timestamp": analysis.metrics.timestamp,
            },
            "chakra_readings": chakra_readings,
            "interpretation": analysis.interpretation,
            "areas_of_attention": analysis.areas_of_attention,
            "is_mock_data": analysis.is_mock_data,
        });

        // T-026 capture mapping to frozen (11 metrics + consent) -- used when analyze saw capture flag
        if analysis.consent.is_some() || analysis.quality.is_some() {
            // emit 11 per FROZEN example (py sidecar keys + body_symmetry etc); values from base + placeholders for now
            res["metrics"] = json!({
                "light_quanta_density": analysis.metrics.fractal_dimension * 1e8,
                "normalized_area": analysis.metrics.symmetry,
                "average_intensity": analysis.metrics.vitality_index,
                "inner_noise": analysis.metrics.entropy,
                "energy_analysis": {"low": 0.3, "medium": 0.5, "high": 0.2},
                "entropy_form_coefficient": analysis.metrics.entropy,
                "fractal_dimension": analysis.metrics.fractal_dimension,
                "correlation_dimension": analysis.metrics.coherence,
                "body_symmetry": analysis.metrics.symmetry,
                "contour_complexity": analysis.metrics.entropy,
                "pattern_regularity": analysis.metrics.coherence,
            });
            if let Some(c) = &analysis.consent {
                res["consent"] = c.clone();
            }
            if let Some(q) = &analysis.quality {
                res["quality_assessment"] = q.clone();
            }
            res["computation_mode"] = json!("capture-mock");
        }
        res
    }
}

/// Where the result came from. Written verbatim to `result.provenance` so a
/// consumer can never mistake simulated metrics for a real reading.
#[derive(Debug, Clone)]
struct Provenance {
    /// `vedic-ephemeris` | `mock`. (`biofield-capture` is only ever emitted by the
    /// `biofield-capture` engine, which owns the persisted reading.)
    backend: &'static str,
    precision: String,
    /// `true` whenever the result is simulated.
    fallback_used: bool,
    fallback_reason: Option<String>,
    birth_data_provided: bool,
    consent_granted: bool,
    /// `options.biofield_capture.reading_id` as received, if any.
    capture_reading_id: Option<String>,
    /// Short machine-readable reason the result is mock (`None` on the real path).
    mock_reason: Option<&'static str>,
}

impl Provenance {
    fn to_json(&self) -> Value {
        json!({
            "backend": self.backend,
            "precision": self.precision,
            "fallback_used": self.fallback_used,
            "fallback_reason": self.fallback_reason,
            "birth_data_provided": self.birth_data_provided,
            "consent_granted": self.consent_granted,
            "capture_reading_id": self.capture_reading_id,
            "real_capture_engine": "biofield-capture",
        })
    }
}

/// Internal enum to carry analysis result + provenance
enum AnalysisOutcome {
    Vedic(VedicAnalysisResult, Provenance),
    Mock(BiofieldAnalysis, Provenance),
}

impl Default for BiofieldEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for BiofieldEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    fn required_phase(&self) -> u8 {
        1 // Requires somatic awareness (phase 1)
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        let outcome = self.analyze(&input)?;

        let (analysis, witness_prompt, provenance) = match &outcome {
            AnalysisOutcome::Vedic(vedic_result, provenance) => {
                let prompt =
                    generate_vedic_witness_prompt(&vedic_result.analysis, &vedic_result.context);
                (&vedic_result.analysis, prompt, provenance)
            }
            AnalysisOutcome::Mock(mock_analysis, provenance) => {
                let prompt = generate_witness_prompt(mock_analysis);
                (mock_analysis, prompt, provenance)
            }
        };
        let backend = provenance.backend;
        let precision = provenance.precision.clone();
        // Mock and capture-mock paths are fallbacks; the vedic path is real.
        let fallback_used = provenance.fallback_used;

        if witness_prompt.is_empty() {
            return Err(EngineError::CalculationError(
                "Witness prompt generation failed: empty result".to_string(),
            ));
        }

        let consciousness_level = input
            .options
            .get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(1);

        let elapsed = start.elapsed();

        let mut result = Self::serialize_result(analysis);

        if let Some(reason) = provenance.mock_reason {
            result["mock_reason"] = json!(reason);
        }
        result["provenance"] = provenance.to_json();

        if analysis.consent.is_some() {
            // T-026 capture: preserve 11-metric + consent from serialize (frozen contract); no full mock notice
            if result.get("computation_mode").is_none() {
                result["computation_mode"] = json!("capture-mock");
            }
        } else if analysis.is_mock_data {
            result["notice"] = json!(
                "This is simulated data. Full biofield analysis requires PIP hardware integration."
            );
            result["future_capabilities"] = json!([
                "Real-time biofield imaging",
                "Chakra activity measurement",
                "Aura color spectrum analysis",
                "Energy flow pattern tracking",
                "Biofield coherence monitoring",
                "Before/after intervention comparison",
            ]);
            result["computation_mode"] = json!("mock");
        } else {
            result["computation_mode"] = json!("vedic");
        }

        // ── Witness Dyad injection ───────────────────────────────────────────────
        // Inject Aletheios / Pichet perspectives directly into result JSON so the
        // frontend can read result.witness_layer without needing a new EngineOutput field.
        let dyad = generate_witness_dyad(analysis);
        result["witness_layer"] = json!({
            "aletheios": { "perspective": dyad.aletheios },
            "pichet":    { "perspective": dyad.pichet },
            "synthesis": dyad.synthesis,
        });

        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result,
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: backend.to_string(),
                precision_achieved: precision,
                cached: false,
                timestamp: Utc::now(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                fallback_used,
                provider_id: None,
            },
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let mut messages = Vec::new();
        let mut valid = true;

        if output.witness_prompt.is_empty() {
            messages.push("Witness prompt is empty".to_string());
            valid = false;
        }

        if output.result.get("metrics").is_none() {
            messages.push("Missing 'metrics' field in result".to_string());
            valid = false;
        }

        if output.result.get("chakra_readings").is_none() {
            messages.push("Missing 'chakra_readings' field in result".to_string());
            valid = false;
        }

        if output.result.get("is_mock_data").is_none() {
            messages.push("Missing 'is_mock_data' flag".to_string());
            valid = false;
        }

        match output.result.get("provenance") {
            Some(provenance) if provenance.is_object() => {
                let backend_ok = matches!(
                    provenance.get("backend").and_then(Value::as_str),
                    Some("vedic-ephemeris" | "biofield-capture" | "mock")
                );
                if !backend_ok {
                    messages.push(
                        "provenance.backend must be vedic-ephemeris | biofield-capture | mock"
                            .to_string(),
                    );
                    valid = false;
                }
                let fallback_used = provenance.get("fallback_used").and_then(Value::as_bool);
                if fallback_used.is_none() {
                    messages.push("provenance.fallback_used must be a boolean".to_string());
                    valid = false;
                }
                let is_mock = output.result.get("is_mock_data").and_then(Value::as_bool);
                if is_mock == Some(true) && fallback_used != Some(true) {
                    messages.push(
                        "is_mock_data is true but provenance.fallback_used is not".to_string(),
                    );
                    valid = false;
                }
                for key in ["precision", "birth_data_provided", "consent_granted"] {
                    if provenance.get(key).is_none() {
                        messages.push(format!("provenance.{key} is missing"));
                        valid = false;
                    }
                }
            }
            _ => {
                messages.push("Missing 'provenance' object in result".to_string());
                valid = false;
            }
        }

        if let Some(metrics) = output.result.get("metrics") {
            if let Some(fd) = metrics.get("fractal_dimension").and_then(|v| v.as_f64()) {
                if !(1.0..=2.0).contains(&fd) {
                    messages.push(format!("fractal_dimension {} out of range [1.0, 2.0]", fd));
                    valid = false;
                }
            }

            for field in ["entropy", "coherence", "symmetry", "vitality_index"] {
                if let Some(val) = metrics.get(field).and_then(|v| v.as_f64()) {
                    if !(0.0..=1.0).contains(&val) {
                        messages.push(format!("{} {} out of range [0.0, 1.0]", field, val));
                        valid = false;
                    }
                }
            }
        }

        if let Some(readings) = output
            .result
            .get("chakra_readings")
            .and_then(|v| v.as_array())
        {
            if readings.len() != 7 {
                messages.push(format!(
                    "Expected 7 chakra readings, got {}",
                    readings.len()
                ));
                valid = false;
            }
        }

        let confidence = if valid { 1.0 } else { 0.0 };

        Ok(ValidationResult {
            valid,
            confidence,
            messages,
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        // Vedic analysis: cache by birth data + transit hour
        if let Some(birth_data) = &input.birth_data {
            let birth_str = format!(
                "{}_{}",
                birth_data.date,
                birth_data.time.as_deref().unwrap_or("12:00")
            );
            let transit_hour = input.current_time.format("%Y-%m-%d-%H").to_string();
            let raw = format!("biofield:vedic:{}:{}", birth_str, transit_hour);
            let hash = format!("{:x}", Sha256::digest(raw.as_bytes()));
            return format!("biofield:vedic:{}", hash);
        }

        // Mock data: existing cache logic
        if let Some(user_id) = input.options.get("user_id").and_then(|v| v.as_str()) {
            format!("biofield:user:{}", user_id)
        } else if let Some(seed) = input.options.get("seed").and_then(|v| v.as_u64()) {
            format!("biofield:seed:{}", seed)
        } else {
            format!(
                "biofield:random:{}",
                Utc::now().timestamp_nanos_opt().unwrap_or(0)
            )
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noesis_core::{BirthData, Precision};
    use std::collections::HashMap;

    fn create_test_input() -> EngineInput {
        let mut options = HashMap::new();
        options.insert("seed".to_string(), json!(42));

        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        }
    }

    fn create_vedic_input() -> EngineInput {
        EngineInput {
            birth_data: Some(BirthData {
                name: Some("Test".to_string()),
                date: "1990-01-01".to_string(),
                time: Some("14:30".to_string()),
                latitude: 12.9716,
                longitude: 77.5946,
                timezone: "Asia/Kolkata".to_string(),
            }),
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = BiofieldEngine::new();
        assert_eq!(engine.engine_id(), "biofield");
        assert_eq!(engine.engine_name(), "Biofield");
        assert_eq!(engine.required_phase(), 1);
    }

    #[tokio::test]
    async fn test_calculate_returns_mock_data() {
        let engine = BiofieldEngine::new();
        let input = create_test_input();

        let result = engine.calculate(input).await;
        assert!(result.is_ok(), "Calculation should succeed");

        let output = result.unwrap();
        assert_eq!(output.engine_id, "biofield");
        assert!(!output.witness_prompt.is_empty());

        let is_mock = output
            .result
            .get("is_mock_data")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        assert!(is_mock, "Should indicate mock data");

        assert!(output.result.get("notice").is_some());
        assert!(output.result.get("future_capabilities").is_some());
        assert_eq!(
            output
                .result
                .get("computation_mode")
                .and_then(|v| v.as_str()),
            Some("mock")
        );
        assert_eq!(output.metadata.backend, "mock");
    }

    #[tokio::test]
    async fn test_calculate_with_birth_data_returns_vedic() {
        let engine = BiofieldEngine::new();
        let input = create_vedic_input();

        let result = engine.calculate(input).await;
        assert!(
            result.is_ok(),
            "Vedic calculation should succeed: {:?}",
            result.err()
        );

        let output = result.unwrap();
        assert_eq!(output.engine_id, "biofield");
        assert!(!output.witness_prompt.is_empty());

        let is_mock = output
            .result
            .get("is_mock_data")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        assert!(!is_mock, "Should NOT be mock data when birth_data provided");

        assert_eq!(
            output
                .result
                .get("computation_mode")
                .and_then(|v| v.as_str()),
            Some("vedic")
        );
        assert_eq!(output.metadata.backend, "vedic-ephemeris");

        // Should not have mock notice
        assert!(output.result.get("notice").is_none());
    }

    #[tokio::test]
    async fn test_vedic_output_passes_validation() {
        let engine = BiofieldEngine::new();
        let input = create_vedic_input();

        let output = engine.calculate(input).await.unwrap();
        let validation = engine.validate(&output).await.unwrap();

        assert!(
            validation.valid,
            "Vedic output should pass validation: {:?}",
            validation.messages
        );
    }

    #[tokio::test]
    async fn test_calculate_with_seed_is_reproducible() {
        let engine = BiofieldEngine::new();

        let mut options = HashMap::new();
        options.insert("seed".to_string(), json!(12345));

        let input1 = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: options.clone(),
        };

        let input2 = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };

        let output1 = engine.calculate(input1).await.unwrap();
        let output2 = engine.calculate(input2).await.unwrap();

        let fd1 = output1.result["metrics"]["fractal_dimension"].as_f64();
        let fd2 = output2.result["metrics"]["fractal_dimension"].as_f64();
        assert_eq!(fd1, fd2, "Same seed should produce same results");
    }

    #[tokio::test]
    async fn test_calculate_with_user_id() {
        let engine = BiofieldEngine::new();

        let mut options = HashMap::new();
        options.insert("user_id".to_string(), json!("test_user_123"));

        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };

        let output = engine.calculate(input).await.unwrap();
        assert!(output.result.get("metrics").is_some());
    }

    #[tokio::test]
    async fn test_validate_output() {
        let engine = BiofieldEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        let validation = engine.validate(&output).await.unwrap();

        assert!(
            validation.valid,
            "Valid output should pass validation: {:?}",
            validation.messages
        );
        assert_eq!(validation.confidence, 1.0);
    }

    #[tokio::test]
    async fn test_validate_detects_empty_witness_prompt() {
        let engine = BiofieldEngine::new();

        let output = EngineOutput {
            engine_id: "biofield".to_string(),
            result: json!({
                "metrics": {
                    "fractal_dimension": 1.5,
                    "entropy": 0.5,
                    "coherence": 0.6,
                    "symmetry": 0.7,
                    "vitality_index": 0.6,
                },
                "chakra_readings": [],
                "is_mock_data": true,
            }),
            witness_prompt: "".to_string(),
            consciousness_level: 1,
            metadata: CalculationMetadata {
                calculation_time_ms: 1.0,
                backend: "test".to_string(),
                precision_achieved: "test".to_string(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: String::new(),
                fallback_used: false,
                provider_id: None,
            },
        };

        let validation = engine.validate(&output).await.unwrap();
        assert!(!validation.valid);
        assert!(validation.messages.iter().any(|m| m.contains("empty")));
    }

    #[tokio::test]
    async fn test_cache_key_with_seed() {
        let engine = BiofieldEngine::new();
        let input = create_test_input();

        let key = engine.cache_key(&input);
        assert!(key.starts_with("biofield:seed:"));
        assert!(key.contains("42"));
    }

    #[tokio::test]
    async fn test_cache_key_with_user_id() {
        let engine = BiofieldEngine::new();

        let mut options = HashMap::new();
        options.insert("user_id".to_string(), json!("user123"));

        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };

        let key = engine.cache_key(&input);
        assert!(key.starts_with("biofield:user:"));
        assert!(key.contains("user123"));
    }

    #[tokio::test]
    async fn test_cache_key_vedic() {
        let engine = BiofieldEngine::new();
        let input = create_vedic_input();

        let key = engine.cache_key(&input);
        assert!(key.starts_with("biofield:vedic:"));
    }

    #[tokio::test]
    async fn test_output_has_chakra_readings() {
        let engine = BiofieldEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();

        let readings = output
            .result
            .get("chakra_readings")
            .and_then(|v| v.as_array())
            .expect("Should have chakra_readings array");

        assert_eq!(readings.len(), 7, "Should have 7 chakra readings");

        let first = &readings[0];
        assert!(first.get("chakra").is_some());
        assert!(first.get("activity_level").is_some());
        assert!(first.get("balance").is_some());
        assert!(first.get("color_intensity").is_some());
    }

    #[tokio::test]
    async fn test_metrics_in_valid_ranges() {
        let engine = BiofieldEngine::new();

        for seed in 0..20 {
            let mut options = HashMap::new();
            options.insert("seed".to_string(), json!(seed));

            let input = EngineInput {
                birth_data: None,
                current_time: Utc::now(),
                location: None,
                precision: Precision::Standard,
                options,
            };

            let output = engine.calculate(input).await.unwrap();
            let metrics = output.result.get("metrics").unwrap();

            let fd = metrics["fractal_dimension"].as_f64().unwrap();
            assert!(
                (1.0..=2.0).contains(&fd),
                "fractal_dimension {} out of range",
                fd
            );

            for field in ["entropy", "coherence", "symmetry", "vitality_index"] {
                let val = metrics[field].as_f64().unwrap();
                assert!((0.0..=1.0).contains(&val), "{} {} out of range", field, val);
            }
        }
    }

    // T-026 P2 start roundtrip test: capture mapping to frozen (11 metrics, consent)
    // refs extraction files + FROZEN + bootstrap + detailed-task-list T-026 + EXECUTION-STATUS. Keep is_mock.
    #[tokio::test]
    async fn test_calculate_capture_roundtrip_maps_to_frozen_11_metrics_consent() {
        let engine = BiofieldEngine::new();
        let mut options = HashMap::new();
        options.insert("capture".to_string(), json!(true));
        options.insert("consent".to_string(), json!({"granted":true,"scopes":["biofield-capture"],"timestamp":"2026-07-17T12:00:00Z"}));

        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };

        let output = engine.calculate(input).await.unwrap();
        assert_eq!(output.engine_id, "biofield");
        let res = &output.result;
        // 11 metrics present per frozen contract
        let m = res.get("metrics").expect("metrics");
        for key in [
            "light_quanta_density",
            "normalized_area",
            "average_intensity",
            "inner_noise",
            "energy_analysis",
            "entropy_form_coefficient",
            "fractal_dimension",
            "correlation_dimension",
            "body_symmetry",
            "contour_complexity",
            "pattern_regularity",
        ] {
            assert!(
                m.get(key).is_some(),
                "missing 11-metric key in capture map: {}",
                key
            );
        }
        assert!(res.get("consent").is_some(), "consent echoed for frozen");
        assert_eq!(
            res.get("computation_mode").and_then(|v| v.as_str()),
            Some("capture-mock")
        );
        // honesty: capture-shaped without a reading id is declared mock
        assert_eq!(res["is_mock_data"], json!(true));
        assert_eq!(res["mock_reason"], json!("no-reading-id"));
        assert_eq!(res["provenance"]["fallback_used"], json!(true));
        assert_eq!(res["provenance"]["backend"], json!("mock"));
        assert_eq!(res["provenance"]["consent_granted"], json!(true));
        assert!(res["provenance"]["capture_reading_id"].is_null());
    }

    #[tokio::test]
    async fn test_provenance_vedic_path_is_real() {
        let engine = BiofieldEngine::new();
        let output = engine.calculate(create_vedic_input()).await.unwrap();
        let prov = &output.result["provenance"];
        assert_eq!(prov["backend"], json!("vedic-ephemeris"));
        assert_eq!(prov["fallback_used"], json!(false));
        assert!(prov["fallback_reason"].is_null());
        assert_eq!(prov["birth_data_provided"], json!(true));
        assert_eq!(prov["consent_granted"], json!(false));
        assert_eq!(prov["precision"], json!("Standard"));
        assert!(output.result.get("mock_reason").is_none());
        assert_eq!(output.result["is_mock_data"], json!(false));
        assert_eq!(output.metadata.backend, "vedic-ephemeris");
        assert!(!output.metadata.fallback_used);
        assert!(output.metadata.provider_id.is_none());
        let validation = engine.validate(&output).await.unwrap();
        assert!(validation.valid, "{:?}", validation.messages);
    }

    #[tokio::test]
    async fn test_provenance_mock_path_declares_fallback() {
        let engine = BiofieldEngine::new();
        let output = engine.calculate(create_test_input()).await.unwrap();
        let prov = &output.result["provenance"];
        assert_eq!(prov["backend"], json!("mock"));
        assert_eq!(prov["precision"], json!("simulated"));
        assert_eq!(prov["fallback_used"], json!(true));
        assert_eq!(
            prov["fallback_reason"],
            json!("no birth_data provided; biofield metrics are simulated")
        );
        assert_eq!(prov["birth_data_provided"], json!(false));
        assert_eq!(prov["consent_granted"], json!(false));
        assert_eq!(output.result["mock_reason"], json!("no-birth-data"));
        assert_eq!(output.result["is_mock_data"], json!(true));
        assert_eq!(output.result["computation_mode"], json!("mock"));
        assert!(output.metadata.fallback_used);
        let validation = engine.validate(&output).await.unwrap();
        assert!(validation.valid, "{:?}", validation.messages);
    }

    #[tokio::test]
    async fn test_provenance_capture_with_reading_id_is_declared_mock() {
        let engine = BiofieldEngine::new();
        let mut options = HashMap::new();
        options.insert(
            "biofield_capture".to_string(),
            json!({"reading_id": "11111111-2222-3333-4444-555555555555"}),
        );
        options.insert(
            "consent".to_string(),
            json!({"granted": true, "scopes": ["biofield-capture"]}),
        );
        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };

        let output = engine.calculate(input).await.unwrap();
        let res = &output.result;
        assert_eq!(res["is_mock_data"], json!(true));
        assert_eq!(
            res["mock_reason"],
            json!("biofield-engine-cannot-resolve-reading")
        );
        assert_eq!(res["computation_mode"], json!("capture-mock"));
        let prov = &res["provenance"];
        assert_eq!(prov["backend"], json!("mock"));
        assert_eq!(prov["fallback_used"], json!(true));
        assert!(output.metadata.fallback_used);
        assert_eq!(
            prov["capture_reading_id"],
            json!("11111111-2222-3333-4444-555555555555")
        );
        assert_eq!(prov["real_capture_engine"], json!("biofield-capture"));
        let reason = prov["fallback_reason"].as_str().unwrap();
        assert!(reason.contains("biofield-capture"), "reason was {reason}");
        assert!(reason.contains("11111111-2222-3333-4444-555555555555"));
        assert_eq!(prov["consent_granted"], json!(true));
        let validation = engine.validate(&output).await.unwrap();
        assert!(validation.valid, "{:?}", validation.messages);
    }

    #[tokio::test]
    async fn test_validate_rejects_missing_provenance() {
        let engine = BiofieldEngine::new();
        let mut output = engine.calculate(create_test_input()).await.unwrap();
        output.result.as_object_mut().unwrap().remove("provenance");
        let validation = engine.validate(&output).await.unwrap();
        assert!(!validation.valid);
        assert!(validation.messages.iter().any(|m| m.contains("provenance")));
    }

    #[tokio::test]
    async fn test_validate_rejects_mock_without_fallback_flag() {
        let engine = BiofieldEngine::new();
        let mut output = engine.calculate(create_test_input()).await.unwrap();
        output.result["provenance"]["fallback_used"] = json!(false);
        let validation = engine.validate(&output).await.unwrap();
        assert!(!validation.valid);
        assert!(validation
            .messages
            .iter()
            .any(|m| m.contains("fallback_used")));
    }
}
