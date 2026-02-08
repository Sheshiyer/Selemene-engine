pub mod ghati_calculator;
pub mod panchanga_integration;
pub mod realtime_tracker;

pub use ghati_calculator::{
    GhatiCalculator, GhatiCalculatorFactory,
    GhatiCalculationMethod, GhatiPrecision, GhatiCalculationConfig,
    GhatiTime, GhatiBoundary, GhatiTransition, SolarTimeCorrection,
    FixedGhatiCalculator, HybridGhatiCalculator,
};

pub use panchanga_integration::{
    GhatiPanchangaService, GhatiPanchangaResult, GhatiPanchangaChange,
    PanchangaElement, PanchangaCalculator, MockPanchangaCalculator, EnginePanchangaCalculator,
};

pub use realtime_tracker::{
    GhatiRealtimeTracker, GhatiTrackingService, GhatiTrackingEvent, GhatiEventType,
    GhatiTrackerConfig, GhatiTrackerState,
};

/// Time conversion utilities
pub mod utils {
    use chrono::{DateTime, Utc, Duration};
    use crate::models::Coordinates;
    use super::GhatiTime;

    /// Convert UTC timestamp to Ghati time string
    pub fn utc_to_ghati_string(_utc_time: DateTime<Utc>, _location: Coordinates) -> String {
        // This is a placeholder implementation
        // TODO: Use actual GhatiCalculator
        format!("Ghati {}:{}:{}", 0, 0, 0)
    }

    /// Convert Ghati time string to UTC timestamp
    pub fn ghati_string_to_utc(_ghati_string: &str, _location: Coordinates) -> Result<DateTime<Utc>, String> {
        // This is a placeholder implementation
        // TODO: Use actual GhatiCalculator
        Ok(Utc::now())
    }

    /// Get current Ghati time
    pub fn get_current_ghati(location: Coordinates) -> GhatiTime {
        // This is a placeholder implementation
        // TODO: Use actual GhatiCalculator
        GhatiTime {
            ghati: 0,
            pala: 0,
            vipala: 0,
            utc_timestamp: Utc::now(),
            location,
            calculation_method: super::GhatiCalculationMethod::Fixed,
            precision: super::GhatiPrecision::Standard,
        }
    }

    /// Calculate time until next Ghati boundary
    pub fn time_until_next_ghati(_location: Coordinates) -> Duration {
        // This is a placeholder implementation
        // TODO: Use actual GhatiCalculator
        Duration::minutes(24)
    }
}
