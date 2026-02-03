//! Hora - Planetary hours
//!
//! The day is divided into 24 Horas (planetary hours), ruled by the seven planets
//! in a specific sequence. Each Hora is approximately 1 hour long.

use serde::{Deserialize, Serialize};

/// A single Hora (planetary hour)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hora {
    /// Hour number (1-24)
    pub number: u8,
    /// Ruling planet
    pub ruler: Planet,
    /// Start time (HH:MM)
    pub start: String,
    /// End time (HH:MM)
    pub end: String,
    /// Is this hour favorable
    pub is_favorable: bool,
    /// Quality description
    pub quality: String,
}

/// Planets that rule the Horas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Planet {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
}

impl Planet {
    /// Get the planet name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Planet::Sun => "Sun",
            Planet::Moon => "Moon",
            Planet::Mars => "Mars",
            Planet::Mercury => "Mercury",
            Planet::Jupiter => "Jupiter",
            Planet::Venus => "Venus",
            Planet::Saturn => "Saturn",
        }
    }
    
    /// Get the symbol of the planet
    pub fn symbol(&self) -> &'static str {
        match self {
            Planet::Sun => "☉",
            Planet::Moon => "☽",
            Planet::Mars => "♂",
            Planet::Mercury => "☿",
            Planet::Jupiter => "♃",
            Planet::Venus => "♀",
            Planet::Saturn => "♄",
        }
    }
    
    /// Get activities suitable for this planet's Hora
    pub fn suitable_activities(&self) -> Vec<&'static str> {
        match self {
            Planet::Sun => vec![
                "Government work",
                "Leadership activities",
                "Authority matters",
                "Political activities",
                "Seeking favors from superiors",
            ],
            Planet::Moon => vec![
                "Emotional matters",
                "Family activities",
                "Travel",
                "Agriculture",
                "Food and nourishment",
                "Connecting with women",
            ],
            Planet::Mars => vec![
                "Sports and competition",
                "Debates and arguments",
                "Military matters",
                "Surgery",
                "Mechanical work",
                "Physical exercise",
            ],
            Planet::Mercury => vec![
                "Communication",
                "Writing",
                "Business",
                "Education",
                "Technology",
                "Negotiations",
                "Short travels",
            ],
            Planet::Jupiter => vec![
                "Religious activities",
                "Teaching",
                "Learning",
                "Marriage",
                "Charity",
                "Legal matters",
                "Consulting elders",
            ],
            Planet::Venus => vec![
                "Romance",
                "Arts and music",
                "Shopping",
                "Social activities",
                "Beauty treatments",
                "Marriage proposals",
                "Luxury purchases",
            ],
            Planet::Saturn => vec![
                "Hard work",
                "Research",
                "Long-term planning",
                "Construction",
                "Disciplined activities",
                "Avoid: new beginnings",
            ],
        }
    }
    
    /// Get activities to avoid in this planet's Hora
    pub fn activities_to_avoid(&self) -> Vec<&'static str> {
        match self {
            Planet::Sun => vec!["Wasting time", "Disobeying authority"],
            Planet::Moon => vec!["Starting conflicts", "Major decisions during waning"],
            Planet::Mars => vec!["Peace negotiations", "Starting new businesses"],
            Planet::Mercury => vec!["Signing important contracts during combustion"],
            Planet::Jupiter => vec!["Materialistic pursuits", "Gambling"],
            Planet::Venus => vec!["Strenuous labor", "Arguments with spouse"],
            Planet::Saturn => vec!["New ventures", "Risky investments", "Marriage"],
        }
    }
}

/// Collection of all 24 Horas for a day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoraTimings {
    /// Day Horas (sunrise to sunset) - typically 12
    pub day_horas: Vec<Hora>,
    /// Night Horas (sunset to sunrise) - typically 12
    pub night_horas: Vec<Hora>,
    /// Day of week (determines starting planet)
    pub day_of_week: String,
    /// Sunrise time
    pub sunrise: String,
    /// Sunset time
    pub sunset: String,
}

