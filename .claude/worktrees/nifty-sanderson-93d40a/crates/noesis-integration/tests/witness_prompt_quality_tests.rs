//! Witness prompt quality contract tests for v3.0.0 launch.
//!
//! Enforces cross-engine style rules:
//! - Question format (`?` suffix)
//! - Non-prescriptive language (no "you should" / "you must")
//! - Non-trivial length

use std::collections::HashMap;

fn sample_prompts_by_engine() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        (
            "panchanga",
            "What quality of attention does today's lunar rhythm invite in your decisions?",
        ),
        (
            "numerology",
            "How does your life-path pattern want to be expressed with greater integrity today?",
        ),
        (
            "biorhythm",
            "What shifts when you honor your current energy cycle instead of pushing against it?",
        ),
        (
            "human-design",
            "Where does your strategy and authority feel most trustworthy in this moment?",
        ),
        (
            "gene-keys",
            "Which shadow reaction is ready to be observed as a doorway to your gift?",
        ),
        (
            "vimshottari",
            "How is your current dasha period shaping the lesson you are being asked to embody?",
        ),
        (
            "biofield",
            "What sensation in your body is asking for kind, non-judging awareness right now?",
        ),
        (
            "vedic-clock",
            "What action aligns with the present organ-time current and your inner pacing?",
        ),
        (
            "face-reading",
            "What does your present expression reveal about the quality of self-contact you are living?",
        ),
        (
            "nadabrahman",
            "Which tone or raga helps you return to centered listening in this phase?",
        ),
        (
            "transits",
            "What transit pressure feels like an invitation to mature your response rather than react?",
        ),
        (
            "tarot",
            "What symbol in this spread mirrors the question your deeper self is holding?",
        ),
        (
            "i-ching",
            "What movement from this hexagram becomes clear when you read it as reflection?",
        ),
        (
            "enneagram",
            "What pattern of fixation is softening as you observe it without defense?",
        ),
        (
            "sacred-geometry",
            "What inner order becomes visible when you contemplate this form with patience?",
        ),
        (
            "sigil-forge",
            "What intention is becoming more coherent as you encode it into living symbol?",
        ),
    ])
}

fn assert_prompt_contract(engine_id: &str, prompt: &str) {
    let normalized = prompt.to_lowercase();
    let banned = ["you should", "you must"];

    assert!(
        prompt.trim_end().ends_with('?'),
        "{engine_id}: witness prompt must end with '?' -- got: {prompt}"
    );

    assert!(
        prompt.trim().len() >= 24,
        "{engine_id}: witness prompt too short for reflective depth -- got: {prompt}"
    );

    for phrase in banned {
        assert!(
            !normalized.contains(phrase),
            "{engine_id}: witness prompt contains banned prescriptive phrase '{phrase}' -- got: {prompt}"
        );
    }
}

#[test]
fn witness_prompt_quality_contract_covers_all_16_engines() {
    let prompts = sample_prompts_by_engine();

    let expected_engine_ids = [
        "biofield",
        "biorhythm",
        "enneagram",
        "face-reading",
        "gene-keys",
        "human-design",
        "i-ching",
        "nadabrahman",
        "numerology",
        "panchanga",
        "sacred-geometry",
        "sigil-forge",
        "tarot",
        "transits",
        "vedic-clock",
        "vimshottari",
    ];

    assert_eq!(
        prompts.len(),
        expected_engine_ids.len(),
        "Prompt contract must include all 16 engines"
    );

    for engine_id in expected_engine_ids {
        let prompt = prompts
            .get(engine_id)
            .unwrap_or_else(|| panic!("Missing prompt entry for engine: {engine_id}"));
        assert_prompt_contract(engine_id, prompt);
    }
}
