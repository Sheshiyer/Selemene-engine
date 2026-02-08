//! FAPI-097: Shesh's Birth Profile Validation
//!
//! **Hypothesis:** FreeAstrologyAPI birth chart calculations produce correct
//! results for Shesh's known birth data.
//!
//! **Test:** Validate all profile attributes against known reference values:
//! - Birth: 1991-09-14, 09:30 AM, Bangalore (12.9716 N, 77.5946 E)
//! - Ascendant: Scorpio
//! - Moon: Virgo
//! - Nakshatra: Uttara Phalguni
//! - Sun Maha Dasha start: 1991-09-14
//!
//! **Success criteria:** All attributes match expected values exactly.
//!
//! Run with: cargo test --package noesis-vedic-api --test fapi097_shesh_profile_validation

use noesis_vedic_api::chart::ZodiacSign;
use noesis_vedic_api::dasha::{DashaLevel, DashaPlanet};
use noesis_vedic_api::test_mocks::{
    shesh_birth_chart, shesh_navamsa_chart, shesh_panchang, shesh_vimshottari_dasha,
    MockApiClient, MockResponses,
    SHESH_DAY, SHESH_HOUR, SHESH_LAT, SHESH_LNG, SHESH_MINUTE, SHESH_MONTH, SHESH_SECOND,
    SHESH_TZONE, SHESH_YEAR,
};
use noesis_vedic_api::vimshottari::query::dasha_lord_by_date;

// ---------------------------------------------------------------------------
// Constants: Expected profile values from reference
// ---------------------------------------------------------------------------

const EXPECTED_ASCENDANT: ZodiacSign = ZodiacSign::Scorpio;
const EXPECTED_MOON_SIGN: ZodiacSign = ZodiacSign::Virgo;
const EXPECTED_MOON_NAKSHATRA: &str = "Uttara Phalguni";
const EXPECTED_MOON_NAKSHATRA_PADA: u8 = 2;
const EXPECTED_BIRTH_DASHA_PLANET: DashaPlanet = DashaPlanet::Sun;
const EXPECTED_SUN_MD_START: &str = "1991-09-14";

// ---------------------------------------------------------------------------
// FAPI-097 Tests: Birth Data Constants
// ---------------------------------------------------------------------------

#[test]
fn fapi097_birth_data_constants_correct() {
    assert_eq!(SHESH_YEAR, 1991, "Birth year");
    assert_eq!(SHESH_MONTH, 9, "Birth month (September)");
    assert_eq!(SHESH_DAY, 14, "Birth day");
    assert_eq!(SHESH_HOUR, 9, "Birth hour (9 AM)");
    assert_eq!(SHESH_MINUTE, 30, "Birth minute");
    assert_eq!(SHESH_SECOND, 0, "Birth second");
    assert!((SHESH_LAT - 12.9716).abs() < 0.001, "Bangalore latitude");
    assert!((SHESH_LNG - 77.5946).abs() < 0.001, "Bangalore longitude");
    assert!((SHESH_TZONE - 5.5).abs() < 0.001, "IST timezone");

    println!("  [PASS] Birth data constants: 1991-09-14 09:30 AM IST, Bangalore ({}, {})",
        SHESH_LAT, SHESH_LNG);
}

// ---------------------------------------------------------------------------
// FAPI-097 Tests: Ascendant Validation
// ---------------------------------------------------------------------------

#[test]
fn fapi097_ascendant_is_scorpio() {
    let chart = shesh_birth_chart();

    assert_eq!(
        chart.ascendant.sign, EXPECTED_ASCENDANT,
        "Ascendant should be Scorpio, got {:?}",
        chart.ascendant.sign
    );

    // Scorpio ruler is Mars
    assert_eq!(
        chart.ascendant.sign.ruler(),
        "Mars",
        "Scorpio ruler should be Mars"
    );

    println!(
        "  [PASS] Ascendant: {} at {:.1} deg (nakshatra: {}, pada {})",
        chart.ascendant.sign.as_str(),
        chart.ascendant.degree,
        chart.ascendant.nakshatra,
        chart.ascendant.pada
    );
}

#[test]
fn fapi097_ascendant_nakshatra_is_anuradha() {
    let chart = shesh_birth_chart();

    assert_eq!(
        chart.ascendant.nakshatra, "Anuradha",
        "Ascendant nakshatra should be Anuradha, got {}",
        chart.ascendant.nakshatra
    );

    println!(
        "  [PASS] Ascendant nakshatra: {} (pada {})",
        chart.ascendant.nakshatra, chart.ascendant.pada
    );
}

