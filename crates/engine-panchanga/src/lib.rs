//! Panchanga Consciousness Engine
//!
//! Calculates the five limbs of Vedic time: Tithi, Nakshatra, Yoga, Karana, Vara.
//! Migrated from the original Selemene Engine with ConsciousnessEngine trait implementation.

use async_trait::async_trait;
use chrono::{
    DateTime, Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Utc,
};
use engine_human_design::ephemeris::{EphemerisCalculator, HDPlanet};
use noesis_core::{CalculationMetadata, ValidationResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;

pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};

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
    "Gara",
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

/// Approximate Lahiri ayanamsa used to convert tropical longitudes to sidereal.
const LAHIRI_AYANAMSA_DEGREES: f64 = 23.72;

/// Convert tropical longitude to sidereal longitude using Lahiri ayanamsa.
fn to_sidereal_longitude(tropical_longitude: f64) -> f64 {
    (tropical_longitude - LAHIRI_AYANAMSA_DEGREES).rem_euclid(360.0)
}

fn parse_local_datetime(date: &str, time: &str) -> Option<NaiveDateTime> {
    let local_date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let local_time = NaiveTime::parse_from_str(time, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(time, "%H:%M:%S"))
        .ok()?;
    Some(NaiveDateTime::new(local_date, local_time))
}

fn parse_local_datetime_to_utc(
    date: &str,
    time: &str,
    tz_offset_hours: f64,
) -> Option<chrono::DateTime<Utc>> {
    let local_dt = parse_local_datetime(date, time)?;
    let offset_seconds = (tz_offset_hours * 3600.0).round() as i64;
    let utc_dt = local_dt - chrono::Duration::seconds(offset_seconds);
    Some(Utc.from_utc_datetime(&utc_dt))
}

fn precise_tropical_positions(utc_dt: &DateTime<Utc>) -> Option<(f64, f64)> {
    let ephemeris = EphemerisCalculator::new("");
    let sun = ephemeris.get_planet_position(HDPlanet::Sun, utc_dt).ok()?;
    let moon = ephemeris.get_planet_position(HDPlanet::Moon, utc_dt).ok()?;
    Some((sun.longitude, moon.longitude))
}

/// Julian Day (UT) of a UTC instant, seconds included.
fn julian_day_from_utc(utc_dt: &DateTime<Utc>) -> f64 {
    const UNIX_EPOCH_JD: f64 = 2_440_587.5;
    UNIX_EPOCH_JD
        + utc_dt.timestamp() as f64 / 86_400.0
        + utc_dt.timestamp_subsec_nanos() as f64 / 86_400e9
}

// ---------------------------------------------------------------------------
// PanchangaResult — local structured output
// ---------------------------------------------------------------------------

/// Which path produced the solar and lunar longitudes that tithi, nakshatra,
/// yoga and karana are derived from. Vara is calendar arithmetic on the local
/// date and does not depend on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionSource {
    /// Swiss Ephemeris computed from the `.se1` data files.
    SwissEphemerisFiles,
    /// Swiss Ephemeris substituted its built-in Moshier ephemeris because no
    /// `.se1` files were reachable. A fallback (`fallback_used: true`).
    SwissEphemerisMoshier,
    /// Swiss Ephemeris was unavailable for this input and the mean-longitude
    /// approximation was used. A fallback (`fallback_used: true`).
    MeanLongitude,
}

impl PositionSource {
    /// Whether this source is a degraded path (`metadata.fallback_used`).
    pub fn is_fallback(self) -> bool {
        self != PositionSource::SwissEphemerisFiles
    }
}

/// The day a vara is the weekday of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VaraBasis {
    /// The local civil date (midnight to midnight) at the location's UTC
    /// offset. Sunrise-based reckoning would need sunrise for the location,
    /// which this engine does not compute.
    LocalDate,
}

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
    /// The day the vara was taken from. `None` only when deserializing a
    /// result serialized before this field existed (those used the UTC date).
    #[serde(default)]
    pub vara_basis: Option<VaraBasis>,

    /// Solar longitude in degrees (0..360)
    pub solar_longitude: f64,
    /// Lunar longitude in degrees (0..360)
    pub lunar_longitude: f64,
    /// Julian Day Number used for the calculation
    pub julian_day: f64,

    /// Path that produced the longitudes. `None` only when deserializing a
    /// result serialized before this field existed.
    #[serde(default)]
    pub position_source: Option<PositionSource>,
    /// The birth time was missing and local noon was assumed. Input
    /// defaulting, not a calculation fallback: `fallback_used` ignores it.
    #[serde(default)]
    pub birth_time_assumed: bool,
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

    367.0 * year as f64 - (7.0 * (year as f64 + ((month + 9) as f64 / 12.0).floor()) / 4.0).floor()
        + (275.0 * month as f64 / 9.0).floor()
        + day as f64
        + 1721013.5
        + utc_hour / 24.0
        + utc_minute / 1440.0
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
    let l = 218.3164477 + 481267.88123421 * t - 0.0015786 * t * t + t * t * t / 538841.0
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

