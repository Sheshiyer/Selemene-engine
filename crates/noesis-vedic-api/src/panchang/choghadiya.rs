//! Choghadiya - Vedic time periods for selecting auspicious timings
//!
//! Choghadiya divides the day and night into specific time periods
//! classified as good, medium, or bad for various activities.

use serde::{Deserialize, Serialize};

use super::clock::{in_period, split_span};
use super::hora::Planet;

/// Collection of all Choghadiya periods for a day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoghadiyaTimings {
    /// Day Choghadiyas (sunrise to sunset)
    pub day: Vec<Choghadiya>,
    /// Night Choghadiyas (sunset to sunrise)
    pub night: Vec<Choghadiya>,
    /// Day of week
    pub day_of_week: String,
    /// Sunrise time
    pub sunrise: String,
    /// Sunset time
    pub sunset: String,
}

impl ChoghadiyaTimings {
    /// Get the current Choghadiya for a local `HH:MM` (or `HH:MM:SS`) time.
    ///
    /// Each Choghadiya covers `[start, end)`; night ones may cross midnight.
    pub fn get_current(&self, current_time: &str) -> Option<&Choghadiya> {
        self.day
            .iter()
            .chain(self.night.iter())
            .find(|c| in_period(current_time, &c.start, &c.end))
    }

    /// Get all favorable Choghadiyas for starting new activities
    pub fn get_favorable(&self) -> Vec<&Choghadiya> {
        let all: Vec<&Choghadiya> = self.day.iter().chain(self.night.iter()).collect();

        all.into_iter()
            .filter(|c| c.nature.is_favorable())
            .collect()
    }

    /// Get all unfavorable Choghadiyas to avoid
    pub fn get_unfavorable(&self) -> Vec<&Choghadiya> {
        let all: Vec<&Choghadiya> = self.day.iter().chain(self.night.iter()).collect();

        all.into_iter()
            .filter(|c| !c.nature.is_favorable())
            .collect()
    }

    /// Find best Choghadiya for a specific type of activity
    pub fn find_best_for_activity(&self, activity_type: ActivityCategory) -> Option<&Choghadiya> {
        let all: Vec<&Choghadiya> = self.day.iter().chain(self.night.iter()).collect();

        all.into_iter()
            .filter(|c| c.suitable_for.contains(&activity_type))
            .max_by_key(|c| c.nature.score())
    }
}

/// A single Choghadiya period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choghadiya {
    /// Name of the Choghadiya
    pub name: ChoghadiyaName,
    /// Start time (HH:MM)
    pub start: String,
    /// End time (HH:MM)
    pub end: String,
    /// Duration in minutes
    pub duration_minutes: u32,
    /// Nature: good, medium, bad
    pub nature: ChoghadiyaNature,
    /// Ruling planet
    pub ruler: String,
    /// Suitable activities
    pub suitable_for: Vec<ActivityCategory>,
    /// Activities to avoid
    pub avoid: Vec<ActivityCategory>,
}

/// Names of the Choghadiya periods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoghadiyaName {
    /// Shubh - Auspicious
    Shubh,
    /// Labh - Profit
    Labh,
    /// Amrit - Nectar
    Amrit,
    /// Char - Moving (neutral, good for travel)
    Char,
    /// Rog - Disease (inauspicious)
    Rog,
    /// Kaal - Death (inauspicious)
    Kaal,
    /// Udveg - Anxiety (inauspicious)
    Udveg,
    /// Kal - Time (moderately inauspicious)
    #[serde(rename = "kal")]
    KalPeriod,
}

