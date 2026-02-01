//! Face zone wisdom data
//!
//! Traditional knowledge from Chinese Face Reading (Mian Xiang),
//! Ayurvedic face analysis, and Western physiognomy.

use crate::models::{FaceZone, Element, Dosha};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Wisdom data for a specific face zone
#[derive(Debug, Clone)]
pub struct FaceZoneWisdom {
    /// The face zone
    pub zone: FaceZone,
    /// TCM organ correlation
    pub tcm_organ: &'static str,
    /// Ayurvedic correlation
    pub ayurvedic_correlation: &'static str,
    /// Emotional connection
    pub emotional_connection: &'static str,
    /// Associated element
    pub element: Element,
    /// Indicators for this zone
    pub indicators: Vec<ZoneIndicator>,
}

/// Indicator observation for a face zone
#[derive(Debug, Clone)]
pub struct ZoneIndicator {
    /// What to observe (e.g., "deep lines")
    pub observation: &'static str,
    /// Possible meaning (e.g., "digestive stress")
    pub possible_meaning: &'static str,
    /// Which tradition this comes from
    pub tradition: &'static str,
}

/// Static face zone wisdom database
static FACE_ZONE_WISDOM: Lazy<HashMap<FaceZone, FaceZoneWisdom>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Forehead - Bladder, Intestines, Fire/Water
    map.insert(FaceZone::Forehead, FaceZoneWisdom {
        zone: FaceZone::Forehead,
        tcm_organ: "Bladder, Small Intestine",
        ayurvedic_correlation: "Third Eye (Ajna) - intuition, mental clarity",
        emotional_connection: "Worry, overthinking, mental processing",
        element: Element::Water,
        indicators: vec![
            ZoneIndicator {
                observation: "Horizontal lines",
                possible_meaning: "Mental overwork, worry patterns",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "High, broad forehead",
                possible_meaning: "Strong intellectual capacity, visionary thinking",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Narrow forehead",
                possible_meaning: "Focused, practical thinking style",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Vertical line between brows",
                possible_meaning: "Liver stress, held frustration",
                tradition: "Chinese Mian Xiang",
            },
        ],
    });

    // Eyebrows - Liver, Gallbladder
    map.insert(FaceZone::Eyebrows, FaceZoneWisdom {
        zone: FaceZone::Eyebrows,
        tcm_organ: "Liver, Gallbladder",
        ayurvedic_correlation: "Related to Pitta dosha - metabolism and transformation",
        emotional_connection: "Decision-making, assertiveness, anger processing",
        element: Element::Wood,
        indicators: vec![
            ZoneIndicator {
                observation: "Thick, strong eyebrows",
                possible_meaning: "Strong life force, determination",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Sparse eyebrows",
                possible_meaning: "May indicate thyroid or kidney considerations",
                tradition: "Ayurveda",
            },
            ZoneIndicator {
                observation: "Unibrow tendency",
                possible_meaning: "Intense focus, strong willpower",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "High arched brows",
                possible_meaning: "Selective, discerning nature",
                tradition: "Western Physiognomy",
            },
        ],
    });

    // Eyes - Liver, Heart, Kidneys
    map.insert(FaceZone::Eyes, FaceZoneWisdom {
        zone: FaceZone::Eyes,
        tcm_organ: "Liver (opens to eyes), Heart (Shen visible in eyes)",
        ayurvedic_correlation: "Alochaka Pitta - visual perception, insight",
        emotional_connection: "Spirit (Shen), emotional state, authenticity",
        element: Element::Fire,
        indicators: vec![
            ZoneIndicator {
                observation: "Bright, clear eyes",
                possible_meaning: "Strong Shen (spirit), good vital energy",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Wide-set eyes",
                possible_meaning: "Broad perspective, tolerant nature",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Close-set eyes",
                possible_meaning: "Focused, detail-oriented nature",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Dark circles under eyes",
                possible_meaning: "Kidney energy depletion, need for rest",
                tradition: "Ayurveda",
            },
            ZoneIndicator {
                observation: "Yellowing of sclera",
                possible_meaning: "Liver considerations",
                tradition: "Chinese Mian Xiang",
            },
        ],
    });

    // Nose - Spleen, Stomach, Heart
    map.insert(FaceZone::Nose, FaceZoneWisdom {
        zone: FaceZone::Nose,
        tcm_organ: "Spleen, Stomach, Heart",
        ayurvedic_correlation: "Related to Kapha dosha - earthiness and stability",
        emotional_connection: "Self-esteem, wealth consciousness, groundedness",
        element: Element::Earth,
        indicators: vec![
            ZoneIndicator {
                observation: "Strong, prominent nose",
                possible_meaning: "Leadership qualities, strong sense of self",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Redness at nose tip",
                possible_meaning: "Heart energy patterns, possible heat",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Wide nostrils",
                possible_meaning: "Strong vital capacity, generous nature",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Narrow bridge",
                possible_meaning: "Refined sensibilities, aesthetic nature",
                tradition: "Western Physiognomy",
            },
        ],
    });

    // Cheeks - Lungs, Large Intestine
    map.insert(FaceZone::Cheeks, FaceZoneWisdom {
        zone: FaceZone::Cheeks,
        tcm_organ: "Lungs, Large Intestine",
        ayurvedic_correlation: "Avalambaka Kapha - respiratory support",
        emotional_connection: "Grief, letting go, boundaries",
        element: Element::Metal,
        indicators: vec![
            ZoneIndicator {
                observation: "High cheekbones",
                possible_meaning: "Strong constitution, natural authority",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Pale cheeks",
                possible_meaning: "Lung qi deficiency, need for breath work",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Flushed cheeks",
                possible_meaning: "Heat patterns, possibly yin deficiency",
                tradition: "Ayurveda",
            },
            ZoneIndicator {
                observation: "Full, rounded cheeks",
                possible_meaning: "Kapha constitution, nurturing nature",
                tradition: "Ayurveda",
            },
        ],
    });

    // Mouth - Spleen, Stomach
    map.insert(FaceZone::Mouth, FaceZoneWisdom {
        zone: FaceZone::Mouth,
        tcm_organ: "Spleen, Stomach",
        ayurvedic_correlation: "Bodhaka Kapha - taste, initial digestion",
        emotional_connection: "Expression, nourishment, sensuality",
        element: Element::Earth,
        indicators: vec![
            ZoneIndicator {
                observation: "Full lips",
                possible_meaning: "Sensual nature, good digestive capacity",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Thin lips",
                possible_meaning: "Precision in speech, discerning nature",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Downturned corners",
                possible_meaning: "May indicate chronic disappointment patterns",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Upturned corners",
                possible_meaning: "Optimistic disposition, joyful nature",
                tradition: "Western Physiognomy",
            },
        ],
    });

    // Chin - Kidneys, reproductive organs
    map.insert(FaceZone::Chin, FaceZoneWisdom {
        zone: FaceZone::Chin,
        tcm_organ: "Kidneys, Reproductive organs",
        ayurvedic_correlation: "Related to Shukra dhatu - vitality and creativity",
        emotional_connection: "Willpower, determination, life force",
        element: Element::Water,
        indicators: vec![
            ZoneIndicator {
                observation: "Strong, prominent chin",
                possible_meaning: "Strong willpower, determination",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Receding chin",
                possible_meaning: "Adaptable nature, may need support in assertiveness",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Cleft chin",
                possible_meaning: "Creative nature, attraction to drama/arts",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Breakouts on chin",
                possible_meaning: "Hormonal patterns, kidney energy considerations",
                tradition: "Ayurveda",
            },
        ],
    });

    // Ears - Kidneys
    map.insert(FaceZone::Ears, FaceZoneWisdom {
        zone: FaceZone::Ears,
        tcm_organ: "Kidneys",
        ayurvedic_correlation: "Connected to Majja dhatu - nervous system, marrow",
        emotional_connection: "Ancestral wisdom, receptivity, longevity",
        element: Element::Water,
        indicators: vec![
            ZoneIndicator {
                observation: "Large, thick earlobes",
                possible_meaning: "Strong kidney essence, potential for longevity",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Ears set high",
                possible_meaning: "Quick mind, intellectual nature",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Ears set low",
                possible_meaning: "Practical, grounded approach",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Pale ears",
                possible_meaning: "Kidney yang deficiency, need for warmth",
                tradition: "Chinese Mian Xiang",
            },
        ],
    });

    // Jawline - Stomach, intestines, willpower
    map.insert(FaceZone::Jawline, FaceZoneWisdom {
        zone: FaceZone::Jawline,
        tcm_organ: "Stomach, Large Intestine",
        ayurvedic_correlation: "Related to digestive fire (Agni)",
        emotional_connection: "Determination, follow-through, stubbornness",
        element: Element::Earth,
        indicators: vec![
            ZoneIndicator {
                observation: "Strong, defined jawline",
                possible_meaning: "Determination, strong will",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Soft jawline",
                possible_meaning: "Flexible, adaptable, possibly Kapha constitution",
                tradition: "Ayurveda",
            },
            ZoneIndicator {
                observation: "Tight jaw muscles",
                possible_meaning: "Held tension, possibly suppressed anger",
                tradition: "Western Physiognomy",
            },
            ZoneIndicator {
                observation: "Breakouts along jawline",
                possible_meaning: "Hormonal or digestive patterns",
                tradition: "Ayurveda",
            },
        ],
    });

    // Temples - Gallbladder, Liver
    map.insert(FaceZone::Temples, FaceZoneWisdom {
        zone: FaceZone::Temples,
        tcm_organ: "Gallbladder, Liver",
        ayurvedic_correlation: "Connected to Pitta subdoshas",
        emotional_connection: "Decision-making, life direction, temporal awareness",
        element: Element::Wood,
        indicators: vec![
            ZoneIndicator {
                observation: "Prominent veins at temples",
                possible_meaning: "May indicate liver qi stagnation or stress",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Hollow temples",
                possible_meaning: "May indicate qi or blood deficiency",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Full temples",
                possible_meaning: "Good vitality, strong life force",
                tradition: "Chinese Mian Xiang",
            },
            ZoneIndicator {
                observation: "Headaches at temples",
                possible_meaning: "Gallbladder meridian patterns",
                tradition: "Chinese Mian Xiang",
            },
        ],
    });

    map
});

