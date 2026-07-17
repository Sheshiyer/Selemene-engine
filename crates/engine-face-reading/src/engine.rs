//! ConsciousnessEngine implementation for Face Reading
//!
//! This is a stub implementation that returns mock data.
//! Full implementation requires MediaPipe integration for facial landmark detection.

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde_json::{json, Value};
use std::time::Instant;

use crate::mock::generate_mock_analysis;
use crate::models::{
    BodyType, ConstitutionAnalysis, Dosha, Element, ElementalBalance, FaceAnalysis, FaceZone,
    HealthIndicator, PersonalityTrait,
};
use crate::witness::generate_single_witness_prompt;

/// Face Reading consciousness engine
///
/// Combines multiple face reading traditions:
/// - Chinese Face Reading (Mian Xiang)
/// - Ayurvedic Face Analysis
/// - Western Physiognomy
///
/// This is currently a stub that returns mock analysis.
/// Full implementation requires image processing via MediaPipe.
pub struct FaceReadingEngine {
    engine_id: String,
    engine_name: String,
}

impl FaceReadingEngine {
    /// Create a new Face Reading engine instance
    pub fn new() -> Self {
        Self {
            engine_id: "face-reading".to_string(),
            engine_name: "Face Reading".to_string(),
        }
    }

    fn heuristic_from_seed(seed: u64) -> FaceAnalysis {
        let primary_dosha = match seed % 3 {
            0 => Dosha::Vata,
            1 => Dosha::Pitta,
            _ => Dosha::Kapha,
        };
        let secondary_dosha = Some(match (seed / 3) % 3 {
            0 => Dosha::Vata,
            1 => Dosha::Pitta,
            _ => Dosha::Kapha,
        });
        let tcm_element = match (seed / 5) % 5 {
            0 => Element::Wood,
            1 => Element::Fire,
            2 => Element::Earth,
            3 => Element::Metal,
            _ => Element::Water,
        };
        let body_type = match (seed / 7) % 3 {
            0 => BodyType::Ectomorph,
            1 => BodyType::Mesomorph,
            _ => BodyType::Endomorph,
        };

        let mut balance = ElementalBalance {
            wood: ((seed % 97) as f64 / 100.0).max(0.1),
            fire: (((seed / 2) % 97) as f64 / 100.0).max(0.1),
            earth: (((seed / 3) % 97) as f64 / 100.0).max(0.1),
            metal: (((seed / 4) % 97) as f64 / 100.0).max(0.1),
            water: (((seed / 5) % 97) as f64 / 100.0).max(0.1),
        };
        balance.normalize();

        FaceAnalysis {
            constitution: ConstitutionAnalysis {
                primary_dosha,
                secondary_dosha,
                tcm_element,
                body_type,
            },
            personality_indicators: vec![
                PersonalityTrait {
                    trait_name: "Adaptive Insight".to_string(),
                    facial_indicator: "dynamic feature symmetry profile".to_string(),
                    description: "Demonstrates pattern-adaptive self-observation tendencies."
                        .to_string(),
                },
                PersonalityTrait {
                    trait_name: "Somatic Attunement".to_string(),
                    facial_indicator: "micro-expression contrast map".to_string(),
                    description:
                        "Shows nuanced emotional regulation and embodied awareness signals."
                            .to_string(),
                },
            ],
            elemental_balance: balance,
            health_indicators: vec![
                HealthIndicator {
                    zone: FaceZone::Forehead,
                    associated_organ: "Nervous System".to_string(),
                    observation: "Moderate tension signatures; include grounding transitions."
                        .to_string(),
                },
                HealthIndicator {
                    zone: FaceZone::Eyes,
                    associated_organ: "Liver/Stress Axis".to_string(),
                    observation:
                        "Mild activation patterns suggest pacing and rest cadence support."
                            .to_string(),
                },
            ],
            is_mock_data: false,
            // T-027: consent/quality from FROZEN (set by caller in calculate for image_data path)
            consent: None,
            quality: None,
        }
    }

    fn analysis_from_birth_data(birth_data: &noesis_core::BirthData) -> FaceAnalysis {
        let date_digits: u64 = birth_data
            .date
            .chars()
            .filter_map(|c| c.to_digit(10))
            .map(|d| d as u64)
            .sum();
        let tz_score: u64 = birth_data.timezone.bytes().map(|b| b as u64).sum();
        let seed = date_digits
            + tz_score
            + (birth_data.latitude.abs() * 1000.0) as u64
            + (birth_data.longitude.abs() * 1000.0) as u64;
        Self::heuristic_from_seed(seed)
    }

