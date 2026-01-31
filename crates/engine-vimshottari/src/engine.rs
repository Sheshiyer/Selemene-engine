//! ConsciousnessEngine trait implementation for Vimshottari Dasha
//!
//! Integrates the 120-year planetary period timeline with the Noesis platform.
//! Supports two input modes:
//! 1. birth_data -> calculate Moon nakshatra -> generate full dasha timeline
//! 2. moon_longitude provided in options -> derive nakshatra directly

use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime, NaiveDateTime, TimeZone, Utc};
use noesis_core::{
    ConsciousnessEngine, EngineError, EngineInput, EngineOutput, ValidationResult,
    CalculationMetadata,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;

use crate::calculator::{
    calculate_birth_nakshatra,
    calculate_dasha_balance,
    calculate_mahadashas,
    calculate_complete_timeline,
    find_current_period,
    calculate_upcoming_transitions,
    enrich_period_with_qualities,
    get_nakshatra_from_longitude,
};
use crate::witness::generate_witness_prompt;

/// Vimshottari Dasha consciousness engine implementing the universal trait
pub struct VimshottariEngine {
    engine_id: String,
    engine_name: String,
    #[allow(dead_code)]
    hd_engine: Option<Arc<engine_human_design::HumanDesignEngine>>,
}

impl VimshottariEngine {
    /// Create a new Vimshottari engine instance
    pub fn new() -> Self {
        Self {
            engine_id: "vimshottari".to_string(),
            engine_name: "Vimshottari Dasha".to_string(),
            hd_engine: None,
        }
    }

    /// Create a new Vimshottari engine with HD engine dependency
    pub fn with_hd_engine(hd_engine: Arc<engine_human_design::HumanDesignEngine>) -> Self {
        Self {
            engine_id: "vimshottari".to_string(),
            engine_name: "Vimshottari Dasha".to_string(),
            hd_engine: Some(hd_engine),
        }
    }

    /// Parse birth_data into a UTC DateTime
    fn parse_birth_datetime(
        date_str: &str,
        time_str: Option<&str>,
    ) -> Result<chrono::DateTime<Utc>, EngineError> {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|e| EngineError::CalculationError(
                format!("Invalid date format '{}': {}", date_str, e)
            ))?;

        let time = if let Some(t) = time_str {
            NaiveTime::parse_from_str(t, "%H:%M")
                .or_else(|_| NaiveTime::parse_from_str(t, "%H:%M:%S"))
                .map_err(|e| EngineError::CalculationError(
                    format!("Invalid time format '{}': {}", t, e)
                ))?
        } else {
            NaiveTime::from_hms_opt(12, 0, 0).unwrap() // Default to noon
        };

        let naive_dt = NaiveDateTime::new(date, time);
        Ok(Utc.from_utc_datetime(&naive_dt))
    }

    /// Extract Moon longitude from options (Mode 2: direct longitude)
    fn extract_moon_longitude(
        options: &std::collections::HashMap<String, Value>,
    ) -> Result<f64, EngineError> {
        let longitude = options
            .get("moon_longitude")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| EngineError::CalculationError(
                "Missing or invalid 'moon_longitude' in options".to_string()
            ))?;

        if !(0.0..360.0).contains(&longitude) {
            return Err(EngineError::CalculationError(
                format!("moon_longitude must be 0-360, got {}", longitude)
            ));
        }

        Ok(longitude)
    }

    /// Serialize the complete timeline into a JSON Value for the EngineOutput
    fn serialize_timeline(
        birth_time: chrono::DateTime<Utc>,
        nakshatra_name: &str,
        nakshatra_number: u8,
        moon_longitude: f64,
        mahadashas: &[crate::models::Mahadasha],
        current_period: Option<&crate::models::CurrentPeriod>,
        upcoming: &[crate::models::UpcomingTransition],
        enrichment: Option<&crate::models::PeriodEnrichment>,
    ) -> Value {
        // Serialize mahadashas (top level only to keep output manageable)
        let maha_json: Vec<Value> = mahadashas.iter().map(|m| {
            json!({
                "planet": m.planet.as_str(),
                "start_date": m.start_date.to_rfc3339(),
                "end_date": m.end_date.to_rfc3339(),
                "duration_years": m.duration_years,
                "antardasha_count": m.antardashas.len(),
            })
        }).collect();

        let current_json = current_period.map(|cp| {
            json!({
                "mahadasha": {
                    "planet": cp.mahadasha.planet.as_str(),
                    "start": cp.mahadasha.start.to_rfc3339(),
                    "end": cp.mahadasha.end.to_rfc3339(),
                    "years": cp.mahadasha.years,
                },
                "antardasha": {
                    "planet": cp.antardasha.planet.as_str(),
                    "start": cp.antardasha.start.to_rfc3339(),
                    "end": cp.antardasha.end.to_rfc3339(),
                    "years": cp.antardasha.years,
                },
                "pratyantardasha": {
                    "planet": cp.pratyantardasha.planet.as_str(),
                    "start": cp.pratyantardasha.start.to_rfc3339(),
                    "end": cp.pratyantardasha.end.to_rfc3339(),
                    "days": cp.pratyantardasha.days,
                },
            })
        });

        let upcoming_json: Vec<Value> = upcoming.iter().map(|t| {
            json!({
                "type": format!("{:?}", t.transition_type),
                "from_planet": t.from_planet.as_str(),
                "to_planet": t.to_planet.as_str(),
                "date": t.transition_date.to_rfc3339(),
                "days_until": t.days_until,
            })
        }).collect();

        let enrichment_json = enrichment.map(|e| {
            json!({
                "mahadasha_themes": e.mahadasha_themes,
                "antardasha_themes": e.antardasha_themes,
                "pratyantardasha_themes": e.pratyantardasha_themes,
                "combined_description": e.combined_description,
                "life_areas": e.life_areas,
                "opportunities": e.opportunities,
                "challenges": e.challenges,
            })
        });

        json!({
            "birth_nakshatra": {
                "name": nakshatra_name,
                "number": nakshatra_number,
                "moon_longitude": moon_longitude,
            },
            "timeline": {
                "birth_date": birth_time.to_rfc3339(),
                "total_years": 120,
                "mahadashas": maha_json,
            },
            "current_period": current_json,
            "upcoming_transitions": upcoming_json,
            "period_enrichment": enrichment_json,
        })
    }
}

