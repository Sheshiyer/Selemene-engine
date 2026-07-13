//! Engine → dyad pillar routing, ported from witness-agents/src/types/engine.ts.

use serde_json::Value;

/// Which pillar/dyad voice should lead interpretation for an engine result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingMode {
    /// Aletheios-primary: analytical, truth-revealing.
    AletheiosPrimary,
    /// Pichet-primary: somatic, vitalizing.
    PichetPrimary,
    /// Dyad-synthesis: co-interpretation.
    DyadSynthesis,
}

/// Map a Selemene engine id to its routing mode.
pub fn routing_for_engine(engine_id: &str) -> Option<RoutingMode> {
    match engine_id {
        "vimshottari" | "human-design" | "enneagram" | "i-ching" | "numerology" => {
            Some(RoutingMode::AletheiosPrimary)
        }
        "biorhythm" | "vedic-clock" | "biofield" | "face-reading" | "nadabrahman" => {
            Some(RoutingMode::PichetPrimary)
        }
        "panchanga" | "gene-keys" | "tarot" | "sacred-geometry" | "sigil-forge" | "transits" => {
            Some(RoutingMode::DyadSynthesis)
        }
        _ => None,
    }
}

/// Partition a list of engine results by routing mode.
pub fn partition_by_routing(
    results: &[(String, Value)],
) -> (Vec<&Value>, Vec<&Value>, Vec<&Value>) {
    let mut aletheios = vec![];
    let mut pichet = vec![];
    let mut dyad = vec![];
    for (engine_id, value) in results {
        match routing_for_engine(engine_id) {
            Some(RoutingMode::AletheiosPrimary) => aletheios.push(value),
            Some(RoutingMode::PichetPrimary) => pichet.push(value),
            Some(RoutingMode::DyadSynthesis) | None => dyad.push(value),
        }
    }
    (aletheios, pichet, dyad)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn routing_for_engine_maps_known_ids() {
        assert_eq!(
            routing_for_engine("vimshottari"),
            Some(RoutingMode::AletheiosPrimary)
        );
        assert_eq!(
            routing_for_engine("biofield"),
            Some(RoutingMode::PichetPrimary)
        );
        assert_eq!(
            routing_for_engine("panchanga"),
            Some(RoutingMode::DyadSynthesis)
        );
        assert_eq!(routing_for_engine("unknown"), None);
    }

    #[test]
    fn partition_by_routing_groups_results() {
        let results = vec![
            ("vimshottari".to_string(), json!({"a": 1})),
            ("biofield".to_string(), json!({"b": 2})),
            ("panchanga".to_string(), json!({"c": 3})),
        ];
        let (a, p, d) = partition_by_routing(&results);
        assert_eq!(a.len(), 1);
        assert_eq!(p.len(), 1);
        assert_eq!(d.len(), 1);
        assert_eq!(a[0]["a"], 1);
        assert_eq!(p[0]["b"], 2);
        assert_eq!(d[0]["c"], 3);
    }
}
