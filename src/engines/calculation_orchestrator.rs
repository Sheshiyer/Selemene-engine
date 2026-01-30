use crate::models::{PanchangaRequest, PanchangaResult, EngineError, PrecisionLevel};
use crate::config::EngineConfig;
use super::{
    HybridBackend, NativeSolarEngine, NativeLunarEngine, 
    SwissEphemerisEngine, ValidationEngine, Backend, PanchangaCalculator
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use chrono::TimeZone;

/// Main calculation orchestrator that coordinates all engines
pub struct CalculationOrchestrator {
    hybrid_backend: HybridBackend,
    native_solar: NativeSolarEngine,
    native_lunar: NativeLunarEngine,
    swiss_ephemeris: SwissEphemerisEngine,
    validation_engine: ValidationEngine,
    panchanga_calculator: PanchangaCalculator,
    #[allow(dead_code)]
    config: Arc<RwLock<EngineConfig>>,
}

impl CalculationOrchestrator {
    pub fn new(config: Arc<RwLock<EngineConfig>>) -> Self {
        let native_solar = NativeSolarEngine::new();
        let native_lunar = NativeLunarEngine::new();
        
        let swiss_ephemeris_path = config
            .try_read()
            .map(|config_guard| config_guard.swiss_ephemeris_path.clone())
            .unwrap_or_else(|_| "data/ephemeris".to_string());
        let swiss_ephemeris = SwissEphemerisEngine::new(swiss_ephemeris_path);
        
        // Initialize Swiss Ephemeris (synchronous for now)
        // TODO: Make this async when Swiss Ephemeris is properly implemented
        // if let Err(e) = swiss_ephemeris.initialize().await {
        //     warn!("Failed to initialize Swiss Ephemeris: {}", e);
        // }
        
        let validation_engine = ValidationEngine::new(
            native_solar.clone(),
            native_lunar.clone(),
            swiss_ephemeris.clone(),
        );

        let panchanga_calculator = PanchangaCalculator::new();

        Self {
            hybrid_backend: HybridBackend::new(config.clone()),
            native_solar,
            native_lunar,
            swiss_ephemeris,
            validation_engine,
            panchanga_calculator,
            config,
        }
    }

    /// Calculate Panchanga for a given request
    pub async fn calculate_panchanga(
        &self,
        request: PanchangaRequest,
    ) -> Result<PanchangaResult, EngineError> {
        let start_time = std::time::Instant::now();
        
        info!("Starting Panchanga calculation for date: {}", request.date);
        
        // 1. Request validation and preprocessing
        let validated_request = self.validate_request(request)?;
        
        // 2. Backend selection
        let backend_choice = self.hybrid_backend.select_backend(&validated_request).await?;
        
        // 3. Calculation execution
        let calculation_result = match backend_choice {
            Backend::Native => self.calculate_with_native(&validated_request).await?,
            Backend::Swiss => self.calculate_with_swiss(&validated_request).await?,
            Backend::Validated => self.calculate_with_validation(&validated_request).await?,
        };
        
        // 4. Result post-processing
        let final_result = self.post_process_result(calculation_result)?;
        
        let duration = start_time.elapsed();
        info!("Panchanga calculation completed in {:?}", duration);
        
        Ok(final_result)
    }

    /// Calculate Panchanga for a date range in parallel
    pub async fn calculate_range_parallel(
        &self,
        requests: Vec<PanchangaRequest>,
    ) -> Result<Vec<PanchangaResult>, EngineError> {
        let start_time = std::time::Instant::now();
        info!("Starting parallel Panchanga calculation for {} dates", requests.len());
        
        let mut results = Vec::with_capacity(requests.len());
        let mut errors = Vec::new();
        
        // Process requests in parallel chunks
        let chunk_size = (requests.len() / num_cpus::get()).max(1);
        
        for chunk in requests.chunks(chunk_size) {
            let chunk_results: Vec<Result<PanchangaResult, EngineError>> = 
                futures::future::join_all(
                    chunk.iter().map(|req| self.calculate_panchanga(req.clone()))
                ).await;
            
            for result in chunk_results {
                match result {
                    Ok(panchanga) => results.push(panchanga),
                    Err(e) => {
                        error!("Calculation failed: {}", e);
                        errors.push(e);
                    }
                }
            }
        }
        
        let duration = start_time.elapsed();
        info!("Parallel calculation completed in {:?} with {} errors", duration, errors.len());
        
        if !errors.is_empty() {
            warn!("Some calculations failed: {:?}", errors);
        }
        
        Ok(results)
    }

    fn validate_request(&self, request: PanchangaRequest) -> Result<PanchangaRequest, EngineError> {
        // TODO: Implement comprehensive request validation
        // - Date range validation
        // - Coordinate validation
        // - Precision level validation
        
        if request.date.is_empty() {
            return Err(EngineError::ValidationError("Date is required".to_string()));
        }
        
        Ok(request)
    }

    async fn calculate_with_native(
        &self,
        request: &PanchangaRequest,
    ) -> Result<PanchangaResult, EngineError> {
        info!("Using native engines for calculation");
        
        let precision = request.precision.unwrap_or(PrecisionLevel::High);
        let jd = self.parse_date_to_jd(&request.date)?;
        
        // Calculate solar position
        let solar_longitude = self.native_solar.solar_longitude(jd, precision)?;
        
        // Calculate lunar position
        let lunar_longitude = self.native_lunar.lunar_longitude(jd, precision)?;
        
        // Calculate all Panchanga elements
        let tithi_info = self.panchanga_calculator.calculate_tithi(solar_longitude, lunar_longitude, jd)?;
        let nakshatra_info = self.panchanga_calculator.calculate_nakshatra(lunar_longitude, jd)?;
        let yoga_info = self.panchanga_calculator.calculate_yoga(solar_longitude, lunar_longitude, jd)?;
        let karana_info = self.panchanga_calculator.calculate_karana(solar_longitude, lunar_longitude, jd)?;
        let vara_info = self.panchanga_calculator.calculate_vara(jd)?;
        
        Ok(PanchangaResult {
            date: request.date.clone(),
            tithi: Some(tithi_info.number as f64),
            nakshatra: Some(nakshatra_info.number as f64),
            yoga: Some(yoga_info.number as f64),
            karana: Some(karana_info.number as f64),
            vara: Some(vara_info.number as f64),
            solar_longitude,
            lunar_longitude,
            precision: precision as u8,
            backend: "native".to_string(),
            calculation_time: Some(chrono::Utc::now()),
        })
    }

    async fn calculate_with_swiss(
        &self,
        request: &PanchangaRequest,
    ) -> Result<PanchangaResult, EngineError> {
        info!("Using Swiss Ephemeris for calculation");
        
        let precision = request.precision.unwrap_or(PrecisionLevel::High);
        let jd = self.parse_date_to_jd(&request.date)?;
        
        // Calculate solar position
        let solar_position = self.swiss_ephemeris.calculate_solar_position(jd).await?;
        
        // Calculate lunar position
        let lunar_position = self.swiss_ephemeris.calculate_lunar_position(jd).await?;
        
        // Calculate all Panchanga elements
        let tithi_info = self.panchanga_calculator.calculate_tithi(solar_position.longitude, lunar_position.longitude, jd)?;
        let nakshatra_info = self.panchanga_calculator.calculate_nakshatra(lunar_position.longitude, jd)?;
        let yoga_info = self.panchanga_calculator.calculate_yoga(solar_position.longitude, lunar_position.longitude, jd)?;
        let karana_info = self.panchanga_calculator.calculate_karana(solar_position.longitude, lunar_position.longitude, jd)?;
        let vara_info = self.panchanga_calculator.calculate_vara(jd)?;
        
        Ok(PanchangaResult {
            date: request.date.clone(),
            tithi: Some(tithi_info.number as f64),
            nakshatra: Some(nakshatra_info.number as f64),
            yoga: Some(yoga_info.number as f64),
            karana: Some(karana_info.number as f64),
            vara: Some(vara_info.number as f64),
            solar_longitude: solar_position.longitude,
            lunar_longitude: lunar_position.longitude,
            precision: precision as u8,
            backend: "swiss".to_string(),
            calculation_time: Some(chrono::Utc::now()),
        })
    }

    async fn calculate_with_validation(
        &self,
        request: &PanchangaRequest,
    ) -> Result<PanchangaResult, EngineError> {
        info!("Using cross-validation for calculation");
        
        // Calculate with both backends and validate
        let native_result = self.calculate_with_native(request).await?;
        let swiss_result = self.calculate_with_swiss(request).await?;
        
        // Validate results
        let validation_result = self.validation_engine
            .validate_tithi_calculation(
                self.parse_date_to_jd(&request.date)?,
                request.precision.unwrap_or(PrecisionLevel::High)
            ).await?;
        
        if !validation_result.passed {
            warn!("Validation failed, using Swiss Ephemeris result as fallback");
            return Ok(swiss_result);
        }
        
        // Return the more precise result
        if native_result.precision >= swiss_result.precision {
            Ok(native_result)
        } else {
            Ok(swiss_result)
        }
    }

    fn post_process_result(&self, result: PanchangaResult) -> Result<PanchangaResult, EngineError> {
        // TODO: Implement result post-processing
        // - Format validation
        // - Unit conversion
        // - Cultural adjustments
        Ok(result)
    }

    fn parse_date_to_jd(&self, _date_str: &str) -> Result<f64, EngineError> {
        Self::parse_date_str_to_julian_day(_date_str)
    }


}

impl CalculationOrchestrator {
    fn parse_date_str_to_julian_day(date_str: &str) -> Result<f64, EngineError> {
        // Accept full RFC3339 timestamps (preferred) or simple YYYY-MM-DD dates.
        // This keeps existing API requests working while allowing time-precision
        // requests for Ghati boundary calculations.

        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(Self::datetime_utc_to_julian_day(dt.with_timezone(&Utc)));
        }

        // Common non-offset format: "YYYY-MM-DD HH:MM:SS" (assumed UTC)
        if let Ok(ndt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
            let dt = Utc.from_utc_datetime(&ndt);
            return Ok(Self::datetime_utc_to_julian_day(dt));
        }

        // Date-only format: "YYYY-MM-DD" (assumed UTC midnight)
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let ndt = date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| EngineError::ValidationError("Invalid date value".to_string()))?;
            let dt = Utc.from_utc_datetime(&ndt);
            return Ok(Self::datetime_utc_to_julian_day(dt));
        }

        Err(EngineError::ValidationError(format!(
            "Invalid date format. Use RFC3339 (e.g. 2026-01-31T12:34:56Z) or YYYY-MM-DD. Got: {}",
            date_str
        )))
    }

    fn datetime_utc_to_julian_day(dt: DateTime<Utc>) -> f64 {
        // JD at Unix epoch 1970-01-01T00:00:00Z
        const JD_UNIX_EPOCH: f64 = 2440587.5;
        let seconds = dt.timestamp() as f64;
        let nanos = dt.timestamp_subsec_nanos() as f64;
        JD_UNIX_EPOCH + (seconds + nanos / 1_000_000_000.0) / 86_400.0
    }
}

