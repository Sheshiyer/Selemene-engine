//! Noesis Witness — Self-inquiry prompt generation and LLM-powered Dyad interpretation.
//!
//! Every engine output includes a witness_prompt (rule-based, always available).
//! For enterprise users, `interpret_with_llm()` calls an OpenAI-compatible model
//! with full multi-engine context to produce rich Aletheios/Pichet perspectives.

pub mod interpret;
pub mod llm;
pub mod routing;

pub use interpret::{
    interpret_with_llm, LiveBiofieldScores, RelationshipMode, WitnessContext, WitnessDyadLlm,
};
pub use routing::{partition_by_routing, routing_for_engine, RoutingMode};

/// Generate a witness prompt appropriate to the user's consciousness level (rule-based fallback).
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
