//! MVP-16: Workflow Synthesis Integration Tests
//!
//! Verifies correct synthesis behavior for all 6 workflow synthesizers:
//! 1. BirthBlueprintSynthesizer — numerology + HD + vimshottari
//! 2. DailyPracticeSynthesizer — panchanga + vedic-clock + biorhythm
//! 3. DecisionSupportSynthesis — tarot + i-ching + HD authority
//! 4. SelfInquirySynthesis — gene-keys + enneagram
//! 5. CreativeExpressionSynthesis — sigil-forge + sacred-geometry
//! 6. FullSpectrumSynthesizer — cross-engine theme detection
//!
//! Each test constructs mock engine outputs with realistic data and verifies
//! the synthesizer produces expected themes, alignments, and tensions.

use chrono::Utc;
use noesis_core::{CalculationMetadata, EngineInput, EngineOutput, Precision};
use noesis_orchestrator::workflow::synthesis::{
    CreativeExpressionSynthesis, DecisionSupportSynthesis, SelfInquirySynthesis,
};
use noesis_orchestrator::FullSpectrumResult;
use noesis_orchestrator::{
    BirthBlueprintSynthesizer, DailyPracticeSynthesizer, FullSpectrumSynthesizer, Synthesizer,
};
use serde_json::json;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn mock_output(engine_id: &str, result: serde_json::Value) -> EngineOutput {
    EngineOutput {
        engine_id: engine_id.to_string(),
        result,
        witness_prompt: format!("Witness from {}", engine_id),
        consciousness_level: 0,
        metadata: CalculationMetadata {
            calculation_time_ms: 1.0,
            backend: "mock".to_string(),
            precision_achieved: "standard".to_string(),
            cached: false,
            timestamp: Utc::now(),
            engine_version: String::new(),
        },
    }
            generated_image: None,
            generated_audio: None,
}

