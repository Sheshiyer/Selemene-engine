//! Static raga data loading
//!
//! Loads raga data from embedded JSON. The data files are compiled into the binary
//! to avoid runtime filesystem dependencies.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::models::{Raga, RagaRecommendation};

/// Embedded raga database (compiled into binary)
const MELAKARTA_JSON: &str = include_str!("../../../data/nadabrahman/melakarta_ragas.json");
const TIME_MAPPINGS_JSON: &str = include_str!("../../../data/nadabrahman/time_raga_mappings.json");
const CHAKRA_FREQ_JSON: &str = include_str!("../../../data/nadabrahman/chakra_frequencies.json");

/// Parsed raga database (lazy singleton)
static RAGA_DB: OnceLock<RagaDatabase> = OnceLock::new();

/// In-memory raga database
pub struct RagaDatabase {
    pub ragas: HashMap<u32, Raga>,
    pub time_mappings: Value,
    pub chakra_frequencies: Value,
}

impl RagaDatabase {
    fn load() -> Self {
        let melakarta: Value =
            serde_json::from_str(MELAKARTA_JSON).expect("Failed to parse melakarta_ragas.json");
        let time_mappings: Value = serde_json::from_str(TIME_MAPPINGS_JSON)
            .expect("Failed to parse time_raga_mappings.json");
        let chakra_frequencies: Value = serde_json::from_str(CHAKRA_FREQ_JSON)
            .expect("Failed to parse chakra_frequencies.json");

        let mut ragas = HashMap::new();

        if let Some(raga_array) = melakarta.get("ragas").and_then(|v| v.as_array()) {
            for raga_val in raga_array {
                if let Ok(raga) = serde_json::from_value::<Raga>(raga_val.clone()) {
                    ragas.insert(raga.number, raga);
                }
            }
        }

        RagaDatabase {
            ragas,
            time_mappings,
            chakra_frequencies,
        }
    }
}

/// Get the global raga database
pub fn raga_db() -> &'static RagaDatabase {
    RAGA_DB.get_or_init(RagaDatabase::load)
}

/// Look up a raga by number
pub fn get_raga(number: u32) -> Option<&'static Raga> {
    raga_db().ragas.get(&number)
}

/// Get ragas for a specific prahar (time period)
///
/// JSON structure: `prahars[].prahar` = number, `primary_ragas[]` = array of objects
/// with `melakarta_number`, `name`, `rationale`; `secondary_ragas` = array of raga numbers.
pub fn get_prahar_ragas(prahar_number: u32) -> Vec<RagaRecommendation> {
    let db = raga_db();
    let mut recommendations = Vec::new();

    if let Some(prahars) = db.time_mappings.get("prahars").and_then(|v| v.as_array()) {
        for prahar in prahars {
            let pn = prahar.get("prahar").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            if pn != prahar_number {
                continue;
            }

            // Primary ragas (array of objects with melakarta_number, name, rationale)
            if let Some(primary_ragas) = prahar.get("primary_ragas").and_then(|v| v.as_array()) {
                for (i, raga) in primary_ragas.iter().enumerate() {
                    let raga_num = raga
                        .get("melakarta_number")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;
                    let raga_name = raga
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    let reason = raga
                        .get("rationale")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    recommendations.push(RagaRecommendation {
                        raga_number: raga_num,
                        raga_name,
                        reason,
                        score: 1.0 - (i as f64 * 0.05),
                    });
                }
            }

            // Secondary ragas (array of plain raga numbers)
            if let Some(secondary) = prahar.get("secondary_ragas").and_then(|v| v.as_array()) {
                for (i, raga_val) in secondary.iter().enumerate() {
                    let raga_num = raga_val.as_u64().unwrap_or(0) as u32;
                    let raga_name = get_raga(raga_num)
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| format!("Raga {}", raga_num));

                    recommendations.push(RagaRecommendation {
                        raga_number: raga_num,
                        raga_name,
                        reason: "Secondary recommendation for this time period".to_string(),
                        score: 0.7 - (i as f64 * 0.1),
                    });
                }
            }

            break;
        }
    }

    recommendations
}