// Helper trait for cloning engines
trait CloneEngine {
    fn clone(&self) -> Self;
}

impl CloneEngine for NativeSolarEngine {
    fn clone(&self) -> Self {
        NativeSolarEngine::new()
    }
}

impl CloneEngine for NativeLunarEngine {
    fn clone(&self) -> Self {
        NativeLunarEngine::new()
    }
}

impl CloneEngine for SwissEphemerisEngine {
    fn clone(&self) -> Self {
        SwissEphemerisEngine::new(self.get_data_path().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jd_unix_epoch_is_correct() {
        let jd = CalculationOrchestrator::parse_date_str_to_julian_day("1970-01-01T00:00:00Z")
            .expect("parse");
        assert!((jd - 2440587.5).abs() < 1e-9);
    }

    #[test]
    fn jd_j2000_is_correct() {
        // J2000 epoch: 2000-01-01T12:00:00Z == JD 2451545.0
        let jd = CalculationOrchestrator::parse_date_str_to_julian_day("2000-01-01T12:00:00Z")
            .expect("parse");
        assert!((jd - 2451545.0).abs() < 1e-9);
    }

    #[test]
    fn date_only_is_midnight_utc() {
        // Midnight at 2000-01-01 is JD 2451544.5
        let jd = CalculationOrchestrator::parse_date_str_to_julian_day("2000-01-01")
            .expect("parse");
        assert!((jd - 2451544.5).abs() < 1e-9);
    }
}
