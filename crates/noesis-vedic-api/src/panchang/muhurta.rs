//! Muhurta calculations - Auspicious and inauspicious time periods
//!
//! Muhurtas are specific time windows during the day that are considered
//! favorable or unfavorable for various activities.

use serde::{Deserialize, Serialize};

use super::clock::{format_time, parse_time, split_span};

/// Collection of all Muhurta timings for a day
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MuhurtaCollection {
    /// Abhijit Muhurta - Victorious midday (highly auspicious)
    pub abhijit: Option<Muhurta>,
    /// Amrit Kaal - Nectar time (auspicious for beginnings)
    pub amrit_kaal: Option<Muhurta>,
    /// Rahu Kalam - Rahu period (inauspicious, avoid new ventures)
    pub rahu_kalam: Option<Muhurta>,
    /// Yama Gandam - Yama period (inauspicious)
    pub yama_gandam: Option<Muhurta>,
    /// Gulika Kaal - Gulika rising time (inauspicious)
    pub gulika_kaal: Option<Muhurta>,
    /// Dur Muhurta - Bad time (avoid)
    pub dur_muhurta: Option<Muhurta>,
    /// Varjyam - Avoidable period
    pub varjyam: Option<Muhurta>,
    /// Brahma Muhurta - Creator's time (ideal for meditation)
    pub brahma_muhurta: Option<Muhurta>,
}

/// A single Muhurta time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Muhurta {
    /// Name of the Muhurta
    pub name: String,
    /// Start time (HH:MM format)
    pub start: String,
    /// End time (HH:MM format)
    pub end: String,
    /// Duration in minutes
    pub duration_minutes: u32,
    /// Nature: auspicious, inauspicious, mixed
    pub nature: MuhurtaNature,
    /// Deity or planetary ruler
    pub ruler: String,
    /// Activities suitable for this Muhurta
    pub suitable_activities: Vec<String>,
    /// Activities to avoid
    pub avoid_activities: Vec<String>,
}

/// Nature of a Muhurta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MuhurtaNature {
    /// Highly favorable
    Auspicious,
    /// Favorable
    Favorable,
    /// Mixed results
    Mixed,
    /// Unfavorable
    Inauspicious,
    /// Highly unfavorable
    VeryInauspicious,
}

impl MuhurtaNature {
    pub fn as_str(&self) -> &'static str {
        match self {
            MuhurtaNature::Auspicious => "auspicious",
            MuhurtaNature::Favorable => "favorable",
            MuhurtaNature::Mixed => "mixed",
            MuhurtaNature::Inauspicious => "inauspicious",
            MuhurtaNature::VeryInauspicious => "very_inauspicious",
        }
    }

    /// Check if this Muhurta is good for starting new activities
    pub fn is_good_for_starting(&self) -> bool {
        matches!(self, MuhurtaNature::Auspicious | MuhurtaNature::Favorable)
    }

    /// Check if this Muhurta should be avoided
    pub fn should_avoid(&self) -> bool {
        matches!(
            self,
            MuhurtaNature::Inauspicious | MuhurtaNature::VeryInauspicious
        )
    }
}

/// Abhijit Muhurta - The victorious midday period
///
/// This is one of the most auspicious Muhurtas, occurring around midday.
/// It is considered favorable for all activities, especially important beginnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbhijitMuhurta {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub quality: String,
}

impl AbhijitMuhurta {
    /// Create with default quality description
    pub fn new(start: String, end: String, duration: u32) -> Self {
        Self {
            start,
            end,
            duration_minutes: duration,
            quality: "Victorious - favorable for all activities".to_string(),
        }
    }
}

/// One of the three weekday-dependent eighths of the daytime.
///
/// Rahu Kalam, Yama Gandam and Gulika Kaal each occupy one of the eight
/// equal segments of sunrise→sunset. Which segment depends on the weekday.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DayEighth {
    RahuKalam,
    YamaGandam,
    GulikaKaal,
}

