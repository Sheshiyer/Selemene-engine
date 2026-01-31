//! Panchanga Consciousness Engine
//!
//! Calculates the five limbs of Vedic time: Tithi, Nakshatra, Yoga, Karana, Vara.
//! Migrated from the original Selemene Engine with ConsciousnessEngine trait implementation.

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{CalculationMetadata, ValidationResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Name lookup tables
// ---------------------------------------------------------------------------

const TITHI_NAMES: [&str; 30] = [
    "Pratipada (Shukla)",
    "Dwitiya (Shukla)",
    "Tritiya (Shukla)",
    "Chaturthi (Shukla)",
    "Panchami (Shukla)",
    "Shashthi (Shukla)",
    "Saptami (Shukla)",
    "Ashtami (Shukla)",
    "Navami (Shukla)",
    "Dashami (Shukla)",
    "Ekadashi (Shukla)",
    "Dwadashi (Shukla)",
    "Trayodashi (Shukla)",
    "Chaturdashi (Shukla)",
    "Purnima",
    "Pratipada (Krishna)",
    "Dwitiya (Krishna)",
    "Tritiya (Krishna)",
    "Chaturthi (Krishna)",
    "Panchami (Krishna)",
    "Shashthi (Krishna)",
    "Saptami (Krishna)",
    "Ashtami (Krishna)",
    "Navami (Krishna)",
    "Dashami (Krishna)",
    "Ekadashi (Krishna)",
    "Dwadashi (Krishna)",
    "Trayodashi (Krishna)",
    "Chaturdashi (Krishna)",
    "Amavasya",
];

const NAKSHATRA_NAMES: [&str; 27] = [
    "Ashwini",
    "Bharani",
    "Krittika",
    "Rohini",
    "Mrigashira",
    "Ardra",
    "Punarvasu",
    "Pushya",
    "Ashlesha",
    "Magha",
    "Purva Phalguni",
    "Uttara Phalguni",
    "Hasta",
    "Chitra",
    "Swati",
    "Vishakha",
    "Anuradha",
    "Jyeshtha",
    "Mula",
    "Purva Ashadha",
    "Uttara Ashadha",
    "Shravana",
    "Dhanishta",
    "Shatabhisha",
    "Purva Bhadrapada",
    "Uttara Bhadrapada",
    "Revati",
];

const YOGA_NAMES: [&str; 27] = [
    "Vishkambha",
    "Priti",
    "Ayushman",
    "Saubhagya",
    "Shobhana",
    "Atiganda",
    "Sukarma",
    "Dhriti",
    "Shula",
    "Ganda",
    "Vriddhi",
    "Dhruva",
    "Vyaghata",
    "Harshana",
    "Vajra",
    "Siddhi",
    "Vyatipata",
    "Variyan",
    "Parigha",
    "Shiva",
    "Siddha",
    "Sadhya",
    "Shubha",
    "Shukla",
    "Brahma",
    "Indra",
    "Vaidhriti",
];

const KARANA_NAMES: [&str; 11] = [
    "Bava",
    "Balava",
    "Kaulava",
    "Taitila",
    "Garaja",
    "Vanija",
    "Vishti",
    "Shakuni",
    "Chatushpada",
    "Naga",
    "Kimstughna",
];

const VARA_NAMES: [&str; 7] = [
    "Ravivara (Sunday)",
    "Somavara (Monday)",
    "Mangalavara (Tuesday)",
    "Budhavara (Wednesday)",
    "Guruvara (Thursday)",
    "Shukravara (Friday)",
    "Shanivara (Saturday)",
];

// ---------------------------------------------------------------------------
// PanchangaResult — local structured output
// ---------------------------------------------------------------------------

/// Structured Panchanga calculation result.
///
/// Holds both the raw numeric indices and the resolved Sanskrit names
/// for each of the five limbs of Vedic time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangaResult {
    /// Tithi index (0-based, 0..30)
    pub tithi_index: u8,
    /// Tithi name
    pub tithi_name: String,
    /// Raw tithi value (continuous)
    pub tithi_value: f64,

    /// Nakshatra index (0-based, 0..27)
    pub nakshatra_index: u8,
    /// Nakshatra name
    pub nakshatra_name: String,
    /// Raw nakshatra value (continuous)
    pub nakshatra_value: f64,

    /// Yoga index (0-based, 0..27)
    pub yoga_index: u8,
    /// Yoga name
    pub yoga_name: String,
    /// Raw yoga value (continuous)
    pub yoga_value: f64,

    /// Karana index (0-based, 0..11)
    pub karana_index: u8,
    /// Karana name
    pub karana_name: String,
    /// Raw karana value (continuous)
    pub karana_value: f64,

    /// Vara (day of week, 0 = Sunday)
    pub vara_index: u8,
    /// Vara name
    pub vara_name: String,

    /// Solar longitude in degrees (0..360)
    pub solar_longitude: f64,
    /// Lunar longitude in degrees (0..360)
    pub lunar_longitude: f64,
    /// Julian Day Number used for the calculation
    pub julian_day: f64,
}

