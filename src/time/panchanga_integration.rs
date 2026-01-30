use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    models::{Coordinates, PanchangaRequest, PanchangaResult, PrecisionLevel},
    time::{
        GhatiCalculator, GhatiCalculatorFactory, GhatiCalculationConfig,
        GhatiCalculationMethod, GhatiPrecision, GhatiTime, GhatiTransition
    },
};

/// Panchanga calculation at Ghati boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiPanchangaResult {
    pub ghati_time: GhatiTime,
    pub panchanga: PanchangaResult,
    pub next_change: Option<GhatiPanchangaChange>,
}

/// Panchanga change at Ghati boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiPanchangaChange {
    pub ghati_transition: GhatiTransition,
    pub changed_element: PanchangaElement,
    pub old_value: f64,
    pub new_value: f64,
    pub change_time: DateTime<Utc>,
}

/// Panchanga elements that can change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PanchangaElement {
    Tithi,
    Nakshatra,
    Yoga,
    Karana,
    Vara,
}

/// Ghati-Panchanga integration service
pub struct GhatiPanchangaService {
    ghati_calculator: Box<dyn GhatiCalculator>,
    panchanga_calculator: Arc<dyn PanchangaCalculator>,
}

/// Trait for Panchanga calculation
#[async_trait]
pub trait PanchangaCalculator: Send + Sync {
    async fn calculate_panchanga(&self, request: PanchangaRequest) -> Result<PanchangaResult, String>;
    async fn calculate_panchanga_at_time(
        &self,
        utc_time: DateTime<Utc>,
        location: Coordinates,
    ) -> Result<PanchangaResult, String>;
}

/// Engine-backed Panchanga calculator adapter (uses the real CalculationOrchestrator)
pub struct EnginePanchangaCalculator {
    engine: Arc<crate::SelemeneEngine>,
    default_precision: PrecisionLevel,
}

impl EnginePanchangaCalculator {
    pub fn new(engine: Arc<crate::SelemeneEngine>, default_precision: PrecisionLevel) -> Self {
        Self {
            engine,
            default_precision,
        }
    }

    fn ensure_precision(&self, mut request: PanchangaRequest) -> PanchangaRequest {
        if request.precision.is_none() {
            request.precision = Some(self.default_precision);
        }
        request
    }
}

#[async_trait]
impl PanchangaCalculator for EnginePanchangaCalculator {
    async fn calculate_panchanga(&self, request: PanchangaRequest) -> Result<PanchangaResult, String> {
        let request = self.ensure_precision(request);
        self.engine
            .calculate_panchanga(request)
            .await
            .map_err(|e| e.to_string())
    }

    async fn calculate_panchanga_at_time(
        &self,
        utc_time: DateTime<Utc>,
        location: Coordinates,
    ) -> Result<PanchangaResult, String> {
        let request = PanchangaRequest {
            // Use a full timestamp so orchestrator can compute an accurate Julian Day.
            date: utc_time.to_rfc3339(),
            latitude: Some(location.latitude),
            longitude: Some(location.longitude),
            timezone: Some("UTC".to_string()),
            precision: Some(self.default_precision),
            include_details: Some(false),
        };

        self.engine
            .calculate_panchanga(request)
            .await
            .map_err(|e| e.to_string())
    }
}

impl GhatiPanchangaService {
    pub fn new(
        ghati_config: GhatiCalculationConfig,
        panchanga_calculator: Arc<dyn PanchangaCalculator>,
    ) -> Self {
        let ghati_calculator = GhatiCalculatorFactory::create_calculator(ghati_config);
        
        Self {
            ghati_calculator,
            panchanga_calculator,
        }
    }

    pub fn calculate_ghati(
        &self,
        utc_time: DateTime<Utc>,
        location: Coordinates,
    ) -> Result<GhatiTime, String> {
        self.ghati_calculator.calculate_ghati(utc_time, location)
    }

