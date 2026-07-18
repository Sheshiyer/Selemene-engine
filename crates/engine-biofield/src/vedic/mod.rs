//! Vedic biofield analysis — birth-data-driven computation
//!
//! Derives biofield metrics from Vedic astrological correspondences:
//! planetary positions → dignity/strength → chakra activity → global metrics.
//!
//! This replaces mock data when birth_data is provided.

pub mod chakra_map;
pub mod planetary;
pub mod temporal;

use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use engine_human_design::EphemerisCalculator;
use noesis_core::{BirthData, EngineError};

use crate::models::{BiofieldAnalysis, BiofieldMetrics, Chakra};
use crate::wisdom::get_chakra_wisdom;

use self::chakra_map::{
    calculate_all_chakra_readings, calculate_coherence, calculate_entropy,
    calculate_fractal_dimension, calculate_symmetry, calculate_vitality_index, chakra_planets,
};
use self::planetary::{
    calculate_all_positions, calculate_natal_aspects, calculate_planet_strengths, VedicPlanet,
    VedicPosition, ZodiacSign,
};
use self::temporal::apply_temporal_modulation;

/// Vedic analysis context for interpretation and witness generation
pub struct VedicContext {
    pub strongest_planet: (VedicPlanet, ZodiacSign),
    pub weakest_planet: (VedicPlanet, ZodiacSign),
    pub dominant_chakra: Chakra,
    pub weakest_chakra: Chakra,
    /// Planet-sign pairs for interpretation text
    pub planet_signs: Vec<(VedicPlanet, ZodiacSign)>,
}

/// Result of Vedic biofield analysis
pub struct VedicAnalysisResult {
    pub analysis: BiofieldAnalysis,
    pub context: VedicContext,
}

/// Orchestrator for Vedic birth-data-driven biofield analysis
pub struct VedicBiofieldAnalyzer {
    calculator: EphemerisCalculator,
}

impl VedicBiofieldAnalyzer {
    pub fn new() -> Self {
        Self {
            calculator: EphemerisCalculator::new(""),
        }
    }

    /// Perform complete Vedic biofield analysis from birth data
    pub fn analyze(
        &self,
        birth_data: &BirthData,
        current_time: DateTime<Utc>,
    ) -> Result<VedicAnalysisResult, EngineError> {
        // 1. Parse birth datetime
        let birth_dt = parse_birth_datetime(birth_data)?;

        // 2. Calculate natal planetary positions
        let natal_positions = calculate_all_positions(&self.calculator, &birth_dt)?;

        // 3. Detect aspects between natal planets
        let natal_aspects = calculate_natal_aspects(&natal_positions);

        // 4. Calculate planetary strengths
        let strengths = calculate_planet_strengths(&natal_positions, &natal_aspects);

        // 5. Derive chakra readings
        let chakra_readings = calculate_all_chakra_readings(&strengths, &natal_aspects);

        // 6. Compute global metrics
        let fractal_dimension = calculate_fractal_dimension(&natal_positions, &natal_aspects);
        let entropy = calculate_entropy(&natal_positions);
        let coherence = calculate_coherence(&natal_aspects);
        let symmetry = calculate_symmetry(&natal_positions, &strengths);
        let vitality_index =
            calculate_vitality_index(fractal_dimension, entropy, coherence, symmetry);

        // 7. Build base metrics
        let mut metrics = BiofieldMetrics {
            fractal_dimension,
            entropy,
            coherence,
            symmetry,
            vitality_index,
            chakra_readings,
            timestamp: current_time,
        };

        // 8. Apply temporal modulation from current transits
        let transit_positions = calculate_all_positions(&self.calculator, &current_time)?;
        apply_temporal_modulation(
            &mut metrics,
            &transit_positions,
            &natal_positions,
            &current_time,
        );

        // Recalculate vitality after modulation
        metrics.vitality_index = calculate_vitality_index(
            metrics.fractal_dimension,
            metrics.entropy,
            metrics.coherence,
            metrics.symmetry,
        );

        // 9. Build context for interpretation
        let context = build_vedic_context(&natal_positions, &strengths, &metrics);

        // 10. Generate interpretation and areas of attention
        let interpretation = generate_vedic_interpretation(&metrics, &context);
        let areas_of_attention = identify_vedic_areas_of_attention(&metrics, &context);

        Ok(VedicAnalysisResult {
            analysis: BiofieldAnalysis {
                metrics,
                interpretation,
                areas_of_attention,
                is_mock_data: false,
                // T-026: capture mapping placeholder (consent/quality for frozen); keep None here for Vedic birth path
                consent: None,
                quality: None,
            },
            context,
        })
    }
}

