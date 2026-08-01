//! The weekly and monthly surfaces.
//!
//! Only two contributors vary by date: the three named cycles, which the
//! biorhythm engine emits as a real forward series, and the chronofield, which
//! decrements deterministically toward the next period boundary. Everything
//! else is held at its day-zero reading and named in `held_constant`, so a
//! reader can see which parts of the window are genuinely forward-looking and
//! which are a single reading repeated.

use chrono::{Datelike, Duration, NaiveDate};
use noesis_core::EngineError;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::composite::compose_with_overrides;
use crate::contributors::{chronofield_at_offset, min_days_until, three_wave_from_forecast};
use crate::models::{
    CalendarDay, Contributor, ContributorId, MonthlyAlignmentCalendar, RiskDay, TransitionMarker,
    WeeklyRiskLandscape,
};

/// Forecast horizon requested from the biorhythm engine. Thirty-five days
/// covers any civil month from any day of that month.
pub const FORECAST_DAYS: i64 = 35;

/// Day-varying readings extracted once and reused by both surfaces.
pub struct DateSeries {
    /// Three-wave sub-score by date, from the biorhythm forecast.
    pub three_wave: HashMap<NaiveDate, f64>,
    /// Dates the biorhythm engine flagged critical.
    pub critical: HashSet<NaiveDate>,
    /// Vimshottari period boundaries falling on a date.
    pub transitions: HashMap<NaiveDate, TransitionMarker>,
    /// Smallest non-negative `days_until` at day zero, if any.
    pub min_days_until: Option<i64>,
}

fn parse_ymd(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

impl DateSeries {
    pub fn build(
        biorhythm: Option<&(Value, String)>,
        vimshottari: Option<&(Value, String)>,
    ) -> Self {
        let mut three_wave = HashMap::new();
        let mut critical = HashSet::new();

        if let Some((result, _)) = biorhythm {
            if let Some(rows) = result.get("forecast").and_then(Value::as_array) {
                for row in rows {
                    let (Some(date), Some(score)) = (
                        row.get("date").and_then(Value::as_str).and_then(parse_ymd),
                        three_wave_from_forecast(row),
                    ) else {
                        continue;
                    };
                    three_wave.insert(date, score);
                }
            }
            if let Some(days) = result.get("critical_days").and_then(Value::as_array) {
                for day in days {
                    if let Some(date) = day.as_str().and_then(parse_ymd) {
                        critical.insert(date);
                    }
                }
            }
        }

        let mut transitions = HashMap::new();
        if let Some((result, _)) = vimshottari {
            if let Some(rows) = result.get("upcoming_transitions").and_then(Value::as_array) {
                for row in rows {
                    let Some(date) = row
                        .get("date")
                        .and_then(Value::as_str)
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|d| d.date_naive())
                    else {
                        continue;
                    };
                    transitions.insert(
                        date,
                        TransitionMarker {
                            level: row
                                .get("type")
                                .and_then(Value::as_str)
                                .unwrap_or("Unknown")
                                .to_string(),
                            from_planet: row
                                .get("from_planet")
                                .and_then(Value::as_str)
                                .unwrap_or("Unknown")
                                .to_string(),
                            to_planet: row
                                .get("to_planet")
                                .and_then(Value::as_str)
                                .unwrap_or("Unknown")
                                .to_string(),
                        },
                    );
                }
            }
        }

        let min_days_until = vimshottari.and_then(|(r, _)| min_days_until(r));

        Self {
            three_wave,
            critical,
            transitions,
            min_days_until,
        }
    }

    /// Three-wave score for a date: the day-zero reading for today, the
    /// forecast series for a forward date, and nothing at all for a date the
    /// series does not reach.
    fn three_wave_on(
        &self,
        date: NaiveDate,
        today: NaiveDate,
        day_zero: Option<f64>,
    ) -> Option<f64> {
        if date == today {
            day_zero
        } else {
            self.three_wave.get(&date).copied()
        }
    }

    fn days_to_transition(&self, offset: i64) -> Option<i64> {
        self.min_days_until.map(|d| (d - offset).max(0))
    }
}

/// The names of the contributors that do not move across the window.
fn held_constant(contributors: &[Contributor]) -> Vec<String> {
    contributors
        .iter()
        .filter(|c| {
            c.normalized.is_some()
                && !matches!(
                    c.id,
                    ContributorId::ThreeWaveCycle | ContributorId::Chronofield
                )
        })
        .map(|c| c.id.as_str().to_string())
        .collect()
}