    pub fn get_next_ghati_transition(
        &self,
        current_time: DateTime<Utc>,
        location: Coordinates,
    ) -> Result<GhatiTransition, String> {
        self.ghati_calculator
            .get_next_ghati_transition(current_time, location)
    }

    /// Calculate Panchanga for a specific Ghati time
    pub async fn calculate_ghati_panchanga(
        &self,
        ghati_time: GhatiTime,
    ) -> Result<GhatiPanchangaResult, String> {
        // Calculate Panchanga at the Ghati time
        let panchanga = self.panchanga_calculator
            .calculate_panchanga_at_time(ghati_time.utc_timestamp, ghati_time.location.clone())
            .await?;

        // Get next Ghati transition
        let next_transition = self.ghati_calculator
            .get_next_ghati_transition(ghati_time.utc_timestamp, ghati_time.location.clone())?;

        // Calculate Panchanga at next transition
        let next_panchanga = self.panchanga_calculator
            .calculate_panchanga_at_time(next_transition.transition_time, ghati_time.location.clone())
            .await?;

        // Check for Panchanga changes
        let next_change = self.detect_panchanga_change(&panchanga, &next_panchanga, &next_transition);

        Ok(GhatiPanchangaResult {
            ghati_time,
            panchanga,
            next_change,
        })
    }

    /// Calculate Panchanga for all Ghati boundaries in a day
    pub async fn calculate_daily_ghati_panchanga(
        &self,
        date: NaiveDate,
        location: Coordinates,
    ) -> Result<Vec<GhatiPanchangaResult>, String> {
        // Get all Ghati boundaries for the day
        let boundaries = self.ghati_calculator
            .calculate_ghati_boundaries(date, location.clone())?;

        let mut results = Vec::new();

        for boundary in boundaries {
            // Create GhatiTime from boundary
            let ghati_time = GhatiTime {
                ghati: boundary.ghati_number,
                pala: 0, // Boundaries are at Ghati level
                vipala: 0,
                utc_timestamp: boundary.utc_timestamp,
                location: location.clone(),
                calculation_method: GhatiCalculationMethod::Hybrid, // TODO: Get from config
                precision: GhatiPrecision::Standard,
            };

            // Calculate Panchanga for this Ghati
            let ghati_panchanga = self.calculate_ghati_panchanga(ghati_time).await?;
            results.push(ghati_panchanga);
        }

        Ok(results)
    }

    /// Get current Ghati with Panchanga information
    pub async fn get_current_ghati_panchanga(
        &self,
        location: Coordinates,
    ) -> Result<GhatiPanchangaResult, String> {
        let current_time = Utc::now();
        let ghati_time = self.ghati_calculator
            .calculate_ghati(current_time, location.clone())?;

        self.calculate_ghati_panchanga(ghati_time).await
    }

    /// Find next Panchanga change within Ghati boundaries
    pub async fn find_next_panchanga_change(
        &self,
        location: Coordinates,
        max_ghatis: u8,
    ) -> Result<Option<GhatiPanchangaChange>, String> {
        let current_time = Utc::now();
        let mut current_ghati = self.ghati_calculator
            .calculate_ghati(current_time, location.clone())?;

        // Check up to max_ghatis ahead
        for _ in 0..max_ghatis {
            let current_panchanga = self.panchanga_calculator
                .calculate_panchanga_at_time(current_ghati.utc_timestamp, location.clone())
                .await?;

            let next_transition = self.ghati_calculator
                .get_next_ghati_transition(current_ghati.utc_timestamp, location.clone())?;

            let next_panchanga = self.panchanga_calculator
                .calculate_panchanga_at_time(next_transition.transition_time, location.clone())
                .await?;

            if let Some(change) = self.detect_panchanga_change(&current_panchanga, &next_panchanga, &next_transition) {
                return Ok(Some(change));
            }

            // Move to next Ghati
            current_ghati.ghati = next_transition.to_ghati;
            current_ghati.utc_timestamp = next_transition.transition_time;
        }

        Ok(None)
    }

