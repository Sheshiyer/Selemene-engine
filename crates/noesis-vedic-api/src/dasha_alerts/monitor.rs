//! Dasha transition monitor

use chrono::{NaiveDate, Utc};
use super::{DashaTransitionEvent, DashaTransitionType, TransitionSignificance, DashaAlertConfig};
use super::types::UpcomingDashaTransitions;

/// Monitor for dasha transitions
pub struct DashaTransitionMonitor {
    config: DashaAlertConfig,
    birth_date: NaiveDate,
    moon_nakshatra: u8,
}

impl DashaTransitionMonitor {
    /// Create a new monitor
    pub fn new(birth_date: NaiveDate, moon_nakshatra: u8, config: DashaAlertConfig) -> Self {
        Self {
            config,
            birth_date,
            moon_nakshatra,
        }
    }
    
    /// Check for upcoming transitions
    pub fn check_upcoming(&self, check_date: NaiveDate, days_ahead: u32) -> Vec<DashaTransitionEvent> {
        let mut events = Vec::new();
        
        // This would integrate with actual dasha calculations
        // Placeholder logic for demonstration
        
        events
    }
    
    /// Get significance of a dasha lord
    pub fn get_lord_significance(&self, lord: &str) -> TransitionSignificance {
        match lord.to_lowercase().as_str() {
            "rahu" | "ketu" | "saturn" => TransitionSignificance::Major,
            "jupiter" | "venus" | "mars" => TransitionSignificance::Moderate,
            _ => TransitionSignificance::Minor,
        }
    }
    
    /// Generate guidance for a transition
    pub fn generate_guidance(&self, from_lord: &str, to_lord: &str) -> String {
        let from_nature = self.get_lord_nature(from_lord);
        let to_nature = self.get_lord_nature(to_lord);
        
        format!(
            "Transitioning from {} {} period to {} {} period. {}",
            from_lord, from_nature,
            to_lord, to_nature,
            self.get_transition_advice(from_lord, to_lord)
        )
    }
    
    fn get_lord_nature(&self, lord: &str) -> &'static str {
        match lord.to_lowercase().as_str() {
            "sun" => "authoritative",
            "moon" => "emotional and nurturing",
            "mars" => "energetic and assertive",
            "mercury" => "intellectual and communicative",
            "jupiter" => "expansive and spiritual",
            "venus" => "artistic and pleasure-seeking",
            "saturn" => "disciplined and karmic",
            "rahu" => "material and desire-driven",
            "ketu" => "spiritual and detaching",
            _ => "transformative",
        }
    }
    
    fn get_transition_advice(&self, from: &str, to: &str) -> String {
        match to.to_lowercase().as_str() {
            "saturn" => "Prepare for a period of hard work and discipline. Focus on long-term goals.".to_string(),
            "jupiter" => "A period of growth and opportunities begins. Stay ethical and optimistic.".to_string(),
            "rahu" => "Material desires may intensify. Stay grounded and avoid shortcuts.".to_string(),
            "ketu" => "Spiritual insights increase. Let go of attachments gracefully.".to_string(),
            "venus" => "Relationships and creativity flourish. Enjoy life's pleasures mindfully.".to_string(),
            "mars" => "Energy and initiative increase. Channel aggression constructively.".to_string(),
            "mercury" => "Communication and learning are favored. Start new courses or skills.".to_string(),
            "moon" => "Emotional sensitivity heightens. Nurture yourself and loved ones.".to_string(),
            "sun" => "Leadership opportunities arise. Step into authority with confidence.".to_string(),
            _ => "Embrace the changes with awareness.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lord_significance() {
        let config = DashaAlertConfig::default();
        let birth = NaiveDate::from_ymd_opt(1990, 1, 15).unwrap();
        let monitor = DashaTransitionMonitor::new(birth, 4, config);
        
        assert_eq!(monitor.get_lord_significance("Saturn"), TransitionSignificance::Major);
        assert_eq!(monitor.get_lord_significance("Mercury"), TransitionSignificance::Minor);
    }
}
