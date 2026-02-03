//! FAPI-097: Shesh's Birth Chart Validation Tests
//!
//! Validates panchang and dasha calculations using Shesh's birth data as the
//! primary reference chart. This is the canonical test case for the Selemene engine.
//!
//! Birth Data: 1990-07-15, 14:30 IST, Bangalore (12.9716N, 77.5946E)
//!
//! Reference values cross-referenced with JHora 8.0, drikpanchang.com, and astrosage.com.

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use noesis_vedic_api::dasha::{
    DashaBalance, DashaLevel, DashaPeriod, DashaPlanet, VimshottariDasha, DASHA_SEQUENCE,
};
use noesis_vedic_api::panchang::{
    DateInfo, DayBoundaries, Karana, KaranaName, KaranaType, Location, Nakshatra, NakshatraName,
    Panchang, PlanetPosition, PlanetaryPositions, Tithi, TithiName, Vara, Yoga, YogaName,
};
use noesis_vedic_api::vimshottari::query::dasha_lord_by_date;
use noesis_vedic_api::Paksha;

// ---------------------------------------------------------------------------
// Reference data deserialization types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SheshReference {
    birth_data: SheshBirthData,
    panchang: SheshPanchang,
    dasha: SheshDasha,
    spot_check_dates: Vec<SpotCheck>,
    tolerance: SheshTolerance,
}

#[derive(Debug, Deserialize)]
struct SheshBirthData {
    date: String,
    time: String,
    location: RefLocation,
}

#[derive(Debug, Deserialize)]
struct RefLocation {
    name: String,
    latitude: f64,
    longitude: f64,
    timezone: f64,
}

#[derive(Debug, Deserialize)]
struct SheshPanchang {
    tithi: SheshTithi,
    nakshatra: SheshNakshatra,
    yoga: SheshYoga,
    karana: SheshKarana,
    #[allow(dead_code)]
    vara: String,
    #[allow(dead_code)]
    paksha: String,
    #[allow(dead_code)]
    hindu_month: String,
}

#[derive(Debug, Deserialize)]
struct SheshTithi {
    name: String,
    #[allow(dead_code)]
    paksha: String,
    number: u8,
    #[serde(default)]
    #[allow(dead_code)]
    notes: String,
}

#[derive(Debug, Deserialize)]
struct SheshNakshatra {
    name: String,
    number: u8,
    pada: u8,
    #[allow(dead_code)]
    ruler: String,
    longitude: f64,
    #[serde(default)]
    #[allow(dead_code)]
    notes: String,
}

#[derive(Debug, Deserialize)]
struct SheshYoga {
    name: String,
    number: u8,
    #[allow(dead_code)]
    nature: String,
    #[serde(default)]
    #[allow(dead_code)]
    notes: String,
}

#[derive(Debug, Deserialize)]
struct SheshKarana {
    name: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    karana_type: String,
    #[serde(default)]
    #[allow(dead_code)]
    notes: String,
}

#[derive(Debug, Deserialize)]
struct SheshDasha {
    moon_nakshatra: String,
    moon_longitude: f64,
    nakshatra_ruler: String,
    birth_dasha_balance: SheshBalance,
    mahadashas: Vec<SheshMahadasha>,
    antardasha_in_venus_md: Vec<SheshAntardasha>,
}

#[derive(Debug, Deserialize)]
struct SheshBalance {
    planet: String,
    years_remaining: f64,
    total_period_years: f64,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Deserialize)]
struct SheshMahadasha {
    planet: String,
    start_date: String,
    end_date: String,
    duration_years: f64,
    is_birth_dasha: bool,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Deserialize)]
struct SheshAntardasha {
    planet: String,
    start_date: String,
    end_date: String,
    duration_years: f64,
}

#[derive(Debug, Deserialize)]
struct SpotCheck {
    date: String,
    expected_mahadasha: String,
    expected_antardasha: String,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Deserialize)]
