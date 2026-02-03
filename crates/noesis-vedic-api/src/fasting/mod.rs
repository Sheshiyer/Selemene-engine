//! Fasting (Vrat) Module
//!
//! FAPI-117: Hindu fasting days and observances

pub mod types;
pub mod calendar;
pub mod rules;

pub use types::*;
pub use calendar::*;
pub use rules::*;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Type of fast
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastType {
    /// Complete fast (no food or water)
    Nirjala,
    /// Fruit diet only
    Phalahara,
    /// Single meal
    Ekadashi,
    /// No grains
    AnnaVrat,
    /// Milk/water only
    DudhVrat,
    /// Partial fast
    Partial,
}

impl std::fmt::Display for FastType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FastType::Nirjala => write!(f, "Nirjala (Waterless)"),
            FastType::Phalahara => write!(f, "Phalahara (Fruits only)"),
            FastType::Ekadashi => write!(f, "Ekadashi (Single meal)"),
            FastType::AnnaVrat => write!(f, "Anna Vrat (No grains)"),
            FastType::DudhVrat => write!(f, "Dudh Vrat (Milk only)"),
            FastType::Partial => write!(f, "Partial Fast"),
        }
    }
}

/// Fasting day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastingDay {
    /// Date
    pub date: NaiveDate,
    /// Name of vrat
    pub name: String,
    /// Type of fast
    pub fast_type: FastType,
    /// Associated deity
    pub deity: String,
    /// Tithi
    pub tithi: String,
    /// Breaking time (parana)
    pub parana_time: Option<String>,
    /// What to eat
    pub allowed_foods: Vec<String>,
    /// What to avoid
    pub restricted_foods: Vec<String>,
    /// Benefits
    pub benefits: String,
    /// Mantras to chant
    pub mantras: Vec<String>,
}

/// Monthly fasting schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyFastingSchedule {
    pub year: i32,
    pub month: u32,
    pub fasting_days: Vec<FastingDay>,
    pub ekadashis: Vec<FastingDay>,
    pub pradosh_vrats: Vec<FastingDay>,
    pub special_vrats: Vec<FastingDay>,
}