pub fn weekly_risk_landscape(
    contributors: &[Contributor],
    series: &DateSeries,
    today: NaiveDate,
    saturn_pressure: bool,
) -> Result<WeeklyRiskLandscape, EngineError> {
    let day_zero_wave = contributors
        .iter()
        .find(|c| c.id == ContributorId::ThreeWaveCycle)
        .and_then(|c| c.normalized);

    let mut days = Vec::with_capacity(7);
    for offset in 0..7_i64 {
        let date = today + Duration::days(offset);
        let three_wave = series.three_wave_on(date, today, day_zero_wave);

        let mut overrides: HashMap<ContributorId, Option<f64>> = HashMap::new();
        overrides.insert(ContributorId::ThreeWaveCycle, three_wave);
        if let Some(min_days) = series.min_days_until {
            overrides.insert(
                ContributorId::Chronofield,
                Some(chronofield_at_offset(min_days, offset)),
            );
        }

        let composite = compose_with_overrides(contributors, &overrides)?;

        days.push(RiskDay {
            date,
            value: composite.value,
            convergence: composite.convergence,
            three_wave,
            days_to_transition: series.days_to_transition(offset),
            critical_cycle_day: series.critical.contains(&date),
            transition_on_date: series.transitions.get(&date).cloned(),
        });
    }

    Ok(WeeklyRiskLandscape {
        start_date: today,
        days,
        held_constant: held_constant(contributors),
        saturn_pressure,
    })
}

/// First and last civil day of the month containing `date`.
fn month_bounds(date: NaiveDate) -> Option<(NaiveDate, NaiveDate)> {
    let first = NaiveDate::from_ymd_opt(date.year(), date.month(), 1)?;
    let next_month = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)?
    };
    Some((first, next_month - Duration::days(1)))
}