/// Get ragas by dosha affinity
pub fn get_ragas_for_dosha(dosha: &str) -> Vec<RagaRecommendation> {
    let db = raga_db();
    let dosha_lower = dosha.to_lowercase();

    db.ragas
        .values()
        .filter(|r| {
            r.dosha_affinity
                .iter()
                .any(|d| d.to_lowercase() == dosha_lower)
        })
        .map(|r| RagaRecommendation {
            raga_number: r.number,
            raga_name: r.name.clone(),
            reason: format!("Affinity with {} dosha", dosha),
            score: 0.7,
        })
        .collect()
}

/// Get ragas by mood/rasa
pub fn get_ragas_for_rasa(rasa: &str) -> Vec<RagaRecommendation> {
    let db = raga_db();
    let rasa_lower = rasa.to_lowercase();

    db.ragas
        .values()
        .filter(|r| r.rasa.to_lowercase() == rasa_lower)
        .map(|r| RagaRecommendation {
            raga_number: r.number,
            raga_name: r.name.clone(),
            reason: format!("Expresses {} rasa", rasa),
            score: 0.7,
        })
        .collect()
}

/// Get chakra frequency data for a chakra name.
///
/// Accepts either Sanskrit name ("Muladhara") or English name ("root", "Root").
/// Returns (solfeggio_frequency, binaural_beat_frequency).
pub fn get_chakra_frequency(chakra_name: &str) -> Option<(f64, f64)> {
    let db = raga_db();
    let name_lower = chakra_name.to_lowercase();

    if let Some(chakras) = db
        .chakra_frequencies
        .get("chakras")
        .and_then(|v| v.as_array())
    {
        for chakra in chakras {
            // Match against Sanskrit name or English name
            let sanskrit = chakra.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let english = chakra.get("english").and_then(|v| v.as_str()).unwrap_or("");

            if sanskrit.to_lowercase() == name_lower || english.to_lowercase() == name_lower {
                let solfeggio = chakra
                    .get("solfeggio_frequency")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let binaural = chakra
                    .get("binaural_parameters")
                    .and_then(|v| v.get("beat_frequency_hz"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                return Some((solfeggio, binaural));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raga_db_loads() {
        let db = raga_db();
        assert_eq!(db.ragas.len(), 72, "Should load all 72 Melakarta ragas");
    }

    #[test]
    fn test_get_raga() {
        let raga = get_raga(1).expect("Raga 1 should exist");
        assert_eq!(raga.name, "Kanakangi");
        assert_eq!(raga.chakra, 1);
    }

    #[test]
    fn test_get_raga_29() {
        let raga = get_raga(29).expect("Raga 29 should exist");
        assert_eq!(raga.name, "Dheerasankarabharanam");
    }

    #[test]
    fn test_get_prahar_ragas() {
        let ragas = get_prahar_ragas(1);
        assert!(
            !ragas.is_empty(),
            "Prahar 1 should have raga recommendations"
        );
        assert!(
            ragas[0].score >= ragas.last().unwrap().score,
            "Primary should score highest"
        );
    }

    #[test]
    fn test_get_ragas_for_dosha() {
        let vata_ragas = get_ragas_for_dosha("vata");
        assert!(!vata_ragas.is_empty(), "Should find ragas for vata dosha");
    }

    #[test]
    fn test_get_ragas_for_rasa() {
        let shanta_ragas = get_ragas_for_rasa("shanta");
        assert!(
            !shanta_ragas.is_empty(),
            "Should find ragas for shanta rasa"
        );
    }

    #[test]
    fn test_get_chakra_frequency_english() {
        let (solfeggio, _binaural) =
            get_chakra_frequency("root").expect("Root chakra should exist");
        assert!(
            (solfeggio - 396.0).abs() < 0.1,
            "Root chakra should be 396Hz, got {}",
            solfeggio
        );
    }

    #[test]
    fn test_get_chakra_frequency_sanskrit() {
        let (solfeggio, _) = get_chakra_frequency("Muladhara").expect("Muladhara should exist");
        assert!((solfeggio - 396.0).abs() < 0.1);
    }

    #[test]
    fn test_get_chakra_frequency_heart() {
        let (solfeggio, _) = get_chakra_frequency("heart").expect("Heart chakra should exist");
        assert!(
            (solfeggio - 639.0).abs() < 0.1,
            "Heart chakra should be 639Hz, got {}",
            solfeggio
        );
    }
}
