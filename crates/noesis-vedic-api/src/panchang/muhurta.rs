//! Muhurta calculations - Auspicious and inauspicious time periods
//!
//! Muhurtas are specific time windows during the day that are considered
//! favorable or unfavorable for various activities.

use serde::{Deserialize, Serialize};
use chrono::NaiveTime;

/// Collection of all Muhurta timings for a day
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MuhurtaCollection {
    /// Abhijit Muhurta - Victorious midday (highly auspicious)
    pub abhijit: Option<Muhurta>,
    /// Amrit Kaal - Nectar time (auspicious for beginnings)
    pub amrit_kaal: Option<Muhurta>,
    /// Rahu Kalam - Rahu period (inauspicious, avoid new ventures)
    pub rahu_kalam: Option<Muhurta>,
    /// Yama Gandam - Yama period (inauspicious)
    pub yama_gandam: Option<Muhurta>,
    /// Gulika Kaal - Gulika rising time (inauspicious)
    pub gulika_kaal: Option<Muhurta>,
    /// Dur Muhurta - Bad time (avoid)
    pub dur_muhurta: Option<Muhurta>,
    /// Varjyam - Avoidable period
    pub varjyam: Option<Muhurta>,
    /// Brahma Muhurta - Creator's time (ideal for meditation)
    pub brahma_muhurta: Option<Muhurta>,
}

/// A single Muhurta time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Muhurta {
    /// Name of the Muhurta
    pub name: String,
    /// Start time (HH:MM format)
    pub start: String,
    /// End time (HH:MM format)
    pub end: String,
    /// Duration in minutes
    pub duration_minutes: u32,
    /// Nature: auspicious, inauspicious, mixed
    pub nature: MuhurtaNature,
    /// Deity or planetary ruler
    pub ruler: String,
    /// Activities suitable for this Muhurta
    pub suitable_activities: Vec<String>,
    /// Activities to avoid
    pub avoid_activities: Vec<String>,
}

/// Nature of a Muhurta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MuhurtaNature {
    /// Highly favorable
    Auspicious,
    /// Favorable
    Favorable,
    /// Mixed results
    Mixed,
    /// Unfavorable
    Inauspicious,
    /// Highly unfavorable
    VeryInauspicious,
}

impl MuhurtaNature {
    pub fn as_str(&self) -> &'static str {
        match self {
            MuhurtaNature::Auspicious => "auspicious",
            MuhurtaNature::Favorable => "favorable",
            MuhurtaNature::Mixed => "mixed",
            MuhurtaNature::Inauspicious => "inauspicious",
            MuhurtaNature::VeryInauspicious => "very_inauspicious",
        }
    }
    
    /// Check if this Muhurta is good for starting new activities
    pub fn is_good_for_starting(&self) -> bool {
        matches!(self, MuhurtaNature::Auspicious | MuhurtaNature::Favorable)
    }
    
    /// Check if this Muhurta should be avoided
    pub fn should_avoid(&self) -> bool {
        matches!(self, MuhurtaNature::Inauspicious | MuhurtaNature::VeryInauspicious)
    }
}

/// Abhijit Muhurta - The victorious midday period
/// 
/// This is one of the most auspicious Muhurtas, occurring around midday.
/// It is considered favorable for all activities, especially important beginnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbhijitMuhurta {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub quality: String,
}

impl AbhijitMuhurta {
    /// Create with default quality description
    pub fn new(start: String, end: String, duration: u32) -> Self {
        Self {
            start,
            end,
            duration_minutes: duration,
            quality: "Victorious - favorable for all activities".to_string(),
        }
    }
}

/// Rahu Kalam - Rahu period
/// 
/// This period is ruled by Rahu (North Node) and is considered
/// inauspicious for starting new ventures. It varies by day of week.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RahuKalam {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub day_of_week: String,
}

impl RahuKalam {
    /// Get Rahu Kalam times for each day of the week
    /// 
    /// Rahu Kalam is approximately 1.5 hours long and occurs at different times:
    /// - Sunday: 4:30 PM - 6:00 PM
    /// - Monday: 7:30 AM - 9:00 AM
    /// - Tuesday: 3:00 PM - 4:30 PM
    /// - Wednesday: 12:00 PM - 1:30 PM
    /// - Thursday: 1:30 PM - 3:00 PM
    /// - Friday: 10:30 AM - 12:00 PM
    /// - Saturday: 9:00 AM - 10:30 AM
    pub fn for_day(day: &str, sunrise: &str, sunset: &str) -> Self {
        // Simplified calculation - actual implementation would use sunrise/sunset
        let (start, end) = match day.to_lowercase().as_str() {
            "sunday" | "sun" => ("16:30", "18:00"),
            "monday" | "mon" => ("07:30", "09:00"),
            "tuesday" | "tue" => ("15:00", "16:30"),
            "wednesday" | "wed" => ("12:00", "13:30"),
            "thursday" | "thu" => ("13:30", "15:00"),
            "friday" | "fri" => ("10:30", "12:00"),
            "saturday" | "sat" => ("09:00", "10:30"),
            _ => ("12:00", "13:30"), // Default to Wednesday
        };
        
        Self {
            start: start.to_string(),
            end: end.to_string(),
            duration_minutes: 90,
            day_of_week: day.to_string(),
        }
    }
}

/// Yama Gandam - Yama period
/// 
/// Ruled by Yama (lord of death), this period should be avoided
/// for important activities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamaGandam {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub day_of_week: String,
}

