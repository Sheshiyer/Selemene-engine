//! Reviewed input guidance; catalog metadata does not assert execution or delivery.
use serde_json::{json, Value};
pub fn requirements(id: &str) -> Value {
    let (kind, required, notes) = match id {
        "human-design" | "vimshottari" | "transits" => (
            "calculation",
            vec!["birth_date", "birth_time", "birth_location", "timezone"],
            "Qualified natal inputs; unknown birth time must remain visible",
        ),
        "gene-keys" => (
            "symbolic_mapping",
            vec!["birth_date", "birth_time", "birth_location", "timezone"],
            "Shared Human Design gate lineage; no invented line",
        ),
        "numerology" => (
            "calculation",
            vec!["full_name", "birth_date"],
            "Use explicitly supplied name spelling",
        ),
        "biorhythm" => (
            "calculation",
            vec!["birth_date", "current_time"],
            "Formula output is not a measured feeling",
        ),
        "panchanga" => (
            "calculation",
            vec!["current_time", "location"],
            "Preserve astronomical method and witnessed instant",
        ),
        "vedic-clock" | "nadabrahman" => (
            "calculation",
            vec!["current_time", "timezone"],
            "Recommendations do not establish audio delivery",
        ),
        "tarot" => (
            "symbolic_draw",
            vec!["options.question", "options.spread"],
            "Explicit user-requested cast with draw provenance",
        ),
        "i-ching" => (
            "symbolic_draw",
            vec!["options.question"],
            "Seeded three-coin or manual selection; yarrow unsupported",
        ),
        "enneagram" => (
            "assessment",
            vec!["options.answers_or_self_supplied_type"],
            "Never infer a type from birth data",
        ),
        "sigil-forge" => (
            "construction",
            vec!["options.intention"],
            "Rendering and artifact delivery require separate receipts",
        ),
        "sacred-geometry" => (
            "construction",
            vec!["options.form"],
            "Unsupported constructions remain unavailable",
        ),
        "raaga" => (
            "recommendation",
            vec!["options.selection"],
            "Audio synthesis requires separate consent and configured provider",
        ),
        "biofield" => (
            "qualified_observation",
            vec!["options.mode"],
            "Separate natal-symbolic and measured capture provenance",
        ),
        "face-reading" | "biofield-capture" => (
            "capture",
            vec!["capture", "consent"],
            "No measured output without actual capture/backend evidence",
        ),
        _ => (
            "unreviewed",
            vec![],
            "Consult engine schema; runtime registration is not input or result acceptance",
        ),
    };
    json!({"schema":"noesis.engine-input-guidance.v1","kind":kind,"required_inputs":required,"notes":notes,"supporting_runtime":matches!(id,"biofield-capture"|"financial-biosensor"),"registered":true,"callable_for_session":null,"awaiting_user_input":null,"computed":false,"artifact_delivered":false,"readiness":"requires_validated_request"})
}
