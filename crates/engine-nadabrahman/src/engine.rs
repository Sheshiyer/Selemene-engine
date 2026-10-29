//! ConsciousnessEngine implementation for NadaBrahman
//!
//! Raga-based sound therapy engine. Recommends ragas based on
//! time of day, dosha constitution, mood/rasa, and chakra frequencies.

use async_trait::async_trait;
use chrono::{FixedOffset, Offset, Utc};
use chrono_tz::Tz;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::Instant;

use crate::data::{
    get_chakra_frequency, get_prahar_ragas, get_raga, get_ragas_for_dosha, get_ragas_for_rasa,
};
use crate::models::{
    ChakraFrequency, NadaBrahmanAnalysis, Prahar, PraharRecommendation, RagaRecommendation,
};
use crate::witness::generate_witness_prompt;

/// Largest accepted `options.timezone_offset`, in minutes (±18:00).
const MAX_OFFSET_MINUTES: i64 = 18 * 60;

/// The clock used to place `current_time` in a prahar.
///
/// `current_time` is a UTC instant; the prahar is a local time-of-day concept,
/// so the caller's zone decides which 3-hour block applies.
#[derive(Debug, Clone, PartialEq)]
struct TimeBasis {
    /// IANA zone name, when `options.timezone` was given
    timezone: Option<String>,
    /// Offset from UTC at `current_time`, in minutes
    offset_minutes: i32,
    /// Local clock hour (0-23) used for the prahar
    local_hour: u32,
    /// "options" when a zone was supplied, "utc_default" otherwise
    source: &'static str,
}

impl TimeBasis {
    /// Resolve the local clock from options, in order:
    /// 1) `options.timezone` (IANA name, offset resolved at the instant so DST applies)
    /// 2) `options.timezone_offset` (minutes east of UTC, e.g. 330 for IST, -300 for EST)
    /// 3) UTC
    fn resolve(input: &EngineInput) -> Result<Self, EngineError> {
        let instant = input.current_time;

        if let Some(value) = input.options.get("timezone") {
            let name = value.as_str().ok_or_else(|| {
                EngineError::ValidationError(
                    "options.timezone must be an IANA zone name string".to_string(),
                )
            })?;
            let tz: Tz = name.parse().map_err(|_| {
                EngineError::ValidationError(format!(
                    "options.timezone '{}' is not a known IANA zone",
                    name
                ))
            })?;
            let local = instant.with_timezone(&tz);
            return Ok(Self {
                timezone: Some(tz.name().to_string()),
                offset_minutes: local.offset().fix().local_minus_utc() / 60,
                local_hour: local.hour(),
                source: "options",
            });
        }

        if let Some(value) = input.options.get("timezone_offset") {
            let minutes = value
                .as_i64()
                .filter(|m| m.abs() <= MAX_OFFSET_MINUTES)
                .ok_or_else(|| {
                    EngineError::ValidationError(format!(
                        "options.timezone_offset must be whole minutes within ±{}",
                        MAX_OFFSET_MINUTES
                    ))
                })? as i32;
            let offset = FixedOffset::east_opt(minutes * 60)
                .expect("bounded by MAX_OFFSET_MINUTES, well inside ±24h");
            return Ok(Self {
                timezone: None,
                offset_minutes: minutes,
                local_hour: instant.with_timezone(&offset).hour(),
                source: "options",
            });
        }

        Ok(Self {
            timezone: None,
            offset_minutes: 0,
            local_hour: instant.hour(),
            source: "utc_default",
        })
    }

    fn to_json(&self) -> Value {
        json!({
            "timezone": self.timezone,
            "offset_minutes": self.offset_minutes,
            "local_hour": self.local_hour,
            "source": self.source,
        })
    }
}

/// NadaBrahman consciousness engine
///
/// Recommends ragas for sound therapy based on Ayurvedic and
/// Carnatic music principles. Uses time of day (prahar system),
/// dosha constitution, mood/rasa, and chakra-frequency mappings.
pub struct NadaBrahmanEngine {
    engine_id: String,
    engine_name: String,
}

