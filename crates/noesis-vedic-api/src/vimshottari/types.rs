//! Vimshottari Dasha types
//!
//! FAPI-031: Define Vimshottari request/response types

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// Planet (Graha) that rules a Dasha period
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DashaLord {
    Sun,
    Moon,
    Mars,
    Rahu,
    Jupiter,
    Saturn,
    Mercury,
    Ketu,
    Venus,
}

impl DashaLord {
    /// Get the total years for this lord's Mahadasha
    pub fn mahadasha_years(&self) -> f64 {
        match self {
            DashaLord::Sun => 6.0,
            DashaLord::Moon => 10.0,
            DashaLord::Mars => 7.0,
            DashaLord::Rahu => 18.0,
            DashaLord::Jupiter => 16.0,
            DashaLord::Saturn => 19.0,
            DashaLord::Mercury => 17.0,
            DashaLord::Ketu => 7.0,
            DashaLord::Venus => 20.0,
        }
    }

    /// Get the natural sequence order (Ketu starts the 120-year cycle)
    pub fn sequence_order(&self) -> u8 {
        match self {
            DashaLord::Ketu => 0,
            DashaLord::Venus => 1,
            DashaLord::Sun => 2,
            DashaLord::Moon => 3,
            DashaLord::Mars => 4,
            DashaLord::Rahu => 5,
            DashaLord::Jupiter => 6,
            DashaLord::Saturn => 7,
            DashaLord::Mercury => 8,
        }
    }

    /// Get the next lord in sequence
    pub fn next(&self) -> DashaLord {
        match self {
            DashaLord::Ketu => DashaLord::Venus,
            DashaLord::Venus => DashaLord::Sun,
            DashaLord::Sun => DashaLord::Moon,
            DashaLord::Moon => DashaLord::Mars,
            DashaLord::Mars => DashaLord::Rahu,
            DashaLord::Rahu => DashaLord::Jupiter,
            DashaLord::Jupiter => DashaLord::Saturn,
            DashaLord::Saturn => DashaLord::Mercury,
            DashaLord::Mercury => DashaLord::Ketu,
        }
    }

    /// Get all lords in Vimshottari sequence
    pub fn sequence() -> [DashaLord; 9] {
        [
            DashaLord::Ketu,
            DashaLord::Venus,
            DashaLord::Sun,
            DashaLord::Moon,
            DashaLord::Mars,
            DashaLord::Rahu,
            DashaLord::Jupiter,
            DashaLord::Saturn,
            DashaLord::Mercury,
        ]
    }
}

impl std::fmt::Display for DashaLord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DashaLord::Sun => write!(f, "Sun"),
            DashaLord::Moon => write!(f, "Moon"),
            DashaLord::Mars => write!(f, "Mars"),
            DashaLord::Rahu => write!(f, "Rahu"),
            DashaLord::Jupiter => write!(f, "Jupiter"),
            DashaLord::Saturn => write!(f, "Saturn"),
            DashaLord::Mercury => write!(f, "Mercury"),
            DashaLord::Ketu => write!(f, "Ketu"),
            DashaLord::Venus => write!(f, "Venus"),
        }
    }
}

/// Level of Dasha detail to retrieve
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashaLevel {
    /// Mahadasha only (9 periods)
    Mahadasha,
    /// Mahadasha + Antardasha (81 periods)
    Antardasha,
    /// Down to Pratyantardasha (729 periods)
    Pratyantardasha,
    /// Down to Sookshma Dasha (6561 periods)
    Sookshma,
    /// Down to Prana Dasha (full detail)
    Prana,
}

impl Default for DashaLevel {
    fn default() -> Self {
        DashaLevel::Antardasha
    }
}

/// A single Dasha period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaPeriod {
    /// Ruling planet
    pub lord: DashaLord,
    /// Level of this period
    pub level: DashaLevel,
    /// Start date of the period
    pub start_date: NaiveDate,
    /// End date of the period
    pub end_date: NaiveDate,
    /// Duration in days
    pub duration_days: i64,
    /// Sub-periods (if requested)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sub_periods: Vec<DashaPeriod>,
}

impl DashaPeriod {
    /// Check if a given date falls within this period
    pub fn contains_date(&self, date: NaiveDate) -> bool {
        date >= self.start_date && date <= self.end_date
    }

    /// Get the percentage elapsed as of a given date
    pub fn percent_elapsed(&self, as_of: NaiveDate) -> f64 {
        if as_of < self.start_date {
            return 0.0;
        }
        if as_of > self.end_date {
            return 100.0;
        }
        
        let total_days = (self.end_date - self.start_date).num_days() as f64;
        let elapsed_days = (as_of - self.start_date).num_days() as f64;
        (elapsed_days / total_days) * 100.0
    }

