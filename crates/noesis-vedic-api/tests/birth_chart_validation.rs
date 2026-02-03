use noesis_vedic_api::birth_chart::dignities::{chart_dignities, DignityStatus};
use noesis_vedic_api::birth_chart::status::{compute_statuses};
use noesis_vedic_api::birth_chart::aspects::{calculate_aspects, AspectType};
use noesis_vedic_api::chart::{
    BirthChart, NativeInfo, PlanetPosition, HousePosition, HouseType,
    AscendantInfo, MoonInfo, SpecialPoints, ZodiacSign,
};

fn planet(name: &str, sign: ZodiacSign, degree: f64, house: u8) -> PlanetPosition {
    PlanetPosition {
        name: name.to_string(),
        longitude: 0.0,
        sign,
        degree,
        minutes: 0.0,
        house,
        is_retrograde: false,
        is_combust: false,
        nakshatra: "".to_string(),
        pada: 1,
        speed: 0.0,
        latitude: 0.0,
    }
}

fn sample_chart() -> BirthChart {
    let planets = vec![
        planet("Sun", ZodiacSign::Aries, 10.0, 1),
        planet("Moon", ZodiacSign::Leo, 10.0, 5),  // 120° from Sun (Trine)
        planet("Mercury", ZodiacSign::Aries, 12.0, 1),
        planet("Venus", ZodiacSign::Taurus, 20.0, 2),
    ];

    BirthChart {
        native: NativeInfo {
            birth_date: "1991-08-13".to_string(),
            birth_time: "13:31".to_string(),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: 5.5,
        },
        ayanamsa: 24.0,
        house_system: "placidus".to_string(),
        planets,
        houses: vec![HousePosition {
            number: 1,
            sign: ZodiacSign::Aries,
            cusp: 0.0,
            degree: 0.0,
            house_type: HouseType::Dharma,
            is_kendra: true,
            is_panapara: false,
            is_apoklima: false,
        }],
        ascendant: AscendantInfo {
            sign: ZodiacSign::Aries,
            degree: 0.0,
            nakshatra: "".to_string(),
            pada: 1,
        },
        moon: MoonInfo {
            sign: ZodiacSign::Leo,
            degree: 0.0,
            nakshatra: "".to_string(),
            pada: 1,
            rashi_lord: "Sun".to_string(),
        },
        special_points: SpecialPoints {
            lagna: 0.0,
            midheaven: None,
            part_of_fortune: None,
        },
        notes: vec![],
    }
}

#[test]
fn test_chart_dignities() {
    let chart = sample_chart();
    let dignities = chart_dignities(&chart);
    let sun = dignities.iter().find(|d| d.planet == "Sun").expect("sun");
    assert_eq!(sun.status, DignityStatus::Exalted);
}

#[test]
fn test_chart_statuses_combust() {
    let chart = sample_chart();
    let statuses = compute_statuses(&chart);
    let mercury = statuses.iter().find(|s| s.planet == "Mercury").expect("mercury");
    assert!(mercury.is_combust);
}

#[test]
fn test_chart_aspects() {
    let chart = sample_chart();
    let aspects = calculate_aspects(&chart);
    let has_trine = aspects.iter().any(|a| {
        a.aspect_type == AspectType::Trine &&
        ((a.planet_a == "Sun" && a.planet_b == "Moon") || (a.planet_a == "Moon" && a.planet_b == "Sun"))
    });
    assert!(has_trine, "expected Sun-Moon trine");
}