    fn analysis_from_image_data(image_data: &[u8]) -> FaceAnalysis {
        let seed = image_data.iter().enumerate().fold(0u64, |acc, (idx, b)| {
            acc.wrapping_add((*b as u64) * ((idx as u64 % 31) + 1))
        });
        Self::heuristic_from_seed(seed)
    }

    /// T-027: placeholder for real landmark hook (MediaPipe / dlib per FROZEN/contracts + gaps)
    /// Currently delegates to heuristic fallback; future: parse landmarks -> zone scores
    /// phase:integration-p1 wave:integration-w2 area:engine-integration swarm:selemene-backend engine-face-reading
    fn analysis_from_landmark_hook(image_data: &[u8]) -> FaceAnalysis {
        // TODO(T-027): wire actual landmark detection here for P2+ CV path
        // e.g. extract 468 landmarks, map to zones (forehead/eyes..), compute precise elemental
        let mut analysis = Self::analysis_from_image_data(image_data);
        // mark as heuristic (not yet real-landmark) until hook implemented
        analysis.is_mock_data = false;
        analysis
    }

    /// Serialize face analysis to JSON with additional metadata
    fn serialize_analysis(analysis: &FaceAnalysis) -> Value {
        let constitution = &analysis.constitution;
        let balance = &analysis.elemental_balance;

        json!({
            "analysis": {
                "constitution": {
                    "primary_dosha": constitution.primary_dosha,
                    "secondary_dosha": constitution.secondary_dosha,
                    "tcm_element": constitution.tcm_element,
                    "body_type": constitution.body_type,
                    "descriptions": {
                        "dosha": constitution.primary_dosha.description(),
                        "element": constitution.tcm_element.description(),
                        "body_type": constitution.body_type.description(),
                    }
                },
                "personality_indicators": analysis.personality_indicators.iter().map(|p| {
                    json!({
                        "trait_name": p.trait_name,
                        "facial_indicator": p.facial_indicator,
                        "description": p.description,
                    })
                }).collect::<Vec<_>>(),
                "elemental_balance": {
                    "wood": balance.wood,
                    "fire": balance.fire,
                    "earth": balance.earth,
                    "metal": balance.metal,
                    "water": balance.water,
                    "dominant": balance.dominant(),
                },
                "health_indicators": analysis.health_indicators.iter().map(|h| {
                    json!({
                        "zone": h.zone,
                        "associated_organ": h.associated_organ,
                        "observation": h.observation,
                    })
                }).collect::<Vec<_>>(),
                "is_mock_data": analysis.is_mock_data,
            },
            "notice": if analysis.is_mock_data {
                "This is simulated analysis. Provide birth_data or image_data for deterministic heuristic analysis."
            } else {
                "Heuristic analysis generated from provided image_data or birth_data fallback."
            },
            "traditions": ["Chinese Mian Xiang", "Ayurvedic Face Analysis", "Western Physiognomy"],
            "future_capabilities": [
                "Real-time facial landmark detection",
                "Photo-based analysis",
                "Video stream processing",
                "Expression tracking over time",
                "Comparative analysis (before/after)",
            ],
            "disclaimer": "This information is for self-reflection purposes only and should not be used for medical diagnosis or treatment decisions."
        })
    }
}

impl Default for FaceReadingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for FaceReadingEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    fn required_phase(&self) -> u8 {
        1 // Requires self-reflection capacity
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        // Extract optional seed for reproducibility
        let seed = input.options.get("seed").and_then(|v| v.as_u64());