impl NadaBrahmanEngine {
    pub fn new() -> Self {
        Self {
            engine_id: "nadabrahman".to_string(),
            engine_name: "NadaBrahman".to_string(),
        }
    }

    /// Perform the NadaBrahman analysis
    fn analyze(
        &self,
        input: &EngineInput,
        time_basis: &TimeBasis,
    ) -> Result<NadaBrahmanAnalysis, EngineError> {
        let prahar = Prahar::from_hour(time_basis.local_hour);

        // Get time-based recommendations
        let time_ragas = get_prahar_ragas(prahar.number());
        let primary_raga = time_ragas.first().cloned().unwrap_or(RagaRecommendation {
            raga_number: 29,
            raga_name: "Dheerasankarabharanam".to_string(),
            reason: "Universal raga (fallback)".to_string(),
            score: 0.5,
        });

        let time_recommendation = PraharRecommendation {
            prahar_name: prahar.name().to_string(),
            prahar_number: prahar.number(),
            time_range: prahar.time_range().to_string(),
            primary_raga: primary_raga.clone(),
            secondary_ragas: time_ragas.iter().skip(1).cloned().collect(),
            dosha_dominance: self.prahar_dosha(prahar).to_string(),
            energy_quality: self.prahar_energy(prahar).to_string(),
        };

        // Combine recommendations from multiple sources
        let mut all_recommendations = time_ragas;

        // Dosha-based recommendations
        let dosha = input
            .options
            .get("dosha")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let dosha_recommendation = if let Some(ref dosha_name) = dosha {
            let dosha_ragas = get_ragas_for_dosha(dosha_name);
            all_recommendations.extend(dosha_ragas);
            Some(dosha_name.clone())
        } else {
            None
        };

        // Rasa/mood-based recommendations
        let rasa = input
            .options
            .get("rasa")
            .or_else(|| input.options.get("mood"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let rasa_mapping = if let Some(ref rasa_name) = rasa {
            let rasa_ragas = get_ragas_for_rasa(rasa_name);
            all_recommendations.extend(rasa_ragas);
            Some(rasa_name.clone())
        } else {
            None
        };

        // Deduplicate and sort by score
        all_recommendations = self.deduplicate_recommendations(all_recommendations);
        all_recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_recommendations.truncate(5);

        // Chakra frequency
        let chakra_frequency = input
            .options
            .get("chakra")
            .and_then(|v| v.as_str())
            .and_then(|name| {
                get_chakra_frequency(name).map(|(solfeggio, binaural)| ChakraFrequency {
                    chakra_name: name.to_string(),
                    solfeggio_hz: solfeggio,
                    binaural_target_hz: binaural,
                })
            });

        Ok(NadaBrahmanAnalysis {
            time_recommendation,
            recommendations: all_recommendations,
            chakra_frequency,
            dosha_recommendation,
            rasa_mapping,
        })
    }

    /// Deduplicate recommendations by raga number, keeping highest score
    fn deduplicate_recommendations(
        &self,
        recs: Vec<RagaRecommendation>,
    ) -> Vec<RagaRecommendation> {
        let mut seen = std::collections::HashMap::new();
        for rec in recs {
            seen.entry(rec.raga_number)
                .and_modify(|existing: &mut RagaRecommendation| {
                    if rec.score > existing.score {
                        existing.score = rec.score;
                        existing.reason = format!("{} + {}", existing.reason, rec.reason);
                    }
                })
                .or_insert(rec);
        }
        seen.into_values().collect()
    }

    /// Get the dominant dosha for a prahar
    fn prahar_dosha(&self, prahar: Prahar) -> &'static str {
        match prahar {
            Prahar::First | Prahar::Second => "kapha",
            Prahar::Third | Prahar::Fourth => "pitta",
            Prahar::Fifth | Prahar::Sixth => "vata",
            Prahar::Seventh | Prahar::Eighth => "vata",
        }
    }

    /// Get the energy quality for a prahar
    fn prahar_energy(&self, prahar: Prahar) -> &'static str {
        match prahar {
            Prahar::First => "ascending",
            Prahar::Second => "expansive",
            Prahar::Third => "peak",
            Prahar::Fourth => "sustaining",
            Prahar::Fifth => "transitional",
            Prahar::Sixth => "inward",
            Prahar::Seventh => "deep",
            Prahar::Eighth => "awakening",
        }
    }

