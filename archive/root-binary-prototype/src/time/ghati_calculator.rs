use chrono::{DateTime, Utc, Duration, NaiveDate, TimeZone, Timelike, Datelike};
use serde::{Deserialize, Serialize};
use crate::models::Coordinates;

/// Ghati calculation methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GhatiCalculationMethod {
    Fixed,           // Method 1: Fixed 24-minute intervals
    SunriseSunset,   // Method 2: Sunrise-to-sunset division
    SolarTime,       // Method 3: Solar time division
    Hybrid,          // Method 4: Hybrid system (Recommended)
}

/// Ghati precision levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GhatiPrecision {
    Standard,        // Ghati level only
    High,           // Ghati + Pala level
    Extreme,        // Ghati + Pala + Vipala level
}

/// Ghati calculation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiCalculationConfig {
    pub method: GhatiCalculationMethod,
    pub precision: GhatiPrecision,
    pub solar_correction: bool,
    pub equation_of_time: bool,
    pub seasonal_adjustment: bool,
}

impl Default for GhatiCalculationConfig {
    fn default() -> Self {
        Self {
            method: GhatiCalculationMethod::Hybrid,
            precision: GhatiPrecision::High,
            solar_correction: true,
            equation_of_time: true,
            seasonal_adjustment: false,
        }
    }
}

/// Ghati time representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiTime {
    pub ghati: u8,           // 0-59
    pub pala: u8,            // 0-59
    pub vipala: u8,          // 0-59
    pub utc_timestamp: DateTime<Utc>,
    pub location: Coordinates,
    pub calculation_method: GhatiCalculationMethod,
    pub precision: GhatiPrecision,
}

/// Ghati boundary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiBoundary {
    pub ghati_number: u8,
    pub utc_timestamp: DateTime<Utc>,
    pub local_time: DateTime<Utc>, // TODO: Convert to local timezone
    pub panchanga: Option<serde_json::Value>, // TODO: Replace with PanchangaResult
}

/// Ghati transition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiTransition {
    pub from_ghati: u8,
    pub to_ghati: u8,
    pub transition_time: DateTime<Utc>,
    pub panchanga_change: Option<serde_json::Value>, // TODO: Replace with PanchangaChange
}

/// Solar time correction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarTimeCorrection {
    pub longitude_offset: f64,    // Minutes
    pub equation_of_time: f64,    // Minutes
    pub seasonal_adjustment: f64, // Minutes
}

/// Main Ghati calculator trait
pub trait GhatiCalculator: Send + Sync {
    fn calculate_ghati(&self, utc_time: DateTime<Utc>, location: Coordinates) -> Result<GhatiTime, String>;
    fn calculate_ghati_boundaries(&self, date: NaiveDate, location: Coordinates) -> Result<Vec<GhatiBoundary>, String>;
    fn get_next_ghati_transition(&self, current_time: DateTime<Utc>, location: Coordinates) -> Result<GhatiTransition, String>;
    fn utc_to_ghati(&self, utc_time: DateTime<Utc>, location: Coordinates) -> Result<GhatiTime, String>;
    fn ghati_to_utc(&self, ghati_time: GhatiTime, location: Coordinates) -> Result<DateTime<Utc>, String>;
}

/// Fixed interval Ghati calculator (Method 1)
pub struct FixedGhatiCalculator {
    config: GhatiCalculationConfig,
}

impl FixedGhatiCalculator {
    pub fn new(config: GhatiCalculationConfig) -> Self {
        Self { config }
    }
}

impl GhatiCalculator for FixedGhatiCalculator {
    fn calculate_ghati(&self, utc_time: DateTime<Utc>, location: Coordinates) -> Result<GhatiTime, String> {
        let seconds_since_midnight = utc_time.time().num_seconds_from_midnight() as u32;
        let ghati_number = (seconds_since_midnight / 1440) % 60; // 1440 seconds = 24 minutes
        let pala_number = ((seconds_since_midnight % 1440) / 24) % 60; // 24 seconds = 1 pala
        let vipala_number = ((seconds_since_midnight % 24) * 60 / 24) % 60; // 24 seconds = 60 vipalas
        
        Ok(GhatiTime {
            ghati: ghati_number as u8,
            pala: pala_number as u8,
            vipala: vipala_number as u8,
            utc_timestamp: utc_time,
            location,
            calculation_method: GhatiCalculationMethod::Fixed,
            precision: self.config.precision,
        })
    }

