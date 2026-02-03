//! Vimshottari Dasha types and calculations

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};

/// Complete Vimshottari Dasha system for a birth chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimshottariDasha {
    /// Birth date
    pub birth_date: String,
    /// Moon Nakshatra at birth
    pub moon_nakshatra: String,
    /// Moon's position in the nakshatra (0-360°)
    pub moon_longitude: f64,
    /// Balance of first dasha at birth
    pub balance: DashaBalance,
    /// Major periods (Mahadashas)
    pub mahadashas: Vec<DashaPeriod>,
    /// Current Mahadasha
    pub current_mahadasha: DashaPeriod,
    /// Current Antardasha
    pub current_antardasha: Option<DashaPeriod>,
    /// Current Pratyantardasha
    pub current_pratyantardasha: Option<DashaPeriod>,
    /// Current Sookshma (optional)
    pub current_sookshma: Option<DashaPeriod>,
}

/// Balance of first dasha at birth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaBalance {
    /// Planet ruling the first dasha
    pub planet: DashaPlanet,
    /// Years remaining at birth
    pub years_remaining: f64,
    /// Months remaining
    pub months_remaining: f64,
    /// Days remaining
    pub days_remaining: f64,
    /// Total period in years
    pub total_period_years: f64,
}

/// A Dasha period (Mahadasha, Antardasha, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaPeriod {
    /// Ruling planet
    pub planet: DashaPlanet,
    /// Level of this period
    pub level: DashaLevel,
    /// Start date
    pub start_date: String,
    /// End date
    pub end_date: String,
    /// Duration in years
    pub duration_years: f64,
    /// Duration in days
    pub duration_days: u32,
    /// Sub-periods (if available)
    pub sub_periods: Option<Vec<DashaPeriod>>,
}

impl DashaPeriod {
    /// Check if a given date falls within this period
    pub fn contains_date(&self, date: &str) -> bool {
        date >= self.start_date.as_str() && date <= self.end_date.as_str()
    }
    
    /// Get progress percentage at a given date
    pub fn progress_at(&self, date: &str) -> Option<f64> {
        if !self.contains_date(date) {
            return None;
        }
        
        // Simplified calculation - would need proper date parsing
        Some(0.5) // Placeholder
    }
}

/// Dasha level (hierarchy)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashaLevel {
    /// Mahadasha - Major period (~120 years total)
    Mahadasha,
    /// Antardasha - Sub-period (within Mahadasha)
    Antardasha,
    /// Pratyantardasha - Sub-sub-period
    Pratyantardasha,
    /// Sookshma - Minute period
    Sookshma,
    /// Praana - Very subtle period
    Praana,
}

impl DashaLevel {
    /// Get the depth level as a number
    pub fn depth(&self) -> u8 {
        match self {
            DashaLevel::Mahadasha => 1,
            DashaLevel::Antardasha => 2,
            DashaLevel::Pratyantardasha => 3,
            DashaLevel::Sookshma => 4,
            DashaLevel::Praana => 5,
        }
    }
    
    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            DashaLevel::Mahadasha => "Mahadasha",
            DashaLevel::Antardasha => "Antardasha",
            DashaLevel::Pratyantardasha => "Pratyantardasha",
            DashaLevel::Sookshma => "Sookshma",
            DashaLevel::Praana => "Praana",
        }
    }
    
    /// Get parent level
    pub fn parent(&self) -> Option<DashaLevel> {
        match self {
            DashaLevel::Mahadasha => None,
            DashaLevel::Antardasha => Some(DashaLevel::Mahadasha),
            DashaLevel::Pratyantardasha => Some(DashaLevel::Antardasha),
            DashaLevel::Sookshma => Some(DashaLevel::Pratyantardasha),
            DashaLevel::Praana => Some(DashaLevel::Sookshma),
        }
    }
}

/// The nine planets of Vimshottari Dasha
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashaPlanet {
    /// Ketu (South Node) - 7 years
    Ketu,
    /// Venus - 20 years
    Venus,
    /// Sun - 6 years
    Sun,
    /// Moon - 10 years
    Moon,
    /// Mars - 7 years
    Mars,
    /// Rahu (North Node) - 18 years
    Rahu,
    /// Jupiter - 16 years
    Jupiter,
    /// Saturn - 19 years
    Saturn,
    /// Mercury - 17 years
    Mercury,
}

impl DashaPlanet {
    /// Get the full dasha period in years
    pub fn full_period_years(&self) -> f64 {
        match self {
            DashaPlanet::Ketu => 7.0,
            DashaPlanet::Venus => 20.0,
            DashaPlanet::Sun => 6.0,
            DashaPlanet::Moon => 10.0,
            DashaPlanet::Mars => 7.0,
            DashaPlanet::Rahu => 18.0,
            DashaPlanet::Jupiter => 16.0,
            DashaPlanet::Saturn => 19.0,
            DashaPlanet::Mercury => 17.0,
        }
    }
    
    /// Get period in months
    pub fn full_period_months(&self) -> f64 {
        self.full_period_years() * 12.0
    }
    