// ---------------------------------------------------------------------------
// Core calculation functions (migrated from simple.rs)
// ---------------------------------------------------------------------------

/// Calculate the Julian Day Number for a given date and time.
///
/// `date` must be "YYYY-MM-DD", `time` must be "HH:MM".
/// `tz_offset_hours` is the UTC offset (e.g. 5.5 for IST).
pub fn calculate_julian_day(date: &str, time: &str, tz_offset_hours: f64) -> f64 {
    let date_parts: Vec<&str> = date.split('-').collect();
    let time_parts: Vec<&str> = time.split(':').collect();

    if date_parts.len() != 3 || time_parts.len() < 2 {
        return 0.0;
    }

    let year: i32 = date_parts[0].parse().unwrap_or(2000);
    let month: i32 = date_parts[1].parse().unwrap_or(1);
    let day: i32 = date_parts[2].parse().unwrap_or(1);
    let hour: f64 = time_parts[0].parse().unwrap_or(0.0);
    let minute: f64 = time_parts[1].parse().unwrap_or(0.0);

    // Convert local time to UTC
    let utc_hour = hour - tz_offset_hours;
    let utc_minute = minute;

    // Meeus Julian Day formula (approximate)
    let jd = 367.0 * year as f64
        - (7.0 * (year as f64 + ((month + 9) as f64 / 12.0).floor()) / 4.0).floor()
        + (275.0 * month as f64 / 9.0).floor()
        + day as f64
        + 1721013.5
        + utc_hour / 24.0
        + utc_minute / 1440.0;

    jd
}

/// Calculate apparent solar longitude (degrees, 0..360) for a given JD.
pub fn calculate_solar_position(jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    let l0 = 280.46645 + 36000.76983 * t + 0.0003032 * t * t;
    let l0 = l0 % 360.0;
    if l0 < 0.0 {
        l0 + 360.0
    } else {
        l0
    }
}

/// Calculate apparent lunar longitude (degrees, 0..360) for a given JD.
pub fn calculate_lunar_position(jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    let l = 218.3164477
        + 481267.88123421 * t
        - 0.0015786 * t * t
        + t * t * t / 538841.0
        - t * t * t * t / 65194000.0;
    let l = l % 360.0;
    if l < 0.0 {
        l + 360.0
    } else {
        l
    }
}

/// Calculate Tithi (lunar day, 0..30 continuous).
pub fn calculate_tithi(solar_longitude: f64, lunar_longitude: f64) -> f64 {
    let mut tithi = (lunar_longitude - solar_longitude) / 12.0;
    if tithi < 0.0 {
        tithi += 30.0;
    }
    tithi
}

/// Calculate Nakshatra (lunar mansion, 0..27 continuous).
pub fn calculate_nakshatra(lunar_longitude: f64) -> f64 {
    lunar_longitude / (360.0 / 27.0)
}

/// Calculate Yoga (luni-solar combination, 0..27 continuous).
pub fn calculate_yoga(solar_longitude: f64, lunar_longitude: f64) -> f64 {
    let yoga = (solar_longitude + lunar_longitude) / (360.0 / 27.0);
    yoga % 27.0
}

/// Calculate Karana (half-tithi, 0..11).
pub fn calculate_karana(tithi: f64) -> f64 {
    let karana = (tithi as i32) % 11;
    if karana == 0 {
        11.0
    } else {
        karana as f64
    }
}