fn test_input() -> EngineInput {
    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

// ===========================================================================
// 1. BirthBlueprintSynthesizer Tests
// ===========================================================================

#[test]
fn birth_blueprint_life_path_1_manifestor_produces_leadership_theme() {
    let mut results = HashMap::new();

    results.insert(
        "numerology".to_string(),
        mock_output(
            "numerology",
            json!({
                "life_path": 1,
                "expression_number": 8,
                "soul_urge": 5
            }),
        ),
    );

    results.insert(
        "human-design".to_string(),
        mock_output(
            "human-design",
            json!({
                "type": "Manifestor",
                "authority": "Emotional",
                "profile": "1/3",
                "defined_centers": ["G Center", "Throat"],
                "undefined_centers": ["Head", "Ajna"]
            }),
        ),
    );

    results.insert(
        "vimshottari".to_string(),
        mock_output(
            "vimshottari",
            json!({
                "current_dasha": {
                    "mahadasha": "Sun",
                    "antardasha": "Moon",
                    "years_remaining": 4.5
                }
            }),
        ),
    );

    let synthesis = BirthBlueprintSynthesizer::synthesize(&results, &test_input());

    // Themes must be non-empty
    assert!(
        !synthesis.themes.is_empty(),
        "Birth blueprint should produce themes, got 0"
    );

    // Leadership theme must appear (Life Path 1 = Leadership, Manifestor = Leadership)
    let leadership = synthesis.themes.iter().find(|t| t.name == "Leadership");
    assert!(
        leadership.is_some(),
        "Should find 'Leadership' theme from LP 1 + Manifestor. Themes: {:?}",
        synthesis.themes.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    let leadership = leadership.unwrap();
    // Leadership should have sources from both numerology and human-design
    assert!(
        leadership.sources.len() >= 2,
        "Leadership theme should have at least 2 sources, got {:?}",
        leadership.sources
    );
    assert!(
        leadership.sources.contains(&"numerology".to_string()),
        "Leadership sources should include numerology"
    );
    assert!(
        leadership.sources.contains(&"human-design".to_string()),
        "Leadership sources should include human-design"
    );

    // Must have at least one alignment (LP 1 + Manifestor = leadership alignment)
    assert!(
        !synthesis.alignments.is_empty(),
        "LP 1 + Manifestor should produce at least one alignment"
    );

    // Summary must be non-empty
    assert!(!synthesis.summary.is_empty(), "Summary should not be empty");
}

#[test]
fn birth_blueprint_soul_urge_7_manifestor_produces_visibility_tension() {
    let mut results = HashMap::new();

    results.insert(
        "numerology".to_string(),
        mock_output(
            "numerology",
            json!({
                "life_path": 1,
                "expression_number": 8,
                "soul_urge": 7
            }),
        ),
    );

    results.insert(
        "human-design".to_string(),
        mock_output(
            "human-design",
            json!({
                "type": "Manifestor",
                "authority": "Emotional",
                "profile": "1/3",
                "defined_centers": ["G Center", "Throat"],
                "undefined_centers": []
            }),
        ),
    );

    let synthesis = BirthBlueprintSynthesizer::synthesize(&results, &test_input());

    // Soul Urge 7 (introspection) + Manifestor (visibility) should produce tension
    assert!(
        !synthesis.tensions.is_empty(),
        "Soul Urge 7 + Manifestor should produce visibility/introspection tension"
    );

    let visibility_tension = synthesis
        .tensions
        .iter()
        .find(|t| t.aspect.contains("Visibility") || t.aspect.contains("Introspection"));
    assert!(
        visibility_tension.is_some(),
        "Should find visibility/introspection tension. Tensions: {:?}",
        synthesis
            .tensions
            .iter()
            .map(|t| &t.aspect)
            .collect::<Vec<_>>()
    );

    let tension = visibility_tension.unwrap();
    assert!(
        !tension.integration_hint.is_empty(),
        "Tension should have an integration hint"
    );
}

#[test]
fn birth_blueprint_sun_dasha_g_center_alignment() {
    let mut results = HashMap::new();

    results.insert(
        "numerology".to_string(),
        mock_output(
            "numerology",
            json!({
                "life_path": 3,
                "expression_number": 5,
                "soul_urge": 2
            }),
        ),
    );

    results.insert(
        "human-design".to_string(),
        mock_output(
            "human-design",
            json!({
                "type": "Generator",
                "authority": "Sacral",
                "profile": "2/4",
                "defined_centers": ["G Center", "Sacral"],
                "undefined_centers": ["Head"]
            }),
        ),
    );

    results.insert(
        "vimshottari".to_string(),
        mock_output(
            "vimshottari",
            json!({
                "current_dasha": {
                    "mahadasha": "Sun",
                    "antardasha": "Venus",
                    "years_remaining": 3.0
                }
            }),
        ),
    );

    let synthesis = BirthBlueprintSynthesizer::synthesize(&results, &test_input());

    // Sun dasha + defined G Center should produce an identity alignment
    let identity_alignment = synthesis
        .alignments
        .iter()
        .find(|a| a.aspect.contains("Identity") || a.aspect.contains("identity"));
    assert!(
        identity_alignment.is_some(),
        "Sun dasha + G Center should produce identity alignment. Alignments: {:?}",
        synthesis
            .alignments
            .iter()
            .map(|a| &a.aspect)
            .collect::<Vec<_>>()
    );
}

#[test]
fn birth_blueprint_partial_results_still_produces_output() {
    let mut results = HashMap::new();

    // Only numerology -- no HD, no vimshottari
    results.insert(
        "numerology".to_string(),
        mock_output(
            "numerology",
            json!({
                "life_path": 9,
                "expression_number": 6,
                "soul_urge": 3
            }),
        ),
    );

    let synthesis = BirthBlueprintSynthesizer::synthesize(&results, &test_input());

    assert!(
        !synthesis.themes.is_empty(),
        "Even with partial data, numerology should produce themes"
    );
    assert!(
        !synthesis.summary.is_empty(),
        "Summary should still be generated"
    );
}

// ===========================================================================
// 2. DailyPracticeSynthesizer Tests
// ===========================================================================

#[test]
fn daily_practice_produces_activity_recommendations() {
    let mut results = HashMap::new();

    results.insert(
        "panchanga".to_string(),
        mock_output(
            "panchanga",
            json!({
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
            }),
        ),
    );

    results.insert(
        "vedic-clock".to_string(),
        mock_output(
            "vedic-clock",
            json!({
                "ghati": 25,
                "pala": 30,
                "muhurta": {
                    "name": "Abhijit",
                    "quality": "Auspicious"
                },
                "active_organ": "Heart",
                "dosha": "Pitta",
                "recommended_activity": "Important meetings"
            }),
        ),
    );

    results.insert(
        "biorhythm".to_string(),
        mock_output(
            "biorhythm",
            json!({
                "physical": 0.7,
                "emotional": 0.5,
                "intellectual": 0.3
            }),
        ),
    );

    let synthesis = DailyPracticeSynthesizer::synthesize(&results, &test_input());

    // Should have energy alignment (Nanda tithi + Auspicious muhurta + positive composite)
    let has_alignment_or_theme = !synthesis.alignments.is_empty() || !synthesis.themes.is_empty();
    assert!(
        has_alignment_or_theme,
        "Daily practice with favorable conditions should find alignments or themes"
    );

    // Summary must reference the day's conditions
    assert!(
        !synthesis.summary.is_empty(),
        "Summary should describe daily conditions"
    );
    assert!(
        synthesis.summary.contains("Shukla Panchami")
            || synthesis.summary.contains("Pitta")
            || synthesis.summary.contains("Rohini"),
        "Summary should reference at least one temporal condition. Got: {}",
        synthesis.summary
    );
}

#[test]
fn daily_practice_energy_tension_low_physical_pitta_time() {
    let mut results = HashMap::new();

    results.insert("panchanga".to_string(), mock_output("panchanga", json!({
        "tithi": { "name": "Krishna Chaturthi", "number": 4, "paksha": "Krishna" },
        "nakshatra": { "name": "Ashlesha", "number": 9, "quality": "Sharp", "deity": "Nagas" },
        "yoga": "Vajra",
        "karana": "Bava",
        "vara": "Tuesday"
    })));

    results.insert(
        "vedic-clock".to_string(),
        mock_output(
            "vedic-clock",
            json!({
                "ghati": 30,
                "pala": 0,
                "muhurta": { "name": "Madhyahna", "quality": "Neutral" },
                "active_organ": "Heart",
                "dosha": "Pitta",
                "recommended_activity": "Focused work"
            }),
        ),
    );

    results.insert(
        "biorhythm".to_string(),
        mock_output(
            "biorhythm",
            json!({
                "physical": -0.5,
                "emotional": 0.5,
                "intellectual": 0.5
            }),
        ),
    );

    let synthesis = DailyPracticeSynthesizer::synthesize(&results, &test_input());

    // Low physical (-0.5) + Pitta time should produce energy tension
    let energy_tension = synthesis
        .tensions
        .iter()
        .find(|t| t.aspect.contains("Energy") || t.aspect.contains("energy"));
    assert!(
        energy_tension.is_some(),
        "Low physical biorhythm during Pitta time should create tension. Tensions: {:?}",
        synthesis
            .tensions
            .iter()
            .map(|t| &t.aspect)
            .collect::<Vec<_>>()
    );
}

#[test]
fn daily_practice_rikta_tithi_high_biorhythm_tension() {
    let mut results = HashMap::new();

    // Rikta tithi (number 4, 9, or 14 = depleted)
    results.insert("panchanga".to_string(), mock_output("panchanga", json!({
        "tithi": { "name": "Shukla Chaturthi", "number": 4, "paksha": "Shukla" },
        "nakshatra": { "name": "Pushya", "number": 8, "quality": "Light", "deity": "Brihaspati" },
        "yoga": "Shiva",
        "karana": "Bava",
        "vara": "Wednesday"
    })));

    results.insert(
        "vedic-clock".to_string(),
        mock_output(
            "vedic-clock",
            json!({
                "ghati": 20,
                "pala": 15,
                "muhurta": { "name": "Vijaya", "quality": "Neutral" },
                "active_organ": "Liver",
                "dosha": "Kapha",
                "recommended_activity": "Physical exercise"
            }),
        ),
    );

    results.insert(
        "biorhythm".to_string(),
        mock_output(
            "biorhythm",
            json!({
                "physical": 0.8,
                "emotional": 0.6,
                "intellectual": 0.4
            }),
        ),
    );

    let synthesis = DailyPracticeSynthesizer::synthesize(&results, &test_input());

    // Rikta tithi (depleted cosmic timing) + high biorhythm composite should create tension
    let rhythm_tension = synthesis.tensions.iter().find(|t| {
        t.aspect.contains("Rhythm") || t.aspect.contains("Cosmic") || t.aspect.contains("Personal")
    });
    assert!(
        rhythm_tension.is_some(),
        "Rikta tithi + high biorhythm should produce rhythm tension. Tensions: {:?}",
        synthesis
            .tensions
            .iter()
            .map(|t| &t.aspect)
            .collect::<Vec<_>>()
    );
}

// ===========================================================================
// 3. DecisionSupportSynthesis Tests
// ===========================================================================

#[test]
fn decision_support_tarot_iching_hd_produces_themes_and_summary() {
    let mut results = HashMap::new();

    results.insert(
        "tarot".to_string(),
        mock_output(
            "tarot",
            json!({
                "spread": "THREE_CARD",
                "cards": [
                    { "name": "The Hermit", "position": "past", "arcana": "major" },
                    { "name": "The Chariot", "position": "present", "arcana": "major" },
                    { "name": "Three of Cups", "position": "future", "arcana": "minor" }
                ]
            }),
        ),
    );

    results.insert(
        "i-ching".to_string(),
        mock_output(
            "i-ching",
            json!({
                "hexagram": { "number": 5, "name": "Waiting" },
                "changing_lines": [2, 5]
            }),
        ),
    );

    results.insert(
        "human-design".to_string(),
        mock_output(
            "human-design",
            json!({
                "authority": "sacral",
                "type": "Generator"
            }),
        ),
    );

    let synthesis = DecisionSupportSynthesis::synthesize(&results, &test_input());

    // Should produce themes from all three systems
    assert!(
        !synthesis.themes.is_empty(),
        "Decision support with all 3 engines should produce themes"
    );

    // Should have major arcana theme (2 major arcana cards)
    let archetypal = synthesis
        .themes
        .iter()
        .find(|t| t.name.contains("Archetypal") || t.name.contains("archetypal"));
    assert!(
        archetypal.is_some(),
        "2 major arcana cards should produce archetypal theme. Themes: {:?}",
        synthesis.themes.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // I-Ching hexagram 5 = wait type, should produce hexagram theme
    let hex_theme = synthesis.themes.iter().find(|t| t.name.contains("Waiting"));
    assert!(
        hex_theme.is_some(),
        "I-Ching hexagram 'Waiting' should appear in themes. Themes: {:?}",
        synthesis.themes.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Summary must be non-empty
    assert!(!synthesis.summary.is_empty(), "Summary should not be empty");
}

#[test]
fn decision_support_directional_alignment_both_action() {
    let mut results = HashMap::new();

    // The Chariot = action card
    results.insert(
        "tarot".to_string(),
        mock_output(
            "tarot",
            json!({
                "spread": "SINGLE",
                "cards": [
                    { "name": "The Chariot", "position": "present", "arcana": "major" }
                ]
            }),
        ),
    );

    // Hexagram 1 = action hexagram
    results.insert(
        "i-ching".to_string(),
        mock_output(
            "i-ching",
            json!({
                "hexagram": { "number": 1, "name": "The Creative" },
                "changing_lines": []
            }),
        ),
    );

    results.insert(
        "human-design".to_string(),
        mock_output(
            "human-design",
            json!({
                "authority": "sacral",
                "type": "Generator"
            }),
        ),
    );

    let synthesis = DecisionSupportSynthesis::synthesize(&results, &test_input());

    // Both tarot (Chariot=action) and I-Ching (hex 1=action) point to action
    let directional = synthesis.alignments.iter().find(|a| {
        a.aspect.contains("Directional")
            || a.aspect.contains("Action")
            || a.aspect.contains("Body-Action")
    });
    assert!(
        directional.is_some(),
        "Chariot + Hex 1 should produce directional/action alignment. Alignments: {:?}",
        synthesis
            .alignments
            .iter()
            .map(|a| &a.aspect)
            .collect::<Vec<_>>()
    );
}

#[test]
fn decision_support_produces_witness_prompts() {
    // Test the static method for witness prompt generation
    let hd_data = json!({ "authority": "sacral", "type": "Generator" });
    let tarot_data = json!({
        "cards": [{ "name": "The Hermit", "position": "past", "arcana": "major" }]
    });
    let iching_data = json!({ "name": "Waiting" });

    let prompts =
        DecisionSupportSynthesis::generate_witness_prompts(&hd_data, &tarot_data, &iching_data);

    assert!(
        !prompts.is_empty(),
        "Should generate at least one witness prompt"
    );

    // Should have a body-awareness prompt
    let body_prompt = prompts
        .iter()
        .find(|p| p.text.contains("body") || p.text.contains("sensation"));
    assert!(
        body_prompt.is_some(),
        "Should include body-awareness prompt. Prompts: {:?}",
        prompts.iter().map(|p| &p.text).collect::<Vec<_>>()
    );
}

#[test]
fn decision_support_handles_missing_engines() {
    let mut results = HashMap::new();

    // Only tarot -- no I-Ching, no HD
    results.insert(
        "tarot".to_string(),
        mock_output(
            "tarot",
            json!({
                "spread": "SINGLE",
                "cards": [
                    { "name": "The Magician", "position": "present", "arcana": "major" }
                ]
            }),
        ),
    );

    let synthesis = DecisionSupportSynthesis::synthesize(&results, &test_input());

    // Should still produce some output with partial data
    assert!(
        !synthesis.summary.is_empty(),
        "Should produce summary even with partial data"
    );
}

// ===========================================================================
// 4. SelfInquirySynthesis Tests
// ===========================================================================

#[test]
fn self_inquiry_gene_keys_enneagram_produces_shadow_themes() {
    let mut results = HashMap::new();

    results.insert(
        "gene-keys".to_string(),
        mock_output(
            "gene-keys",
            json!({
                "spheres": {
                    "life_work": {
                        "gene_key": 55,
                        "shadow": "Victimization",
                        "gift": "Freedom",
                        "siddhi": "Freedom"
                    },
                    "evolution": {
                        "gene_key": 59,
                        "shadow": "Dishonesty",
                        "gift": "Intimacy",
                        "siddhi": "Transparency"
                    }
                }
            }),
        ),
    );

    results.insert(
        "enneagram".to_string(),
        mock_output(
            "enneagram",
            json!({
                "type": 4,
                "core_fear": "having no identity",
                "core_weakness": "envy",
                "healthy_traits": ["creative", "authentic", "compassion"],
                "integration": 1
            }),
        ),
    );

    let synthesis = SelfInquirySynthesis::synthesize(&results, &test_input());

    // Should produce themes from both systems
    assert!(
        !synthesis.themes.is_empty(),
        "Self-inquiry with both engines should produce themes"
    );

    // Should find shadow theme from Gene Keys
    let shadow_theme = synthesis
        .themes
        .iter()
        .find(|t| t.name.contains("Shadow") || t.name.contains("shadow"));
    assert!(
        shadow_theme.is_some(),
        "Should find shadow theme from Gene Keys. Themes: {:?}",
        synthesis.themes.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Should find enneagram type theme
    let enn_theme = synthesis
        .themes
        .iter()
        .find(|t| t.name.contains("Enneagram") || t.name.contains("Individualist"));
    assert!(
        enn_theme.is_some(),
        "Should find Enneagram type theme. Themes: {:?}",
        synthesis.themes.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Summary should not be empty
    assert!(
        !synthesis.summary.is_empty(),
        "Summary should describe the inquiry"
    );
}

#[test]
fn self_inquiry_shadow_fear_alignment() {
    let mut results = HashMap::new();

    // Gene Key 55 shadow = "Victimization" contains no direct mapping,
    // but envy maps to Type 4
    results.insert(
        "gene-keys".to_string(),
        mock_output(
            "gene-keys",
            json!({
                "spheres": {
                    "life_work": {
                        "gene_key": 44,
                        "shadow": "Interference",
                        "gift": "Teamwork",
                        "siddhi": "Synarchy"
                    },
                    "evolution": {
                        "gene_key": 1,
                        "shadow": "Entropy",
                        "gift": "Freshness",
                        "siddhi": "Beauty"
                    }
                }
            }),
        ),
    );

    results.insert(
        "enneagram".to_string(),
        mock_output(
            "enneagram",
            json!({
                "type": 4,
                "core_fear": "having no identity",
                "core_weakness": "envy",
                "healthy_traits": ["creative"],
                "integration": 1
            }),
        ),
    );

    let synthesis = SelfInquirySynthesis::synthesize(&results, &test_input());

    // Should produce growth tensions (shadow-gift journey + core type tension)
    assert!(
        !synthesis.tensions.is_empty(),
        "Self-inquiry should produce growth edge tensions"
    );

    // Should find the shadow-gift journey tension
    let shadow_gift = synthesis
        .tensions
        .iter()
        .find(|t| t.aspect.contains("Shadow") && t.aspect.contains("Gift"));
    assert!(
        shadow_gift.is_some(),
        "Should find Shadow-Gift Journey tension. Tensions: {:?}",
        synthesis
            .tensions
            .iter()
            .map(|t| &t.aspect)
            .collect::<Vec<_>>()
    );

    // Should find core type tension
    let core_tension = synthesis
        .tensions
        .iter()
        .find(|t| t.aspect.contains("Core Type"));
    assert!(
        core_tension.is_some(),
        "Should find Core Type Tension. Tensions: {:?}",
        synthesis
            .tensions
            .iter()
            .map(|t| &t.aspect)
            .collect::<Vec<_>>()
    );
}

#[test]
fn self_inquiry_siddhi_integration_alignment() {
    let mut results = HashMap::new();

    results.insert(
        "gene-keys".to_string(),
        mock_output(
            "gene-keys",
            json!({
                "spheres": {
                    "life_work": {
                        "gene_key": 22,
                        "shadow": "Dishonour",
                        "gift": "Graciousness",
                        "siddhi": "Grace"
                    }
                }
            }),
        ),
    );

    results.insert(
        "enneagram".to_string(),
        mock_output(
            "enneagram",
            json!({
                "type": 9,
                "core_fear": "loss of connection",
                "core_weakness": "sloth",
                "healthy_traits": ["peaceful", "accepting"],
                "integration": 3
            }),
        ),
    );

    let synthesis = SelfInquirySynthesis::synthesize(&results, &test_input());

    // With both siddhi and integration present, should find highest potential alignment
    let potential = synthesis
        .alignments
        .iter()
        .find(|a| a.aspect.contains("Potential") || a.aspect.contains("Highest"));
    assert!(
        potential.is_some(),
        "Siddhi + integration point should produce Highest Potential alignment. Alignments: {:?}",
        synthesis
            .alignments
            .iter()
            .map(|a| &a.aspect)
            .collect::<Vec<_>>()
    );
}

// ===========================================================================
// 5. CreativeExpressionSynthesis Tests
// ===========================================================================

#[test]
fn creative_expression_sigil_geometry_produces_themes() {
    let mut results = HashMap::new();

    results.insert(
        "sigil-forge".to_string(),
        mock_output(
            "sigil-forge",
            json!({
                "intention": "Manifest creative abundance in my life",
                "distilled": "CREATE ABUNDANCE",
                "method": "letter_elimination"
            }),
        ),
    );

    results.insert(
        "sacred-geometry".to_string(),
        mock_output(
            "sacred-geometry",
            json!({
                "form": "Seed of Life",
                "qualities": ["creation", "potential", "genesis"],
                "meditation": "Contemplate the seven circles"
            }),
        ),
    );

    let synthesis = CreativeExpressionSynthesis::synthesize(&results, &test_input());

    // Should produce themes from both engines
    assert!(
        !synthesis.themes.is_empty(),
        "Creative expression with both engines should produce themes"
    );

    // Should have distilled intention theme
    let intention_theme = synthesis.themes.iter().find(|t| {
        t.name.contains("Intention") || t.name.contains("intention") || t.name.contains("CREATE")
    });
    assert!(
        intention_theme.is_some(),
        "Should find distilled intention theme. Themes: {:?}",
        synthesis.themes.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Should have sacred form theme
    let form_theme = synthesis
        .themes
        .iter()
        .find(|t| t.name.contains("Seed") || t.name.contains("Form") || t.name.contains("form"));
    assert!(
        form_theme.is_some(),
        "Should find sacred form theme. Themes: {:?}",
        synthesis.themes.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Summary should be non-empty
    assert!(!synthesis.summary.is_empty(), "Summary should not be empty");
}

#[test]
fn creative_expression_keyword_quality_shared_resonance() {
    let mut results = HashMap::new();

    // "heal" intention -> restorative energy; qualities include "healing" -> shared resonance
    results.insert(
        "sigil-forge".to_string(),
        mock_output(
            "sigil-forge",
            json!({
                "intention": "Heal and restore my inner balance",
                "distilled": "RESTORE BALANCE",
                "method": "letter_elimination"
            }),
        ),
    );

    results.insert(
        "sacred-geometry".to_string(),
        mock_output(
            "sacred-geometry",
            json!({
                "form": "Seed of Life",
                "qualities": ["creation", "potential", "healing"],
                "meditation": "Seven circles of creation"
            }),
        ),
    );

    let synthesis = CreativeExpressionSynthesis::synthesize(&results, &test_input());

    // "heal" keyword in intention matches "healing" quality -> Shared Resonance alignment
    let alignment = synthesis
        .alignments
        .iter()
        .find(|a| a.aspect.contains("Resonance") || a.aspect.contains("heal"));
    assert!(
        alignment.is_some(),
        "Heal keyword + healing quality should produce shared resonance alignment. Alignments: {:?}",
        synthesis.alignments.iter().map(|a| &a.aspect).collect::<Vec<_>>()
    );

    // Verify the alignment links both engines
    let alignment = alignment.unwrap();
    assert!(
        alignment.engines.contains(&"sigil-forge".to_string()),
        "Alignment should reference sigil-forge"
    );
    assert!(
        alignment.engines.contains(&"sacred-geometry".to_string()),
        "Alignment should reference sacred-geometry"
    );
}

#[test]
fn creative_expression_formless_to_form_tension() {
    let mut results = HashMap::new();

    results.insert(
        "sigil-forge".to_string(),
        mock_output(
            "sigil-forge",
            json!({
                "intention": "Express my truth",
                "distilled": "TRUTH",
                "method": "letter_elimination"
            }),
        ),
    );

    results.insert(
        "sacred-geometry".to_string(),
        mock_output(
            "sacred-geometry",
            json!({
                "form": "Circle",
                "qualities": ["wholeness", "unity"],
                "meditation": "The perfect circle"
            }),
        ),
    );

    let synthesis = CreativeExpressionSynthesis::synthesize(&results, &test_input());

    // Should always have the Formless-to-Form tension
    let formless = synthesis
        .tensions
        .iter()
        .find(|t| t.aspect.contains("Formless") || t.aspect.contains("Form"));
    assert!(
        formless.is_some(),
        "Should find Formless to Form tension. Tensions: {:?}",
        synthesis
            .tensions
            .iter()
            .map(|t| &t.aspect)
            .collect::<Vec<_>>()
    );
}

#[test]
fn creative_expression_generates_creative_direction() {
    let sigil_data = json!({
        "available": true,
        "intention": "Expand my vision",
        "distilled": "EXPAND VISION",
        "energy": "expansive",
        "keywords": ["expand"]
    });
    let geo_data = json!({
        "available": true,
        "form": "Fibonacci Spiral",
        "qualities": ["growth", "expansion"],
        "visual_description": "A spiral expanding in golden ratio proportions"
    });

    let direction =
        CreativeExpressionSynthesis::generate_creative_direction(&sigil_data, &geo_data);

    assert!(
        direction.get("core_direction").is_some(),
        "Should produce core_direction"
    );
    assert!(
        direction.get("creative_prompt").is_some(),
        "Should produce creative_prompt"
    );

    let approaches = direction
        .get("suggested_approach")
        .and_then(|v| v.as_array());
    assert!(
        approaches.is_some() && !approaches.unwrap().is_empty(),
        "Should produce suggested approaches"
    );
}

// ===========================================================================
// 6. FullSpectrumSynthesizer Tests
// ===========================================================================

#[test]
fn full_spectrum_cross_engine_theme_detection() {
    let synthesizer = FullSpectrumSynthesizer::new();

    let mut successful_outputs = HashMap::new();

    // All three engines mention "gift" or "leadership" via category keywords
    successful_outputs.insert(
        "numerology".to_string(),
        mock_output(
            "numerology",
            json!({ "life_path": 1, "gifts": ["leadership", "creativity"] }),
        ),
    );
    successful_outputs.insert(
        "human-design".to_string(),
        mock_output(
            "human-design",
            json!({ "type": "Manifestor", "authority": "Emotional" }),
        ),
    );
    successful_outputs.insert(
        "gene-keys".to_string(),
        mock_output(
            "gene-keys",
            json!({ "shadow": "Control", "gift": "Leadership" }),
        ),
    );
    successful_outputs.insert(
        "panchanga".to_string(),
        mock_output(
            "panchanga",
            json!({ "tithi": "Shukla Panchami", "nakshatra": "Rohini" }),
        ),
    );
    successful_outputs.insert(
        "vimshottari".to_string(),
        mock_output(
            "vimshottari",
            json!({ "current_dasha": { "mahadasha": "Sun", "antardasha": "Moon" } }),
        ),
    );

    let result = FullSpectrumResult {
        execution_id: "test-full".to_string(),
        by_category: HashMap::new(),
        successful_outputs,
        failed_engines: HashMap::new(),
        total_time_ms: 100.0,
        engines_attempted: 5,
        engines_succeeded: 5,
        timestamp: Utc::now(),
    };

    let synthesis = synthesizer.synthesize(&result);

    assert_eq!(
        synthesis.engines_analyzed, 5,
        "Should analyze all 5 engines"
    );
    assert!(
        !synthesis.narrative.is_empty(),
        "Should produce a narrative"
    );

    // With 5+ engines, we should find at least some cross-engine themes
    // "gift" keyword appears in numerology (gifts array) and gene-keys (gift field)
    let all_themes: Vec<_> = synthesis
        .primary_themes
        .iter()
        .chain(synthesis.secondary_themes.iter())
        .collect();

    // Verify strength > 0 for detected themes
    for theme in &all_themes {
        assert!(
            theme.strength > 0.0,
            "Theme '{}' should have strength > 0, got {}",
            theme.theme,
            theme.strength
        );
        assert!(
            theme.sources.len() >= 2,
            "Cross-engine theme '{}' should have at least 2 sources, got {}",
            theme.theme,
            theme.sources.len()
        );
    }
}

#[test]
fn full_spectrum_primary_themes_have_witness_prompts() {
    let synthesizer = FullSpectrumSynthesizer::new().with_threshold(2);

    let mut successful_outputs = HashMap::new();

    // "path" keyword should be detected across multiple engines
    successful_outputs.insert(
        "numerology".to_string(),
        mock_output(
            "numerology",
            json!({ "life_path": 6, "path_description": "The Nurturer" }),
        ),
    );
    successful_outputs.insert(
        "human-design".to_string(),
        mock_output(
            "human-design",
            json!({ "type": "Generator", "path": "Sacral response" }),
        ),
    );
    successful_outputs.insert(
        "gene-keys".to_string(),
        mock_output(
            "gene-keys",
            json!({ "shadow": "Impatience", "gift": "Patience", "path": "Gene Keys Golden Path" }),
        ),
    );

    let result = FullSpectrumResult {
        execution_id: "test-prompts".to_string(),
        by_category: HashMap::new(),
        successful_outputs,
        failed_engines: HashMap::new(),
        total_time_ms: 50.0,
        engines_attempted: 3,
        engines_succeeded: 3,
        timestamp: Utc::now(),
    };

    let synthesis = synthesizer.synthesize(&result);

    // Every primary theme should have a witness prompt
    for theme in &synthesis.primary_themes {
        assert!(
            theme.witness_prompt.is_some(),
            "Primary theme '{}' should have a witness prompt",
            theme.theme
        );
        let prompt = theme.witness_prompt.as_ref().unwrap();
        assert!(
            !prompt.is_empty(),
            "Witness prompt for '{}' should not be empty",
            theme.theme
        );
    }
}

#[test]
fn full_spectrum_confidence_above_zero_with_results() {
    let synthesizer = FullSpectrumSynthesizer::new();

    let mut successful_outputs = HashMap::new();
    for id in &[
        "numerology",
        "human-design",
        "panchanga",
        "biorhythm",
        "gene-keys",
    ] {
        successful_outputs.insert(
            id.to_string(),
            mock_output(
                id,
                json!({
                    "type": "mock",
                    "gift": "awareness"
                }),
            ),
        );
    }

    let result = FullSpectrumResult {
        execution_id: "test-conf".to_string(),
        by_category: HashMap::new(),
        successful_outputs,
        failed_engines: HashMap::new(),
        total_time_ms: 50.0,
        engines_attempted: 5,
        engines_succeeded: 5,
        timestamp: Utc::now(),
    };

    let synthesis = synthesizer.synthesize(&result);

    assert!(
        synthesis.confidence > 0.0,
        "Confidence should be > 0 with 5 successful engines, got {}",
        synthesis.confidence
    );
    assert!(
        synthesis.confidence <= 1.0,
        "Confidence should be <= 1.0, got {}",
        synthesis.confidence
    );
}

#[test]
fn full_spectrum_empty_results_produces_fallback_narrative() {
    let synthesizer = FullSpectrumSynthesizer::new();

    let result = FullSpectrumResult {
        execution_id: "test-empty".to_string(),
        by_category: HashMap::new(),
        successful_outputs: HashMap::new(),
        failed_engines: HashMap::new(),
        total_time_ms: 0.0,
        engines_attempted: 0,
        engines_succeeded: 0,
        timestamp: Utc::now(),
    };

    let synthesis = synthesizer.synthesize(&result);

    assert_eq!(synthesis.engines_analyzed, 0);
    assert!(synthesis.primary_themes.is_empty());
    assert!(
        !synthesis.narrative.is_empty(),
        "Should have fallback narrative"
    );
}

// ===========================================================================
// Cross-cutting: Verify Synthesizer trait contract for all implementations
// ===========================================================================

#[test]
fn all_synthesizers_handle_empty_results() {
    let empty: HashMap<String, EngineOutput> = HashMap::new();
    let input = test_input();

    // Each synthesizer should handle empty results gracefully (no panic)
    let birth = BirthBlueprintSynthesizer::synthesize(&empty, &input);
    assert!(birth.themes.is_empty() || !birth.summary.is_empty());

    let daily = DailyPracticeSynthesizer::synthesize(&empty, &input);
    assert!(daily.themes.is_empty() || !daily.summary.is_empty());

    let decision = DecisionSupportSynthesis::synthesize(&empty, &input);
    assert!(!decision.summary.is_empty());

    let inquiry = SelfInquirySynthesis::synthesize(&empty, &input);
    assert!(!inquiry.summary.is_empty());

    let creative = CreativeExpressionSynthesis::synthesize(&empty, &input);
    assert!(!creative.summary.is_empty());
}

#[test]
fn all_synthesizers_produce_nonempty_summary_with_full_data() {
    let input = test_input();

    // Birth Blueprint
    let mut birth_results = HashMap::new();
    birth_results.insert(
        "numerology".to_string(),
        mock_output(
            "numerology",
            json!({
                "life_path": 5, "expression_number": 3, "soul_urge": 9
            }),
        ),
    );
    birth_results.insert(
        "human-design".to_string(),
        mock_output(
            "human-design",
            json!({
                "type": "Manifesting Generator", "authority": "Sacral", "profile": "3/5",
                "defined_centers": ["Root", "Sacral", "Throat"], "undefined_centers": ["Head"]
            }),
        ),
    );
    birth_results.insert("vimshottari".to_string(), mock_output("vimshottari", json!({
        "current_dasha": { "mahadasha": "Jupiter", "antardasha": "Saturn", "years_remaining": 5.0 }
    })));
    let birth = BirthBlueprintSynthesizer::synthesize(&birth_results, &input);
    assert!(
        !birth.summary.is_empty(),
        "Birth Blueprint summary is empty"
    );

    // Daily Practice
    let mut daily_results = HashMap::new();
    daily_results.insert("panchanga".to_string(), mock_output("panchanga", json!({
        "tithi": { "name": "Poornima", "number": 15, "paksha": "Shukla" },
        "nakshatra": { "name": "Swati", "number": 15, "quality": "Movable", "deity": "Vayu" },
        "yoga": "Siddha", "karana": "Bava", "vara": "Friday"
    })));
    daily_results.insert(
        "vedic-clock".to_string(),
        mock_output(
            "vedic-clock",
            json!({
                "ghati": 10, "pala": 45,
                "muhurta": { "name": "Brahma", "quality": "Auspicious" },
                "active_organ": "Lungs", "dosha": "Vata", "recommended_activity": "Creative work"
            }),
        ),
    );
    daily_results.insert(
        "biorhythm".to_string(),
        mock_output(
            "biorhythm",
            json!({
                "physical": 0.9, "emotional": 0.8, "intellectual": 0.7
            }),
        ),
    );
    let daily = DailyPracticeSynthesizer::synthesize(&daily_results, &input);
    assert!(!daily.summary.is_empty(), "Daily Practice summary is empty");

    // Decision Support
    let mut dec_results = HashMap::new();
    dec_results.insert(
        "tarot".to_string(),
        mock_output(
            "tarot",
            json!({
                "spread": "SINGLE",
                "cards": [{ "name": "The Sun", "position": "present", "arcana": "major" }]
            }),
        ),
    );
    dec_results.insert(
        "i-ching".to_string(),
        mock_output(
            "i-ching",
            json!({
                "hexagram": { "number": 14, "name": "Great Possession" }, "changing_lines": []
            }),
        ),
    );
    dec_results.insert(
        "human-design".to_string(),
        mock_output(
            "human-design",
            json!({
                "authority": "emotional", "type": "Projector"
            }),
        ),
    );
    let dec = DecisionSupportSynthesis::synthesize(&dec_results, &input);
    assert!(!dec.summary.is_empty(), "Decision Support summary is empty");

    // Self-Inquiry
    let mut si_results = HashMap::new();
    si_results.insert("gene-keys".to_string(), mock_output("gene-keys", json!({
        "spheres": {
            "life_work": { "gene_key": 1, "shadow": "Entropy", "gift": "Freshness", "siddhi": "Beauty" }
        }
    })));
    si_results.insert("enneagram".to_string(), mock_output("enneagram", json!({
        "type": 7, "core_fear": "being trapped in pain",
        "core_weakness": "gluttony", "healthy_traits": ["joyful", "productive"], "integration": 5
    })));
    let si = SelfInquirySynthesis::synthesize(&si_results, &input);
    assert!(!si.summary.is_empty(), "Self-Inquiry summary is empty");

    // Creative Expression
    let mut ce_results = HashMap::new();
    ce_results.insert(
        "sigil-forge".to_string(),
        mock_output(
            "sigil-forge",
            json!({
                "intention": "Transform my relationship with fear",
                "distilled": "TRANSFORM FEAR", "method": "letter_elimination"
            }),
        ),
    );
    ce_results.insert(
        "sacred-geometry".to_string(),
        mock_output(
            "sacred-geometry",
            json!({
                "form": "Merkaba", "qualities": ["transformation", "balance", "protection"],
                "meditation": "The star tetrahedron"
            }),
        ),
    );
    let ce = CreativeExpressionSynthesis::synthesize(&ce_results, &input);
    assert!(
        !ce.summary.is_empty(),
        "Creative Expression summary is empty"
    );
}
