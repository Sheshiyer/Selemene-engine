//! Vimshottari query helpers

use crate::dasha::{DashaLevel, DashaPeriod, DashaPlanet, VimshottariDasha};

/// Find the active dasha period at the requested level for a given date.
pub fn dasha_period_by_date<'a>(
    dasha: &'a VimshottariDasha,
    date: &str,
    level: DashaLevel,
) -> Option<&'a DashaPeriod> {
    find_period_by_date(&dasha.mahadashas, date, level)
}

/// Find the dasha lord (planet) at the requested level for a given date.
pub fn dasha_lord_by_date(
    dasha: &VimshottariDasha,
    date: &str,
    level: DashaLevel,
) -> Option<DashaPlanet> {
    dasha_period_by_date(dasha, date, level).map(|p| p.planet)
}

fn find_period_by_date<'a>(
    periods: &'a [DashaPeriod],
    date: &str,
    level: DashaLevel,
) -> Option<&'a DashaPeriod> {
    for period in periods {
        if !period.contains_date(date) {
            continue;
        }

        if period.level == level {
            return Some(period);
        }

        if let Some(subs) = period.sub_periods.as_ref() {
            if let Some(found) = find_period_by_date(subs, date, level) {
                return Some(found);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_period(level: DashaLevel, start: &str, end: &str) -> DashaPeriod {
        DashaPeriod {
            planet: DashaPlanet::Mars,
            level,
            start_date: start.to_string(),
            end_date: end.to_string(),
            duration_years: 1.0,
            duration_days: 365,
            sub_periods: None,
        }
    }

    #[test]
    fn test_find_mahadasha_by_date() {
        let dasha = VimshottariDasha {
            birth_date: "1991-08-13".to_string(),
            moon_nakshatra: "Uttara Phalguni".to_string(),
            moon_longitude: 156.0,
            balance: crate::dasha::DashaBalance {
                planet: DashaPlanet::Mars,
                years_remaining: 3.0,
                months_remaining: 0.0,
                days_remaining: 0.0,
                total_period_years: 7.0,
            },
            mahadashas: vec![sample_period(DashaLevel::Mahadasha, "2020-01-01", "2027-01-01")],
            current_mahadasha: sample_period(DashaLevel::Mahadasha, "2020-01-01", "2027-01-01"),
            current_antardasha: None,
            current_pratyantardasha: None,
            current_sookshma: None,
        };

        let period = dasha_period_by_date(&dasha, "2024-01-01", DashaLevel::Mahadasha);
        assert!(period.is_some());
        assert_eq!(period.unwrap().planet, DashaPlanet::Mars);
    }
}
