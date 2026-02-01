//! Daily Practice Synthesis — Combine temporal recommendations
//!
//! Finds optimal windows where all three systems agree:
//! - Panchanga: Tithi, Nakshatra, Yoga quality
//! - Vedic Clock: Dosha time, Muhurta quality
//! - Biorhythm: Physical, emotional, intellectual cycles

use super::Synthesizer;
use crate::workflow::daily_practice::{BiorhythmData, PanchangaData, VedicClockData};
use crate::workflow::models::{
    Alignment, SynthesisResult as ExtSynthesisResult, Tension, Theme,
};

// Re-export for trait impl
pub type SynthesisResult = ExtSynthesisResult;
use noesis_core::{EngineInput, EngineOutput};
use std::collections::HashMap;

/// Synthesizer for Daily Practice workflow
pub struct DailyPracticeSynthesizer;

impl Synthesizer for DailyPracticeSynthesizer {
    fn synthesize(
        results: &HashMap<String, EngineOutput>,
        _input: &EngineInput,
    ) -> SynthesisResult {
        // Extract data from each engine
        let panchanga = results
            .get("panchanga")
            .and_then(|o| PanchangaData::from_json(&o.result));
        
        let vedic_clock = results
            .get("vedic-clock")
            .and_then(|o| VedicClockData::from_json(&o.result));
        
        let biorhythm = results
            .get("biorhythm")
            .and_then(|o| BiorhythmData::from_json(&o.result));

        let mut themes = Vec::new();
        let mut alignments = Vec::new();
        let mut tensions = Vec::new();

        // Analyze current state across systems
        let mut activity_recommendations: HashMap<String, Vec<String>> = HashMap::new();

        // Collect activities from each system
        if let Some(ref panch) = panchanga {
            for activity in panch.recommended_activities() {
                activity_recommendations
                    .entry(categorize_activity(&activity))
                    .or_default()
                    .push(format!("Panchanga: {}", activity));
            }
        }

        if let Some(ref vc) = vedic_clock {
            for activity in vc.optimal_activities() {
                activity_recommendations
                    .entry(categorize_activity(&activity))
                    .or_default()
                    .push(format!("Vedic Clock: {}", activity));
            }
        }

        if let Some(ref bio) = biorhythm {
            for (activity, _score) in bio.activity_fit() {
                activity_recommendations
                    .entry(categorize_activity(&activity))
                    .or_default()
                    .push(format!("Biorhythm: {}", activity));
            }
        }

        // Create themes from activities that appear in multiple systems
        for (category, sources) in &activity_recommendations {
            if sources.len() >= 2 {
                let theme = Theme::new(
                    category.clone(),
                    format!("Multiple systems support {}", category.to_lowercase())
                ).with_sources(
                    sources.iter()
                        .map(|s| s.split(':').next().unwrap_or("unknown").to_string())
                        .collect()
                );
                themes.push(theme);
            }
        }

        // Find alignments
        if let Some(alignment) = find_energy_alignment(&panchanga, &vedic_clock, &biorhythm) {
            alignments.push(alignment);
        }

        if let Some(alignment) = find_activity_alignment(&panchanga, &vedic_clock) {
            alignments.push(alignment);
        }

        // Find tensions
        if let Some(tension) = find_energy_tension(&biorhythm, &vedic_clock) {
            tensions.push(tension);
        }

        if let Some(tension) = find_rhythm_tension(&panchanga, &biorhythm) {
            tensions.push(tension);
        }

        // Generate summary
        let summary = generate_daily_summary(&panchanga, &vedic_clock, &biorhythm, &themes, &alignments, &tensions);

        SynthesisResult {
            themes,
            alignments,
            tensions,
            summary,
        }
    }
}

