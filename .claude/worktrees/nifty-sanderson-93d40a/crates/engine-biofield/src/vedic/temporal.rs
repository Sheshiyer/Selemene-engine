//! Temporal modulation of natal biofield metrics
//!
//! Current planetary transits modulate the natal baseline by ±5-10%
//! to produce day-to-day variation while preserving the core pattern.

use chrono::{DateTime, Datelike, Utc};

use crate::models::BiofieldMetrics;

use super::planetary::{angular_diff, VedicPlanet, VedicPosition};

/// Apply temporal modulation to natal metrics based on current transits.
///
/// This creates meaningful day-to-day variation while keeping metrics
/// within their valid ranges and close to the natal baseline.
pub fn apply_temporal_modulation(
    metrics: &mut BiofieldMetrics,
    transit_positions: &[VedicPosition],
    natal_positions: &[VedicPosition],
    current_time: &DateTime<Utc>,
) {
    // Moon transit modulation (fastest, most noticeable)
    apply_moon_modulation(metrics, transit_positions, natal_positions);

    // Jupiter/Saturn transit modulation (slower background)
    apply_outer_planet_modulation(metrics, transit_positions, natal_positions);

    // Retrograde transit modulation
    apply_retrograde_modulation(metrics, transit_positions);

    // Lunar phase modulation (affects Moon-ruled chakras)
    apply_lunar_phase_modulation(metrics, transit_positions);

    // Day of week (subtle Vedic correspondence)
    apply_day_of_week_modulation(metrics, current_time);

    // Clamp all values to valid ranges
    clamp_metrics(metrics);
}

/// Moon transit relative to natal Moon: ±5% to coherence and symmetry
fn apply_moon_modulation(
    metrics: &mut BiofieldMetrics,
    transit_positions: &[VedicPosition],
    natal_positions: &[VedicPosition],
) {
    let transit_moon = transit_positions
        .iter()
        .find(|p| p.planet == VedicPlanet::Moon);
    let natal_moon = natal_positions
        .iter()
        .find(|p| p.planet == VedicPlanet::Moon);

    if let (Some(t_moon), Some(n_moon)) = (transit_moon, natal_moon) {
        let angle = angular_diff(t_moon.longitude, n_moon.longitude);

        // Trine (120°) and sextile (60°) positions are harmonious
        // Square (90°) and opposition (180°) are challenging
        let modulation = if (angle - 120.0).abs() < 10.0 || (angle - 60.0).abs() < 10.0 {
            0.05 // harmonious transit
        } else if (angle - 90.0).abs() < 10.0 || (angle - 180.0).abs() < 10.0 {
            -0.05 // challenging transit
        } else if angle < 10.0 {
            0.03 // conjunction (return to natal)
        } else {
            0.0
        };

        metrics.coherence += modulation;
        metrics.symmetry += modulation * 0.8;
    }
}

/// Jupiter/Saturn transits to natal positions: ±3% to vitality
fn apply_outer_planet_modulation(
    metrics: &mut BiofieldMetrics,
    transit_positions: &[VedicPosition],
    natal_positions: &[VedicPosition],
) {
    let outer_planets = [VedicPlanet::Jupiter, VedicPlanet::Saturn];
    let mut vitality_mod = 0.0_f64;

    for &outer in &outer_planets {
        let transit = transit_positions.iter().find(|p| p.planet == outer);
        if let Some(t_pos) = transit {
            for n_pos in natal_positions {
                let angle = angular_diff(t_pos.longitude, n_pos.longitude);

                // Tight aspect (within 5°) from outer planets
                if angle < 5.0 || (angle - 120.0).abs() < 5.0 || (angle - 60.0).abs() < 5.0 {
                    let mod_val = if outer == VedicPlanet::Jupiter {
                        0.03
                    } else {
                        -0.02
                    };
                    vitality_mod += mod_val;
                } else if (angle - 90.0).abs() < 5.0 || (angle - 180.0).abs() < 5.0 {
                    let mod_val = if outer == VedicPlanet::Jupiter {
                        -0.01
                    } else {
                        -0.03
                    };
                    vitality_mod += mod_val;
                }
            }
        }
    }

    // Cap total outer planet modulation
    metrics.vitality_index += vitality_mod.clamp(-0.05, 0.05);
}