// ---------------------------------------------------------------------------
// FAPI-097 Tests: Moon Validation
// ---------------------------------------------------------------------------

#[test]
fn fapi097_moon_in_virgo() {
    let chart = shesh_birth_chart();

    assert_eq!(
        chart.moon.sign, EXPECTED_MOON_SIGN,
        "Moon should be in Virgo, got {:?}",
        chart.moon.sign
    );

    // Virgo ruler is Mercury
    assert_eq!(
        chart.moon.rashi_lord, "Mercury",
        "Moon rashi lord should be Mercury"
    );

    println!(
        "  [PASS] Moon: {} at {:.1} deg (rashi lord: {})",
        chart.moon.sign.as_str(),
        chart.moon.degree,
        chart.moon.rashi_lord
    );
}

#[test]
fn fapi097_moon_nakshatra_is_uttara_phalguni() {
    let chart = shesh_birth_chart();

    assert_eq!(
        chart.moon.nakshatra, EXPECTED_MOON_NAKSHATRA,
        "Moon nakshatra should be Uttara Phalguni, got {}",
        chart.moon.nakshatra
    );

    assert_eq!(
        chart.moon.pada, EXPECTED_MOON_NAKSHATRA_PADA,
        "Moon nakshatra pada should be 2, got {}",
        chart.moon.pada
    );

    println!(
        "  [PASS] Moon nakshatra: {} pada {}",
        chart.moon.nakshatra, chart.moon.pada
    );
}

// ---------------------------------------------------------------------------
// FAPI-097 Tests: Sun Maha Dasha Start
// ---------------------------------------------------------------------------

#[test]
fn fapi097_sun_mahadasha_starts_at_birth() {
    let dasha = shesh_vimshottari_dasha();

    assert_eq!(
        dasha.balance.planet, EXPECTED_BIRTH_DASHA_PLANET,
        "Birth dasha should be Sun (Uttara Phalguni ruler), got {:?}",
        dasha.balance.planet
    );

    assert_eq!(
        dasha.mahadashas[0].start_date, EXPECTED_SUN_MD_START,
        "Sun Mahadasha should start on {}, got {}",
        EXPECTED_SUN_MD_START,
        dasha.mahadashas[0].start_date
    );

    assert_eq!(
        dasha.mahadashas[0].planet,
        DashaPlanet::Sun,
        "First Mahadasha should be Sun"
    );

    println!(
        "  [PASS] Sun Mahadasha start: {} (matches birth date)",
        dasha.mahadashas[0].start_date
    );
}

#[test]
fn fapi097_birth_dasha_identified_at_birth_date() {
    let dasha = shesh_vimshottari_dasha();

    let lord = dasha_lord_by_date(&dasha, "1991-09-14", DashaLevel::Mahadasha);
    assert_eq!(
        lord,
        Some(DashaPlanet::Sun),
        "Dasha lord at birth should be Sun"
    );

    println!("  [PASS] Dasha lord at 1991-09-14: Sun");
}

// ---------------------------------------------------------------------------
// FAPI-097 Tests: Planetary Positions
// ---------------------------------------------------------------------------

#[test]
fn fapi097_all_nine_planets_present() {
    let chart = shesh_birth_chart();

    assert_eq!(
        chart.planets.len(),
        9,
        "Chart should have exactly 9 planets"
    );

    let planet_names: Vec<&str> = chart.planets.iter().map(|p| p.name.as_str()).collect();
    for expected in &["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"] {
        assert!(
            planet_names.contains(expected),
            "Planet {} missing from chart",
            expected
        );
    }

    println!("  [PASS] All 9 Vedic planets present in chart");
}

#[test]
fn fapi097_sun_in_leo() {
    let chart = shesh_birth_chart();
    let sun = chart.get_planet("Sun").expect("Sun should be in chart");

    assert_eq!(
        sun.sign,
        ZodiacSign::Leo,
        "Sun should be in Leo (own sign), got {:?}",
        sun.sign
    );
    assert!(sun.in_own_sign(), "Sun in Leo is in own sign");
    assert_eq!(sun.house, 10, "Sun should be in 10th house from Scorpio");

    println!(
        "  [PASS] Sun: {} at {:.1} deg (house {}, own sign: true)",
        sun.sign.as_str(),
        sun.degree,
        sun.house
    );
}

#[test]
fn fapi097_jupiter_exalted_in_cancer() {
    let chart = shesh_birth_chart();
    let jupiter = chart.get_planet("Jupiter").expect("Jupiter should be in chart");

    assert_eq!(
        jupiter.sign,
        ZodiacSign::Cancer,
        "Jupiter should be in Cancer (exalted), got {:?}",
        jupiter.sign
    );
    assert_eq!(jupiter.house, 9, "Jupiter should be in 9th house");

    println!(
        "  [PASS] Jupiter: {} at {:.1} deg (house {}, exalted in Cancer)",
        jupiter.sign.as_str(),
        jupiter.degree,
        jupiter.house
    );
}