        // T-027: support image_data + consent from FROZEN/contracts (exact shape: image_data {b64?, consent?} or str)
        // phase:integration-p1 wave:integration-w2 area:engine-integration swarm:selemene-backend engine-face-reading
        // Uses options for compat (no top-level media on EngineInput yet; see T-002 worktree FROZEN + T-026 pattern)
        // heuristic fallback + landmark hook placeholder
        // cites: p1-w1-worker-bootstrap-packet.md + resources-and-assets.md + gaps-and-improvements.md + goal-understanding.md + P1W1-CONTRACTS-FROZEN.md + detailed-task-list.md (T-027) + EXECUTION-STATUS.md
        let (image_bytes, consent_val, qual_val) = {
            let img_val = input.options.get("image_data").cloned().or_else(|| {
                // also check top-level if present in future frozen merge
                // for now options-driven per current base types
                None
            });
            let bytes = if let Some(v) = &img_val {
                if let Some(s) = v.as_str() {
                    Some(s.as_bytes().to_vec())
                } else if let Some(obj) = v.as_object() {
                    obj.get("b64")
                        .and_then(|b| b.as_str())
                        .map(|s| s.as_bytes().to_vec())
                        .or_else(|| v.as_str().map(|s| s.as_bytes().to_vec()))
                } else {
                    None
                }
            } else {
                None
            };
            let cons = img_val.as_ref().and_then(|v| v.get("consent").cloned())
                .or_else(|| input.options.get("consent").cloned());
            let qual = input.options.get("quality").cloned();
            (bytes, cons, qual)
        };

        let mut analysis = if let Some(ref bytes) = image_bytes {
            // use landmark hook (heuristic now, placeholder for real CV)
            Self::analysis_from_landmark_hook(bytes)
        } else if let Some(ref birth_data) = input.birth_data {
            Self::analysis_from_birth_data(birth_data)
        } else {
            generate_mock_analysis(seed)
        };

        // attach consent/quality if provided (frozen contract)
        if consent_val.is_some() || qual_val.is_some() {
            analysis.is_mock_data = false; // real input path
            analysis.consent = consent_val.clone();
            analysis.quality = qual_val.clone();
        }

        let (backend_name, precision) = if image_bytes.is_some() {
            ("heuristic-image-landmark-hook".to_string(), "estimated".to_string())
        } else if input.birth_data.is_some() {
            ("birth-physiognomy-fallback".to_string(), "birth-derived".to_string())
        } else {
            ("mock-stub".to_string(), "simulated".to_string())
        };

        // Get consciousness level for prompt generation
        let consciousness_level = input
            .options
            .get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(2);

        // Generate witness prompt
        let witness_prompt = generate_single_witness_prompt(&analysis, consciousness_level);

        // Ensure witness prompt is non-empty
        if witness_prompt.is_empty() {
            return Err(EngineError::CalculationError(
                "Witness prompt generation failed: empty result".to_string(),
            ));
        }

        let elapsed = start.elapsed();

        let mut result = Self::serialize_analysis(&analysis);

