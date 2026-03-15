//! Swiss Ephemeris wrapper for transit calculations
//!
//! Reuses `EphemerisCalculator` and `EPHE_MUTEX` from engine-human-design
//! to avoid duplicate Swiss Ephemeris initialization and ensure thread safety.

use chrono::{DateTime, Datelike, Timelike, Utc};
use engine_human_design::{ephemeris::EPHE_MUTEX, EphemerisCalculator, HDPlanet};
use libswisseph_sys::tuple_result::simple::{swe_get_ayanamsa_ut, swe_julday, swe_set_sid_mode};
use noesis_core::EngineError;

use crate::models::{PlanetaryPosition, TransitPlanet, ZodiacSign};

/// Swiss Ephemeris sidereal mode constant for Lahiri ayanamsha.
const SIDM_LAHIRI: i32 = 1;

fn datetime_to_jd(datetime: &DateTime<Utc>) -> f64 {
    unsafe {
        swe_julday(
            datetime.year(),
            datetime.month() as i32,
            datetime.day() as i32,
            datetime.hour() as f64
                + datetime.minute() as f64 / 60.0
                + datetime.second() as f64 / 3600.0,
            1,
        )
    }
}

fn lahiri_ayanamsa(datetime: &DateTime<Utc>) -> f64 {
    let jd_ut = datetime_to_jd(datetime);

    // Swiss Ephemeris sidereal mode is global state; guard it with the shared mutex
    // used throughout the native ephemeris-backed engines.
    let _guard = EPHE_MUTEX.lock().unwrap();
    unsafe {
        swe_set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);
        swe_get_ayanamsa_ut(jd_ut)
    }
}

fn to_sidereal_longitude(tropical_longitude: f64, datetime: &DateTime<Utc>) -> f64 {
    (tropical_longitude - lahiri_ayanamsa(datetime)).rem_euclid(360.0)
}

/// Map TransitPlanet to engine-human-design's HDPlanet
fn to_hd_planet(planet: TransitPlanet) -> HDPlanet {
    match planet {
        TransitPlanet::Sun => HDPlanet::Sun,
        TransitPlanet::Moon => HDPlanet::Moon,
        TransitPlanet::Mercury => HDPlanet::Mercury,
        TransitPlanet::Venus => HDPlanet::Venus,
        TransitPlanet::Mars => HDPlanet::Mars,
        TransitPlanet::Jupiter => HDPlanet::Jupiter,
        TransitPlanet::Saturn => HDPlanet::Saturn,
        TransitPlanet::Uranus => HDPlanet::Uranus,
        TransitPlanet::Neptune => HDPlanet::Neptune,
        TransitPlanet::Pluto => HDPlanet::Pluto,
        TransitPlanet::Rahu => HDPlanet::NorthNode,
        TransitPlanet::Ketu => HDPlanet::SouthNode,
    }
}

/// Calculate planetary position for a single planet at a given time.
///
/// Thread-safe: delegates to `EphemerisCalculator` which serializes
/// Swiss Ephemeris C library access via `EPHE_MUTEX`.
pub fn calculate_position(
    calculator: &EphemerisCalculator,
    planet: TransitPlanet,
    datetime: &DateTime<Utc>,
) -> Result<PlanetaryPosition, EngineError> {
    let hd_planet = to_hd_planet(planet);
    let pos = calculator.get_planet_position(hd_planet, datetime)?;
    let longitude = to_sidereal_longitude(pos.longitude, datetime);

    let sign = ZodiacSign::from_longitude(longitude);
    let degree_in_sign = ZodiacSign::degree_in_sign(longitude);
    let is_retrograde = pos.speed < 0.0;

    Ok(PlanetaryPosition {
        planet,
        longitude,
        latitude: pos.latitude,
        speed: pos.speed,
        sign,
        degree_in_sign,
        is_retrograde,
    })
}

/// Calculate positions for all transit planets at a given time.
pub fn calculate_all_positions(
    calculator: &EphemerisCalculator,
    datetime: &DateTime<Utc>,
) -> Result<Vec<PlanetaryPosition>, EngineError> {
    TransitPlanet::all()
        .iter()
        .map(|&planet| calculate_position(calculator, planet, datetime))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sun_position() {
        let calc = EphemerisCalculator::new("");
        let now = Utc::now();
        let pos = calculate_position(&calc, TransitPlanet::Sun, &now);
        assert!(pos.is_ok(), "Sun position should succeed: {:?}", pos.err());
        let pos = pos.unwrap();
        assert!((0.0..360.0).contains(&pos.longitude));
        assert!(!pos.is_retrograde, "Sun is never retrograde");
    }

    #[test]
    fn test_calculate_all_positions() {
        let calc = EphemerisCalculator::new("");
        let now = Utc::now();
        let positions = calculate_all_positions(&calc, &now);
        assert!(positions.is_ok());
        let positions = positions.unwrap();
        assert_eq!(positions.len(), 12, "Should have all 12 planets");
    }

    #[test]
    fn test_sign_from_longitude() {
        assert_eq!(ZodiacSign::from_longitude(0.0), ZodiacSign::Aries);
        assert_eq!(ZodiacSign::from_longitude(45.0), ZodiacSign::Taurus);
        assert_eq!(ZodiacSign::from_longitude(90.0), ZodiacSign::Cancer);
        assert_eq!(ZodiacSign::from_longitude(270.0), ZodiacSign::Capricorn);
        assert_eq!(ZodiacSign::from_longitude(359.9), ZodiacSign::Pisces);
    }
}
