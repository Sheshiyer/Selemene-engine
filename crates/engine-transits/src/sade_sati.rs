//! Sade Sati detection — Saturn's 7.5-year transit over natal Moon
//!
//! Sade Sati activates when Saturn transits the 12th, 1st, or 2nd sign
//! from the natal Moon position.

use crate::models::{PlanetaryPosition, SadeSatiPhase, SadeSatiStatus, TransitPlanet, ZodiacSign};

/// Detect Sade Sati status from transit Saturn and natal Moon positions.
pub fn detect_sade_sati(
    transit_positions: &[PlanetaryPosition],
    natal_positions: &[PlanetaryPosition],
) -> SadeSatiStatus {
    let saturn = transit_positions
        .iter()
        .find(|p| p.planet == TransitPlanet::Saturn);
    let moon = natal_positions
        .iter()
        .find(|p| p.planet == TransitPlanet::Moon);

    match (saturn, moon) {
        (Some(saturn), Some(moon)) => {
            let saturn_sign = saturn.sign;
            let moon_sign = moon.sign;

            let saturn_idx = saturn_sign.index() as i8;
            let moon_idx = moon_sign.index() as i8;

            // Calculate sign distance (Saturn from Moon)
            let distance = ((saturn_idx - moon_idx + 12) % 12) as u8;

            let phase = match distance {
                11 => Some(SadeSatiPhase::Rising), // 12th from Moon
                0 => Some(SadeSatiPhase::Peak),    // Same sign as Moon
                1 => Some(SadeSatiPhase::Setting), // 2nd from Moon
                _ => None,
            };

            SadeSatiStatus {
                is_active: phase.is_some(),
                phase,
                saturn_sign,
                moon_sign,
            }
        }
        _ => SadeSatiStatus {
            is_active: false,
            phase: None,
            saturn_sign: ZodiacSign::Aries,
            moon_sign: ZodiacSign::Aries,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pos(planet: TransitPlanet, sign: ZodiacSign) -> PlanetaryPosition {
        let longitude = sign.index() as f64 * 30.0 + 15.0;
        PlanetaryPosition {
            planet,
            longitude,
            latitude: 0.0,
            speed: 0.03,
            sign,
            degree_in_sign: 15.0,
            is_retrograde: false,
        }
    }

    #[test]
    fn test_sade_sati_peak() {
        let transits = vec![make_pos(TransitPlanet::Saturn, ZodiacSign::Aquarius)];
        let natal = vec![make_pos(TransitPlanet::Moon, ZodiacSign::Aquarius)];

        let status = detect_sade_sati(&transits, &natal);
        assert!(status.is_active);
        assert_eq!(status.phase, Some(SadeSatiPhase::Peak));
    }

    #[test]
    fn test_sade_sati_rising() {
        // Saturn in Capricorn (12th from Aquarius Moon)
        let transits = vec![make_pos(TransitPlanet::Saturn, ZodiacSign::Capricorn)];
        let natal = vec![make_pos(TransitPlanet::Moon, ZodiacSign::Aquarius)];

        let status = detect_sade_sati(&transits, &natal);
        assert!(status.is_active);
        assert_eq!(status.phase, Some(SadeSatiPhase::Rising));
    }

    #[test]
    fn test_sade_sati_setting() {
        // Saturn in Pisces (2nd from Aquarius Moon)
        let transits = vec![make_pos(TransitPlanet::Saturn, ZodiacSign::Pisces)];
        let natal = vec![make_pos(TransitPlanet::Moon, ZodiacSign::Aquarius)];

        let status = detect_sade_sati(&transits, &natal);
        assert!(status.is_active);
        assert_eq!(status.phase, Some(SadeSatiPhase::Setting));
    }

    #[test]
    fn test_no_sade_sati() {
        // Saturn in Leo, Moon in Aquarius — no Sade Sati
        let transits = vec![make_pos(TransitPlanet::Saturn, ZodiacSign::Leo)];
        let natal = vec![make_pos(TransitPlanet::Moon, ZodiacSign::Aquarius)];

        let status = detect_sade_sati(&transits, &natal);
        assert!(!status.is_active);
        assert_eq!(status.phase, None);
    }

    #[test]
    fn test_sade_sati_wraparound() {
        // Saturn in Pisces (12th from Aries Moon)
        let transits = vec![make_pos(TransitPlanet::Saturn, ZodiacSign::Pisces)];
        let natal = vec![make_pos(TransitPlanet::Moon, ZodiacSign::Aries)];

        let status = detect_sade_sati(&transits, &natal);
        assert!(status.is_active);
        assert_eq!(status.phase, Some(SadeSatiPhase::Rising));
    }
}
