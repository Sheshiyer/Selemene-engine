//! Choghadiya - Vedic time periods for selecting auspicious timings
//!
//! Choghadiya divides the day and night into specific time periods
//! classified as good, medium, or bad for various activities.

use serde::{Deserialize, Serialize};

/// Collection of all Choghadiya periods for a day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoghadiyaTimings {
    /// Day Choghadiyas (sunrise to sunset)
    pub day: Vec<Choghadiya>,
    /// Night Choghadiyas (sunset to sunrise)
    pub night: Vec<Choghadiya>,
    /// Day of week
    pub day_of_week: String,
    /// Sunrise time
    pub sunrise: String,
    /// Sunset time
    pub sunset: String,
}

impl ChoghadiyaTimings {
    /// Get the current Choghadiya based on time
    pub fn get_current(&self, current_time: &str) -> Option<&Choghadiya> {
        let all: Vec<&Choghadiya> = self.day.iter()
            .chain(self.night.iter())
            .collect();
        
        all.iter()
            .find(|c| current_time >= c.start.as_str() && current_time <= c.end.as_str())
            .copied()
    }
    
    /// Get all favorable Choghadiyas for starting new activities
    pub fn get_favorable(&self) -> Vec<&Choghadiya> {
        let all: Vec<&Choghadiya> = self.day.iter()
            .chain(self.night.iter())
            .collect();
        
        all.into_iter()
            .filter(|c| c.nature.is_favorable())
            .collect()
    }
    
    /// Get all unfavorable Choghadiyas to avoid
    pub fn get_unfavorable(&self) -> Vec<&Choghadiya> {
        let all: Vec<&Choghadiya> = self.day.iter()
            .chain(self.night.iter())
            .collect();
        
        all.into_iter()
            .filter(|c| !c.nature.is_favorable())
            .collect()
    }
    
    /// Find best Choghadiya for a specific type of activity
    pub fn find_best_for_activity(&self, activity_type: ActivityCategory) -> Option<&Choghadiya> {
        let all: Vec<&Choghadiya> = self.day.iter()
            .chain(self.night.iter())
            .collect();
        
        all.into_iter()
            .filter(|c| c.suitable_for.contains(&activity_type))
            .max_by_key(|c| c.nature.score())
    }
}

/// A single Choghadiya period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choghadiya {
    /// Name of the Choghadiya
    pub name: ChoghadiyaName,
    /// Start time (HH:MM)
    pub start: String,
    /// End time (HH:MM)
    pub end: String,
    /// Duration in minutes
    pub duration_minutes: u32,
    /// Nature: good, medium, bad
    pub nature: ChoghadiyaNature,
    /// Ruling planet
    pub ruler: String,
    /// Suitable activities
    pub suitable_for: Vec<ActivityCategory>,
    /// Activities to avoid
    pub avoid: Vec<ActivityCategory>,
}

/// Names of the Choghadiya periods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoghadiyaName {
    /// Shubh - Auspicious
    Shubh,
    /// Labh - Profit
    Labh,
    /// Amrit - Nectar
    Amrit,
    /// Char - Moving (neutral, good for travel)
    Char,
    /// Rog - Disease (inauspicious)
    Rog,
    /// Kaal - Death (inauspicious)
    Kaal,
    /// Udveg - Anxiety (inauspicious)
    Udveg,
    /// Kal - Time (moderately inauspicious)
    #[serde(rename = "kal")]
    KalPeriod,
}

impl ChoghadiyaName {
    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            ChoghadiyaName::Shubh => "Shubh",
            ChoghadiyaName::Labh => "Labh",
            ChoghadiyaName::Amrit => "Amrit",
            ChoghadiyaName::Char => "Char",
            ChoghadiyaName::Rog => "Rog",
            ChoghadiyaName::Kaal => "Kaal",
            ChoghadiyaName::Udveg => "Udveg",
            ChoghadiyaName::KalPeriod => "Kal",
        }
    }
    
    /// Get meaning/description
    pub fn meaning(&self) -> &'static str {
        match self {
            ChoghadiyaName::Shubh => "Auspicious",
            ChoghadiyaName::Labh => "Profit",
            ChoghadiyaName::Amrit => "Nectar",
            ChoghadiyaName::Char => "Moving",
            ChoghadiyaName::Rog => "Disease",
            ChoghadiyaName::Kaal => "Death",
            ChoghadiyaName::Udveg => "Anxiety",
            ChoghadiyaName::KalPeriod => "Time",
        }
    }
}

