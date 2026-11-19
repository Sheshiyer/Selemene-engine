//! ConsciousnessEngine implementation for VedicClock-TCM
//!
//! Provides the standard Noesis engine interface for temporal recommendations
//! based on TCM organ clock and Vedic time cycles.

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Offset, SecondsFormat, TimeZone, Utc};
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde_json::{json, Value};
use std::time::Instant;

use crate::calculator::{get_current_organ, get_local_hour};
use crate::choghadiya_integration::{activity_label, recommendations_for_choghadiya_period};
use crate::dosha::get_dosha_for_hour;
use crate::hora_integration::recommendations_for_hora_period;
use crate::integration::{get_temporal_recommendation, synthesize_organ_dosha};
use crate::models::{Activity, UpcomingTransition, VedicClockResult};
use crate::recommendations::{get_optimal_timing, is_favorable_now};
use crate::sun::{SUN_EVENT_ALTITUDE_DEG, SUN_EVENT_METHOD};
use crate::vedic_time::{
    choghadiya_at, hora_at, hora_is_favorable, vedic_day_for, ChoghadiyaPeriod, HoraPeriod,
    VedicDay,
};
use crate::witness::generate_witness_prompt_seeded;
use noesis_vedic_api::panchang::{
    get_activities_to_avoid, get_suitable_activities, GulikaKaal, RahuKalam, YamaGandam,
};

/// VedicClock-TCM consciousness engine
pub struct VedicClockEngine {
    engine_id: String,
    engine_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TimezoneResolution {
    offset_minutes: i32,
    source: &'static str,
    /// IANA zone name when the offset came from one (resolved at the instant,
    /// so it follows daylight saving).
    zone: Option<String>,
}

/// Valid UTC offsets in minutes: UTC−12:00 to UTC+14:00.
const OFFSET_RANGE_MINUTES: std::ops::RangeInclusive<i64> = -720..=840;

/// Caller-supplied Panchanga indices. Tithi 15–29 (Krishna paksha) fold onto
/// the same 15 tithi qualities as 0–14; nakshatra 0–26.
const TITHI_RANGE: std::ops::RangeInclusive<u64> = 0..=29;
const NAKSHATRA_RANGE: std::ops::RangeInclusive<u64> = 0..=26;

/// Every input that determines `result` and `witness_prompt`. Two requests
/// with equal `ReplayInputs` produce identical results.
struct ReplayInputs {
    current_time: DateTime<Utc>,
    timezone: TimezoneResolution,
    location: Option<LocationResolution>,
    activity: Option<Activity>,
    tithi: Option<u8>,
    nakshatra: Option<u8>,
    consciousness_level: u8,
}

impl ReplayInputs {
    /// Canonical text of the inputs, with the engine version, used for the
    /// cache key, the replay fingerprint and the witness-prompt seed.
    fn canonical(&self) -> String {
        let location = self
            .location
            .as_ref()
            .map(|l| format!("{:.6},{:.6},{}", l.latitude, l.longitude, l.source))
            .unwrap_or_else(|| "none".to_string());
        format!(
            "vedic-clock:v{}:t{}:tz{},{},{}:loc{}:a{:?}:ti{:?}:nk{:?}:cl{}",
            env!("CARGO_PKG_VERSION"),
            self.current_time
                .to_rfc3339_opts(SecondsFormat::AutoSi, true),
            self.timezone.offset_minutes,
            self.timezone.source,
            self.timezone.zone.as_deref().unwrap_or("-"),
            location,
            self.activity,
            self.tithi,
            self.nakshatra,
            self.consciousness_level,
        )
    }

