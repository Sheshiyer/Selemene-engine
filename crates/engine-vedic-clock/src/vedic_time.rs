//! Hora and Choghadiya for an instant, bounded by local sunrise and sunset.
//!
//! The Vedic day runs from one sunrise to the next and takes the weekday of
//! the local date on which it starts. Day periods divide sunrise→sunset and
//! night periods divide sunset→next sunrise: 12 + 12 Horas, 8 + 8 Choghadiyas.

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, Utc, Weekday};
use noesis_vedic_api::panchang::{ChoghadiyaName, ChoghadiyaNature, Planet};

use crate::sun::{sun_event, SunEvent};

/// Hora order: each planetary hour is ruled by the next planet in this cycle.
const HORA_ORDER: [Planet; 7] = [
    Planet::Sun,
    Planet::Venus,
    Planet::Mercury,
    Planet::Moon,
    Planet::Saturn,
    Planet::Jupiter,
    Planet::Mars,
];

/// The local day containing the instant, bounded by sunrises.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VedicDay {
    /// Local civil date on which this Vedic day's sunrise falls.
    pub date: NaiveDate,
    pub weekday: Weekday,
    pub sunrise: DateTime<Utc>,
    pub sunset: DateTime<Utc>,
    pub next_sunrise: DateTime<Utc>,
}

/// Day half (sunrise→sunset) or night half (sunset→next sunrise).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayHalf {
    Day,
    Night,
}