impl DayEighth {
    /// Segment number (1–8, counted from sunrise) for each weekday,
    /// Sunday first.
    const fn segments(self) -> [u32; 7] {
        match self {
            DayEighth::RahuKalam => [8, 2, 7, 5, 6, 4, 3],
            DayEighth::YamaGandam => [5, 4, 3, 2, 1, 7, 6],
            DayEighth::GulikaKaal => [7, 6, 5, 4, 3, 2, 1],
        }
    }

    /// The segment (1–8) this period falls in on `day`.
    pub(crate) fn segment(self, day: &str) -> Option<u32> {
        Some(self.segments()[weekday_index(day)?])
    }

    /// Segment *n* of the day, `[sunrise + (n−1)·D/8, sunrise + n·D/8)` with
    /// `D = sunset − sunrise`, as `(start, end, duration_minutes)`.
    ///
    /// Returns `None` for an unknown weekday or unparseable sunrise/sunset.
    pub(crate) fn period(
        self,
        day: &str,
        sunrise: &str,
        sunset: &str,
    ) -> Option<(String, String, u32)> {
        let n = self.segment(day)?;
        split_span(sunrise, sunset, 8)?
            .into_iter()
            .nth(n as usize - 1)
    }
}

/// Abhijit Muhurta: the 8th of the 15 equal daytime muhurtas, centred on
/// local noon, as `(start, end, duration_minutes)`.
pub(crate) fn abhijit_period(sunrise: &str, sunset: &str) -> Option<(String, String, u32)> {
    split_span(sunrise, sunset, 15)?.into_iter().nth(7)
}

/// Sunday = 0 … Saturday = 6. Accepts full or three-letter English names in
/// any case.
fn weekday_index(day: &str) -> Option<usize> {
    match day.trim().to_lowercase().as_str() {
        "sunday" | "sun" => Some(0),
        "monday" | "mon" => Some(1),
        "tuesday" | "tue" => Some(2),
        "wednesday" | "wed" => Some(3),
        "thursday" | "thu" => Some(4),
        "friday" | "fri" => Some(5),
        "saturday" | "sat" => Some(6),
        _ => None,
    }
}

/// Rahu Kalam - Rahu period
///
/// This period is ruled by Rahu (North Node) and is considered
/// inauspicious for starting new ventures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RahuKalam {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub day_of_week: String,
}

impl RahuKalam {
    /// Rahu Kalam for `day`: one eighth of sunrise→sunset, in segment
    /// Sun 8, Mon 2, Tue 7, Wed 5, Thu 6, Fri 4, Sat 3.
    ///
    /// Returns `None` for an unknown weekday or unparseable sunrise/sunset.
    pub fn for_day(day: &str, sunrise: &str, sunset: &str) -> Option<Self> {
        let (start, end, duration_minutes) = DayEighth::RahuKalam.period(day, sunrise, sunset)?;
        Some(Self {
            start,
            end,
            duration_minutes,
            day_of_week: day.to_string(),
        })
    }
}

/// Yama Gandam - Yama period
///
/// Ruled by Yama (lord of death), this period should be avoided
/// for important activities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamaGandam {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub day_of_week: String,
}

impl YamaGandam {
    /// Yama Gandam for `day`: one eighth of sunrise→sunset, in segment
    /// Sun 5, Mon 4, Tue 3, Wed 2, Thu 1, Fri 7, Sat 6.
    ///
    /// Returns `None` for an unknown weekday or unparseable sunrise/sunset.
    pub fn for_day(day: &str, sunrise: &str, sunset: &str) -> Option<Self> {
        let (start, end, duration_minutes) = DayEighth::YamaGandam.period(day, sunrise, sunset)?;
        Some(Self {
            start,
            end,
            duration_minutes,
            day_of_week: day.to_string(),
        })
    }
}

/// Gulika Kaal - Gulika period
///
/// Gulika is the son of Saturn and this period is considered inauspicious.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GulikaKaal {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub day_of_week: String,
}