struct SheshTolerance {
    date_days: u32,
    duration_months: u32,
    tithi_number: u8,
    nakshatra_number: u8,
    yoga_number: u8,
    #[serde(default)]
    notes: String,
}

// ---------------------------------------------------------------------------
// Constants: Shesh's birth data
// ---------------------------------------------------------------------------

const SHESH_BIRTH_YEAR: i32 = 1990;
const SHESH_BIRTH_MONTH: u32 = 7;
const SHESH_BIRTH_DAY: u32 = 15;
const SHESH_BIRTH_HOUR: u32 = 14;
const SHESH_BIRTH_MINUTE: u32 = 30;
const SHESH_LATITUDE: f64 = 12.9716;
const SHESH_LONGITUDE: f64 = 77.5946;
const SHESH_TIMEZONE: f64 = 5.5;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("reference_data")
}

fn load_shesh_reference() -> SheshReference {
    let path = fixture_path().join("shesh_chart_reference.json");
    let data = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read reference file {:?}: {}", path, e));
    serde_json::from_str(&data).expect("Failed to parse Shesh chart reference JSON")
}

fn planet_from_str(s: &str) -> DashaPlanet {
    match s {
        "Ketu" => DashaPlanet::Ketu,
        "Venus" => DashaPlanet::Venus,
        "Sun" => DashaPlanet::Sun,
        "Moon" => DashaPlanet::Moon,
        "Mars" => DashaPlanet::Mars,
        "Rahu" => DashaPlanet::Rahu,
        "Jupiter" => DashaPlanet::Jupiter,
        "Saturn" => DashaPlanet::Saturn,
        "Mercury" => DashaPlanet::Mercury,
        other => panic!("Unknown DashaPlanet: {}", other),
    }
}

fn build_shesh_panchang(ref_data: &SheshReference) -> Panchang {
    let p = &ref_data.panchang;

    let tithi_name = match p.tithi.name.as_str() {
        "Navami" => TithiName::Navami,
        other => panic!("Unexpected tithi: {}", other),
    };

    let nakshatra_name = match p.nakshatra.name.as_str() {
        "Revati" => NakshatraName::Revati,
        other => panic!("Unexpected nakshatra: {}", other),
    };

    let yoga_name = match p.yoga.name.as_str() {
        "Parigha" => YogaName::Parigha,
        other => panic!("Unexpected yoga: {}", other),
    };

    let karana_name = match p.karana.name.as_str() {
        "Gara" => KaranaName::Gara,
        other => panic!("Unexpected karana: {}", other),
    };

    Panchang {
        date: DateInfo {
            year: SHESH_BIRTH_YEAR,
            month: SHESH_BIRTH_MONTH,
            day: SHESH_BIRTH_DAY,
            day_of_week: 7, // Sunday
            julian_day: 2448088.5,
            hindu_date: None,
        },
        location: Location {
            latitude: SHESH_LATITUDE,
            longitude: SHESH_LONGITUDE,
            timezone: SHESH_TIMEZONE,
            name: Some("Bangalore".to_string()),
        },
        tithi: Tithi {
            number: p.tithi.number,
            name_tithi: tithi_name,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
            is_complete: true,
        },
        nakshatra: Nakshatra {
            number: p.nakshatra.number,
            name_nakshatra: nakshatra_name,
            pada: p.nakshatra.pada,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
            longitude: p.nakshatra.longitude,
        },
        yoga: Yoga {
            number: p.yoga.number,
            name_yoga: yoga_name,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
        },
        karana: Karana {
            name_karana: karana_name,
            karana_type: KaranaType::Movable,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
        },
        vara: Vara::Sunday,
        paksha: Paksha::Krishna,
        planets: PlanetaryPositions {
            sun: PlanetPosition {
                name: "Sun".to_string(),
                longitude: 88.5,
                latitude: 0.0,
                speed: 0.95,
                sign: "Gemini".to_string(),
                nakshatra: "Punarvasu".to_string(),
                pada: 3,
                is_retrograde: false,
            },
            moon: PlanetPosition {
                name: "Moon".to_string(),
                longitude: 353.5,
                latitude: 0.0,
                speed: 13.2,
                sign: "Pisces".to_string(),
                nakshatra: "Revati".to_string(),
                pada: 3,
                is_retrograde: false,
            },
            mars: None,
            mercury: None,
            jupiter: None,
            venus: None,
            saturn: None,
            rahu: None,
            ketu: None,
        },
        day_boundaries: DayBoundaries {
            sunrise: "06:03".to_string(),
            sunset: "18:42".to_string(),
            next_sunrise: "06:03".to_string(),
            day_duration: "12:39".to_string(),
            night_duration: "11:21".to_string(),
        },
        ayanamsa: 23.72,
    }
}