impl Default for VedicBiofieldAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse BirthData into a DateTime<Utc>
fn parse_birth_datetime(birth_data: &BirthData) -> Result<DateTime<Utc>, EngineError> {
    let date = NaiveDate::parse_from_str(&birth_data.date, "%Y-%m-%d")
        .map_err(|e| EngineError::ValidationError(format!("Invalid date: {}", e)))?;

    let time_str = birth_data.time.as_deref().unwrap_or("12:00");
    let time = NaiveTime::parse_from_str(time_str, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(time_str, "%H:%M:%S"))
        .map_err(|e| EngineError::ValidationError(format!("Invalid time: {}", e)))?;

    let naive_dt = date.and_time(time);
    Ok(Utc.from_utc_datetime(&naive_dt))
}

/// Build VedicContext from analysis data
fn build_vedic_context(
    positions: &[VedicPosition],
    strengths: &[planetary::PlanetStrength],
    metrics: &BiofieldMetrics,
) -> VedicContext {
    // Find strongest and weakest planets
    let strongest = strengths
        .iter()
        .max_by(|a, b| a.total.partial_cmp(&b.total).unwrap())
        .unwrap();
    let weakest = strengths
        .iter()
        .min_by(|a, b| a.total.partial_cmp(&b.total).unwrap())
        .unwrap();

    let strongest_sign = positions
        .iter()
        .find(|p| p.planet == strongest.planet)
        .map(|p| p.sign)
        .unwrap_or(ZodiacSign::Aries);
    let weakest_sign = positions
        .iter()
        .find(|p| p.planet == weakest.planet)
        .map(|p| p.sign)
        .unwrap_or(ZodiacSign::Aries);

    // Find dominant and weakest chakras
    let dominant_chakra = metrics
        .chakra_readings
        .iter()
        .max_by(|a, b| a.activity_level.partial_cmp(&b.activity_level).unwrap())
        .map(|r| r.chakra)
        .unwrap_or(Chakra::Heart);
    let weakest_chakra = metrics
        .chakra_readings
        .iter()
        .min_by(|a, b| a.activity_level.partial_cmp(&b.activity_level).unwrap())
        .map(|r| r.chakra)
        .unwrap_or(Chakra::Root);

    let planet_signs: Vec<_> = positions.iter().map(|p| (p.planet, p.sign)).collect();

    VedicContext {
        strongest_planet: (strongest.planet, strongest_sign),
        weakest_planet: (weakest.planet, weakest_sign),
        dominant_chakra,
        weakest_chakra,
        planet_signs,
    }
}

/// Generate interpretation text using Vedic planetary context
fn generate_vedic_interpretation(metrics: &BiofieldMetrics, context: &VedicContext) -> String {
    let mut parts = Vec::new();

    // Vitality assessment
    if metrics.vitality_index > 0.7 {
        parts.push("Overall vitality is strong".to_string());
    } else if metrics.vitality_index < 0.4 {
        parts.push("Vitality may benefit from attention".to_string());
    } else {
        parts.push("Vitality is in a moderate range".to_string());
    }

    // Planetary influence
    let (strong_planet, strong_sign) = &context.strongest_planet;
    parts.push(format!(
        "{} in {} provides the strongest planetary influence in your biofield",
        strong_planet, strong_sign
    ));

    // Dominant chakra
    let (primary, _secondary) = chakra_planets(context.dominant_chakra);
    if let Some(wisdom) = get_chakra_wisdom(context.dominant_chakra) {
        parts.push(format!(
            "Your {} shows the highest activity, supported by {} energy",
            wisdom.name, primary
        ));
    }

    // Coherence assessment
    if metrics.coherence > 0.7 {
        parts.push("Coherence is notably high, suggesting aligned energy flow".to_string());
    } else if metrics.coherence < 0.4 {
        parts.push("Coherence patterns suggest potential for more integration".to_string());
    }

    // Symmetry assessment
    if metrics.symmetry < 0.5 {
        parts.push("Left-right balance shows some asymmetry worth observing".to_string());
    }

    parts.join(". ") + "."
}