/// Categorize an activity into broad themes
fn categorize_activity(activity: &str) -> String {
    let lower = activity.to_lowercase();
    
    if lower.contains("physical") || lower.contains("exercise") || lower.contains("movement") || lower.contains("yoga") {
        "Physical Activity".to_string()
    } else if lower.contains("creative") || lower.contains("expression") || lower.contains("art") {
        "Creative Expression".to_string()
    } else if lower.contains("learn") || lower.contains("study") || lower.contains("mental") || lower.contains("focus") {
        "Mental Work".to_string()
    } else if lower.contains("social") || lower.contains("relationship") || lower.contains("connect") || lower.contains("meeting") {
        "Social Connection".to_string()
    } else if lower.contains("spiritual") || lower.contains("meditation") || lower.contains("practice") {
        "Spiritual Practice".to_string()
    } else if lower.contains("rest") || lower.contains("recovery") || lower.contains("easy") {
        "Rest & Recovery".to_string()
    } else if lower.contains("decision") || lower.contains("important") || lower.contains("venture") {
        "Important Decisions".to_string()
    } else if lower.contains("beginning") || lower.contains("start") || lower.contains("new") {
        "New Beginnings".to_string()
    } else {
        "General Activity".to_string()
    }
}

/// Find alignment in energy levels
fn find_energy_alignment(
    panchanga: &Option<PanchangaData>,
    vedic_clock: &Option<VedicClockData>,
    biorhythm: &Option<BiorhythmData>,
) -> Option<Alignment> {
    let mut high_energy_systems = Vec::new();

    if let Some(ref panch) = panchanga {
        // Auspicious tithis and nakshatras indicate high energy
        if panch.tithi.quality.contains("Nanda") || panch.tithi.quality.contains("Bhadra") {
            high_energy_systems.push("panchanga".to_string());
        }
    }

    if let Some(ref vc) = vedic_clock {
        // Auspicious muhurta indicates good timing
        if vc.muhurta_quality.to_lowercase().contains("auspicious") {
            high_energy_systems.push("vedic-clock".to_string());
        }
    }

    if let Some(ref bio) = biorhythm {
        // Positive composite indicates good overall state
        if bio.composite > 0.3 {
            high_energy_systems.push("biorhythm".to_string());
        }
    }

    if high_energy_systems.len() >= 2 {
        Some(
            Alignment::new(
                "Favorable timing",
                "Multiple systems indicate supportive conditions for action"
            )
            .with_engines(high_energy_systems)
            .with_confidence(0.8)
        )
    } else {
        None
    }
}

/// Find alignment in activity recommendations
fn find_activity_alignment(
    panchanga: &Option<PanchangaData>,
    vedic_clock: &Option<VedicClockData>,
) -> Option<Alignment> {
    let panch_activities = panchanga.as_ref().map(|p| p.recommended_activities()).unwrap_or_default();
    let vc_activities = vedic_clock.as_ref().map(|v| v.optimal_activities()).unwrap_or_default();

    // Check for overlapping activity types
    let panch_lower: Vec<String> = panch_activities.iter().map(|a| a.to_lowercase()).collect();
    let vc_lower: Vec<String> = vc_activities.iter().map(|a| a.to_lowercase()).collect();

    let mut common = Vec::new();
    for pa in &panch_lower {
        for va in &vc_lower {
            if categories_overlap(pa, va) {
                common.push(categorize_activity(pa));
            }
        }
    }

    if !common.is_empty() {
        Some(
            Alignment::new(
                format!("Activity alignment: {}", common[0]),
                "Panchanga and Vedic Clock agree on optimal activities"
            )
            .with_engines(vec!["panchanga".to_string(), "vedic-clock".to_string()])
            .with_confidence(0.75)
        )
    } else {
        None
    }
}

/// Check if two activity descriptions have overlapping categories
fn categories_overlap(a: &str, b: &str) -> bool {
    let category_a = categorize_activity(a);
    let category_b = categorize_activity(b);
    category_a == category_b && category_a != "General Activity"
}