impl GulikaKaal {
    /// Gulika Kaal for `day`: one eighth of sunrise→sunset, in segment
    /// Sun 7, Mon 6, Tue 5, Wed 4, Thu 3, Fri 2, Sat 1.
    ///
    /// Returns `None` for an unknown weekday or unparseable sunrise/sunset.
    pub fn for_day(day: &str, sunrise: &str, sunset: &str) -> Option<Self> {
        let (start, end, duration_minutes) = DayEighth::GulikaKaal.period(day, sunrise, sunset)?;
        Some(Self {
            start,
            end,
            duration_minutes,
            day_of_week: day.to_string(),
        })
    }
}

/// Brahma Muhurta - Creator's time
///
/// The 14th of the 15 night muhurtas: from 96 to 48 minutes before sunrise.
/// Considered the most auspicious time for meditation and spiritual practices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrahmaMuhurta {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub description: String,
}

impl BrahmaMuhurta {
    /// Brahma Muhurta for a given sunrise (`HH:MM` or `HH:MM:SS`):
    /// sunrise − 96 min to sunrise − 48 min.
    ///
    /// Returns `None` if `sunrise` does not parse.
    pub fn from_sunrise(sunrise: &str) -> Option<Self> {
        const MUHURTA_SECONDS: u32 = 48 * 60;
        const DAY_SECONDS: u32 = 24 * 60 * 60;
        let sunrise = parse_time(sunrise)?;
        let before =
            |muhurtas: u32| format_time(sunrise + DAY_SECONDS - muhurtas * MUHURTA_SECONDS);
        Some(Self {
            start: before(2),
            end: before(1),
            duration_minutes: 48,
            description: "Creator's time - ideal for meditation and spiritual practices"
                .to_string(),
        })
    }
}

/// Amrit Kaal - Nectar time
///
/// A highly auspicious period for starting new ventures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmritKaal {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub quality: String,
}

impl AmritKaal {
    pub fn new(start: String, end: String, duration: u32) -> Self {
        Self {
            start,
            end,
            duration_minutes: duration,
            quality: "Nectar time - highly auspicious for beginnings".to_string(),
        }
    }
}

/// Helper functions for Muhurta calculations
pub mod utils {
    use super::*;

    /// Check if `current_time` (`HH:MM` or `HH:MM:SS`) falls within a
    /// Muhurta. The period is half-open, `[start, end)`, and may cross
    /// midnight.
    pub fn is_within_muhurta(current_time: &str, muhurta: &Muhurta) -> bool {
        super::super::clock::in_period(current_time, &muhurta.start, &muhurta.end)
    }

    /// Get all inauspicious periods for the day
    pub fn get_inauspicious_periods(muhurtas: &MuhurtaCollection) -> Vec<&Muhurta> {
        let mut periods = Vec::new();

        if let Some(ref rahu) = muhurtas.rahu_kalam {
            if rahu.nature.should_avoid() {
                periods.push(rahu);
            }
        }
        if let Some(ref yama) = muhurtas.yama_gandam {
            if yama.nature.should_avoid() {
                periods.push(yama);
            }
        }
        if let Some(ref gulika) = muhurtas.gulika_kaal {
            if gulika.nature.should_avoid() {
                periods.push(gulika);
            }
        }
        if let Some(ref dur) = muhurtas.dur_muhurta {
            if dur.nature.should_avoid() {
                periods.push(dur);
            }
        }
        if let Some(ref varjyam) = muhurtas.varjyam {
            if varjyam.nature.should_avoid() {
                periods.push(varjyam);
            }
        }

        periods
    }