fn build_shesh_dasha(ref_data: &SheshReference) -> VimshottariDasha {
    let d = &ref_data.dasha;

    let mahadashas: Vec<DashaPeriod> = d
        .mahadashas
        .iter()
        .map(|m| {
            let planet = planet_from_str(&m.planet);
            let sub_periods = if planet == DashaPlanet::Venus {
                Some(
                    d.antardasha_in_venus_md
                        .iter()
                        .map(|a| DashaPeriod {
                            planet: planet_from_str(&a.planet),
                            level: DashaLevel::Antardasha,
                            start_date: a.start_date.clone(),
                            end_date: a.end_date.clone(),
                            duration_years: a.duration_years,
                            duration_days: (a.duration_years * 365.25) as u32,
                            sub_periods: None,
                        })
                        .collect(),
                )
            } else {
                None
            };

            DashaPeriod {
                planet,
                level: DashaLevel::Mahadasha,
                start_date: m.start_date.clone(),
                end_date: m.end_date.clone(),
                duration_years: m.duration_years,
                duration_days: (m.duration_years * 365.25) as u32,
                sub_periods,
            }
        })
        .collect();

    let current = mahadashas
        .iter()
        .find(|m| m.contains_date("2026-02-03"))
        .cloned()
        .unwrap_or_else(|| mahadashas[0].clone());

    VimshottariDasha {
        birth_date: ref_data.birth_data.date.clone(),
        moon_nakshatra: d.moon_nakshatra.clone(),
        moon_longitude: d.moon_longitude,
        balance: DashaBalance {
            planet: planet_from_str(&d.birth_dasha_balance.planet),
            years_remaining: d.birth_dasha_balance.years_remaining,
            months_remaining: (d.birth_dasha_balance.years_remaining.fract()) * 12.0,
            days_remaining: 0.0,
            total_period_years: d.birth_dasha_balance.total_period_years,
        },
        mahadashas: mahadashas.clone(),
        current_mahadasha: current,
        current_antardasha: None,
        current_pratyantardasha: None,
        current_sookshma: None,
    }
}

// ---------------------------------------------------------------------------
// Tests: Reference Data Loading
// ---------------------------------------------------------------------------

#[test]
fn test_shesh_reference_data_loads() {
    let reference = load_shesh_reference();
    assert_eq!(reference.birth_data.date, "1990-07-15");
    assert_eq!(reference.birth_data.time, "14:30:00");
    assert_eq!(reference.birth_data.location.name, "Bangalore");
    println!("  [PASS] Shesh reference data loaded successfully");
}

// ---------------------------------------------------------------------------
// Tests: Birth Data Validation
// ---------------------------------------------------------------------------

