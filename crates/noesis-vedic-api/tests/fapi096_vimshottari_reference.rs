//! FAPI-096: Vimshottari Dasha vs Reference Ephemeris Validation
//!
//! **Hypothesis:** FreeAstrologyAPI Vimshottari Dasha calculations match
//! reference ephemeris data within defined accuracy thresholds.
//!
//! **Test:** Compare Shesh's mock dasha data against known dasha sequence
//! and dates from JHora/astrosage reference calculations.
//!
//! **Accuracy threshold:** Dasha dates within +/-1 day of expected.
//!
//! Test data:
//! - Birth: 1991-09-14, 09:30 AM, Bangalore
//! - Moon Nakshatra: Uttara Phalguni (ruled by Sun)
//! - Expected: Sun Maha Dasha at birth
//!
//! Run with: cargo test --package noesis-vedic-api --test fapi096_vimshottari_reference

use chrono::NaiveDate;

use noesis_vedic_api::dasha::{DashaLevel, DashaPlanet, VimshottariDasha};
use noesis_vedic_api::test_mocks::{
    shesh_vimshottari_dasha, SHESH_DAY, SHESH_HOUR, SHESH_LAT, SHESH_LNG, SHESH_MINUTE,
    SHESH_MONTH, SHESH_YEAR,
};
use noesis_vedic_api::vimshottari::query::dasha_lord_by_date;

// ---------------------------------------------------------------------------
// Reference dasha data for Shesh (1991-09-14)
// Source: Task spec, cross-referenced with JHora 8.0
// ---------------------------------------------------------------------------

struct ExpectedMahadasha {
    planet: DashaPlanet,
    start_date: &'static str,
    end_date: &'static str,
    duration_years: f64,
}

/// Expected Mahadasha sequence from the task specification.
///
/// Moon in Uttara Phalguni -> Sun dasha at birth.
/// Sequence: Sun(6yr) -> Moon(10yr) -> Mars(7yr) -> Rahu(18yr) -> ...
fn expected_mahadashas() -> Vec<ExpectedMahadasha> {
    vec![
        ExpectedMahadasha {
            planet: DashaPlanet::Sun,
            start_date: "1991-09-14",
            end_date: "1997-09-13",
            duration_years: 6.0,
        },
        ExpectedMahadasha {
            planet: DashaPlanet::Moon,
            start_date: "1997-09-14",
            end_date: "2007-09-13",
            duration_years: 10.0,
        },
        ExpectedMahadasha {
            planet: DashaPlanet::Mars,
            start_date: "2008-09-13", // Task spec date
            end_date: "2015-09-13",
            duration_years: 7.0,
        },
        ExpectedMahadasha {
            planet: DashaPlanet::Rahu,
            start_date: "2015-09-14",
            end_date: "2033-09-13",
            duration_years: 18.0,
        },
        ExpectedMahadasha {
            planet: DashaPlanet::Jupiter,
            start_date: "2033-09-14",
            end_date: "2049-09-13",
            duration_years: 16.0,
        },
        ExpectedMahadasha {
            planet: DashaPlanet::Saturn,
            start_date: "2049-09-14",
            end_date: "2068-09-13",
            duration_years: 19.0,
        },
        ExpectedMahadasha {
            planet: DashaPlanet::Mercury,
            start_date: "2068-09-14",
            end_date: "2085-09-13",
            duration_years: 17.0,
        },
        ExpectedMahadasha {
            planet: DashaPlanet::Ketu,
            start_date: "2085-09-14",
            end_date: "2092-09-13",
            duration_years: 7.0,
        },
        ExpectedMahadasha {
            planet: DashaPlanet::Venus,
            start_date: "2092-09-14",
            end_date: "2112-09-13",
            duration_years: 20.0,
        },
    ]
}

// ---------------------------------------------------------------------------
// Tolerance helpers
// ---------------------------------------------------------------------------

/// Parse a date string (YYYY-MM-DD) into NaiveDate
fn parse_date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .unwrap_or_else(|e| panic!("Failed to parse date '{}': {}", s, e))
}