fn karana_slot_from_tithi(tithi: f64) -> u8 {
    let slot = (tithi * 2.0).floor() as i32 + 1;
    slot.clamp(1, 60) as u8
}

fn karana_identity(slot: u8) -> (u8, &'static str) {
    match slot {
        1 => (10, KARANA_NAMES[10]),
        2..=57 => {
            let idx = ((slot - 2) % 7) as usize;
            (idx as u8, KARANA_NAMES[idx])
        }
        58 => (7, KARANA_NAMES[7]),
        59 => (8, KARANA_NAMES[8]),
        60 => (9, KARANA_NAMES[9]),
        _ => unreachable!("karana slot must be in 1..=60"),
    }
}

/// Calculate Karana slot (half-tithi, 1..=60).
pub fn calculate_karana(tithi: f64) -> f64 {
    karana_slot_from_tithi(tithi) as f64
}

/// Calculate Vara (day of the week, 0 = Sunday .. 6 = Saturday) of the civil
/// date containing `jd`. Pass the local Julian Day (UT Julian Day plus the
/// UTC offset in days) for the local weekday.
pub fn calculate_vara(jd: f64) -> i32 {
    let day_number = (jd + 1.5) as i64;
    (day_number % 7) as i32
}

// ---------------------------------------------------------------------------
// High-level Panchanga calculation
// ---------------------------------------------------------------------------

/// Compute a full `PanchangaResult` from date, time, and timezone offset.
pub fn compute_panchanga(date: &str, time: &str, tz_offset_hours: f64) -> PanchangaResult {
    compute_panchanga_reporting_fallback(date, time, tz_offset_hours).0
}

/// Like [`compute_panchanga`], also reporting whether a fallback was used: the
/// mean-longitude approximation replaced Swiss Ephemeris positions, or Swiss
/// Ephemeris itself substituted Moshier for the `.se1` files. Which one is
/// recorded in [`PanchangaResult::position_source`].
pub fn compute_panchanga_reporting_fallback(
    date: &str,
    time: &str,
    tz_offset_hours: f64,
) -> (PanchangaResult, bool) {
    match parse_local_datetime_to_utc(date, time, tz_offset_hours) {
        Some(utc_dt) => compute_panchanga_at_instant(&utc_dt, tz_offset_hours),
        // Swiss Ephemeris needs a parsed instant; keep the original
        // mean-longitude path for input only the Meeus formula accepts.
        None => {
            let jd = calculate_julian_day(date, time, tz_offset_hours);
            build_result(
                jd,
                calculate_solar_position(jd),
                calculate_lunar_position(jd),
                PositionSource::MeanLongitude,
                tz_offset_hours,
            )
        }
    }
}

/// Compute the Panchanga for a UTC instant, reporting whether a fallback was
/// used as [`compute_panchanga_reporting_fallback`] does. `tz_offset_hours`
/// is the local UTC offset at that instant; it only selects the local date
/// for the vara and never moves the positions.
pub fn compute_panchanga_at_instant(
    utc_dt: &DateTime<Utc>,
    tz_offset_hours: f64,
) -> (PanchangaResult, bool) {
    let jd = julian_day_from_utc(utc_dt);

    // Prefer local Swiss Ephemeris positions for correct transition timing,
    // but retain the original mean-longitude fallback if ephemeris access fails.
    // Moshier substitution inside Swiss Ephemeris is a fallback too.
    engine_human_design::ephemeris::reset_moshier_observed();
    let (solar_lng_tropical, lunar_lng_tropical, position_source) =
        match precise_tropical_positions(utc_dt) {
            Some((sun, moon)) if engine_human_design::ephemeris::moshier_observed() => {
                (sun, moon, PositionSource::SwissEphemerisMoshier)
            }
            Some((sun, moon)) => (sun, moon, PositionSource::SwissEphemerisFiles),
            None => (
                calculate_solar_position(jd),
                calculate_lunar_position(jd),
                PositionSource::MeanLongitude,
            ),
        };
    build_result(
        jd,
        solar_lng_tropical,
        lunar_lng_tropical,
        position_source,
        tz_offset_hours,
    )
}

