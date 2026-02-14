//! Natal-to-transit aspect calculator
//!
//! Detects Conjunction, Opposition, Trine, Square, and Sextile aspects
//! between transiting planets and natal planet positions.

use crate::models::{AspectType, PlanetaryPosition, TransitAspect, TransitPlanet};

/// Calculate all aspects between transit positions and natal positions.
///
/// Uses default orbs per aspect type. Only returns aspects within orb.
pub fn calculate_aspects(
    transit_positions: &[PlanetaryPosition],
    natal_positions: &[PlanetaryPosition],
) -> Vec<TransitAspect> {
    calculate_aspects_with_orbs(transit_positions, natal_positions, None)
}

/// Calculate aspects with optional custom orb multiplier.
///
/// `orb_multiplier` scales the default orbs (e.g., 0.5 = tight orbs, 1.5 = wide orbs).
pub fn calculate_aspects_with_orbs(
    transit_positions: &[PlanetaryPosition],
    natal_positions: &[PlanetaryPosition],
    orb_multiplier: Option<f64>,
) -> Vec<TransitAspect> {
    let multiplier = orb_multiplier.unwrap_or(1.0);
    let mut aspects = Vec::new();

    for transit in transit_positions {
        for natal in natal_positions {
            for &aspect_type in AspectType::all() {
                let orb_limit = aspect_type.default_orb() * multiplier;
                if let Some(aspect) = check_aspect(transit, natal, aspect_type, orb_limit) {
                    aspects.push(aspect);
                }
            }
        }
    }

    // Sort by tightest orb first
    aspects.sort_by(|a, b| {
        a.orb
            .partial_cmp(&b.orb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    aspects
}

/// Check if a specific aspect exists between two planet positions.
fn check_aspect(
    transit: &PlanetaryPosition,
    natal: &PlanetaryPosition,
    aspect_type: AspectType,
    orb_limit: f64,
) -> Option<TransitAspect> {
    // The separation between the two planets (shortest arc)
    let separation = angular_diff(transit.longitude, natal.longitude);
    let target_angle = aspect_type.angle();
    let orb = (separation - target_angle).abs();

    if orb <= orb_limit {
        let is_applying = is_aspect_applying(transit, natal, aspect_type);

        Some(TransitAspect {
            transiting_planet: transit.planet,
            natal_planet: natal.planet,
            aspect_type,
            orb,
            is_applying,
            nature: aspect_type.nature(),
        })
    } else {
        None
    }
}

/// Shortest angular difference between two angles
fn angular_diff(a: f64, b: f64) -> f64 {
    let diff = (a - b).abs();
    if diff > 180.0 {
        360.0 - diff
    } else {
        diff
    }
}

/// Determine if an aspect is applying (getting tighter) or separating.
///
/// An aspect is applying when the transiting planet is moving toward
/// the exact aspect angle relative to the natal position.
fn is_aspect_applying(
    transit: &PlanetaryPosition,
    natal: &PlanetaryPosition,
    aspect_type: AspectType,
) -> bool {
    let current_sep = angular_diff(transit.longitude, natal.longitude);
    let target = aspect_type.angle();
    let current_orb = (current_sep - target).abs();

    // Simulate position in 1 day using speed
    let future_sep = angular_diff(transit.longitude + transit.speed, natal.longitude);
    let future_orb = (future_sep - target).abs();

    // Applying if the orb is getting smaller
    future_orb < current_orb
}

/// Filter aspects to only significant ones (slow-planet transits or tight orbs)
pub fn significant_aspects(aspects: &[TransitAspect]) -> Vec<TransitAspect> {
    aspects
        .iter()
        .filter(|a| is_slow_planet(a.transiting_planet) || a.orb < 2.0)
        .cloned()
        .collect()
}

/// Whether a planet is a slow-moving outer planet (more significant transits)
fn is_slow_planet(planet: TransitPlanet) -> bool {
    matches!(
        planet,
        TransitPlanet::Jupiter
            | TransitPlanet::Saturn
            | TransitPlanet::Uranus
            | TransitPlanet::Neptune
            | TransitPlanet::Pluto
            | TransitPlanet::Rahu
            | TransitPlanet::Ketu
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ZodiacSign;

    fn normalize_angle(angle: f64) -> f64 {
        ((angle % 360.0) + 360.0) % 360.0
    }

    fn make_position(planet: TransitPlanet, longitude: f64, speed: f64) -> PlanetaryPosition {
        PlanetaryPosition {
            planet,
            longitude,
            latitude: 0.0,
            speed,
            sign: ZodiacSign::from_longitude(longitude),
            degree_in_sign: ZodiacSign::degree_in_sign(longitude),
            is_retrograde: speed < 0.0,
        }
    }

    #[test]
    fn test_conjunction_detection() {
        let transit = vec![make_position(TransitPlanet::Saturn, 100.0, 0.03)];
        let natal = vec![make_position(TransitPlanet::Sun, 102.0, 1.0)];

        let aspects = calculate_aspects(&transit, &natal);
        assert!(
            aspects
                .iter()
                .any(|a| a.aspect_type == AspectType::Conjunction),
            "Should detect conjunction at 2 degree orb"
        );
    }

    #[test]
    fn test_opposition_detection() {
        let transit = vec![make_position(TransitPlanet::Jupiter, 0.0, 0.08)];
        let natal = vec![make_position(TransitPlanet::Moon, 183.0, 13.0)];

        let aspects = calculate_aspects(&transit, &natal);
        assert!(
            aspects
                .iter()
                .any(|a| a.aspect_type == AspectType::Opposition),
            "Should detect opposition at 3 degree orb"
        );
    }

    #[test]
    fn test_trine_detection() {
        let transit = vec![make_position(TransitPlanet::Venus, 0.0, 1.2)];
        let natal = vec![make_position(TransitPlanet::Mars, 122.0, 0.5)];

        let aspects = calculate_aspects(&transit, &natal);
        assert!(
            aspects.iter().any(|a| a.aspect_type == AspectType::Trine),
            "Should detect trine at 2 degree orb"
        );
    }

    #[test]
    fn test_no_aspect_outside_orb() {
        let transit = vec![make_position(TransitPlanet::Mars, 0.0, 0.5)];
        let natal = vec![make_position(TransitPlanet::Sun, 50.0, 1.0)];

        let aspects = calculate_aspects(&transit, &natal);
        assert!(aspects.is_empty(), "50 degrees matches no standard aspect");
    }

    #[test]
    fn test_applying_aspect() {
        // Saturn at 98 moving toward natal Sun at 100 — applying conjunction
        let transit = vec![make_position(TransitPlanet::Saturn, 98.0, 0.05)];
        let natal = vec![make_position(TransitPlanet::Sun, 100.0, 1.0)];

        let aspects = calculate_aspects(&transit, &natal);
        let conjunction = aspects
            .iter()
            .find(|a| a.aspect_type == AspectType::Conjunction);
        assert!(conjunction.is_some());
        assert!(conjunction.unwrap().is_applying, "Should be applying");
    }

    #[test]
    fn test_retrograde_separating() {
        // Saturn at 98 retrograde (negative speed) moving away from natal Sun at 100
        // At 98, moving to 97.95 — separation from 100 grows, so it's separating
        let transit = vec![make_position(TransitPlanet::Saturn, 98.0, -0.05)];
        let natal = vec![make_position(TransitPlanet::Sun, 100.0, 1.0)];

        let aspects = calculate_aspects(&transit, &natal);
        let conjunction = aspects
            .iter()
            .find(|a| a.aspect_type == AspectType::Conjunction);
        assert!(conjunction.is_some());
        // Retrograde at 98 moving to 97.95 — further from Sun at 100 = separating
        assert!(!conjunction.unwrap().is_applying);
    }

    #[test]
    fn test_normalize_angle() {
        assert!((normalize_angle(-10.0) - 350.0).abs() < 0.001);
        assert!((normalize_angle(370.0) - 10.0).abs() < 0.001);
        assert!((normalize_angle(180.0) - 180.0).abs() < 0.001);
    }

    #[test]
    fn test_angular_diff() {
        assert!((angular_diff(10.0, 350.0) - 20.0).abs() < 0.001);
        assert!((angular_diff(0.0, 180.0) - 180.0).abs() < 0.001);
        assert!((angular_diff(90.0, 90.0)).abs() < 0.001);
    }
}