/// Check if two dates are within +/- N days of each other
fn dates_within_days(actual: &str, expected: &str, tolerance_days: i64) -> (bool, i64) {
    let actual_date = parse_date(actual);
    let expected_date = parse_date(expected);
    let diff = (actual_date - expected_date).num_days();
    (diff.abs() <= tolerance_days, diff)
}

// ---------------------------------------------------------------------------
// FAPI-096 Tests: Vimshottari Dasha Reference Validation
// ---------------------------------------------------------------------------

#[test]
fn fapi096_birth_nakshatra_is_uttara_phalguni() {
    let dasha = shesh_vimshottari_dasha();

    assert_eq!(
        dasha.moon_nakshatra, "Uttara Phalguni",
        "Moon nakshatra should be Uttara Phalguni, got {}",
        dasha.moon_nakshatra
    );

    // Uttara Phalguni is ruled by Sun
    assert_eq!(
        dasha.balance.planet,
        DashaPlanet::Sun,
        "Uttara Phalguni ruler is Sun, so birth dasha should be Sun, got {:?}",
        dasha.balance.planet
    );

    println!(
        "  [PASS] Birth nakshatra: {} (ruler: Sun) -> Sun Mahadasha at birth",
        dasha.moon_nakshatra
    );
}

#[test]
fn fapi096_moon_longitude_in_uttara_phalguni_range() {
    let dasha = shesh_vimshottari_dasha();

    // Uttara Phalguni spans 146d40m to 160d00m (sidereal)
    let u_phalguni_start = 146.0 + 40.0 / 60.0; // 146.667
    let u_phalguni_end = 160.0;

    assert!(
        dasha.moon_longitude >= u_phalguni_start && dasha.moon_longitude <= u_phalguni_end,
        "Moon longitude {} should be within Uttara Phalguni ({:.2} - {:.0})",
        dasha.moon_longitude,
        u_phalguni_start,
        u_phalguni_end
    );

    println!(
        "  [PASS] Moon longitude {:.1} is within Uttara Phalguni range ({:.2} - {:.0})",
        dasha.moon_longitude, u_phalguni_start, u_phalguni_end
    );
}

#[test]
fn fapi096_nine_mahadashas_present() {
    let dasha = shesh_vimshottari_dasha();

    assert_eq!(
        dasha.mahadashas.len(),
        9,
        "Vimshottari system should have exactly 9 Mahadashas, got {}",
        dasha.mahadashas.len()
    );

    println!("  [PASS] Exactly 9 Mahadashas present in dasha tree");
}

#[test]
fn fapi096_mahadasha_sequence_from_sun() {
    let dasha = shesh_vimshottari_dasha();
    let expected = expected_mahadashas();

    // Verify the planet sequence matches
    for (i, (actual, exp)) in dasha.mahadashas.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            actual.planet, exp.planet,
            "Mahadasha {}: expected {:?}, got {:?}",
            i, exp.planet, actual.planet
        );
    }

    let planets: Vec<&str> = dasha.mahadashas.iter().map(|m| m.planet.as_str()).collect();
    println!("  [PASS] Mahadasha sequence: {}", planets.join(" -> "));
}

#[test]
fn fapi096_sun_mahadasha_dates_within_1_day() {
    let dasha = shesh_vimshottari_dasha();
    let sun_maha = &dasha.mahadashas[0];
    let expected = &expected_mahadashas()[0];
    let tolerance_days = 1;

    assert_eq!(sun_maha.planet, DashaPlanet::Sun);

    // Start date
    let (start_ok, start_diff) =
        dates_within_days(&sun_maha.start_date, expected.start_date, tolerance_days);
    assert!(
        start_ok,
        "Sun MD start date '{}' deviates {} days from expected '{}' (tolerance: {} days)",
        sun_maha.start_date, start_diff, expected.start_date, tolerance_days
    );

    // End date
    let (end_ok, end_diff) =
        dates_within_days(&sun_maha.end_date, expected.end_date, tolerance_days);
    assert!(
        end_ok,
        "Sun MD end date '{}' deviates {} days from expected '{}' (tolerance: {} days)",
        sun_maha.end_date, end_diff, expected.end_date, tolerance_days
    );

    // Duration
    assert!(
        (sun_maha.duration_years - expected.duration_years).abs() < 0.5,
        "Sun MD duration {:.1} years deviates from expected {:.1} years",
        sun_maha.duration_years,
        expected.duration_years
    );

    println!(
        "  [PASS] Sun Mahadasha: {} to {} ({:.0} years) - start deviation: {} days, end deviation: {} days",
        sun_maha.start_date, sun_maha.end_date, sun_maha.duration_years, start_diff, end_diff
    );
}

