//! Panchang API — native facade backed by `engine-panchanga`.
//!
//! PR2 of 3 (#871 stack): the previous implementation called a non-existent
//! `/panchang` (and `/sunrise-sunset`) vendor endpoint that returned 403.
//! This module now computes the five-limb Panchanga directly via
//! `engine-panchanga::compute_panchanga` and maps the result into the
//! existing `Panchang` struct so `CachedVedicClient::get_panchang`,
//! `get_complete_panchang`, and `noesis-integration::fetch_panchang` keep
//! working with the same public contract.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::client::VedicApiClient;
use crate::error::{VedicApiError, VedicApiResult};
use crate::panchang::data::{
    DateInfo, DayBoundaries, Karana, KaranaName, KaranaType, Location, Nakshatra, NakshatraName,
    Paksha, Panchang, PlanetPosition as PanchangPlanetPosition, PlanetaryPositions, Tithi,
    TithiName, Vara, Yoga, YogaName,
};

/// Request parameters for Panchang API call
#[derive(Debug, Clone, Serialize)]
pub struct PanchangApiRequest {
    /// Date for panchang calculation (YYYY-MM-DD)
    pub date: String,
    /// Latitude of location
    pub latitude: f64,
    /// Longitude of location
    pub longitude: f64,
    /// Time for calculation (HH:MM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    /// Timezone offset in hours (e.g., 5.5 for IST)
    pub timezone: f64,
    /// Ayanamsa type (lahiri, raman, krishnamurti, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl PanchangApiRequest {
    /// Create a new Panchang API request
    pub fn new(date: NaiveDate, latitude: f64, longitude: f64, timezone: f64) -> Self {
        Self {
            date: date.format("%Y-%m-%d").to_string(),
            latitude,
            longitude,
            time: None,
            timezone,
            ayanamsa: Some("lahiri".to_string()),
        }
    }

    pub fn with_time(mut self, time: NaiveTime) -> Self {
        self.time = Some(time.format("%H:%M").to_string());
        self
    }

    pub fn with_ayanamsa(mut self, ayanamsa: &str) -> Self {
        self.ayanamsa = Some(ayanamsa.to_string());
        self
    }
}