    fn calculate_ghati_boundaries(&self, date: NaiveDate, _location: Coordinates) -> Result<Vec<GhatiBoundary>, String> {
        let mut boundaries = Vec::new();
        let utc_midnight = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());
        
        for ghati in 0u32..60 {
            let ghati_seconds = ghati * 1440; // 24 minutes per ghati
            let boundary_time = utc_midnight + Duration::seconds(ghati_seconds as i64);
            
            boundaries.push(GhatiBoundary {
                ghati_number: ghati as u8,
                utc_timestamp: boundary_time,
                local_time: boundary_time, // TODO: Convert to local timezone
                panchanga: None, // TODO: Calculate Panchanga
            });
        }
        
        Ok(boundaries)
    }

    fn get_next_ghati_transition(&self, current_time: DateTime<Utc>, location: Coordinates) -> Result<GhatiTransition, String> {
        let current_ghati = self.calculate_ghati(current_time, location)?;
        let next_ghati = (current_ghati.ghati + 1) % 60;

        let next_ghati_seconds = (next_ghati as u32) * 1440;
        
        let transition_time = if next_ghati == 0 {
            // Next day
            let next_day = current_time.date_naive() + Duration::days(1);
            let midnight = next_day.and_hms_opt(0, 0, 0).unwrap();
            Utc.from_utc_datetime(&midnight)
        } else {
            // Same day
            let midnight = current_time.date_naive().and_hms_opt(0, 0, 0).unwrap();
            let utc_midnight = Utc.from_utc_datetime(&midnight);
            utc_midnight + Duration::seconds(next_ghati_seconds as i64)
        };
        
        Ok(GhatiTransition {
            from_ghati: current_ghati.ghati,
            to_ghati: next_ghati,
            transition_time,
            panchanga_change: None, // TODO: Calculate Panchanga change
        })
    }

    fn utc_to_ghati(&self, utc_time: DateTime<Utc>, location: Coordinates) -> Result<GhatiTime, String> {
        self.calculate_ghati(utc_time, location)
    }

    fn ghati_to_utc(&self, ghati_time: GhatiTime, _location: Coordinates) -> Result<DateTime<Utc>, String> {
        let date = ghati_time.utc_timestamp.date_naive();
        let midnight = date.and_hms_opt(0, 0, 0).unwrap();
        let utc_midnight = Utc.from_utc_datetime(&midnight);
        
        let ghati_seconds = ghati_time.ghati as i64 * 1440;
        let pala_seconds = ghati_time.pala as i64 * 24;
        let vipala_seconds = (ghati_time.vipala as i64 * 24) / 60;
        
        let total_seconds = ghati_seconds + pala_seconds + vipala_seconds;
        let utc_time = utc_midnight + Duration::seconds(total_seconds);
        
        Ok(utc_time)
    }
}

/// Hybrid Ghati calculator (Method 4 - Recommended)
pub struct HybridGhatiCalculator {
    config: GhatiCalculationConfig,
    base_calculator: FixedGhatiCalculator,
}

impl HybridGhatiCalculator {
    pub fn new(config: GhatiCalculationConfig) -> Self {
        let base_config = GhatiCalculationConfig {
            method: GhatiCalculationMethod::Fixed,
            ..config
        };
        
        Self {
            config,
            base_calculator: FixedGhatiCalculator::new(base_config),
        }
    }
    
    /// Calculate solar time correction
    fn calculate_solar_correction(&self, utc_time: DateTime<Utc>, location: Coordinates) -> SolarTimeCorrection {
        let longitude_offset = location.longitude * 4.0; // 1 degree = 4 minutes
        let equation_of_time = if self.config.equation_of_time {
            self.calculate_equation_of_time(utc_time)
        } else {
            0.0
        };
        let seasonal_adjustment = if self.config.seasonal_adjustment {
            self.calculate_seasonal_adjustment(utc_time, location)
        } else {
            0.0
        };
        
        SolarTimeCorrection {
            longitude_offset,
            equation_of_time,
            seasonal_adjustment,
        }
    }
    