    /// 64-bit FNV-1a of [`Self::canonical`]: stable across platforms and
    /// releases of the Rust toolchain, unlike `std::hash`.
    fn fingerprint(&self) -> u64 {
        self.canonical()
            .bytes()
            .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
            })
    }

    fn to_json(&self) -> Value {
        json!({
            "current_time": self.current_time.to_rfc3339_opts(SecondsFormat::AutoSi, true),
            "timezone": {
                "offset_minutes": self.timezone.offset_minutes,
                "source": self.timezone.source,
                "zone": self.timezone.zone,
            },
            "location": self.location.as_ref().map(|l| json!({
                "latitude": l.latitude,
                "longitude": l.longitude,
                "source": l.source,
            })),
            "activity": self.activity.map(|a| a.display_name()),
            "tithi_index": self.tithi,
            "nakshatra_index": self.nakshatra,
            "consciousness_level": self.consciousness_level,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct LocationResolution {
    latitude: f64,
    longitude: f64,
    source: &'static str,
}

/// Hora and Choghadiya for the instant, or why they were not computed.
enum VedicTime {
    Computed {
        location: LocationResolution,
        day: VedicDay,
        hora: HoraPeriod,
        choghadiya: ChoghadiyaPeriod,
    },
    NotComputed {
        reason: &'static str,
        detail: &'static str,
        location: Option<LocationResolution>,
    },
}

impl VedicClockEngine {
    /// Create a new VedicClock engine instance
    pub fn new() -> Self {
        Self {
            engine_id: "vedic-clock".to_string(),
            engine_name: "Vedic Clock".to_string(),
        }
    }

    /// UTC offset (minutes) in force at `instant` for a timezone string: an
    /// IANA zone (with its daylight-saving rules), a `±HH:MM` / `±HH` offset,
    /// or one of `PST`, `CST`, `AEST` (accepted as panchanga accepts them).
    /// Returns the IANA name alongside when one was used. Anything else is a
    /// `ValidationError`, never UTC.
    fn offset_for_timezone(
        tz: &str,
        instant: DateTime<Utc>,
    ) -> Result<(i32, Option<String>), EngineError> {
        let unknown = || {
            EngineError::ValidationError(format!(
                "unknown timezone {tz:?}: expected an IANA zone name such as \"Asia/Kolkata\" \
                 or a UTC offset such as \"+05:30\""
            ))
        };

        if let Some(rest) = tz.strip_prefix('+').or_else(|| tz.strip_prefix('-')) {
            let (hours, minutes) = rest.split_once(':').unwrap_or((rest, "00"));
            let digits = |s: &str, len: std::ops::RangeInclusive<usize>| {
                len.contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit())
            };
            if !digits(hours, 1..=2) || !digits(minutes, 2..=2) {
                return Err(unknown());
            }
            let (hours, minutes): (i64, i64) = (hours.parse().unwrap(), minutes.parse().unwrap());
            let sign = if tz.starts_with('-') { -1 } else { 1 };
            let offset = sign * (hours * 60 + minutes);
            if minutes >= 60 || !OFFSET_RANGE_MINUTES.contains(&offset) {
                return Err(unknown());
            }
            return Ok((offset as i32, None));
        }

        match tz {
            "PST" => Ok((-480, None)),
            "CST" => Ok((-360, None)),
            "AEST" => Ok((600, None)),
            _ => {
                let zone: chrono_tz::Tz = tz.parse().map_err(|_| unknown())?;
                let offset = zone
                    .offset_from_utc_datetime(&instant.naive_utc())
                    .fix()
                    .local_minus_utc()
                    / 60;
                Ok((offset, Some(zone.name().to_string())))
            }
        }
    }

    /// Resolve the timezone offset in force at the requested instant.
    /// Priority:
    /// 1) options.timezone_offset (integer minutes, −720..=840)
    /// 2) birth_data.timezone (IANA name, resolved with daylight saving at
    ///    the instant, or ±HH:MM)
    /// 3) UTC (0), reported as `default_utc`
    ///
    /// A malformed or out-of-range value is a `ValidationError`; it never
    /// falls through to the next source or to UTC.
    fn resolve_timezone(input: &EngineInput) -> Result<TimezoneResolution, EngineError> {
        match input.options.get("timezone_offset") {
            None | Some(Value::Null) => {}
            Some(value) => {
                let offset = value
                    .as_i64()
                    .filter(|offset| OFFSET_RANGE_MINUTES.contains(offset))
                    .ok_or_else(|| {
                        EngineError::ValidationError(format!(
                            "options.timezone_offset must be an integer number of minutes \
                             from -720 to 840, got {value}"
                        ))
                    })?;
                return Ok(TimezoneResolution {
                    offset_minutes: offset as i32,
                    source: "options.timezone_offset",
                    zone: None,
                });
            }
        }

        if let Some(birth) = input.birth_data.as_ref() {
            let (offset_minutes, zone) =
                Self::offset_for_timezone(&birth.timezone, input.current_time)?;
            return Ok(TimezoneResolution {
                offset_minutes,
                source: "birth_data.timezone",
                zone,
            });
        }

        Ok(TimezoneResolution {
            offset_minutes: 0,
            source: "default_utc",
            zone: None,
        })
    }

    /// Resolve the observer location used for sunrise/sunset.
    /// Priority:
    /// 1) options.latitude + options.longitude
    /// 2) input.location
    /// 3) birth_data latitude/longitude
    ///
    /// Returns `Ok(None)` when no location was supplied; never guesses one.
    fn resolve_location(input: &EngineInput) -> Result<Option<LocationResolution>, EngineError> {
        let option = |key: &str| -> Result<Option<f64>, EngineError> {
            match input.options.get(key) {
                None | Some(Value::Null) => Ok(None),
                Some(value) => value.as_f64().map(Some).ok_or_else(|| {
                    EngineError::ValidationError(format!("options.{key} must be a number"))
                }),
            }
        };

        let resolved = match (option("latitude")?, option("longitude")?) {
            (Some(latitude), Some(longitude)) => Some(LocationResolution {
                latitude,
                longitude,
                source: "options",
            }),
            (None, None) => input
                .location
                .as_ref()
                .map(|loc| LocationResolution {
                    latitude: loc.latitude,
                    longitude: loc.longitude,
                    source: "location",
                })
                .or_else(|| {
                    input.birth_data.as_ref().map(|birth| LocationResolution {
                        latitude: birth.latitude,
                        longitude: birth.longitude,
                        source: "birth_data",
                    })
                }),
            _ => {
                return Err(EngineError::ValidationError(
                    "options.latitude and options.longitude must be supplied together".to_string(),
                ))
            }
        };

        if let Some(location) = &resolved {
            let valid = location.latitude.is_finite()
                && location.longitude.is_finite()
                && (-90.0..=90.0).contains(&location.latitude)
                && (-180.0..=180.0).contains(&location.longitude);
            if !valid {
                return Err(EngineError::ValidationError(format!(
                    "{} latitude/longitude out of range: ({}, {})",
                    location.source, location.latitude, location.longitude
                )));
            }
        }

        Ok(resolved)
    }

    /// Compute hora and choghadiya from local sunrise/sunset, offline.
    fn compute_vedic_time(
        datetime: DateTime<Utc>,
        timezone: &TimezoneResolution,
        location: Option<LocationResolution>,
    ) -> VedicTime {
        let Some(location) = location else {
            return VedicTime::NotComputed {
                reason: "location_missing",
                detail: "hora and choghadiya need local sunrise and sunset; supply \
                         options.latitude/longitude, location, or birth_data",
                location: None,
            };
        };

        match vedic_day_for(
            datetime,
            location.latitude,
            location.longitude,
            timezone.offset_minutes,
        ) {
            Some(day) => VedicTime::Computed {
                hora: hora_at(&day, datetime),
                choghadiya: choghadiya_at(&day, datetime),
                location,
                day,
            },
            None => VedicTime::NotComputed {
                reason: "no_sunrise_or_sunset",
                detail: "the Sun does not rise or set at this latitude on this date",
                location: Some(location),
            },
        }
    }

    /// Add `vedic_day`, and `hora`/`choghadiya` when computed, to the result.
    fn add_vedic_time(output: &mut Value, vedic_time: &VedicTime, offset_minutes: i32) {
        let local = |instant: &DateTime<Utc>| -> String {
            match FixedOffset::east_opt(offset_minutes * 60) {
                Some(tz) => instant
                    .with_timezone(&tz)
                    .to_rfc3339_opts(SecondsFormat::Secs, true),
                None => instant.to_rfc3339_opts(SecondsFormat::Secs, true),
            }
        };
        let location_json = |location: &LocationResolution| {
            json!({
                "latitude": location.latitude,
                "longitude": location.longitude,
                "source": location.source,
            })
        };

        match vedic_time {
            VedicTime::Computed {
                location,
                day,
                hora,
                choghadiya,
            } => {
                output["vedic_day"] = json!({
                    "status": "computed",
                    "date": day.date.to_string(),
                    "weekday": weekday_name(day.weekday),
                    "sunrise": local(&day.sunrise),
                    "sunset": local(&day.sunset),
                    "next_sunrise": local(&day.next_sunrise),
                    "location": location_json(location),
                    "sun_event_method": SUN_EVENT_METHOD,
                    "sun_event_altitude_deg": SUN_EVENT_ALTITUDE_DEG,
                });
                output["day_periods"] = day_periods(day, offset_minutes);
                output["hora"] = json!({
                    "number": hora.number,
                    "ruler": hora.ruler.as_str(),
                    "period": hora.half.as_str(),
                    "start": local(&hora.start),
                    "end": local(&hora.end),
                    "is_favorable": hora_is_favorable(hora.ruler),
                    "recommendations": recommendations_for_hora_period(hora),
                });
                output["choghadiya"] = json!({
                    "number": choghadiya.number,
                    "name": choghadiya.name.as_str(),
                    "meaning": choghadiya.name.meaning(),
                    "nature": choghadiya.nature.as_str(),
                    "ruler": choghadiya.ruler.as_str(),
                    "period": choghadiya.half.as_str(),
                    "start": local(&choghadiya.start),
                    "end": local(&choghadiya.end),
                    "suitable_for": get_suitable_activities(choghadiya.name)
                        .into_iter()
                        .map(activity_label)
                        .collect::<Vec<_>>(),
                    "avoid": get_activities_to_avoid(choghadiya.name)
                        .into_iter()
                        .map(activity_label)
                        .collect::<Vec<_>>(),
                    "recommendations": recommendations_for_choghadiya_period(choghadiya),
                });
            }
            VedicTime::NotComputed {
                reason,
                detail,
                location,
            } => {
                output["vedic_day"] = json!({
                    "status": "not_computed",
                    "not_computed_reason": reason,
                    "detail": detail,
                    "omitted": ["hora", "choghadiya", "day_periods"],
                    "location": location.as_ref().map(location_json),
                });
            }
        }
    }

    /// Extract optional activity from input options
    fn get_activity(options: &std::collections::HashMap<String, Value>) -> Option<Activity> {
        options
            .get("activity")
            .and_then(|v| v.as_str())
            .and_then(|s| match s.to_lowercase().as_str() {
                "meditation" => Some(Activity::Meditation),
                "exercise" => Some(Activity::Exercise),
                "work" => Some(Activity::Work),
                "eating" => Some(Activity::Eating),
                "sleep" => Some(Activity::Sleep),
                "creative" => Some(Activity::Creative),
                "social" => Some(Activity::Social),
                _ => None,
            })
    }

    /// Extract optional Panchanga indices from options. A value that is not
    /// an integer in range is a `ValidationError` rather than being wrapped
    /// onto another tithi or nakshatra.
    fn get_panchanga_indices(
        options: &std::collections::HashMap<String, Value>,
    ) -> Result<(Option<u8>, Option<u8>), EngineError> {
        let index = |key: &str, range: std::ops::RangeInclusive<u64>| match options.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(value) => value
                .as_u64()
                .filter(|v| range.contains(v))
                .map(|v| Some(v as u8))
                .ok_or_else(|| {
                    EngineError::ValidationError(format!(
                        "options.{key} must be an integer from {} to {}, got {value}",
                        range.start(),
                        range.end()
                    ))
                }),
        };
        Ok((
            index("tithi_index", TITHI_RANGE)?,
            index("nakshatra_index", NAKSHATRA_RANGE)?,
        ))
    }

    fn get_consciousness_level(options: &std::collections::HashMap<String, Value>) -> u8 {
        options
            .get("consciousness_level")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(2)
    }

    /// Resolve and validate every input that determines the result.
    fn replay_inputs(input: &EngineInput) -> Result<ReplayInputs, EngineError> {
        let (tithi, nakshatra) = Self::get_panchanga_indices(&input.options)?;
        Ok(ReplayInputs {
            current_time: input.current_time,
            timezone: Self::resolve_timezone(input)?,
            location: Self::resolve_location(input)?,
            activity: Self::get_activity(&input.options),
            tithi,
            nakshatra,
            consciousness_level: Self::get_consciousness_level(&input.options),
        })
    }

    /// Build the result JSON
    fn build_result(
        &self,
        result: &VedicClockResult,
        activity: Option<Activity>,
        datetime: chrono::DateTime<Utc>,
        timezone: &TimezoneResolution,
        vedic_time: &VedicTime,
    ) -> Value {
        let local_hour = get_local_hour(datetime, timezone.offset_minutes);
        let mut output = json!({
            "current_organ": {
                "organ": result.current_organ.organ,
                "element": result.current_organ.element,
                "time_window": result.current_organ.time_range_display(),
                "peak_energy": result.current_organ.peak_energy,
                "associated_emotion": result.current_organ.associated_emotion,
                "recommended_activities": result.current_organ.recommended_activities,
            },
            "current_dosha": {
                "dosha": result.current_dosha.dosha,
                "qualities": result.current_dosha.qualities,
            },
            "recommendation": {
                "time_window": result.recommendation.time_window,
                "organ": result.recommendation.organ,
                "dosha": result.recommendation.dosha,
                "activities": result.recommendation.activities,
                "panchanga_quality": result.recommendation.panchanga_quality,
            },
            "synthesis": synthesize_organ_dosha(datetime, timezone.offset_minutes),
            "calculated_for": result.calculated_for,
            "timezone": {
                "offset_minutes": timezone.offset_minutes,
                "source": timezone.source,
                "local_hour": local_hour,
            },
        });

        // Add activity-specific timing if requested
        if let Some(activity) = activity {
            let optimal_times = get_optimal_timing(activity, datetime, timezone.offset_minutes);
            let (is_favorable, reason) =
                is_favorable_now(activity, datetime, timezone.offset_minutes);

            output["activity_timing"] = json!({
                "activity": activity.display_name(),
                "is_favorable_now": is_favorable,
                "reason": reason,
                "optimal_windows": optimal_times.iter().map(|w| {
                    json!({
                        "time_window": w.time_range_display(),
                        "quality": w.quality,
                        "reason": w.reason,
                    })
                }).collect::<Vec<_>>(),
            });
        }

        // Add upcoming transitions if available
        if let Some(upcoming) = &result.upcoming {
            output["upcoming_transitions"] = json!(upcoming);
        }

        Self::add_vedic_time(&mut output, vedic_time, timezone.offset_minutes);

        output
    }

    /// Generate upcoming transitions for the next few hours
    fn get_upcoming_transitions(
        datetime: chrono::DateTime<Utc>,
        timezone_offset: i32,
    ) -> Vec<UpcomingTransition> {
        let current_hour = get_local_hour(datetime, timezone_offset);
        let mut transitions = Vec::new();

        // Look ahead 6 hours for transitions
        for offset in 1..=6 {
            let future_hour = (current_hour as u32 + offset as u32) % 24;

            // Check if this is an organ transition (odd hours: 1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23)
            if future_hour % 2 == 1 {
                let new_organ = crate::wisdom::get_organ_for_hour(future_hour as u8);
                transitions.push(UpcomingTransition {
                    time: format!("{}:00", future_hour),
                    description: format!("{} time begins", new_organ.organ.display_name()),
                    new_organ: Some(new_organ.organ),
                    new_dosha: None,
                });
            }

            // Check for dosha transitions (2, 6, 10, 14, 18, 22)
            if [2, 6, 10, 14, 18, 22].contains(&future_hour) {
                let new_dosha = get_dosha_for_hour(future_hour as u8);
                transitions.push(UpcomingTransition {
                    time: format!("{}:00", future_hour),
                    description: format!("{} period begins", new_dosha.dosha.display_name()),
                    new_organ: None,
                    new_dosha: Some(new_dosha.dosha),
                });
            }
        }

        transitions
    }
}

/// Rahu Kalam, Yama Gandam and Gulika Kaal for `day`: each one eighth of
/// sunrise→sunset in its weekday segment, from the shared
/// `noesis_vedic_api::panchang::muhurta` table. Times are RFC 3339 at the
/// request's offset, to the minute (sunrise and sunset are truncated to the
/// minute before the day is split, as in the muhurta code).
fn day_periods(day: &VedicDay, offset_minutes: i32) -> Value {
    let Some(tz) = FixedOffset::east_opt(offset_minutes * 60) else {
        return Value::Null;
    };
    let hhmm = |instant: &DateTime<Utc>| instant.with_timezone(&tz).format("%H:%M").to_string();
    let (sunrise, sunset) = (hhmm(&day.sunrise), hhmm(&day.sunset));
    let weekday = weekday_name(day.weekday);
    // Every segment lies between sunrise and sunset, so on the sunrise's
    // local date.
    let date = day.sunrise.with_timezone(&tz).date_naive();
    let instant = |hm: &str| -> Option<String> {
        let time = chrono::NaiveTime::parse_from_str(hm, "%H:%M").ok()?;
        let local = date.and_time(time).and_local_timezone(tz).single()?;
        Some(local.to_rfc3339_opts(SecondsFormat::Secs, true))
    };
    let period = |found: Option<(String, String, u32)>| -> Value {
        match found
            .and_then(|(start, end, minutes)| Some((instant(&start)?, instant(&end)?, minutes)))
        {
            Some((start, end, minutes)) => {
                json!({ "start": start, "end": end, "duration_minutes": minutes })
            }
            None => Value::Null,
        }
    };
    json!({
        "rahu_kalam": period(
            RahuKalam::for_day(weekday, &sunrise, &sunset)
                .map(|p| (p.start, p.end, p.duration_minutes)),
        ),
        "yama_gandam": period(
            YamaGandam::for_day(weekday, &sunrise, &sunset)
                .map(|p| (p.start, p.end, p.duration_minutes)),
        ),
        "gulika_kaal": period(
            GulikaKaal::for_day(weekday, &sunrise, &sunset)
                .map(|p| (p.start, p.end, p.duration_minutes)),
        ),
        "method": "eighth_of_daytime",
    })
}

/// Where each part of the result comes from, so calculated values, table
/// lookups, caller-supplied inputs and templated narrative are told apart.
fn provenance_detail(vedic_time: &VedicTime, replay: &ReplayInputs) -> Value {
    let sun_derived = |what: &str| match vedic_time {
        VedicTime::Computed { .. } => json!({
            "origin": "calculated",
            "method": what,
        }),
        VedicTime::NotComputed { reason, .. } => json!({
            "origin": "not_computed",
            "reason": reason,
        }),
    };
    let panchanga_quality = if replay.tithi.is_some() || replay.nakshatra.is_some() {
        json!({
            "origin": "table_lookup",
            "inputs": "caller_supplied",
            "note": "tithi_index/nakshatra_index come from options; the engine does not compute them",
        })
    } else {
        json!({ "origin": "not_supplied" })
    };
    json!({
        "current_organ": { "origin": "table_lookup", "keyed_by": "local_hour" },
        "current_dosha": { "origin": "table_lookup", "keyed_by": "local_hour" },
        "upcoming_transitions": { "origin": "table_lookup", "keyed_by": "local_hour" },
        "sunrise_sunset": sun_derived(SUN_EVENT_METHOD),
        "hora": sun_derived("sunrise_to_sunset_in_12"),
        "choghadiya": sun_derived("sunrise_to_sunset_in_8"),
        "day_periods": sun_derived("eighth_of_daytime"),
        "panchanga_quality": panchanga_quality,
        "recommendation": { "origin": "derived_interpretation" },
        "synthesis": { "origin": "derived_interpretation" },
        "activity_timing": match replay.activity {
            Some(_) => json!({ "origin": "derived_interpretation" }),
            None => json!({ "origin": "not_requested" }),
        },
        "witness_prompt": {
            "origin": "templated_narrative",
            "selection": "seeded_by_replay_fingerprint",
        },
        "fallback": {
            "used": false,
            "reason": "calculate has no remote provider or fallback branch; sunrise/sunset are computed offline",
        },
        "mock": { "used": false },
        "generated_artifacts": {
            "status": "not_applicable",
            "reason": "the engine renders no media",
        },
    })
}

fn weekday_name(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Sun => "Sunday",
        chrono::Weekday::Mon => "Monday",
        chrono::Weekday::Tue => "Tuesday",
        chrono::Weekday::Wed => "Wednesday",
        chrono::Weekday::Thu => "Thursday",
        chrono::Weekday::Fri => "Friday",
        chrono::Weekday::Sat => "Saturday",
    }
}

