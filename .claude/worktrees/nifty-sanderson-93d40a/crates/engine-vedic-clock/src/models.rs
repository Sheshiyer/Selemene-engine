//! Data structures for the VedicClock-TCM engine
//!
//! Combines TCM organ clock concepts with Vedic Ayurvedic doshas
//! to provide temporal recommendations for daily activities.

use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// The 12 organs in the TCM organ clock system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum Organ {
    Lung,
    LargeIntestine,
    Stomach,
    Spleen,
    Heart,
    SmallIntestine,
    Bladder,
    Kidney,
    Pericardium,
    TripleWarmer,
    Gallbladder,
    Liver,
}

impl Organ {
    /// Get all organs in order of the 24-hour cycle
    pub fn all_in_cycle_order() -> [Organ; 12] {
        [
            Organ::Lung,           // 3-5 AM
            Organ::LargeIntestine, // 5-7 AM
            Organ::Stomach,        // 7-9 AM
            Organ::Spleen,         // 9-11 AM
            Organ::Heart,          // 11 AM-1 PM
            Organ::SmallIntestine, // 1-3 PM
            Organ::Bladder,        // 3-5 PM
            Organ::Kidney,         // 5-7 PM
            Organ::Pericardium,    // 7-9 PM
            Organ::TripleWarmer,   // 9-11 PM
            Organ::Gallbladder,    // 11 PM-1 AM
            Organ::Liver,          // 1-3 AM
        ]
    }

    /// Get the display name of the organ
    pub fn display_name(&self) -> &'static str {
        match self {
            Organ::Lung => "Lung",
            Organ::LargeIntestine => "Large Intestine",
            Organ::Stomach => "Stomach",
            Organ::Spleen => "Spleen",
            Organ::Heart => "Heart",
            Organ::SmallIntestine => "Small Intestine",
            Organ::Bladder => "Bladder",
            Organ::Kidney => "Kidney",
            Organ::Pericardium => "Pericardium",
            Organ::TripleWarmer => "Triple Warmer",
            Organ::Gallbladder => "Gallbladder",
            Organ::Liver => "Liver",
        }
    }
}

/// The Five Elements in TCM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum Element {
    Wood,
    Fire,
    Earth,
    Metal,
    Water,
}

impl Element {
    /// Get the display name of the element
    pub fn display_name(&self) -> &'static str {
        match self {
            Element::Wood => "Wood",
            Element::Fire => "Fire",
            Element::Earth => "Earth",
            Element::Metal => "Metal",
            Element::Water => "Water",
        }
    }

    /// Get associated qualities
    pub fn qualities(&self) -> &'static [&'static str] {
        match self {
            Element::Wood => &["growth", "flexibility", "vision", "planning"],
            Element::Fire => &["warmth", "joy", "connection", "transformation"],
            Element::Earth => &["stability", "nourishment", "centering", "grounding"],
            Element::Metal => &["clarity", "precision", "release", "boundaries"],
            Element::Water => &["stillness", "wisdom", "depth", "restoration"],
        }
    }
}

/// The three Ayurvedic doshas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum Dosha {
    Vata,
    Pitta,
    Kapha,
}

impl Dosha {
    /// Get the display name of the dosha
    pub fn display_name(&self) -> &'static str {
        match self {
            Dosha::Vata => "Vata",
            Dosha::Pitta => "Pitta",
            Dosha::Kapha => "Kapha",
        }
    }

    /// Get elemental composition
    pub fn elements(&self) -> (&'static str, &'static str) {
        match self {
            Dosha::Vata => ("Air", "Space"),
            Dosha::Pitta => ("Fire", "Water"),
            Dosha::Kapha => ("Earth", "Water"),
        }
    }
}

/// A 2-hour time window in the organ clock
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct OrganWindow {
    /// The active organ
    pub organ: Organ,
    /// Associated TCM element
    pub element: Element,
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23, may wrap around midnight)
    pub end_hour: u8,
    /// Peak energy description
    pub peak_energy: String,
    /// Associated emotion when balanced
    pub associated_emotion: String,
    /// Recommended activities for this window
    pub recommended_activities: Vec<String>,
}

