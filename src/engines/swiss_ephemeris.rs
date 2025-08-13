use crate::models::{PrecisionLevel, EngineError};
use swisseph::{SweDate, SweFlag, SweResult};
use std::path::Path;

/// Swiss Ephemeris engine errors
#[derive(Debug, thiserror::Error)]
pub enum SwissEphemerisError {
    #[error("Swiss Ephemeris calculation failed: {0}")]
    CalculationError(String),
    #[error("Ephemeris data not found at path: {0}")]
    DataNotFound(String),
    #[error("Invalid date: {0}")]
    InvalidDate(String),
}

impl From<SwissEphemerisError> for EngineError {
    fn from(err: SwissEphemerisError) -> Self {
        EngineError::CalculationError(err.to_string())
    }
}

/// Swiss Ephemeris engine for reliable astronomical calculations
pub struct SwissEphemerisEngine {
    data_path: String,
    initialized: bool,
}

impl SwissEphemerisEngine {
    pub fn new(data_path: String) -> Self {
        Self {
            data_path,
            initialized: false,
        }
    }

    /// Initialize the Swiss Ephemeris engine
    pub fn initialize(&mut self) -> Result<(), SwissEphemerisError> {
        if !Path::new(&self.data_path).exists() {
            return Err(SwissEphemerisError::DataNotFound(self.data_path.clone()));
        }

        // Set the ephemeris path
        swisseph::set_ephe_path(&self.data_path);
        self.initialized = true;
        
        Ok(())
    }

    /// Calculate solar position using Swiss Ephemeris
    pub fn calculate_solar_position(
        &self,
        jd: f64,
        _precision: PrecisionLevel, // Swiss Ephemeris has fixed precision
    ) -> Result<SolarPosition, SwissEphemerisError> {
        if !self.initialized {
            return Err(SwissEphemerisError::CalculationError(
                "Engine not initialized".to_string()
            ));
        }

        let flags = SweFlag::SEFLG_SWIEPH | SweFlag::SEFLG_SPEED;
        
        match swisseph::calc_ut(jd, swisseph::SE_SUN, flags) {
            SweResult::Ok(result) => {
                Ok(SolarPosition {
                    longitude: result.longitude,
                    latitude: result.latitude,
                    distance: result.distance,
                    longitude_speed: result.longitude_speed,
                    latitude_speed: result.latitude_speed,
                    distance_speed: result.distance_speed,
                })
            }
            SweResult::Err(e) => Err(SwissEphemerisError::CalculationError(e.to_string())),
        }
    }

    /// Calculate lunar position using Swiss Ephemeris
    pub fn calculate_lunar_position(
        &self,
        jd: f64,
        _precision: PrecisionLevel,
    ) -> Result<LunarPosition, SwissEphemerisError> {
        if !self.initialized {
            return Err(SwissEphemerisError::CalculationError(
                "Engine not initialized".to_string()
            ));
        }

        let flags = SweFlag::SEFLG_SWIEPH | SweFlag::SEFLG_SPEED;
        
        match swisseph::calc_ut(jd, swisseph::SE_MOON, flags) {
            SweResult::Ok(result) => {
                Ok(LunarPosition {
                    longitude: result.longitude,
                    latitude: result.latitude,
                    distance: result.distance,
                    longitude_speed: result.longitude_speed,
                    latitude_speed: result.latitude_speed,
                    distance_speed: result.distance_speed,
                })
            }
            SweResult::Err(e) => Err(SwissEphemerisError::CalculationError(e.to_string())),
        }
    }

    /// Calculate planetary position using Swiss Ephemeris
    pub fn calculate_planetary_position(
        &self,
        jd: f64,
        planet: i32,
    ) -> Result<PlanetaryPosition, SwissEphemerisError> {
        if !self.initialized {
            return Err(SwissEphemerisError::CalculationError(
                "Engine not initialized".to_string()
            ));
        }

        let flags = SweFlag::SEFLG_SWIEPH | SweFlag::SEFLG_SPEED;
        
        match swisseph::calc_ut(jd, planet, flags) {
            SweResult::Ok(result) => {
                Ok(PlanetaryPosition {
                    longitude: result.longitude,
                    latitude: result.latitude,
                    distance: result.distance,
                    longitude_speed: result.longitude_speed,
                    latitude_speed: result.latitude_speed,
                    distance_speed: result.distance_speed,
                    planet_id: planet,
                })
            }
            SweResult::Err(e) => Err(SwissEphemerisError::CalculationError(e.to_string())),
        }
    }

    /// Calculate houses using Swiss Ephemeris
    pub fn calculate_houses(
        &self,
        jd: f64,
        latitude: f64,
        longitude: f64,
        house_system: i32,
    ) -> Result<HouseCalculation, SwissEphemerisError> {
        if !self.initialized {
            return Err(SwissEphemerisError::CalculationError(
                "Engine not initialized".to_string()
            ));
        }

        match swisseph::houses_ex(jd, latitude, longitude, house_system) {
            SweResult::Ok(result) => {
                Ok(HouseCalculation {
                    houses: result.houses,
                    ascendant: result.ascendant,
                    mc: result.mc,
                    armc: result.armc,
                    vertex: result.vertex,
                    equatorial_ascendant: result.equasc,
                })
            }
            SweResult::Err(e) => Err(SwissEphemerisError::CalculationError(e.to_string())),
        }
    }

    /// Check if the engine is ready for calculations
    pub fn is_ready(&self) -> bool {
        self.initialized
    }

    /// Get the ephemeris data path
    pub fn get_data_path(&self) -> &str {
        &self.data_path
    }
}

/// Solar position data from Swiss Ephemeris
#[derive(Debug, Clone)]
pub struct SolarPosition {
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
    pub longitude_speed: f64,
    pub latitude_speed: f64,
    pub distance_speed: f64,
}

/// Lunar position data from Swiss Ephemeris
#[derive(Debug, Clone)]
pub struct LunarPosition {
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
    pub longitude_speed: f64,
    pub latitude_speed: f64,
    pub distance_speed: f64,
}

/// Planetary position data from Swiss Ephemeris
#[derive(Debug, Clone)]
pub struct PlanetaryPosition {
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
    pub longitude_speed: f64,
    pub latitude_speed: f64,
    pub distance_speed: f64,
    pub planet_id: i32,
}

/// House calculation data from Swiss Ephemeris
#[derive(Debug, Clone)]
pub struct HouseCalculation {
    pub houses: [f64; 13],
    pub ascendant: f64,
    pub mc: f64,
    pub armc: f64,
    pub vertex: f64,
    pub equatorial_ascendant: f64,
}
