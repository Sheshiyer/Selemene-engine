//! ConsciousnessEngine implementation for Transit Analysis
//!
//! Calculates natal and transit planetary positions using Swiss Ephemeris,
//! detects aspects, Sade Sati status, and generates witness prompts.

use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use engine_human_design::EphemerisCalculator;
use noesis_core::{
    BirthData, CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::time::Instant;

use crate::aspects::{calculate_aspects, significant_aspects};
use crate::ephemeris::calculate_all_positions;
use crate::models::{AspectNature, PeriodQuality, TransitAnalysisResult, TransitPlanet};
use crate::sade_sati::detect_sade_sati;
use crate::witness::generate_witness_prompt;

/// Transit analysis consciousness engine
///
/// Calculates current planetary transits relative to natal positions
/// and generates consciousness-level tiered witness prompts.
pub struct TransitsEngine {
    engine_id: String,
    engine_name: String,
    calculator: EphemerisCalculator,
}

impl TransitsEngine {
    pub fn new() -> Self {
        Self {
            engine_id: "transits".to_string(),
            engine_name: "Planetary Transits".to_string(),
            calculator: EphemerisCalculator::new(""),
        }
    }

    /// Parse BirthData into a DateTime<Utc>
    fn parse_birth_datetime(birth_data: &BirthData) -> Result<chrono::DateTime<Utc>, EngineError> {
        let date = NaiveDate::parse_from_str(&birth_data.date, "%Y-%m-%d")
            .map_err(|e| EngineError::ValidationError(format!("Invalid date: {}", e)))?;

        let time_str = birth_data.time.as_deref().unwrap_or("12:00");
        let time = NaiveTime::parse_from_str(time_str, "%H:%M")
            .or_else(|_| NaiveTime::parse_from_str(time_str, "%H:%M:%S"))
            .map_err(|e| EngineError::ValidationError(format!("Invalid time: {}", e)))?;

        let naive_dt = date.and_time(time);
        Ok(Utc.from_utc_datetime(&naive_dt))
    }

    /// Perform transit analysis
    fn analyze(&self, input: &EngineInput) -> Result<TransitAnalysisResult, EngineError> {
        // Calculate natal positions (requires birth_data)
        let birth_data = input.birth_data.as_ref().ok_or_else(|| {
            EngineError::ValidationError("Transit analysis requires birth_data".to_string())
        })?;
        let birth_dt = Self::parse_birth_datetime(birth_data)?;

        let natal_positions = calculate_all_positions(&self.calculator, &birth_dt)?;

        // Calculate current transit positions
        let transit_time = input.current_time;
        let transit_positions = calculate_all_positions(&self.calculator, &transit_time)?;

        // Calculate aspects between transits and natal
        let all_aspects = calculate_aspects(&transit_positions, &natal_positions);
        let aspects = significant_aspects(&all_aspects);

        // Detect Sade Sati
        let sade_sati = detect_sade_sati(&transit_positions, &natal_positions);

        // Detect retrograde planets
        let retrograde_planets: Vec<TransitPlanet> = transit_positions
            .iter()
            .filter(|p| p.is_retrograde)
            .map(|p| p.planet)
            .collect();

        // Assess period quality
        let period_quality = assess_period_quality(&aspects, &sade_sati, &retrograde_planets);

        Ok(TransitAnalysisResult {
            natal_positions,
            transit_positions,
            aspects,
            sade_sati,
            period_quality,
            retrograde_planets,
        })
    }

    /// Serialize analysis result to JSON
    fn serialize_result(analysis: &TransitAnalysisResult) -> serde_json::Value {
        let natal: Vec<_> = analysis
            .natal_positions
            .iter()
            .map(|p| {
                json!({
                    "planet": p.planet,
                    "longitude": (p.longitude * 1000.0).round() / 1000.0,
                    "sign": p.sign.name(),
                    "degree_in_sign": (p.degree_in_sign * 100.0).round() / 100.0,
                    "is_retrograde": p.is_retrograde,
                })
            })
            .collect();

        let transits: Vec<_> = analysis
            .transit_positions
            .iter()
            .map(|p| {
                json!({
                    "planet": p.planet,
                    "longitude": (p.longitude * 1000.0).round() / 1000.0,
                    "sign": p.sign.name(),
                    "degree_in_sign": (p.degree_in_sign * 100.0).round() / 100.0,
                    "speed": (p.speed * 10000.0).round() / 10000.0,
                    "is_retrograde": p.is_retrograde,
                })
            })
            .collect();

        let aspects: Vec<_> = analysis
            .aspects
            .iter()
            .map(|a| {
                json!({
                    "transiting_planet": a.transiting_planet.name(),
                    "natal_planet": a.natal_planet.name(),
                    "aspect_type": a.aspect_type.to_string(),
                    "orb": (a.orb * 100.0).round() / 100.0,
                    "is_applying": a.is_applying,
                    "nature": format!("{:?}", a.nature),
                })
            })
            .collect();

        let retrogrades: Vec<_> = analysis
            .retrograde_planets
            .iter()
            .map(|p| p.name())
            .collect();

        json!({
            "natal_positions": natal,
            "transit_positions": transits,
            "aspects": aspects,
            "sade_sati": {
                "is_active": analysis.sade_sati.is_active,
                "phase": analysis.sade_sati.phase.as_ref().map(|p| format!("{}", p)),
                "saturn_sign": analysis.sade_sati.saturn_sign.name(),
                "moon_sign": analysis.sade_sati.moon_sign.name(),
            },
            "period_quality": analysis.period_quality.to_string(),
            "retrograde_planets": retrogrades,
        })
    }
}

impl Default for TransitsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for TransitsEngine {
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

        let analysis = self.analyze(&input)?;

        let consciousness_level = input
            .options
            .get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(1);

        let witness_prompt = generate_witness_prompt(&analysis, consciousness_level);

        if witness_prompt.is_empty() {
            return Err(EngineError::CalculationError(
                "Witness prompt generation failed: empty result".to_string(),
            ));
        }

        let result = Self::serialize_result(&analysis);
        let elapsed = start.elapsed();

        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result,
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: "swiss-ephemeris".to_string(),
                precision_achieved: format!("{:?}", input.precision),
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

        if output.result.get("natal_positions").is_none() {
            messages.push("Missing natal_positions".to_string());
            valid = false;
        }

        if output.result.get("transit_positions").is_none() {
            messages.push("Missing transit_positions".to_string());
            valid = false;
        }

        if output.result.get("aspects").is_none() {
            messages.push("Missing aspects".to_string());
            valid = false;
        }

        if output.result.get("sade_sati").is_none() {
            messages.push("Missing sade_sati".to_string());
            valid = false;
        }

        if output.result.get("period_quality").is_none() {
            messages.push("Missing period_quality".to_string());
            valid = false;
        }

        // Check natal positions count
        if let Some(natal) = output
            .result
            .get("natal_positions")
            .and_then(|v| v.as_array())
        {
            if natal.len() != 12 {
                messages.push(format!("Expected 12 natal positions, got {}", natal.len()));
                valid = false;
            }
        }

        // Check transit positions count
        if let Some(transits) = output
            .result
            .get("transit_positions")
            .and_then(|v| v.as_array())
        {
            if transits.len() != 12 {
                messages.push(format!(
                    "Expected 12 transit positions, got {}",
                    transits.len()
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
        let birth_str = input
            .birth_data
            .as_ref()
            .map(|b| format!("{}_{}", b.date, b.time.as_deref().unwrap_or("12:00")))
            .unwrap_or_default();

        // Cache with hour granularity for transits
        let transit_hour = input.current_time.format("%Y-%m-%d-%H").to_string();

        let raw = format!("transits:{}:{}", birth_str, transit_hour);
        let hash = format!("{:x}", Sha256::digest(raw.as_bytes()));
        format!("transits:{}", hash)
    }
}

/// Assess overall period quality from aspects, Sade Sati, and retrogrades.
fn assess_period_quality(
    aspects: &[crate::models::TransitAspect],
    sade_sati: &crate::models::SadeSatiStatus,
    retrograde_planets: &[TransitPlanet],
) -> PeriodQuality {
    let mut score: i32 = 0;

    // Aspect scoring
    for aspect in aspects {
        match aspect.nature {
            AspectNature::Harmonious => score += 2,
            AspectNature::Challenging => score -= 2,
            AspectNature::Neutral => score += 1,
        }
    }

    // Sade Sati is challenging
    if sade_sati.is_active {
        score -= 3;
    }

    // Many retrogrades signal a reflective/slower period
    let retrograde_count = retrograde_planets.len();
    if retrograde_count >= 4 {
        score -= 2;
    } else if retrograde_count >= 2 {
        score -= 1;
    }

    match score {
        6.. => PeriodQuality::HighlyFavorable,
        3..=5 => PeriodQuality::Favorable,
        -2..=2 => PeriodQuality::Mixed,
        -5..=-3 => PeriodQuality::Challenging,
        _ => PeriodQuality::Difficult,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SadeSatiStatus, ZodiacSign};

    #[test]
    fn test_engine_creation() {
        let engine = TransitsEngine::new();
        assert_eq!(engine.engine_id(), "transits");
        assert_eq!(engine.engine_name(), "Planetary Transits");
        assert_eq!(engine.required_phase(), 0);
    }

    #[test]
    fn test_period_quality_favorable() {
        let aspects = vec![];
        let sade_sati = SadeSatiStatus {
            is_active: false,
            phase: None,
            saturn_sign: ZodiacSign::Aries,
            moon_sign: ZodiacSign::Aries,
        };
        let retrogrades = vec![];

        let quality = assess_period_quality(&aspects, &sade_sati, &retrogrades);
        assert_eq!(quality, PeriodQuality::Mixed);
    }

    #[test]
    fn test_period_quality_with_sade_sati() {
        let aspects = vec![];
        let sade_sati = SadeSatiStatus {
            is_active: true,
            phase: Some(crate::models::SadeSatiPhase::Peak),
            saturn_sign: ZodiacSign::Aquarius,
            moon_sign: ZodiacSign::Aquarius,
        };
        let retrogrades = vec![TransitPlanet::Mercury, TransitPlanet::Saturn];

        let quality = assess_period_quality(&aspects, &sade_sati, &retrogrades);
        assert!(
            matches!(
                quality,
                PeriodQuality::Challenging | PeriodQuality::Difficult
            ),
            "Sade Sati + retrogrades should be challenging: {:?}",
            quality
        );
    }
}
