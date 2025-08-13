use crate::models::{PanchangaRequest, PanchangaResult, EngineError, PrecisionLevel};
use crate::EngineConfig;
use super::{
    HybridBackend, NativeSolarEngine, NativeLunarEngine, 
    SwissEphemerisEngine, ValidationEngine, Backend
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Main calculation orchestrator that coordinates all engines
pub struct CalculationOrchestrator {
    hybrid_backend: HybridBackend,
    native_solar: NativeSolarEngine,
    native_lunar: NativeLunarEngine,
    swiss_ephemeris: SwissEphemerisEngine,
    validation_engine: ValidationEngine,
    config: Arc<RwLock<EngineConfig>>,
}

impl CalculationOrchestrator {
    pub fn new(config: Arc<RwLock<EngineConfig>>) -> Self {
        let native_solar = NativeSolarEngine::new();
        let native_lunar = NativeLunarEngine::new();
        
        let swiss_ephemeris_path = {
            let config_guard = config.blocking_read();
            config_guard.engines.swiss_ephemeris.data_path.clone()
        };
        let mut swiss_ephemeris = SwissEphemerisEngine::new(swiss_ephemeris_path);
        
        // Initialize Swiss Ephemeris
        if let Err(e) = swiss_ephemeris.initialize() {
            warn!("Failed to initialize Swiss Ephemeris: {}", e);
        }
        
        let validation_engine = ValidationEngine::new(
            native_solar.clone(),
            native_lunar.clone(),
            swiss_ephemeris.clone(),
        );

        Self {
            hybrid_backend: HybridBackend::new(config.clone()),
            native_solar,
            native_lunar,
            swiss_ephemeris,
            validation_engine,
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
        
        // Calculate Tithi
        let tithi = self.calculate_tithi(solar_longitude, lunar_longitude);
        
        // TODO: Calculate other Panchanga elements
        // - Nakshatra
        // - Yoga
        // - Karana
        // - Vara (weekday)
        
        Ok(PanchangaResult {
            date: request.date.clone(),
            tithi: Some(tithi),
            nakshatra: None, // TODO: Implement
            yoga: None,       // TODO: Implement
            karana: None,     // TODO: Implement
            vara: None,       // TODO: Implement
            solar_longitude,
            lunar_longitude,
            precision: precision as u8,
            backend: "native".to_string(),
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
        let solar_position = self.swiss_ephemeris.calculate_solar_position(jd, precision)?;
        
        // Calculate lunar position
        let lunar_position = self.swiss_ephemeris.calculate_lunar_position(jd, precision)?;
        
        // Calculate Tithi
        let tithi = self.calculate_tithi(solar_position.longitude, lunar_position.longitude);
        
        Ok(PanchangaResult {
            date: request.date.clone(),
            tithi: Some(tithi),
            nakshatra: None, // TODO: Implement
            yoga: None,       // TODO: Implement
            karana: None,     // TODO: Implement
            vara: None,       // TODO: Implement
            solar_longitude: solar_position.longitude,
            lunar_longitude: lunar_position.longitude,
            precision: precision as u8,
            backend: "swiss".to_string(),
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

    fn parse_date_to_jd(&self, date_str: &str) -> Result<f64, EngineError> {
        // TODO: Implement proper date parsing to Julian Day
        // For now, return a placeholder
        Ok(2451545.0) // J2000
    }

    fn calculate_tithi(&self, solar_longitude: f64, lunar_longitude: f64) -> f64 {
        let diff = (lunar_longitude - solar_longitude).rem_euclid(360.0);
        (diff / 12.0).floor() + 1.0 // Tithi 1-30
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
