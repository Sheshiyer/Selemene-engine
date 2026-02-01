//! Daily Practice Workflow — Temporal optimization
//!
//! Executes: panchanga, vedic-clock, biorhythm
//! Synthesizes daily rhythms for optimal activity timing.

use chrono::{DateTime, Duration, Utc};
use noesis_core::EngineInput;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Daily Practice specific data extracted from engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPracticeData {
    /// From Panchanga
    pub panchanga: Option<PanchangaData>,
    /// From Vedic Clock
    pub vedic_clock: Option<VedicClockData>,
    /// From Biorhythm
    pub biorhythm: Option<BiorhythmData>,
}

/// Panchanga engine data relevant to synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangaData {
    pub tithi: TithiInfo,
    pub nakshatra: NakshatraInfo,
    pub yoga: String,
    pub karana: String,
    pub vara: String, // Day of week
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TithiInfo {
    pub name: String,
    pub number: u8,
    pub paksha: String, // Shukla or Krishna
    pub quality: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NakshatraInfo {
    pub name: String,
    pub number: u8,
    pub quality: String,
    pub ruling_deity: String,
}

impl PanchangaData {
    /// Extract from engine output JSON
    pub fn from_json(value: &Value) -> Option<Self> {
        let tithi_val = value.get("tithi")?;
        let nakshatra_val = value.get("nakshatra")?;

        let tithi = TithiInfo {
            name: tithi_val.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            number: tithi_val.get("number").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
            paksha: tithi_val.get("paksha").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            quality: tithi_quality(tithi_val.get("number").and_then(|v| v.as_u64()).unwrap_or(0) as u8),
        };

        let nakshatra = NakshatraInfo {
            name: nakshatra_val.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            number: nakshatra_val.get("number").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
            quality: nakshatra_val.get("quality").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            ruling_deity: nakshatra_val.get("deity").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
        };

        let yoga = value.get("yoga")
            .and_then(|v| v.get("name").or(Some(v)))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let karana = value.get("karana")
            .and_then(|v| v.get("name").or(Some(v)))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let vara = value.get("vara")
            .or_else(|| value.get("weekday"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        Some(Self {
            tithi,
            nakshatra,
            yoga,
            karana,
            vara,
        })
    }

    /// Get activity recommendations based on Panchanga
    pub fn recommended_activities(&self) -> Vec<String> {
        let mut activities = Vec::new();

        // Tithi-based recommendations
        match self.tithi.number {
            1 | 6 | 11 => activities.push("New beginnings, starting projects".to_string()),
            2 | 7 | 12 => activities.push("Nurturing relationships, care".to_string()),
            4 | 9 | 14 => activities.push("Challenging work, destruction of obstacles".to_string()),
            5 | 10 | 15 => activities.push("Completion, celebration, rest".to_string()),
            _ => activities.push("General activity day".to_string()),
        }

        // Vara-based recommendations
        match self.vara.to_lowercase().as_str() {
            "sunday" | "ravivar" => activities.push("Leadership, public activities".to_string()),
            "monday" | "somvar" => activities.push("Emotional work, intuition".to_string()),
            "tuesday" | "mangalvar" => activities.push("Physical activity, courage".to_string()),
            "wednesday" | "budhvar" => activities.push("Learning, communication".to_string()),
            "thursday" | "guruvar" => activities.push("Teaching, spiritual practice".to_string()),
            "friday" | "shukravar" => activities.push("Creative expression, relationships".to_string()),
            "saturday" | "shanivar" => activities.push("Discipline, organization, service".to_string()),
            _ => {}
        }

        activities
    }
}

/// Vedic Clock engine data relevant to synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VedicClockData {
    pub current_ghati: u8,
    pub current_pala: u8,
    pub current_muhurta: String,
    pub muhurta_quality: String,
    pub active_organ: String,
    pub dosha_time: String,
    pub recommended_activity: String,
}

impl VedicClockData {
    /// Extract from engine output JSON
    pub fn from_json(value: &Value) -> Option<Self> {
        Some(Self {
            current_ghati: value.get("ghati").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
            current_pala: value.get("pala").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
            current_muhurta: value.get("muhurta")
                .and_then(|v| v.get("name").or(Some(v)))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            muhurta_quality: value.get("muhurta")
                .and_then(|v| v.get("quality"))
                .and_then(|v| v.as_str())
                .unwrap_or("Neutral")
                .to_string(),
            active_organ: value.get("active_organ")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            dosha_time: value.get("dosha")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            recommended_activity: value.get("recommended_activity")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    /// Get optimal activities for current time
    pub fn optimal_activities(&self) -> Vec<String> {
        let mut activities = Vec::new();

        // Dosha-based recommendations
        match self.dosha_time.to_lowercase().as_str() {
            "vata" => {
                activities.push("Creative work, brainstorming".to_string());
                activities.push("Movement, yoga".to_string());
            }
            "pitta" => {
                activities.push("Focused mental work".to_string());
                activities.push("Decision making".to_string());
            }
            "kapha" => {
                activities.push("Physical exercise".to_string());
                activities.push("Starting new projects".to_string());
            }
            _ => {}
        }

        // Muhurta quality
        if self.muhurta_quality.to_lowercase().contains("auspicious") {
            activities.push("Important decisions, new ventures".to_string());
        }

        if !self.recommended_activity.is_empty() {
            activities.push(self.recommended_activity.clone());
        }

        activities
    }
}

/// Biorhythm engine data relevant to synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiorhythmData {
    pub physical: CycleData,
    pub emotional: CycleData,
    pub intellectual: CycleData,
    pub composite: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleData {
    pub value: f64,      // -1.0 to 1.0
    pub phase: String,   // "High", "Low", "Critical", "Rising", "Falling"
    pub description: String,
}

impl BiorhythmData {
    /// Extract from engine output JSON
    pub fn from_json(value: &Value) -> Option<Self> {
        let physical_val = value.get("physical").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let emotional_val = value.get("emotional").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let intellectual_val = value.get("intellectual").and_then(|v| v.as_f64()).unwrap_or(0.0);

        Some(Self {
            physical: CycleData {
                value: physical_val,
                phase: cycle_phase(physical_val),
                description: physical_description(physical_val),
            },
            emotional: CycleData {
                value: emotional_val,
                phase: cycle_phase(emotional_val),
                description: emotional_description(emotional_val),
            },
            intellectual: CycleData {
                value: intellectual_val,
                phase: cycle_phase(intellectual_val),
                description: intellectual_description(intellectual_val),
            },
            composite: (physical_val + emotional_val + intellectual_val) / 3.0,
        })
    }

    /// Get activity recommendations based on cycles
    pub fn activity_fit(&self) -> Vec<(String, f64)> {
        let mut activities = vec![];

        // Physical activities
        if self.physical.value > 0.3 {
            activities.push(("Physical exercise".to_string(), self.physical.value));
        }

        // Emotional activities
        if self.emotional.value > 0.3 {
            activities.push(("Social interactions".to_string(), self.emotional.value));
            activities.push(("Creative expression".to_string(), self.emotional.value));
        }

        // Intellectual activities
        if self.intellectual.value > 0.3 {
            activities.push(("Complex problem solving".to_string(), self.intellectual.value));
            activities.push(("Learning new skills".to_string(), self.intellectual.value));
        }

        // Rest recommendations
        if self.composite < -0.3 {
            activities.push(("Rest and recovery".to_string(), 0.8));
        }

        activities
    }

    /// Check if any cycle is critical (near zero crossing)
    pub fn has_critical_day(&self) -> bool {
        self.physical.value.abs() < 0.1 
            || self.emotional.value.abs() < 0.1 
            || self.intellectual.value.abs() < 0.1
    }
}

fn cycle_phase(value: f64) -> String {
    if value.abs() < 0.1 {
        "Critical".to_string()
    } else if value > 0.7 {
        "Peak".to_string()
    } else if value > 0.0 {
        "Rising/High".to_string()
    } else if value > -0.7 {
        "Falling/Low".to_string()
    } else {
        "Trough".to_string()
    }
}

fn physical_description(value: f64) -> String {
    if value > 0.7 {
        "High physical energy, great for exercise".to_string()
    } else if value > 0.3 {
        "Good physical stamina".to_string()
    } else if value > -0.3 {
        "Moderate physical state".to_string()
    } else {
        "Low physical energy, take it easy".to_string()
    }
}

fn emotional_description(value: f64) -> String {
    if value > 0.7 {
        "Emotionally vibrant, great for connection".to_string()
    } else if value > 0.3 {
        "Emotionally stable and open".to_string()
    } else if value > -0.3 {
        "Emotionally neutral".to_string()
    } else {
        "Emotionally sensitive, self-care recommended".to_string()
    }
}

fn intellectual_description(value: f64) -> String {
    if value > 0.7 {
        "Peak mental clarity, tackle complex tasks".to_string()
    } else if value > 0.3 {
        "Good focus and concentration".to_string()
    } else if value > -0.3 {
        "Moderate mental energy".to_string()
    } else {
        "Mental fog, routine tasks preferred".to_string()
    }
}

fn tithi_quality(number: u8) -> String {
    // Rikta tithis (4, 9, 14) are challenging
    // Poornima (15) and Amavasya (30) are powerful
    // Panchami (5), Dashami (10), Poornima (15) are auspicious
    match number {
        4 | 9 | 14 => "Rikta (Depleted)".to_string(),
        5 | 10 => "Nanda (Joy)".to_string(),
        1 | 6 | 11 => "Bhadra (Auspicious)".to_string(),
        2 | 7 | 12 => "Jaya (Victory)".to_string(),
        15 => "Poornima (Full)".to_string(),
        30 => "Amavasya (New)".to_string(),
        _ => "Purna (Complete)".to_string(),
    }
}

/// Create input specifically for Daily Practice workflow
pub fn create_daily_practice_input(current_time: DateTime<Utc>, latitude: f64, longitude: f64) -> EngineInput {
    EngineInput {
        birth_data: None,
        current_time,
        location: Some(noesis_core::Coordinates {
            latitude,
            longitude,
            altitude: None,
        }),
        precision: noesis_core::Precision::Standard,
        options: std::collections::HashMap::new(),
    }
}

/// Represents an optimal time window found across all three systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub quality: f64,
    pub activities: Vec<String>,
    pub supporting_systems: Vec<String>,
}

impl OptimalWindow {
    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn panchanga_from_json() {
        let json = json!({
            "tithi": {
                "name": "Shukla Panchami",
                "number": 5,
                "paksha": "Shukla"
            },
            "nakshatra": {
                "name": "Rohini",
                "number": 4,
                "quality": "Fixed",
                "deity": "Brahma"
            },
            "yoga": "Shiva",
            "karana": "Bava",
            "vara": "Thursday"
        });

        let data = PanchangaData::from_json(&json).unwrap();
        assert_eq!(data.tithi.name, "Shukla Panchami");
        assert_eq!(data.tithi.number, 5);
        assert_eq!(data.nakshatra.name, "Rohini");
    }

    #[test]
    fn vedic_clock_from_json() {
        let json = json!({
            "ghati": 25,
            "pala": 30,
            "muhurta": {
                "name": "Abhijit",
                "quality": "Auspicious"
            },
            "active_organ": "Heart",
            "dosha": "Pitta",
            "recommended_activity": "Important meetings"
        });

        let data = VedicClockData::from_json(&json).unwrap();
        assert_eq!(data.current_ghati, 25);
        assert_eq!(data.dosha_time, "Pitta");
    }

    #[test]
    fn biorhythm_from_json() {
        let json = json!({
            "physical": 0.8,
            "emotional": 0.5,
            "intellectual": -0.2
        });

        let data = BiorhythmData::from_json(&json).unwrap();
        assert_eq!(data.physical.value, 0.8);
        assert_eq!(data.physical.phase, "Peak");
        assert!(data.composite > 0.0);
    }

    #[test]
    fn biorhythm_critical_day() {
        let data = BiorhythmData {
            physical: CycleData { value: 0.05, phase: "Critical".to_string(), description: String::new() },
            emotional: CycleData { value: 0.5, phase: "High".to_string(), description: String::new() },
            intellectual: CycleData { value: 0.5, phase: "High".to_string(), description: String::new() },
            composite: 0.35,
        };

        assert!(data.has_critical_day());
    }
}