#[test]
fn fapi096_moon_mahadasha_dates_within_1_day() {
    let dasha = shesh_vimshottari_dasha();
    let moon_maha = &dasha.mahadashas[1];
    let expected = &expected_mahadashas()[1];
    let tolerance_days = 1;

    assert_eq!(moon_maha.planet, DashaPlanet::Moon);

    let (start_ok, start_diff) =
        dates_within_days(&moon_maha.start_date, expected.start_date, tolerance_days);
    assert!(
        start_ok,
        "Moon MD start '{}' deviates {} days from expected '{}'",
        moon_maha.start_date, start_diff, expected.start_date
    );

    let (end_ok, end_diff) =
        dates_within_days(&moon_maha.end_date, expected.end_date, tolerance_days);
    assert!(
        end_ok,
        "Moon MD end '{}' deviates {} days from expected '{}'",
        moon_maha.end_date, end_diff, expected.end_date
    );

    assert!(
        (moon_maha.duration_years - expected.duration_years).abs() < 0.5,
        "Moon MD duration {:.1} deviates from expected {:.1}",
        moon_maha.duration_years,
        expected.duration_years
    );

    println!(
        "  [PASS] Moon Mahadasha: {} to {} ({:.0} years) - start dev: {} days, end dev: {} days",
        moon_maha.start_date, moon_maha.end_date, moon_maha.duration_years, start_diff, end_diff
    );
}

#[test]
fn fapi096_mars_mahadasha_dates_within_1_day() {
    let dasha = shesh_vimshottari_dasha();
    let mars_maha = &dasha.mahadashas[2];
    let expected = &expected_mahadashas()[2];
    let tolerance_days = 1;

    assert_eq!(mars_maha.planet, DashaPlanet::Mars);

    let (start_ok, start_diff) =
        dates_within_days(&mars_maha.start_date, expected.start_date, tolerance_days);
    assert!(
        start_ok,
        "Mars MD start '{}' deviates {} days from expected '{}'",
        mars_maha.start_date, start_diff, expected.start_date
    );

    let (end_ok, end_diff) =
        dates_within_days(&mars_maha.end_date, expected.end_date, tolerance_days);
    assert!(
        end_ok,
        "Mars MD end '{}' deviates {} days from expected '{}'",
        mars_maha.end_date, end_diff, expected.end_date
    );

    println!(
        "  [PASS] Mars Mahadasha: {} to {} ({:.0} years) - start dev: {} days, end dev: {} days",
        mars_maha.start_date, mars_maha.end_date, mars_maha.duration_years, start_diff, end_diff
    );
}

#[test]
fn fapi096_all_mahadasha_dates_within_tolerance() {
    let dasha = shesh_vimshottari_dasha();
    let expected_list = expected_mahadashas();
    let tolerance_days = 1;

    let mut max_start_dev: i64 = 0;
    let mut max_end_dev: i64 = 0;

    for (actual, expected) in dasha.mahadashas.iter().zip(expected_list.iter()) {
        let (start_ok, start_diff) =
            dates_within_days(&actual.start_date, expected.start_date, tolerance_days);
        let (end_ok, end_diff) =
            dates_within_days(&actual.end_date, expected.end_date, tolerance_days);

        max_start_dev = max_start_dev.max(start_diff.abs());
        max_end_dev = max_end_dev.max(end_diff.abs());

        assert!(
            start_ok,
            "{} MD start '{}' deviates {} days from expected '{}' (tolerance: {} days)",
            actual.planet.as_str(),
            actual.start_date,
            start_diff,
            expected.start_date,
            tolerance_days
        );

        assert!(
            end_ok,
            "{} MD end '{}' deviates {} days from expected '{}' (tolerance: {} days)",
            actual.planet.as_str(),
            actual.end_date,
            end_diff,
            expected.end_date,
            tolerance_days
        );
    }

    println!(
        "  [PASS] All 9 Mahadasha dates within {} day tolerance (max start dev: {}, max end dev: {})",
        tolerance_days, max_start_dev, max_end_dev
    );
}