/// Calculate Vara (day of the week, 0 = Sunday .. 6 = Saturday).
pub fn calculate_vara(jd: f64) -> i32 {
    let day_number = (jd + 1.5) as i64;
    (day_number % 7) as i32
}

// ---------------------------------------------------------------------------
// High-level Panchanga calculation
// ---------------------------------------------------------------------------

/// Compute a full `PanchangaResult` from date, time, and timezone offset.
pub fn compute_panchanga(date: &str, time: &str, tz_offset_hours: f64) -> PanchangaResult {
    let jd = calculate_julian_day(date, time, tz_offset_hours);
    let solar_lng = calculate_solar_position(jd);
    let lunar_lng = calculate_lunar_position(jd);
    let tithi_val = calculate_tithi(solar_lng, lunar_lng);
    let nakshatra_val = calculate_nakshatra(lunar_lng);
    let yoga_val = calculate_yoga(solar_lng, lunar_lng);
    let karana_val = calculate_karana(tithi_val);
    let vara_val = calculate_vara(jd);

    let tithi_idx = (tithi_val.floor() as usize).min(29);
    let nakshatra_idx = (nakshatra_val.floor() as usize).min(26);
    let yoga_idx = (yoga_val.floor() as usize).min(26);
    let karana_idx = ((karana_val.floor() as usize).max(1) - 1).min(10);
    let vara_idx = (vara_val as usize).min(6);

    PanchangaResult {
        tithi_index: tithi_idx as u8,
        tithi_name: TITHI_NAMES[tithi_idx].to_string(),
        tithi_value: tithi_val,

        nakshatra_index: nakshatra_idx as u8,
        nakshatra_name: NAKSHATRA_NAMES[nakshatra_idx].to_string(),
        nakshatra_value: nakshatra_val,

        yoga_index: yoga_idx as u8,
        yoga_name: YOGA_NAMES[yoga_idx].to_string(),
        yoga_value: yoga_val,

        karana_index: karana_idx as u8,
        karana_name: KARANA_NAMES[karana_idx].to_string(),
        karana_value: karana_val,

        vara_index: vara_idx as u8,
        vara_name: VARA_NAMES[vara_idx].to_string(),

        solar_longitude: solar_lng,
        lunar_longitude: lunar_lng,
        julian_day: jd,
    }
}

// ---------------------------------------------------------------------------
// Witness prompt generation
// ---------------------------------------------------------------------------

fn generate_witness_prompt(result: &PanchangaResult) -> String {
    format!(
        "You were born under Tithi {} and Nakshatra {}. \
         The Tithi reflects the angle between Sun and Moon at your birth \
         — a mirror of the dance between your conscious will and emotional nature. \
         What pattern do you notice when your intentions and feelings are misaligned?",
        result.tithi_name, result.nakshatra_name
    )
}

// ---------------------------------------------------------------------------
// Timezone offset helper
// ---------------------------------------------------------------------------

/// Derive a numeric UTC offset from a timezone string.
///
/// Supports IANA names for a handful of common zones and explicit
/// "+HH:MM" / "-HH:MM" offsets. Defaults to 0.0 (UTC) if unknown.
fn tz_offset_from_string(tz: &str) -> f64 {
    // Try explicit numeric offset first: "+05:30", "-08:00", etc.
    if tz.starts_with('+') || tz.starts_with('-') {
        let parts: Vec<&str> = tz[1..].split(':').collect();
        let sign: f64 = if tz.starts_with('-') { -1.0 } else { 1.0 };
        let hours: f64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let minutes: f64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        return sign * (hours + minutes / 60.0);
    }

    // Common IANA zone names (enough for MVP)
    match tz {
        "Asia/Kolkata" | "Asia/Calcutta" => 5.5,
        "Asia/Tokyo" => 9.0,
        "Asia/Shanghai" | "Asia/Hong_Kong" => 8.0,
        "Asia/Dubai" => 4.0,
        "Asia/Kathmandu" => 5.75,
        "Europe/London" | "GMT" | "UTC" => 0.0,
        "Europe/Paris" | "Europe/Berlin" | "CET" => 1.0,
        "Europe/Moscow" => 3.0,
        "America/New_York" | "US/Eastern" | "EST" => -5.0,
        "America/Chicago" | "US/Central" | "CST" => -6.0,
        "America/Denver" | "US/Mountain" | "MST" => -7.0,
        "America/Los_Angeles" | "US/Pacific" | "PST" => -8.0,
        "Pacific/Honolulu" | "HST" => -10.0,
        "Australia/Sydney" | "AEST" => 10.0,
        _ => 0.0,
    }
}

