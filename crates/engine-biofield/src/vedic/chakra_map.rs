//! Chakra-planet correspondences and biofield metric derivation
//!
//! Maps Vedic planetary strengths to chakra activity levels and
//! derives global biofield metrics from natal chart analysis.

use crate::models::{Chakra, ChakraReading};

use super::planetary::{
    get_planet_strength, AspectType, PlanetAspect, PlanetStrength, VedicPlanet, VedicPosition,
};

/// Static chakra-planet correspondence
struct ChakraCorrespondence {
    chakra: Chakra,
    primary: VedicPlanet,
    secondary: VedicPlanet,
    color: &'static str,
}

/// Vedic chakra-planet correspondence table
const CORRESPONDENCES: [ChakraCorrespondence; 7] = [
    ChakraCorrespondence {
        chakra: Chakra::Root,
        primary: VedicPlanet::Mars,
        secondary: VedicPlanet::Saturn,
        color: "red",
    },
    ChakraCorrespondence {
        chakra: Chakra::Sacral,
        primary: VedicPlanet::Venus,
        secondary: VedicPlanet::Moon,
        color: "orange",
    },
    ChakraCorrespondence {
        chakra: Chakra::SolarPlexus,
        primary: VedicPlanet::Sun,
        secondary: VedicPlanet::Mars,
        color: "yellow",
    },
    ChakraCorrespondence {
        chakra: Chakra::Heart,
        primary: VedicPlanet::Venus,
        secondary: VedicPlanet::Moon,
        color: "green",
    },
    ChakraCorrespondence {
        chakra: Chakra::Throat,
        primary: VedicPlanet::Mercury,
        secondary: VedicPlanet::Jupiter,
        color: "blue",
    },
    ChakraCorrespondence {
        chakra: Chakra::ThirdEye,
        primary: VedicPlanet::Saturn,
        secondary: VedicPlanet::Moon,
        color: "indigo",
    },
    ChakraCorrespondence {
        chakra: Chakra::Crown,
        primary: VedicPlanet::Jupiter,
        secondary: VedicPlanet::Sun,
        color: "violet",
    },
];

/// Get the primary and secondary planet for a chakra
pub fn chakra_planets(chakra: Chakra) -> (VedicPlanet, VedicPlanet) {
    for c in &CORRESPONDENCES {
        if c.chakra == chakra {
            return (c.primary, c.secondary);
        }
    }
    // Fallback (shouldn't happen with 7 chakras)
    (VedicPlanet::Sun, VedicPlanet::Moon)
}

/// Calculate chakra activity level from planetary strengths (0.0-1.0)
/// Weighted: primary 70%, secondary 30%
pub fn chakra_activity(chakra: Chakra, strengths: &[PlanetStrength]) -> f64 {
    let (primary, secondary) = chakra_planets(chakra);
    let primary_strength = get_planet_strength(strengths, primary);
    let secondary_strength = get_planet_strength(strengths, secondary);
    (primary_strength * 0.7 + secondary_strength * 0.3).clamp(0.0, 1.0)
}

/// Calculate chakra balance from aspects to ruling planets (-1.0 to 1.0)
/// Positive = more benefic influence, Negative = more malefic influence
pub fn chakra_balance(chakra: Chakra, aspects: &[PlanetAspect]) -> f64 {
    let (primary, secondary) = chakra_planets(chakra);

    let mut benefic_count = 0.0_f64;
    let mut malefic_count = 0.0_f64;

    for aspect in aspects {
        let involves_ruler = aspect.planet_a == primary
            || aspect.planet_b == primary
            || aspect.planet_a == secondary
            || aspect.planet_b == secondary;

        if !involves_ruler {
            continue;
        }

        if aspect.aspect_type.is_harmonious() {
            benefic_count += 1.0;
        } else if aspect.aspect_type.is_challenging() {
            malefic_count += 1.0;
        } else {
            // Conjunction: check the other planet's nature
            let other = if aspect.planet_a == primary || aspect.planet_a == secondary {
                aspect.planet_b
            } else {
                aspect.planet_a
            };
            if other.is_benefic() {
                benefic_count += 0.5;
            } else {
                malefic_count += 0.5;
            }
        }
    }

    let total = benefic_count + malefic_count;
    if total == 0.0 {
        return 0.0; // neutral
    }

    // Normalize to [-1.0, 1.0]
    ((benefic_count - malefic_count) / total).clamp(-1.0, 1.0)
}