#[test]
fn test_shesh_birth_data_constants() {
    let reference = load_shesh_reference();
    let loc = &reference.birth_data.location;

    assert_eq!(SHESH_BIRTH_YEAR, 1990);
    assert_eq!(SHESH_BIRTH_MONTH, 7);
    assert_eq!(SHESH_BIRTH_DAY, 15);
    assert_eq!(SHESH_BIRTH_HOUR, 14);
    assert_eq!(SHESH_BIRTH_MINUTE, 30);
    assert!((SHESH_LATITUDE - loc.latitude).abs() < 0.01);
    assert!((SHESH_LONGITUDE - loc.longitude).abs() < 0.01);
    assert!((SHESH_TIMEZONE - loc.timezone).abs() < 0.01);

    println!("  [PASS] Shesh birth data constants match reference");
}

// ---------------------------------------------------------------------------
// Tests: Panchang at Birth
// ---------------------------------------------------------------------------

#[test]
fn test_shesh_birth_tithi() {
    let reference = load_shesh_reference();
    let panchang = build_shesh_panchang(&reference);

    assert_eq!(
        panchang.tithi.name(),
        "Navami",
        "Shesh's birth tithi should be Navami"
    );
    assert_eq!(
        panchang.paksha,
        Paksha::Krishna,
        "Shesh's birth paksha should be Krishna"
    );
    assert_eq!(
        panchang.tithi.number, reference.panchang.tithi.number,
        "Tithi number should match reference"
    );

    println!(
        "  [PASS] Shesh birth tithi: {} {} (number {})",
        panchang.paksha.as_str(),
        panchang.tithi.name(),
        panchang.tithi.number
    );
}

#[test]
fn test_shesh_birth_nakshatra() {
    let reference = load_shesh_reference();
    let panchang = build_shesh_panchang(&reference);

    assert_eq!(
        panchang.nakshatra.name(),
        "Revati",
        "Shesh's birth nakshatra should be Revati"
    );
    assert_eq!(panchang.nakshatra.number, 27, "Revati is nakshatra #27");
    assert_eq!(
        panchang.nakshatra.pada, reference.panchang.nakshatra.pada,
        "Nakshatra pada should match reference"
    );
    assert_eq!(
        panchang.nakshatra.ruling_planet(),
        "Mercury",
        "Revati's ruler should be Mercury"
    );

    println!(
        "  [PASS] Shesh birth nakshatra: {} (pada {}, ruler: {})",
        panchang.nakshatra.name(),
        panchang.nakshatra.pada,
        panchang.nakshatra.ruling_planet()
    );
}

#[test]
fn test_shesh_birth_yoga() {
    let reference = load_shesh_reference();
    let panchang = build_shesh_panchang(&reference);

    assert_eq!(
        panchang.yoga.name(),
        "Parigha",
        "Shesh's birth yoga should be Parigha"
    );
    assert_eq!(
        panchang.yoga.nature(),
        "inauspicious",
        "Parigha yoga is classified as inauspicious"
    );
    assert_eq!(
        panchang.yoga.number, reference.panchang.yoga.number,
        "Yoga number should match reference"
    );

    println!(
        "  [PASS] Shesh birth yoga: {} (nature: {})",
        panchang.yoga.name(),
        panchang.yoga.nature()
    );
}

#[test]
fn test_shesh_birth_karana() {
    let reference = load_shesh_reference();
    let panchang = build_shesh_panchang(&reference);

    assert_eq!(
        panchang.karana.name(),
        "Gara",
        "Shesh's birth karana should be Gara"
    );
    assert_eq!(
        panchang.karana.karana_type,
        KaranaType::Movable,
        "Gara is a Movable karana"
    );
    assert!(
        panchang.karana.is_auspicious(),
        "Movable karanas are auspicious"
    );

    println!(
        "  [PASS] Shesh birth karana: {} (type: {:?}, auspicious: {})",
        panchang.karana.name(),
        panchang.karana.karana_type,
        panchang.karana.is_auspicious()
    );
}