// ---------------------------------------------------------------------------
// PanchangaEngine — ConsciousnessEngine implementation
// ---------------------------------------------------------------------------

/// The Panchanga consciousness engine.
///
/// Stateless — all configuration comes through `EngineInput`.
pub struct PanchangaEngine;

impl PanchangaEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PanchangaEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for PanchangaEngine {
    fn engine_id(&self) -> &str {
        "panchanga"
    }

    fn engine_name(&self) -> &str {
        "Panchanga"
    }

    fn required_phase(&self) -> u8 {
        0 // Available at the earliest consciousness phase
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        let birth = input.birth_data.as_ref().ok_or_else(|| {
            EngineError::CalculationError(
                "birth_data is required for Panchanga calculations".into(),
            )
        })?;

        let date = &birth.date;
        let time = birth.time.as_deref().unwrap_or("12:00");
        let tz_offset = tz_offset_from_string(&birth.timezone);

        let result = compute_panchanga(date, time, tz_offset);
        let witness_prompt = generate_witness_prompt(&result);

        let result_json = serde_json::to_value(&result).map_err(|e| {
            EngineError::CalculationError(format!("failed to serialize PanchangaResult: {e}"))
        })?;

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(EngineOutput {
            engine_id: self.engine_id().to_string(),
            result: result_json,
            witness_prompt,
            consciousness_level: 0,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed_ms,
                backend: "native-rust".to_string(),
                precision_achieved: format!("{:?}", input.precision),
                cached: false,
                timestamp: Utc::now(),
            },
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let mut messages: Vec<String> = Vec::new();
        let mut valid = true;

        // Attempt to deserialize the result back into PanchangaResult
        let pr: PanchangaResult = serde_json::from_value(output.result.clone()).map_err(|e| {
            EngineError::ValidationError(format!("cannot deserialize PanchangaResult: {e}"))
        })?;

        // Solar longitude must be 0..360
        if pr.solar_longitude < 0.0 || pr.solar_longitude >= 360.0 {
            valid = false;
            messages.push(format!(
                "solar_longitude out of range: {}",
                pr.solar_longitude
            ));
        }

        // Lunar longitude must be 0..360
        if pr.lunar_longitude < 0.0 || pr.lunar_longitude >= 360.0 {
            valid = false;
            messages.push(format!(
                "lunar_longitude out of range: {}",
                pr.lunar_longitude
            ));
        }

        // Tithi value must be 0..30
        if pr.tithi_value < 0.0 || pr.tithi_value >= 30.0 {
            valid = false;
            messages.push(format!("tithi_value out of range: {}", pr.tithi_value));
        }

        // Nakshatra value must be 0..27
        if pr.nakshatra_value < 0.0 || pr.nakshatra_value >= 27.0 {
            valid = false;
            messages.push(format!(
                "nakshatra_value out of range: {}",
                pr.nakshatra_value
            ));
        }

        // Yoga value must be 0..27
        if pr.yoga_value < 0.0 || pr.yoga_value >= 27.0 {
            valid = false;
            messages.push(format!("yoga_value out of range: {}", pr.yoga_value));
        }

        // Julian day should be positive and reasonable (> 0 AD)
        if pr.julian_day < 1721425.5 {
            valid = false;
            messages.push(format!(
                "julian_day seems too early (before 1 AD): {}",
                pr.julian_day
            ));
        }

        if valid {
            messages.push("all Panchanga values within expected ranges".to_string());
        }

        let confidence = if valid { 1.0 } else { 0.0 };

        Ok(ValidationResult {
            valid,
            confidence,
            messages,
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        let birth = input.birth_data.as_ref();
        let date = birth.map(|b| b.date.as_str()).unwrap_or("");
        let time = birth
            .and_then(|b| b.time.as_deref())
            .unwrap_or("12:00");
        let lat = birth.map(|b| b.latitude).unwrap_or(0.0);
        let lon = birth.map(|b| b.longitude).unwrap_or(0.0);

        let raw = format!("panchanga:{}:{}:{:.6}:{:.6}", date, time, lat, lon);
        let hash = Sha256::digest(raw.as_bytes());
        format!("panchanga:{:x}", hash)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use noesis_core::{BirthData, EngineInput, Precision};
    use std::collections::HashMap;

    fn test_birth_data() -> BirthData {
        BirthData {
            name: Some("Test".to_string()),
            date: "1991-08-13".to_string(),
            time: Some("13:31".to_string()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: "Asia/Kolkata".to_string(),
        }
    }

    fn test_input() -> EngineInput {
        EngineInput {
            birth_data: Some(test_birth_data()),
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        }
    }

    #[test]
    fn test_julian_day_reasonable() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        // JD for 1991-08-13 should be around 2448483
        assert!(jd > 2448480.0 && jd < 2448490.0, "JD = {jd}");
    }

    #[test]
    fn test_solar_position_range() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        let solar = calculate_solar_position(jd);
        assert!(solar >= 0.0 && solar < 360.0, "solar = {solar}");
    }

    #[test]
    fn test_lunar_position_range() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        let lunar = calculate_lunar_position(jd);
        assert!(lunar >= 0.0 && lunar < 360.0, "lunar = {lunar}");
    }

    #[test]
    fn test_tithi_range() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        let solar = calculate_solar_position(jd);
        let lunar = calculate_lunar_position(jd);
        let tithi = calculate_tithi(solar, lunar);
        assert!(tithi >= 0.0 && tithi < 30.0, "tithi = {tithi}");
    }

    #[test]
    fn test_nakshatra_range() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        let lunar = calculate_lunar_position(jd);
        let nak = calculate_nakshatra(lunar);
        assert!(nak >= 0.0 && nak < 27.0, "nakshatra = {nak}");
    }