#[test]
fn fapi097_mercury_exalted_in_virgo() {
    let chart = shesh_birth_chart();
    let mercury = chart.get_planet("Mercury").expect("Mercury should be in chart");

    assert_eq!(mercury.sign, ZodiacSign::Virgo, "Mercury should be in Virgo");
    assert!(mercury.is_exalted(), "Mercury is exalted in Virgo");
    assert_eq!(mercury.house, 11, "Mercury should be in 11th house");

    println!(
        "  [PASS] Mercury: {} at {:.1} deg (house {}, exalted: true)",
        mercury.sign.as_str(),
        mercury.degree,
        mercury.house
    );
}

#[test]
fn fapi097_saturn_own_sign_retrograde() {
    let chart = shesh_birth_chart();
    let saturn = chart.get_planet("Saturn").expect("Saturn should be in chart");

    assert_eq!(saturn.sign, ZodiacSign::Capricorn, "Saturn should be in Capricorn");
    assert!(saturn.in_own_sign(), "Saturn in Capricorn is in own sign");
    assert!(saturn.is_retrograde, "Saturn should be retrograde");
    assert_eq!(saturn.house, 3, "Saturn should be in 3rd house");

    println!(
        "  [PASS] Saturn: {} at {:.1} deg (house {}, own sign, retrograde)",
        saturn.sign.as_str(),
        saturn.degree,
        saturn.house
    );
}

#[test]
fn fapi097_mars_debilitated_in_cancer() {
    let chart = shesh_birth_chart();
    let mars = chart.get_planet("Mars").expect("Mars should be in chart");

    assert_eq!(mars.sign, ZodiacSign::Cancer, "Mars should be in Cancer");
    assert!(mars.is_debilitated(), "Mars is debilitated in Cancer");
    assert_eq!(mars.house, 9, "Mars should be in 9th house");

    println!(
        "  [PASS] Mars: {} at {:.1} deg (house {}, debilitated)",
        mars.sign.as_str(),
        mars.degree,
        mars.house
    );
}

// ---------------------------------------------------------------------------
// FAPI-097 Tests: Houses
// ---------------------------------------------------------------------------

#[test]
fn fapi097_twelve_houses_from_scorpio() {
    let chart = shesh_birth_chart();

    assert_eq!(chart.houses.len(), 12, "Should have 12 houses");
    assert_eq!(
        chart.houses[0].sign,
        ZodiacSign::Scorpio,
        "1st house should be Scorpio"
    );
    assert_eq!(
        chart.houses[1].sign,
        ZodiacSign::Sagittarius,
        "2nd house should be Sagittarius"
    );
    assert_eq!(
        chart.houses[2].sign,
        ZodiacSign::Capricorn,
        "3rd house should be Capricorn"
    );

    println!("  [PASS] 12 whole-sign houses starting from Scorpio");
}

// ---------------------------------------------------------------------------
// FAPI-097 Tests: Cross-Consistency
// ---------------------------------------------------------------------------

#[test]
fn fapi097_panchang_chart_moon_consistency() {
    let panchang = shesh_panchang();
    let chart = shesh_birth_chart();

    // Moon nakshatra should match
    assert_eq!(
        panchang.nakshatra.name(),
        chart.moon.nakshatra.as_str(),
        "Panchang nakshatra ({}) should match chart moon nakshatra ({})",
        panchang.nakshatra.name(),
        chart.moon.nakshatra
    );

    // Moon sign should be Virgo in both
    assert_eq!(
        panchang.planets.moon.sign, "Virgo",
        "Panchang moon sign should be Virgo"
    );
    assert_eq!(
        chart.moon.sign,
        ZodiacSign::Virgo,
        "Chart moon sign should be Virgo"
    );

    println!(
        "  [PASS] Panchang <-> Chart consistency: Moon in {} {} (pada {})",
        chart.moon.sign.as_str(),
        chart.moon.nakshatra,
        chart.moon.pada
    );
}