/// Retrograde transits slightly reduce coherence
fn apply_retrograde_modulation(metrics: &mut BiofieldMetrics, transit_positions: &[VedicPosition]) {
    let retrograde_count = transit_positions
        .iter()
        .filter(|p| p.is_retrograde && !matches!(p.planet, VedicPlanet::Rahu | VedicPlanet::Ketu))
        .count();

    // Each retrograde planet reduces coherence slightly
    let reduction = (retrograde_count as f64) * 0.01;
    metrics.coherence -= reduction;
}

/// Lunar phase affects Sacral (Moon-ruled) and Third Eye (Moon secondary) chakras
fn apply_lunar_phase_modulation(
    metrics: &mut BiofieldMetrics,
    transit_positions: &[VedicPosition],
) {
    let sun = transit_positions
        .iter()
        .find(|p| p.planet == VedicPlanet::Sun);
    let moon = transit_positions
        .iter()
        .find(|p| p.planet == VedicPlanet::Moon);

    if let (Some(sun_pos), Some(moon_pos)) = (sun, moon) {
        // Lunar phase: elongation from Sun (0=new, 180=full)
        let elongation = ((moon_pos.longitude - sun_pos.longitude) + 360.0) % 360.0;

        // Full moon (around 180°) boosts Moon-related chakras
        // New moon (around 0°/360°) slightly reduces them
        let phase_factor = if (elongation - 180.0).abs() < 30.0 {
            0.03 // near full moon
        } else if !(30.0..=330.0).contains(&elongation) {
            -0.02 // near new moon
        } else {
            0.0
        };

        // Apply to Sacral (index 1) and Third Eye (index 5)
        if let Some(sacral) = metrics.chakra_readings.get_mut(1) {
            sacral.activity_level = (sacral.activity_level + phase_factor).clamp(0.0, 1.0);
        }
        if let Some(third_eye) = metrics.chakra_readings.get_mut(5) {
            third_eye.activity_level = (third_eye.activity_level + phase_factor).clamp(0.0, 1.0);
        }
    }
}

/// Day of week: 2% boost to associated chakra (Vedic Vara system)
fn apply_day_of_week_modulation(metrics: &mut BiofieldMetrics, current_time: &DateTime<Utc>) {
    // Vedic day-planet-chakra mapping:
    // Sunday=Sun→SolarPlexus, Monday=Moon→Sacral, Tuesday=Mars→Root,
    // Wednesday=Mercury→Throat, Thursday=Jupiter→Crown,
    // Friday=Venus→Heart, Saturday=Saturn→ThirdEye
    let chakra_index = match current_time.weekday() {
        chrono::Weekday::Sun => 2, // Solar Plexus
        chrono::Weekday::Mon => 1, // Sacral
        chrono::Weekday::Tue => 0, // Root
        chrono::Weekday::Wed => 4, // Throat
        chrono::Weekday::Thu => 6, // Crown
        chrono::Weekday::Fri => 3, // Heart
        chrono::Weekday::Sat => 5, // Third Eye
    };

    if let Some(reading) = metrics.chakra_readings.get_mut(chakra_index) {
        reading.activity_level = (reading.activity_level + 0.02).clamp(0.0, 1.0);
    }
}