impl Default for VimshottariEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for VimshottariEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    fn required_phase(&self) -> u8 {
        2 // Requires deeper consciousness (same as Gene Keys)
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        // Determine Moon longitude and birth time
        let (moon_longitude, birth_time, backend) = if let Some(ref birth_data) = input.birth_data {
            // Mode 1: Calculate from birth_data using Swiss Ephemeris
            let utc_dt = Self::parse_birth_datetime(
                &birth_data.date,
                birth_data.time.as_deref(),
            )?;

            let _nakshatra = calculate_birth_nakshatra(utc_dt, "")
                .map_err(|e| EngineError::CalculationError(
                    format!("Failed to calculate birth nakshatra: {}", e)
                ))?;

            // Get precise Moon longitude from Swiss Ephemeris
            let ephe = engine_human_design::ephemeris::EphemerisCalculator::new("");
            let moon_pos = ephe.get_planet_position(
                engine_human_design::ephemeris::HDPlanet::Moon,
                &utc_dt,
            )?;

            (moon_pos.longitude, utc_dt, "swiss-ephemeris")
        } else if input.options.contains_key("moon_longitude") {
            // Mode 2: Moon longitude provided directly
            let longitude = Self::extract_moon_longitude(&input.options)?;

            // Extract birth date from options or use a default
            let date_str = input.options.get("birth_date")
                .and_then(|v| v.as_str())
                .unwrap_or("2000-01-01");
            let time_str = input.options.get("birth_time")
                .and_then(|v| v.as_str());

            let birth_time = Self::parse_birth_datetime(date_str, time_str)?;

            (longitude, birth_time, "moon-longitude")
        } else {
            return Err(EngineError::CalculationError(
                "Vimshottari requires either birth_data or moon_longitude in options".to_string()
            ));
        };

        // Step 1: Get birth nakshatra from Moon longitude
        let nakshatra = get_nakshatra_from_longitude(moon_longitude);

        // Step 2: Calculate dasha balance (remaining portion of first Mahadasha)
        let balance = calculate_dasha_balance(moon_longitude, nakshatra);

