//! Transit API — native facade backed by `engine-transits`.
//!
//! PR2 of 3 (#871 stack): the previous implementation POSTed to a
//! non-existent vendor endpoint (`/transits`) that returned 403. This module
//! now computes transit positions directly via the in-tree
//! `engine-transits` crate (Swiss Ephemeris, Lahiri sidereal). All
//! Swiss-Ephemeris-touching code is wrapped in `tokio::task::spawn_blocking`
//! because libswisseph-sys uses a global mutex.
//!
//! Public types (`TransitRequest`, `TransitApiResponse`, …) are unchanged so
//! existing callers and downstream consumers keep compiling.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::client::VedicApiClient;
use crate::error::{VedicApiError, VedicApiResult};

/// Request for transit calculation
#[derive(Debug, Clone, Serialize)]
pub struct TransitRequest {
    /// Birth date
    pub birth_date: String,
    /// Birth time
    pub birth_time: String,
    /// Birth latitude
    pub latitude: f64,
    /// Birth longitude
    pub longitude: f64,
    /// Timezone
    pub timezone: f64,
    /// Date to calculate transits for
    pub transit_date: String,
    /// Ayanamsa
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl TransitRequest {
    pub fn new(
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
        transit_date: NaiveDate,
    ) -> Self {
        Self {
            birth_date: birth_datetime.date().format("%Y-%m-%d").to_string(),
            birth_time: birth_datetime.time().format("%H:%M:%S").to_string(),
            latitude,
            longitude,
            timezone,
            transit_date: transit_date.format("%Y-%m-%d").to_string(),
            ayanamsa: Some("lahiri".to_string()),
        }
    }
}

/// API response for transits (shape preserved from previous wire contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitApiResponse {
    pub transits: Vec<TransitPlanetResponse>,
    #[serde(default)]
    pub sade_sati: Option<SadeSatiResponse>,
    #[serde(default)]
    pub jupiter_transit: Option<JupiterTransitResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitPlanetResponse {
    pub planet: String,
    pub sign: String,
    pub degree: f64,
    #[serde(default)]
    pub is_retrograde: Option<bool>,
    #[serde(default)]
    pub natal_aspects: Option<Vec<NatalAspectResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatalAspectResponse {
    pub natal_planet: String,
    pub aspect_type: String,
    pub orb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SadeSatiResponse {
    pub is_active: bool,
    #[serde(default)]
    pub phase: Option<String>,
    pub saturn_sign: String,
    pub moon_sign: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterTransitResponse {
    pub sign: String,
    pub from_ascendant: u8,
    pub from_moon: u8,
    #[serde(default)]
    pub quality: Option<String>,
}

// ---------------------------------------------------------------------------
// Native facade — pure-native via engine-transits
// ---------------------------------------------------------------------------

/// Parse "YYYY-MM-DD" + "HH:MM[:SS]" with a numeric UTC offset into a UTC
/// `DateTime` suitable for Swiss-Ephemeris-backed engines.
fn parse_local_to_utc(
    date_str: &str,
    time_str: &str,
    tz_offset_hours: f64,
) -> VedicApiResult<chrono::DateTime<Utc>> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
        VedicApiError::ParseError(format!("invalid date '{}': {}", date_str, e))
    })?;
    let time = NaiveTime::parse_from_str(time_str, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(time_str, "%H:%M"))
        .map_err(|e| VedicApiError::ParseError(format!("invalid time '{}': {}", time_str, e)))?;

    let naive = NaiveDateTime::new(date, time);
    // Convert local civil time to UTC: utc = local - tz_offset
    let offset_seconds = (tz_offset_hours * 3600.0) as i64;
    let utc = Utc
        .from_utc_datetime(&naive)
        .checked_sub_signed(chrono::Duration::seconds(offset_seconds))
        .ok_or_else(|| {
            VedicApiError::ParseError("datetime out of range".to_string())
        })?;
    Ok(utc)
}

/// Compute all 12 transit-planet positions for `transit_dt` (UTC) on a
/// blocking thread (Swiss Ephemeris uses a global C-FFI mutex; calling on
/// the async reactor would risk deadlock).
async fn compute_positions(
    transit_dt: chrono::DateTime<Utc>,
) -> VedicApiResult<Vec<engine_transits::models::PlanetaryPosition>> {
    tokio::task::spawn_blocking(move || {
        let calculator = engine_human_design::EphemerisCalculator::new("");
        engine_transits::ephemeris::calculate_all_positions(&calculator, &transit_dt)
    })
    .await
    .map_err(|e| VedicApiError::ServiceUnavailable(format!("ephemeris task join failed: {}", e)))?
    .map_err(|e| VedicApiError::ServiceUnavailable(format!("ephemeris calculation failed: {}", e)))
}

/// Map a native engine `PlanetaryPosition` into the API-shaped `TransitPlanetResponse`.
fn position_to_response(
    pos: &engine_transits::models::PlanetaryPosition,
) -> TransitPlanetResponse {
    TransitPlanetResponse {
        planet: pos.planet.name().to_string(),
        sign: pos.sign.name().to_string(),
        degree: pos.degree_in_sign,
        is_retrograde: Some(pos.is_retrograde),
        natal_aspects: None,
    }
}

/// Compute Sade Sati from a Moon position (natal) and a Saturn position
/// (transit), comparing zodiac signs. Mirrors the local
/// `transits::sade_sati::check_sade_sati` semantics but operates on
/// engine-native `ZodiacSign` enums.
fn sade_sati_from_positions(
    moon: &engine_transits::models::PlanetaryPosition,
    saturn: &engine_transits::models::PlanetaryPosition,
) -> SadeSatiResponse {
    let moon_idx = moon.sign.index() as i16;
    let saturn_idx = saturn.sign.index() as i16;
    // Position 1..=12 of Saturn relative to Moon
    let position = ((saturn_idx - moon_idx).rem_euclid(12) + 1) as u8;
    let (is_active, phase) = match position {
        12 => (true, Some("Rising".to_string())),
        1 => (true, Some("Peak".to_string())),
        2 => (true, Some("Setting".to_string())),
        _ => (false, None),
    };
    SadeSatiResponse {
        is_active,
        phase,
        saturn_sign: saturn.sign.name().to_string(),
        moon_sign: moon.sign.name().to_string(),
    }
}

/// Compute Jupiter-transit summary relative to a natal Moon sign. Ascendant
/// is approximated as Moon sign when no chart is available (the previous
/// vendor response carried the same fields populated similarly).
fn jupiter_transit_from_positions(
    moon: &engine_transits::models::PlanetaryPosition,
    jupiter: &engine_transits::models::PlanetaryPosition,
) -> JupiterTransitResponse {
    let moon_idx = moon.sign.index() as i16;
    let jup_idx = jupiter.sign.index() as i16;
    let from_moon = ((jup_idx - moon_idx).rem_euclid(12) + 1) as u8;
    let quality = match from_moon {
        2 | 5 | 7 | 9 | 11 => Some("Favorable".to_string()),
        3 | 6 | 10 | 12 => Some("Challenging".to_string()),
        _ => Some("Neutral".to_string()),
    };
    JupiterTransitResponse {
        sign: jupiter.sign.name().to_string(),
        from_ascendant: from_moon, // approximated from Moon sign
        from_moon,
        quality,
    }
}

impl VedicApiClient {
    /// Get transit analysis via native `engine-transits`.
    ///
    /// PR2: no longer makes an HTTP call. Computes natal Moon (for Sade Sati
    /// reference) and current sidereal positions of all 12 transit planets
    /// via Swiss Ephemeris, then assembles a `TransitApiResponse` matching
    /// the prior wire contract.
    pub async fn get_transits(
        &self,
        request: &TransitRequest,
    ) -> VedicApiResult<TransitApiResponse> {
        // 1) Natal datetime (for Moon sign / Sade Sati reference).
        let natal_dt = parse_local_to_utc(
            &request.birth_date,
            &request.birth_time,
            request.timezone,
        )?;

        // 2) Transit datetime — date at noon UTC is a stable reference for
        //    "today's transits". The vendor endpoint had no time component.
        let transit_date = NaiveDate::parse_from_str(&request.transit_date, "%Y-%m-%d")
            .map_err(|e| {
                VedicApiError::ParseError(format!(
                    "invalid transit_date '{}': {}",
                    request.transit_date, e
                ))
            })?;
        let transit_naive =
            NaiveDateTime::new(transit_date, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        let transit_dt = Utc.from_utc_datetime(&transit_naive);

        // 3) Native ephemeris call (blocking).
        let natal_positions = compute_positions(natal_dt).await?;
        let transit_positions = compute_positions(transit_dt).await?;

        // 4) Locate natal Moon and transit Saturn / Jupiter.
        let natal_moon = natal_positions
            .iter()
            .find(|p| p.planet == engine_transits::models::TransitPlanet::Moon)
            .cloned();
        let transit_saturn = transit_positions
            .iter()
            .find(|p| p.planet == engine_transits::models::TransitPlanet::Saturn)
            .cloned();
        let transit_jupiter = transit_positions
            .iter()
            .find(|p| p.planet == engine_transits::models::TransitPlanet::Jupiter)
            .cloned();

        // 5) Assemble response.
        let transits = transit_positions
            .iter()
            .map(position_to_response)
            .collect();

        let sade_sati = match (natal_moon.as_ref(), transit_saturn.as_ref()) {
            (Some(m), Some(s)) => Some(sade_sati_from_positions(m, s)),
            _ => None,
        };
        let jupiter_transit = match (natal_moon.as_ref(), transit_jupiter.as_ref()) {
            (Some(m), Some(j)) => Some(jupiter_transit_from_positions(m, j)),
            _ => None,
        };

        Ok(TransitApiResponse {
            transits,
            sade_sati,
            jupiter_transit,
        })
    }

    /// Get current transits for a birth chart (native facade).
    pub async fn get_current_transits(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<TransitApiResponse> {
        let today = chrono::Utc::now().date_naive();
        let request = TransitRequest::new(birth_datetime, latitude, longitude, timezone, today);
        self.get_transits(&request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    fn test_client() -> VedicApiClient {
        VedicApiClient::new(crate::mocks::mock_config("http://localhost:0"))
    }

    #[test]
    fn test_transit_request() {
        let birth = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let transit = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let request = TransitRequest::new(birth, 12.97, 77.59, 5.5, transit);

        assert_eq!(request.birth_date, "1990-06-15");
        assert_eq!(request.transit_date, "2024-01-15");
    }

    #[test]
    fn test_parse_local_to_utc_ist() {
        // 1991-08-13 13:31:00 IST (+5:30) → 08:01:00 UTC
        let utc = parse_local_to_utc("1991-08-13", "13:31:00", 5.5).expect("parse");
        assert_eq!(utc.to_rfc3339(), "1991-08-13T08:01:00+00:00");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_transits_bangalore_1991() {
        let client = test_client();
        let birth = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1991, 8, 13).unwrap(),
            NaiveTime::from_hms_opt(13, 31, 0).unwrap(),
        );
        let transit = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let req = TransitRequest::new(birth, 12.9716, 77.5946, 5.5, transit);

        let resp = client.get_transits(&req).await.expect("native transits");
        // 12 transit planets (Sun..Pluto + Rahu + Ketu).
        assert_eq!(resp.transits.len(), 12);
        // Sade sati and jupiter blocks should be populated when natal Moon is known.
        assert!(resp.sade_sati.is_some());
        assert!(resp.jupiter_transit.is_some());

        // Sun is never retrograde — sanity check on at least one entry.
        let sun = resp
            .transits
            .iter()
            .find(|p| p.planet == "Sun")
            .expect("Sun present");
        assert_eq!(sun.is_retrograde, Some(false));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_current_transits_returns_full_set() {
        let client = test_client();
        let birth = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1991, 8, 13).unwrap(),
            NaiveTime::from_hms_opt(13, 31, 0).unwrap(),
        );
        let resp = client
            .get_current_transits(birth, 12.9716, 77.5946, 5.5)
            .await
            .expect("native current transits");
        assert_eq!(resp.transits.len(), 12);
    }
}
