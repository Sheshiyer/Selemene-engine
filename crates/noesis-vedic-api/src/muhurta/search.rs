//! Date-range muhurta search.
//!
//! PR3 — closes the long-standing gap where `MuhurtaSearchCriteria` had no
//! actual search loop and the only available path was the dead vendor POST.
//! `search_muhurtas` walks every day in `[from_date, to_date]`, computes
//! panchang + sunrise/sunset for each, then iterates one-hour slots over
//! the 24h period anchored on sunrise, running the activity-specific
//! evaluator with the current panchang + Rahu/Yama/Gulika dosha flags.
//!
//! Quality threshold filtering and `TimePreference` filtering happen inside
//! the loop so the caller receives a ready-to-render `MuhurtaResults`.
//!
//! All Swiss Ephemeris work goes through `engine_panchanga::compute_panchanga`
//! which already serialises C-FFI access through `EPHE_MUTEX`; we wrap each
//! call in `spawn_blocking` so the search stays cooperative.

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

use super::business::evaluate_business_muhurta;
use super::general::evaluate_general_muhurta;
use super::marriage::evaluate_marriage_muhurta;
use super::travel::evaluate_travel_muhurta;
use super::types::{
    MuhurtaActivity, MuhurtaQuality, MuhurtaResults, MuhurtaSearchCriteria, SelectedMuhurta,
    TimePreference,
};
use crate::panchang::muhurta::{GulikaKaal, RahuKalam, YamaGandam};

/// Convert a `MuhurtaQuality` into an ordinal so we can compare against
/// `min_quality`. Higher = better.
fn quality_rank(q: MuhurtaQuality) -> u8 {
    match q {
        MuhurtaQuality::Avoid => 0,
        MuhurtaQuality::NotRecommended => 1,
        MuhurtaQuality::Average => 2,
        MuhurtaQuality::Good => 3,
        MuhurtaQuality::Excellent => 4,
    }
}

/// Returns true if `slot_local` matches the caller's `TimePreference`.
/// `None` and `Any` match everything.
fn matches_time_preference(slot_local: NaiveTime, pref: Option<TimePreference>) -> bool {
    let Some(pref) = pref else { return true };
    let hour = slot_local.hour();
    match pref {
        TimePreference::Any => true,
        TimePreference::Morning => (4..12).contains(&hour),
        TimePreference::Afternoon => (12..17).contains(&hour),
        TimePreference::Evening => (17..21).contains(&hour),
        TimePreference::Night => !(4..21).contains(&hour),
    }
}

