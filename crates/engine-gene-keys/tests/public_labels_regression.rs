//! Regression anchors captured from official per-key public headings on 2026-10-08.
//! These fixtures cover labels and coordinates, not licensed author meanings.

use engine_gene_keys::wisdom::{gene_keys, get_gene_key, get_gene_key_provenance};
use engine_gene_keys::{ConsciousnessEngine, EngineInput, GeneKeysEngine};
use serde_json::{json, Value};

const LABELS: [(u8, &str, &str, &str); 64] = [
    (1, "Entropy", "Freshness", "Beauty"),
    (2, "Dislocation", "Orientation", "Unity"),
    (3, "Chaos", "Innovation", "Innocence"),
    (4, "Intolerance", "Understanding", "Forgiveness"),
    (5, "Impatience", "Patience", "Timelessness"),
    (6, "Conflict", "Diplomacy", "Peace"),
    (7, "Division", "Guidance", "Virtue"),
    (8, "Mediocrity", "Style", "Exquisiteness"),
    (9, "Inertia", "Determination", "Invincibility"),
    (10, "Self Obsession", "Naturalness", "Being"),
    (11, "Obscurity", "Idealism", "Light"),
    (12, "Vanity", "Discrimination", "Purity"),
    (13, "Discord", "Discernment", "Empathy"),
    (14, "Compromise", "Competence", "Bounteousness"),
    (15, "Dullness", "Magnetism", "Florescence"),
    (16, "Indifference", "Versatility", "Mastery"),
    (17, "Opinion", "Far-sightedness", "Omniscience"),
    (18, "Judgment", "Integrity", "Perfection"),
    (19, "Co-dependence", "Sensitivity", "Sacrifice"),
    (20, "Superficiality", "Self Assurance", "Presence"),
    (21, "Control", "Authority", "Valor"),
    (22, "Dishonor", "Graciousness", "Grace"),
    (23, "Complexity", "Simplicity", "Quintessence"),
    (24, "Addiction", "Invention", "Silence"),
    (25, "Constriction", "Acceptance", "Universal Love"),
    (26, "Pride", "Artfulness", "Invisibility"),
    (27, "Selfishness", "Altruism", "Selflessness"),
    (28, "Purposelessness", "Totality", "Immortality"),
    (29, "Half-Heartedness", "Commitment", "Devotion"),
    (30, "Desire", "Lightness", "Rapture"),
    (31, "Arrogance", "Leadership", "Humility"),
    (32, "Failure", "Preservation", "Veneration"),
    (33, "Forgetting", "Mindfulness", "Revelation"),
    (34, "Force", "Strength", "Majesty"),
    (35, "Hunger", "Adventure", "Boundlessness"),
    (36, "Turbulence", "Humanity", "Compassion"),
    (37, "Weakness", "Equality", "Tenderness"),
    (38, "Struggle", "Perseverance", "Honor"),
    (39, "Provocation", "Dynamism", "Liberation"),
    (40, "Exhaustion", "Resolve", "Divine Will"),
    (41, "Fantasy", "Anticipation", "Emanation"),
    (42, "Expectation", "Detachment", "Celebration"),
    (43, "Deafness", "Insight", "Epiphany"),
    (44, "Interference", "Teamwork", "Synarchy"),
    (45, "Dominance", "Synergy", "Communion"),
    (46, "Seriousness", "Delight", "Ecstasy"),
    (47, "Oppression", "Transmutation", "Transfiguration"),
    (48, "Inadequacy", "Resourcefulness", "Wisdom"),
    (49, "Reaction", "Revolution", "Rebirth"),
    (50, "Corruption", "Equilibrium", "Harmony"),
    (51, "Agitation", "Initiative", "Awakening"),
    (52, "Stress", "Restraint", "Stillness"),
    (53, "Immaturity", "Expansion", "Superabundance"),
    (54, "Greed", "Aspiration", "Ascension"),
    (55, "Victimisation", "Freedom", "Freedom"),
    (56, "Distraction", "Enrichment", "Intoxication"),
    (57, "Unease", "Intuition", "Clarity"),
    (58, "Dissatisfaction", "Vitality", "Bliss"),
    (59, "Dishonesty", "Intimacy", "Transparency"),
    (60, "Limitation", "Realism", "Justice"),
    (61, "Psychosis", "Inspiration", "Sanctity"),
    (62, "Intellect", "Precision", "Impeccability"),
    (63, "Doubt", "Inquiry", "Truth"),
    (64, "Confusion", "Imagination", "Illumination"),
];

// Independently recorded Rave Mandala order, with opposition at offset32.
const WHEEL: [u8; 64] = [
    17, 21, 51, 42, 3, 27, 24, 2, 23, 8, 20, 16, 35, 45, 12, 15, 52, 39, 53, 62, 56, 31, 33, 7, 4,
    29, 59, 40, 64, 47, 6, 46, 18, 48, 57, 32, 50, 28, 44, 1, 43, 14, 34, 9, 5, 26, 11, 10, 58, 38,
    54, 61, 60, 41, 19, 13, 49, 30, 55, 37, 63, 22, 36, 25,
];

