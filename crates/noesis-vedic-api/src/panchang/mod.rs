//! Panchang (Vedic Almanac) Module
//!
//! This module provides comprehensive Panchang calculations including:
//! - Tithi (lunar day)
//! - Nakshatra (lunar mansion)
//! - Yoga (lunar-solar combinations)
//! - Karana (half-tithi)
//! - Vara (day of week)
//! - Muhurtas (auspicious time periods)
//! - Hora (planetary hours)
//! - Choghadiya (auspicious time windows)
//!
//! # Example
//! ```no_run
//! use noesis_vedic_api::panchang::{Panchang, PanchangQuery};
//!
//! // Create a Panchang query
//! let query = PanchangQuery::new(2024, 1, 1, 12.97, 77.59)
//!     .at(12, 0, 0)
//!     .with_timezone(5.5);
//!
//! // The query can then be used with the client
//! // let panchang = client.get_panchang_with_query(&query).await?;
//! ```

// Core data types
pub mod data;
pub mod dto;

// Muhurta calculations
pub mod muhurta;

// Hora (planetary hours)
pub mod hora;

// Choghadiya
pub mod choghadiya;

// Re-export main types for convenience
pub use choghadiya::*;
pub use data::*;
pub use hora::*;
pub use muhurta::*;

use crate::error::VedicApiResult;

/// Complete Panchang information including all sub-systems
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletePanchang {
    /// Core Panchang elements (Tithi, Nakshatra, Yoga, Karana, Vara)
    pub panchang: Panchang,
    /// Muhurtas for the day
    pub muhurtas: MuhurtaCollection,
    /// Hora timings
    pub hora_timings: HoraTimings,
    /// Choghadiya timings
    pub choghadiya: ChoghadiyaTimings,
    /// Additional metadata
    pub metadata: PanchangMetadata,
}

/// Metadata for Panchang calculations
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PanchangMetadata {
    /// Source of calculation (API, Native Engine, etc.)
    pub source: String,
    /// Calculation timestamp
    pub calculated_at: String,
    /// Ayanamsa used
    pub ayanamsa: String,
    /// Location timezone
    pub timezone: f64,
    /// Whether DST is active
    pub dst_active: bool,
}

impl CompletePanchang {
    /// Get a summary of all auspicious times today
    pub fn auspicious_times_summary(&self) -> AuspiciousTimesSummary {
        let mut good_muhrutas = vec![];
        let mut good_choghadiyas = vec![];
        let mut good_horas = vec![];

        // Collect good Muhurtas
        if let Some(ref amrit) = self.muhurtas.amrit_kaal {
            if amrit.nature.is_good_for_starting() {
                good_muhrutas.push((
                    "Amrit Kaal".to_string(),
                    amrit.start.clone(),
                    amrit.end.clone(),
                ));
            }
        }
        if let Some(ref abhijit) = self.muhurtas.abhijit {
            if abhijit.nature.is_good_for_starting() {
                good_muhrutas.push((
                    "Abhijit".to_string(),
                    abhijit.start.clone(),
                    abhijit.end.clone(),
                ));
            }
        }
        if let Some(ref brahma) = self.muhurtas.brahma_muhurta {
            if brahma.nature.is_good_for_starting() {
                good_muhrutas.push((
                    "Brahma Muhurta".to_string(),
                    brahma.start.clone(),
                    brahma.end.clone(),
                ));
            }
        }

        // Collect good Choghadiyas
        for choghadiya in &self.choghadiya.day {
            if choghadiya.nature.is_favorable() {
                good_choghadiyas.push((
                    choghadiya.name.as_str().to_string(),
                    choghadiya.start.clone(),
                    choghadiya.end.clone(),
                ));
            }
        }

        // Collect good Horas
        for hora in &self.hora_timings.day_horas {
            if hora.is_favorable {
                good_horas.push((
                    format!("{} Hora", hora.ruler.as_str()),
                    hora.start.clone(),
                    hora.end.clone(),
                ));
            }
        }

        AuspiciousTimesSummary {
            muhurtas: good_muhrutas,
            choghadiyas: good_choghadiyas,
            horas: good_horas,
        }
    }