impl Default for VedicClockEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsciousnessEngine for VedicClockEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    fn required_phase(&self) -> u8 {
        0 // Available at all consciousness phases
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = Instant::now();

        // Every input that determines the result, validated once
        let replay = Self::replay_inputs(&input)?;
        let timezone = &replay.timezone;
        let activity = replay.activity;
        let datetime = replay.current_time;

        // Hora and choghadiya, bounded by local sunrise/sunset
        let vedic_time = Self::compute_vedic_time(datetime, timezone, replay.location.clone());

        // Calculate current organ and dosha
        let current_organ = get_current_organ(datetime, timezone.offset_minutes);
        let local_hour = get_local_hour(datetime, timezone.offset_minutes);
        let current_dosha = get_dosha_for_hour(local_hour);

        // Get temporal recommendation
        let recommendation = get_temporal_recommendation(
            datetime,
            timezone.offset_minutes,
            replay.tithi,
            replay.nakshatra,
        );

        // Get upcoming transitions
        let upcoming = Some(Self::get_upcoming_transitions(
            datetime,
            timezone.offset_minutes,
        ));

        // Build the result
        let result = VedicClockResult {
            current_organ,
            current_dosha,
            recommendation,
            upcoming,
            calculated_for: datetime.to_rfc3339(),
        };