impl ChoghadiyaName {
    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            ChoghadiyaName::Shubh => "Shubh",
            ChoghadiyaName::Labh => "Labh",
            ChoghadiyaName::Amrit => "Amrit",
            ChoghadiyaName::Char => "Char",
            ChoghadiyaName::Rog => "Rog",
            ChoghadiyaName::Kaal => "Kaal",
            ChoghadiyaName::Udveg => "Udveg",
            ChoghadiyaName::KalPeriod => "Kal",
        }
    }

    /// Get meaning/description
    pub fn meaning(&self) -> &'static str {
        match self {
            ChoghadiyaName::Shubh => "Auspicious",
            ChoghadiyaName::Labh => "Profit",
            ChoghadiyaName::Amrit => "Nectar",
            ChoghadiyaName::Char => "Moving",
            ChoghadiyaName::Rog => "Disease",
            ChoghadiyaName::Kaal => "Death",
            ChoghadiyaName::Udveg => "Anxiety",
            ChoghadiyaName::KalPeriod => "Time",
        }
    }
}

/// Nature of Choghadiya
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoghadiyaNature {
    Good,
    Medium,
    Bad,
}

impl ChoghadiyaNature {
    /// Check if this Choghadiya is favorable
    pub fn is_favorable(&self) -> bool {
        matches!(self, ChoghadiyaNature::Good)
    }

    /// Get numerical score for ranking (higher is better)
    pub fn score(&self) -> u8 {
        match self {
            ChoghadiyaNature::Good => 3,
            ChoghadiyaNature::Medium => 2,
            ChoghadiyaNature::Bad => 1,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ChoghadiyaNature::Good => "good",
            ChoghadiyaNature::Medium => "medium",
            ChoghadiyaNature::Bad => "bad",
        }
    }
}

/// Categories of activities for Choghadiya matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityCategory {
    /// New beginnings, ventures
    StartingNew,
    /// Business and commerce
    Business,
    /// Education and learning
    Education,
    /// Travel and movement
    Travel,
    /// Religious activities
    Religious,
    /// Medical procedures
    Medical,
    /// Legal matters
    Legal,
    /// Construction
    Construction,
    /// Marriage and partnerships
    Marriage,
    /// Purchasing property/vehicles
    Purchasing,
    /// Any activity
    Any,
}

/// Day-wise Choghadiya sequences
///
/// Each Choghadiya belongs to a planet: Udveg (Sun), Char (Venus),
/// Labh (Mercury), Amrit (Moon), Kaal (Saturn), Shubh (Jupiter), Rog (Mars).
/// By day the sequence starts with the weekday lord's Choghadiya and steps
/// through the Hora order (Sun → Venus → Mercury → Moon → Saturn → Jupiter →
/// Mars): UD → CH → LA → AM → KA → SH → RO.
/// By night it starts with the lord five weekdays on and steps five places
/// in that order: SH → AM → CH → RO → KA → LA → UD.
/// The eighth period repeats the first.
pub struct ChoghadiyaSequence;

/// The Hora order of planets, which also orders the Choghadiyas.
const HORA_ORDER: [Planet; 7] = [
    Planet::Sun,
    Planet::Venus,
    Planet::Mercury,
    Planet::Moon,
    Planet::Saturn,
    Planet::Jupiter,
    Planet::Mars,
];

impl ChoghadiyaSequence {
    /// Get the day sequence for a given day of week
    pub fn get_day_sequence(day: &str) -> Vec<(ChoghadiyaName, ChoghadiyaNature, &'static str)> {
        Self::sequence(Self::lord_index(day), 1)
    }

    /// Get the night sequence for a given day of week
    pub fn get_night_sequence(day: &str) -> Vec<(ChoghadiyaName, ChoghadiyaNature, &'static str)> {
        Self::sequence(Self::lord_index(day) + 5, 5)
    }

    /// Position of the weekday lord in the Hora order (Sunday if unknown).
    fn lord_index(day: &str) -> usize {
        let lord = super::hora::HoraSequence::get_starting_planet(day);
        HORA_ORDER.iter().position(|p| *p == lord).unwrap_or(0)
    }