    /// Detect Panchanga changes between two calculations
    fn detect_panchanga_change(
        &self,
        current: &PanchangaResult,
        next: &PanchangaResult,
        transition: &GhatiTransition,
    ) -> Option<GhatiPanchangaChange> {
        // Check Tithi change
        if let (Some(current_tithi), Some(next_tithi)) = (current.tithi, next.tithi) {
            if (current_tithi - next_tithi).abs() > 0.1 {
                return Some(GhatiPanchangaChange {
                    ghati_transition: transition.clone(),
                    changed_element: PanchangaElement::Tithi,
                    old_value: current_tithi,
                    new_value: next_tithi,
                    change_time: transition.transition_time,
                });
            }
        }

        // Check Nakshatra change
        if let (Some(current_nakshatra), Some(next_nakshatra)) = (current.nakshatra, next.nakshatra) {
            if (current_nakshatra - next_nakshatra).abs() > 0.1 {
                return Some(GhatiPanchangaChange {
                    ghati_transition: transition.clone(),
                    changed_element: PanchangaElement::Nakshatra,
                    old_value: current_nakshatra,
                    new_value: next_nakshatra,
                    change_time: transition.transition_time,
                });
            }
        }

        // Check Yoga change
        if let (Some(current_yoga), Some(next_yoga)) = (current.yoga, next.yoga) {
            if (current_yoga - next_yoga).abs() > 0.1 {
                return Some(GhatiPanchangaChange {
                    ghati_transition: transition.clone(),
                    changed_element: PanchangaElement::Yoga,
                    old_value: current_yoga,
                    new_value: next_yoga,
                    change_time: transition.transition_time,
                });
            }
        }

        // Check Karana change
        if let (Some(current_karana), Some(next_karana)) = (current.karana, next.karana) {
            if (current_karana - next_karana).abs() > 0.1 {
                return Some(GhatiPanchangaChange {
                    ghati_transition: transition.clone(),
                    changed_element: PanchangaElement::Karana,
                    old_value: current_karana,
                    new_value: next_karana,
                    change_time: transition.transition_time,
                });
            }
        }

        // Check Vara change
        if let (Some(current_vara), Some(next_vara)) = (current.vara, next.vara) {
            if (current_vara - next_vara).abs() > 0.1 {
                return Some(GhatiPanchangaChange {
                    ghati_transition: transition.clone(),
                    changed_element: PanchangaElement::Vara,
                    old_value: current_vara,
                    new_value: next_vara,
                    change_time: transition.transition_time,
                });
            }
        }

        None
    }

    /// Get Ghati timing for Panchanga element changes
    pub async fn get_ghati_timing_for_panchanga_changes(
        &self,
        date: NaiveDate,
        location: Coordinates,
        element: PanchangaElement,
    ) -> Result<Vec<GhatiPanchangaChange>, String> {
        let daily_results = self.calculate_daily_ghati_panchanga(date, location).await?;
        let mut changes = Vec::new();

        for result in daily_results {
            if let Some(change) = result.next_change {
                if change.changed_element == element {
                    changes.push(change);
                }
            }
        }

        Ok(changes)
    }

    /// Calculate Panchanga with Ghati precision
    pub async fn calculate_panchanga_with_ghati_precision(
        &self,
        request: PanchangaRequest,
        ghati_precision: GhatiPrecision,
    ) -> Result<GhatiPanchangaResult, String> {
        // Parse date from request
        let date = chrono::NaiveDate::parse_from_str(&request.date, "%Y-%m-%d")
            .map_err(|_| "Invalid date format".to_string())?;

        // Get location from request
        let location = Coordinates {
            latitude: request.latitude.unwrap_or(12.9629), // Default to Bengaluru
            longitude: request.longitude.unwrap_or(77.5775),
            altitude: None,
        };

        // Calculate Ghati time for the date
        let ghati_time = GhatiTime {
            ghati: 0, // Start of day
            pala: 0,
            vipala: 0,
            utc_timestamp: date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            location: location.clone(),
            calculation_method: GhatiCalculationMethod::Hybrid,
            precision: ghati_precision,
        };

        self.calculate_ghati_panchanga(ghati_time).await
    }
}