    /// Get period in days (approximate)
    pub fn full_period_days(&self) -> f64 {
        self.full_period_years() * 365.25
    }
    
    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            DashaPlanet::Ketu => "Ketu",
            DashaPlanet::Venus => "Venus",
            DashaPlanet::Sun => "Sun",
            DashaPlanet::Moon => "Moon",
            DashaPlanet::Mars => "Mars",
            DashaPlanet::Rahu => "Rahu",
            DashaPlanet::Jupiter => "Jupiter",
            DashaPlanet::Saturn => "Saturn",
            DashaPlanet::Mercury => "Mercury",
        }
    }
    
    /// Get symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            DashaPlanet::Ketu => "☋",
            DashaPlanet::Venus => "♀",
            DashaPlanet::Sun => "☉",
            DashaPlanet::Moon => "☽",
            DashaPlanet::Mars => "♂",
            DashaPlanet::Rahu => "☊",
            DashaPlanet::Jupiter => "♃",
            DashaPlanet::Saturn => "♄",
            DashaPlanet::Mercury => "☿",
        }
    }
    
    /// Get nature (benefic, malefic, neutral)
    pub fn nature(&self) -> &'static str {
        match self {
            DashaPlanet::Jupiter | DashaPlanet::Venus | DashaPlanet::Moon | DashaPlanet::Mercury => "benefic",
            DashaPlanet::Saturn | DashaPlanet::Mars | DashaPlanet::Rahu | DashaPlanet::Ketu => "malefic",
            DashaPlanet::Sun => "neutral",
        }
    }
    
    /// Check if this planet is benefic
    pub fn is_benefic(&self) -> bool {
        matches!(self, 
            DashaPlanet::Jupiter | DashaPlanet::Venus | DashaPlanet::Moon | DashaPlanet::Mercury
        )
    }
    
    /// Check if this planet is malefic
    pub fn is_malefic(&self) -> bool {
        matches!(self,
            DashaPlanet::Saturn | DashaPlanet::Mars | DashaPlanet::Rahu | DashaPlanet::Ketu
        )
    }
    
    /// Get ruling Nakshatras
    pub fn ruling_nakshatras(&self) -> Vec<&'static str> {
        match self {
            DashaPlanet::Ketu => vec!["Ashwini", "Magha", "Mula"],
            DashaPlanet::Venus => vec!["Bharani", "Purva Phalguni", "Purva Ashadha"],
            DashaPlanet::Sun => vec!["Krittika", "Uttara Phalguni", "Uttara Ashadha"],
            DashaPlanet::Moon => vec!["Rohini", "Hasta", "Shravana"],
            DashaPlanet::Mars => vec!["Mrigashira", "Chitra", "Dhanishta"],
            DashaPlanet::Rahu => vec!["Ardra", "Swati", "Shatabhisha"],
            DashaPlanet::Jupiter => vec!["Punarvasu", "Vishakha", "Purva Bhadrapada"],
            DashaPlanet::Saturn => vec!["Pushya", "Anuradha", "Uttara Bhadrapada"],
            DashaPlanet::Mercury => vec!["Ashlesha", "Jyeshtha", "Revati"],
        }
    }
}

/// Full Dasha tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaTree {
    pub birth_date: String,
    pub moon_nakshatra: String,
    pub balance: DashaBalance,
    pub mahadashas: Vec<DashaPeriod>,
}

impl DashaTree {
    /// Get current running dashas at a specific date
    pub fn at_date(&self, date: &str) -> Option<CurrentDashas> {
        // Find the mahadasha
        let mahadasha = self.mahadashas.iter()
            .find(|m| m.contains_date(date))?;
        
        // Find antardasha within mahadasha
        let antardasha = mahadasha.sub_periods.as_ref()
            .and_then(|subs| subs.iter().find(|a| a.contains_date(date)));
        
        // Find pratyantardasha within antardasha
        let pratyantardasha = antardasha
            .and_then(|a| a.sub_periods.as_ref())
            .and_then(|subs| subs.iter().find(|p| p.contains_date(date)));
        
        // Find sookshma within pratyantardasha
        let sookshma = pratyantardasha
            .and_then(|p| p.sub_periods.as_ref())
            .and_then(|subs| subs.iter().find(|s| s.contains_date(date)));
        
        Some(CurrentDashas {
            mahadasha: mahadasha.clone(),
            antardasha: antardasha.cloned(),
            pratyantardasha: pratyantardasha.cloned(),
            sookshma: sookshma.cloned(),
        })
    }
    
    /// Get summary for a date
    pub fn summary_at(&self, date: &str) -> String {
        match self.at_date(date) {
            Some(dashas) => {
                let mut summary = format!("{} MD", dashas.mahadasha.planet.as_str());
                if let Some(ant) = dashas.antardasha {
                    summary.push_str(&format!(" - {} AD", ant.planet.as_str()));
                }
                if let Some(prat) = dashas.pratyantardasha {
                    summary.push_str(&format!(" - {} PD", prat.planet.as_str()));
                }
                summary
            }
            None => "No dasha found for date".to_string(),
        }
    }
}