    #[test]
    fn test_yoga_range() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        let solar = calculate_solar_position(jd);
        let lunar = calculate_lunar_position(jd);
        let yoga = calculate_yoga(solar, lunar);
        assert!(yoga >= 0.0 && yoga < 27.0, "yoga = {yoga}");
    }

    #[test]
    fn test_vara_range() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        let vara = calculate_vara(jd);
        assert!((0..7).contains(&vara), "vara = {vara}");
    }

    #[test]
    fn test_compute_panchanga_names_populated() {
        let p = compute_panchanga("1991-08-13", "13:31", 5.5);
        assert!(!p.tithi_name.is_empty());
        assert!(!p.nakshatra_name.is_empty());
        assert!(!p.yoga_name.is_empty());
        assert!(!p.karana_name.is_empty());
        assert!(!p.vara_name.is_empty());
    }

    #[test]
    fn test_cache_key_deterministic() {
        let engine = PanchangaEngine::new();
        let input = test_input();
        let k1 = engine.cache_key(&input);
        let k2 = engine.cache_key(&input);
        assert_eq!(k1, k2);
        assert!(k1.starts_with("panchanga:"));
    }

    #[test]
    fn test_engine_metadata() {
        let engine = PanchangaEngine::new();
        assert_eq!(engine.engine_id(), "panchanga");
        assert_eq!(engine.engine_name(), "Panchanga");
        assert_eq!(engine.required_phase(), 0);
    }

    #[tokio::test]
    async fn test_calculate_returns_valid_output() {
        let engine = PanchangaEngine::new();
        let input = test_input();
        let output = engine.calculate(input).await.expect("calculate failed");
        assert_eq!(output.engine_id, "panchanga");
        assert!(!output.witness_prompt.is_empty());
        assert_eq!(output.metadata.backend, "native-rust");

        // The result should deserialize back into PanchangaResult
        let pr: PanchangaResult =
            serde_json::from_value(output.result).expect("bad PanchangaResult JSON");
        assert!(pr.solar_longitude >= 0.0 && pr.solar_longitude < 360.0);
    }

    #[tokio::test]
    async fn test_calculate_missing_birth_data_errors() {
        let engine = PanchangaEngine::new();
        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };
        let result = engine.calculate(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_accepts_good_output() {
        let engine = PanchangaEngine::new();
        let input = test_input();
        let output = engine.calculate(input).await.unwrap();
        let vr = engine.validate(&output).await.unwrap();
        assert!(vr.valid);
        assert_eq!(vr.confidence, 1.0);
    }
}
