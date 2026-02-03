//! Dasha Alerts Module
//!
//! FAPI-114: Dasha transition alerts and notifications

pub mod types;
pub mod monitor;
pub mod notifications;

pub use types::*;
pub use monitor::*;
pub use notifications::*;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Type of dasha transition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashaTransitionType {
    Mahadasha,
    Antardasha,
    Pratyantardasha,
    Sookshmadasha,
}

/// Dasha transition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaTransitionEvent {
    /// Type of transition
    pub transition_type: DashaTransitionType,
    /// Lord of ending period
    pub from_lord: String,
    /// Lord of starting period
    pub to_lord: String,
    /// Date of transition
    pub transition_date: NaiveDate,
    /// Days until transition
    pub days_until: i64,
    /// Significance of this transition
    pub significance: TransitionSignificance,
    /// Predictions and guidance
    pub guidance: String,
}

/// Significance level of a transition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionSignificance {
    Major,
    Moderate,
    Minor,
}

/// Dasha alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaAlertConfig {
    /// Enable alerts
    pub enabled: bool,
    /// Alert days before Mahadasha change
    pub mahadasha_alert_days: Vec<u32>,
    /// Alert days before Antardasha change
    pub antardasha_alert_days: Vec<u32>,
    /// Include pratyantardasha alerts
    pub include_pratyantardasha: bool,
    /// Custom messages
    pub custom_messages: std::collections::HashMap<String, String>,
}

impl Default for DashaAlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mahadasha_alert_days: vec![365, 180, 90, 30, 7],
            antardasha_alert_days: vec![30, 14, 7, 1],
            include_pratyantardasha: false,
            custom_messages: std::collections::HashMap::new(),
        }
    }
}