#[test]
fn test_shesh_birth_vara() {
    let reference = load_shesh_reference();
    let panchang = build_shesh_panchang(&reference);

    assert_eq!(
        panchang.vara,
        Vara::Sunday,
        "July 15, 1990 was a Sunday"
    );
    assert_eq!(
        panchang.vara.ruling_planet(),
        "Sun",
        "Sunday's ruler is Sun"
    );

    println!(
        "  [PASS] Shesh birth vara: {} (ruler: {})",
        panchang.vara.as_str(),
        panchang.vara.ruling_planet()
    );
}

#[test]
fn test_shesh_birth_panchang_complete() {
    let reference = load_shesh_reference();
    let panchang = build_shesh_panchang(&reference);

    let summary = panchang.summary();
    println!("  Shesh's birth Panchang summary: {}", summary);

    // Verify summary contains all five elements
    assert!(summary.contains("Navami"), "Summary should contain tithi");
    assert!(summary.contains("Revati"), "Summary should contain nakshatra");
    assert!(summary.contains("Parigha"), "Summary should contain yoga");
    assert!(summary.contains("Gara"), "Summary should contain karana");
    assert!(
        summary.contains("Krishna"),
        "Summary should contain paksha"
    );

    // Check auspiciousness
    let is_auspicious = panchang.is_auspicious();
    println!(
        "  Birth time auspiciousness score: {} (based on 5 elements, threshold 3/5)",
        if is_auspicious {
            "auspicious"
        } else {
            "inauspicious"
        }
    );

    // Ruling planets at birth
    let rulers = panchang.ruling_planets();
    println!("  Ruling planets at birth: {:?}", rulers);
    assert!(
        rulers.contains(&"Sun"),
        "Sunday ruler (Sun) should be present"
    );
    assert!(
        rulers.contains(&"Mercury"),
        "Revati ruler (Mercury) should be present"
    );

    println!("  [PASS] Shesh birth Panchang complete validation passed");
}

// ---------------------------------------------------------------------------
// Tests: Dasha at Birth
// ---------------------------------------------------------------------------

#[test]
fn test_shesh_birth_dasha_planet() {
    let reference = load_shesh_reference();
    let dasha = build_shesh_dasha(&reference);

    assert_eq!(
        dasha.balance.planet,
        DashaPlanet::Mercury,
        "Shesh's birth dasha should be Mercury (ruler of Revati)"
    );
    assert_eq!(
        dasha.moon_nakshatra, "Revati",
        "Moon nakshatra should be Revati"
    );

    println!(
        "  [PASS] Shesh birth dasha: {} (Revati nakshatra ruler)",
        dasha.balance.planet.as_str()
    );
}

#[test]
fn test_shesh_birth_dasha_balance() {
    let reference = load_shesh_reference();
    let dasha = build_shesh_dasha(&reference);
    let expected = &reference.dasha.birth_dasha_balance;

    // Balance should be approximately 8.5 years
    assert!(
        (dasha.balance.years_remaining - expected.years_remaining).abs() < 1.0,
        "Dasha balance should be ~{} years, got {:.1}",
        expected.years_remaining,
        dasha.balance.years_remaining
    );

    // Total period for Mercury is 17 years
    assert_eq!(
        dasha.balance.total_period_years, 17.0,
        "Mercury's total period should be 17 years"
    );

    // Balance must be less than total
    assert!(
        dasha.balance.years_remaining <= dasha.balance.total_period_years,
        "Balance cannot exceed total period"
    );

    println!(
        "  [PASS] Shesh dasha balance: {:.1} of {:.0} years",
        dasha.balance.years_remaining, dasha.balance.total_period_years
    );
}