    /// Get all auspicious periods for the day
    pub fn get_auspicious_periods(muhurtas: &MuhurtaCollection) -> Vec<&Muhurta> {
        let mut periods = Vec::new();

        if let Some(ref abhijit) = muhurtas.abhijit {
            if abhijit.nature.is_good_for_starting() {
                periods.push(abhijit);
            }
        }
        if let Some(ref amrit) = muhurtas.amrit_kaal {
            if amrit.nature.is_good_for_starting() {
                periods.push(amrit);
            }
        }
        if let Some(ref brahma) = muhurtas.brahma_muhurta {
            if brahma.nature.is_good_for_starting() {
                periods.push(brahma);
            }
        }

        periods
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_muhurta_nature() {
        assert!(MuhurtaNature::Auspicious.is_good_for_starting());
        assert!(!MuhurtaNature::Inauspicious.is_good_for_starting());
        assert!(MuhurtaNature::Inauspicious.should_avoid());
    }

    const WEEKDAYS: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];

    /// Expected `(start, end)` on a 06:00–18:00 day, Sunday first.
    fn twelve_hour_day(period: DayEighth) -> [(&'static str, &'static str); 7] {
        match period {
            DayEighth::RahuKalam => [
                ("16:30", "18:00"),
                ("07:30", "09:00"),
                ("15:00", "16:30"),
                ("12:00", "13:30"),
                ("13:30", "15:00"),
                ("10:30", "12:00"),
                ("09:00", "10:30"),
            ],
            DayEighth::YamaGandam => [
                ("12:00", "13:30"),
                ("10:30", "12:00"),
                ("09:00", "10:30"),
                ("07:30", "09:00"),
                ("06:00", "07:30"),
                ("15:00", "16:30"),
                ("13:30", "15:00"),
            ],
            DayEighth::GulikaKaal => [
                ("15:00", "16:30"),
                ("13:30", "15:00"),
                ("12:00", "13:30"),
                ("10:30", "12:00"),
                ("09:00", "10:30"),
                ("07:30", "09:00"),
                ("06:00", "07:30"),
            ],
        }
    }

    #[test]
    fn eighths_on_a_twelve_hour_day_for_every_weekday() {
        for (i, day) in WEEKDAYS.iter().enumerate() {
            let rahu = RahuKalam::for_day(day, "06:00", "18:00").unwrap();
            let yama = YamaGandam::for_day(day, "06:00", "18:00").unwrap();
            let gulika = GulikaKaal::for_day(day, "06:00", "18:00").unwrap();
            let got = [
                (
                    DayEighth::RahuKalam,
                    rahu.start,
                    rahu.end,
                    rahu.duration_minutes,
                ),
                (
                    DayEighth::YamaGandam,
                    yama.start,
                    yama.end,
                    yama.duration_minutes,
                ),
                (
                    DayEighth::GulikaKaal,
                    gulika.start,
                    gulika.end,
                    gulika.duration_minutes,
                ),
            ];
            for (period, start, end, minutes) in got {
                let (want_start, want_end) = twelve_hour_day(period)[i];
                assert_eq!(
                    (start.as_str(), end.as_str()),
                    (want_start, want_end),
                    "{period:?} on {day}"
                );
                assert_eq!(minutes, 90, "{period:?} on {day}");
            }
        }
    }