impl HoraTimings {
    /// Get the current Hora based on time
    pub fn get_current_hora(&self, current_time: &str) -> Option<&Hora> {
        let all_horas: Vec<&Hora> = self.day_horas.iter()
            .chain(self.night_horas.iter())
            .collect();
        
        all_horas.iter()
            .find(|h| current_time >= h.start.as_str() && current_time <= h.end.as_str())
            .copied()
    }
    
    /// Get Horas ruled by a specific planet
    pub fn get_horas_by_planet(&self, planet: Planet) -> Vec<&Hora> {
        let all_horas: Vec<&Hora> = self.day_horas.iter()
            .chain(self.night_horas.iter())
            .collect();
        
        all_horas.into_iter()
            .filter(|h| h.ruler == planet)
            .collect()
    }
    
    /// Get favorable Horas for a specific activity type
    pub fn get_favorable_horas(&self, activity_type: ActivityType) -> Vec<&Hora> {
        let all_horas: Vec<&Hora> = self.day_horas.iter()
            .chain(self.night_horas.iter())
            .collect();
        
        let preferred_planet = match activity_type {
            ActivityType::Business => Planet::Mercury,
            ActivityType::Romance => Planet::Venus,
            ActivityType::Study => Planet::Jupiter,
            ActivityType::Health => Planet::Sun,
            ActivityType::Travel => Planet::Moon,
            ActivityType::Spiritual => Planet::Jupiter,
            ActivityType::Sports => Planet::Mars,
            ActivityType::Social => Planet::Venus,
            ActivityType::HardWork => Planet::Saturn,
            ActivityType::NewBeginnings => Planet::Sun,
        };
        
        all_horas.into_iter()
            .filter(|h| h.ruler == preferred_planet)
            .collect()
    }
    
    /// Get total count
    pub fn total_horas(&self) -> usize {
        self.day_horas.len() + self.night_horas.len()
    }
}

/// Types of activities for Hora recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityType {
    Business,
    Romance,
    Study,
    Health,
    Travel,
    Spiritual,
    Sports,
    Social,
    HardWork,
    NewBeginnings,
}

/// The Hora sequence order for each day of the week
/// Each day starts with a different planet's Hora
pub struct HoraSequence;

impl HoraSequence {
    /// Get the starting planet for a given day
    pub fn get_starting_planet(day: &str) -> Planet {
        match day.to_lowercase().as_str() {
            "sunday" | "sun" => Planet::Sun,
            "monday" | "mon" => Planet::Moon,
            "tuesday" | "tue" => Planet::Mars,
            "wednesday" | "wed" => Planet::Mercury,
            "thursday" | "thu" => Planet::Jupiter,
            "friday" | "fri" => Planet::Venus,
            "saturday" | "sat" => Planet::Saturn,
            _ => Planet::Sun,
        }
    }
    
    /// Generate the 24 Hora sequence for a given day
    /// The sequence is: Sun → Venus → Mercury → Moon → Saturn → Jupiter → Mars → repeat
    pub fn generate_sequence(day: &str) -> Vec<Planet> {
        let start = Self::get_starting_planet(day);
        let sequence_order = vec![
            Planet::Sun,
            Planet::Venus,
            Planet::Mercury,
            Planet::Moon,
            Planet::Saturn,
            Planet::Jupiter,
            Planet::Mars,
        ];
        
        // Find starting position
        let start_idx = sequence_order.iter()
            .position(|&p| p == start)
            .unwrap_or(0);
        
        // Generate 24 Horas by cycling through the sequence
        let mut horas = Vec::with_capacity(24);
        for i in 0..24 {
            let idx = (start_idx + i) % 7;
            horas.push(sequence_order[idx]);
        }
        
        horas
    }
}

/// Calculate Hora timings for a day
/// 
/// Day Horas = time between sunrise and sunset divided by 12
/// Night Horas = time between sunset and next sunrise divided by 12
pub fn calculate_hora_timings(
    day: &str,
    sunrise: &str,
    sunset: &str,
    next_sunrise: &str,
) -> HoraTimings {
    let sequence = HoraSequence::generate_sequence(day);
    
    // Parse times (simplified - assumes HH:MM format)
    let day_horas = calculate_period_horas(&sequence[0..12], sunrise, sunset, 1);
    let night_horas = calculate_period_horas(&sequence[12..24], sunset, next_sunrise, 13);
    
    HoraTimings {
        day_horas,
        night_horas,
        day_of_week: day.to_string(),
        sunrise: sunrise.to_string(),
        sunset: sunset.to_string(),
    }
}