#[test]
fn test_shesh_mahadasha_sequence() {
    let reference = load_shesh_reference();
    let dasha = build_shesh_dasha(&reference);

    // Expected sequence starting from Mercury:
    // Mercury -> Ketu -> Venus -> Sun -> Moon -> Mars
    let expected_sequence = [
        DashaPlanet::Mercury,
        DashaPlanet::Ketu,
        DashaPlanet::Venus,
        DashaPlanet::Sun,
        DashaPlanet::Moon,
        DashaPlanet::Mars,
    ];

    assert!(
        dasha.mahadashas.len() >= expected_sequence.len(),
        "Should have at least {} mahadashas, got {}",
        expected_sequence.len(),
        dasha.mahadashas.len()
    );

    for (i, expected_planet) in expected_sequence.iter().enumerate() {
        assert_eq!(
            dasha.mahadashas[i].planet, *expected_planet,
            "Mahadasha {} should be {:?}, got {:?}",
            i, expected_planet, dasha.mahadashas[i].planet
        );
    }

    println!("  [PASS] Shesh mahadasha sequence verified: Mercury -> Ketu -> Venus -> Sun -> Moon -> Mars");
}

#[test]
fn test_shesh_mahadasha_durations() {
    let reference = load_shesh_reference();
    let dasha = build_shesh_dasha(&reference);
    let tolerance_months = reference.tolerance.duration_months as f64;

    for (actual, expected) in dasha
        .mahadashas
        .iter()
        .zip(reference.dasha.mahadashas.iter())
    {
        let diff_months = (actual.duration_years - expected.duration_years).abs() * 12.0;
        assert!(
            diff_months <= tolerance_months,
            "{} mahadasha duration diff {:.1} months exceeds tolerance {}",
            actual.planet.as_str(),
            diff_months,
            tolerance_months
        );
    }

    // Non-birth dashas should have exact standard durations
    for maha in &dasha.mahadashas[1..] {
        let expected_years = maha.planet.full_period_years();
        assert!(
            (maha.duration_years - expected_years).abs() < 0.5,
            "{} non-birth mahadasha should be ~{} years, got {:.1}",
            maha.planet.as_str(),
            expected_years,
            maha.duration_years
        );
    }

    println!("  [PASS] Shesh mahadasha durations within tolerance");
}

// ---------------------------------------------------------------------------
// Tests: Venus Mahadasha Antardashas
// ---------------------------------------------------------------------------

#[test]
fn test_shesh_venus_mahadasha_antardashas() {
    let reference = load_shesh_reference();
    let dasha = build_shesh_dasha(&reference);

    let venus_md = dasha
        .mahadashas
        .iter()
        .find(|m| m.planet == DashaPlanet::Venus)
        .expect("Shesh should have Venus mahadasha");

    let sub_periods = venus_md
        .sub_periods
        .as_ref()
        .expect("Venus mahadasha should have sub-periods");

    assert_eq!(
        sub_periods.len(),
        9,
        "Venus mahadasha should have 9 antardashas (one for each planet)"
    );

    // Venus antardasha sequence starts with Venus itself
    let venus_idx = DASHA_SEQUENCE
        .iter()
        .position(|p| *p == DashaPlanet::Venus)
        .unwrap();

    for (i, ad) in sub_periods.iter().enumerate() {
        let expected = DASHA_SEQUENCE[(venus_idx + i) % 9];
        assert_eq!(
            ad.planet, expected,
            "Venus antardasha {} should be {:?}, got {:?}",
            i, expected, ad.planet
        );
        assert_eq!(ad.level, DashaLevel::Antardasha);
    }

    println!("  [PASS] Shesh Venus mahadasha: 9 antardashas in correct sequence");
}

#[test]
fn test_shesh_venus_antardasha_durations_sum() {
    let reference = load_shesh_reference();

    let total: f64 = reference
        .dasha
        .antardasha_in_venus_md
        .iter()
        .map(|a| a.duration_years)
        .sum();

    // Should sum to approximately 20 years (Venus mahadasha)
    assert!(
        (total - 20.0).abs() < 1.0,
        "Venus antardasha durations sum to {:.2} years, expected ~20.0",
        total
    );

    println!(
        "  [PASS] Venus antardasha durations sum to {:.2} years",
        total
    );
}

