//! Hora Alarms Module
//!
//! FAPI-113: Planetary hour notifications and alarms

pub mod types;
pub mod calculator;
pub mod scheduler;

pub use types::*;
pub use calculator::*;
pub use scheduler::*;

use chrono::{NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

/// Planetary hora (hour)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryHora {
    /// Hora number (1-24)
    pub number: u8,
    /// Ruling planet
    pub ruler: String,
    /// Start time
    pub start_time: NaiveTime,
    /// End time
    pub end_time: NaiveTime,
    /// Is this a day hora or night hora
    pub is_day_hora: bool,
    /// Quality assessment
    pub quality: HoraQuality,
}

/// Quality of a hora for various activities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoraQuality {
    Excellent,
    Good,
    Neutral,
    Challenging,
}

/// Hora alarm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoraAlarm {
    /// Alarm ID
    pub id: String,
    /// Planet to watch for
    pub planet: String,
    /// Notify at start of hora
    pub notify_start: bool,
    /// Notify minutes before
    pub notify_before_minutes: Option<u32>,
    /// Custom message
    pub message: Option<String>,
    /// Is alarm enabled
    pub enabled: bool,
}

/// Hora alarm notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoraNotification {
    /// Associated hora
    pub hora: PlanetaryHora,
    /// Notification time
    pub notification_time: NaiveDateTime,
    /// Message
    pub message: String,
    /// Has been delivered
    pub delivered: bool,
}
