//! Daily panchang calculator

use super::{DailyPanchang, KaranaInfo, NakshatraInfo, TimePeriod, TithiInfo, YogaInfo};
use crate::panchang::muhurta::{abhijit_period, DayEighth};
use crate::sun::{local_event_hhmm, SunEvent};
use chrono::{Datelike, NaiveDate};

/// Calculate daily panchang for a given date and location
pub fn calculate_daily_panchang(
    date: NaiveDate,
    latitude: f64,
    longitude: f64,
    timezone: f64,
) -> DailyPanchang {
    // Sunrise/sunset in local clock time ("polar_day"/"polar_night" if none)
    let sunrise = calculate_sunrise(date, latitude, longitude, timezone);
    let sunset = calculate_sunset(date, latitude, longitude, timezone);

    // Get vara (weekday)
    let vara = get_vara(date);

    // Calculate Rahu Kalam based on weekday
    let rahu_kalam = calculate_rahu_kalam(&vara, &sunrise, &sunset);
    let yama_gandam = calculate_yama_gandam(&vara, &sunrise, &sunset);
    let gulika_kaal = calculate_gulika_kaal(&vara, &sunrise, &sunset);
    let auspicious_periods = calculate_abhijit(&sunrise, &sunset);

    DailyPanchang {
        date,
        vara,
        tithi: TithiInfo {
            name: "Panchami".to_string(),
            number: 5,
            paksha: "Shukla".to_string(),
            end_time: "14:30".to_string(),
            deity: "Lakshmi".to_string(),
        },
        nakshatra: NakshatraInfo {
            name: "Rohini".to_string(),
            number: 4,
            end_time: "16:45".to_string(),
            deity: "Brahma".to_string(),
            ruler: "Moon".to_string(),
        },
        yoga: YogaInfo {
            name: "Siddhi".to_string(),
            number: 21,
            end_time: "18:20".to_string(),
            meaning: "Accomplishment".to_string(),
        },
        karana: KaranaInfo {
            name: "Bava".to_string(),
            number: 1,
            end_time: "08:15".to_string(),
        },
        sunrise,
        sunset,
        moonrise: Some("20:30".to_string()),
        moonset: Some("08:45".to_string()),
        rahu_kalam,
        yama_gandam,
        gulika_kaal,
        auspicious_periods,
        festivals: vec![],
        hindu_month: "Chaitra".to_string(),
        hindu_year: "Vikram 2081".to_string(),
        notes: vec![],
    }
}

fn calculate_sunrise(date: NaiveDate, lat: f64, lon: f64, tz: f64) -> String {
    local_event_hhmm(date, lat, lon, tz, SunEvent::Rise)
}

fn calculate_sunset(date: NaiveDate, lat: f64, lon: f64, tz: f64) -> String {
    local_event_hhmm(date, lat, lon, tz, SunEvent::Set)
}

fn get_vara(date: NaiveDate) -> String {
    let weekdays = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    weekdays[date.weekday().num_days_from_sunday() as usize].to_string()
}

fn calculate_rahu_kalam(vara: &str, sunrise: &str, sunset: &str) -> TimePeriod {
    eighth_period("Rahu Kalam", DayEighth::RahuKalam, vara, sunrise, sunset)
}

fn calculate_yama_gandam(vara: &str, sunrise: &str, sunset: &str) -> TimePeriod {
    eighth_period("Yama Gandam", DayEighth::YamaGandam, vara, sunrise, sunset)
}

fn calculate_gulika_kaal(vara: &str, sunrise: &str, sunset: &str) -> TimePeriod {
    eighth_period("Gulika Kaal", DayEighth::GulikaKaal, vara, sunrise, sunset)
}

/// Inauspicious eighth of the day for `vara`, shared with
/// `panchang::muhurta`. Start and end are empty if sunrise/sunset do not
/// parse.
fn eighth_period(
    name: &str,
    period: DayEighth,
    vara: &str,
    sunrise: &str,
    sunset: &str,
) -> TimePeriod {
    let (start, end) = period
        .period(vara, sunrise, sunset)
        .map(|(start, end, _)| (start, end))
        .unwrap_or_default();
    TimePeriod {
        name: name.to_string(),
        start,
        end,
        is_auspicious: false,
    }
}