/// Map activity level to color intensity string
fn color_intensity_from_activity(activity: f64, color: &str) -> String {
    let intensity = if activity > 0.7 {
        "bright"
    } else if activity > 0.5 {
        "moderate"
    } else if activity > 0.3 {
        "dim"
    } else {
        "faint"
    };

    format!("{} {}", intensity, color)
}

/// Calculate readings for all 7 chakras
pub fn calculate_all_chakra_readings(
    strengths: &[PlanetStrength],
    aspects: &[PlanetAspect],
) -> Vec<ChakraReading> {
    CORRESPONDENCES
        .iter()
        .map(|c| {
            let activity_level = chakra_activity(c.chakra, strengths);
            let balance = chakra_balance(c.chakra, aspects);
            let color_intensity = color_intensity_from_activity(activity_level, c.color);

            ChakraReading {
                chakra: c.chakra,
                activity_level,
                balance,
                color_intensity,
            }
        })
        .collect()
}

// ── Global Biofield Metrics ──────────────────────────────────────────

/// Fractal dimension from aspect pattern diversity (1.0-2.0)
///
/// diversity_score = (unique_aspect_types/5.0)*0.5 + (occupied_signs/12.0)*0.5
/// fractal_dimension = 1.0 + diversity_score * 0.8
pub fn calculate_fractal_dimension(positions: &[VedicPosition], aspects: &[PlanetAspect]) -> f64 {
    // Count unique aspect types present
    let mut has_aspect = [false; 5];
    for aspect in aspects {
        let idx = match aspect.aspect_type {
            AspectType::Conjunction => 0,
            AspectType::Opposition => 1,
            AspectType::Trine => 2,
            AspectType::Square => 3,
            AspectType::Sextile => 4,
        };
        has_aspect[idx] = true;
    }
    let unique_aspects = has_aspect.iter().filter(|&&b| b).count() as f64;

    // Count occupied signs
    let mut sign_occupied = [false; 12];
    for pos in positions {
        sign_occupied[pos.sign.index() as usize] = true;
    }
    let occupied_signs = sign_occupied.iter().filter(|&&b| b).count() as f64;

    let diversity_score = (unique_aspects / 5.0) * 0.5 + (occupied_signs / 12.0) * 0.5;
    (1.0 + diversity_score * 0.8).clamp(1.0, 2.0)
}

/// Shannon entropy of planetary sign distribution (0.0-1.0)
///
/// Planets evenly spread across signs = high entropy.
/// All planets in one sign = near-zero entropy.
pub fn calculate_entropy(positions: &[VedicPosition]) -> f64 {
    let total = positions.len() as f64;
    if total == 0.0 {
        return 0.0;
    }

    // Count planets per sign
    let mut counts = [0u32; 12];
    for pos in positions {
        counts[pos.sign.index() as usize] += 1;
    }

    // Shannon entropy: H = -sum(p * log2(p))
    let mut h = 0.0_f64;
    for &count in &counts {
        if count > 0 {
            let p = count as f64 / total;
            h -= p * p.log2();
        }
    }

    // Normalize by max entropy (log2(12) for 12 signs)
    let max_entropy = 12.0_f64.log2();
    (h / max_entropy).clamp(0.0, 1.0)
}

/// Coherence from harmonious vs challenging aspect ratio (0.0-1.0)
///
/// More harmonious aspects = higher coherence.
/// Conjunctions counted based on benefic/malefic nature.
pub fn calculate_coherence(aspects: &[PlanetAspect]) -> f64 {
    if aspects.is_empty() {
        return 0.5; // neutral default
    }

    let mut harmonious = 0.0_f64;
    let mut challenging = 0.0_f64;

    for aspect in aspects {
        if aspect.aspect_type.is_harmonious() {
            harmonious += 1.0;
        } else if aspect.aspect_type.is_challenging() {
            challenging += 1.0;
        } else {
            // Conjunction: classify by planet nature
            let a_benefic = aspect.planet_a.is_benefic();
            let b_benefic = aspect.planet_b.is_benefic();
            if a_benefic && b_benefic {
                harmonious += 1.0;
            } else if !a_benefic && !b_benefic {
                challenging += 1.0;
            } else {
                harmonious += 0.5;
                challenging += 0.5;
            }
        }
    }

    let total = harmonious + challenging;
    if total == 0.0 {
        return 0.5;
    }

    (harmonious / total).clamp(0.0, 1.0)
}