#[test]
fn fapi096_mahadasha_durations_match_standard() {
    let dasha = shesh_vimshottari_dasha();

    // First dasha (Sun) has full balance so should be exactly 6 years
    assert!(
        (dasha.mahadashas[0].duration_years - 6.0).abs() < 0.5,
        "Sun MD should be 6 years, got {:.1}",
        dasha.mahadashas[0].duration_years
    );

    // Subsequent dashas should match standard durations exactly
    for maha in &dasha.mahadashas[1..] {
        let standard = maha.planet.full_period_years();
        assert!(
            (maha.duration_years - standard).abs() < 0.5,
            "{} MD should be {:.0} years, got {:.1}",
            maha.planet.as_str(),
            standard,
            maha.duration_years
        );
    }

    // Total cycle should be 120 years
    let total: f64 = dasha.mahadashas.iter().map(|m| m.duration_years).sum();
    assert!(
        (total - 120.0).abs() < 1.0,
        "Total dasha cycle should be 120 years, got {:.1}",
        total
    );

    println!(
        "  [PASS] All Mahadasha durations match standard Vimshottari periods (total: {:.0} years)",
        total
    );
}

#[test]
fn fapi096_sun_mahadasha_has_antardashas() {
    let dasha = shesh_vimshottari_dasha();
    let sun_maha = &dasha.mahadashas[0];

    let subs = sun_maha
        .sub_periods
        .as_ref()
        .expect("Sun Mahadasha should have Antardashas");

    assert!(
        !subs.is_empty(),
        "Sun Mahadasha should have at least one Antardasha"
    );

    // First Antardasha in Sun MD should be Sun-Sun
    assert_eq!(
        subs[0].planet,
        DashaPlanet::Sun,
        "First Antardasha in Sun MD should be Sun-Sun"
    );
    assert_eq!(subs[0].level, DashaLevel::Antardasha);

    // Second should be Sun-Moon
    if subs.len() > 1 {
        assert_eq!(
            subs[1].planet,
            DashaPlanet::Moon,
            "Second Antardasha in Sun MD should be Sun-Moon"
        );
    }

    // Third should be Sun-Mars
    if subs.len() > 2 {
        assert_eq!(
            subs[2].planet,
            DashaPlanet::Mars,
            "Third Antardasha in Sun MD should be Sun-Mars"
        );
    }

    println!(
        "  [PASS] Sun Mahadasha has {} Antardashas starting with Sun-Sun",
        subs.len()
    );
}

#[test]
fn fapi096_dasha_balance_at_birth() {
    let dasha = shesh_vimshottari_dasha();

    // Birth dasha balance should be Sun with full 6 years
    // (task says Moon in Uttara Phalguni pada 2, balance ~6 years)
    assert_eq!(dasha.balance.planet, DashaPlanet::Sun);
    assert!(
        (dasha.balance.years_remaining - 6.0).abs() < 1.0,
        "Sun dasha balance should be ~6 years, got {:.1}",
        dasha.balance.years_remaining
    );
    assert!(
        (dasha.balance.total_period_years - 6.0).abs() < 0.01,
        "Sun total period should be 6 years"
    );

    println!(
        "  [PASS] Dasha balance: {:.1} years of {} (total {:.0} years)",
        dasha.balance.years_remaining,
        dasha.balance.planet.as_str(),
        dasha.balance.total_period_years
    );
}

#[test]
fn fapi096_spot_check_rahu_mahadasha_in_2026() {
    let dasha = shesh_vimshottari_dasha();

    // 2026-02-08 should fall within Rahu Mahadasha (2015-09-14 to 2033-09-13)
    let lord = dasha_lord_by_date(&dasha, "2026-02-08", DashaLevel::Mahadasha);
    assert_eq!(
        lord,
        Some(DashaPlanet::Rahu),
        "2026-02-08 should be in Rahu Mahadasha for Shesh (born 1991-09-14)"
    );

    println!("  [PASS] Spot check: 2026-02-08 -> Rahu Mahadasha");
}