#[test]
fn all64_labels_match_reviewed_primary_heading_snapshot() {
    assert_eq!(gene_keys().len(), 64);
    for (number, shadow, gift, siddhi) in LABELS {
        let key = get_gene_key(number).unwrap();
        assert_eq!(
            (key.shadow.as_str(), key.gift.as_str(), key.siddhi.as_str()),
            (shadow, gift, siddhi),
            "key {number}"
        );
        assert_eq!(key.name, format!("Gene Key {number}"));
        assert!(key.codon.is_none() && key.amino_acid.is_none() && key.physiology.is_none());
        assert!(
            key.shadow_description.is_empty()
                && key.gift_description.is_empty()
                && key.siddhi_description.is_empty()
        );
        assert!(key.life_theme.is_none());
    }
}

#[test]
fn partners_are_complete_involutions_and_two_public_link_conflicts_are_retained() {
    let mut agreements = 0;
    let mut conflicts = Vec::new();
    for (position, number) in WHEEL.iter().copied().enumerate() {
        let key = get_gene_key(number).unwrap();
        let expected = WHEEL[(position + 32) % 64];
        assert_eq!(key.programming_partner, Some(expected));
        assert_ne!(expected, number);
        assert_eq!(
            get_gene_key(expected).unwrap().programming_partner,
            Some(number)
        );
        let source = get_gene_key_provenance(number).unwrap();
        assert_eq!(
            source["url"],
            format!("https://genekeys.com/gene-key-{number}/")
        );
        if source["public_page_partner_coordinate"] == expected {
            agreements += 1;
        } else {
            conflicts.push((
                number,
                source["public_page_partner_coordinate"].as_u64().unwrap(),
                expected,
            ));
            assert_eq!(
                source["programming_partner_status"],
                "official_page_link_conflict_resolved_by_opposition"
            );
        }
    }
    conflicts.sort_unstable();
    assert_eq!(agreements, 62);
    assert_eq!(conflicts, vec![(6, 2, 36), (8, 8, 14)]);
}

#[test]
fn source_contract_does_not_claim_licensed_full_wisdom_or_biology() {
    let source: Value = serde_json::from_str(include_str!(
        "../../../data/gene-keys/verified-public-labels.json"
    ))
    .unwrap();
    assert_eq!(source["metadata"]["full_wisdom_status"], "unavailable");
    assert_eq!(
        source["metadata"]["rights"]["official_corpus_licence_claimed"],
        false
    );
    assert_eq!(
        source["metadata"]["programming_partner_source"]["public_page_agreements"],
        62
    );
    assert_eq!(
        source["metadata"]["programming_partner_source"]["public_page_conflicts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn api_payload_exposes_verified_labels_null_author_meanings_and_original_questions() {
    let engine = GeneKeysEngine::new();
    let mut input = EngineInput {
        birth_data: None,
        current_time: chrono::Utc::now(),
        location: None,
        precision: noesis_core::Precision::Standard,
        options: std::collections::HashMap::new(),
    };
    input.options.insert(
        "hd_gates".into(),
        json!({"personality_sun":24,"personality_earth":44,"design_sun":64,"design_earth":63}),
    );
    let result = engine.calculate(input).await.unwrap().result;
    assert_eq!(
        result["meaning_source_quality"]["status"],
        "verified_public_labels_only"
    );
    assert_eq!(
        result["meaning_source_quality"]["full_wisdom"],
        "unavailable"
    );
    assert_eq!(
        result["frequency_assessment_context"]["measurement_status"],
        "not_measured"
    );
    assert_eq!(result["activation_spheres"]["lifes_work"]["key_number"], 24);
    assert_eq!(result["activation_spheres"]["radiance"]["key_number"], 64);
    for key in result["active_keys"].as_array().unwrap() {
        assert!(key["line"].is_null());
        assert_eq!(key["full_meanings_status"], "unavailable");
        assert_eq!(key["meaning_source_quality"], "verified_public_labels_only");
        assert!(key["source_provenance"]["url"]
            .as_str()
            .unwrap()
            .starts_with("https://genekeys.com/gene-key-"));
    }
    for item in result["frequency_assessments"].as_array().unwrap() {
        for field in [
            "shadow_description",
            "gift_description",
            "siddhi_description",
        ] {
            assert!(item[field].is_null());
        }
        assert_eq!(
            item["description_status"],
            "unavailable_authorized_meanings_not_supplied"
        );
        assert_eq!(
            item["recognition_prompts_source"],
            "selemene_original_questions_not_author_text"
        );
        for band in ["shadow", "gift", "siddhi"] {
            assert!(!item["recognition_prompts"][band]
                .as_array()
                .unwrap()
                .is_empty());
        }
    }
}
