//! Time-of-day helpers for dividing sunrise/sunset spans into periods.
//!
//! Times are local wall-clock strings (`HH:MM` or `HH:MM:SS`). A span whose
//! end is earlier than its start crosses midnight.

const SECONDS_PER_DAY: u32 = 24 * 60 * 60;

/// Parse `HH:MM` or `HH:MM:SS` into seconds after midnight.
pub(crate) fn parse_time(time: &str) -> Option<u32> {
    let mut parts = time.trim().split(':');
    let hours: u32 = parts.next()?.parse().ok()?;
    let minutes: u32 = parts.next()?.parse().ok()?;
    let seconds: u32 = match parts.next() {
        Some(s) => s.parse().ok()?,
        None => 0,
    };
    if parts.next().is_some() || hours > 23 || minutes > 59 || seconds > 59 {
        return None;
    }
    Some(hours * 3600 + minutes * 60 + seconds)
}

/// Format seconds after midnight as `HH:MM`, wrapping past 24:00.
pub(crate) fn format_time(seconds: u32) -> String {
    let seconds = seconds % SECONDS_PER_DAY;
    format!("{:02}:{:02}", seconds / 3600, (seconds / 60) % 60)
}

/// Split `start`→`end` into `parts` equal periods.
///
/// Returns each period's `(start, end)` as `HH:MM` plus its length in whole
/// minutes. Consecutive periods share their boundary string. Returns `None`
/// if either time does not parse or the span is empty.
pub(crate) fn split_span(start: &str, end: &str, parts: u32) -> Option<Vec<(String, String, u32)>> {
    let from = parse_time(start)?;
    let to = parse_time(end)?;
    let span = (to + SECONDS_PER_DAY - from) % SECONDS_PER_DAY;
    if span == 0 || parts == 0 {
        return None;
    }
    let boundary = |k: u32| from + (u64::from(span) * u64::from(k) / u64::from(parts)) as u32;
    Some(
        (0..parts)
            .map(|k| {
                let (a, b) = (boundary(k), boundary(k + 1));
                (format_time(a), format_time(b), (b - a) / 60)
            })
            .collect(),
    )
}

/// Whether `time` lies in the half-open period `[start, end)`, allowing the
/// period to cross midnight.
pub(crate) fn in_period(time: &str, start: &str, end: &str) -> bool {
    let (Some(t), Some(s), Some(e)) = (parse_time(time), parse_time(start), parse_time(end)) else {
        return false;
    };
    if s <= e {
        s <= t && t < e
    } else {
        t >= s || t < e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minutes_and_seconds() {
        assert_eq!(parse_time("06:30"), Some(6 * 3600 + 30 * 60));
        assert_eq!(parse_time("18:29:23"), Some(18 * 3600 + 29 * 60 + 23));
        assert_eq!(parse_time("24:00"), None);
        assert_eq!(parse_time("6"), None);
        assert_eq!(parse_time("ab:cd"), None);
    }

    #[test]
    fn splits_a_day_span_evenly() {
        let periods = split_span("06:00", "18:00", 8).unwrap();
        assert_eq!(periods.len(), 8);
        assert_eq!(periods[0], ("06:00".into(), "07:30".into(), 90));
        assert_eq!(periods[7], ("16:30".into(), "18:00".into(), 90));
    }

    #[test]
    fn splits_a_night_span_across_midnight() {
        let periods = split_span("18:00", "06:00", 12).unwrap();
        assert_eq!(periods[5], ("23:00".into(), "00:00".into(), 60));
        assert_eq!(periods[6], ("00:00".into(), "01:00".into(), 60));
        assert_eq!(periods[11].1, "06:00");
    }

    #[test]
    fn rejects_empty_or_unparseable_spans() {
        assert!(split_span("06:00", "06:00", 8).is_none());
        assert!(split_span("06:00", "late", 8).is_none());
    }

    #[test]
    fn period_membership_is_half_open_and_wraps() {
        assert!(in_period("06:00", "06:00", "07:30"));
        assert!(!in_period("07:30", "06:00", "07:30"));
        assert!(in_period("23:59", "23:00", "00:30"));
        assert!(in_period("00:10", "23:00", "00:30"));
        assert!(!in_period("00:30", "23:00", "00:30"));
        assert!(!in_period("bad", "06:00", "07:30"));
    }
}