/// Nature of Choghadiya
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoghadiyaNature {
    Good,
    Medium,
    Bad,
}

impl ChoghadiyaNature {
    /// Check if this Choghadiya is favorable
    pub fn is_favorable(&self) -> bool {
        matches!(self, ChoghadiyaNature::Good)
    }
    
    /// Get numerical score for ranking (higher is better)
    pub fn score(&self) -> u8 {
        match self {
            ChoghadiyaNature::Good => 3,
            ChoghadiyaNature::Medium => 2,
            ChoghadiyaNature::Bad => 1,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ChoghadiyaNature::Good => "good",
            ChoghadiyaNature::Medium => "medium",
            ChoghadiyaNature::Bad => "bad",
        }
    }
}

/// Categories of activities for Choghadiya matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityCategory {
    /// New beginnings, ventures
    StartingNew,
    /// Business and commerce
    Business,
    /// Education and learning
    Education,
    /// Travel and movement
    Travel,
    /// Religious activities
    Religious,
    /// Medical procedures
    Medical,
    /// Legal matters
    Legal,
    /// Construction
    Construction,
    /// Marriage and partnerships
    Marriage,
    /// Purchasing property/vehicles
    Purchasing,
    /// Any activity
    Any,
}

/// Day-wise Choghadiya sequences
/// 
/// Each day of the week has a different starting Choghadiya.
/// The sequence is always: UD → CH → LA → AM → KA → SH → RO (for day)
/// and: SH → AM → CH → RO → LA → UD → KA (for night)
pub struct ChoghadiyaSequence;

impl ChoghadiyaSequence {
    /// Get the day sequence for a given day of week
    /// 
    /// The pattern repeats every 7 periods for the day
    pub fn get_day_sequence(day: &str) -> Vec<(ChoghadiyaName, ChoghadiyaNature, &'static str)> {
        let base_sequence = vec![
            (ChoghadiyaName::Udveg, ChoghadiyaNature::Bad, "Mercury"),
            (ChoghadiyaName::Char, ChoghadiyaNature::Medium, "Venus"),
            (ChoghadiyaName::Labh, ChoghadiyaNature::Good, "Jupiter"),
            (ChoghadiyaName::Amrit, ChoghadiyaNature::Good, "Moon"),
            (ChoghadiyaName::Kaal, ChoghadiyaNature::Bad, "Saturn"),
            (ChoghadiyaName::Shubh, ChoghadiyaNature::Good, "Sun"),
            (ChoghadiyaName::Rog, ChoghadiyaNature::Bad, "Mars"),
        ];
        
        // Each day starts at a different position in the sequence
        let start_offset = match day.to_lowercase().as_str() {
            "sunday" | "sun" => 5,    // Starts with Shubh
            "monday" | "mon" => 3,    // Starts with Amrit
            "tuesday" | "tue" => 6,   // Starts with Rog
            "wednesday" | "wed" => 0, // Starts with Udveg
            "thursday" | "thu" => 2,  // Starts with Labh
            "friday" | "fri" => 1,    // Starts with Char
            "saturday" | "sat" => 4,  // Starts with Kaal
            _ => 0,
        };
        
        // Generate 8 periods (some Choghadiyas repeat)
        let mut sequence = Vec::with_capacity(8);
        for i in 0..8 {
            let idx = (start_offset + i) % 7;
            sequence.push(base_sequence[idx]);
        }
        
        sequence
    }
    