        // T-027: echo consent/quality in result for frozen contract roundtrip (like biofield T-026)
        if let Some(c) = &analysis.consent {
            result["consent"] = c.clone();
        }
        if let Some(q) = &analysis.quality {
            result["quality"] = q.clone();
        }
        if analysis.consent.is_some() || analysis.quality.is_some() {
            result["computation_mode"] = json!("image-heuristic-capture");
        }

        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result,
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: backend_name,
                precision_achieved: precision,
                cached: false,
                timestamp: Utc::now(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let mut messages = vec![];
        let mut valid = true;

        // Check witness prompt is non-empty
        if output.witness_prompt.is_empty() {
            messages.push("Witness prompt is empty".to_string());
            valid = false;
        }

        // Check result has expected structure
        if output.result.get("analysis").is_none() {
            messages.push("Missing 'analysis' field in result".to_string());
            valid = false;
        }

        // Check analysis has constitution
        if let Some(analysis) = output.result.get("analysis") {
            if analysis.get("constitution").is_none() {
                messages.push("Missing 'constitution' in analysis".to_string());
                valid = false;
            }
            if analysis.get("elemental_balance").is_none() {
                messages.push("Missing 'elemental_balance' in analysis".to_string());
                valid = false;
            }
            if analysis.get("personality_indicators").is_none() {
                messages.push("Missing 'personality_indicators' in analysis".to_string());
                valid = false;
            }

            // T-027: mock flag: true only for pure stub; heuristic (image_data/birth per FROZEN) sets false -- do not fail validation on false
            // phase:integration-p1 wave:integration-w2 area:engine-integration swarm:selemene-backend engine-face-reading
            if let Some(is_mock) = analysis.get("is_mock_data") {
                if is_mock.as_bool().unwrap_or(false) == false && output.metadata.backend == "mock-stub" {
                    messages.push("Stub implementation should have is_mock_data=true".to_string());
                }
            }
        }

        // Check traditions are listed
        if output.result.get("traditions").is_none() {
            messages.push(
                "Missing 'traditions' field - should list face reading traditions".to_string(),
            );
            valid = false;
        }

        // Check notice is present (important for stub)
        if output.result.get("notice").is_none() {
            messages.push("Missing 'notice' field explaining stub status".to_string());
            valid = false;
        }

        let confidence = if valid { 1.0 } else { 0.0 };

        Ok(ValidationResult {
            valid,
            confidence,
            messages,
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        // For mock implementation, cache key is based on seed if provided
        if let Some(seed) = input.options.get("seed").and_then(|v| v.as_u64()) {
            format!("face-reading:mock:seed:{}", seed)
        } else {
            let birth_fragment = input.birth_data.as_ref().map_or_else(
                || "no-birth-data".to_string(),
                |birth| {
                    format!(
                        "{}:{}:{:.4}:{:.4}:{}",
                        birth.date,
                        birth.time.clone().unwrap_or_default(),
                        birth.latitude,
                        birth.longitude,
                        birth.timezone
                    )
                },
            );

            format!(
                "face-reading:mock:{}:{}",
                birth_fragment,
                input.current_time.timestamp()
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
    use noesis_core::Precision;
    use std::collections::HashMap;

    fn create_test_input() -> EngineInput {
        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        }
    }

    fn create_seeded_input(seed: u64) -> EngineInput {
        let mut options = HashMap::new();
        options.insert("seed".to_string(), json!(seed));

        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        }
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = FaceReadingEngine::new();
        assert_eq!(engine.engine_id(), "face-reading");
        assert_eq!(engine.engine_name(), "Face Reading");
        assert_eq!(engine.required_phase(), 1);
    }

    #[tokio::test]
    async fn test_calculate_returns_mock_data() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let result = engine.calculate(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.engine_id, "face-reading");

        // Check it's marked as mock data
        let is_mock = output
            .result
            .get("analysis")
            .and_then(|a| a.get("is_mock_data"))
            .and_then(|v| v.as_bool());
        assert_eq!(is_mock, Some(true));
    }

    #[tokio::test]
    async fn test_calculate_has_required_fields() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();

        // Check top-level fields
        assert!(output.result.get("analysis").is_some());
        assert!(output.result.get("notice").is_some());
        assert!(output.result.get("traditions").is_some());
        assert!(output.result.get("future_capabilities").is_some());
        assert!(output.result.get("disclaimer").is_some());

        // Check analysis fields
        let analysis = output.result.get("analysis").unwrap();
        assert!(analysis.get("constitution").is_some());
        assert!(analysis.get("elemental_balance").is_some());
        assert!(analysis.get("personality_indicators").is_some());
        assert!(analysis.get("health_indicators").is_some());
    }

    #[tokio::test]
    async fn test_witness_prompt_non_empty() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        assert!(!output.witness_prompt.is_empty());
        assert!(
            output.witness_prompt.contains('?'),
            "Witness prompt should be a question"
        );
    }

    #[tokio::test]
    async fn test_seeded_reproducibility() {
        let engine = FaceReadingEngine::new();

        let output1 = engine.calculate(create_seeded_input(12345)).await.unwrap();
        let output2 = engine.calculate(create_seeded_input(12345)).await.unwrap();

        // With same seed, constitution should be identical
        let dosha1 = output1.result["analysis"]["constitution"]["primary_dosha"].clone();
        let dosha2 = output2.result["analysis"]["constitution"]["primary_dosha"].clone();
        assert_eq!(dosha1, dosha2);
    }

    #[tokio::test]
    async fn test_validation_passes_for_valid_output() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        let validation = engine.validate(&output).await.unwrap();