/// Identify areas that may benefit from attention using Vedic context
fn identify_vedic_areas_of_attention(
    metrics: &BiofieldMetrics,
    context: &VedicContext,
) -> Vec<String> {
    let mut areas = Vec::new();

    // Check each chakra
    for reading in &metrics.chakra_readings {
        if reading.activity_level < 0.4 {
            let (primary, _) = chakra_planets(reading.chakra);
            if let Some(wisdom) = get_chakra_wisdom(reading.chakra) {
                areas.push(format!(
                    "{} ({}) - lower activity linked to {} placement",
                    wisdom.name, wisdom.location, primary
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

    // Overall metric alerts
    if metrics.coherence < 0.4 {
        areas.push("Overall coherence may benefit from grounding practices".to_string());
    }
    if metrics.entropy < 0.3 {
        areas.push("Energy patterns appear uniform - gentle movement may help".to_string());
    }

    // Weakest planet info
    let (weak_planet, weak_sign) = &context.weakest_planet;
    areas.push(format!(
        "{} in {} shows lower strength - related chakras may need attention",
        weak_planet, weak_sign
    ));

    if areas.is_empty() {
        areas.push("No specific areas require immediate attention".to_string());
    }

    areas
}

/// Generate Vedic-aware witness prompt that references specific findings
pub fn generate_vedic_witness_prompt(
    analysis: &BiofieldAnalysis,
    context: &VedicContext,
) -> String {
    let mut prompts = Vec::new();

    // General body awareness
    prompts.push(
        "What sensations do you notice in your body right now? \
         Without trying to change anything, simply observe what is present."
            .to_string(),
    );

    // Dominant chakra prompt
    let chakra_location = chakra_body_location(context.dominant_chakra);
    let (primary, _) = chakra_planets(context.dominant_chakra);
    prompts.push(format!(
        "Your {} chakra shows strong {} energy today. \
         What sensations do you notice in {}?",
        context.dominant_chakra.name(),
        primary,
        chakra_location
    ));

    // Weakest chakra prompt
    let weak_location = chakra_body_location(context.weakest_chakra);
    prompts.push(format!(
        "Bring gentle attention to {}. \
         What is present there? If this area could speak, what might it want to express?",
        weak_location
    ));

    // Integrative prompt based on vitality
    if analysis.metrics.vitality_index > 0.7 {
        prompts.push(
            "As you hold awareness of your whole body, \
             what feels most alive? What is your body celebrating right now?"
                .to_string(),
        );
    } else if analysis.metrics.vitality_index < 0.4 {
        prompts.push(
            "What does your body need most right now? \
             Not what you think it should need, but what it is actually asking for."
                .to_string(),
        );
    } else {
        prompts.push(
            "Taking in your body as a whole, \
             what is the overall felt sense? \
             If you could describe it in one word or image, what would it be?"
                .to_string(),
        );
    }

    prompts.join(" ")
}

/// Map chakra to body location description
fn chakra_body_location(chakra: Chakra) -> &'static str {
    match chakra {
        Chakra::Root => "the base of your spine",
        Chakra::Sacral => "your lower belly, below your navel",
        Chakra::SolarPlexus => "your solar plexus area",
        Chakra::Heart => "the center of your chest",
        Chakra::Throat => "your throat and neck",
        Chakra::ThirdEye => "the space between your eyebrows",
        Chakra::Crown => "the top of your head",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noesis_core::BirthData;

    fn test_birth_data() -> BirthData {
        BirthData {
            name: Some("Test".to_string()),
            date: "1990-01-01".to_string(),
            time: Some("14:30".to_string()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: "Asia/Kolkata".to_string(),
        }
    }

    #[test]
    fn test_parse_birth_datetime() {
        let bd = test_birth_data();
        let dt = parse_birth_datetime(&bd);
        assert!(dt.is_ok(), "Should parse birth datetime: {:?}", dt.err());
    }

    #[test]
    fn test_vedic_analysis_produces_valid_result() {
        let analyzer = VedicBiofieldAnalyzer::new();
        let bd = test_birth_data();
        let now = Utc::now();

        let result = analyzer.analyze(&bd, now);
        assert!(
            result.is_ok(),
            "Vedic analysis should succeed: {:?}",
            result.err()
        );

        let result = result.unwrap();
        let analysis = &result.analysis;

        // Not mock data
        assert!(!analysis.is_mock_data);

        // Metrics in valid ranges
        let m = &analysis.metrics;
        assert!(m.fractal_dimension >= 1.0 && m.fractal_dimension <= 2.0);
        assert!(m.entropy >= 0.0 && m.entropy <= 1.0);
        assert!(m.coherence >= 0.0 && m.coherence <= 1.0);
        assert!(m.symmetry >= 0.0 && m.symmetry <= 1.0);
        assert!(m.vitality_index >= 0.0 && m.vitality_index <= 1.0);

        // 7 chakra readings
        assert_eq!(m.chakra_readings.len(), 7);
        for reading in &m.chakra_readings {
            assert!(reading.activity_level >= 0.0 && reading.activity_level <= 1.0);
            assert!(reading.balance >= -1.0 && reading.balance <= 1.0);
            assert!(!reading.color_intensity.is_empty());
        }

        // Interpretation is non-empty
        assert!(!analysis.interpretation.is_empty());
        assert!(!analysis.areas_of_attention.is_empty());
    }

    #[test]
    fn test_vedic_analysis_is_deterministic() {
        let analyzer = VedicBiofieldAnalyzer::new();
        let bd = test_birth_data();
        let fixed_time = Utc::now();

        let result1 = analyzer.analyze(&bd, fixed_time).unwrap();
        let result2 = analyzer.analyze(&bd, fixed_time).unwrap();

        assert_eq!(
            result1.analysis.metrics.fractal_dimension, result2.analysis.metrics.fractal_dimension,
            "Same input should produce same fractal_dimension"
        );
        assert_eq!(
            result1.analysis.metrics.entropy, result2.analysis.metrics.entropy,
            "Same input should produce same entropy"
        );
    }

    #[test]
    fn test_vedic_analysis_varies_with_time() {
        let analyzer = VedicBiofieldAnalyzer::new();
        let bd = test_birth_data();

        // Two different "current times" — 6 months apart
        let time1 = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let time2 = chrono::Utc.with_ymd_and_hms(2026, 7, 1, 12, 0, 0).unwrap();

        let result1 = analyzer.analyze(&bd, time1).unwrap();
        let result2 = analyzer.analyze(&bd, time2).unwrap();

        // Temporal modulation should produce different metrics for different times
        // At least one metric should differ (transit positions change significantly in 6 months)
        let differs = (result1.analysis.metrics.coherence - result2.analysis.metrics.coherence)
            .abs()
            > 0.001
            || (result1.analysis.metrics.vitality_index - result2.analysis.metrics.vitality_index)
                .abs()
                > 0.001;

        assert!(differs, "Different times should produce different metrics");
    }

    #[test]
    fn test_vedic_witness_prompt_non_empty() {
        let analyzer = VedicBiofieldAnalyzer::new();
        let bd = test_birth_data();
        let result = analyzer.analyze(&bd, Utc::now()).unwrap();

        let prompt = generate_vedic_witness_prompt(&result.analysis, &result.context);
        assert!(!prompt.is_empty());
        assert!(prompt.contains("?"), "Should contain questions");
    }

    #[test]
    fn test_vedic_context_has_valid_data() {
        let analyzer = VedicBiofieldAnalyzer::new();
        let bd = test_birth_data();
        let result = analyzer.analyze(&bd, Utc::now()).unwrap();

        let ctx = &result.context;
        assert!(!ctx.planet_signs.is_empty());
        assert_eq!(ctx.planet_signs.len(), 9, "Should have 9 planet-sign pairs");
    }
}