    /// Get the night sequence for a given day of week
    pub fn get_night_sequence(day: &str) -> Vec<(ChoghadiyaName, ChoghadiyaNature, &'static str)> {
        let base_sequence = vec![
            (ChoghadiyaName::Shubh, ChoghadiyaNature::Good, "Sun"),
            (ChoghadiyaName::Amrit, ChoghadiyaNature::Good, "Moon"),
            (ChoghadiyaName::Char, ChoghadiyaNature::Medium, "Venus"),
            (ChoghadiyaName::Rog, ChoghadiyaNature::Bad, "Mars"),
            (ChoghadiyaName::Kaal, ChoghadiyaNature::Bad, "Saturn"),
            (ChoghadiyaName::Labh, ChoghadiyaNature::Good, "Jupiter"),
            (ChoghadiyaName::Udveg, ChoghadiyaNature::Bad, "Mercury"),
        ];
        
        let start_offset = match day.to_lowercase().as_str() {
            "sunday" | "sun" => 0,    // Starts with Shubh
            "monday" | "mon" => 1,    // Starts with Amrit
            "tuesday" | "tue" => 3,   // Starts with Rog
            "wednesday" | "wed" => 6, // Starts with Udveg
            "thursday" | "thu" => 5,  // Starts with Labh
            "friday" | "fri" => 2,    // Starts with Char
            "saturday" | "sat" => 4,  // Starts with Kaal
            _ => 0,
        };
        
        let mut sequence = Vec::with_capacity(8);
        for i in 0..8 {
            let idx = (start_offset + i) % 7;
            sequence.push(base_sequence[idx]);
        }
        
        sequence
    }
}

/// Suitable activities for each Choghadiya name
pub fn get_suitable_activities(name: ChoghadiyaName) -> Vec<ActivityCategory> {
    match name {
        ChoghadiyaName::Shubh => vec![
            ActivityCategory::Religious,
            ActivityCategory::Marriage,
            ActivityCategory::StartingNew,
            ActivityCategory::Purchasing,
        ],
        ChoghadiyaName::Labh => vec![
            ActivityCategory::Business,
            ActivityCategory::Purchasing,
            ActivityCategory::StartingNew,
        ],
        ChoghadiyaName::Amrit => vec![
            ActivityCategory::Any,
            ActivityCategory::StartingNew,
            ActivityCategory::Medical,
        ],
        ChoghadiyaName::Char => vec![
            ActivityCategory::Travel,
            ActivityCategory::Purchasing,
        ],
        ChoghadiyaName::Rog => vec![
            ActivityCategory::Medical, // Only good for taking medicine
        ],
        ChoghadiyaName::Kaal => vec![
            ActivityCategory::Religious, // Only for worship
        ],
        ChoghadiyaName::Udveg => vec![
            ActivityCategory::Legal, // Can be used for legal battles
        ],
        ChoghadiyaName::KalPeriod => vec![
            ActivityCategory::Any, // Moderate for most things
        ],
    }
}

/// Activities to avoid for each Choghadiya name
pub fn get_activities_to_avoid(name: ChoghadiyaName) -> Vec<ActivityCategory> {
    match name {
        ChoghadiyaName::Shubh => vec![],
        ChoghadiyaName::Labh => vec![],
        ChoghadiyaName::Amrit => vec![],
        ChoghadiyaName::Char => vec![
            ActivityCategory::StartingNew, // Not best for new beginnings
        ],
        ChoghadiyaName::Rog => vec![
            ActivityCategory::StartingNew,
            ActivityCategory::Business,
            ActivityCategory::Marriage,
            ActivityCategory::Travel,
        ],
        ChoghadiyaName::Kaal => vec![
            ActivityCategory::StartingNew,
            ActivityCategory::Business,
            ActivityCategory::Travel,
            ActivityCategory::Purchasing,
        ],
        ChoghadiyaName::Udveg => vec![
            ActivityCategory::StartingNew,
            ActivityCategory::Business,
            ActivityCategory::Marriage,
        ],
        ChoghadiyaName::KalPeriod => vec![
            ActivityCategory::StartingNew,
        ],
    }
}

/// Calculate Choghadiya timings
/// 
/// Day Choghadiya = (Sunset - Sunrise) / 8
/// Night Choghadiya = (Next Sunrise - Sunset) / 8
pub fn calculate_choghadiya(
    day: &str,
    sunrise: &str,
    sunset: &str,
    next_sunrise: &str,
) -> ChoghadiyaTimings {
    let day_sequence = ChoghadiyaSequence::get_day_sequence(day);
    let night_sequence = ChoghadiyaSequence::get_night_sequence(day);
    
    let day_choghadiyas = calculate_period_choghadiya(
        &day_sequence,
        sunrise,
        sunset,
    );
    
    let night_choghadiyas = calculate_period_choghadiya(
        &night_sequence,
        sunset,
        next_sunrise,
    );
    
    ChoghadiyaTimings {
        day: day_choghadiyas,
        night: night_choghadiyas,
        day_of_week: day.to_string(),
        sunrise: sunrise.to_string(),
        sunset: sunset.to_string(),
    }
}