#[test]
fn fapi097_dasha_chart_consistency() {
    let dasha = shesh_vimshottari_dasha();
    let chart = shesh_birth_chart();

    // Moon nakshatra in dasha should match chart
    assert_eq!(
        dasha.moon_nakshatra,
        chart.moon.nakshatra,
        "Dasha moon nakshatra ({}) should match chart ({})",
        dasha.moon_nakshatra,
        chart.moon.nakshatra
    );

    // Nakshatra ruler should be Sun (Uttara Phalguni -> Sun)
    assert_eq!(
        dasha.balance.planet,
        DashaPlanet::Sun,
        "Dasha balance planet should be Sun"
    );

    println!(
        "  [PASS] Dasha <-> Chart consistency: {} -> {} -> {} Mahadasha",
        chart.moon.nakshatra,
        dasha.balance.planet.as_str(),
        dasha.mahadashas[0].planet.as_str()
    );
}

#[test]
fn fapi097_all_three_mocks_consistent() {
    let panchang = shesh_panchang();
    let chart = shesh_birth_chart();
    let dasha = shesh_vimshottari_dasha();

    // Moon nakshatra consistency across all three
    assert_eq!(panchang.nakshatra.name(), "Uttara Phalguni");
    assert_eq!(chart.moon.nakshatra, "Uttara Phalguni");
    assert_eq!(dasha.moon_nakshatra, "Uttara Phalguni");

    // Moon longitude consistency
    let panchang_moon_lon = panchang.planets.moon.longitude;
    let dasha_moon_lon = dasha.moon_longitude;
    assert!(
        (panchang_moon_lon - dasha_moon_lon).abs() < 1.0,
        "Moon longitude should be consistent: panchang={:.1}, dasha={:.1}",
        panchang_moon_lon,
        dasha_moon_lon
    );

    println!("  [PASS] All 3 mocks (panchang, chart, dasha) are internally consistent");
}

// ---------------------------------------------------------------------------
// FAPI-097 Tests: MockApiClient
// ---------------------------------------------------------------------------

#[test]
fn fapi097_mock_api_client_returns_correct_profile() {
    let client = MockApiClient::new();

    let chart = client.get_birth_chart();
    assert_eq!(chart.ascendant.sign, EXPECTED_ASCENDANT);
    assert_eq!(chart.moon.sign, EXPECTED_MOON_SIGN);
    assert_eq!(chart.moon.nakshatra, EXPECTED_MOON_NAKSHATRA);

    let dasha = client.get_vimshottari_dasha();
    assert_eq!(dasha.balance.planet, EXPECTED_BIRTH_DASHA_PLANET);
    assert_eq!(dasha.mahadashas[0].start_date, EXPECTED_SUN_MD_START);

    assert_eq!(client.call_count(), 2);

    println!("  [PASS] MockApiClient returns correct profile (2 calls tracked)");
}

#[test]
fn fapi097_mock_responses_json_well_formed() {
    // Verify all JSON mock responses are valid
    let panchang_val = MockResponses::panchang_response();
    assert!(panchang_val.is_object(), "Panchang response should be a JSON object");

    let dasha_val = MockResponses::vimshottari_response();
    assert!(dasha_val.is_object(), "Dasha response should be a JSON object");

    let chart_val = MockResponses::birth_chart_response();
    assert!(chart_val.is_object(), "Chart response should be a JSON object");

    let navamsa_val = MockResponses::navamsa_response();
    assert!(navamsa_val.is_object(), "Navamsa response should be a JSON object");

    // Verify string versions are non-empty
    assert!(!MockResponses::panchang_json().is_empty());
    assert!(!MockResponses::vimshottari_json().is_empty());
    assert!(!MockResponses::birth_chart_json().is_empty());
    assert!(!MockResponses::navamsa_json().is_empty());

    println!("  [PASS] All 4 MockResponses produce valid JSON");
}

// ---------------------------------------------------------------------------
// FAPI-097 Tests: Navamsa (D9) chart
// ---------------------------------------------------------------------------

#[test]
fn fapi097_navamsa_chart_has_all_planets() {
    let navamsa = shesh_navamsa_chart();

    assert_eq!(
        navamsa.navamsa_positions.len(),
        9,
        "Navamsa should have 9 planet positions"
    );

    assert_eq!(
        navamsa.d9_lagna,
        ZodiacSign::Cancer,
        "D9 Lagna should be Cancer"
    );

    println!(
        "  [PASS] Navamsa chart: {} planets, D9 Lagna: {}",
        navamsa.navamsa_positions.len(),
        navamsa.d9_lagna.as_str()
    );
}

#[test]
fn fapi097_navamsa_native_info_matches() {
    let navamsa = shesh_navamsa_chart();

    assert_eq!(navamsa.source.birth_date, "1991-09-14");
    assert_eq!(navamsa.source.birth_time, "09:30:00");
    assert!((navamsa.source.latitude - SHESH_LAT).abs() < 0.001);
    assert!((navamsa.source.longitude - SHESH_LNG).abs() < 0.001);

    println!("  [PASS] Navamsa native info matches birth data");
}