        // Step 3: Generate 9 Mahadashas
        let mahadashas = calculate_mahadashas(
            birth_time,
            nakshatra.ruling_planet,
            balance,
        );

        // Step 4: Build complete 3-level timeline (729 Pratyantardashas)
        let complete_timeline = calculate_complete_timeline(mahadashas);

        // Step 5: Find current period
        let current_time = input.current_time;
        let current_period = find_current_period(&complete_timeline, current_time);

        // Step 6: Calculate upcoming transitions
        let upcoming = calculate_upcoming_transitions(&complete_timeline, current_time, 5);

        // Step 7: Enrich current period with planetary qualities
        let enrichment = current_period.as_ref().map(|cp| {
            enrich_period_with_qualities(
                &cp.mahadasha.planet,
                &cp.antardasha.planet,
                &cp.pratyantardasha.planet,
            )
        });

        // Step 8: Get consciousness level
        let consciousness_level = input.options.get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(3); // Default to intermediate

        // Step 9: Generate witness prompt
        let witness_prompt = if let Some(ref cp) = current_period {
            generate_witness_prompt(cp, &upcoming, consciousness_level)
        } else {
            // Fallback: generate a basic prompt when current period not found
            format!(
                "Your Vimshottari timeline begins in {} nakshatra (ruled by {}). \
                 What patterns of time and consciousness are unfolding in your life?",
                nakshatra.name,
                nakshatra.ruling_planet.as_str(),
            )
        };

        // Ensure witness prompt is non-empty
        if witness_prompt.is_empty() {
            return Err(EngineError::CalculationError(
                "Witness prompt generation failed: empty result".to_string()
            ));
        }

        // Step 10: Serialize result
        let result = Self::serialize_timeline(
            birth_time,
            &nakshatra.name,
            nakshatra.number,
            moon_longitude,
            &complete_timeline,
            current_period.as_ref(),
            &upcoming,
            enrichment.as_ref(),
        );

        let elapsed = start.elapsed();

        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result,
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: backend.to_string(),
                precision_achieved: format!("{:?}", input.precision),
                cached: false,
                timestamp: Utc::now(),
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

        // Check result has timeline structure
        if output.result.get("timeline").is_none() {
            messages.push("Missing 'timeline' field in result".to_string());
            valid = false;
        }

        // Check timeline has mahadashas
        if let Some(timeline) = output.result.get("timeline") {
            if let Some(mahadashas) = timeline.get("mahadashas") {
                if let Some(arr) = mahadashas.as_array() {
                    if arr.len() != 9 {
                        messages.push(format!(
                            "Expected 9 mahadashas, found {}",
                            arr.len()
                        ));
                        valid = false;
                    }
                } else {
                    messages.push("'mahadashas' is not an array".to_string());
                    valid = false;
                }
            } else {
                messages.push("Missing 'mahadashas' in timeline".to_string());
                valid = false;
            }
        }

        // Check birth nakshatra is present
        if output.result.get("birth_nakshatra").is_none() {
            messages.push("Missing 'birth_nakshatra' field in result".to_string());
            valid = false;
        }

        // Check consciousness level is in valid range
        if output.consciousness_level > 6 {
            messages.push(format!(
                "Invalid consciousness_level: {}",
                output.consciousness_level
            ));
            valid = false;
        }

        // Check period enrichment (archetypal depth)
        if output.result.get("period_enrichment").is_none()
            || output.result.get("period_enrichment") == Some(&Value::Null)
        {
            // Enrichment may be null if current_time is outside the timeline
            // This is a warning, not a failure
            messages.push(
                "Period enrichment is null - current time may be outside timeline range"
                    .to_string(),
            );
        }

        let confidence = if valid { 1.0 } else { 0.0 };