    /// Get inauspicious times to avoid
    pub fn inauspicious_times_summary(&self) -> InauspiciousTimesSummary {
        let mut bad_muhrutas = vec![];
        let mut bad_choghadiyas = vec![];

        // Collect bad Muhurtas
        if let Some(ref rahu) = self.muhurtas.rahu_kalam {
            bad_muhrutas.push((
                "Rahu Kalam".to_string(),
                rahu.start.clone(),
                rahu.end.clone(),
            ));
        }
        if let Some(ref yama) = self.muhurtas.yama_gandam {
            bad_muhrutas.push((
                "Yama Gandam".to_string(),
                yama.start.clone(),
                yama.end.clone(),
            ));
        }
        if let Some(ref gulika) = self.muhurtas.gulika_kaal {
            bad_muhrutas.push((
                "Gulika Kaal".to_string(),
                gulika.start.clone(),
                gulika.end.clone(),
            ));
        }

        // Collect bad Choghadiyas
        for choghadiya in self
            .choghadiya
            .day
            .iter()
            .chain(self.choghadiya.night.iter())
        {
            if !choghadiya.nature.is_favorable() {
                bad_choghadiyas.push((
                    choghadiya.name.as_str().to_string(),
                    choghadiya.start.clone(),
                    choghadiya.end.clone(),
                ));
            }
        }

        InauspiciousTimesSummary {
            muhurtas: bad_muhrutas,
            choghadiyas: bad_choghadiyas,
        }
    }

    /// Check if a specific time is good for starting something new
    pub fn is_good_for_starting(&self, time: &str) -> bool {
        // Check Choghadiya
        if let Some(choghadiya) = self.choghadiya.get_current(time) {
            if !choghadiya.nature.is_favorable() {
                return false;
            }
        }

        // Check Muhurtas
        if let Some(ref rahu) = self.muhurtas.rahu_kalam {
            if time >= rahu.start.as_str() && time <= rahu.end.as_str() {
                return false;
            }
        }
        if let Some(ref yama) = self.muhurtas.yama_gandam {
            if time >= yama.start.as_str() && time <= yama.end.as_str() {
                return false;
            }
        }
        if let Some(ref gulika) = self.muhurtas.gulika_kaal {
            if time >= gulika.start.as_str() && time <= gulika.end.as_str() {
                return false;
            }
        }

        true
    }

    /// Get best time for a specific activity type
    pub fn best_time_for(
        &self,
        activity: choghadiya::ActivityCategory,
    ) -> Option<(String, String)> {
        // First check Choghadiya recommendations
        if let Some(best) = self.choghadiya.find_best_for_activity(activity) {
            return Some((best.start.clone(), best.end.clone()));
        }

        // Then check Muhurtas
        match activity {
            choghadiya::ActivityCategory::StartingNew
            | choghadiya::ActivityCategory::Business
            | choghadiya::ActivityCategory::Purchasing => {
                if let Some(ref abhijit) = self.muhurtas.abhijit {
                    return Some((abhijit.start.clone(), abhijit.end.clone()));
                }
            }
            choghadiya::ActivityCategory::Religious => {
                if let Some(ref brahma) = self.muhurtas.brahma_muhurta {
                    return Some((brahma.start.clone(), brahma.end.clone()));
                }
            }
            _ => {}
        }

        None
    }
}

/// Summary of auspicious times
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AuspiciousTimesSummary {
    pub muhurtas: Vec<(String, String, String)>, // name, start, end
    pub choghadiyas: Vec<(String, String, String)>,
    pub horas: Vec<(String, String, String)>,
}

/// Summary of inauspicious times
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InauspiciousTimesSummary {
    pub muhurtas: Vec<(String, String, String)>,
    pub choghadiyas: Vec<(String, String, String)>,
}

/// Builder for creating Panchang queries
#[derive(Debug, Clone, Default)]
pub struct PanchangQuery {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    pub include_muhurtas: bool,
    pub include_hora: bool,
    pub include_choghadiya: bool,
}

impl PanchangQuery {
    /// Create a new query for a specific date and location
    pub fn new(year: i32, month: u32, day: u32, lat: f64, lng: f64) -> Self {
        Self {
            year,
            month,
            day,
            latitude: lat,
            longitude: lng,
            hour: 12,
            minute: 0,
            second: 0,
            timezone: 5.5, // Default IST
            include_muhurtas: true,
            include_hora: true,
            include_choghadiya: true,
        }
    }

    /// Set the time of day
    pub fn at(mut self, hour: u32, minute: u32, second: u32) -> Self {
        self.hour = hour;
        self.minute = minute;
        self.second = second;
        self
    }

    /// Set the timezone
    pub fn with_timezone(mut self, tz: f64) -> Self {
        self.timezone = tz;
        self
    }