/// Current running dashas at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentDashas {
    pub mahadasha: DashaPeriod,
    pub antardasha: Option<DashaPeriod>,
    pub pratyantardasha: Option<DashaPeriod>,
    pub sookshma: Option<DashaPeriod>,
}

impl CurrentDashas {
    /// Get the most specific period
    pub fn most_specific(&self) -> &DashaPeriod {
        if let Some(ref s) = self.sookshma {
            return s;
        }
        if let Some(ref p) = self.pratyantardasha {
            return p;
        }
        if let Some(ref a) = self.antardasha {
            return a;
        }
        &self.mahadasha
    }
    
    /// Format as standard dasha notation (MD-AD-PD)
    pub fn to_notation(&self) -> String {
        let mut parts = vec![format!("{}", self.mahadasha.planet.as_str())];
        
        if let Some(ref ant) = self.antardasha {
            parts.push(format!("{}", ant.planet.as_str()));
        }
        if let Some(ref prat) = self.pratyantardasha {
            parts.push(format!("{}", prat.planet.as_str()));
        }
        if let Some(ref sook) = self.sookshma {
            parts.push(format!("{}", sook.planet.as_str()));
        }
        
        parts.join("-")
    }
}

/// Standard Vimshottari sequence order
pub const DASHA_SEQUENCE: [DashaPlanet; 9] = [
    DashaPlanet::Ketu,
    DashaPlanet::Venus,
    DashaPlanet::Sun,
    DashaPlanet::Moon,
    DashaPlanet::Mars,
    DashaPlanet::Rahu,
    DashaPlanet::Jupiter,
    DashaPlanet::Saturn,
    DashaPlanet::Mercury,
];

/// Calculate balance of dasha based on Moon's position in nakshatra
pub fn calculate_dasha_balance(
    nakshatra: u8,  // 1-27
    pada: u8,       // 1-4
    moon_longitude: f64,
) -> DashaBalance {
    // Find starting planet based on nakshatra
    // Each planet rules 3 nakshatras
    let start_planet = match ((nakshatra - 1) / 3) % 9 {
        0 => DashaPlanet::Ketu,
        1 => DashaPlanet::Venus,
        2 => DashaPlanet::Sun,
        3 => DashaPlanet::Moon,
        4 => DashaPlanet::Mars,
        5 => DashaPlanet::Rahu,
        6 => DashaPlanet::Jupiter,
        7 => DashaPlanet::Saturn,
        _ => DashaPlanet::Mercury,
    };
    
    // Calculate balance based on pada (quarter)
    // If born in 1st pada = full period remaining
    // If born in 4th pada = 1/4 period remaining
    let total_period = start_planet.full_period_years();
    let balance_fraction = (5 - pada) as f64 / 4.0; // 1st pada = 1.0, 4th pada = 0.25
    let years_remaining = total_period * balance_fraction;
    
    DashaBalance {
        planet: start_planet,
        years_remaining,
        months_remaining: (years_remaining % 1.0) * 12.0,
        days_remaining: ((years_remaining % 1.0) * 12.0 % 1.0) * 30.0,
        total_period_years: total_period,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dasha_planet_periods() {
        assert_eq!(DashaPlanet::Sun.full_period_years(), 6.0);
        assert_eq!(DashaPlanet::Moon.full_period_years(), 10.0);
        assert_eq!(DashaPlanet::Saturn.full_period_years(), 19.0);
    }

    #[test]
    fn test_dasha_planet_nature() {
        assert!(DashaPlanet::Jupiter.is_benefic());
        assert!(DashaPlanet::Saturn.is_malefic());
        assert!(!DashaPlanet::Sun.is_benefic());
        assert!(!DashaPlanet::Sun.is_malefic());
    }

    #[test]
    fn test_dasha_level() {
        assert_eq!(DashaLevel::Mahadasha.depth(), 1);
        assert_eq!(DashaLevel::Pratyantardasha.depth(), 3);
        assert_eq!(DashaLevel::Antardasha.parent(), Some(DashaLevel::Mahadasha));
    }

    #[test]
    fn test_dasha_sequence() {
        assert_eq!(DASHA_SEQUENCE.len(), 9);
        assert_eq!(DASHA_SEQUENCE[0], DashaPlanet::Ketu);
        assert_eq!(DASHA_SEQUENCE[5], DashaPlanet::Rahu);
    }

    #[test]
    fn test_current_dashas_notation() {
        let dashas = CurrentDashas {
            mahadasha: DashaPeriod {
                planet: DashaPlanet::Mars,
                level: DashaLevel::Mahadasha,
                start_date: "2020-01-01".to_string(),
                end_date: "2027-01-01".to_string(),
                duration_years: 7.0,
                duration_days: 2557,
                sub_periods: None,
            },
            antardasha: Some(DashaPeriod {
                planet: DashaPlanet::Saturn,
                level: DashaLevel::Antardasha,
                start_date: "2024-01-01".to_string(),
                end_date: "2025-01-01".to_string(),
                duration_years: 1.0,
                duration_days: 365,
                sub_periods: None,
            }),
            pratyantardasha: None,
            sookshma: None,
        };
        
        assert_eq!(dashas.to_notation(), "Mars-Saturn");
    }
}