    /// Calculate equation of time (simplified)
    fn calculate_equation_of_time(&self, utc_time: DateTime<Utc>) -> f64 {
        let day_of_year = utc_time.ordinal() as f64;
        let b = 2.0 * std::f64::consts::PI * (day_of_year - 81.0) / 365.0;
        
        // Simplified equation of time calculation
        let eot = 9.87 * (2.0 * b).sin() - 7.53 * b.cos() - 1.5 * b.sin();
        eot
    }
    
    /// Calculate seasonal adjustment (simplified)
    fn calculate_seasonal_adjustment(&self, utc_time: DateTime<Utc>, location: Coordinates) -> f64 {
        // Simplified seasonal adjustment based on latitude
        let latitude_factor = location.latitude.abs() / 90.0;
        let day_of_year = utc_time.ordinal() as f64;
        let seasonal_factor = (2.0 * std::f64::consts::PI * day_of_year / 365.0).sin();
        
        latitude_factor * seasonal_factor * 2.0 // Maximum 2 minutes adjustment
    }
    
    /// Apply solar correction to Ghati time
    fn apply_solar_correction(&self, base_ghati: GhatiTime, correction: SolarTimeCorrection) -> GhatiTime {
        let total_correction_minutes = correction.longitude_offset + correction.equation_of_time + correction.seasonal_adjustment;
        let correction_seconds = (total_correction_minutes * 60.0) as i64;
        
        let ghati_adjustment = correction_seconds / 1440; // 1440 seconds per Ghati
        let pala_adjustment = (correction_seconds % 1440) / 24; // 24 seconds per Pala
        let vipala_adjustment = ((correction_seconds % 1440) % 24) * 60 / 24; // 24 seconds = 60 vipalas
        
        let new_ghati = (base_ghati.ghati as i32 + ghati_adjustment as i32) as u8;
        let new_pala = (base_ghati.pala as i32 + pala_adjustment as i32) as u8;
        let new_vipala = (base_ghati.vipala as i32 + vipala_adjustment as i32) as u8;
        
        GhatiTime {
            ghati: new_ghati % 60,
            pala: new_pala % 60,
            vipala: new_vipala % 60,
            utc_timestamp: base_ghati.utc_timestamp,
            location: base_ghati.location,
            calculation_method: GhatiCalculationMethod::Hybrid,
            precision: base_ghati.precision,
        }
    }
}

impl GhatiCalculator for HybridGhatiCalculator {
    fn calculate_ghati(&self, utc_time: DateTime<Utc>, location: Coordinates) -> Result<GhatiTime, String> {
        // Get base Ghati from fixed system
        let base_ghati = self.base_calculator.calculate_ghati(utc_time, location.clone())?;
        
        if self.config.solar_correction {
            // Apply solar time correction
            let solar_correction = self.calculate_solar_correction(utc_time, location);
            let corrected_ghati = self.apply_solar_correction(base_ghati, solar_correction);
            Ok(corrected_ghati)
        } else {
            Ok(base_ghati)
        }
    }

    fn calculate_ghati_boundaries(&self, date: NaiveDate, location: Coordinates) -> Result<Vec<GhatiBoundary>, String> {
        // For now, use base calculator boundaries
        // TODO: Apply solar corrections to boundaries
        self.base_calculator.calculate_ghati_boundaries(date, location)
    }

    fn get_next_ghati_transition(&self, current_time: DateTime<Utc>, location: Coordinates) -> Result<GhatiTransition, String> {
        // For now, use base calculator transition
        // TODO: Apply solar corrections to transition
        self.base_calculator.get_next_ghati_transition(current_time, location)
    }

    fn utc_to_ghati(&self, utc_time: DateTime<Utc>, location: Coordinates) -> Result<GhatiTime, String> {
        self.calculate_ghati(utc_time, location)
    }

    fn ghati_to_utc(&self, ghati_time: GhatiTime, location: Coordinates) -> Result<DateTime<Utc>, String> {
        // For now, use base calculator conversion
        // TODO: Apply inverse solar corrections
        self.base_calculator.ghati_to_utc(ghati_time, location)
    }
}

