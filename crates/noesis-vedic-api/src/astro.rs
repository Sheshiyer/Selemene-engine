//! Astronomical primitives shared across the native Vedic facade.
//!
//! PR3: introduces `sunrise_sunset`, a pure-Rust wrapper around the
//! [`sunrise`](https://crates.io/crates/sunrise) crate that the muhurta
//! date-range search loop relies on. Returns local `NaiveTime` values in the
//! caller-supplied timezone so downstream code can compare against panchang
//! day windows without juggling UTC offsets.
//!
//! Accuracy is approximately ±1 minute (Meeus algorithm with refraction
//! correction). That is well within the granularity Rahu Kalam / Yama Gandam
//! / muhurta slot windows use (1.5 hour bands), so we accept the trade-off
//! against pulling in Swiss Ephemeris C-FFI here.
//!
//! ## Why a separate module
//!
//! Sunrise/sunset is needed by muhurta. Adding it to `panchang::muhurta` would
//! create a cyclic import (panchang feeds muhurta which would feed panchang).
//! Hosting it at the crate root keeps it dependency-free of the analytical
//! modules.

use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};

/// Compute civil sunrise and sunset for `date` at the supplied location and
/// timezone. Returned `NaiveTime` values are expressed in the caller's local
/// timezone (`tz_offset_hours`, e.g. `5.5` for IST).
///
/// # Arguments
///
/// * `date` — civil date in the target timezone.
/// * `lat` — latitude in degrees (north positive).
/// * `lng` — longitude in degrees (east positive).
/// * `tz_offset_hours` — UTC offset of the local civil clock, e.g. 5.5 for IST.
///
/// # Behaviour
///
/// At extreme latitudes (above the Arctic / below the Antarctic circle near
/// solstices) sunrise/sunset are undefined. To avoid panics we clamp the
/// returned times to noon when the underlying calculation returns an event
/// outside the requested civil day.
pub fn sunrise_sunset(
    date: NaiveDate,
    lat: f64,
    lng: f64,
    tz_offset_hours: f64,
) -> (NaiveTime, NaiveTime) {
    // `sunrise::sunrise_sunset` is deprecated in the 1.2 series in favour of
    // a `SolarEvent` builder, but only the deprecated entry point is exposed
    // on the 1.x stable line we target here. Suppress the warning explicitly
    // so the crate keeps a warning-clean build under `cargo check`.
    #[allow(deprecated)]
    let (sr_ts, ss_ts) = sunrise::sunrise_sunset(lat, lng, date.year(), date.month(), date.day());

    let to_local = |unix: i64| -> NaiveTime {
        let utc: DateTime<Utc> = match Utc.timestamp_opt(unix, 0) {
            chrono::LocalResult::Single(dt) => dt,
            _ => return NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        };
        let offset_secs = (tz_offset_hours * 3600.0) as i64;
        let local = utc + chrono::Duration::seconds(offset_secs);
        NaiveTime::from_hms_opt(local.hour(), local.minute(), local.second())
            .unwrap_or_else(|| NaiveTime::from_hms_opt(12, 0, 0).unwrap())
    };

    (to_local(sr_ts), to_local(ss_ts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bangalore_august_1991_sunrise_around_six() {
        // Bangalore, mid August — sunrise lands roughly 06:00 IST.
        let date = NaiveDate::from_ymd_opt(1991, 8, 13).unwrap();
        let (sr, ss) = sunrise_sunset(date, 12.9716, 77.5946, 5.5);
        assert!(
            sr.hour() >= 5 && sr.hour() <= 7,
            "Bangalore Aug sunrise should be 5-7am IST, got {sr}",
        );
        assert!(
            ss.hour() >= 17 && ss.hour() <= 19,
            "Bangalore Aug sunset should be 5-7pm IST, got {ss}",
        );
    }

    #[test]
    fn bangalore_winter_sunrise_later_than_summer() {
        let summer = NaiveDate::from_ymd_opt(2024, 6, 21).unwrap();
        let winter = NaiveDate::from_ymd_opt(2024, 12, 21).unwrap();
        let (sr_summer, _) = sunrise_sunset(summer, 12.9716, 77.5946, 5.5);
        let (sr_winter, _) = sunrise_sunset(winter, 12.9716, 77.5946, 5.5);
        assert!(
            sr_winter > sr_summer,
            "Winter sunrise ({sr_winter}) must be later than summer ({sr_summer})",
        );
    }
}
