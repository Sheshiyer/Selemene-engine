//! Swiss Ephemeris wrapper for Human Design planetary calculations
//!
//! Provides clean API for calculating all 13 planetary positions needed for HD charts.

use chrono::{DateTime, Utc, Timelike, Datelike};
use noesis_core::EngineError;

/// Planet identifiers for Swiss Ephemeris
#[derive(Debug, Clone, Copy)]
pub enum HDPlanet {
    Sun = 0,
    Moon = 1,
    Mercury = 2,
    Venus = 3,
    Mars = 4,
    Jupiter = 5,
    Saturn = 6,
    Uranus = 7,
    Neptune = 8,
    Pluto = 9,
    NorthNode = 10,    // True Node (better for HD)
    SouthNode = -10,   // Calculated as opposite of North Node
    Earth = 14,        // Geo-centric Earth position
}

/// Planetary position result
#[derive(Debug, Clone)]
pub struct PlanetPosition {
    pub longitude: f64,  // 0-360 degrees
    pub latitude: f64,   // degrees
    pub distance: f64,   // AU
    pub speed: f64,      // degrees per day
}

/// Swiss Ephemeris calculator for Human Design
pub struct EphemerisCalculator {
    data_path: String,
}

impl EphemerisCalculator {
    /// Create new calculator
    /// 
    /// # Arguments
    /// * `data_path` - Path to Swiss Ephemeris data files (use "" for default)
    /// 
    /// If path is empty, checks SWISS_EPHE_PATH env var, then tries common locations:
    /// - ./data/ephemeris
    /// - ../data/ephemeris (for crate tests)
    /// - /Volumes/madara/2026/witnessos/Selemene-engine/data/ephemeris (absolute fallback)
    pub fn new(data_path: impl Into<String>) -> Self {
        let mut data_path = data_path.into();
        
        // If empty, try to find ephemeris data
        if data_path.is_empty() {
            // Check environment variable first
            if let Ok(env_path) = std::env::var("SWISS_EPHE_PATH") {
                if std::path::Path::new(&env_path).exists() {
                    data_path = env_path;
                }
            }
            
            // Try common relative paths
            if data_path.is_empty() {
                let candidates = [
                    "data/ephemeris",
                    "./data/ephemeris",
                    "../data/ephemeris",
                    "../../data/ephemeris",
                    "../../../data/ephemeris",
                    "/Volumes/madara/2026/witnessos/Selemene-engine/data/ephemeris",
                ];
                
                for candidate in candidates {
                    let path = std::path::Path::new(candidate);
                    if path.exists() && path.join("sepl_18.se1").exists() {
                        data_path = candidate.to_string();
                        break;
                    }
                }
            }
        }
        
        swisseph::swe::set_ephe_path(&data_path);
        Self { data_path }
    }

    /// Convert DateTime to Julian Day using Swiss Ephemeris
    fn datetime_to_jd(dt: &DateTime<Utc>) -> f64 {
        let year = dt.year();
        let month = dt.month() as i32;
        let day = dt.day() as i32;
        let hour = dt.hour() as f64 
            + dt.minute() as f64 / 60.0 
            + dt.second() as f64 / 3600.0;

        // Use Swiss Ephemeris built-in Julian Day calculation
        swisseph::swe::julday(year, month, day, hour, 1)
    }

    /// Calculate position for a single planet
    pub fn get_planet_position(
        &self,
        planet: HDPlanet,
        datetime: &DateTime<Utc>,
    ) -> Result<PlanetPosition, EngineError> {
        let jd = Self::datetime_to_jd(datetime);
        let planet_id = planet as i32;
        let flags = 258; // SEFLG_SPEED | SEFLG_SWIEPH

        // Handle Earth as opposite of Sun (geocentric)
        if planet_id == 14 {
            let sun = self.get_planet_position(HDPlanet::Sun, datetime)?;
            let earth_longitude = (sun.longitude + 180.0) % 360.0;
            return Ok(PlanetPosition {
                longitude: earth_longitude,
                latitude: -sun.latitude,
                distance: sun.distance,
                speed: sun.speed,
            });
        }

        // Handle South Node as opposite of North Node
        if planet_id == -10 {
            let north_node = self.get_planet_position(HDPlanet::NorthNode, datetime)?;
            let south_longitude = (north_node.longitude + 180.0) % 360.0;
            return Ok(PlanetPosition {
                longitude: south_longitude,
                latitude: -north_node.latitude,
                distance: north_node.distance,
                speed: -north_node.speed,
            });
        }

        match swisseph::swe::calc_ut(jd, planet_id as u32, flags) {
            Ok(result) => Ok(PlanetPosition {
                longitude: result.out[0],
                latitude: result.out[1],
                distance: result.out[2],
                speed: result.out[3],
            }),
            Err(e) => Err(EngineError::CalculationError(format!(
                "Swiss Ephemeris calculation failed for planet {:?}: {:?}",
                planet, e
            ))),
        }
    }

    /// Calculate all 13 planetary positions for Human Design
    pub fn get_all_planets(
        &self,
        datetime: &DateTime<Utc>,
    ) -> Result<Vec<(HDPlanet, PlanetPosition)>, EngineError> {
        let planets = [
            HDPlanet::Sun,
            HDPlanet::Earth,
            HDPlanet::Moon,
            HDPlanet::NorthNode,
            HDPlanet::SouthNode,
            HDPlanet::Mercury,
            HDPlanet::Venus,
            HDPlanet::Mars,
            HDPlanet::Jupiter,
            HDPlanet::Saturn,
            HDPlanet::Uranus,
            HDPlanet::Neptune,
            HDPlanet::Pluto,
        ];

        planets
            .iter()
            .map(|&planet| {
                let pos = self.get_planet_position(planet, datetime)?;
                Ok((planet, pos))
            })
            .collect()
    }

    /// Get data path
    pub fn data_path(&self) -> &str {
        &self.data_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sun_position_j2000() {
        let calc = EphemerisCalculator::new("");
        let dt = DateTime::parse_from_rfc3339("2000-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let pos = calc.get_planet_position(HDPlanet::Sun, &dt).unwrap();

        // Sun should be around 280° on Jan 1, 2000
        assert!((pos.longitude - 280.0).abs() < 5.0);
        assert!(pos.latitude.abs() < 1.0);
    }

    #[test]
    fn test_all_planets() {
        let calc = EphemerisCalculator::new("");
        let dt = DateTime::parse_from_rfc3339("2000-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let positions = calc.get_all_planets(&dt).unwrap();

        // Should get exactly 13 planets
        assert_eq!(positions.len(), 13);

        // All longitudes should be 0-360
        for (planet, pos) in positions {
            assert!(
                pos.longitude >= 0.0 && pos.longitude < 360.0,
                "{:?} longitude out of range: {}",
                planet,
                pos.longitude
            );
        }
    }

    #[test]
    fn test_south_node_opposite_north_node() {
        let calc = EphemerisCalculator::new("");
        let dt = DateTime::parse_from_rfc3339("2000-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let north = calc.get_planet_position(HDPlanet::NorthNode, &dt).unwrap();
        let south = calc.get_planet_position(HDPlanet::SouthNode, &dt).unwrap();

        let diff = (north.longitude - south.longitude + 360.0) % 360.0;
        assert!((diff - 180.0).abs() < 0.1);
    }
}