pub fn monthly_alignment_calendar(
    contributors: &[Contributor],
    series: &DateSeries,
    today: NaiveDate,
) -> Result<MonthlyAlignmentCalendar, EngineError> {
    let (first, last) = month_bounds(today).ok_or_else(|| {
        EngineError::CalculationError(format!("could not resolve the month containing {}", today))
    })?;

    let day_zero_wave = contributors
        .iter()
        .find(|c| c.id == ContributorId::ThreeWaveCycle)
        .and_then(|c| c.normalized);

    let mut days = Vec::new();
    let mut transition_dates = Vec::new();
    let mut critical_dates = Vec::new();

    let mut date = first;
    while date <= last {
        let offset = (date - today).num_days();
        let three_wave = series.three_wave_on(date, today, day_zero_wave);
        let transition_on_date = series.transitions.get(&date).cloned();
        let critical_cycle_day = series.critical.contains(&date);

        if transition_on_date.is_some() {
            transition_dates.push(date);
        }
        if critical_cycle_day {
            critical_dates.push(date);
        }

        days.push(CalendarDay {
            date,
            three_wave,
            days_to_transition: if offset >= 0 {
                series.days_to_transition(offset)
            } else {
                None
            },
            transition_on_date,
            critical_cycle_day,
        });

        date += Duration::days(1);
    }

    Ok(MonthlyAlignmentCalendar {
        year: today.year(),
        month: today.month(),
        days,
        transition_dates,
        critical_dates,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::ContributorStatus;
    use serde_json::json;

    fn contributor(id: ContributorId, normalized: Option<f64>) -> Contributor {
        Contributor {
            id,
            engine_id: "test".to_string(),
            engine_version: "0".to_string(),
            fields_consumed: vec!["/test".to_string()],
            normalization: "test".to_string(),
            raw: Value::Null,
            normalized,
            status: if normalized.is_some() {
                ContributorStatus::Present
            } else {
                ContributorStatus::absent("test", "test")
            },
            observation: "test".to_string(),
        }
    }

    fn base_set() -> Vec<Contributor> {
        vec![
            contributor(ContributorId::HeartRateVariability, None),
            contributor(ContributorId::Chronofield, Some(0.5)),
            contributor(ContributorId::ActivePlanetaryWeather, Some(0.5)),
            contributor(ContributorId::ThreeWaveCycle, Some(0.5)),
            contributor(ContributorId::EnergeticAuthority, None),
            contributor(ContributorId::GiftShadowSpectrum, None),
        ]
    }

    fn series() -> DateSeries {
        let biorhythm = (
            json!({
                "critical_days": ["2026-08-03"],
                "forecast": [
                    { "date": "2026-08-02", "physical": 60.0, "emotional": 60.0, "intellectual": 60.0 },
                    { "date": "2026-08-03", "physical": 30.0, "emotional": 30.0, "intellectual": 30.0 },
                ],
            }),
            "test-fixture".to_string(),
        );
        let vimshottari = (
            json!({
                "current_period": { "mahadasha": {} },
                "upcoming_transitions": [{
                    "type": "Antardasha", "from_planet": "Ketu", "to_planet": "Venus",
                    "date": "2026-08-05T00:00:00Z", "days_until": 4
                }],
            }),
            "test-fixture".to_string(),
        );
        DateSeries::build(Some(&biorhythm), Some(&vimshottari))
    }

    #[test]
    fn weekly_landscape_has_exactly_seven_days() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let w = weekly_risk_landscape(&base_set(), &series(), today, false).unwrap();
        assert_eq!(w.days.len(), 7);
        assert_eq!(w.start_date, today);
        assert_eq!(w.days[0].date, today);
        assert_eq!(w.days[6].date, NaiveDate::from_ymd_opt(2026, 8, 7).unwrap());
    }

    #[test]
    fn weekly_landscape_names_what_it_held_constant() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let w = weekly_risk_landscape(&base_set(), &series(), today, false).unwrap();
        assert!(w
            .held_constant
            .contains(&"active_planetary_weather".to_string()));
        assert!(!w.held_constant.contains(&"three_wave_cycle".to_string()));
        assert!(!w.held_constant.contains(&"chronofield".to_string()));
    }

    #[test]
    fn the_three_wave_series_actually_moves_across_the_window() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let w = weekly_risk_landscape(&base_set(), &series(), today, false).unwrap();
        assert_eq!(w.days[0].three_wave, Some(0.5));
        assert_eq!(w.days[1].three_wave, Some(0.6));
        assert_eq!(w.days[2].three_wave, Some(0.3));
        // Beyond the supplied forecast the series simply stops.
        assert_eq!(w.days[3].three_wave, None);
    }

    #[test]
    fn transitions_and_critical_days_land_on_their_dates() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let w = weekly_risk_landscape(&base_set(), &series(), today, true).unwrap();
        assert!(w.saturn_pressure);
        assert!(w.days[2].critical_cycle_day);
        assert!(w.days[4].transition_on_date.is_some());
        assert_eq!(
            w.days[4].transition_on_date.as_ref().unwrap().level,
            "Antardasha"
        );
    }

    #[test]
    fn days_to_transition_decrements_and_floors_at_zero() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let w = weekly_risk_landscape(&base_set(), &series(), today, false).unwrap();
        assert_eq!(w.days[0].days_to_transition, Some(4));
        assert_eq!(w.days[4].days_to_transition, Some(0));
        assert_eq!(w.days[6].days_to_transition, Some(0));
    }

    #[test]
    fn monthly_calendar_covers_every_civil_day() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let m = monthly_alignment_calendar(&base_set(), &series(), today).unwrap();
        assert_eq!(m.year, 2026);
        assert_eq!(m.month, 8);
        assert_eq!(m.days.len(), 31);
        assert_eq!(m.transition_dates.len(), 1);
        assert_eq!(m.critical_dates.len(), 1);
    }

    #[test]
    fn monthly_calendar_handles_february_and_december() {
        let feb = NaiveDate::from_ymd_opt(2026, 2, 15).unwrap();
        let m = monthly_alignment_calendar(&base_set(), &series(), feb).unwrap();
        assert_eq!(m.days.len(), 28);

        let dec = NaiveDate::from_ymd_opt(2026, 12, 20).unwrap();
        let m = monthly_alignment_calendar(&base_set(), &series(), dec).unwrap();
        assert_eq!(m.days.len(), 31);
    }

    #[test]
    fn dates_before_today_carry_no_forward_reading() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
        let m = monthly_alignment_calendar(&base_set(), &series(), today).unwrap();
        assert_eq!(m.days[0].days_to_transition, None);
    }
}
