//! Native birth-chart construction.
//!
//! PR3 needs a typed [`crate::birth_chart::types::BirthChart`] (the
//! enrichment-layer shape used by yogas / shadbala / ashtakavarga) built
//! from raw birth inputs **without** touching the network. The vendor
//! `/planets` POST is dead, and chart_mapping only handles the JSON
//! envelope. This module fills that gap by:
//!
//! 1. Calling Swiss Ephemeris through `engine_transits::ephemeris` on a
//!    `spawn_blocking` thread (libswisseph uses a global mutex).
//! 2. Deriving the sidereal ascendant via the Meeus algorithm (Lahiri
//!    ayanamsa to match the rest of the crate).
//! 3. Filling whole-sign houses, dignities, retrograde/combust flags and
//!    nakshatra info so downstream analytical modules can run unchanged.
//!
//! The output shape (`birth_chart::types::BirthChart`) is preserved exactly
//! so existing detectors (`detect_raj_yogas`, `calculate_full_shadbala`,
//! etc.) keep their signatures.

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};

use crate::birth_chart::types::{
    calculate_dignity, BirthChart, HouseCusp, Planet, PlanetPosition, ZodiacSign,
};
use crate::error::{VedicApiError, VedicApiResult};

/// Lahiri (Chitrapaksha) ayanamsa in degrees for a Julian Day.
///
/// Matches the formula used in `noesis-api` so downstream sidereal values
/// stay aligned across crates.
fn lahiri_ayanamsa(jd: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    23.85 + t * 1.3968
}

/// Julian Day (UT) from a UTC chrono DateTime.
fn jd_from_utc(dt: &DateTime<Utc>) -> f64 {
    2451545.0 + (dt.timestamp() - 946_728_000) as f64 / 86_400.0
}

/// Tropical ascendant in degrees [0,360) for a Julian Day and geographic
/// location. Meeus algorithm — agrees with noesis-api's reference impl.
fn tropical_ascendant(jd: f64, lat: f64, lng: f64) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    let gmst = (280.460_618_37 + 360.985_647_366_29 * (jd - 2451545.0) + 0.000_387_933 * t * t
        - t * t * t / 38_710_000.0)
        .rem_euclid(360.0);
    let lst = (gmst + lng).rem_euclid(360.0);
    let ramc = lst.to_radians();
    let eps = (23.439_291_111 - 0.013_004_167 * t).to_radians();
    let lat_r = lat.to_radians();
    let num = ramc.cos();
    let den = -(ramc.sin() * eps.cos() + lat_r.tan() * eps.sin());
    f64::atan2(num, den).to_degrees().rem_euclid(360.0)
}

/// Map an `engine_transits::models::TransitPlanet` to our local
/// `birth_chart::types::Planet`. Returns `None` for outer planets the
/// classical Vedic system doesn't use (Uranus/Neptune/Pluto).
fn classical_planet_for(p: engine_transits::models::TransitPlanet) -> Option<Planet> {
    use engine_transits::models::TransitPlanet as T;
    match p {
        T::Sun => Some(Planet::Sun),
        T::Moon => Some(Planet::Moon),
        T::Mars => Some(Planet::Mars),
        T::Mercury => Some(Planet::Mercury),
        T::Jupiter => Some(Planet::Jupiter),
        T::Venus => Some(Planet::Venus),
        T::Saturn => Some(Planet::Saturn),
        T::Rahu => Some(Planet::Rahu),
        T::Ketu => Some(Planet::Ketu),
        _ => None,
    }
}

/// Compute whole-sign house (1..=12) for a planet sign given the lagna sign.
fn whole_sign_house(planet_sign: ZodiacSign, lagna_sign: ZodiacSign) -> u8 {
    let diff = (planet_sign.number() as i16 - lagna_sign.number() as i16).rem_euclid(12);
    (diff + 1) as u8
}

