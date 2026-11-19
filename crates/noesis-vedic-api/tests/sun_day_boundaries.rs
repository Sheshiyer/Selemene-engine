//! Day boundaries use longitude, timezone and the real solar position (#1537).
//! Reference values: Swiss Ephemeris `swe_rise_trans`, local civil time.

use chrono::NaiveDate;
use noesis_vedic_api::daily_panchang::calculate_daily_panchang;
use noesis_vedic_api::panchang::api::compute_panchang_native;

fn minutes(hhmm: &str) -> i32 {
    let (h, m) = hhmm.split_once(':').expect("HH:MM");
    h.parse::<i32>().unwrap() * 60 + m.parse::<i32>().unwrap()
}

fn assert_within_a_minute(label: &str, got: &str, expected: &str) {
    assert!(
        (minutes(got) - minutes(expected)).abs() <= 1,
        "{label}: got {got}, expected {expected}"
    );
}

#[test]
fn native_panchang_day_boundaries_match_reference() {
    // Mumbai, January: the worst case of the old approximation (52 min).
    let p = compute_panchang_native(2026, 1, 15, 12, 0, 0, 19.0760, 72.8777, 5.5).unwrap();
    assert_within_a_minute("sunrise", &p.day_boundaries.sunrise, "07:14");
    assert_within_a_minute("sunset", &p.day_boundaries.sunset, "18:21");
    // Next sunrise is the following day's, not a copy of today's.
    let next = compute_panchang_native(2026, 1, 16, 12, 0, 0, 19.0760, 72.8777, 5.5).unwrap();
    assert_eq!(p.day_boundaries.next_sunrise, next.day_boundaries.sunrise);

    // Negative offset (New York, EST).
    let p = compute_panchang_native(2026, 10, 29, 12, 0, 0, 40.7128, -74.0060, -5.0).unwrap();
    assert_within_a_minute("NY sunrise", &p.day_boundaries.sunrise, "06:23");
    assert_within_a_minute("NY sunset", &p.day_boundaries.sunset, "16:55");
}

#[test]
fn native_panchang_reports_polar_night() {
    let p = compute_panchang_native(2026, 12, 21, 12, 0, 0, 78.22, 15.65, 1.0).unwrap();
    assert_eq!(p.day_boundaries.sunrise, "polar_night");
    assert_eq!(p.day_boundaries.sunset, "polar_night");
}

#[test]
fn daily_panchang_sunrise_is_computed_not_constant() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 29).unwrap();
    let kolkata = calculate_daily_panchang(date, 22.5726, 88.3639, 5.5);
    assert_within_a_minute("Kolkata sunrise", &kolkata.sunrise, "05:39");
    assert_within_a_minute("Kolkata sunset", &kolkata.sunset, "17:00");

    let london = calculate_daily_panchang(date, 51.5074, -0.1278, 0.0);
    assert_within_a_minute("London sunrise", &london.sunrise, "06:48");
    assert_within_a_minute("London sunset", &london.sunset, "16:39");

    let date = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();
    let svalbard = calculate_daily_panchang(date, 78.22, 15.65, 2.0);
    assert_eq!(svalbard.sunrise, "polar_day");
    assert_eq!(svalbard.sunset, "polar_day");
}