        Ok(ValidationResult {
            valid,
            confidence,
            messages,
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        if let Some(ref birth_data) = input.birth_data {
            format!(
                "vim:{}:{}:{:.4}:{:.4}",
                birth_data.date,
                birth_data.time.as_ref().unwrap_or(&"12:00".to_string()),
                birth_data.latitude,
                birth_data.longitude
            )
        } else if let Ok(lng) = Self::extract_moon_longitude(&input.options) {
            let date = input.options.get("birth_date")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!("vim:moon:{:.6}:{}", lng, date)
        } else {
            format!("vim:invalid:{}", Utc::now().timestamp())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noesis_core::{BirthData, Precision};
    use std::collections::HashMap;

    /// Helper: create input with moon_longitude in options (Mode 2)
    fn create_test_input_with_moon_longitude(longitude: f64) -> EngineInput {
        let mut options = HashMap::new();
        options.insert(
            "moon_longitude".to_string(),
            json!(longitude),
        );
        options.insert(
            "birth_date".to_string(),
            json!("1985-06-15"),
        );
        options.insert(
            "birth_time".to_string(),
            json!("14:30"),
        );

        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        }
    }

    /// Helper: create input with birth_data (Mode 1)
    fn create_test_input_with_birth_data() -> EngineInput {
        EngineInput {
            birth_data: Some(BirthData {
                name: Some("Test User".to_string()),
                date: "1985-06-15".to_string(),
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
        let engine = VimshottariEngine::new();
        assert_eq!(engine.engine_id(), "vimshottari");
        assert_eq!(engine.engine_name(), "Vimshottari Dasha");
        assert_eq!(engine.required_phase(), 2);
    }

    #[tokio::test]
    async fn test_engine_default() {
        let engine = VimshottariEngine::default();
        assert_eq!(engine.engine_id(), "vimshottari");
    }

    #[tokio::test]
    async fn test_calculate_with_moon_longitude() {
        let engine = VimshottariEngine::new();
        let input = create_test_input_with_moon_longitude(125.0); // Magha nakshatra

        let result = engine.calculate(input).await;
        assert!(result.is_ok(), "Calculation should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert_eq!(output.engine_id, "vimshottari");
        assert!(!output.witness_prompt.is_empty(), "Witness prompt should not be empty");
        assert_eq!(output.consciousness_level, 3); // Default

        // Check timeline structure
        let timeline = output.result.get("timeline").expect("should have timeline");
        let mahadashas = timeline.get("mahadashas").expect("should have mahadashas");
        assert_eq!(mahadashas.as_array().unwrap().len(), 9);

        // Check birth nakshatra
        let nak = output.result.get("birth_nakshatra").expect("should have birth_nakshatra");
        assert_eq!(nak.get("name").unwrap().as_str().unwrap(), "Magha");
        assert_eq!(nak.get("number").unwrap().as_u64().unwrap(), 10);
    }

    #[tokio::test]
    async fn test_calculate_with_birth_data() {
        let engine = VimshottariEngine::new();
        let input = create_test_input_with_birth_data();

        let result = engine.calculate(input).await;
        assert!(result.is_ok(), "Calculation with birth_data should succeed: {:?}", result.err());

        let output = result.unwrap();
        assert_eq!(output.engine_id, "vimshottari");
        assert!(!output.witness_prompt.is_empty());

        // Timeline should have 9 mahadashas
        let timeline = output.result.get("timeline").unwrap();
        let mahadashas = timeline.get("mahadashas").unwrap().as_array().unwrap();
        assert_eq!(mahadashas.len(), 9);
    }

    #[tokio::test]
    async fn test_cache_key_with_birth_data() {
        let engine = VimshottariEngine::new();
        let input = create_test_input_with_birth_data();

        let key = engine.cache_key(&input);
        assert!(key.starts_with("vim:"), "Cache key should start with 'vim:': {}", key);
        assert!(key.contains("1985-06-15"), "Cache key should contain date");
        assert!(key.contains("14:30"), "Cache key should contain time");
    }

    #[tokio::test]
    async fn test_cache_key_with_moon_longitude() {
        let engine = VimshottariEngine::new();
        let input = create_test_input_with_moon_longitude(125.0);

        let key = engine.cache_key(&input);
        assert!(key.starts_with("vim:moon:"), "Cache key should start with 'vim:moon:': {}", key);
        assert!(key.contains("125"), "Cache key should contain longitude");
    }

    #[tokio::test]
    async fn test_cache_key_deterministic() {
        let engine = VimshottariEngine::new();
        let input1 = create_test_input_with_moon_longitude(125.0);
        let input2 = create_test_input_with_moon_longitude(125.0);

        let key1 = engine.cache_key(&input1);
        let key2 = engine.cache_key(&input2);
        assert_eq!(key1, key2, "Same inputs should produce same cache key");
    }

    #[tokio::test]
    async fn test_validation_valid_output() {
        let engine = VimshottariEngine::new();
        let input = create_test_input_with_moon_longitude(125.0);
        let output = engine.calculate(input).await.unwrap();

        let validation = engine.validate(&output).await.unwrap();
        assert!(validation.valid, "Valid output should pass validation: {:?}", validation.messages);
        assert_eq!(validation.confidence, 1.0);
    }

    #[tokio::test]
    async fn test_validation_empty_witness_prompt() {
        let engine = VimshottariEngine::new();
        let output = EngineOutput {
            engine_id: "vimshottari".to_string(),
            result: json!({
                "timeline": {
                    "mahadashas": [null, null, null, null, null, null, null, null, null]
                },
                "birth_nakshatra": { "name": "Magha", "number": 10 },
            }),
            witness_prompt: "".to_string(),
            consciousness_level: 3,
            metadata: CalculationMetadata {
                calculation_time_ms: 1.0,
                backend: "test".to_string(),
                precision_achieved: "Standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
            },
        };

        let validation = engine.validate(&output).await.unwrap();
        assert!(!validation.valid);
        assert!(validation.messages.iter().any(|m| m.contains("empty")));
    }

    #[tokio::test]
    async fn test_validation_missing_timeline() {
        let engine = VimshottariEngine::new();
        let output = EngineOutput {
            engine_id: "vimshottari".to_string(),
            result: json!({
                "birth_nakshatra": { "name": "Magha" },
            }),
            witness_prompt: "A witness prompt".to_string(),
            consciousness_level: 3,
            metadata: CalculationMetadata {
                calculation_time_ms: 1.0,
                backend: "test".to_string(),
                precision_achieved: "Standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
            },
        };

        let validation = engine.validate(&output).await.unwrap();
        assert!(!validation.valid);
        assert!(validation.messages.iter().any(|m| m.contains("timeline")));
    }

    #[tokio::test]
    async fn test_missing_input_data() {
        let engine = VimshottariEngine::new();
        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };

        let result = engine.calculate(input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("requires either"),
            "Error should mention missing input: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_invalid_moon_longitude() {
        let engine = VimshottariEngine::new();
        let mut options = HashMap::new();
        options.insert("moon_longitude".to_string(), json!(400.0)); // Invalid
        options.insert("birth_date".to_string(), json!("2000-01-01"));

        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };

        let result = engine.calculate(input).await;
        assert!(result.is_err(), "Should reject invalid moon_longitude");
    }

    #[tokio::test]
    async fn test_consciousness_level_from_options() {
        let engine = VimshottariEngine::new();
        let mut input = create_test_input_with_moon_longitude(200.0);
        input.options.insert("consciousness_level".to_string(), json!(5));

        let output = engine.calculate(input).await.unwrap();
        assert_eq!(output.consciousness_level, 5);
    }

    #[tokio::test]
    async fn test_different_nakshatras_produce_different_timelines() {
        let engine = VimshottariEngine::new();

        // Ashwini (Ketu ruled, 0-13.33 degrees)
        let input1 = create_test_input_with_moon_longitude(5.0);
        let output1 = engine.calculate(input1).await.unwrap();

        // Rohini (Moon ruled, 40-53.33 degrees)
        let input2 = create_test_input_with_moon_longitude(45.0);
        let output2 = engine.calculate(input2).await.unwrap();

        let nak1 = output1.result["birth_nakshatra"]["name"].as_str().unwrap();
        let nak2 = output2.result["birth_nakshatra"]["name"].as_str().unwrap();

        assert_ne!(nak1, nak2, "Different longitudes should produce different nakshatras");
        assert_eq!(nak1, "Ashwini");
        assert_eq!(nak2, "Rohini");

        // First mahadasha planet should differ
        let planet1 = output1.result["timeline"]["mahadashas"][0]["planet"].as_str().unwrap();
        let planet2 = output2.result["timeline"]["mahadashas"][0]["planet"].as_str().unwrap();
        assert_ne!(planet1, planet2, "Different nakshatras should have different starting planets");
    }
}