fn build_result(
    jd: f64,
    solar_lng_tropical: f64,
    lunar_lng_tropical: f64,
    position_source: PositionSource,
    tz_offset_hours: f64,
) -> (PanchangaResult, bool) {
    let solar_lng = to_sidereal_longitude(solar_lng_tropical);
    let lunar_lng = to_sidereal_longitude(lunar_lng_tropical);

    let tithi_val = calculate_tithi(solar_lng, lunar_lng);
    let nakshatra_val = calculate_nakshatra(lunar_lng);
    let yoga_val = calculate_yoga(solar_lng, lunar_lng);
    let karana_val = calculate_karana(tithi_val);
    let karana_slot = karana_slot_from_tithi(tithi_val);
    let (karana_idx, karana_name) = karana_identity(karana_slot);
    let vara_val = calculate_vara(jd + tz_offset_hours / 24.0);

    let tithi_idx = (tithi_val.floor() as usize).min(29);
    let nakshatra_idx = (nakshatra_val.floor() as usize).min(26);
    let yoga_idx = (yoga_val.floor() as usize).min(26);
    let vara_idx = (vara_val as usize).min(6);

    let result = PanchangaResult {
        tithi_index: tithi_idx as u8,
        tithi_name: TITHI_NAMES[tithi_idx].to_string(),
        tithi_value: tithi_val,

        nakshatra_index: nakshatra_idx as u8,
        nakshatra_name: NAKSHATRA_NAMES[nakshatra_idx].to_string(),
        nakshatra_value: nakshatra_val,

        yoga_index: yoga_idx as u8,
        yoga_name: YOGA_NAMES[yoga_idx].to_string(),
        yoga_value: yoga_val,

        karana_index: karana_idx,
        karana_name: karana_name.to_string(),
        karana_value: karana_val,

        vara_index: vara_idx as u8,
        vara_name: VARA_NAMES[vara_idx].to_string(),
        vara_basis: Some(VaraBasis::LocalDate),

        solar_longitude: solar_lng,
        lunar_longitude: lunar_lng,
        julian_day: jd,
        position_source: Some(position_source),
        birth_time_assumed: false,
    };
    (result, position_source.is_fallback())
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
// Timezone resolution
// ---------------------------------------------------------------------------

/// A `birth_data.timezone` value: an IANA zone, resolved with its daylight
/// saving rules, or a fixed UTC offset.
#[derive(Debug, Clone, Copy)]
enum Zone {
    Iana(chrono_tz::Tz),
    Fixed(chrono::FixedOffset),
}

/// Parse a timezone string: any IANA name (`"Asia/Kolkata"`,
/// `"America/New_York"`), a `±HH:MM` or `±HH` offset, or one of the standard
/// time abbreviations the earlier lookup table accepted that are not IANA
/// names (`PST`, `CST`, `AEST`). Anything else is an error rather than UTC.
fn parse_zone(tz: &str) -> Result<Zone, EngineError> {
    let unknown = || {
        EngineError::ValidationError(format!(
            "unknown timezone {tz:?}: expected an IANA zone name such as \"Asia/Kolkata\" \
             or a UTC offset such as \"+05:30\""
        ))
    };
    let fixed = |hours: i32| chrono::FixedOffset::east_opt(hours * 3600).map(Zone::Fixed);

    if let Some(rest) = tz.strip_prefix('+').or_else(|| tz.strip_prefix('-')) {
        let (hours, minutes) = rest.split_once(':').unwrap_or((rest, "00"));
        let all_digits =
            |s: &str| (1..=2).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit());
        if !all_digits(hours) || minutes.len() != 2 || !all_digits(minutes) {
            return Err(unknown());
        }
        let (hours, minutes): (i32, i32) = (hours.parse().unwrap(), minutes.parse().unwrap());
        let sign = if tz.starts_with('-') { -1 } else { 1 };
        if hours > 14 || minutes >= 60 {
            return Err(unknown());
        }
        return chrono::FixedOffset::east_opt(sign * (hours * 3600 + minutes * 60))
            .map(Zone::Fixed)
            .ok_or_else(unknown);
    }

    match tz {
        "PST" => fixed(-8),
        "CST" => fixed(-6),
        "AEST" => fixed(10),
        _ => tz.parse::<chrono_tz::Tz>().ok().map(Zone::Iana),
    }
    .ok_or_else(unknown)
}

impl Zone {
    /// UTC offset in hours in force at `utc_dt`.
    fn offset_hours_at(&self, utc_dt: &DateTime<Utc>) -> f64 {
        let seconds = match self {
            Zone::Iana(tz) => tz.offset_from_utc_datetime(&utc_dt.naive_utc()).fix(),
            Zone::Fixed(offset) => *offset,
        }
        .local_minus_utc();
        seconds as f64 / 3600.0
    }

    /// The UTC instant of a local wall-clock time.
    fn local_to_utc(&self, local: &NaiveDateTime) -> DateTime<Utc> {
        match self {
            Zone::Iana(tz) => local_to_utc(tz, local),
            Zone::Fixed(offset) => local_to_utc(offset, local),
        }
    }
}