/// Get wisdom for a specific face zone
pub fn get_zone_wisdom(zone: FaceZone) -> Option<&'static FaceZoneWisdom> {
    FACE_ZONE_WISDOM.get(&zone)
}

/// Get all face zone wisdom entries
pub fn all_zone_wisdom() -> impl Iterator<Item = &'static FaceZoneWisdom> {
    FACE_ZONE_WISDOM.values()
}

/// Get face zones associated with a specific element
pub fn zones_for_element(element: Element) -> Vec<FaceZone> {
    FACE_ZONE_WISDOM
        .iter()
        .filter(|(_, w)| w.element == element)
        .map(|(z, _)| *z)
        .collect()
}

/// Dosha facial characteristics
#[derive(Debug, Clone)]
pub struct DoshaFacialSigns {
    pub dosha: Dosha,
    pub facial_shape: &'static str,
    pub skin_quality: &'static str,
    pub eye_characteristics: &'static str,
    pub lip_characteristics: &'static str,
}

/// Get facial signs for each dosha
pub fn dosha_facial_signs() -> Vec<DoshaFacialSigns> {
    vec![
        DoshaFacialSigns {
            dosha: Dosha::Vata,
            facial_shape: "Narrow, angular, elongated features",
            skin_quality: "Dry, thin, prone to fine lines",
            eye_characteristics: "Small, active, quick-moving",
            lip_characteristics: "Thin, may be dry or cracked",
        },
        DoshaFacialSigns {
            dosha: Dosha::Pitta,
            facial_shape: "Heart-shaped, medium, symmetrical",
            skin_quality: "Warm, oily T-zone, prone to redness",
            eye_characteristics: "Medium, penetrating, sharp gaze",
            lip_characteristics: "Medium, well-defined, may redden easily",
        },
        DoshaFacialSigns {
            dosha: Dosha::Kapha,
            facial_shape: "Round, full, soft features",
            skin_quality: "Smooth, oily, thick, well-hydrated",
            eye_characteristics: "Large, calm, steady gaze",
            lip_characteristics: "Full, moist, well-defined",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_zones_have_wisdom() {
        for zone in FaceZone::all() {
            let wisdom = get_zone_wisdom(*zone);
            assert!(wisdom.is_some(), "Missing wisdom for zone: {:?}", zone);
        }
    }

    #[test]
    fn test_zones_have_indicators() {
        for zone in FaceZone::all() {
            let wisdom = get_zone_wisdom(*zone).unwrap();
            assert!(!wisdom.indicators.is_empty(), 
                "Zone {:?} should have indicators", zone);
        }
    }

    #[test]
    fn test_zones_for_element() {
        let water_zones = zones_for_element(Element::Water);
        assert!(water_zones.contains(&FaceZone::Chin));
        assert!(water_zones.contains(&FaceZone::Ears));
    }

    #[test]
    fn test_dosha_signs_complete() {
        let signs = dosha_facial_signs();
        assert_eq!(signs.len(), 3);
    }
}