    /// Disable Muhurta calculations
    pub fn without_muhurtas(mut self) -> Self {
        self.include_muhurtas = false;
        self
    }

    /// Disable Hora calculations
    pub fn without_hora(mut self) -> Self {
        self.include_hora = false;
        self
    }

    /// Disable Choghadiya calculations
    pub fn without_choghadiya(mut self) -> Self {
        self.include_choghadiya = false;
        self
    }
}

/// Get the current Tithi from API response data.
///
/// Accepts the `tithi` object from a FreeAstrologyAPI panchang response.
/// Expected fields: `number` (u8), `name` (string), `start` (string), `end` (string), `paksha` (string).
pub fn parse_tithi_from_response(data: &serde_json::Value) -> VedicApiResult<Tithi> {
    let number = data
        .get("number")
        .and_then(|v| v.as_u64())
        .map(|n| n as u8)
        .unwrap_or(1);

    let name_str = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Pratipada");

    let name_tithi = TithiName::from_number(number as u32);

    let start_time = data
        .get("start")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let end_time = data
        .get("end")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let _ = name_str; // name is used for logging/debugging; struct uses enum

    Ok(Tithi {
        number,
        name_tithi,
        start_time,
        end_time,
        is_complete: true,
    })
}

/// Get the current Nakshatra from API response data.
///
/// Accepts the `nakshatra` object from a FreeAstrologyAPI panchang response.
/// Expected fields: `number` (u8), `name` (string), `pada` (u8), `start` (string), `end` (string), `longitude` (f64).
pub fn parse_nakshatra_from_response(data: &serde_json::Value) -> VedicApiResult<Nakshatra> {
    let number = data
        .get("number")
        .and_then(|v| v.as_u64())
        .map(|n| n as u8)
        .unwrap_or(1);

    let name_nakshatra = NakshatraName::from_number(number as u32);

    let pada = data
        .get("pada")
        .and_then(|v| v.as_u64())
        .map(|n| n as u8)
        .unwrap_or(1);

    let start_time = data
        .get("start")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let end_time = data
        .get("end")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let longitude = data
        .get("longitude")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    Ok(Nakshatra {
        number,
        name_nakshatra,
        pada,
        start_time,
        end_time,
        longitude,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panchang_query_builder() {
        let query = PanchangQuery::new(2024, 1, 15, 12.97, 77.59)
            .at(14, 30, 0)
            .with_timezone(5.5)
            .without_hora();

        assert_eq!(query.year, 2024);
        assert_eq!(query.hour, 14);
        assert!(!query.include_hora);
        assert!(query.include_muhurtas);
    }

    #[test]
    fn test_parse_tithi_from_response() {
        let data = serde_json::json!({
            "number": 5,
            "name": "Panchami",
            "start": "06:30",
            "end": "08:45",
            "paksha": "Shukla"
        });
        let tithi = parse_tithi_from_response(&data).unwrap();
        assert_eq!(tithi.number, 5);
        assert_eq!(tithi.name(), "Panchami");
        assert_eq!(tithi.start_time, "06:30");
        assert_eq!(tithi.end_time, "08:45");
        assert!(tithi.is_auspicious());
    }

    #[test]
    fn test_parse_tithi_missing_fields() {
        let data = serde_json::json!({});
        let tithi = parse_tithi_from_response(&data).unwrap();
        assert_eq!(tithi.number, 1);
        assert_eq!(tithi.name(), "Pratipada");
    }

    #[test]
    fn test_parse_nakshatra_from_response() {
        let data = serde_json::json!({
            "number": 4,
            "name": "Rohini",
            "pada": 2,
            "start": "10:00",
            "end": "12:30",
            "longitude": 46.5
        });
        let nakshatra = parse_nakshatra_from_response(&data).unwrap();
        assert_eq!(nakshatra.number, 4);
        assert_eq!(nakshatra.name(), "Rohini");
        assert_eq!(nakshatra.pada, 2);
        assert_eq!(nakshatra.start_time, "10:00");
        assert_eq!(nakshatra.end_time, "12:30");
        assert!((nakshatra.longitude - 46.5).abs() < f64::EPSILON);
        assert!(nakshatra.is_auspicious());
    }

    #[test]
    fn test_parse_nakshatra_missing_fields() {
        let data = serde_json::json!({});
        let nakshatra = parse_nakshatra_from_response(&data).unwrap();
        assert_eq!(nakshatra.number, 1);
        assert_eq!(nakshatra.name(), "Ashwini");
        assert_eq!(nakshatra.pada, 1);
    }
}