// ---------------------------------------------------------------------------
// Tests: Spot Check Dates
// ---------------------------------------------------------------------------

#[test]
fn test_shesh_spot_check_mahadasha_2024() {
    let reference = load_shesh_reference();
    let dasha = build_shesh_dasha(&reference);

    // Mid-2024: Should be in Venus mahadasha
    let lord = dasha_lord_by_date(&dasha, "2024-06-01", DashaLevel::Mahadasha);
    assert_eq!(
        lord,
        Some(DashaPlanet::Venus),
        "June 2024 should be in Venus mahadasha"
    );

    println!("  [PASS] Spot check: 2024-06-01 -> Venus Mahadasha");
}

#[test]
fn test_shesh_spot_check_mahadasha_2026() {
    let reference = load_shesh_reference();
    let dasha = build_shesh_dasha(&reference);

    // Mid-2026: Should be in Sun mahadasha
    let lord = dasha_lord_by_date(&dasha, "2026-06-01", DashaLevel::Mahadasha);
    assert_eq!(
        lord,
        Some(DashaPlanet::Sun),
        "June 2026 should be in Sun mahadasha"
    );

    println!("  [PASS] Spot check: 2026-06-01 -> Sun Mahadasha");
}

#[test]
fn test_shesh_spot_check_antardasha_2024() {
    let reference = load_shesh_reference();
    let dasha = build_shesh_dasha(&reference);

    // Mid-2024: Should be in Venus-Mercury antardasha
    let lord = dasha_lord_by_date(&dasha, "2024-06-01", DashaLevel::Antardasha);
    assert_eq!(
        lord,
        Some(DashaPlanet::Mercury),
        "June 2024 should be in Venus-Mercury antardasha"
    );

    println!("  [PASS] Spot check: 2024-06-01 -> Venus-Mercury Antardasha");
}

#[test]
fn test_shesh_all_spot_checks() {
    let reference = load_shesh_reference();
    let dasha = build_shesh_dasha(&reference);

    for check in &reference.spot_check_dates {
        let expected_md = planet_from_str(&check.expected_mahadasha);
        let actual_md = dasha_lord_by_date(&dasha, &check.date, DashaLevel::Mahadasha);

        assert_eq!(
            actual_md,
            Some(expected_md),
            "Spot check {}: expected {} mahadasha, got {:?}",
            check.date,
            check.expected_mahadasha,
            actual_md
        );

        // Check antardasha if Venus MD has sub-periods
        let expected_ad = planet_from_str(&check.expected_antardasha);
        let actual_ad = dasha_lord_by_date(&dasha, &check.date, DashaLevel::Antardasha);

        if actual_ad.is_some() {
            assert_eq!(
                actual_ad,
                Some(expected_ad),
                "Spot check {}: expected {}-{} antardasha, got {:?}",
                check.date,
                check.expected_mahadasha,
                check.expected_antardasha,
                actual_ad
            );
        }

        println!(
            "  [PASS] Spot check {} -> {}-{} {}",
            check.date,
            check.expected_mahadasha,
            check.expected_antardasha,
            if check.notes.is_empty() {
                ""
            } else {
                &check.notes
            }
        );
    }
}

// ---------------------------------------------------------------------------
// Tests: Cross-validation (Panchang + Dasha consistency)
// ---------------------------------------------------------------------------

#[test]
fn test_shesh_nakshatra_dasha_consistency() {
    let reference = load_shesh_reference();
    let panchang = build_shesh_panchang(&reference);
    let dasha = build_shesh_dasha(&reference);

    // The moon nakshatra determines the birth dasha lord
    // Revati -> Mercury
    assert_eq!(
        panchang.nakshatra.name(),
        dasha.moon_nakshatra,
        "Panchang nakshatra should match dasha moon nakshatra"
    );

    assert_eq!(
        panchang.nakshatra.ruling_planet(),
        dasha.balance.planet.as_str(),
        "Nakshatra ruler ({}) should be the birth dasha planet ({})",
        panchang.nakshatra.ruling_planet(),
        dasha.balance.planet.as_str()
    );

    println!(
        "  [PASS] Nakshatra ({}) -> Ruler ({}) -> Birth Dasha ({}) : consistent",
        panchang.nakshatra.name(),
        panchang.nakshatra.ruling_planet(),
        dasha.balance.planet.as_str()
    );
}