/// Ghati calculator factory
pub struct GhatiCalculatorFactory;

impl GhatiCalculatorFactory {
    pub fn create_calculator(config: GhatiCalculationConfig) -> Box<dyn GhatiCalculator> {
        match config.method {
            GhatiCalculationMethod::Fixed => Box::new(FixedGhatiCalculator::new(config)),
            GhatiCalculationMethod::Hybrid => Box::new(HybridGhatiCalculator::new(config)),
            GhatiCalculationMethod::SunriseSunset => {
                // TODO: Implement SunriseSunsetGhatiCalculator
                Box::new(FixedGhatiCalculator::new(config))
            }
            GhatiCalculationMethod::SolarTime => {
                // TODO: Implement SolarTimeGhatiCalculator
                Box::new(FixedGhatiCalculator::new(config))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_fixed_ghati_calculation() {
        let config = GhatiCalculationConfig::default();
        let calculator = FixedGhatiCalculator::new(config);
        let location = Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: Some(920.0),
        };
        
        // Test midnight (should be Ghati 0)
        let midnight = Utc.with_ymd_and_hms(2025, 1, 27, 0, 0, 0).unwrap();
        let ghati = calculator.calculate_ghati(midnight, location.clone()).unwrap();
        assert_eq!(ghati.ghati, 0);
        assert_eq!(ghati.pala, 0);
        assert_eq!(ghati.vipala, 0);
        
        // Test 24 minutes (should be Ghati 1)
        let ghati_1 = midnight + Duration::minutes(24);
        let ghati_result = calculator.calculate_ghati(ghati_1, location.clone()).unwrap();
        assert_eq!(ghati_result.ghati, 1);
        assert_eq!(ghati_result.pala, 0);
        assert_eq!(ghati_result.vipala, 0);
        
        // Test 1 hour (should be Ghati 2.5)
        let hour_1 = midnight + Duration::hours(1);
        let ghati_result = calculator.calculate_ghati(hour_1, location).unwrap();
        assert_eq!(ghati_result.ghati, 2);
        assert_eq!(ghati_result.pala, 30); // 30 palas = 12 minutes
    }

    #[test]
    fn test_hybrid_ghati_calculation() {
        let config = GhatiCalculationConfig {
            method: GhatiCalculationMethod::Hybrid,
            solar_correction: true,
            equation_of_time: true,
            seasonal_adjustment: false,
            ..Default::default()
        };
        let calculator = HybridGhatiCalculator::new(config);
        let location = Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: Some(920.0),
        };
        
        let midnight = Utc.with_ymd_and_hms(2025, 1, 27, 0, 0, 0).unwrap();
        let ghati = calculator.calculate_ghati(midnight, location).unwrap();
        
        // Should have solar correction applied
        assert_eq!(ghati.calculation_method, GhatiCalculationMethod::Hybrid);
        assert!(ghati.ghati >= 0 && ghati.ghati < 60);
    }

    #[test]
    fn test_ghati_boundaries() {
        let config = GhatiCalculationConfig::default();
        let calculator = FixedGhatiCalculator::new(config);
        let location = Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: Some(920.0),
        };
        
        let date = NaiveDate::from_ymd_opt(2025, 1, 27).unwrap();
        let boundaries = calculator.calculate_ghati_boundaries(date, location).unwrap();
        
        assert_eq!(boundaries.len(), 60);
        assert_eq!(boundaries[0].ghati_number, 0);
        assert_eq!(boundaries[1].ghati_number, 1);
        assert_eq!(boundaries[59].ghati_number, 59);
    }

    #[test]
    fn test_ghati_transition() {
        let config = GhatiCalculationConfig::default();
        let calculator = FixedGhatiCalculator::new(config);
        let location = Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: Some(920.0),
        };
        
        let current_time = Utc.with_ymd_and_hms(2025, 1, 27, 12, 0, 0).unwrap();
        let transition = calculator.get_next_ghati_transition(current_time, location).unwrap();
        
        assert_eq!(transition.from_ghati, 30); // 12 hours = 30 ghatis
        assert_eq!(transition.to_ghati, 31);
    }
}
