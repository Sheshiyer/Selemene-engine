//! Offline sunrise/sunset for a latitude/longitude.
//!
//! Uses the NOAA solar-position equations (Meeus, *Astronomical Algorithms*,
//! low-precision solar coordinates): declination and equation of time are
//! evaluated at the event and the hour angle is iterated until it settles.
//! The event is the apparent rise/set of the Sun's upper limb with standard
//! refraction (geometric altitude -0.833°). Accuracy is about one minute
//! below ±65° latitude. Pure arithmetic: no network, no ephemeris files.

use chrono::{DateTime, Duration, NaiveDate, Utc};

/// Geometric altitude of the Sun's centre at apparent rise/set.
pub const SUN_EVENT_ALTITUDE_DEG: f64 = -0.833;

/// Identifier recorded in engine output so replays can name the method.
pub const SUN_EVENT_METHOD: &str = "noaa-solar-position";

const J2000: f64 = 2_451_545.0;
const UNIX_EPOCH_JD: f64 = 2_440_587.5;
const MINUTES_PER_DAY: f64 = 1440.0;
const ITERATIONS: usize = 4;

/// Which horizon crossing to compute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SunEvent {
    Rise,
    Set,
}

/// Sunrise or sunset (UTC) on the local civil `date` at the location.
///
/// `timezone_offset_minutes` only picks which solar day belongs to the local
/// date; the returned instant is absolute. Returns `None` when the Sun does
/// not cross the horizon that day (polar day or night).
pub fn sun_event(
    date: NaiveDate,
    latitude: f64,
    longitude: f64,
    timezone_offset_minutes: i32,
    event: SunEvent,
) -> Option<DateTime<Utc>> {
    let utc_midnight = date.and_hms_opt(0, 0, 0)?.and_utc();
    let jd_midnight = julian_day(utc_midnight);

    // Shift by whole days so the solar noon used lands on the local date
    // even where the civil offset is far from longitude / 15.
    let approx_noon = 720.0 - 4.0 * longitude;
    let local_noon = 720.0 - f64::from(timezone_offset_minutes);
    let day_shift = ((local_noon - approx_noon) / MINUTES_PER_DAY).round() * MINUTES_PER_DAY;

    let mut minutes = approx_noon + day_shift;
    for _ in 0..ITERATIONS {
        let (declination, equation_of_time) =
            solar_position(jd_midnight + minutes / MINUTES_PER_DAY);
        let hour_angle = hour_angle_deg(latitude, declination)?;
        let signed = match event {
            SunEvent::Rise => hour_angle,
            SunEvent::Set => -hour_angle,
        };
        minutes = 720.0 - 4.0 * (longitude + signed) - equation_of_time + day_shift;
    }

    let millis = (minutes * 60_000.0).round() as i64;
    Some(utc_midnight + Duration::milliseconds(millis))
}

fn julian_day(instant: DateTime<Utc>) -> f64 {
    instant.timestamp_millis() as f64 / 86_400_000.0 + UNIX_EPOCH_JD
}

/// Hour angle (degrees) of the event, or `None` if the Sun never reaches it.
fn hour_angle_deg(latitude: f64, declination: f64) -> Option<f64> {
    let lat = latitude.to_radians();
    let dec = declination.to_radians();
    let cos_h = (SUN_EVENT_ALTITUDE_DEG.to_radians().sin() - lat.sin() * dec.sin())
        / (lat.cos() * dec.cos());
    if !(-1.0..=1.0).contains(&cos_h) {
        return None;
    }
    Some(cos_h.acos().to_degrees())
}

/// Apparent solar declination (degrees) and equation of time (minutes).
fn solar_position(jd: f64) -> (f64, f64) {
    let t = (jd - J2000) / 36_525.0;

    let mean_longitude = (280.46646 + t * (36000.76983 + t * 0.0003032)).rem_euclid(360.0);
    let mean_anomaly = 357.52911 + t * (35999.05029 - 0.0001537 * t);
    let eccentricity = 0.016_708_634 - t * (0.000_042_037 + 0.000_000_126_7 * t);

    let m = mean_anomaly.to_radians();
    let center = m.sin() * (1.914_602 - t * (0.004_817 + 0.000_014 * t))
        + (2.0 * m).sin() * (0.019_993 - 0.000_101 * t)
        + (3.0 * m).sin() * 0.000_289;
    let true_longitude = mean_longitude + center;

    let omega = (125.04 - 1934.136 * t).to_radians();
    let apparent_longitude = (true_longitude - 0.005_69 - 0.004_78 * omega.sin()).to_radians();

    let seconds = 21.448 - t * (46.815 + t * (0.000_59 - t * 0.001_813));
    let mean_obliquity = 23.0 + (26.0 + seconds / 60.0) / 60.0;
    let obliquity = (mean_obliquity + 0.002_56 * omega.cos()).to_radians();

    let declination = (obliquity.sin() * apparent_longitude.sin())
        .asin()
        .to_degrees();

    let y = (obliquity / 2.0).tan().powi(2);
    let l0 = mean_longitude.to_radians();
    let e = eccentricity;
    let equation_of_time = 4.0
        * (y * (2.0 * l0).sin() - 2.0 * e * m.sin() + 4.0 * e * y * m.sin() * (2.0 * l0).cos()
            - 0.5 * y * y * (4.0 * l0).sin()
            - 1.25 * e * e * (2.0 * m).sin())
        .to_degrees();

    (declination, equation_of_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, Timelike};

    fn local_hm(instant: DateTime<Utc>, offset_minutes: i32) -> (u32, u32) {
        let local = instant.with_timezone(&FixedOffset::east_opt(offset_minutes * 60).unwrap());
        (local.hour(), local.minute())
    }

    #[test]
    fn equinox_on_equator_at_greenwich_is_near_six() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 20).unwrap();
        let rise = sun_event(date, 0.0, 0.0, 0, SunEvent::Rise).unwrap();
        let set = sun_event(date, 0.0, 0.0, 0, SunEvent::Set).unwrap();
        // 06:04:10 / 18:10:41 UTC (Swiss Ephemeris agrees within 3 s): equation of
        // time and refraction move both events off 06:00 / 18:00.
        assert_eq!(local_hm(rise, 0), (6, 4));
        assert_eq!(local_hm(set, 0), (18, 10));
    }

    #[test]
    fn polar_night_has_no_sunrise() {
        let date = NaiveDate::from_ymd_opt(2026, 12, 21).unwrap();
        assert!(sun_event(date, 78.22, 15.65, 60, SunEvent::Rise).is_none());
        assert!(sun_event(date, 78.22, 15.65, 60, SunEvent::Set).is_none());
    }

    #[test]
    fn events_land_on_the_requested_local_date_far_from_greenwich() {
        // Kiritimati: UTC+14 at ~157° W, the worst case for the day shift.
        let date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let offset = 14 * 60;
        let rise = sun_event(date, 1.87, -157.4, offset, SunEvent::Rise).unwrap();
        let set = sun_event(date, 1.87, -157.4, offset, SunEvent::Set).unwrap();
        let tz = FixedOffset::east_opt(offset * 60).unwrap();
        assert_eq!(rise.with_timezone(&tz).date_naive(), date);
        assert_eq!(set.with_timezone(&tz).date_naive(), date);
        assert!(rise < set);
    }
}