    /// Get the remaining duration from a given date
    pub fn days_remaining(&self, as_of: NaiveDate) -> i64 {
        if as_of > self.end_date {
            return 0;
        }
        (self.end_date - as_of).num_days()
    }
}

/// Complete Vimshottari Dasha timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimshottariTimeline {
    /// Birth datetime used for calculation
    pub birth_datetime: NaiveDateTime,
    /// Moon's nakshatra at birth
    pub birth_nakshatra: String,
    /// Balance of birth dasha
    pub birth_dasha_balance: DashaBalance,
    /// All Mahadasha periods
    pub mahadashas: Vec<DashaPeriod>,
}

impl VimshottariTimeline {
    /// Get the current Mahadasha for a given date
    pub fn current_mahadasha(&self, date: NaiveDate) -> Option<&DashaPeriod> {
        self.mahadashas.iter().find(|md| md.contains_date(date))
    }

    /// Get current Antardasha for a given date
    pub fn current_antardasha(&self, date: NaiveDate) -> Option<&DashaPeriod> {
        self.current_mahadasha(date)
            .and_then(|md| md.sub_periods.iter().find(|ad| ad.contains_date(date)))
    }

    /// Get all current dasha levels at a given date
    pub fn current_dashas_at(&self, date: NaiveDate) -> CurrentDashas {
        let mahadasha = self.current_mahadasha(date).cloned();
        let antardasha = mahadasha.as_ref()
            .and_then(|md| md.sub_periods.iter().find(|ad| ad.contains_date(date)).cloned());
        let pratyantardasha = antardasha.as_ref()
            .and_then(|ad| ad.sub_periods.iter().find(|pd| pd.contains_date(date)).cloned());
        
        CurrentDashas {
            mahadasha,
            antardasha,
            pratyantardasha,
            sookshma: None,
            prana: None,
        }
    }
}

/// Balance of dasha at birth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaBalance {
    /// Lord of the birth dasha
    pub lord: DashaLord,
    /// Years remaining
    pub years: u32,
    /// Months remaining
    pub months: u32,
    /// Days remaining
    pub days: u32,
    /// Total days remaining
    pub total_days: i64,
}

/// Current dasha periods at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentDashas {
    pub mahadasha: Option<DashaPeriod>,
    pub antardasha: Option<DashaPeriod>,
    pub pratyantardasha: Option<DashaPeriod>,
    pub sookshma: Option<DashaPeriod>,
    pub prana: Option<DashaPeriod>,
}

impl CurrentDashas {
    /// Get a display string like "Sun-Moon-Mars"
    pub fn display_string(&self) -> String {
        let mut parts = vec![];
        if let Some(ref md) = self.mahadasha {
            parts.push(md.lord.to_string());
        }
        if let Some(ref ad) = self.antardasha {
            parts.push(ad.lord.to_string());
        }
        if let Some(ref pd) = self.pratyantardasha {
            parts.push(pd.lord.to_string());
        }
        parts.join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dasha_lord_years() {
        assert_eq!(DashaLord::Sun.mahadasha_years(), 6.0);
        assert_eq!(DashaLord::Moon.mahadasha_years(), 10.0);
        assert_eq!(DashaLord::Venus.mahadasha_years(), 20.0);
        
        // Total should be 120 years
        let total: f64 = DashaLord::sequence().iter().map(|l| l.mahadasha_years()).sum();
        assert_eq!(total, 120.0);
    }

    #[test]
    fn test_dasha_lord_sequence() {
        let seq = DashaLord::sequence();
        assert_eq!(seq[0], DashaLord::Ketu);
        assert_eq!(seq[8], DashaLord::Mercury);
    }

    #[test]
    fn test_dasha_lord_next() {
        assert_eq!(DashaLord::Ketu.next(), DashaLord::Venus);
        assert_eq!(DashaLord::Mercury.next(), DashaLord::Ketu);
    }

    #[test]
    fn test_dasha_period_contains() {
        let period = DashaPeriod {
            lord: DashaLord::Sun,
            level: DashaLevel::Mahadasha,
            start_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            duration_days: 2192,
            sub_periods: vec![],
        };

        assert!(period.contains_date(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap()));
        assert!(!period.contains_date(NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()));
    }

    #[test]
    fn test_dasha_period_percent_elapsed() {
        let period = DashaPeriod {
            lord: DashaLord::Moon,
            level: DashaLevel::Mahadasha,
            start_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2030, 1, 1).unwrap(),
            duration_days: 3652,
            sub_periods: vec![],
        };

        let half = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let percent = period.percent_elapsed(half);
        assert!(percent > 49.0 && percent < 51.0);
    }
}