    fn sequence(
        start: usize,
        step: usize,
    ) -> Vec<(ChoghadiyaName, ChoghadiyaNature, &'static str)> {
        (0..8)
            .map(|i| {
                let ruler = HORA_ORDER[(start + step * i) % 7];
                let (name, nature) = choghadiya_of(ruler);
                (name, nature, ruler.as_str())
            })
            .collect()
    }
}

/// The Choghadiya ruled by a planet, with its nature.
fn choghadiya_of(ruler: Planet) -> (ChoghadiyaName, ChoghadiyaNature) {
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

/// Suitable activities for each Choghadiya name
pub fn get_suitable_activities(name: ChoghadiyaName) -> Vec<ActivityCategory> {
    match name {
        ChoghadiyaName::Shubh => vec![
            ActivityCategory::Religious,
            ActivityCategory::Marriage,
            ActivityCategory::StartingNew,
            ActivityCategory::Purchasing,
        ],
        ChoghadiyaName::Labh => vec![
            ActivityCategory::Business,
            ActivityCategory::Purchasing,
            ActivityCategory::StartingNew,
        ],
        ChoghadiyaName::Amrit => vec![
            ActivityCategory::Any,
            ActivityCategory::StartingNew,
            ActivityCategory::Medical,
        ],
        ChoghadiyaName::Char => vec![ActivityCategory::Travel, ActivityCategory::Purchasing],
        ChoghadiyaName::Rog => vec![
            ActivityCategory::Medical, // Only good for taking medicine
        ],
        ChoghadiyaName::Kaal => vec![
            ActivityCategory::Religious, // Only for worship
        ],
        ChoghadiyaName::Udveg => vec![
            ActivityCategory::Legal, // Can be used for legal battles
        ],
        ChoghadiyaName::KalPeriod => vec![
            ActivityCategory::Any, // Moderate for most things
        ],
    }
}

/// Activities to avoid for each Choghadiya name
pub fn get_activities_to_avoid(name: ChoghadiyaName) -> Vec<ActivityCategory> {
    match name {
        ChoghadiyaName::Shubh => vec![],
        ChoghadiyaName::Labh => vec![],
        ChoghadiyaName::Amrit => vec![],
        ChoghadiyaName::Char => vec![
            ActivityCategory::StartingNew, // Not best for new beginnings
        ],
        ChoghadiyaName::Rog => vec![
            ActivityCategory::StartingNew,
            ActivityCategory::Business,
            ActivityCategory::Marriage,
            ActivityCategory::Travel,
        ],
        ChoghadiyaName::Kaal => vec![
            ActivityCategory::StartingNew,
            ActivityCategory::Business,
            ActivityCategory::Travel,
            ActivityCategory::Purchasing,
        ],
        ChoghadiyaName::Udveg => vec![
            ActivityCategory::StartingNew,
            ActivityCategory::Business,
            ActivityCategory::Marriage,
        ],
        ChoghadiyaName::KalPeriod => vec![ActivityCategory::StartingNew],
    }
}

/// Calculate Choghadiya timings
///
/// Day Choghadiya = (Sunset - Sunrise) / 8
/// Night Choghadiya = (Next Sunrise - Sunset) / 8
pub fn calculate_choghadiya(
    day: &str,
    sunrise: &str,
    sunset: &str,
    next_sunrise: &str,
) -> ChoghadiyaTimings {
    let day_sequence = ChoghadiyaSequence::get_day_sequence(day);
    let night_sequence = ChoghadiyaSequence::get_night_sequence(day);

    let day_choghadiyas = calculate_period_choghadiya(&day_sequence, sunrise, sunset);

    let night_choghadiyas = calculate_period_choghadiya(&night_sequence, sunset, next_sunrise);

    ChoghadiyaTimings {
        day: day_choghadiyas,
        night: night_choghadiyas,
        day_of_week: day.to_string(),
        sunrise: sunrise.to_string(),
        sunset: sunset.to_string(),
    }
}

/// Calculate Choghadiyas for a single period (day or night) by dividing
/// `start_time`→`end_time` into equal parts.
///
/// Returns no Choghadiyas if either time cannot be parsed.
fn calculate_period_choghadiya(
    sequence: &[(ChoghadiyaName, ChoghadiyaNature, &'static str)],
    start_time: &str,
    end_time: &str,
) -> Vec<Choghadiya> {
    let Some(spans) = split_span(start_time, end_time, sequence.len() as u32) else {
        return Vec::new();
    };

    sequence
        .iter()
        .zip(spans)
        .map(
            |((name, nature, ruler), (start, end, duration_minutes))| Choghadiya {
                name: *name,
                start,
                end,
                duration_minutes,
                nature: *nature,
                ruler: ruler.to_string(),
                suitable_for: get_suitable_activities(*name),
                avoid: get_activities_to_avoid(*name),
            },
        )
        .collect()
}

/// Get recommendations for specific activities
pub fn get_recommendations(
    choghadiyas: &ChoghadiyaTimings,
    activity: ActivityCategory,
) -> Vec<Recommendation> {
    let mut recommendations = Vec::new();

    let all: Vec<&Choghadiya> = choghadiyas
        .day
        .iter()
        .chain(choghadiyas.night.iter())
        .collect();

    for choghadiya in all {
        let score = if choghadiya.suitable_for.contains(&activity) {
            3
        } else if choghadiya.avoid.contains(&activity) {
            0
        } else {
            1
        };

        if score > 0 {
            recommendations.push(Recommendation {
                choghadiya_name: choghadiya.name.as_str().to_string(),
                start: choghadiya.start.clone(),
                end: choghadiya.end.clone(),
                score,
                reason: format!("{:?} nature", choghadiya.nature),
            });
        }
    }

    // Sort by score descending
    recommendations.sort_by_key(|r| std::cmp::Reverse(r.score));

    recommendations
}

/// Recommendation for an activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub choghadiya_name: String,
    pub start: String,
    pub end: String,
    pub score: u8,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choghadiya_nature() {
        assert!(ChoghadiyaNature::Good.is_favorable());
        assert!(!ChoghadiyaNature::Bad.is_favorable());
        assert_eq!(ChoghadiyaNature::Good.score(), 3);
    }

    use ChoghadiyaName::*;

    const DAYS: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];

    fn names(sequence: &[(ChoghadiyaName, ChoghadiyaNature, &'static str)]) -> Vec<ChoghadiyaName> {
        sequence.iter().map(|(name, _, _)| *name).collect()
    }

    #[test]
    fn test_day_sequences() {
        // Traditional day table: starts with the weekday lord's Choghadiya.
        let expected = [
            [Udveg, Char, Labh, Amrit, Kaal, Shubh, Rog, Udveg],
            [Amrit, Kaal, Shubh, Rog, Udveg, Char, Labh, Amrit],
            [Rog, Udveg, Char, Labh, Amrit, Kaal, Shubh, Rog],
            [Labh, Amrit, Kaal, Shubh, Rog, Udveg, Char, Labh],
            [Shubh, Rog, Udveg, Char, Labh, Amrit, Kaal, Shubh],
            [Char, Labh, Amrit, Kaal, Shubh, Rog, Udveg, Char],
            [Kaal, Shubh, Rog, Udveg, Char, Labh, Amrit, Kaal],
        ];
        for (day, expected) in DAYS.iter().zip(expected) {
            assert_eq!(
                names(&ChoghadiyaSequence::get_day_sequence(day)),
                expected,
                "{day}"
            );
        }
    }

    #[test]
    fn test_night_sequences() {
        // Traditional night table: starts with the lord five weekdays on.
        let expected = [
            [Shubh, Amrit, Char, Rog, Kaal, Labh, Udveg, Shubh],
            [Char, Rog, Kaal, Labh, Udveg, Shubh, Amrit, Char],
            [Kaal, Labh, Udveg, Shubh, Amrit, Char, Rog, Kaal],
            [Udveg, Shubh, Amrit, Char, Rog, Kaal, Labh, Udveg],
            [Amrit, Char, Rog, Kaal, Labh, Udveg, Shubh, Amrit],
            [Rog, Kaal, Labh, Udveg, Shubh, Amrit, Char, Rog],
            [Labh, Udveg, Shubh, Amrit, Char, Rog, Kaal, Labh],
        ];
        for (day, expected) in DAYS.iter().zip(expected) {
            assert_eq!(
                names(&ChoghadiyaSequence::get_night_sequence(day)),
                expected,
                "{day}"
            );
        }
    }

    #[test]
    fn test_rulers_and_natures() {
        for (name, nature, ruler) in ChoghadiyaSequence::get_day_sequence("Sunday") {
            let (expected_ruler, expected_nature) = match name {
                Udveg => ("Sun", ChoghadiyaNature::Bad),
                Char => ("Venus", ChoghadiyaNature::Medium),
                Labh => ("Mercury", ChoghadiyaNature::Good),
                Amrit => ("Moon", ChoghadiyaNature::Good),
                Kaal => ("Saturn", ChoghadiyaNature::Bad),
                Shubh => ("Jupiter", ChoghadiyaNature::Good),
                Rog => ("Mars", ChoghadiyaNature::Bad),
                KalPeriod => unreachable!("not in the seven-period cycle"),
            };
            assert_eq!(ruler, expected_ruler, "{name:?}");
            assert_eq!(nature, expected_nature, "{name:?}");
        }
    }

    #[test]
    fn test_calculate_choghadiya() {
        let choghadiya = calculate_choghadiya("Sunday", "06:00", "18:00", "06:00");
        assert_eq!(choghadiya.day.len(), 8);
        assert_eq!(choghadiya.night.len(), 8);
    }

    #[test]
    fn test_choghadiya_timings_divide_sunrise_and_sunset() {
        // Day 06:32→18:28 = 716 min → 89.5 min each; night 18:28→06:31 = 723 min.
        let c = calculate_choghadiya("Sunday", "06:32", "18:28", "06:31");
        assert_eq!(
            (c.day[0].start.as_str(), c.day[0].end.as_str()),
            ("06:32", "08:01")
        );
        assert_eq!(c.day[0].duration_minutes, 89);
        assert_eq!(c.day[4].start, "12:30");
        assert_eq!(c.day[7].end, "18:28");
        assert_eq!(c.night[0].start, "18:28");
        assert_eq!(c.night[7].end, "06:31");
        for pair in c.day.windows(2).chain(c.night.windows(2)) {
            assert_eq!(pair[0].end, pair[1].start);
        }
    }

    #[test]
    fn test_get_current_crosses_midnight() {
        let c = calculate_choghadiya("Sunday", "06:00", "18:00", "06:00");
        // Night periods are 90 min: the 4th runs 22:30→00:00, the 5th 00:00→01:30.
        assert_eq!(c.get_current("23:59").unwrap().name, Rog);
        assert_eq!(c.get_current("00:00").unwrap().name, Kaal);
        assert_eq!(c.get_current("05:59:59").unwrap().name, Shubh);
        // Boundaries belong to the period that starts there.
        assert_eq!(c.get_current("07:30").unwrap().name, Char);
        assert!(c.get_current("not a time").is_none());
    }

    #[test]
    fn test_unparseable_times_give_no_periods() {
        let c = calculate_choghadiya("Sunday", "dawn", "18:00", "06:00");
        assert!(c.day.is_empty());
        assert_eq!(c.night.len(), 8);
    }

    #[test]
    fn test_suitable_activities() {
        let activities = get_suitable_activities(ChoghadiyaName::Amrit);
        assert!(activities.contains(&ActivityCategory::Any));

        let activities = get_suitable_activities(ChoghadiyaName::Char);
        assert!(activities.contains(&ActivityCategory::Travel));
    }
}
