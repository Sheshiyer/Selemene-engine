//! Shared handling of a missing birth time.
//!
//! Engines that need a time of day but receive none assume local noon. This is
//! input defaulting, not a calculation fallback (`fallback_used` ignores it),
//! but callers must still be able to tell that the time was not supplied.
//! Every engine that defaults a birth time reports it the same way:
//!
//! * `result.birth_time_assumed`: `true` when the default was used, `false`
//!   when the caller supplied a time.
//! * `result.assumed_birth_time`: the local time that was assumed (`"HH:MM"`),
//!   present only when `birth_time_assumed` is `true`.

use chrono::NaiveTime;
use serde_json::{json, Value};

use crate::EngineError;

/// Local time assumed when the birth time is missing.
pub const DEFAULT_BIRTH_TIME: &str = "12:00";

/// A birth time of day and whether it was assumed rather than supplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BirthTime {
    /// Local time of day.
    pub time: NaiveTime,
    /// `true` when no time was supplied and [`DEFAULT_BIRTH_TIME`] was used.
    pub assumed: bool,
}

impl BirthTime {
    /// Parse a supplied `HH:MM` or `HH:MM:SS` time, or assume
    /// [`DEFAULT_BIRTH_TIME`] when none was given. A supplied time that does
    /// not parse is a [`EngineError::ValidationError`], never a silent default.
    pub fn resolve(time: Option<&str>) -> Result<Self, EngineError> {
        let (text, assumed) = match time {
            Some(text) => (text, false),
            None => (DEFAULT_BIRTH_TIME, true),
        };
        let time = NaiveTime::parse_from_str(text, "%H:%M")
            .or_else(|_| NaiveTime::parse_from_str(text, "%H:%M:%S"))
            .map_err(|e| EngineError::ValidationError(format!("Invalid time: {}", e)))?;
        Ok(Self { time, assumed })
    }

    /// Write `birth_time_assumed` (and `assumed_birth_time` when assumed) into
    /// an engine's JSON result object. Does nothing if `result` is not an
    /// object.
    pub fn write_to(&self, result: &mut Value) {
        let Some(object) = result.as_object_mut() else {
            return;
        };
        object.insert("birth_time_assumed".to_string(), json!(self.assumed));
        if self.assumed {
            object.insert(
                "assumed_birth_time".to_string(),
                json!(self.time.format("%H:%M").to_string()),
            );
        } else {
            object.remove("assumed_birth_time");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_time_assumes_local_noon() {
        let birth_time = BirthTime::resolve(None).unwrap();
        assert_eq!(birth_time.time, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        assert!(birth_time.assumed);
    }

    #[test]
    fn supplied_time_is_not_assumed() {
        for (text, expected) in [("14:30", (14, 30, 0)), ("06:05:09", (6, 5, 9))] {
            let birth_time = BirthTime::resolve(Some(text)).unwrap();
            let (h, m, s) = expected;
            assert_eq!(birth_time.time, NaiveTime::from_hms_opt(h, m, s).unwrap());
            assert!(!birth_time.assumed, "{text}");
        }
    }

    #[test]
    fn supplied_time_that_does_not_parse_is_rejected() {
        for text in ["25:99", "noon", ""] {
            match BirthTime::resolve(Some(text)) {
                Err(EngineError::ValidationError(msg)) => {
                    assert!(msg.starts_with("Invalid time"), "{text}: {msg}")
                }
                other => panic!("{text:?}: expected ValidationError, got {other:?}"),
            }
        }
    }

    #[test]
    fn write_to_reports_assumed_time() {
        let mut result = json!({"metrics": {}});
        BirthTime::resolve(None).unwrap().write_to(&mut result);
        assert_eq!(result["birth_time_assumed"], json!(true));
        assert_eq!(result["assumed_birth_time"], json!("12:00"));
    }

    #[test]
    fn write_to_omits_assumed_time_when_supplied() {
        let mut result = json!({"metrics": {}, "assumed_birth_time": "12:00"});
        BirthTime::resolve(Some("14:30"))
            .unwrap()
            .write_to(&mut result);
        assert_eq!(result["birth_time_assumed"], json!(false));
        assert!(result.get("assumed_birth_time").is_none());
    }

    #[test]
    fn write_to_ignores_non_object_results() {
        let mut result = json!([1, 2, 3]);
        BirthTime::resolve(None).unwrap().write_to(&mut result);
        assert_eq!(result, json!([1, 2, 3]));
    }
}