fn weekday_name(d: NaiveDate) -> &'static str {
    use chrono::Weekday;
    match d.weekday() {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

/// Parse an `"HH:MM"` window string from the panchang muhurta helpers into
/// a (start, end) tuple of `NaiveTime`. Falls back to the all-day window
/// `(00:00, 23:59)` on parse failure so a malformed entry can't crash the
/// loop — quality just won't be modulated for that slot.
fn parse_window(start: &str, end: &str) -> (NaiveTime, NaiveTime) {
    let parse = |s: &str| {
        NaiveTime::parse_from_str(s, "%H:%M")
            .ok()
            .unwrap_or_else(|| NaiveTime::from_hms_opt(12, 0, 0).unwrap())
    };
    (parse(start), parse(end))
}

fn time_in_window(t: NaiveTime, start: NaiveTime, end: NaiveTime) -> bool {
    if start <= end {
        t >= start && t < end
    } else {
        // Wraps midnight — shouldn't occur for these helpers but kept safe.
        t >= start || t < end
    }
}

/// Run the activity-specific evaluator with the panchang inputs we have for
/// this slot. Returns `(quality, score, favorable, unfavorable)` tuples in
/// the same shape every evaluator already exposes.
fn evaluate_for_activity(
    activity: MuhurtaActivity,
    tithi: &str,
    nakshatra: &str,
    yoga: &str,
    vara: &str,
    has_rahu: bool,
    has_yama: bool,
    has_gulika: bool,
) -> (MuhurtaQuality, u8, Vec<String>, Vec<String>) {
    match activity {
        MuhurtaActivity::Marriage => {
            evaluate_marriage_muhurta(tithi, nakshatra, yoga, vara, has_rahu, has_yama)
        }
        // Business takes a hora_lord parameter — pass "Mercury" as a neutral
        // default; richer hora lookups can layer on in a follow-up.
        MuhurtaActivity::Business | MuhurtaActivity::NewVenture => {
            evaluate_business_muhurta(tithi, nakshatra, yoga, vara, has_rahu, "Mercury")
        }
        MuhurtaActivity::Travel | MuhurtaActivity::JourneyStart => {
            evaluate_travel_muhurta(nakshatra, vara, has_rahu, None)
        }
        // All other activity types fall through to the general evaluator —
        // Education, Medical, Construction, etc.
        _ => evaluate_general_muhurta(tithi, nakshatra, yoga, vara, has_rahu, has_gulika),
    }
}

/// Walk every day in `[criteria.from_date, criteria.to_date]` and emit
/// `SelectedMuhurta` slots that meet the criteria. PR3's core search loop.
pub async fn search_muhurtas(criteria: &MuhurtaSearchCriteria) -> MuhurtaResults {
    let mut muhurtas: Vec<SelectedMuhurta> = Vec::new();
    let min_rank = quality_rank(criteria.min_quality);

    let mut day = criteria.from_date;
    while day <= criteria.to_date {
        // Sunrise / sunset for this day (PR3 astro primitive).
        let (sunrise, sunset) = crate::astro::sunrise_sunset(
            day,
            criteria.latitude,
            criteria.longitude,
            criteria.timezone,
        );
        let weekday = weekday_name(day);

        // Day-wide dosha windows (Rahu Kalam / Yama Gandam / Gulika Kaal).
        let rahu_kalam = RahuKalam::for_day(
            weekday,
            &sunrise.format("%H:%M").to_string(),
            &sunset.format("%H:%M").to_string(),
        );
        let yama_gandam = YamaGandam::for_day(weekday);
        let gulika = GulikaKaal::for_day(weekday);

        let (rahu_start, rahu_end) = parse_window(&rahu_kalam.start, &rahu_kalam.end);
        let (yama_start, yama_end) = parse_window(&yama_gandam.start, &yama_gandam.end);
        let (gulika_start, gulika_end) = parse_window(&gulika.start, &gulika.end);

        // Walk 24 one-hour slots anchored on sunrise.
        for hour_offset in 0u32..24 {
            let slot_start_dt =
                NaiveDateTime::new(day, sunrise) + Duration::hours(hour_offset as i64);
            let slot_end_dt = slot_start_dt + Duration::hours(1);

            // Re-anchor to civil day for matching the (start..=end) panchang
            // helpers. Slot date may roll to the next civil day past midnight.
            let civil_day = slot_start_dt.date();
            let civil_time = slot_start_dt.time();

            if !matches_time_preference(civil_time, criteria.preferred_time) {
                continue;
            }

            // Panchang for this slot. Runs Swiss Ephemeris under a global
            // mutex — spawn_blocking to stay cooperative.
            let date_str = civil_day.format("%Y-%m-%d").to_string();
            let time_str = civil_time.format("%H:%M").to_string();
            let tz = criteria.timezone;
            let pan = tokio::task::spawn_blocking(move || {
                engine_panchanga::compute_panchanga(&date_str, &time_str, tz)
            })
            .await
            .ok();
            let Some(pan) = pan else {
                continue;
            };

            let has_rahu = time_in_window(civil_time, rahu_start, rahu_end);
            let has_yama = time_in_window(civil_time, yama_start, yama_end);
            let has_gulika = time_in_window(civil_time, gulika_start, gulika_end);

            let (quality, score, favorable, mut unfavorable) = evaluate_for_activity(
                criteria.activity,
                &pan.tithi_name,
                &pan.nakshatra_name,
                &pan.yoga_name,
                &pan.vara_name,
                has_rahu,
                has_yama,
                has_gulika,
            );

            if has_rahu {
                unfavorable.push("Slot intersects Rahu Kalam".to_string());
            }
            if has_yama {
                unfavorable.push("Slot intersects Yama Gandam".to_string());
            }
            if has_gulika {
                unfavorable.push("Slot intersects Gulika Kaal".to_string());
            }

            if quality_rank(quality) < min_rank {
                continue;
            }

            muhurtas.push(SelectedMuhurta {
                start_time: slot_start_dt,
                end_time: slot_end_dt,
                quality,
                tithi: pan.tithi_name,
                nakshatra: pan.nakshatra_name,
                yoga: pan.yoga_name,
                karana: pan.karana_name,
                vara: pan.vara_name,
                score,
                favorable_factors: favorable,
                unfavorable_factors: unfavorable,
                recommendation: format!(
                    "{} muhurta at {} on {}",
                    quality,
                    civil_time.format("%H:%M"),
                    civil_day
                ),
            });
        }

        day += chrono::Duration::days(1);
    }

    let excellent_count = muhurtas
        .iter()
        .filter(|m| m.quality == MuhurtaQuality::Excellent)
        .count();
    let good_count = muhurtas
        .iter()
        .filter(|m| m.quality == MuhurtaQuality::Good)
        .count();

    let advice = if muhurtas.is_empty() {
        format!(
            "No suitable muhurta found for {} between {} and {}. Consider relaxing the minimum quality or widening the date range.",
            criteria.activity, criteria.from_date, criteria.to_date
        )
    } else {
        format!(
            "{} muhurta(s) found across {} day(s) — {} excellent, {} good.",
            muhurtas.len(),
            (criteria.to_date - criteria.from_date).num_days() + 1,
            excellent_count,
            good_count
        )
    };

    MuhurtaResults {
        activity: criteria.activity,
        from_date: criteria.from_date,
        to_date: criteria.to_date,
        muhurtas,
        excellent_count,
        good_count,
        advice,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn search_seven_days_bangalore_returns_at_least_one_match() {
        let from = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 5, 7).unwrap();
        let criteria = MuhurtaSearchCriteria {
            activity: MuhurtaActivity::General,
            from_date: from,
            to_date: to,
            preferred_time: Some(TimePreference::Any),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: 5.5,
            min_quality: MuhurtaQuality::Good,
        };
        let results = search_muhurtas(&criteria).await;
        assert_eq!(results.activity, MuhurtaActivity::General);
        assert_eq!(results.from_date, from);
        assert_eq!(results.to_date, to);
        assert!(
            !results.muhurtas.is_empty(),
            "7-day Bangalore search should produce at least one match"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn from_to_date_propagates_correctly() {
        // Closes the bug from `map_muhurta_response` line 177 — make sure
        // the search-loop based facade reports real dates.
        let from = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 3, 16).unwrap();
        let criteria = MuhurtaSearchCriteria {
            activity: MuhurtaActivity::Travel,
            from_date: from,
            to_date: to,
            preferred_time: None,
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: 5.5,
            min_quality: MuhurtaQuality::NotRecommended,
        };
        let results = search_muhurtas(&criteria).await;
        assert_eq!(results.from_date, from);
        assert_eq!(results.to_date, to);
    }
}