// ---------------------------------------------------------------------------
// Complete validation report
// ---------------------------------------------------------------------------

#[test]
fn fapi097_complete_shesh_profile_report() {
    let chart = shesh_birth_chart();
    let panchang = shesh_panchang();
    let dasha = shesh_vimshottari_dasha();
    let navamsa = shesh_navamsa_chart();

    println!("\n========================================");
    println!("  FAPI-097: SHESH PROFILE VALIDATION");
    println!("========================================");
    println!("  Birth: {}-{:02}-{:02} {:02}:{:02}:{:02} IST",
        SHESH_YEAR, SHESH_MONTH, SHESH_DAY, SHESH_HOUR, SHESH_MINUTE, SHESH_SECOND);
    println!("  Location: Bangalore ({}, {})", SHESH_LAT, SHESH_LNG);
    println!("  Timezone: IST (UTC+{:.1})", SHESH_TZONE);
    println!("----------------------------------------");

    // Profile attributes
    let asc_ok = chart.ascendant.sign == EXPECTED_ASCENDANT;
    let moon_ok = chart.moon.sign == EXPECTED_MOON_SIGN;
    let nak_ok = chart.moon.nakshatra == EXPECTED_MOON_NAKSHATRA;
    let dasha_ok = dasha.balance.planet == EXPECTED_BIRTH_DASHA_PLANET;
    let start_ok = dasha.mahadashas[0].start_date == EXPECTED_SUN_MD_START;

    println!("  PROFILE ATTRIBUTES:");
    println!("    Ascendant:    {} - {}", chart.ascendant.sign.as_str(), if asc_ok { "EXACT MATCH" } else { "MISMATCH" });
    println!("    Moon Sign:    {} - {}", chart.moon.sign.as_str(), if moon_ok { "EXACT MATCH" } else { "MISMATCH" });
    println!("    Nakshatra:    {} pada {} - {}", chart.moon.nakshatra, chart.moon.pada, if nak_ok { "EXACT MATCH" } else { "MISMATCH" });
    println!("    Birth Dasha:  {} - {}", dasha.balance.planet.as_str(), if dasha_ok { "EXACT MATCH" } else { "MISMATCH" });
    println!("    MD Start:     {} - {}", dasha.mahadashas[0].start_date, if start_ok { "EXACT MATCH" } else { "MISMATCH" });

    println!("----------------------------------------");
    println!("  PLANETARY DIGNITIES:");
    for planet in &chart.planets {
        let mut dignity = String::new();
        if planet.in_own_sign() {
            dignity.push_str("own sign");
        }
        if planet.is_exalted() {
            dignity.push_str("exalted");
        }
        if planet.is_debilitated() {
            dignity.push_str("debilitated");
        }
        if planet.is_retrograde {
            if !dignity.is_empty() {
                dignity.push_str(", ");
            }
            dignity.push_str("retrograde");
        }
        if dignity.is_empty() {
            dignity = "neutral".to_string();
        }

        println!("    {}: {} {:.1} deg (house {}) [{}]",
            planet.name, planet.sign.as_str(), planet.degree, planet.house, dignity);
    }

    println!("----------------------------------------");
    println!("  CROSS-VALIDATION:");
    println!("    Panchang Nakshatra: {}", panchang.nakshatra.name());
    println!("    Chart Moon Nakshatra: {}", chart.moon.nakshatra);
    println!("    Dasha Moon Nakshatra: {}", dasha.moon_nakshatra);
    println!("    D9 Lagna: {}", navamsa.d9_lagna.as_str());

    let cross_ok = panchang.nakshatra.name() == chart.moon.nakshatra
        && chart.moon.nakshatra == dasha.moon_nakshatra;
    println!("    Cross-consistency: {}", if cross_ok { "PASS" } else { "FAIL" });

    println!("========================================");
    let all_ok = asc_ok && moon_ok && nak_ok && dasha_ok && start_ok && cross_ok;
    if all_ok {
        println!("  RESULT: ALL PROFILE ATTRIBUTES MATCH");
    } else {
        println!("  RESULT: PROFILE VALIDATION FAILED");
    }
    println!("========================================\n");

    assert!(asc_ok, "Ascendant should be Scorpio");
    assert!(moon_ok, "Moon should be in Virgo");
    assert!(nak_ok, "Nakshatra should be Uttara Phalguni");
    assert!(dasha_ok, "Birth dasha should be Sun");
    assert!(start_ok, "Sun MD should start on 1991-09-14");
    assert!(cross_ok, "All three mocks should be cross-consistent");
}