/// Combustion thresholds (per-planet orbs from BPHS).
fn combust_threshold(planet: Planet) -> f64 {
    match planet {
        Planet::Moon => 12.0,
        Planet::Mars => 17.0,
        Planet::Mercury => 14.0,
        Planet::Jupiter => 11.0,
        Planet::Venus => 10.0,
        Planet::Saturn => 15.0,
        _ => 0.0,
    }
}

fn angular_distance(a: f64, b: f64) -> f64 {
    let diff = (a - b).abs() % 360.0;
    if diff > 180.0 {
        360.0 - diff
    } else {
        diff
    }
}

/// Pada (1..=4) from a sidereal longitude.
fn pada_from_longitude(longitude: f64) -> u8 {
    let nakshatra_span = 360.0_f64 / 27.0;
    let pada_span = nakshatra_span / 4.0;
    let pos = longitude.rem_euclid(nakshatra_span);
    (((pos / pada_span).floor() as i32) + 1).clamp(1, 4) as u8
}

fn nakshatra_name(longitude: f64) -> String {
    engine_vimshottari::get_nakshatra_from_longitude(longitude)
        .name
        .clone()
}

fn local_to_utc(
    date: NaiveDate,
    time: NaiveTime,
    tz_offset_hours: f64,
) -> VedicApiResult<DateTime<Utc>> {
    let naive = NaiveDateTime::new(date, time);
    let offset_seconds = (tz_offset_hours * 3600.0) as i64;
    Utc.from_utc_datetime(&naive)
        .checked_sub_signed(chrono::Duration::seconds(offset_seconds))
        .ok_or_else(|| VedicApiError::ParseError("datetime out of range".to_string()))
}

/// Build a native [`BirthChart`] from raw birth inputs. All Swiss Ephemeris
/// calls run on a `spawn_blocking` thread so this is safe to invoke from any
/// async context.
///
/// `tz_offset_hours` is the local civil clock offset (e.g. `5.5` for IST).
pub async fn build_native_chart(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    lat: f64,
    lng: f64,
    tz_offset_hours: f64,
) -> VedicApiResult<BirthChart> {
    let date =
        NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| VedicApiError::InvalidInput {
            field: "birth_date".to_string(),
            message: format!("invalid date {}-{:02}-{:02}", year, month, day),
        })?;
    let time = NaiveTime::from_hms_opt(hour, minute, second).ok_or_else(|| {
        VedicApiError::InvalidInput {
            field: "birth_time".to_string(),
            message: format!("invalid time {:02}:{:02}:{:02}", hour, minute, second),
        }
    })?;
    let birth_utc = local_to_utc(date, time, tz_offset_hours)?;

    // Swiss Ephemeris is C-FFI under a global mutex — run on a blocking thread.
    let positions: Vec<engine_transits::models::PlanetaryPosition> =
        tokio::task::spawn_blocking(move || {
            let calc = engine_human_design::EphemerisCalculator::new("");
            engine_transits::ephemeris::calculate_all_positions(&calc, &birth_utc)
        })
        .await
        .map_err(|e| {
            VedicApiError::ServiceUnavailable(format!("ephemeris task join failed: {}", e))
        })?
        .map_err(|e| {
            VedicApiError::ServiceUnavailable(format!("ephemeris calculation failed: {}", e))
        })?;

    // Ascendant: tropical via Meeus, then convert to sidereal with Lahiri.
    let jd = jd_from_utc(&birth_utc);
    let trop_asc = tropical_ascendant(jd, lat, lng);
    let ayanamsa = lahiri_ayanamsa(jd);
    let sid_asc = (trop_asc - ayanamsa).rem_euclid(360.0);
    let ascendant_sign = ZodiacSign::from_degree(sid_asc);
    let ascendant_degree = sid_asc % 30.0;

    // Locate the Sun first so we can flag combust planets.
    let sun_long = positions
        .iter()
        .find(|p| matches!(p.planet, engine_transits::models::TransitPlanet::Sun))
        .map(|p| p.longitude)
        .unwrap_or(0.0);
    let sun_sign = ZodiacSign::from_degree(sun_long);

    let mut moon_long: f64 = 0.0;

    let mut planet_positions: Vec<PlanetPosition> = Vec::with_capacity(9);
    for pos in &positions {
        let Some(planet) = classical_planet_for(pos.planet) else {
            continue;
        };
        if matches!(planet, Planet::Moon) {
            moon_long = pos.longitude;
        }
        let sign = ZodiacSign::from_degree(pos.longitude);
        let degree = pos.longitude.rem_euclid(30.0);
        let house = whole_sign_house(sign, ascendant_sign);
        let is_combust = !matches!(planet, Planet::Sun | Planet::Rahu | Planet::Ketu)
            && angular_distance(pos.longitude, sun_long) <= combust_threshold(planet);
        planet_positions.push(PlanetPosition {
            planet,
            sign,
            degree,
            longitude: pos.longitude,
            house,
            nakshatra: nakshatra_name(pos.longitude),
            pada: pada_from_longitude(pos.longitude),
            is_retrograde: pos.is_retrograde,
            is_combust,
            dignity: Some(calculate_dignity(planet, sign)),
        });
    }

    let moon_sign = ZodiacSign::from_degree(moon_long);

    // Whole-sign house cusps anchored on the ascendant sign.
    let houses: Vec<HouseCusp> = (1..=12u8)
        .map(|h| {
            let sign_num = ((ascendant_sign.number() - 1 + (h - 1)) % 12) + 1;
            let sign = ZodiacSign::from_number(sign_num).unwrap_or(ZodiacSign::Aries);
            HouseCusp {
                house: h,
                sign,
                degree: if h == 1 { ascendant_degree } else { 0.0 },
                lord: sign.ruler(),
            }
        })
        .collect();

    Ok(BirthChart {
        planets: planet_positions,
        houses,
        ascendant_sign,
        ascendant_degree,
        moon_sign,
        sun_sign,
        ayanamsa: "lahiri".to_string(),
        ayanamsa_value: ayanamsa,
    })
}