/// A wall time repeated when clocks go back resolves to the earlier instant;
/// one skipped when clocks go forward is read with the offset in force before
/// the change (as Python's `fold=0` does).
fn local_to_utc<Z: TimeZone>(zone: &Z, local: &NaiveDateTime) -> DateTime<Utc> {
    match zone.from_local_datetime(local) {
        LocalResult::Single(dt) | LocalResult::Ambiguous(dt, _) => dt.with_timezone(&Utc),
        LocalResult::None => {
            let before = zone
                .offset_from_utc_datetime(&(*local - Duration::days(1)))
                .fix();
            Utc.from_utc_datetime(&(*local - Duration::seconds(before.local_minus_utc().into())))
        }
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

        // Determine mode: "daily" uses current_time (today) for cosmic weather;
        // default uses birth_data.date for the natal birth panchanga.
        let use_daily = input.options.get("mode").and_then(|v| v.as_str()) == Some("daily");

        let (result, fallback_used) = if use_daily {
            // Daily mode: `current_time` is already the UTC instant. The
            // positions depend only on it; the birth timezone, when given,
            // only picks the local date for the vara (UTC otherwise).
            let tz_offset = match input.birth_data.as_ref() {
                Some(birth) => parse_zone(&birth.timezone)?.offset_hours_at(&input.current_time),
                None => 0.0,
            };
            compute_panchanga_at_instant(&input.current_time, tz_offset)
        } else {
            let birth = input.birth_data.as_ref().ok_or_else(|| {
                EngineError::CalculationError(
                    "birth_data is required for Panchanga calculations".into(),
                )
            })?;
            let time_str = birth.time.as_deref().unwrap_or("12:00");

            // Reject what Swiss Ephemeris cannot parse instead of serving a
            // mean-longitude result computed from a defaulted Julian Day.
            let local = parse_local_datetime(&birth.date, time_str).ok_or_else(|| {
                EngineError::ValidationError(
                    "birth_data.date must be YYYY-MM-DD and birth_data.time HH:MM or HH:MM:SS"
                        .into(),
                )
            })?;
            let zone = parse_zone(&birth.timezone)?;

            let utc_dt = zone.local_to_utc(&local);
            let mut computed = compute_panchanga_at_instant(&utc_dt, zone.offset_hours_at(&utc_dt));
            computed.0.birth_time_assumed = birth.time.is_none();
            computed
        };
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
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                // Moshier or mean-longitude; `result.position_source` says which.
                fallback_used,
                provider_id: None,
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
        let time = birth.and_then(|b| b.time.as_deref()).unwrap_or("12:00");
        let lat = birth.map(|b| b.latitude).unwrap_or(0.0);
        let lon = birth.map(|b| b.longitude).unwrap_or(0.0);

        let raw = format!("panchanga:{}:{}:{:.6}:{:.6}", date, time, lat, lon);
        let hash = Sha256::digest(raw.as_bytes());
        format!("panchanga:{:x}", hash)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
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
        assert!((0.0..360.0).contains(&solar), "solar = {solar}");
    }

    #[test]
    fn test_lunar_position_range() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        let lunar = calculate_lunar_position(jd);
        assert!((0.0..360.0).contains(&lunar), "lunar = {lunar}");
    }

    #[test]
    fn test_tithi_range() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        let solar = calculate_solar_position(jd);
        let lunar = calculate_lunar_position(jd);
        let tithi = calculate_tithi(solar, lunar);
        assert!((0.0..30.0).contains(&tithi), "tithi = {tithi}");
    }

    #[test]
    fn test_nakshatra_range() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        let lunar = calculate_lunar_position(jd);
        let nak = calculate_nakshatra(lunar);
        assert!((0.0..27.0).contains(&nak), "nakshatra = {nak}");
    }

    #[test]
    fn test_yoga_range() {
        let jd = calculate_julian_day("1991-08-13", "13:31", 5.5);
        let solar = calculate_solar_position(jd);
        let lunar = calculate_lunar_position(jd);
        let yoga = calculate_yoga(solar, lunar);
        assert!((0.0..27.0).contains(&yoga), "yoga = {yoga}");
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
    fn test_canonical_birth_nakshatra_is_uttara_phalguni() {
        // Canonical regression case used in engine hygiene triage.
        let p = compute_panchanga("1991-08-13", "13:31", 5.5);
        assert_eq!(p.nakshatra_name, "Uttara Phalguni");
        assert!((146.666667..160.0).contains(&p.lunar_longitude));
    }

    #[test]
    fn test_karana_regression_march_10_2026_day_sequence() {
        // External reference: Bengaluru day Panchang for 2026-03-10
        // shows Vishti until 12:40 PM, then Bava.
        let sunrise = compute_panchanga("2026-03-10", "06:30", 5.5);
        assert_eq!(sunrise.karana_name, "Vishti");

        let midday = compute_panchanga("2026-03-10", "12:21", 5.5);
        assert_eq!(midday.karana_name, "Vishti");

        let after_transition = compute_panchanga("2026-03-10", "12:41", 5.5);
        assert_eq!(after_transition.karana_name, "Bava");
    }

    #[test]
    fn test_birth_reference_1991_08_13_bangalore_matches_trusted_screenshot() {
        let p = compute_panchanga("1991-08-13", "13:31", 5.5);
        let nakshatra_pada = ((p.nakshatra_value.fract() * 4.0).floor() as u8) + 1;
        let moon_sign_index = (p.lunar_longitude / 30.0).floor() as u8;
        let sun_sign_index = (p.solar_longitude / 30.0).floor() as u8;

        assert_eq!(p.tithi_name, "Chaturthi (Shukla)");
        assert_eq!(p.nakshatra_name, "Uttara Phalguni");
        assert_eq!(nakshatra_pada, 4);
        assert_eq!(p.yoga_name, "Siddha");
        assert_eq!(p.karana_name, "Vishti");
        assert_eq!(p.vara_name, "Mangalavara (Tuesday)");
        assert_eq!(moon_sign_index, 5, "expected Kanya moon sign");
        assert_eq!(sun_sign_index, 3, "expected Karka sun sign");
    }

    #[test]
    fn test_ephemeris_positions_report_no_fallback() {
        let (_, fallback_used) = compute_panchanga_reporting_fallback("1991-08-13", "13:31", 5.5);
        assert!(!fallback_used);
    }

    #[test]
    fn test_mean_longitude_fallback_is_reported() {
        // "24:00" is rejected by the chrono parse that feeds Swiss Ephemeris, so
        // the mean-longitude approximation must be used and declared.
        let (p, fallback_used) = compute_panchanga_reporting_fallback("1991-08-13", "24:00", 5.5);
        assert!(fallback_used);
        assert!(!p.tithi_name.is_empty());
    }

    // -----------------------------------------------------------------------
    // #1191 provenance probes: which path produced tithi, nakshatra, yoga and
    // karana, and which inputs were assumed rather than supplied.
    // -----------------------------------------------------------------------

    fn result_of(output: &EngineOutput) -> PanchangaResult {
        serde_json::from_value(output.result.clone()).expect("bad PanchangaResult JSON")
    }

    /// Happy path: `.se1` files are reachable (SWISS_EPHEMERIS_PATH or
    /// data/ephemeris), so positions come from Swiss Ephemeris files.
    #[tokio::test]
    async fn probe_happy_path_reports_swiss_ephemeris_files() {
        let output = PanchangaEngine::new()
            .calculate(test_input())
            .await
            .expect("calculate failed");
        let pr = result_of(&output);
        assert_eq!(
            pr.position_source,
            Some(PositionSource::SwissEphemerisFiles)
        );
        assert!(!pr.birth_time_assumed);
        assert!(!output.metadata.fallback_used);
        assert_eq!(output.result["position_source"], "swiss_ephemeris_files");
        assert_eq!(output.result["birth_time_assumed"], false);
    }

    /// Fallback path: the mean-longitude approximation is labelled as such in
    /// the result, not only through the shared `fallback_used` flag.
    #[test]
    fn probe_mean_longitude_fallback_is_distinguishable() {
        // "24:00" is rejected by the chrono parse that feeds Swiss Ephemeris.
        // `calculate` rejects it as invalid input; the library function keeps
        // the mean-longitude path so it stays observable.
        let (p, fallback_used) = compute_panchanga_reporting_fallback("1991-08-13", "24:00", 5.5);
        assert!(fallback_used);
        assert_eq!(p.position_source, Some(PositionSource::MeanLongitude));
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["position_source"], "mean_longitude");
    }

    /// Boundary: a missing birth time is computed at local noon and the
    /// result says so instead of passing the noon chart off as the birth time.
    #[tokio::test]
    async fn probe_boundary_missing_birth_time_reported_as_assumed_noon() {
        let mut input = test_input();
        input.birth_data.as_mut().unwrap().time = None;
        let output = PanchangaEngine::new()
            .calculate(input)
            .await
            .expect("calculate failed");
        let pr = result_of(&output);
        assert!(pr.birth_time_assumed);
        assert_eq!(output.result["birth_time_assumed"], true);
        // Same positions as an explicit noon, so only the flag differs.
        let noon = compute_panchanga("1991-08-13", "12:00", 5.5);
        assert_eq!(pr.tithi_name, noon.tithi_name);
        assert_eq!(pr.julian_day, noon.julian_day);
        // Input defaulting is not a calculation fallback.
        assert_eq!(
            pr.position_source,
            Some(PositionSource::SwissEphemerisFiles)
        );
        assert!(!output.metadata.fallback_used);
    }

    /// Boundary: tithi wraps at new moon (Amavasya -> Pratipada) rather than
    /// going negative or reaching 30.
    #[test]
    fn probe_boundary_tithi_wraps_at_new_moon() {
        let before = calculate_tithi(0.1, 359.9);
        let after = calculate_tithi(359.9, 0.1);
        assert_eq!(TITHI_NAMES[(before.floor() as usize).min(29)], "Amavasya");
        assert_eq!(
            TITHI_NAMES[(after.floor() as usize).min(29)],
            "Pratipada (Shukla)"
        );
        assert!((0.0..30.0).contains(&before) && (0.0..30.0).contains(&after));
    }

    /// Daily mode reads the time from `current_time`, so nothing is assumed.
    #[tokio::test]
    async fn probe_daily_mode_never_assumes_birth_time() {
        let mut input = test_input();
        input.birth_data.as_mut().unwrap().time = None;
        input
            .options
            .insert("mode".to_string(), serde_json::json!("daily"));
        let output = PanchangaEngine::new().calculate(input).await.unwrap();
        assert!(!result_of(&output).birth_time_assumed);
    }

    /// Invalid input: an unparseable date or time is a validation error, not a
    /// degraded mean-longitude result computed from a defaulted Julian Day.
    #[tokio::test]
    async fn probe_invalid_input_rejected_with_validation_error() {
        let engine = PanchangaEngine::new();
        for (date, time) in [
            ("1991-13-45", Some("13:31")),
            ("not-a-date", Some("13:31")),
            ("1991-08-13", Some("24:00")),
            ("1991-08-13", Some("noon")),
            ("", None),
        ] {
            let mut input = test_input();
            let birth = input.birth_data.as_mut().unwrap();
            birth.date = date.to_string();
            birth.time = time.map(str::to_string);
            match engine.calculate(input).await {
                Err(EngineError::ValidationError(_)) => {}
                other => panic!("{date:?} {time:?}: expected ValidationError, got {other:?}"),
            }
        }
    }

    /// Results serialized before #1191 (no provenance fields) still validate.
    #[tokio::test]
    async fn probe_pre_provenance_result_still_deserializes() {
        let engine = PanchangaEngine::new();
        let mut output = engine.calculate(test_input()).await.unwrap();
        let obj = output.result.as_object_mut().unwrap();
        obj.remove("position_source");
        obj.remove("birth_time_assumed");
        let pr = result_of(&output);
        assert_eq!(pr.position_source, None);
        assert!(!pr.birth_time_assumed);
        assert!(engine.validate(&output).await.unwrap().valid);
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
        assert!(!output.metadata.fallback_used);
        assert!(output.metadata.provider_id.is_none());

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

    /// mode=daily should use current_time date and NOT require birth_data.
    /// Vara index must be in 0..=6.
    #[tokio::test]
    async fn test_calculate_daily_mode_uses_current_time() {
        let engine = PanchangaEngine::new();
        // Use a known Wednesday: 2026-05-13 (UTC).
        use chrono::TimeZone;
        let wednesday_2026_05_13 = Utc.with_ymd_and_hms(2026, 5, 13, 6, 0, 0).unwrap();

        let mut opts = HashMap::new();
        opts.insert("mode".to_string(), serde_json::json!("daily"));

        let input = EngineInput {
            birth_data: None, // no birth_data — daily mode must handle this
            current_time: wednesday_2026_05_13,
            location: None,
            precision: Precision::Standard,
            options: opts,
        };

        let output = engine
            .calculate(input)
            .await
            .expect("daily mode calculate failed");
        let pr: PanchangaResult =
            serde_json::from_value(output.result).expect("bad PanchangaResult JSON");

        // Vara index 0..=6 must be valid
        assert!(
            pr.vara_index <= 6,
            "vara_index out of range: {}",
            pr.vara_index
        );
    }

    /// mode=daily with birth_data present: should still use current_time date, not birth date.
    #[tokio::test]
    async fn test_daily_mode_ignores_birth_date_uses_today() {
        let engine = PanchangaEngine::new();

        use chrono::TimeZone;
        // Pick a day that's different from the birth date in test_input (1991-08-13 = Tuesday)
        let wednesday_2026_05_13 = Utc.with_ymd_and_hms(2026, 5, 13, 6, 0, 0).unwrap();

        let mut opts = HashMap::new();
        opts.insert("mode".to_string(), serde_json::json!("daily"));

        // Birth data present — in daily mode it should only contribute timezone, not the date.
        let birth = BirthData {
            name: None,
            date: "1991-08-13".to_string(), // a Tuesday
            time: Some("05:30".to_string()),
            timezone: "Asia/Kolkata".to_string(),
            latitude: 12.9716,
            longitude: 77.5946,
        };

        let input = EngineInput {
            birth_data: Some(birth),
            current_time: wednesday_2026_05_13,
            location: None,
            precision: Precision::Standard,
            options: opts,
        };

        let daily_output = engine
            .calculate(input.clone())
            .await
            .expect("daily mode failed");
        let daily_pr: PanchangaResult =
            serde_json::from_value(daily_output.result).expect("bad JSON");

        // Compare against the birth-mode output for the same input
        let mut birth_opts = HashMap::new();
        birth_opts.insert("user_id".to_string(), serde_json::json!("test"));
        let birth_input = EngineInput {
            birth_data: Some(BirthData {
                name: None,
                date: "1991-08-13".to_string(),
                time: Some("05:30".to_string()),
                timezone: "Asia/Kolkata".to_string(),
                latitude: 12.9716,
                longitude: 77.5946,
            }),
            current_time: wednesday_2026_05_13,
            location: None,
            precision: Precision::Standard,
            options: birth_opts,
        };
        let birth_output = engine
            .calculate(birth_input)
            .await
            .expect("birth mode failed");
        let birth_pr: PanchangaResult =
            serde_json::from_value(birth_output.result).expect("bad JSON");

        // Daily and birth-mode vara MUST differ (different dates → different vara)
        assert_ne!(
            daily_pr.vara_index, birth_pr.vara_index,
            "daily and birth vara should differ but both returned {} ({})",
            daily_pr.vara_index, daily_pr.vara_name
        );
    }

    fn daily_input(current_time: chrono::DateTime<Utc>, timezone: Option<&str>) -> EngineInput {
        let mut options = HashMap::new();
        options.insert("mode".to_string(), serde_json::json!("daily"));
        EngineInput {
            birth_data: timezone.map(|tz| BirthData {
                timezone: tz.to_string(),
                ..test_birth_data()
            }),
            current_time,
            location: None,
            precision: Precision::Standard,
            options,
        }
    }

    /// #1521: daily mode treats `current_time` as the UTC instant it is. The
    /// birth timezone must not shift it a second time.
    #[tokio::test]
    async fn test_daily_mode_positions_independent_of_birth_timezone() {
        let engine = PanchangaEngine::new();
        let instant = Utc.with_ymd_and_hms(2026, 5, 13, 10, 0, 0).unwrap();
        let reference = compute_panchanga_at_instant(&instant, 0.0).0;

        let without = result_of(&engine.calculate(daily_input(instant, None)).await.unwrap());
        for tz in ["Asia/Kolkata", "+05:30", "America/New_York"] {
            let with = result_of(
                &engine
                    .calculate(daily_input(instant, Some(tz)))
                    .await
                    .unwrap(),
            );
            assert_eq!(with.tithi_name, without.tithi_name, "{tz}");
            assert_eq!(with.nakshatra_name, without.nakshatra_name, "{tz}");
            assert_eq!(with.lunar_longitude, without.lunar_longitude, "{tz}");
            assert_eq!(with.julian_day, without.julian_day, "{tz}");
        }
        assert_eq!(without.lunar_longitude, reference.lunar_longitude);
        // 10:00Z, not 04:30Z: the Julian Day is that of the instant itself.
        assert!((without.julian_day - 2_461_173.916_666_7).abs() < 1e-6);
    }

    fn birth_input(date: &str, time: &str, timezone: &str) -> EngineInput {
        let mut input = test_input();
        input.birth_data = Some(BirthData {
            date: date.to_string(),
            time: Some(time.to_string()),
            timezone: timezone.to_string(),
            ..test_birth_data()
        });
        input
    }

    async fn birth_result(date: &str, time: &str, timezone: &str) -> PanchangaResult {
        let output = PanchangaEngine::new()
            .calculate(birth_input(date, time, timezone))
            .await
            .unwrap_or_else(|e| panic!("{date} {time} {timezone}: {e:?}"));
        result_of(&output)
    }

    fn jd_of(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> f64 {
        julian_day_from_utc(&Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap())
    }

    /// #1521: IANA zones resolve with daylight saving at the instant.
    /// Europe/London switches GMT -> BST at 01:00Z on 2026-03-29 and back at
    /// 01:00Z on 2026-10-25.
    #[tokio::test]
    async fn test_iana_zone_resolves_dst_on_both_sides_of_change() {
        // Spring forward: before (GMT, +0) and after (BST, +1).
        let before = birth_result("2026-03-29", "00:30", "Europe/London").await;
        assert_eq!(before.julian_day, jd_of(2026, 3, 29, 0, 30));
        let after = birth_result("2026-03-29", "02:30", "Europe/London").await;
        assert_eq!(after.julian_day, jd_of(2026, 3, 29, 1, 30));
        // 01:30 does not exist that night: read with the pre-change offset.
        let skipped = birth_result("2026-03-29", "01:30", "Europe/London").await;
        assert_eq!(skipped.julian_day, jd_of(2026, 3, 29, 1, 30));

        // Fall back: 01:30 happens twice; the earlier (BST) instant is used.
        let repeated = birth_result("2026-10-25", "01:30", "Europe/London").await;
        assert_eq!(repeated.julian_day, jd_of(2026, 10, 25, 0, 30));
        let after_fall = birth_result("2026-10-25", "02:30", "Europe/London").await;
        assert_eq!(after_fall.julian_day, jd_of(2026, 10, 25, 2, 30));

        // Same wall time, summer vs winter, in a negative-offset zone.
        let winter = birth_result("2026-01-15", "12:00", "America/New_York").await;
        assert_eq!(winter.julian_day, jd_of(2026, 1, 15, 17, 0));
        let summer = birth_result("2026-07-15", "12:00", "America/New_York").await;
        assert_eq!(summer.julian_day, jd_of(2026, 7, 15, 16, 0));
    }

    /// Zones outside the old 14-arm table resolve instead of becoming UTC,
    /// and fixed offsets keep working.
    #[tokio::test]
    async fn test_any_iana_zone_and_fixed_offsets_resolve() {
        let sao_paulo = birth_result("2000-02-14", "03:15", "America/Sao_Paulo").await;
        // Brazilian summer time (-02:00) was in force in February 2000.
        assert_eq!(sao_paulo.julian_day, jd_of(2000, 2, 14, 5, 15));

        let iana = birth_result("1991-08-13", "13:31", "Asia/Kolkata").await;
        for tz in ["+05:30", "+5:30", "Asia/Calcutta"] {
            assert_eq!(
                birth_result("1991-08-13", "13:31", tz).await.julian_day,
                iana.julian_day
            );
        }
        let negative = birth_result("2026-01-15", "12:00", "-03:00").await;
        assert_eq!(negative.julian_day, jd_of(2026, 1, 15, 15, 0));
        let whole_hours = birth_result("2026-01-15", "12:00", "+09").await;
        assert_eq!(whole_hours.julian_day, jd_of(2026, 1, 15, 3, 0));
        let legacy = birth_result("2026-07-15", "12:00", "PST").await;
        assert_eq!(legacy.julian_day, jd_of(2026, 7, 15, 20, 0));
    }

    #[tokio::test]
    async fn test_unknown_timezone_rejected_not_treated_as_utc() {
        let engine = PanchangaEngine::new();
        for tz in [
            "Mars/Olympus_Mons",
            "",
            "Kolkata",
            "+5x",
            "+05:75",
            "+15:00",
            "05:30",
        ] {
            match engine
                .calculate(birth_input("1991-08-13", "13:31", tz))
                .await
            {
                Err(EngineError::ValidationError(msg)) => {
                    assert!(msg.contains("unknown timezone"), "{tz:?}: {msg}")
                }
                other => panic!("{tz:?}: expected ValidationError, got {other:?}"),
            }
        }
    }

    /// #1521: the vara is the weekday of the local date, so it turns over at
    /// local midnight, not UTC midnight. Asia/Kolkata is UTC+05:30: 18:30Z
    /// on Wednesday 2026-05-13 is midnight going into Thursday there.
    #[tokio::test]
    async fn test_vara_turns_over_at_local_midnight_in_positive_offset_zone() {
        let engine = PanchangaEngine::new();
        let before = Utc.with_ymd_and_hms(2026, 5, 13, 18, 20, 0).unwrap(); // 23:50 IST
        let after = Utc.with_ymd_and_hms(2026, 5, 13, 18, 40, 0).unwrap(); // 00:10 IST

        let daily = |t| engine.calculate(daily_input(t, Some("Asia/Kolkata")));
        let wed = result_of(&daily(before).await.unwrap());
        let thu = result_of(&daily(after).await.unwrap());
        assert_eq!(wed.vara_name, "Budhavara (Wednesday)");
        assert_eq!(thu.vara_name, "Guruvara (Thursday)");
        assert_eq!(thu.vara_basis, Some(VaraBasis::LocalDate));

        // Without a timezone daily mode reads the UTC date: still Wednesday.
        let utc = result_of(&engine.calculate(daily_input(after, None)).await.unwrap());
        assert_eq!(utc.vara_name, "Budhavara (Wednesday)");

        // Birth mode: 02:00 Thursday in Kolkata is 20:30Z Wednesday.
        let birth = birth_result("2026-05-14", "02:00", "Asia/Kolkata").await;
        assert_eq!(birth.vara_name, "Guruvara (Thursday)");
        let output = engine
            .calculate(birth_input("2026-05-14", "02:00", "Asia/Kolkata"))
            .await
            .unwrap();
        assert_eq!(output.result["vara_basis"], "local_date");
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