    /// Serialize analysis to JSON
    fn serialize_result(analysis: &NadaBrahmanAnalysis) -> Value {
        let recommendations: Vec<Value> = analysis
            .recommendations
            .iter()
            .map(|r| {
                let mut rec = json!({
                    "raga_number": r.raga_number,
                    "raga_name": r.raga_name,
                    "reason": r.reason,
                    "score": r.score,
                });

                // Enrich with raga details if available
                if let Some(raga) = get_raga(r.raga_number) {
                    rec["arohanam"] = json!(raga.arohanam);
                    rec["avarohanam"] = json!(raga.avarohanam);
                    rec["mood"] = json!(raga.mood);
                    rec["therapeutic_qualities"] = json!(raga.therapeutic_qualities);
                }

                rec
            })
            .collect();

        let mut result = json!({
            "time_recommendation": {
                "prahar_name": analysis.time_recommendation.prahar_name,
                "prahar_number": analysis.time_recommendation.prahar_number,
                "time_range": analysis.time_recommendation.time_range,
                "primary_raga": {
                    "raga_number": analysis.time_recommendation.primary_raga.raga_number,
                    "raga_name": analysis.time_recommendation.primary_raga.raga_name,
                    "reason": analysis.time_recommendation.primary_raga.reason,
                },
                "dosha_dominance": analysis.time_recommendation.dosha_dominance,
                "energy_quality": analysis.time_recommendation.energy_quality,
            },
            "recommendations": recommendations,
        });

        if let Some(ref freq) = analysis.chakra_frequency {
            result["chakra_frequency"] = json!({
                "chakra_name": freq.chakra_name,
                "solfeggio_hz": freq.solfeggio_hz,
                "binaural_target_hz": freq.binaural_target_hz,
            });
        }

        if let Some(ref dosha) = analysis.dosha_recommendation {
            result["dosha_recommendation"] = json!(dosha);
        }

        if let Some(ref rasa) = analysis.rasa_mapping {
            result["rasa_mapping"] = json!(rasa);
        }

        result
    }
}

impl Default for NadaBrahmanEngine {
    fn default() -> Self {
        Self::new()
    }
}

use chrono::Timelike;

#[async_trait]
impl ConsciousnessEngine for NadaBrahmanEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    fn required_phase(&self) -> u8 {
        0 // Available at all consciousness levels
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        let time_basis = TimeBasis::resolve(&input)?;
        let analysis = self.analyze(&input, &time_basis)?;
        let witness_prompt = generate_witness_prompt(&analysis);

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
            .unwrap_or(0);

        let elapsed = start.elapsed();
        let mut result = Self::serialize_result(&analysis);
        result["time_basis"] = time_basis.to_json();

        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result,
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: "native".to_string(),
                precision_achieved: "standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                // Deterministic local computation; no fallback branch.
                fallback_used: false,
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

        if output.result.get("time_recommendation").is_none() {
            messages.push("Missing 'time_recommendation' field".to_string());
            valid = false;
        }

        if output.result.get("recommendations").is_none() {
            messages.push("Missing 'recommendations' field".to_string());
            valid = false;
        }