    #[test]
    fn segment_table_matches_the_weekday_rule() {
        let table = |p: DayEighth| WEEKDAYS.map(|d| p.segment(d).unwrap());
        assert_eq!(table(DayEighth::RahuKalam), [8, 2, 7, 5, 6, 4, 3]);
        assert_eq!(table(DayEighth::YamaGandam), [5, 4, 3, 2, 1, 7, 6]);
        assert_eq!(table(DayEighth::GulikaKaal), [7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(DayEighth::RahuKalam.segment("thu"), Some(6));
        assert_eq!(DayEighth::RahuKalam.segment("Someday"), None);
    }

    #[test]
    fn eighths_scale_with_a_non_twelve_hour_day() {
        // Thursday 2026-01-15, synthetic Mumbai-like day 07:14–18:21:
        // D = 667 min, so each eighth is 83.375 min.
        let rahu = RahuKalam::for_day("Thursday", "07:14", "18:21").unwrap();
        // Segment 6: 14:10:52.5 → 15:34:15, shown truncated to the minute.
        assert_eq!((rahu.start.as_str(), rahu.end.as_str()), ("14:10", "15:34"));
        assert_eq!(rahu.duration_minutes, 83);

        let yama = YamaGandam::for_day("Thursday", "07:14", "18:21").unwrap();
        assert_eq!((yama.start.as_str(), yama.end.as_str()), ("07:14", "08:37"));

        let gulika = GulikaKaal::for_day("Thursday", "07:14", "18:21").unwrap();
        // Segment 3: 10:00:45 → 11:24:07.5.
        assert_eq!(
            (gulika.start.as_str(), gulika.end.as_str()),
            ("10:00", "11:24")
        );

        // Rahu Kalam on Sunday is the last eighth and ends at sunset.
        let sunday = RahuKalam::for_day("Sunday", "07:14", "18:21").unwrap();
        assert_eq!(sunday.end, "18:21");
    }

    #[test]
    fn eighths_reject_bad_input() {
        assert!(RahuKalam::for_day("Someday", "06:00", "18:00").is_none());
        assert!(YamaGandam::for_day("Monday", "dawn", "18:00").is_none());
        assert!(GulikaKaal::for_day("Monday", "06:00", "06:00").is_none());
    }

    #[test]
    fn abhijit_is_the_eighth_of_fifteen_muhurtas() {
        assert_eq!(
            abhijit_period("06:00", "18:00"),
            Some(("11:36".to_string(), "12:24".to_string(), 48))
        );
        // 07:14–18:21: each muhurta is 44.47 min; the 8th runs
        // 12:25:16 → 13:09:44.
        assert_eq!(
            abhijit_period("07:14", "18:21"),
            Some(("12:25".to_string(), "13:09".to_string(), 44))
        );
        assert_eq!(abhijit_period("bad", "18:00"), None);
    }

    #[test]
    fn is_within_muhurta_is_half_open_and_parses_times() {
        let muhurta = |start: &str, end: &str| Muhurta {
            name: "Test".to_string(),
            start: start.to_string(),
            end: end.to_string(),
            duration_minutes: 0,
            nature: MuhurtaNature::Inauspicious,
            ruler: String::new(),
            suitable_activities: vec![],
            avoid_activities: vec![],
        };
        let rahu = muhurta("14:10", "15:34");
        assert!(utils::is_within_muhurta("14:10", &rahu));
        assert!(utils::is_within_muhurta("15:33:59", &rahu));
        assert!(!utils::is_within_muhurta("15:34", &rahu));
        assert!(!utils::is_within_muhurta("14:09:59", &rahu));
        // "9:30" sorted after "09:00"–"10:30" as text.
        assert!(utils::is_within_muhurta("9:30", &muhurta("09:00", "10:30")));
        // Brahma Muhurta for a 01:00 sunrise crosses midnight.
        let brahma = muhurta("23:24", "00:12");
        assert!(utils::is_within_muhurta("23:59", &brahma));
        assert!(utils::is_within_muhurta("00:05", &brahma));
        assert!(!utils::is_within_muhurta("00:12", &brahma));
        assert!(!utils::is_within_muhurta("late", &rahu));
    }

    #[test]
    fn test_brahma_muhurta() {
        let brahma = BrahmaMuhurta::from_sunrise("06:00").unwrap();
        assert_eq!(
            (brahma.start.as_str(), brahma.end.as_str()),
            ("04:24", "05:12")
        );
        assert_eq!(brahma.duration_minutes, 48);
        assert!(brahma.description.contains("meditation"));

        let brahma = BrahmaMuhurta::from_sunrise("07:14:30").unwrap();
        assert_eq!(
            (brahma.start.as_str(), brahma.end.as_str()),
            ("05:38", "06:26")
        );

        // Sunrise soon after midnight wraps back into the previous day.
        let brahma = BrahmaMuhurta::from_sunrise("01:00").unwrap();
        assert_eq!(
            (brahma.start.as_str(), brahma.end.as_str()),
            ("23:24", "00:12")
        );

        assert!(BrahmaMuhurta::from_sunrise("dawn").is_none());
    }
}