/// Find tension between biorhythm state and vedic clock recommendation
fn find_energy_tension(
    biorhythm: &Option<BiorhythmData>,
    vedic_clock: &Option<VedicClockData>,
) -> Option<Tension> {
    let bio = biorhythm.as_ref()?;
    let vc = vedic_clock.as_ref()?;

    // Physical low but Pitta time (supposed to be active)
    if bio.physical.value < -0.3 && vc.dosha_time.to_lowercase() == "pitta" {
        return Some(
            Tension::new(
                "Energy vs Timing",
                "Your physical biorhythm is low during an active time period"
            )
            .with_perspectives(
                "biorhythm",
                format!("Physical cycle at {:.1} - rest recommended", bio.physical.value),
                "vedic-clock",
                "Pitta time suggests focused activity"
            )
            .with_integration_hint(
                "Honor your body's need for gentleness while staying mentally engaged. \
                Light mental work or contemplation may bridge both needs."
            )
        );
    }

    // Intellectual low but recommending mental work
    if bio.intellectual.value < -0.3 && vc.recommended_activity.to_lowercase().contains("mental") {
        return Some(
            Tension::new(
                "Mental Capacity vs Task",
                "Intellectual cycle low during recommended mental work period"
            )
            .with_perspectives(
                "biorhythm",
                format!("Intellectual cycle at {:.1} - routine tasks preferred", bio.intellectual.value),
                "vedic-clock",
                format!("Recommends: {}", vc.recommended_activity)
            )
            .with_integration_hint(
                "Use this time for review and consolidation rather than new learning. \
                Your subconscious continues processing even when surface clarity is low."
            )
        );
    }

    None
}

/// Find tension between panchanga quality and biorhythm state
fn find_rhythm_tension(
    panchanga: &Option<PanchangaData>,
    biorhythm: &Option<BiorhythmData>,
) -> Option<Tension> {
    let panch = panchanga.as_ref()?;
    let bio = biorhythm.as_ref()?;

    // Rikta tithi (depleted) but high biorhythm
    if panch.tithi.quality.contains("Rikta") && bio.composite > 0.5 {
        return Some(
            Tension::new(
                "Personal vs Cosmic Rhythm",
                "High personal energy during depleted cosmic timing"
            )
            .with_perspectives(
                "panchanga",
                format!("{} is a Rikta tithi - not ideal for new ventures", panch.tithi.name),
                "biorhythm",
                format!("Your cycles show {:.1} composite energy", bio.composite)
            )
            .with_integration_hint(
                "Channel your high energy into completion and clearing rather than initiation. \
                Great day for finishing projects, resolving issues, or deep cleaning."
            )
        );
    }

    // Auspicious day but critical biorhythm
    let is_auspicious = panch.tithi.quality.contains("Nanda") || panch.tithi.quality.contains("Bhadra");
    if is_auspicious && bio.has_critical_day() {
        return Some(
            Tension::new(
                "Cosmic Support vs Personal Threshold",
                "Auspicious timing meets critical day in your cycles"
            )
            .with_perspectives(
                "panchanga",
                format!("{} - generally favorable for action", panch.tithi.name),
                "biorhythm",
                "Critical day - heightened sensitivity and variability"
            )
            .with_integration_hint(
                "Critical days are pivot points. The auspicious timing may help navigate \
                the transition. Stay present and make decisions with extra awareness."
            )
        );
    }

    None
}