impl DayHalf {
    pub fn as_str(&self) -> &'static str {
        match self {
            DayHalf::Day => "day",
            DayHalf::Night => "night",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoraPeriod {
    /// 1-12 by day, 13-24 by night.
    pub number: u8,
    pub ruler: Planet,
    pub half: DayHalf,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoghadiyaPeriod {
    /// 1-8 within its half of the day.
    pub number: u8,
    pub name: ChoghadiyaName,
    pub nature: ChoghadiyaNature,
    pub ruler: Planet,
    pub half: DayHalf,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Find the Vedic day containing `instant` at the location.
///
/// Returns `None` when sunrise or sunset does not occur on a needed date
/// (polar day or night); hora and choghadiya are undefined there.
pub fn vedic_day_for(
    instant: DateTime<Utc>,
    latitude: f64,
    longitude: f64,
    timezone_offset_minutes: i32,
) -> Option<VedicDay> {
    let offset = FixedOffset::east_opt(timezone_offset_minutes * 60)?;
    let local_date = instant.with_timezone(&offset).date_naive();
    let rise = |date| {
        sun_event(
            date,
            latitude,
            longitude,
            timezone_offset_minutes,
            SunEvent::Rise,
        )
    };
    let set = |date| {
        sun_event(
            date,
            latitude,
            longitude,
            timezone_offset_minutes,
            SunEvent::Set,
        )
    };

    let today_rise = rise(local_date)?;
    let date = if instant < today_rise {
        local_date.pred_opt()?
    } else {
        local_date
    };
    let sunrise = if date == local_date {
        today_rise
    } else {
        rise(date)?
    };

    let sunset = set(date)?;
    let next_sunrise = rise(date.succ_opt()?)?;
    // A zero-length half (Sun grazing the horizon) cannot be divided.
    if !(sunrise < sunset && sunset < next_sunrise) {
        return None;
    }

    Some(VedicDay {
        date,
        weekday: date.weekday(),
        sunrise,
        sunset,
        next_sunrise,
    })
}

/// Hora ruling `instant`, which must lie within `day`.
pub fn hora_at(day: &VedicDay, instant: DateTime<Utc>) -> HoraPeriod {
    let (half, index, start, end) = split(day, instant, 12);
    let offset = if half == DayHalf::Day { 0 } else { 12 };
    let position = (day_lord_index(day.weekday) + offset + index) % 7;
    HoraPeriod {
        number: (offset + index + 1) as u8,
        ruler: HORA_ORDER[position],
        half,
        start,
        end,
    }
}

/// Choghadiya containing `instant`, which must lie within `day`.
///
/// The day sequence starts with the weekday lord's choghadiya and steps
/// through the hora order; the night sequence starts five places on and
/// steps by five.
pub fn choghadiya_at(day: &VedicDay, instant: DateTime<Utc>) -> ChoghadiyaPeriod {
    let (half, index, start, end) = split(day, instant, 8);
    let lord = day_lord_index(day.weekday);
    let position = match half {
        DayHalf::Day => (lord + index) % 7,
        DayHalf::Night => (lord + 5 + 5 * index) % 7,
    };
    let ruler = HORA_ORDER[position];
    let (name, nature) = choghadiya_for_ruler(ruler);
    ChoghadiyaPeriod {
        number: (index + 1) as u8,
        name,
        nature,
        ruler,
        half,
        start,
        end,
    }
}

/// Natural benefics give a favourable hora; Sun, Mars and Saturn do not.
pub fn hora_is_favorable(ruler: Planet) -> bool {
    matches!(
        ruler,
        Planet::Jupiter | Planet::Venus | Planet::Mercury | Planet::Moon
    )
}

fn choghadiya_for_ruler(ruler: Planet) -> (ChoghadiyaName, ChoghadiyaNature) {
    match ruler {
        Planet::Sun => (ChoghadiyaName::Udveg, ChoghadiyaNature::Bad),
        Planet::Venus => (ChoghadiyaName::Char, ChoghadiyaNature::Medium),
        Planet::Mercury => (ChoghadiyaName::Labh, ChoghadiyaNature::Good),
        Planet::Moon => (ChoghadiyaName::Amrit, ChoghadiyaNature::Good),
        Planet::Saturn => (ChoghadiyaName::Kaal, ChoghadiyaNature::Bad),
        Planet::Jupiter => (ChoghadiyaName::Shubh, ChoghadiyaNature::Good),
        Planet::Mars => (ChoghadiyaName::Rog, ChoghadiyaNature::Bad),
    }
}

fn day_lord_index(weekday: Weekday) -> usize {
    let lord = match weekday {
        Weekday::Sun => Planet::Sun,
        Weekday::Mon => Planet::Moon,
        Weekday::Tue => Planet::Mars,
        Weekday::Wed => Planet::Mercury,
        Weekday::Thu => Planet::Jupiter,
        Weekday::Fri => Planet::Venus,
        Weekday::Sat => Planet::Saturn,
    };
    HORA_ORDER.iter().position(|p| *p == lord).unwrap_or(0)
}

/// Locate `instant` in the half of `day` it falls in, divided into `parts`
/// equal periods. Boundaries are computed in whole milliseconds so a
/// boundary instant always belongs to the period it starts.
fn split(
    day: &VedicDay,
    instant: DateTime<Utc>,
    parts: i64,
) -> (DayHalf, usize, DateTime<Utc>, DateTime<Utc>) {
    let (half, from, to) = if instant < day.sunset {
        (DayHalf::Day, day.sunrise, day.sunset)
    } else {
        (DayHalf::Night, day.sunset, day.next_sunrise)
    };
    let total = (to - from).num_milliseconds();
    let elapsed = (instant - from).num_milliseconds().clamp(0, total - 1);
    let index = elapsed * parts / total;
    let boundary = |k: i64| from + Duration::milliseconds(total * k / parts);
    let index = if boundary(index + 1) <= instant {
        index + 1
    } else {
        index
    }
    .min(parts - 1);
    (half, index as usize, boundary(index), boundary(index + 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// Synthetic day: sunrise 06:00, sunset 18:00, next sunrise 06:00 UTC.
    fn synthetic_day(date: NaiveDate) -> VedicDay {
        let at = |d: NaiveDate, h| Utc.from_utc_datetime(&d.and_hms_opt(h, 0, 0).unwrap());
        VedicDay {
            date,
            weekday: date.weekday(),
            sunrise: at(date, 6),
            sunset: at(date, 18),
            next_sunrise: at(date.succ_opt().unwrap(), 6),
        }
    }

    #[test]
    fn first_hora_of_each_day_is_its_weekday_lord() {
        // 2026-03-01 is a Sunday.
        let lords = [
            Planet::Sun,
            Planet::Moon,
            Planet::Mars,
            Planet::Mercury,
            Planet::Jupiter,
            Planet::Venus,
            Planet::Saturn,
        ];
        for (i, lord) in lords.iter().enumerate() {
            let date = NaiveDate::from_ymd_opt(2026, 3, 1 + i as u32).unwrap();
            let day = synthetic_day(date);
            assert_eq!(hora_at(&day, day.sunrise).ruler, *lord, "{date}");
        }
    }

    #[test]
    fn last_hora_hands_over_to_next_days_lord() {
        // 24 horas advance the cycle three places, so Sunday's 24th hora is
        // Mercury and the next one (Monday's first) is the Moon.
        let sunday = synthetic_day(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
        let last = hora_at(&sunday, sunday.next_sunrise - Duration::seconds(1));
        assert_eq!(last.number, 24);
        assert_eq!(last.ruler, Planet::Mercury);
    }

    #[test]
    fn sunday_choghadiya_sequences_match_the_traditional_table() {
        let day = synthetic_day(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
        let names = |from: DateTime<Utc>| -> Vec<ChoghadiyaName> {
            (0..8)
                .map(|k| choghadiya_at(&day, from + Duration::minutes(90 * k + 1)).name)
                .collect()
        };
        use ChoghadiyaName::*;
        assert_eq!(
            names(day.sunrise),
            vec![Udveg, Char, Labh, Amrit, Kaal, Shubh, Rog, Udveg]
        );
        assert_eq!(
            names(day.sunset),
            vec![Shubh, Amrit, Char, Rog, Kaal, Labh, Udveg, Shubh]
        );
    }

    #[test]
    fn first_choghadiya_by_weekday() {
        use ChoghadiyaName::*;
        // (day start, night start) for Sunday..Saturday.
        let expected = [
            (Udveg, Shubh),
            (Amrit, Char),
            (Rog, Kaal),
            (Labh, Udveg),
            (Shubh, Amrit),
            (Char, Rog),
            (Kaal, Labh),
        ];
        for (i, (day_start, night_start)) in expected.iter().enumerate() {
            let day = synthetic_day(NaiveDate::from_ymd_opt(2026, 3, 1 + i as u32).unwrap());
            assert_eq!(choghadiya_at(&day, day.sunrise).name, *day_start);
            assert_eq!(choghadiya_at(&day, day.sunset).name, *night_start);
        }
    }

    #[test]
    fn boundaries_are_half_open() {
        let day = synthetic_day(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
        let second = day.sunrise + Duration::hours(1);
        assert_eq!(hora_at(&day, second - Duration::milliseconds(1)).number, 1);
        assert_eq!(hora_at(&day, second).number, 2);
        assert_eq!(hora_at(&day, second).start, second);
        assert_eq!(hora_at(&day, day.sunset).number, 13);
        assert_eq!(hora_at(&day, day.sunset).half, DayHalf::Night);
    }
}