fn calculate_abhijit(sunrise: &str, sunset: &str) -> Vec<TimePeriod> {
    abhijit_period(sunrise, sunset)
        .map(|(start, end, _)| TimePeriod {
            name: "Abhijit Muhurta".to_string(),
            start,
            end,
            is_auspicious: true,
        })
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vara() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(); // Known Sunday
        assert_eq!(get_vara(date), "Sunday");
    }

    #[test]
    fn eighths_follow_the_weekday_table_on_a_twelve_hour_day() {
        // (vara, Rahu, Yama, Gulika) start times for a 06:00–18:00 day.
        let expected = [
            ("Sunday", "16:30", "12:00", "15:00"),
            ("Monday", "07:30", "10:30", "13:30"),
            ("Tuesday", "15:00", "09:00", "12:00"),
            ("Wednesday", "12:00", "07:30", "10:30"),
            ("Thursday", "13:30", "06:00", "09:00"),
            ("Friday", "10:30", "15:00", "07:30"),
            ("Saturday", "09:00", "13:30", "06:00"),
        ];
        for (vara, rahu, yama, gulika) in expected {
            assert_eq!(
                calculate_rahu_kalam(vara, "06:00", "18:00").start,
                rahu,
                "Rahu {vara}"
            );
            assert_eq!(
                calculate_yama_gandam(vara, "06:00", "18:00").start,
                yama,
                "Yama {vara}"
            );
            assert_eq!(
                calculate_gulika_kaal(vara, "06:00", "18:00").start,
                gulika,
                "Gulika {vara}"
            );
        }
    }

    #[test]
    fn eighths_and_abhijit_scale_with_day_length() {
        // Synthetic Thursday 2026-01-15, 07:14–18:21.
        let rahu = calculate_rahu_kalam("Thursday", "07:14", "18:21");
        assert_eq!((rahu.start.as_str(), rahu.end.as_str()), ("14:10", "15:34"));
        assert!(!rahu.is_auspicious);

        let abhijit = calculate_abhijit("07:14", "18:21");
        assert_eq!(abhijit.len(), 1);
        assert_eq!(
            (abhijit[0].start.as_str(), abhijit[0].end.as_str()),
            ("12:25", "13:09")
        );
        assert!(abhijit[0].is_auspicious);
    }

    #[test]
    fn agrees_with_panchang_muhurta() {
        use crate::panchang::muhurta::{GulikaKaal, RahuKalam, YamaGandam};
        let (sunrise, sunset) = ("07:14", "18:21");
        for vara in [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ] {
            let rahu = RahuKalam::for_day(vara, sunrise, sunset).unwrap();
            let yama = YamaGandam::for_day(vara, sunrise, sunset).unwrap();
            let gulika = GulikaKaal::for_day(vara, sunrise, sunset).unwrap();
            let daily = [
                calculate_rahu_kalam(vara, sunrise, sunset),
                calculate_yama_gandam(vara, sunrise, sunset),
                calculate_gulika_kaal(vara, sunrise, sunset),
            ];
            assert_eq!(
                (&daily[0].start, &daily[0].end),
                (&rahu.start, &rahu.end),
                "Rahu {vara}"
            );
            assert_eq!(
                (&daily[1].start, &daily[1].end),
                (&yama.start, &yama.end),
                "Yama {vara}"
            );
            assert_eq!(
                (&daily[2].start, &daily[2].end),
                (&gulika.start, &gulika.end),
                "Gulika {vara}"
            );
        }
    }

    #[test]
    fn unparseable_day_gives_no_periods() {
        let rahu = calculate_rahu_kalam("Monday", "dawn", "18:00");
        assert!(rahu.start.is_empty() && rahu.end.is_empty());
        assert!(calculate_abhijit("dawn", "18:00").is_empty());
    }
}