impl OrganWindow {
    /// Get a human-readable time range string
    pub fn time_range_display(&self) -> String {
        let start = format_hour_12(self.start_hour);
        let end = format_hour_12(self.end_hour);
        format!("{} - {}", start, end)
    }
}

/// A dosha time period
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct DoshaTime {
    /// The dominant dosha
    pub dosha: Dosha,
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23)
    pub end_hour: u8,
    /// Qualities active during this period
    pub qualities: Vec<String>,
}

/// Activity types for optimal timing recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum Activity {
    Meditation,
    Exercise,
    Work,
    Eating,
    Sleep,
    Creative,
    Social,
}

impl Activity {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Activity::Meditation => "Meditation",
            Activity::Exercise => "Exercise",
            Activity::Work => "Work & Focus",
            Activity::Eating => "Eating & Digestion",
            Activity::Sleep => "Sleep & Rest",
            Activity::Creative => "Creative Work",
            Activity::Social => "Social Connection",
        }
    }
}

/// A recommended time window for an activity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TimeWindow {
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23)
    pub end_hour: u8,
    /// Quality/favorability score (0.0 - 1.0)
    pub quality: f64,
    /// Reasoning for this recommendation
    pub reason: String,
}

impl TimeWindow {
    /// Get a human-readable time range string
    pub fn time_range_display(&self) -> String {
        let start = format_hour_12(self.start_hour);
        let end = format_hour_12(self.end_hour);
        format!("{} - {}", start, end)
    }
}

/// A single activity recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ActivityRecommendation {
    /// The recommended activity
    pub activity: String,
    /// Quality/favorability (optimal, favorable, neutral, challenging)
    pub quality: String,
    /// Brief explanation
    pub reason: String,
}

/// A complete temporal recommendation combining organ and dosha
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TemporalRecommendation {
    /// Display string for the time window
    pub time_window: String,
    /// Currently active organ
    pub organ: Organ,
    /// Currently dominant dosha
    pub dosha: Dosha,
    /// Activity recommendations
    pub activities: Vec<ActivityRecommendation>,
    /// Optional Panchanga quality overlay
    pub panchanga_quality: Option<String>,
}

/// Full calculation result from the VedicClock engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct VedicClockResult {
    /// Current organ window
    pub current_organ: OrganWindow,
    /// Current dosha period
    pub current_dosha: DoshaTime,
    /// Combined temporal recommendation
    pub recommendation: TemporalRecommendation,
    /// Optional: upcoming transitions
    pub upcoming: Option<Vec<UpcomingTransition>>,
    /// Timestamp of calculation
    pub calculated_for: String,
}

/// An upcoming time transition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct UpcomingTransition {
    /// Time of transition
    pub time: String,
    /// What's changing
    pub description: String,
    /// New organ (if organ transition)
    pub new_organ: Option<Organ>,
    /// New dosha (if dosha transition)
    pub new_dosha: Option<Dosha>,
}

/// Helper function to format hour in 12-hour format
fn format_hour_12(hour: u8) -> String {
    match hour {
        0 => "12 AM".to_string(),
        1..=11 => format!("{} AM", hour),
        12 => "12 PM".to_string(),
        13..=23 => format!("{} PM", hour - 12),
        _ => format!("{}", hour), // Shouldn't happen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organ_cycle_order() {
        let organs = Organ::all_in_cycle_order();
        assert_eq!(organs[0], Organ::Lung);
        assert_eq!(organs[11], Organ::Liver);
        assert_eq!(organs.len(), 12);
    }

    #[test]
    fn test_element_qualities() {
        assert!(!Element::Wood.qualities().is_empty());
        assert!(Element::Fire.qualities().contains(&"joy"));
    }

    #[test]
    fn test_dosha_elements() {
        let (e1, e2) = Dosha::Vata.elements();
        assert_eq!(e1, "Air");
        assert_eq!(e2, "Space");
    }

    #[test]
    fn test_format_hour_12() {
        assert_eq!(format_hour_12(0), "12 AM");
        assert_eq!(format_hour_12(3), "3 AM");
        assert_eq!(format_hour_12(12), "12 PM");
        assert_eq!(format_hour_12(15), "3 PM");
        assert_eq!(format_hour_12(23), "11 PM");
    }
}
