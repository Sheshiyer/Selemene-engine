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

    /// Perform biofield analysis — routes to Vedic or mock path
    /// T-026: added capture result mapping branch for frozen contracts (11 metrics, consent)
    /// refs p1-w1-worker-bootstrap-packet + 3 extraction + FROZEN + detailed-task-list T-026 + EXECUTION-STATUS. Keep is_mock.
    fn analyze(&self, input: &EngineInput) -> Result<AnalysisOutcome, EngineError> {
        // If birth_data is provided, use real Vedic analysis (T-026 Vedic path harden)
        if let Some(birth_data) = &input.birth_data {
            let result = self.analyzer.analyze(birth_data, input.current_time)?;
            return Ok(AnalysisOutcome::Vedic(result));
        }

        // T-026 capture mapping: if consent or image_data-like in options, map to 11-metric frozen shape (biofield-capture contract)
        let has_capture = input.options.get("image_data").is_some() || input.options.get("consent").is_some() || input.options.get("capture").is_some();
        if has_capture {
            // minimal capture mock mapping to 11 metrics per FROZEN (from py: light_quanta... pattern_regularity) + consent echo; keep is_mock
            let seed = input.options.get("seed").and_then(|v| v.as_u64()).unwrap_or(42);
            let m = generate_mock_metrics(Some(seed));
            // extend to 11 via placeholders matching frozen contract example
            let consent_val = input.options.get("consent").cloned();
            let qual = input.options.get("quality").cloned();
            // for result we'll special case in serialize; here return mock with flags
            let interpretation = "Capture-mapped (11 metrics per frozen; T-026)".to_string();
            let areas = vec!["see result.metrics for 11 fields".to_string()];
            return Ok(AnalysisOutcome::Mock(BiofieldAnalysis {
                metrics: m,
                interpretation,
                areas_of_attention: areas,
                is_mock_data: true,
                consent: consent_val,
                quality: qual,
            }));
        }

        // Fall back to mock data
        let seed = input.options.get("seed").and_then(|v| v.as_u64());
        let metrics = if let Some(user_id) = input.options.get("user_id").and_then(|v| v.as_str()) {
            generate_metrics_for_user(user_id)
        } else {
            generate_mock_metrics(seed)
        };

        let interpretation = Self::generate_mock_interpretation(&metrics);
        let areas_of_attention = Self::identify_mock_areas_of_attention(&metrics);

        Ok(AnalysisOutcome::Mock(BiofieldAnalysis {
            metrics,
            interpretation,
            areas_of_attention,
            is_mock_data: true,
            consent: None,
            quality: None,
        }))
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

/// Internal enum to carry analysis result + optional Vedic context
enum AnalysisOutcome {
    Vedic(VedicAnalysisResult),
    Mock(BiofieldAnalysis),
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

        let (analysis, witness_prompt, backend, precision) = match &outcome {
            AnalysisOutcome::Vedic(vedic_result) => {
                let prompt =
                    generate_vedic_witness_prompt(&vedic_result.analysis, &vedic_result.context);
                (
                    &vedic_result.analysis,
                    prompt,
                    "vedic-ephemeris",
                    format!("{:?}", input.precision),
                )
            }
            AnalysisOutcome::Mock(mock_analysis) => {
                let prompt = generate_witness_prompt(mock_analysis);
                (mock_analysis, prompt, "mock", "simulated".to_string())
            }
        };

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
        for key in ["light_quanta_density", "normalized_area", "average_intensity", "inner_noise", "energy_analysis", "entropy_form_coefficient", "fractal_dimension", "correlation_dimension", "body_symmetry", "contour_complexity", "pattern_regularity"] {
            assert!(m.get(key).is_some(), "missing 11-metric key in capture map: {}", key);
        }
        assert!(res.get("consent").is_some(), "consent echoed for frozen");
        assert_eq!(res.get("computation_mode").and_then(|v| v.as_str()), Some("capture-mock"));
    }
}
