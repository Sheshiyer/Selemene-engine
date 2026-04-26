//! Noesis Witness — Self-inquiry prompt generation for consciousness development
//!
//! Every engine output includes a witness_prompt. This crate provides
//! consciousness-level-appropriate prompt templates.

/// Generate a witness prompt appropriate to the user's consciousness level.
///
/// Levels:
/// - 0 (Dormant): Observational prompts
/// - 1 (Glimpsing): Reflective prompts
/// - 2 (Practicing): Inquiry prompts
/// - 3 (Integrated): Authorship prompts
/// - 4-5 (Embodied): Open prompts
pub fn generate_witness_prompt(engine_id: &str, level: u8, _context: &serde_json::Value) -> String {
    match level {
        0 => format!(
            "Notice what you feel when you read your {} results. No need to interpret — just observe.",
            engine_id
        ),
        1 => format!(
            "What patterns do you see in your {} reading? What feels familiar?",
            engine_id
        ),
        2 => format!(
            "Who is the one observing these {} patterns? Can you separate the observer from what is observed?",
            engine_id
        ),
        3 => format!(
            "Given what {} reveals, how might you consciously choose to respond rather than react?",
            engine_id
        ),
        _ => "What wants to emerge through you right now?".to_string(),
    }
}