impl YamaGandam {
    /// Get Yama Gandam for each day
    pub fn for_day(day: &str) -> Self {
        let (start, end) = match day.to_lowercase().as_str() {
            "sunday" | "sun" => ("12:00", "13:30"),
            "monday" | "mon" => ("10:30", "12:00"),
            "tuesday" | "tue" => ("09:00", "10:30"),
            "wednesday" | "wed" => ("07:30", "09:00"),
            "thursday" | "thu" => ("06:00", "07:30"),
            "friday" | "fri" => ("15:00", "16:30"),
            "saturday" | "sat" => ("13:30", "15:00"),
            _ => ("12:00", "13:30"),
        };
        
        Self {
            start: start.to_string(),
            end: end.to_string(),
            duration_minutes: 90,
            day_of_week: day.to_string(),
        }
    }
}

/// Gulika Kaal - Gulika period
/// 
/// Gulika is the son of Saturn and this period is considered inauspicious.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GulikaKaal {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub day_of_week: String,
}

impl GulikaKaal {
    pub fn for_day(day: &str) -> Self {
        let (start, end) = match day.to_lowercase().as_str() {
            "sunday" | "sun" => ("15:00", "16:30"),
            "monday" | "mon" => ("13:30", "15:00"),
            "tuesday" | "tue" => ("12:00", "13:30"),
            "wednesday" | "wed" => ("10:30", "12:00"),
            "thursday" | "thu" => ("09:00", "10:30"),
            "friday" | "fri" => ("07:30", "09:00"),
            "saturday" | "sat" => ("06:00", "07:30"),
            _ => ("12:00", "13:30"),
        };
        
        Self {
            start: start.to_string(),
            end: end.to_string(),
            duration_minutes: 90,
            day_of_week: day.to_string(),
        }
    }
}

/// Brahma Muhurta - Creator's time
/// 
/// Approximately 1 hour 36 minutes before sunrise.
/// Considered the most auspicious time for meditation and spiritual practices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrahmaMuhurta {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub description: String,
}

impl BrahmaMuhurta {
    /// Calculate Brahma Muhurta based on sunrise time
    pub fn from_sunrise(sunrise: &str) -> Self {
        // Brahma Muhurta is ~96 minutes (1 hour 36 minutes) before sunrise
        // Simplified - actual implementation would parse time and subtract
        Self {
            start: "04:24".to_string(), // Example for 6:00 AM sunrise
            end: sunrise.to_string(),
            duration_minutes: 96,
            description: "Creator's time - ideal for meditation and spiritual practices".to_string(),
        }
    }
}

/// Amrit Kaal - Nectar time
/// 
/// A highly auspicious period for starting new ventures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmritKaal {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub quality: String,
}

impl AmritKaal {
    pub fn new(start: String, end: String, duration: u32) -> Self {
        Self {
            start,
            end,
            duration_minutes: duration,
            quality: "Nectar time - highly auspicious for beginnings".to_string(),
        }
    }
}

/// Helper functions for Muhurta calculations
pub mod utils {
    use super::*;
    
    /// Check if current time falls within a Muhurta
    pub fn is_within_muhurta(current_time: &str, muhurta: &Muhurta) -> bool {
        current_time >= muhurta.start.as_str() && current_time <= muhurta.end.as_str()
    }
    
    /// Get all inauspicious periods for the day
    pub fn get_inauspicious_periods(muhurtas: &MuhurtaCollection) -> Vec<&Muhurta> {
        let mut periods = Vec::new();
        
        if let Some(ref rahu) = muhurtas.rahu_kalam {
            if rahu.nature.should_avoid() {
                periods.push(rahu);
            }
        }
        if let Some(ref yama) = muhurtas.yama_gandam {
            if yama.nature.should_avoid() {
                periods.push(yama);
            }
        }
        if let Some(ref gulika) = muhurtas.gulika_kaal {
            if gulika.nature.should_avoid() {
                periods.push(gulika);
            }
        }
        if let Some(ref dur) = muhurtas.dur_muhurta {
            if dur.nature.should_avoid() {
                periods.push(dur);
            }
        }
        if let Some(ref varjyam) = muhurtas.varjyam {
            if varjyam.nature.should_avoid() {
                periods.push(varjyam);
            }
        }
        
        periods
    }
    
    /// Get all auspicious periods for the day
    pub fn get_auspicious_periods(muhurtas: &MuhurtaCollection) -> Vec<&Muhurta> {
        let mut periods = Vec::new();
        
        if let Some(ref abhijit) = muhurtas.abhijit {
            if abhijit.nature.is_good_for_starting() {
                periods.push(abhijit);
            }
        }
        if let Some(ref amrit) = muhurtas.amrit_kaal {
            if amrit.nature.is_good_for_starting() {
                periods.push(amrit);
            }
        }
        if let Some(ref brahma) = muhurtas.brahma_muhurta {
            if brahma.nature.is_good_for_starting() {
                periods.push(brahma);
            }
        }
        
        periods
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_muhurta_nature() {
        assert!(MuhurtaNature::Auspicious.is_good_for_starting());
        assert!(!MuhurtaNature::Inauspicious.is_good_for_starting());
        assert!(MuhurtaNature::Inauspicious.should_avoid());
    }

    #[test]
    fn test_rahu_kalam() {
        let rahu = RahuKalam::for_day("Sunday", "06:00", "18:00");
        assert_eq!(rahu.start, "16:30");
        assert_eq!(rahu.end, "18:00");
        
        let rahu_mon = RahuKalam::for_day("Monday", "06:00", "18:00");
        assert_eq!(rahu_mon.start, "07:30");
    }

    #[test]
    fn test_brahma_muhurta() {
        let brahma = BrahmaMuhurta::from_sunrise("06:00");
        assert_eq!(brahma.duration_minutes, 96);
        assert!(brahma.description.contains("meditation"));
    }
}