/// Symmetry from planetary strength balance across zodiac hemispheres (0.0-1.0)
///
/// Signs 0-5 (Aries-Virgo) vs 6-11 (Libra-Pisces).
/// Equal distribution = 1.0, all in one half = near 0.0.
pub fn calculate_symmetry(positions: &[VedicPosition], strengths: &[PlanetStrength]) -> f64 {
    let mut first_half = 0.0_f64;
    let mut second_half = 0.0_f64;

    for pos in positions {
        let strength = get_planet_strength(strengths, pos.planet);
        if pos.sign.index() < 6 {
            first_half += strength;
        } else {
            second_half += strength;
        }
    }

    let total = first_half + second_half;
    if total == 0.0 {
        return 0.5;
    }

    (1.0 - (first_half - second_half).abs() / total).clamp(0.0, 1.0)
}

/// Composite vitality index from component metrics
pub fn calculate_vitality_index(
    fractal_dimension: f64,
    entropy: f64,
    coherence: f64,
    symmetry: f64,
) -> f64 {
    let fd_norm = (fractal_dimension - 1.0).clamp(0.0, 1.0);
    (coherence * 0.30 + fd_norm * 0.30 + entropy * 0.20 + symmetry * 0.20).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::super::planetary::{calculate_planet_strengths, ZodiacSign};
    use super::*;

    fn make_position(planet: VedicPlanet, longitude: f64) -> VedicPosition {
        VedicPosition {
            planet,
            longitude,
            latitude: 0.0,
            speed: 1.0,
            sign: ZodiacSign::from_longitude(longitude),
            degree_in_sign: ZodiacSign::degree_in_sign(longitude),
            is_retrograde: false,
        }
    }

    #[test]
    fn test_chakra_planets() {
        let (primary, secondary) = chakra_planets(Chakra::Root);
        assert_eq!(primary, VedicPlanet::Mars);
        assert_eq!(secondary, VedicPlanet::Saturn);

        let (primary, secondary) = chakra_planets(Chakra::Crown);
        assert_eq!(primary, VedicPlanet::Jupiter);
        assert_eq!(secondary, VedicPlanet::Sun);
    }

    #[test]
    fn test_chakra_activity_in_range() {
        let positions = vec![
            make_position(VedicPlanet::Mars, 0.0),     // Aries (own)
            make_position(VedicPlanet::Saturn, 180.0), // Libra (exalted)
        ];
        let aspects = vec![];
        let strengths = calculate_planet_strengths(&positions, &aspects);

        for chakra in Chakra::all() {
            let activity = chakra_activity(chakra, &strengths);
            assert!(
                (0.0..=1.0).contains(&activity),
                "{:?} activity {} out of range",
                chakra,
                activity
            );
        }
    }

    #[test]
    fn test_chakra_balance_in_range() {
        let aspects = vec![PlanetAspect {
            planet_a: VedicPlanet::Sun,
            planet_b: VedicPlanet::Jupiter,
            aspect_type: AspectType::Trine,
            orb: 0.0,
        }];

        for chakra in Chakra::all() {
            let balance = chakra_balance(chakra, &aspects);
            assert!(
                (-1.0..=1.0).contains(&balance),
                "{:?} balance {} out of range",
                chakra,
                balance
            );
        }
    }

    #[test]
    fn test_all_chakra_readings() {
        let positions = vec![
            make_position(VedicPlanet::Sun, 0.0),
            make_position(VedicPlanet::Moon, 60.0),
            make_position(VedicPlanet::Mars, 120.0),
            make_position(VedicPlanet::Mercury, 180.0),
            make_position(VedicPlanet::Jupiter, 240.0),
            make_position(VedicPlanet::Venus, 300.0),
            make_position(VedicPlanet::Saturn, 30.0),
            make_position(VedicPlanet::Rahu, 90.0),
            make_position(VedicPlanet::Ketu, 270.0),
        ];
        let aspects = super::super::planetary::calculate_natal_aspects(&positions);
        let strengths = calculate_planet_strengths(&positions, &aspects);

        let readings = calculate_all_chakra_readings(&strengths, &aspects);

        assert_eq!(readings.len(), 7);
        for reading in &readings {
            assert!((0.0..=1.0).contains(&reading.activity_level));
            assert!((-1.0..=1.0).contains(&reading.balance));
            assert!(!reading.color_intensity.is_empty());
        }
    }

    #[test]
    fn test_fractal_dimension_range() {
        // All planets clustered in one sign
        let clustered: Vec<_> = VedicPlanet::all()
            .iter()
            .map(|&p| make_position(p, 15.0))
            .collect();
        let aspects = super::super::planetary::calculate_natal_aspects(&clustered);
        let fd_clustered = calculate_fractal_dimension(&clustered, &aspects);
        assert!((1.0..=2.0).contains(&fd_clustered));

        // Planets spread across signs
        let spread: Vec<_> = VedicPlanet::all()
            .iter()
            .enumerate()
            .map(|(i, &p)| make_position(p, (i as f64) * 40.0))
            .collect();
        let aspects_spread = super::super::planetary::calculate_natal_aspects(&spread);
        let fd_spread = calculate_fractal_dimension(&spread, &aspects_spread);
        assert!((1.0..=2.0).contains(&fd_spread));

        // Spread should have higher FD than clustered
        assert!(
            fd_spread > fd_clustered,
            "Spread ({}) should have higher FD than clustered ({})",
            fd_spread,
            fd_clustered
        );
    }

    #[test]
    fn test_entropy_extremes() {
        // All in one sign -> low entropy
        let clustered: Vec<_> = VedicPlanet::all()
            .iter()
            .map(|&p| make_position(p, 15.0))
            .collect();
        let entropy_low = calculate_entropy(&clustered);
        assert!(
            entropy_low < 0.2,
            "All clustered should have low entropy: {}",
            entropy_low
        );

        // Spread evenly -> high entropy
        let spread: Vec<_> = VedicPlanet::all()
            .iter()
            .enumerate()
            .map(|(i, &p)| make_position(p, (i as f64) * 40.0))
            .collect();
        let entropy_high = calculate_entropy(&spread);
        assert!(
            entropy_high > 0.7,
            "Spread should have high entropy: {}",
            entropy_high
        );
    }

    #[test]
    fn test_coherence_harmonious() {
        let aspects = vec![
            PlanetAspect {
                planet_a: VedicPlanet::Sun,
                planet_b: VedicPlanet::Jupiter,
                aspect_type: AspectType::Trine,
                orb: 2.0,
            },
            PlanetAspect {
                planet_a: VedicPlanet::Venus,
                planet_b: VedicPlanet::Moon,
                aspect_type: AspectType::Sextile,
                orb: 3.0,
            },
        ];
        let coherence = calculate_coherence(&aspects);
        assert!(
            coherence > 0.8,
            "All harmonious should have high coherence: {}",
            coherence
        );
    }

    #[test]
    fn test_coherence_challenging() {
        let aspects = vec![
            PlanetAspect {
                planet_a: VedicPlanet::Saturn,
                planet_b: VedicPlanet::Mars,
                aspect_type: AspectType::Square,
                orb: 2.0,
            },
            PlanetAspect {
                planet_a: VedicPlanet::Sun,
                planet_b: VedicPlanet::Saturn,
                aspect_type: AspectType::Opposition,
                orb: 3.0,
            },
        ];
        let coherence = calculate_coherence(&aspects);
        assert!(
            coherence < 0.3,
            "All challenging should have low coherence: {}",
            coherence
        );
    }

    #[test]
    fn test_symmetry_balanced() {
        // Equal strength in both hemispheres
        let positions = vec![
            make_position(VedicPlanet::Sun, 15.0),   // Aries (first half)
            make_position(VedicPlanet::Moon, 195.0), // Libra (second half)
        ];
        let aspects = vec![];
        let strengths = calculate_planet_strengths(&positions, &aspects);
        let symmetry = calculate_symmetry(&positions, &strengths);
        assert!(
            symmetry > 0.5,
            "Balanced distribution should have moderate+ symmetry: {}",
            symmetry
        );
    }

    #[test]
    fn test_vitality_index_range() {
        let vitality = calculate_vitality_index(1.5, 0.55, 0.65, 0.75);
        assert!((0.0..=1.0).contains(&vitality));
        assert!(
            vitality > 0.4,
            "Good values should give decent vitality: {}",
            vitality
        );
    }
}
