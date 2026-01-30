use crate::models::{PrecisionLevel, EngineError};
use super::{NativeSolarEngine, NativeLunarEngine, SwissEphemerisEngine};
use super::native_solar::SolarEngineError;
use super::native_lunar::LunarEngineError;
use super::swiss_ephemeris::SwissEphemerisError;

/// Validation engine errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationEngineError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Backend mismatch: {0}")]
    BackendMismatch(String),
    #[error("Tolerance exceeded: expected {expected}, got {actual}")]
    ToleranceExceeded { expected: f64, actual: f64 },
}

impl From<ValidationEngineError> for EngineError {
    fn from(err: ValidationEngineError) -> Self {
        EngineError::ValidationError(err.to_string())
    }
}

impl From<SolarEngineError> for ValidationEngineError {
    fn from(err: SolarEngineError) -> Self {
        ValidationEngineError::ValidationFailed(err.to_string())
    }
}

impl From<LunarEngineError> for ValidationEngineError {
    fn from(err: LunarEngineError) -> Self {
        ValidationEngineError::ValidationFailed(err.to_string())
    }
}

impl From<SwissEphemerisError> for ValidationEngineError {
    fn from(err: SwissEphemerisError) -> Self {
        ValidationEngineError::ValidationFailed(err.to_string())
    }
}

/// Validation engine for cross-checking calculations
pub struct ValidationEngine {
    native_solar: NativeSolarEngine,
    native_lunar: NativeLunarEngine,
    swiss_ephemeris: SwissEphemerisEngine,
    tolerance_standard: f64,
    tolerance_high: f64,
    tolerance_extreme: f64,
}

impl ValidationEngine {
    pub fn new(
        native_solar: NativeSolarEngine,
        native_lunar: NativeLunarEngine,
        swiss_ephemeris: SwissEphemerisEngine,
    ) -> Self {
        Self {
            native_solar,
            native_lunar,
            swiss_ephemeris,
            tolerance_standard: 0.1,   // 6 arcminutes
            tolerance_high: 0.01,      // 36 arcseconds
            tolerance_extreme: 0.001,  // 3.6 arcseconds
        }
    }

    /// Cross-validate solar position calculations
    pub async fn validate_solar_position(
        &self,
        jd: f64,
        precision: PrecisionLevel,
    ) -> Result<ValidationResult, ValidationEngineError> {
        let tolerance = self.get_tolerance(precision);
        
        // Calculate with native engine
        let native_longitude = self.native_solar.solar_longitude(jd, precision)?;
        
        // Calculate with Swiss Ephemeris
        let swiss_position = self.swiss_ephemeris.calculate_solar_position(jd).await?;
        
        // Compare results
        let difference = (native_longitude - swiss_position.longitude).abs();
        let normalized_diff = difference.rem_euclid(360.0);
        let actual_diff = normalized_diff.min(360.0 - normalized_diff);
        
        if actual_diff > tolerance {
            return Err(ValidationEngineError::ToleranceExceeded {
                expected: tolerance,
                actual: actual_diff,
            });
        }
        
        Ok(ValidationResult {
            native_value: native_longitude,
            swiss_value: swiss_position.longitude,
            difference: actual_diff,
            tolerance,
            passed: true,
        })
    }

    /// Cross-validate lunar position calculations
    pub async fn validate_lunar_position(
        &self,
        jd: f64,
        precision: PrecisionLevel,
    ) -> Result<ValidationResult, ValidationEngineError> {
        let tolerance = self.get_tolerance(precision);
        
        // Calculate with native engine
        let native_longitude = self.native_lunar.lunar_longitude(jd, precision)?;
        
        // Calculate with Swiss Ephemeris
        let swiss_position = self.swiss_ephemeris.calculate_lunar_position(jd).await?;
        
        // Compare results
        let difference = (native_longitude - swiss_position.longitude).abs();
        let normalized_diff = difference.rem_euclid(360.0);
        let actual_diff = normalized_diff.min(360.0 - normalized_diff);
        
        if actual_diff > tolerance {
            return Err(ValidationEngineError::ToleranceExceeded {
                expected: tolerance,
                actual: actual_diff,
            });
        }
        
        Ok(ValidationResult {
            native_value: native_longitude,
            swiss_value: swiss_position.longitude,
            difference: actual_diff,
            tolerance,
            passed: true,
        })
    }

    /// Validate Tithi calculation precision
    pub async fn validate_tithi_calculation(
        &self,
        jd: f64,
        precision: PrecisionLevel,
    ) -> Result<TithiValidationResult, ValidationEngineError> {
        let tolerance = self.get_tolerance(precision);
        
        // Calculate Sun-Moon difference with native engines
        let solar_longitude = self.native_solar.solar_longitude(jd, precision)?;
        let lunar_longitude = self.native_lunar.lunar_longitude(jd, precision)?;
        let native_diff = (lunar_longitude - solar_longitude).rem_euclid(360.0);
        
        // Calculate with Swiss Ephemeris
        let swiss_solar = self.swiss_ephemeris.calculate_solar_position(jd).await?;
        let swiss_lunar = self.swiss_ephemeris.calculate_lunar_position(jd).await?;
        let swiss_diff = (swiss_lunar.longitude - swiss_solar.longitude).rem_euclid(360.0);
        
        // Compare results
        let difference = (native_diff - swiss_diff).abs();
        let normalized_diff = difference.rem_euclid(360.0);
        let actual_diff = normalized_diff.min(360.0 - normalized_diff);
        
        if actual_diff > tolerance {
            return Err(ValidationEngineError::ToleranceExceeded {
                expected: tolerance,
                actual: actual_diff,
            });
        }
        
        Ok(TithiValidationResult {
            native_tithi: native_diff / 12.0, // 12° per Tithi
            swiss_tithi: swiss_diff / 12.0,
            difference: actual_diff,
            tolerance,
            passed: true,
        })
    }

    /// Get tolerance based on precision level
    fn get_tolerance(&self, precision: PrecisionLevel) -> f64 {
        match precision {
            PrecisionLevel::Standard => self.tolerance_standard,
            PrecisionLevel::High => self.tolerance_high,
            PrecisionLevel::Extreme => self.tolerance_extreme,
        }
    }

    /// Set custom tolerance values
    pub fn set_tolerances(
        &mut self,
        standard: f64,
        high: f64,
        extreme: f64,
    ) {
        self.tolerance_standard = standard;
        self.tolerance_high = high;
        self.tolerance_extreme = extreme;
    }

    /// Get current tolerance values
    pub fn get_tolerances(&self) -> (f64, f64, f64) {
        (
            self.tolerance_standard,
            self.tolerance_high,
            self.tolerance_extreme,
        )
    }
}

/// Result of validation between backends
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub native_value: f64,
    pub swiss_value: f64,
    pub difference: f64,
    pub tolerance: f64,
    pub passed: bool,
}

/// Result of Tithi validation
#[derive(Debug, Clone)]
pub struct TithiValidationResult {
    pub native_tithi: f64,
    pub swiss_tithi: f64,
    pub difference: f64,
    pub tolerance: f64,
    pub passed: bool,
}