/// Parse the `(birth_date, birth_time)` request strings into civil
/// components callers can pass to [`build_native_chart`].
pub(crate) fn parse_birth_inputs(
    birth_date: &str,
    birth_time: &str,
) -> VedicApiResult<(i32, u32, u32, u32, u32, u32)> {
    let date = NaiveDate::parse_from_str(birth_date, "%Y-%m-%d").map_err(|e| {
        VedicApiError::ParseError(format!("invalid birth_date '{}': {}", birth_date, e))
    })?;
    let time = NaiveTime::parse_from_str(birth_time, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(birth_time, "%H:%M"))
        .map_err(|e| {
            VedicApiError::ParseError(format!("invalid birth_time '{}': {}", birth_time, e))
        })?;
    Ok((
        date.year(),
        date.month(),
        date.day(),
        time.hour(),
        time.minute(),
        time.second(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn bangalore_1991_chart_has_scorpio_ascendant() {
        // 1991-08-13 13:31 IST, Bangalore.
        let chart = build_native_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await
            .expect("native chart should compute");
        // The reference Scorpio-ascendant fixture across this crate ties to
        // the Shesh profile (1991-09-14). For 1991-08-13 13:31 IST Bangalore
        // the rising sign falls in Scorpio per multiple cross-checks; this
        // assertion verifies the construction pipeline lines up with the
        // existing chart_mapping fixtures.
        assert_eq!(chart.ascendant_sign, ZodiacSign::Scorpio);
        assert!(!chart.planets.is_empty());
        assert!(chart
            .planets
            .iter()
            .any(|p| matches!(p.planet, Planet::Sun)));
        assert!(chart
            .planets
            .iter()
            .any(|p| matches!(p.planet, Planet::Moon)));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn houses_count_to_twelve_and_first_matches_ascendant() {
        let chart = build_native_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await
            .unwrap();
        assert_eq!(chart.houses.len(), 12);
        assert_eq!(chart.houses[0].sign, chart.ascendant_sign);
    }
}