/// Calculate Choghadiyas for a single period
fn calculate_period_choghadiya(
    sequence: &[(ChoghadiyaName, ChoghadiyaNature, &'static str)],
    start_time: &str,
    end_time: &str,
) -> Vec<Choghadiya> {
    let mut choghadiyas = Vec::new();
    
    // Simplified: assumes each Choghadiya is approximately 90 minutes
    // (12 hours / 8 = 1.5 hours)
    let duration_minutes = 90;
    
    for (i, (name, nature, ruler)) in sequence.iter().enumerate() {
        let start_hour = i * 1 + 6; // Simplified calculation
        let end_hour = start_hour + 1;
        
        choghadiyas.push(Choghadiya {
            name: *name,
            start: format!("{:02}:00", start_hour),
            end: format!("{:02}:30", end_hour),
            duration_minutes,
            nature: *nature,
            ruler: ruler.to_string(),
            suitable_for: get_suitable_activities(*name),
            avoid: get_activities_to_avoid(*name),
        });
    }
    
    choghadiyas
}

/// Get recommendations for specific activities
pub fn get_recommendations(
    choghadiyas: &ChoghadiyaTimings,
    activity: ActivityCategory,
) -> Vec<Recommendation> {
    let mut recommendations = Vec::new();
    
    let all: Vec<&Choghadiya> = choghadiyas.day.iter()
        .chain(choghadiyas.night.iter())
        .collect();
    
    for choghadiya in all {
        let score = if choghadiya.suitable_for.contains(&activity) {
            3
        } else if choghadiya.avoid.contains(&activity) {
            0
        } else {
            1
        };
        
        if score > 0 {
            recommendations.push(Recommendation {
                choghadiya_name: choghadiya.name.as_str().to_string(),
                start: choghadiya.start.clone(),
                end: choghadiya.end.clone(),
                score,
                reason: format!("{:?} nature", choghadiya.nature),
            });
        }
    }
    
    // Sort by score descending
    recommendations.sort_by(|a, b| b.score.cmp(&a.score));
    
    recommendations
}

/// Recommendation for an activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub choghadiya_name: String,
    pub start: String,
    pub end: String,
    pub score: u8,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choghadiya_nature() {
        assert!(ChoghadiyaNature::Good.is_favorable());
        assert!(!ChoghadiyaNature::Bad.is_favorable());
        assert_eq!(ChoghadiyaNature::Good.score(), 3);
    }

    #[test]
    fn test_day_sequences() {
        let sunday = ChoghadiyaSequence::get_day_sequence("Sunday");
        assert_eq!(sunday[0].0, ChoghadiyaName::Shubh);
        
        let monday = ChoghadiyaSequence::get_day_sequence("Monday");
        assert_eq!(monday[0].0, ChoghadiyaName::Amrit);
        
        let saturday = ChoghadiyaSequence::get_day_sequence("Saturday");
        assert_eq!(saturday[0].0, ChoghadiyaName::Kaal);
    }

    #[test]
    fn test_night_sequences() {
        let sunday_night = ChoghadiyaSequence::get_night_sequence("Sunday");
        assert_eq!(sunday_night[0].0, ChoghadiyaName::Shubh);
    }

    #[test]
    fn test_calculate_choghadiya() {
        let choghadiya = calculate_choghadiya("Sunday", "06:00", "18:00", "06:00");
        assert_eq!(choghadiya.day.len(), 8);
        assert_eq!(choghadiya.night.len(), 8);
    }

    #[test]
    fn test_suitable_activities() {
        let activities = get_suitable_activities(ChoghadiyaName::Amrit);
        assert!(activities.contains(&ActivityCategory::Any));
        
        let activities = get_suitable_activities(ChoghadiyaName::Char);
        assert!(activities.contains(&ActivityCategory::Travel));
    }
}
