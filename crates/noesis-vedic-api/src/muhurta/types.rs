//! Muhurta calculation types
//!
//! FAPI-081: Define Muhurta calculation types

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

/// Type of activity for Muhurta selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MuhurtaActivity {
    Marriage,
    Business,
    Travel,
    Education,
    Medical,
    Construction,
    Religious,
    JourneyStart,
    NewVenture,
    Interview,
    PropertyPurchase,
    VehiclePurchase,
    MovingHouse,
    General,
}

impl std::fmt::Display for MuhurtaActivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MuhurtaActivity::Marriage => write!(f, "Marriage"),
            MuhurtaActivity::Business => write!(f, "Business"),
            MuhurtaActivity::Travel => write!(f, "Travel"),
            MuhurtaActivity::Education => write!(f, "Education"),
            MuhurtaActivity::Medical => write!(f, "Medical"),
            MuhurtaActivity::Construction => write!(f, "Construction"),
            MuhurtaActivity::Religious => write!(f, "Religious"),
            MuhurtaActivity::JourneyStart => write!(f, "Journey Start"),
            MuhurtaActivity::NewVenture => write!(f, "New Venture"),
            MuhurtaActivity::Interview => write!(f, "Interview"),
            MuhurtaActivity::PropertyPurchase => write!(f, "Property Purchase"),
            MuhurtaActivity::VehiclePurchase => write!(f, "Vehicle Purchase"),
            MuhurtaActivity::MovingHouse => write!(f, "Moving House"),
            MuhurtaActivity::General => write!(f, "General"),
        }
    }
}

/// Quality of a Muhurta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MuhurtaQuality {
    Excellent,
    Good,
    Average,
    NotRecommended,
    Avoid,
}

impl std::fmt::Display for MuhurtaQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MuhurtaQuality::Excellent => write!(f, "Excellent"),
            MuhurtaQuality::Good => write!(f, "Good"),
            MuhurtaQuality::Average => write!(f, "Average"),
            MuhurtaQuality::NotRecommended => write!(f, "Not Recommended"),
            MuhurtaQuality::Avoid => write!(f, "Avoid"),
        }
    }
}

/// A selected Muhurta (auspicious time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedMuhurta {
    /// Start time of the Muhurta
    pub start_time: NaiveDateTime,
    /// End time
    pub end_time: NaiveDateTime,
    /// Quality rating
    pub quality: MuhurtaQuality,
    /// Tithi during this period
    pub tithi: String,
    /// Nakshatra during this period
    pub nakshatra: String,
    /// Yoga during this period
    pub yoga: String,
    /// Karana during this period
    pub karana: String,
    /// Day lord (Vara)
    pub vara: String,
    /// Score out of 100
    pub score: u8,
    /// Favorable factors
    pub favorable_factors: Vec<String>,
    /// Unfavorable factors
    pub unfavorable_factors: Vec<String>,
    /// Specific recommendation
    pub recommendation: String,
}

/// Muhurta search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuhurtaSearchCriteria {
    /// Activity type
    pub activity: MuhurtaActivity,
    /// Search start date
    pub from_date: NaiveDate,
    /// Search end date
    pub to_date: NaiveDate,
    /// Preferred time of day
    pub preferred_time: Option<TimePreference>,
    /// Location latitude
    pub latitude: f64,
    /// Location longitude
    pub longitude: f64,
    /// Timezone
    pub timezone: f64,
    /// Minimum quality required
    pub min_quality: MuhurtaQuality,
}

/// Time preference for Muhurta
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimePreference {
    Morning,
    Afternoon,
    Evening,
    Night,
    Any,
}

/// Collection of Muhurtas for a date range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuhurtaResults {
    /// Activity searched for
    pub activity: MuhurtaActivity,
    /// Date range
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    /// Best muhurtas found
    pub muhurtas: Vec<SelectedMuhurta>,
    /// Count of excellent muhurtas
    pub excellent_count: usize,
    /// Count of good muhurtas
    pub good_count: usize,
    /// General advice
    pub advice: String,
}

/// Dosha (negative factors) in a Muhurta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MuhurtaDosha {
    /// Rahu Kalam period
    RahuKalam,
    /// Yama Gandam period
    YamaGandam,
    /// Gulika Kaal period
    GulikaKaal,
    /// Varjyam period
    Varjyam,
    /// Dur Muhurta
    DurMuhurta,
    /// Inauspicious Tithi
    BadTithi,
    /// Inauspicious Nakshatra
    BadNakshatra,
    /// Moon void of course
    VoidMoon,
}

impl std::fmt::Display for MuhurtaDosha {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MuhurtaDosha::RahuKalam => write!(f, "Rahu Kalam"),
            MuhurtaDosha::YamaGandam => write!(f, "Yama Gandam"),
            MuhurtaDosha::GulikaKaal => write!(f, "Gulika Kaal"),
            MuhurtaDosha::Varjyam => write!(f, "Varjyam"),
            MuhurtaDosha::DurMuhurta => write!(f, "Dur Muhurta"),
            MuhurtaDosha::BadTithi => write!(f, "Inauspicious Tithi"),
            MuhurtaDosha::BadNakshatra => write!(f, "Inauspicious Nakshatra"),
            MuhurtaDosha::VoidMoon => write!(f, "Void Moon"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_display() {
        assert_eq!(MuhurtaActivity::Marriage.to_string(), "Marriage");
        assert_eq!(MuhurtaActivity::Business.to_string(), "Business");
    }

    #[test]
    fn test_quality_display() {
        assert_eq!(MuhurtaQuality::Excellent.to_string(), "Excellent");
        assert_eq!(MuhurtaQuality::Avoid.to_string(), "Avoid");
    }

    #[test]
    fn test_dosha_display() {
        assert_eq!(MuhurtaDosha::RahuKalam.to_string(), "Rahu Kalam");
    }
}
