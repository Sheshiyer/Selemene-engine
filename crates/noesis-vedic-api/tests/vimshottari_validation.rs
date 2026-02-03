use noesis_vedic_api::dasha::{DashaLevel, DashaPeriod, DashaPlanet, DashaBalance, VimshottariDasha};
use noesis_vedic_api::vimshottari::query::{dasha_lord_by_date, dasha_period_by_date};

fn period(planet: DashaPlanet, level: DashaLevel, start: &str, end: &str, subs: Option<Vec<DashaPeriod>>) -> DashaPeriod {
    DashaPeriod {
        planet,
        level,
        start_date: start.to_string(),
        end_date: end.to_string(),
        duration_years: 1.0,
        duration_days: 365,
        sub_periods: subs,
    }
}

fn sample_dasha() -> VimshottariDasha {
    let antar = period(
        DashaPlanet::Rahu,
        DashaLevel::Antardasha,
        "2024-01-01",
        "2024-12-31",
        None,
    );

    let maha = period(
        DashaPlanet::Mars,
        DashaLevel::Mahadasha,
        "2019-09-14",
        "2026-09-14",
        Some(vec![antar.clone()]),
    );

    VimshottariDasha {
        birth_date: "1991-08-13".to_string(),
        moon_nakshatra: "Uttara Phalguni".to_string(),
        moon_longitude: 156.0,
        balance: DashaBalance {
            planet: DashaPlanet::Mars,
            years_remaining: 2.0,
            months_remaining: 0.0,
            days_remaining: 0.0,
            total_period_years: 7.0,
        },
        mahadashas: vec![maha.clone()],
        current_mahadasha: maha,
        current_antardasha: Some(antar),
        current_pratyantardasha: None,
        current_sookshma: None,
    }
}

#[test]
fn test_mahadasha_lord_by_date() {
    let dasha = sample_dasha();
    let lord = dasha_lord_by_date(&dasha, "2025-06-01", DashaLevel::Mahadasha);
    assert_eq!(lord, Some(DashaPlanet::Mars));
}

#[test]
fn test_antardasha_lord_by_date() {
    let dasha = sample_dasha();
    let lord = dasha_lord_by_date(&dasha, "2024-06-01", DashaLevel::Antardasha);
    assert_eq!(lord, Some(DashaPlanet::Rahu));
}

#[test]
fn test_dasha_period_by_date_none_when_missing() {
    let dasha = sample_dasha();
    let period = dasha_period_by_date(&dasha, "2030-01-01", DashaLevel::Mahadasha);
    assert!(period.is_none());
}