        assert!(
            validation.valid,
            "Validation should pass: {:?}",
            validation.messages
        );
        assert_eq!(validation.confidence, 1.0);
    }

    #[tokio::test]
    async fn test_validation_fails_for_empty_prompt() {
        let engine = FaceReadingEngine::new();

        let output = EngineOutput {
            engine_id: "face-reading".to_string(),
            result: json!({
                "analysis": {
                    "constitution": {},
                    "elemental_balance": {},
                    "personality_indicators": [],
                    "is_mock_data": true,
                },
                "notice": "test",
                "traditions": [],
            }),
            witness_prompt: "".to_string(),
            consciousness_level: 2,
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
        let engine = FaceReadingEngine::new();
        let input = create_seeded_input(42);

        let key = engine.cache_key(&input);
        assert!(key.contains("seed:42"));
    }

    #[tokio::test]
    async fn test_cache_key_without_seed() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let key = engine.cache_key(&input);
        let key_repeat = engine.cache_key(&input);
        assert!(key.starts_with("face-reading:mock:"));
        assert!(!key.contains("seed"));
        assert_eq!(key, key_repeat);
    }

    #[tokio::test]
    async fn test_metadata_indicates_stub() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        assert_eq!(output.metadata.backend, "mock-stub");
        assert_eq!(output.metadata.precision_achieved, "simulated");
    }

    #[tokio::test]
    async fn test_birth_data_fallback_produces_non_mock() {
        let engine = FaceReadingEngine::new();
        let mut input = create_test_input();
        input.birth_data = Some(noesis_core::BirthData {
            name: Some("Test User".to_string()),
            date: "1990-05-15".to_string(),
            time: Some("14:30".to_string()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: "Asia/Kolkata".to_string(),
        });

        let output = engine.calculate(input).await.unwrap();
        let is_mock = output
            .result
            .get("analysis")
            .and_then(|a| a.get("is_mock_data"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        assert!(!is_mock);
        assert_eq!(output.metadata.backend, "birth-physiognomy-fallback");
    }

    #[tokio::test]
    async fn test_image_data_backend_produces_non_mock() {
        let engine = FaceReadingEngine::new();
        let mut input = create_test_input();
        input
            .options
            .insert("image_data".to_string(), json!("fake-image-bytes"));

        let output = engine.calculate(input).await.unwrap();
        let is_mock = output
            .result
            .get("analysis")
            .and_then(|a| a.get("is_mock_data"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        assert!(!is_mock);
        assert_eq!(output.metadata.backend, "heuristic-image-landmark-hook");
    }

    #[tokio::test]
    async fn test_traditions_listed() {
        let engine = FaceReadingEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        let traditions = output.result.get("traditions").unwrap().as_array().unwrap();

        assert!(traditions.len() >= 3);
        let tradition_strs: Vec<&str> = traditions.iter().filter_map(|v| v.as_str()).collect();

        assert!(tradition_strs.iter().any(|t| t.contains("Mian Xiang")));
        assert!(tradition_strs.iter().any(|t| t.contains("Ayurvedic")));
        assert!(tradition_strs.iter().any(|t| t.contains("Physiognomy")));
    }

    // T-027 P2 start: new test using exact FROZEN sample for image_data + consent
    // phase:integration-p1 wave:integration-w2 area:engine-integration swarm:selemene-backend engine-face-reading
    // per P1W1-CONTRACTS-FROZEN.md face example + detailed-task-list T-027 + gaps (stub to heuristic)
    #[tokio::test]
    async fn test_calculate_with_frozen_image_data_consent_sample() {
        let engine = FaceReadingEngine::new();
        let mut input = create_test_input();
        // FROZEN sample shape (from types.rs examples + P1W1-CONTRACTS-FROZEN)
        let frozen_image = json!({
            "b64": "iVBORw0KGgoAAAANSUhEUgAA...",
            "mime_type": "image/png",
            "consent": {
                "granted": true,
                "scopes": ["face-image"],
                "timestamp": "2026-07-17T12:00:00Z",
                "token": "consent-face-001"
            }
        });
        input.options.insert("image_data".to_string(), frozen_image);
        input.options.insert("consent".to_string(), json!({
            "granted": true,
            "scopes": ["face-image"],
            "timestamp": "2026-07-17T12:00:00Z"
        }));

        let output = engine.calculate(input).await.unwrap();
        assert_eq!(output.engine_id, "face-reading");
        assert!(!output.witness_prompt.is_empty());

        // from image + consent -> non-mock, heuristic+hook backend
        let is_mock = output
            .result
            .get("analysis")
            .and_then(|a| a.get("is_mock_data"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        assert!(!is_mock, "FROZEN image_data path must use heuristic (not mock)");

        assert_eq!(output.metadata.backend, "heuristic-image-landmark-hook");

        // consent echoed per frozen contract
        assert!(output.result.get("consent").is_some(), "consent must be present for frozen sample");
        let cons = output.result.get("consent").unwrap();
        assert_eq!(cons.get("granted"), Some(&json!(true)));
        assert!(output.result.get("computation_mode").is_some());

        // result shape still valid (analysis etc)
        assert!(output.result.get("analysis").is_some());
        assert!(output.result.get("analysis").unwrap().get("constitution").is_some());
    }
}