/// Ensure all metrics stay within valid ranges after modulation
fn clamp_metrics(metrics: &mut BiofieldMetrics) {
    metrics.fractal_dimension = metrics.fractal_dimension.clamp(1.0, 2.0);
    metrics.entropy = metrics.entropy.clamp(0.0, 1.0);
    metrics.coherence = metrics.coherence.clamp(0.0, 1.0);
    metrics.symmetry = metrics.symmetry.clamp(0.0, 1.0);
    metrics.vitality_index = metrics.vitality_index.clamp(0.0, 1.0);

    for reading in &mut metrics.chakra_readings {
        reading.activity_level = reading.activity_level.clamp(0.0, 1.0);
        reading.balance = reading.balance.clamp(-1.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::super::planetary::ZodiacSign;
    use super::*;
    use crate::models::{Chakra, ChakraReading};
    use chrono::TimeZone;

    fn make_test_metrics() -> BiofieldMetrics {
        BiofieldMetrics {
            fractal_dimension: 1.5,
            entropy: 0.55,
            coherence: 0.65,
            symmetry: 0.7,
            vitality_index: 0.6,
            chakra_readings: Chakra::all()
                .into_iter()
                .map(|c| ChakraReading {
                    chakra: c,
                    activity_level: 0.5,
                    balance: 0.0,
                    color_intensity: "moderate green".to_string(),
                })
                .collect(),
            timestamp: Utc::now(),
        }
    }

    fn make_transit_position(planet: VedicPlanet, longitude: f64) -> VedicPosition {
        VedicPosition {
            planet,
            longitude,
            latitude: 0.0,
            speed: if planet == VedicPlanet::Saturn {
                -0.05
            } else {
                1.0
            },
            sign: ZodiacSign::from_longitude(longitude),
            degree_in_sign: ZodiacSign::degree_in_sign(longitude),
            is_retrograde: planet == VedicPlanet::Saturn,
        }
    }

    #[test]
    fn test_temporal_modulation_preserves_ranges() {
        let mut metrics = make_test_metrics();

        let transit_positions = vec![
            make_transit_position(VedicPlanet::Sun, 0.0),
            make_transit_position(VedicPlanet::Moon, 90.0),
            make_transit_position(VedicPlanet::Mars, 45.0),
            make_transit_position(VedicPlanet::Jupiter, 120.0),
            make_transit_position(VedicPlanet::Saturn, 270.0),
            make_transit_position(VedicPlanet::Venus, 180.0),
            make_transit_position(VedicPlanet::Mercury, 60.0),
            make_transit_position(VedicPlanet::Rahu, 150.0),
            make_transit_position(VedicPlanet::Ketu, 330.0),
        ];

        let natal_positions = vec![
            make_transit_position(VedicPlanet::Moon, 0.0),
            make_transit_position(VedicPlanet::Sun, 45.0),
        ];

        let now = Utc::now();
        apply_temporal_modulation(&mut metrics, &transit_positions, &natal_positions, &now);

        assert!(metrics.fractal_dimension >= 1.0 && metrics.fractal_dimension <= 2.0);
        assert!(metrics.entropy >= 0.0 && metrics.entropy <= 1.0);
        assert!(metrics.coherence >= 0.0 && metrics.coherence <= 1.0);
        assert!(metrics.symmetry >= 0.0 && metrics.symmetry <= 1.0);
        assert!(metrics.vitality_index >= 0.0 && metrics.vitality_index <= 1.0);

        for reading in &metrics.chakra_readings {
            assert!(reading.activity_level >= 0.0 && reading.activity_level <= 1.0);
            assert!(reading.balance >= -1.0 && reading.balance <= 1.0);
        }
    }

    #[test]
    fn test_different_times_produce_different_metrics() {
        let transit_positions_a = vec![
            make_transit_position(VedicPlanet::Sun, 0.0),
            make_transit_position(VedicPlanet::Moon, 0.0), // Moon conjunct natal Moon
        ];
        let transit_positions_b = vec![
            make_transit_position(VedicPlanet::Sun, 0.0),
            make_transit_position(VedicPlanet::Moon, 90.0), // Moon square natal Moon
        ];

        let natal_positions = vec![make_transit_position(VedicPlanet::Moon, 0.0)];

        let mut metrics_a = make_test_metrics();
        let mut metrics_b = make_test_metrics();
        let now = Utc::now();

        apply_temporal_modulation(&mut metrics_a, &transit_positions_a, &natal_positions, &now);
        apply_temporal_modulation(&mut metrics_b, &transit_positions_b, &natal_positions, &now);

        // Different Moon positions should produce different coherence
        assert!(
            (metrics_a.coherence - metrics_b.coherence).abs() > 0.001,
            "Different Moon transits should produce different coherence: {} vs {}",
            metrics_a.coherence,
            metrics_b.coherence
        );
    }

    #[test]
    fn test_day_of_week_modulation() {
        // Monday should boost Sacral (index 1)
        let monday = Utc.with_ymd_and_hms(2026, 2, 9, 12, 0, 0).unwrap(); // Monday
        let mut metrics = make_test_metrics();
        let original_sacral = metrics.chakra_readings[1].activity_level;

        apply_day_of_week_modulation(&mut metrics, &monday);
        assert!(
            metrics.chakra_readings[1].activity_level > original_sacral,
            "Monday should boost Sacral chakra"
        );
    }
}