/// Legacy API response shape (preserved for source-compat).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanchangApiResponse {
    pub tithi: TithiApiResponse,
    pub nakshatra: NakshatraApiResponse,
    pub yoga: YogaApiResponse,
    pub karana: KaranaApiResponse,
    pub vara: VaraApiResponse,
    #[serde(default)]
    pub sunrise: Option<String>,
    #[serde(default)]
    pub sunset: Option<String>,
    #[serde(default)]
    pub moonrise: Option<String>,
    #[serde(default)]
    pub moonset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TithiApiResponse {
    pub number: u8,
    pub name: String,
    pub paksha: String,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub deity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NakshatraApiResponse {
    pub number: u8,
    pub name: String,
    #[serde(default)]
    pub pada: Option<u8>,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub lord: Option<String>,
    #[serde(default)]
    pub deity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YogaApiResponse {
    pub number: u8,
    pub name: String,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub meaning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaranaApiResponse {
    pub number: u8,
    pub name: String,
    #[serde(default)]
    pub end_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaraApiResponse {
    pub number: u8,
    pub name: String,
    #[serde(default)]
    pub lord: Option<String>,
}

/// Request for sunrise/sunset calculation
#[derive(Debug, Clone, Serialize)]
pub struct SunriseSunsetRequest {
    pub date: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
}

/// Response from sunrise/sunset API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SunriseSunsetResponse {
    pub sunrise: String,
    pub sunset: String,
    #[serde(default)]
    pub dawn: Option<String>,
    #[serde(default)]
    pub dusk: Option<String>,
    #[serde(default)]
    pub solar_noon: Option<String>,
    #[serde(default)]
    pub day_duration: Option<String>,
}

// ---------------------------------------------------------------------------
// Native facade
// ---------------------------------------------------------------------------

fn map_vara_index(idx: u8) -> Vara {
    match idx {
        0 => Vara::Sunday,
        1 => Vara::Monday,
        2 => Vara::Tuesday,
        3 => Vara::Wednesday,
        4 => Vara::Thursday,
        5 => Vara::Friday,
        _ => Vara::Saturday,
    }
}

fn paksha_from_tithi_index(tithi_idx: u8) -> Paksha {
    // tithi_idx 0..=14 (1..=15) = Shukla, 15..=29 (16..=30) = Krishna.
    if tithi_idx < 15 {
        Paksha::Shukla
    } else {
        Paksha::Krishna
    }
}

fn karana_type_from_index(idx: u8) -> KaranaType {
    // Engine returns karana_index in 0..=10. Slot 7 = Vishti (Bhadra),
    // 8..=10 + 0 in special positions = Fixed (Shakuni, Chatushpada, Naga,
    // Kimstughna). The remaining karanas are movable.
    match idx {
        7 => KaranaType::Vishti, // Vishti / Bhadra
        8 | 9 | 10 => KaranaType::Fixed,
        _ => KaranaType::Movable,
    }
}

/// Compute the full `Panchang` natively. Synchronous — `engine-panchanga`
/// has its own internal Swiss-Ephemeris-backed `precise_tropical_positions`
/// fallback wrapped through `EphemerisCalculator`, but the high-level
/// `compute_panchanga` entry point is sync-only.
///
/// `lat`/`lng`/`tz_offset_hours` drive both the Panchanga (via tz_offset)
/// and the `DayBoundaries` block (via approximate-sunrise math). The HH:MM
/// format is preserved so downstream `calculate_muhurtas`,
/// `calculate_hora_timings`, `calculate_choghadiya` keep working.
pub fn compute_panchang_native(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    lat: f64,
    lng: f64,
    tz_offset_hours: f64,
) -> VedicApiResult<Panchang> {
    let _date =
        NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| VedicApiError::InvalidInput {
            field: "date".to_string(),
            message: format!("invalid date {}-{:02}-{:02}", year, month, day),
        })?;
    let _time = NaiveTime::from_hms_opt(hour, minute, second).ok_or_else(|| {
        VedicApiError::InvalidInput {
            field: "time".to_string(),
            message: format!("invalid time {:02}:{:02}:{:02}", hour, minute, second),
        }
    })?;

    let date_str = format!("{:04}-{:02}-{:02}", year, month, day);
    let time_str = format!("{:02}:{:02}", hour, minute);

    let result = engine_panchanga::compute_panchanga(&date_str, &time_str, tz_offset_hours);

    // ---- Tithi ----
    let tithi_number = result.tithi_index + 1;
    let tithi = Tithi {
        number: tithi_number,
        name_tithi: TithiName::from_number(tithi_number as u32),
        start_time: String::new(),
        end_time: String::new(),
        is_complete: true,
    };

    // ---- Nakshatra ----
    let nakshatra_number = result.nakshatra_index + 1;
    let pada = ((result.nakshatra_value.fract() * 4.0).floor() as u8) + 1;
    let nakshatra = Nakshatra {
        number: nakshatra_number,
        name_nakshatra: NakshatraName::from_number(nakshatra_number as u32),
        pada: pada.min(4).max(1),
        start_time: String::new(),
        end_time: String::new(),
        longitude: result.lunar_longitude,
    };

    // ---- Yoga ----
    let yoga_number = result.yoga_index + 1;
    let yoga = Yoga {
        number: yoga_number,
        name_yoga: YogaName::from_number(yoga_number as u32),
        start_time: String::new(),
        end_time: String::new(),
    };

    // ---- Karana ----
    let karana_number = result.karana_index + 1;
    let karana = Karana {
        name_karana: KaranaName::from_number(karana_number as u32),
        karana_type: karana_type_from_index(result.karana_index),
        start_time: String::new(),
        end_time: String::new(),
    };

    // ---- Vara ----
    let vara = map_vara_index(result.vara_index);

    // ---- Paksha ----
    let paksha = paksha_from_tithi_index(result.tithi_index);

    // ---- Day boundaries ----
    let sunrise = crate::resilience::approximate_sunrise(lat, result.julian_day);
    let sunset = crate::resilience::approximate_sunset(lat, result.julian_day);
    let next_sunrise = crate::resilience::approximate_sunrise(lat, result.julian_day + 1.0);
    let day_boundaries = DayBoundaries {
        sunrise,
        sunset,
        next_sunrise,
        day_duration: String::new(),
        night_duration: String::new(),
    };

    // ---- Planetary positions (minimal — Sun + Moon from engine result) ----
    let planets = PlanetaryPositions {
        sun: PanchangPlanetPosition {
            name: "Sun".to_string(),
            longitude: result.solar_longitude,
            latitude: 0.0,
            speed: 0.0,
            sign: String::new(),
            nakshatra: String::new(),
            pada: 0,
            is_retrograde: false,
        },
        moon: PanchangPlanetPosition {
            name: "Moon".to_string(),
            longitude: result.lunar_longitude,
            latitude: 0.0,
            speed: 0.0,
            sign: String::new(),
            nakshatra: nakshatra.name_nakshatra.as_str().to_string(),
            pada,
            is_retrograde: false,
        },
        mars: None,
        mercury: None,
        jupiter: None,
        venus: None,
        saturn: None,
        rahu: None,
        ketu: None,
    };

    Ok(Panchang {
        date: DateInfo {
            year,
            month,
            day,
            day_of_week: vara.number(),
            julian_day: result.julian_day,
            hindu_date: None,
        },
        location: Location {
            latitude: lat,
            longitude: lng,
            timezone: tz_offset_hours,
            name: None,
        },
        tithi,
        nakshatra,
        yoga,
        karana,
        vara,
        paksha,
        planets,
        day_boundaries,
        ayanamsa: lahiri_ayanamsa_from_julian_day(result.julian_day),
    })
}

/// Lahiri (Chitrapaksha) ayanamsa as a function of Julian Day.
///
/// PR5: delegates to the canonical SwissEph-grounded helper at
/// `engine_human_design::ephemeris::lahiri_ayanamsa`. Previously a J2000
/// linear polynomial (`23.853 + (jd-J2000) * 50.2876/3600/365.25`) which
/// drifted ~75″ off SwissEph truth at modern births. Public signature
/// preserved so all existing call sites in `noesis-vedic-api` continue
/// to work unchanged.
pub(crate) fn lahiri_ayanamsa_from_julian_day(jd: f64) -> f64 {
    engine_human_design::ephemeris::lahiri_ayanamsa(jd)
}

impl VedicApiClient {
    /// Get raw Panchang via native `engine-panchanga`.
    ///
    /// PR2: no HTTP call. Mirrors the legacy `PanchangApiResponse` shape
    /// from the natively-computed `Panchang` so existing JSON consumers
    /// remain source-compatible.
    pub async fn get_panchang_raw(
        &self,
        request: &PanchangApiRequest,
    ) -> VedicApiResult<PanchangApiResponse> {
        let date = NaiveDate::parse_from_str(&request.date, "%Y-%m-%d").map_err(|e| {
            VedicApiError::ParseError(format!("invalid date '{}': {}", request.date, e))
        })?;
        let time = match request.time.as_deref() {
            Some(s) => NaiveTime::parse_from_str(s, "%H:%M")
                .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M:%S"))
                .unwrap_or_else(|_| NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            None => NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        };

        use chrono::{Datelike, Timelike};
        let panchang = compute_panchang_native(
            date.year(),
            date.month(),
            date.day(),
            time.hour(),
            time.minute(),
            time.second(),
            request.latitude,
            request.longitude,
            request.timezone,
        )?;

        Ok(PanchangApiResponse {
            tithi: TithiApiResponse {
                number: panchang.tithi.number,
                name: panchang.tithi.name().to_string(),
                paksha: panchang.paksha.as_str().to_string(),
                end_time: Some(panchang.tithi.end_time.clone()),
                deity: None,
            },
            nakshatra: NakshatraApiResponse {
                number: panchang.nakshatra.number,
                name: panchang.nakshatra.name().to_string(),
                pada: Some(panchang.nakshatra.pada),
                end_time: Some(panchang.nakshatra.end_time.clone()),
                lord: Some(panchang.nakshatra.ruling_planet().to_string()),
                deity: Some(panchang.nakshatra.deity().to_string()),
            },
            yoga: YogaApiResponse {
                number: panchang.yoga.number,
                name: panchang.yoga.name().to_string(),
                end_time: Some(panchang.yoga.end_time.clone()),
                meaning: None,
            },
            karana: KaranaApiResponse {
                number: panchang.karana.name_karana.number(),
                name: panchang.karana.name().to_string(),
                end_time: Some(panchang.karana.end_time.clone()),
            },
            vara: VaraApiResponse {
                number: panchang.vara.number(),
                name: panchang.vara.as_str().to_string(),
                lord: Some(panchang.vara.ruling_planet().to_string()),
            },
            sunrise: Some(panchang.day_boundaries.sunrise.clone()),
            sunset: Some(panchang.day_boundaries.sunset.clone()),
            moonrise: None,
            moonset: None,
        })
    }

    /// Get sunrise and sunset times (native — no HTTP).
    pub async fn get_sunrise_sunset(
        &self,
        request: &SunriseSunsetRequest,
    ) -> VedicApiResult<SunriseSunsetResponse> {
        let date = NaiveDate::parse_from_str(&request.date, "%Y-%m-%d").map_err(|e| {
            VedicApiError::ParseError(format!("invalid date '{}': {}", request.date, e))
        })?;
        let _ = date;
        let jdn = engine_panchanga::calculate_julian_day(&request.date, "12:00", request.timezone);
        let sunrise = crate::resilience::approximate_sunrise(request.latitude, jdn);
        let sunset = crate::resilience::approximate_sunset(request.latitude, jdn);
        Ok(SunriseSunsetResponse {
            sunrise,
            sunset,
            dawn: None,
            dusk: None,
            solar_noon: Some("12:00".to_string()),
            day_duration: None,
        })
    }

    /// Get Panchang for a specific date and location
    pub async fn get_panchang_for_date(
        &self,
        date: NaiveDate,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<PanchangApiResponse> {
        let request = PanchangApiRequest::new(date, latitude, longitude, timezone);
        self.get_panchang_raw(&request).await
    }

    /// Get Panchang for a specific datetime and location
    pub async fn get_panchang_for_datetime(
        &self,
        datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<PanchangApiResponse> {
        let request = PanchangApiRequest::new(datetime.date(), latitude, longitude, timezone)
            .with_time(datetime.time());
        self.get_panchang_raw(&request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panchang_request_creation() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let request = PanchangApiRequest::new(date, 12.97, 77.59, 5.5);

        assert_eq!(request.date, "2024-01-15");
        assert_eq!(request.latitude, 12.97);
        assert_eq!(request.longitude, 77.59);
        assert_eq!(request.timezone, 5.5);
    }

    #[test]
    fn test_panchang_request_with_time() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let request = PanchangApiRequest::new(date, 12.97, 77.59, 5.5).with_time(time);

        assert_eq!(request.time, Some("10:30".to_string()));
    }

    #[test]
    fn test_compute_panchang_native_bangalore_2024() {
        let p = compute_panchang_native(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .expect("native panchang must succeed");
        // Tithi number must be 1..=30
        assert!(p.tithi.number >= 1 && p.tithi.number <= 30);
        // Nakshatra number must be 1..=27
        assert!(p.nakshatra.number >= 1 && p.nakshatra.number <= 27);
        // Pada must be 1..=4
        assert!(p.nakshatra.pada >= 1 && p.nakshatra.pada <= 4);
        // Sunrise/sunset must be non-empty HH:MM strings.
        assert_eq!(p.day_boundaries.sunrise.len(), 5);
        assert_eq!(p.day_boundaries.sunset.len(), 5);
        // ayanamsa populated
        assert!(p.ayanamsa > 0.0);
    }
}