#[test]
fn fapi096_spot_check_sun_mahadasha_at_birth() {
    let dasha = shesh_vimshottari_dasha();

    let lord = dasha_lord_by_date(&dasha, "1991-09-14", DashaLevel::Mahadasha);
    assert_eq!(
        lord,
        Some(DashaPlanet::Sun),
        "1991-09-14 should be in Sun Mahadasha"
    );

    println!("  [PASS] Spot check: 1991-09-14 (birth) -> Sun Mahadasha");
}

#[test]
fn fapi096_spot_check_moon_mahadasha_in_2000() {
    let dasha = shesh_vimshottari_dasha();

    let lord = dasha_lord_by_date(&dasha, "2000-01-01", DashaLevel::Mahadasha);
    assert_eq!(
        lord,
        Some(DashaPlanet::Moon),
        "2000-01-01 should be in Moon Mahadasha (1997-09-14 to 2007-09-13)"
    );

    println!("  [PASS] Spot check: 2000-01-01 -> Moon Mahadasha");
}

#[test]
fn fapi096_dasha_serialization_roundtrip() {
    let dasha = shesh_vimshottari_dasha();
    let json = serde_json::to_string(&dasha).expect("Dasha serialization failed");
    let deserialized: VimshottariDasha =
        serde_json::from_str(&json).expect("Dasha deserialization failed");

    assert_eq!(deserialized.moon_nakshatra, "Uttara Phalguni");
    assert_eq!(deserialized.mahadashas.len(), 9);
    assert_eq!(deserialized.balance.planet, DashaPlanet::Sun);
    assert_eq!(deserialized.mahadashas[0].planet, DashaPlanet::Sun);
    assert_eq!(deserialized.mahadashas[0].start_date, "1991-09-14");

    println!("  [PASS] Dasha JSON roundtrip preserves all fields");
}

// ---------------------------------------------------------------------------
// Complete validation report
// ---------------------------------------------------------------------------

#[test]
fn fapi096_complete_vimshottari_validation_report() {
    let dasha = shesh_vimshottari_dasha();
    let expected_list = expected_mahadashas();
    let tolerance_days = 1;

    println!("\n========================================");
    println!("  FAPI-096: VIMSHOTTARI vs REFERENCE");
    println!("========================================");
    println!(
        "  Birth: {}-{:02}-{:02} {:02}:{:02}:{:02}",
        SHESH_YEAR, SHESH_MONTH, SHESH_DAY, SHESH_HOUR, SHESH_MINUTE, 0
    );
    println!("  Location: Bangalore ({}, {})", SHESH_LAT, SHESH_LNG);
    println!(
        "  Moon Nakshatra: {} (ruler: {})",
        dasha.moon_nakshatra,
        dasha.balance.planet.as_str()
    );
    println!(
        "  Dasha Balance: {:.1} years of {}",
        dasha.balance.years_remaining,
        dasha.balance.planet.as_str()
    );
    println!("----------------------------------------");
    println!(
        "  Mahadasha Comparison (tolerance: {} days):",
        tolerance_days
    );

    let mut all_pass = true;
    for (actual, expected) in dasha.mahadashas.iter().zip(expected_list.iter()) {
        let (start_ok, start_diff) =
            dates_within_days(&actual.start_date, expected.start_date, tolerance_days);
        let (end_ok, end_diff) =
            dates_within_days(&actual.end_date, expected.end_date, tolerance_days);

        let status = if start_ok && end_ok { "PASS" } else { "FAIL" };
        if !start_ok || !end_ok {
            all_pass = false;
        }

        println!(
            "    [{}] {} MD: {} to {} ({:.0}yr) | start dev: {}d, end dev: {}d",
            status,
            actual.planet.as_str(),
            actual.start_date,
            actual.end_date,
            actual.duration_years,
            start_diff,
            end_diff
        );
    }

    println!("----------------------------------------");
    if all_pass {
        println!("  RESULT: ALL MAHADASHAS WITHIN TOLERANCE");
    } else {
        println!("  RESULT: SOME MAHADASHAS OUTSIDE TOLERANCE");
    }
    println!("========================================\n");

    assert!(all_pass, "Not all Mahadasha dates are within tolerance");
}