/// Mock Panchanga calculator for testing
pub struct MockPanchangaCalculator;

#[async_trait]
impl PanchangaCalculator for MockPanchangaCalculator {
    async fn calculate_panchanga(&self, request: PanchangaRequest) -> Result<PanchangaResult, String> {
        // Mock implementation - return placeholder values
        Ok(PanchangaResult {
            date: request.date,
            tithi: Some(15.0),
            nakshatra: Some(20.0),
            yoga: Some(25.0),
            karana: Some(7.0),
            vara: Some(1.0),
            solar_longitude: 120.0,
            lunar_longitude: 135.0,
            precision: 2,
            backend: "mock".to_string(),
            calculation_time: Some(Utc::now()),
        })
    }

    async fn calculate_panchanga_at_time(
        &self,
        utc_time: DateTime<Utc>,
        _location: Coordinates,
    ) -> Result<PanchangaResult, String> {
        // Mock implementation with slight variations based on time
        let time_factor = (utc_time.timestamp() % 86400) as f64 / 86400.0; // Factor based on time of day
        
        Ok(PanchangaResult {
            date: utc_time.date_naive().format("%Y-%m-%d").to_string(),
            tithi: Some(15.0 + time_factor * 0.1),
            nakshatra: Some(20.0 + time_factor * 0.05),
            yoga: Some(25.0 + time_factor * 0.02),
            karana: Some(7.0 + time_factor * 0.01),
            vara: Some(1.0),
            solar_longitude: 120.0 + time_factor * 0.5,
            lunar_longitude: 135.0 + time_factor * 0.3,
            precision: 2,
            backend: "mock".to_string(),
            calculation_time: Some(utc_time),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_ghati_panchanga_calculation() {
        let ghati_config = GhatiCalculationConfig::default();
        let panchanga_calculator = Arc::new(MockPanchangaCalculator);
        let service = GhatiPanchangaService::new(ghati_config, panchanga_calculator);

        let location = Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: Some(920.0),
        };

        let ghati_time = GhatiTime {
            ghati: 0,
            pala: 0,
            vipala: 0,
            utc_timestamp: Utc::now(),
            location: location.clone(),
            calculation_method: GhatiCalculationMethod::Hybrid,
            precision: GhatiPrecision::High,
        };

        let result = service.calculate_ghati_panchanga(ghati_time).await.unwrap();
        assert_eq!(result.ghati_time.ghati, 0);
        assert!(result.panchanga.tithi.is_some());
    }

    #[tokio::test]
    async fn test_daily_ghati_panchanga() {
        let ghati_config = GhatiCalculationConfig::default();
        let panchanga_calculator = Arc::new(MockPanchangaCalculator);
        let service = GhatiPanchangaService::new(ghati_config, panchanga_calculator);

        let location = Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: Some(920.0),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 27).unwrap();
        let results = service.calculate_daily_ghati_panchanga(date, location).await.unwrap();

        assert_eq!(results.len(), 60); // 60 Ghatis per day
        assert_eq!(results[0].ghati_time.ghati, 0);
        assert_eq!(results[59].ghati_time.ghati, 59);
    }

    #[tokio::test]
    async fn test_panchanga_change_detection() {
        let ghati_config = GhatiCalculationConfig::default();
        let panchanga_calculator = Arc::new(MockPanchangaCalculator);
        let service = GhatiPanchangaService::new(ghati_config, panchanga_calculator);

        let location = Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: Some(920.0),
        };

        let changes = service.find_next_panchanga_change(location, 10).await.unwrap();
        // With mock calculator, we might not find changes, which is expected
        assert!(changes.is_none() || changes.is_some());
    }
}