/// Generate daily summary
fn generate_daily_summary(
    panchanga: &Option<PanchangaData>,
    vedic_clock: &Option<VedicClockData>,
    biorhythm: &Option<BiorhythmData>,
    themes: &[Theme],
    alignments: &[Alignment],
    tensions: &[Tension],
) -> String {
    let mut parts = Vec::new();

    // Panchanga context
    if let Some(ref panch) = panchanga {
        parts.push(format!(
            "Today is {} ({}) with {} Nakshatra",
            panch.tithi.name,
            panch.tithi.quality,
            panch.nakshatra.name
        ));
    }

    // Current time state
    if let Some(ref vc) = vedic_clock {
        parts.push(format!(
            "Currently in {} time with {} muhurta",
            vc.dosha_time, vc.muhurta_quality
        ));
    }

    // Biorhythm state
    if let Some(ref bio) = biorhythm {
        let state = if bio.composite > 0.3 {
            "elevated"
        } else if bio.composite < -0.3 {
            "low"
        } else {
            "balanced"
        };
        parts.push(format!("Your biorhythm composite is {} ({:.1})", state, bio.composite));
    }

    // Supported activities
    let strong_themes: Vec<&str> = themes
        .iter()
        .filter(|t| t.strength >= 0.4)
        .take(2)
        .map(|t| t.name.as_str())
        .collect();
    
    if !strong_themes.is_empty() {
        parts.push(format!(
            "Multiple systems support: {}",
            strong_themes.join(", ")
        ));
    }

    // Alignments
    if !alignments.is_empty() {
        parts.push(format!(
            "Favorable conditions: {}",
            alignments.iter().map(|a| a.aspect.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    // Tensions to navigate
    if !tensions.is_empty() {
        parts.push(format!(
            "Navigate: {}",
            tensions.iter().map(|t| t.aspect.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    parts.join(". ") + "."
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mock_output(engine_id: &str, result: serde_json::Value) -> EngineOutput {
        EngineOutput {
            engine_id: engine_id.to_string(),
            result,
            witness_prompt: String::new(),
            consciousness_level: 0,
            metadata: noesis_core::CalculationMetadata {
                calculation_time_ms: 1.0,
                backend: "mock".to_string(),
                precision_achieved: "standard".to_string(),
                cached: false,
                timestamp: chrono::Utc::now(),
            },
        }
    }

    #[test]
    fn synthesize_daily_practice() {
        let mut results = HashMap::new();
        
        results.insert("panchanga".to_string(), mock_output("panchanga", json!({
            "tithi": {
                "name": "Shukla Panchami",
                "number": 5,
                "paksha": "Shukla"
            },
            "nakshatra": {
                "name": "Rohini",
                "number": 4,
                "quality": "Fixed",
                "deity": "Brahma"
            },
            "yoga": "Shiva",
            "karana": "Bava",
            "vara": "Thursday"
        })));
        
        results.insert("vedic-clock".to_string(), mock_output("vedic-clock", json!({
            "ghati": 25,
            "pala": 30,
            "muhurta": {
                "name": "Abhijit",
                "quality": "Auspicious"
            },
            "active_organ": "Heart",
            "dosha": "Pitta",
            "recommended_activity": "Important meetings"
        })));
        
        results.insert("biorhythm".to_string(), mock_output("biorhythm", json!({
            "physical": 0.7,
            "emotional": 0.5,
            "intellectual": 0.3
        })));

        let input = EngineInput {
            birth_data: None,
            current_time: chrono::Utc::now(),
            location: None,
            precision: noesis_core::Precision::Standard,
            options: HashMap::new(),
        };

        let synthesis = DailyPracticeSynthesizer::synthesize(&results, &input);

        // Should have found some alignments
        assert!(!synthesis.alignments.is_empty() || !synthesis.themes.is_empty());
        
        // Summary should not be empty
        assert!(!synthesis.summary.is_empty());
    }

    #[test]
    fn categorize_activities() {
        assert_eq!(categorize_activity("Physical exercise"), "Physical Activity");
        assert_eq!(categorize_activity("Creative expression"), "Creative Expression");
        assert_eq!(categorize_activity("Learning new skills"), "Mental Work");
        assert_eq!(categorize_activity("Rest and recovery"), "Rest & Recovery");
    }

    #[test]
    fn find_tension_low_physical_pitta() {
        let bio = Some(BiorhythmData {
            physical: crate::workflow::daily_practice::CycleData {
                value: -0.5,
                phase: "Low".to_string(),
                description: "Low energy".to_string(),
            },
            emotional: crate::workflow::daily_practice::CycleData {
                value: 0.5,
                phase: "High".to_string(),
                description: String::new(),
            },
            intellectual: crate::workflow::daily_practice::CycleData {
                value: 0.5,
                phase: "High".to_string(),
                description: String::new(),
            },
            composite: 0.17,
        });

        let vc = Some(VedicClockData {
            current_ghati: 30,
            current_pala: 0,
            current_muhurta: "Madhyahna".to_string(),
            muhurta_quality: "Neutral".to_string(),
            active_organ: "Heart".to_string(),
            dosha_time: "Pitta".to_string(),
            recommended_activity: "Focused work".to_string(),
        });

        let tension = find_energy_tension(&bio, &vc);
        assert!(tension.is_some());
        assert!(tension.unwrap().aspect.contains("Energy"));
    }
}