        // Validate recommendations have required fields
        if let Some(recs) = output
            .result
            .get("recommendations")
            .and_then(|v| v.as_array())
        {
            if recs.is_empty() {
                messages.push("Recommendations array is empty".to_string());
                valid = false;
            }
            for (i, rec) in recs.iter().enumerate() {
                if rec.get("raga_number").is_none() {
                    messages.push(format!("Recommendation {} missing raga_number", i));
                    valid = false;
                }
                if rec.get("raga_name").is_none() {
                    messages.push(format!("Recommendation {} missing raga_name", i));
                    valid = false;
                }
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
        // Cache key based on local time (hour granularity) + time basis + options.
        // An unresolvable zone keys on the raw options; calculate() rejects it.
        let basis = match TimeBasis::resolve(input) {
            Ok(b) => format!(
                "{}:{}:{}:h{}",
                b.source,
                b.timezone.as_deref().unwrap_or("-"),
                b.offset_minutes,
                b.local_hour
            ),
            Err(_) => format!(
                "invalid:{:?}:{:?}",
                input.options.get("timezone"),
                input.options.get("timezone_offset")
            ),
        };
        let dosha = input
            .options
            .get("dosha")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        let rasa = input
            .options
            .get("rasa")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        let chakra = input
            .options
            .get("chakra")
            .and_then(|v| v.as_str())
            .unwrap_or("none");

        let raw = format!("nadabrahman:{}:d{}:r{}:c{}", basis, dosha, rasa, chakra);
        let hash = format!("{:x}", Sha256::digest(raw.as_bytes()));
        format!("nadabrahman:{}", hash)
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

    fn create_test_input_with_options(opts: Vec<(&str, Value)>) -> EngineInput {
        let mut options = HashMap::new();
        for (k, v) in opts {
            options.insert(k.to_string(), v);
        }
        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        }
    }

    /// Synthetic fixed instant with options, for time-basis tests.
    fn input_at(rfc3339: &str, opts: Vec<(&str, Value)>) -> EngineInput {
        let mut input = create_test_input_with_options(opts);
        input.current_time = chrono::DateTime::parse_from_rfc3339(rfc3339)
            .unwrap()
            .with_timezone(&Utc);
        input
    }

    async fn prahar_and_basis(input: EngineInput) -> (u64, Value) {
        let output = NadaBrahmanEngine::new().calculate(input).await.unwrap();
        let prahar = output.result["time_recommendation"]["prahar_number"]
            .as_u64()
            .unwrap();
        (prahar, output.result["time_basis"].clone())
    }

    #[tokio::test]
    async fn test_iana_kolkata_uses_local_hour() {
        // 10:00Z is 15:30 IST -> 4th prahar (15:00-18:00), not the 2nd.
        let input = input_at(
            "2026-03-10T10:00:00Z",
            vec![("timezone", json!("Asia/Kolkata"))],
        );
        let (prahar, basis) = prahar_and_basis(input).await;
        assert_eq!(prahar, 4);
        assert_eq!(basis["timezone"], json!("Asia/Kolkata"));
        assert_eq!(basis["offset_minutes"], json!(330));
        assert_eq!(basis["local_hour"], json!(15));
        assert_eq!(basis["source"], json!("options"));
    }

    #[tokio::test]
    async fn test_iana_negative_zone_follows_dst() {
        // America/New_York: 10:00Z is 05:00 EST in January (8th prahar)
        // and 06:00 EDT in July (1st prahar).
        let winter = input_at(
            "2026-01-15T10:00:00Z",
            vec![("timezone", json!("America/New_York"))],
        );
        let (prahar, basis) = prahar_and_basis(winter).await;
        assert_eq!(prahar, 8);
        assert_eq!(basis["offset_minutes"], json!(-300));
        assert_eq!(basis["local_hour"], json!(5));

        let summer = input_at(
            "2026-07-15T10:00:00Z",
            vec![("timezone", json!("America/New_York"))],
        );
        let (prahar, basis) = prahar_and_basis(summer).await;
        assert_eq!(prahar, 1);
        assert_eq!(basis["offset_minutes"], json!(-240));
        assert_eq!(basis["local_hour"], json!(6));
    }

    #[tokio::test]
    async fn test_offset_half_hour() {
        // +330 min: 10:00Z -> 15:30 local -> 4th prahar.
        let input = input_at(
            "2026-03-10T10:00:00Z",
            vec![("timezone_offset", json!(330))],
        );
        let (prahar, basis) = prahar_and_basis(input).await;
        assert_eq!(prahar, 4);
        assert_eq!(basis["timezone"], Value::Null);
        assert_eq!(basis["offset_minutes"], json!(330));
        assert_eq!(basis["local_hour"], json!(15));
        assert_eq!(basis["source"], json!("options"));
    }

    #[tokio::test]
    async fn test_offset_negative_crosses_midnight() {
        // -300 min: 02:00Z -> 21:00 the previous day -> 6th prahar.
        let input = input_at(
            "2026-03-10T02:00:00Z",
            vec![("timezone_offset", json!(-300))],
        );
        let (prahar, basis) = prahar_and_basis(input).await;
        assert_eq!(prahar, 6);
        assert_eq!(basis["offset_minutes"], json!(-300));
        assert_eq!(basis["local_hour"], json!(21));
    }

    #[tokio::test]
    async fn test_no_zone_defaults_to_utc() {
        let input = input_at("2026-03-10T10:00:00Z", vec![]);
        let (prahar, basis) = prahar_and_basis(input).await;
        assert_eq!(prahar, 2);
        assert_eq!(basis["timezone"], Value::Null);
        assert_eq!(basis["offset_minutes"], json!(0));
        assert_eq!(basis["local_hour"], json!(10));
        assert_eq!(basis["source"], json!("utc_default"));
    }

    #[tokio::test]
    async fn test_timezone_takes_precedence_over_offset() {
        let input = input_at(
            "2026-03-10T10:00:00Z",
            vec![
                ("timezone", json!("Asia/Kolkata")),
                ("timezone_offset", json!(-300)),
            ],
        );
        let (prahar, basis) = prahar_and_basis(input).await;
        assert_eq!(prahar, 4);
        assert_eq!(basis["timezone"], json!("Asia/Kolkata"));
    }

    #[tokio::test]
    async fn test_invalid_time_basis_is_rejected() {
        let engine = NadaBrahmanEngine::new();
        for opts in [
            vec![("timezone", json!("Mars/Olympus_Mons"))],
            vec![("timezone", json!(330))],
            vec![("timezone_offset", json!(5.5))],
            vec![("timezone_offset", json!(2000))],
        ] {
            let input = input_at("2026-03-10T10:00:00Z", opts.clone());
            let err = engine.calculate(input).await.unwrap_err();
            assert!(
                matches!(err, EngineError::ValidationError(_)),
                "{:?} should be a validation error, got {:?}",
                opts,
                err
            );
        }
    }

    #[test]
    fn test_cache_key_uses_time_basis() {
        let engine = NadaBrahmanEngine::new();
        let at = "2026-03-10T10:00:00Z";
        let utc = engine.cache_key(&input_at(at, vec![]));
        let ist = engine.cache_key(&input_at(at, vec![("timezone", json!("Asia/Kolkata"))]));
        let ist_offset = engine.cache_key(&input_at(at, vec![("timezone_offset", json!(330))]));
        let est = engine.cache_key(&input_at(at, vec![("timezone_offset", json!(-300))]));
        assert_ne!(utc, ist);
        assert_ne!(utc, est);
        assert_ne!(ist, est);
        // Same local hour but a different reported basis must not share an entry.
        assert_ne!(ist, ist_offset);
        // Stable for identical input.
        assert_eq!(
            ist,
            engine.cache_key(&input_at(at, vec![("timezone", json!("Asia/Kolkata"))]))
        );
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = NadaBrahmanEngine::new();
        assert_eq!(engine.engine_id(), "nadabrahman");
        assert_eq!(engine.engine_name(), "NadaBrahman");
        assert_eq!(engine.required_phase(), 0);
    }

    #[tokio::test]
    async fn test_calculate_basic() {
        let engine = NadaBrahmanEngine::new();
        let input = create_test_input();

        let result = engine.calculate(input).await;
        assert!(
            result.is_ok(),
            "Calculation should succeed: {:?}",
            result.err()
        );

        let output = result.unwrap();
        assert_eq!(output.engine_id, "nadabrahman");
        assert!(!output.witness_prompt.is_empty());
        assert!(output.result.get("time_recommendation").is_some());
        assert!(output.result.get("recommendations").is_some());
    }

    #[tokio::test]
    async fn test_calculate_with_dosha() {
        let engine = NadaBrahmanEngine::new();
        let input = create_test_input_with_options(vec![("dosha", json!("vata"))]);

        let output = engine.calculate(input).await.unwrap();
        assert_eq!(
            output
                .result
                .get("dosha_recommendation")
                .and_then(|v| v.as_str()),
            Some("vata")
        );
    }

    #[tokio::test]
    async fn test_calculate_with_rasa() {
        let engine = NadaBrahmanEngine::new();
        let input = create_test_input_with_options(vec![("rasa", json!("shanta"))]);

        let output = engine.calculate(input).await.unwrap();
        assert_eq!(
            output.result.get("rasa_mapping").and_then(|v| v.as_str()),
            Some("shanta")
        );
    }

    #[tokio::test]
    async fn test_calculate_with_chakra() {
        let engine = NadaBrahmanEngine::new();
        let input = create_test_input_with_options(vec![("chakra", json!("heart"))]);

        let output = engine.calculate(input).await.unwrap();
        let freq = output
            .result
            .get("chakra_frequency")
            .expect("Should have chakra_frequency");
        let hz = freq.get("solfeggio_hz").and_then(|v| v.as_f64()).unwrap();
        assert!(
            (hz - 639.0).abs() < 0.1,
            "Heart chakra should be 639Hz, got {}",
            hz
        );
    }

    #[tokio::test]
    async fn test_calculate_full_options() {
        let engine = NadaBrahmanEngine::new();
        let input = create_test_input_with_options(vec![
            ("dosha", json!("pitta")),
            ("rasa", json!("karuna")),
            ("chakra", json!("root")),
        ]);

        let output = engine.calculate(input).await.unwrap();
        assert!(output.result.get("dosha_recommendation").is_some());
        assert!(output.result.get("rasa_mapping").is_some());
        assert!(output.result.get("chakra_frequency").is_some());
    }

    #[tokio::test]
    async fn test_validate_output() {
        let engine = NadaBrahmanEngine::new();
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
    async fn test_validate_detects_empty_prompt() {
        let engine = NadaBrahmanEngine::new();

        let output = EngineOutput {
            engine_id: "nadabrahman".to_string(),
            result: json!({"time_recommendation": {}, "recommendations": []}),
            witness_prompt: "".to_string(),
            consciousness_level: 0,
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
    }

    #[tokio::test]
    async fn test_cache_key_varies_by_hour() {
        let engine = NadaBrahmanEngine::new();

        let input1 = create_test_input();
        let key1 = engine.cache_key(&input1);

        // Same hour should produce same key (options are the same)
        let input2 = create_test_input();
        let key2 = engine.cache_key(&input2);

        assert_eq!(key1, key2);
        assert!(key1.starts_with("nadabrahman:"));
    }

    #[tokio::test]
    async fn test_cache_key_varies_by_options() {
        let engine = NadaBrahmanEngine::new();

        let input1 = create_test_input_with_options(vec![("dosha", json!("vata"))]);
        let input2 = create_test_input_with_options(vec![("dosha", json!("pitta"))]);

        let key1 = engine.cache_key(&input1);
        let key2 = engine.cache_key(&input2);

        assert_ne!(
            key1, key2,
            "Different doshas should produce different cache keys"
        );
    }

    #[tokio::test]
    async fn test_recommendations_have_raga_details() {
        let engine = NadaBrahmanEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        let recs = output.result["recommendations"].as_array().unwrap();

        assert!(!recs.is_empty());
        let first = &recs[0];
        assert!(first.get("raga_number").is_some());
        assert!(first.get("raga_name").is_some());
        assert!(
            first.get("arohanam").is_some(),
            "Should include swara pattern"
        );
    }

    #[tokio::test]
    async fn test_witness_prompt_contains_question() {
        let engine = NadaBrahmanEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        assert!(
            output.witness_prompt.contains('?'),
            "Witness prompt should contain questions: {}",
            output.witness_prompt
        );
    }
}