        // Witness prompt: templates chosen by the replay fingerprint, so the
        // same request always gets the same prompt.
        let consciousness_level = replay.consciousness_level;
        let witness_prompt = generate_witness_prompt_seeded(
            &result.current_organ.organ,
            &result.current_dosha.dosha,
            consciousness_level,
            replay.fingerprint(),
        );

        // Ensure witness prompt is non-empty
        if witness_prompt.is_empty() {
            return Err(EngineError::CalculationError(
                "Witness prompt generation failed: empty result".to_string(),
            ));
        }

        let mut output = self.build_result(&result, activity, datetime, timezone, &vedic_time);
        output["replay"] = json!({
            "deterministic": true,
            "fingerprint": format!("{:016x}", replay.fingerprint()),
            "inputs": replay.to_json(),
            "excluded": [
                "metadata.timestamp",
                "metadata.calculation_time_ms",
                "metadata.cached",
            ],
        });
        output["provenance_detail"] = provenance_detail(&vedic_time, &replay);

        let elapsed = start.elapsed();

        Ok(EngineOutput {
            engine_id: self.engine_id.clone(),
            result: output,
            witness_prompt,
            consciousness_level,
            metadata: CalculationMetadata {
                calculation_time_ms: elapsed.as_secs_f64() * 1000.0,
                backend: "native".to_string(),
                precision_achieved: format!("{:?}", input.precision),
                cached: false,
                timestamp: Utc::now(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                // Local organ-clock and offline sunrise computation; no provider
                // or fallback branch.
                fallback_used: false,
                provider_id: None,
            },
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let mut messages = Vec::new();
        let mut valid = true;

        // Check witness prompt is non-empty
        if output.witness_prompt.is_empty() {
            messages.push("Witness prompt is empty".to_string());
            valid = false;
        }

        // Check result has expected fields
        if output.result.get("current_organ").is_none() {
            messages.push("Missing 'current_organ' field in result".to_string());
            valid = false;
        }

        if output.result.get("current_dosha").is_none() {
            messages.push("Missing 'current_dosha' field in result".to_string());
            valid = false;
        }

        if output.result.get("recommendation").is_none() {
            messages.push("Missing 'recommendation' field in result".to_string());
            valid = false;
        }

        // Check current_organ has required fields
        if let Some(organ) = output.result.get("current_organ") {
            for field in ["organ", "element", "time_window"] {
                if organ.get(field).is_none() {
                    messages.push(format!("Missing '{}' in current_organ", field));
                    valid = false;
                }
            }
        }

        let confidence = if valid { 1.0 } else { 0.0 };

        Ok(ValidationResult {
            valid,
            confidence,
            messages,
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        // Keyed on every replay input: organ windows change on odd hours and
        // hora/choghadiya at arbitrary instants, and the result echoes the
        // exact instant, so no coarser bucket is safe to share.
        match Self::replay_inputs(input) {
            Ok(replay) => replay.canonical(),
            // `calculate` rejects these inputs; nothing is cached under this.
            Err(_) => "vedic-clock:invalid-input".to_string(),
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use noesis_core::Precision;
    use std::collections::HashMap;

    fn create_test_input() -> EngineInput {
        let mut options = HashMap::new();
        options.insert("timezone_offset".to_string(), json!(0));

        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        }
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = VedicClockEngine::new();
        assert_eq!(engine.engine_id(), "vedic-clock");
        assert_eq!(engine.engine_name(), "Vedic Clock");
        assert_eq!(engine.required_phase(), 0);
    }

    #[tokio::test]
    async fn test_calculate_basic() {
        let engine = VedicClockEngine::new();
        let input = create_test_input();

        let result = engine.calculate(input).await;
        assert!(result.is_ok(), "Calculation should succeed");

        let output = result.unwrap();
        assert_eq!(output.engine_id, "vedic-clock");
        assert!(!output.witness_prompt.is_empty());
    }

    #[tokio::test]
    async fn test_calculate_with_activity() {
        let engine = VedicClockEngine::new();
        let mut options = HashMap::new();
        options.insert("timezone_offset".to_string(), json!(0));
        options.insert("activity".to_string(), json!("meditation"));

        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };

        let result = engine.calculate(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.result.get("activity_timing").is_some());
    }

    #[tokio::test]
    async fn test_calculate_with_panchanga() {
        let engine = VedicClockEngine::new();
        let mut options = HashMap::new();
        options.insert("timezone_offset".to_string(), json!(0));
        options.insert("tithi_index".to_string(), json!(2)); // Tritiya
        options.insert("nakshatra_index".to_string(), json!(3));

        let input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options,
        };

        let result = engine.calculate(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let recommendation = output.result.get("recommendation").unwrap();
        assert!(recommendation.get("panchanga_quality").is_some());
    }

    #[tokio::test]
    async fn test_validate_valid_output() {
        let engine = VedicClockEngine::new();
        let input = create_test_input();

        let output = engine.calculate(input).await.unwrap();
        let validation = engine.validate(&output).await.unwrap();

        assert!(validation.valid);
        assert!((validation.confidence - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_validate_invalid_output() {
        let engine = VedicClockEngine::new();

        let output = EngineOutput {
            engine_id: "vedic-clock".to_string(),
            result: json!({}),              // Missing fields
            witness_prompt: "".to_string(), // Empty
            consciousness_level: 2,
            metadata: CalculationMetadata {
                calculation_time_ms: 1.0,
                backend: "test".to_string(),
                precision_achieved: "Standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: String::new(),
                fallback_used: false,
                provider_id: None,
            },
        };

        let validation = engine.validate(&output).await.unwrap();
        assert!(!validation.valid);
        assert!(!validation.messages.is_empty());
    }

    #[tokio::test]
    async fn test_cache_key_consistency() {
        let engine = VedicClockEngine::new();
        let input = create_test_input();

        let key1 = engine.cache_key(&input);
        let key2 = engine.cache_key(&input);

        assert_eq!(key1, key2, "Cache key should be deterministic");
    }

    #[test]
    fn test_resolve_timezone() {
        let mut input = EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };

        let resolve = |input: &EngineInput| VedicClockEngine::resolve_timezone(input).unwrap();
        assert_eq!(
            resolve(&input),
            TimezoneResolution {
                offset_minutes: 0,
                source: "default_utc",
                zone: None,
            }
        );

        input
            .options
            .insert("timezone_offset".to_string(), json!(330));
        assert_eq!(
            resolve(&input),
            TimezoneResolution {
                offset_minutes: 330,
                source: "options.timezone_offset",
                zone: None,
            }
        );

        input.options.clear();
        input.birth_data = Some(noesis_core::BirthData {
            name: Some("TZ Test".to_string()),
            date: "1991-08-13".to_string(),
            time: Some("13:31".to_string()),
            latitude: 12.9340,
            longitude: 77.6214,
            timezone: "Asia/Kolkata".to_string(),
        });
        assert_eq!(
            resolve(&input),
            TimezoneResolution {
                offset_minutes: 330,
                source: "birth_data.timezone",
                zone: Some("Asia/Kolkata".to_string()),
            }
        );
    }

    #[tokio::test]
    async fn test_fixed_timestamp_uses_birth_timezone_fallback_for_ist() {
        let engine = VedicClockEngine::new();
        let input = EngineInput {
            birth_data: Some(noesis_core::BirthData {
                name: Some("Canonical".to_string()),
                date: "1991-08-13".to_string(),
                time: Some("13:31".to_string()),
                latitude: 12.9340,
                longitude: 77.6214,
                timezone: "Asia/Kolkata".to_string(),
            }),
            current_time: Utc.with_ymd_and_hms(2026, 3, 8, 6, 43, 16).unwrap(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };

        let output = engine
            .calculate(input)
            .await
            .expect("vedic-clock should calculate");
        assert_eq!(output.result["timezone"]["offset_minutes"], 330);
        assert_eq!(output.result["timezone"]["source"], "birth_data.timezone");
        assert_eq!(output.result["timezone"]["local_hour"], 12);
        assert_eq!(output.result["current_organ"]["organ"], "Heart");
    }

    #[tokio::test]
    async fn test_fixed_timestamp_supports_explicit_non_utc_offset_strings() {
        let engine = VedicClockEngine::new();
        let input = EngineInput {
            birth_data: Some(noesis_core::BirthData {
                name: Some("Nepal Offset".to_string()),
                date: "1991-08-13".to_string(),
                time: Some("13:31".to_string()),
                latitude: 27.7172,
                longitude: 85.3240,
                timezone: "+05:45".to_string(),
            }),
            current_time: Utc.with_ymd_and_hms(2026, 3, 8, 6, 43, 16).unwrap(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };

        let output = engine
            .calculate(input)
            .await
            .expect("vedic-clock should calculate");
        assert_eq!(output.result["timezone"]["offset_minutes"], 345);
        assert_eq!(output.result["timezone"]["source"], "birth_data.timezone");
        assert_eq!(output.result["timezone"]["local_hour"], 12);
        assert_eq!(output.result["current_organ"]["organ"], "Heart");
    }

    #[test]
    fn test_get_activity() {
        let mut options = HashMap::new();
        assert!(VedicClockEngine::get_activity(&options).is_none());

        options.insert("activity".to_string(), json!("meditation"));
        assert_eq!(
            VedicClockEngine::get_activity(&options),
            Some(Activity::Meditation)
        );

        options.insert("activity".to_string(), json!("EXERCISE"));
        assert_eq!(
            VedicClockEngine::get_activity(&options),
            Some(Activity::Exercise)
        );
    }
}