/// Calculate Horas for a single period (day or night)
fn calculate_period_horas(
    planets: &[Planet],
    start_time: &str,
    end_time: &str,
    start_number: u8,
) -> Vec<Hora> {
    let mut horas = Vec::new();
    
    // Simplified calculation - in real implementation would parse times
    // and divide the period into 12 equal parts
    let duration_minutes = 60; // Simplified: 1 hour per Hora
    
    for (i, planet) in planets.iter().enumerate() {
        let hora_num = start_number + i as u8;
        let start_hour = hora_num - 1;
        let end_hour = hora_num;
        
        let quality = match planet {
            Planet::Sun => "Power and authority",
            Planet::Moon => "Change and receptivity",
            Planet::Mars => "Action and courage",
            Planet::Mercury => "Communication and skill",
            Planet::Jupiter => "Wisdom and expansion",
            Planet::Venus => "Pleasure and harmony",
            Planet::Saturn => "Restriction and discipline",
        };
        
        let is_favorable = !matches!(planet, Planet::Saturn);
        
        horas.push(Hora {
            number: hora_num,
            ruler: *planet,
            start: format!("{:02}:00", start_hour),
            end: format!("{:02}:00", end_hour),
            is_favorable,
            quality: quality.to_string(),
        });
    }
    
    horas
}

/// Helper to find favorable time windows for activities
pub fn find_favorable_windows(
    hora_timings: &HoraTimings,
    activity: ActivityType,
    min_consecutive_hours: u8,
) -> Vec<(String, String)> {
    let favorable = hora_timings.get_favorable_horas(activity);
    let mut windows = Vec::new();
    
    // Group consecutive favorable Horas
    let mut current_start: Option<&Hora> = None;
    let mut count = 0;
    
    for hora in &favorable {
        if current_start.is_none() {
            current_start = Some(*hora);
            count = 1;
        } else {
            count += 1;
        }
        
        if count >= min_consecutive_hours {
            if let Some(start) = current_start {
                windows.push((start.start.clone(), hora.end.clone()));
            }
        }
    }
    
    windows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planet_enum() {
        assert_eq!(Planet::Sun.as_str(), "Sun");
        assert_eq!(Planet::Moon.symbol(), "☽");
        assert!(!Planet::Sun.suitable_activities().is_empty());
    }

    #[test]
    fn test_hora_sequence() {
        let sunday_seq = HoraSequence::generate_sequence("Sunday");
        assert_eq!(sunday_seq.len(), 24);
        assert_eq!(sunday_seq[0], Planet::Sun); // Sunday starts with Sun
        
        let monday_seq = HoraSequence::generate_sequence("Monday");
        assert_eq!(monday_seq[0], Planet::Moon); // Monday starts with Moon
        
        let saturday_seq = HoraSequence::generate_sequence("Saturday");
        assert_eq!(saturday_seq[0], Planet::Saturn); // Saturday starts with Saturn
    }

    #[test]
    fn test_starting_planet() {
        assert_eq!(HoraSequence::get_starting_planet("Sunday"), Planet::Sun);
        assert_eq!(HoraSequence::get_starting_planet("Friday"), Planet::Venus);
    }

    #[test]
    fn test_hora_timings() {
        let timings = calculate_hora_timings("Sunday", "06:00", "18:00", "06:00");
        assert_eq!(timings.day_horas.len(), 12);
        assert_eq!(timings.night_horas.len(), 12);
        assert_eq!(timings.total_horas(), 24);
    }

    #[test]
    fn test_activity_preferences() {
        let activities = Planet::Venus.suitable_activities();
        assert!(activities.iter().any(|a| a.contains("Romance") || a.contains("romance")));
        
        let activities = Planet::Mercury.suitable_activities();
        assert!(activities.iter().any(|a| a.contains("Business") || a.contains("business")));
    }
}