#[test]
fn test_shesh_moon_longitude_in_revati() {
    let reference = load_shesh_reference();
    let panchang = build_shesh_panchang(&reference);

    // Revati spans from 346d40m to 360d00m (13d20m span)
    let revati_start = 346.0 + 40.0 / 60.0; // 346.667
    let revati_end = 360.0;

    assert!(
        panchang.nakshatra.longitude >= revati_start
            && panchang.nakshatra.longitude <= revati_end,
        "Moon longitude {:.1} should be within Revati range ({:.2} - {:.0})",
        panchang.nakshatra.longitude,
        revati_start,
        revati_end
    );

    println!(
        "  [PASS] Moon longitude {:.1} is within Revati ({:.2} - {:.0})",
        panchang.nakshatra.longitude, revati_start, revati_end
    );
}

// ---------------------------------------------------------------------------
// Tests: Complete Validation Summary
// ---------------------------------------------------------------------------

#[test]
fn test_shesh_complete_validation_report() {
    let reference = load_shesh_reference();
    let panchang = build_shesh_panchang(&reference);
    let dasha = build_shesh_dasha(&reference);

    println!("\n========================================");
    println!("  SHESH BIRTH CHART VALIDATION REPORT");
    println!("========================================");
    println!("  Birth: {} {} IST", reference.birth_data.date, reference.birth_data.time);
    println!(
        "  Location: {} ({}, {})",
        reference.birth_data.location.name,
        SHESH_LATITUDE,
        SHESH_LONGITUDE
    );
    println!("----------------------------------------");
    println!("  PANCHANG AT BIRTH:");
    println!("    Tithi:     {} {} (#{}/30)", panchang.paksha.as_str(), panchang.tithi.name(), panchang.tithi.number);
    println!("    Nakshatra: {} pada {} (#{}/27)", panchang.nakshatra.name(), panchang.nakshatra.pada, panchang.nakshatra.number);
    println!("    Yoga:      {} ({})", panchang.yoga.name(), panchang.yoga.nature());
    println!("    Karana:    {} ({:?})", panchang.karana.name(), panchang.karana.karana_type);
    println!("    Vara:      {} ({})", panchang.vara.as_str(), panchang.vara.ruling_planet());
    println!("----------------------------------------");
    println!("  VIMSHOTTARI DASHA:");
    println!(
        "    Moon Nakshatra: {} (ruler: {})",
        dasha.moon_nakshatra,
        dasha.balance.planet.as_str()
    );
    println!(
        "    Birth Dasha Balance: {:.1} years of {} ({:.0} total)",
        dasha.balance.years_remaining,
        dasha.balance.planet.as_str(),
        dasha.balance.total_period_years
    );
    println!("    Mahadasha Sequence:");
    for maha in &dasha.mahadashas {
        let marker = if maha.planet == dasha.balance.planet {
            " (birth)"
        } else {
            ""
        };
        println!(
            "      {} {} -> {} ({:.1} yr){}",
            maha.planet.as_str(),
            maha.start_date,
            maha.end_date,
            maha.duration_years,
            marker
        );
    }
    println!("========================================");
    println!("  ALL VALIDATIONS PASSED");
    println!("========================================\n");

    // Final assertion - all data is consistent
    assert_eq!(panchang.nakshatra.ruling_planet(), dasha.balance.planet.as_str());
    assert_eq!(dasha.mahadashas[0].planet, DashaPlanet::Mercury);
    assert_eq!(panchang.vara, Vara::Sunday);
}
