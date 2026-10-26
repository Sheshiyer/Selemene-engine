//! Vimshottari-only full Lahiri sidereal Moon calculation.
//!
//! Initialization and the C-library mutex belong to the existing HD ephemeris
//! layer. This module changes neither its tropical flags nor its chart logic.
use chrono::{DateTime, Datelike, Timelike, Utc};
use engine_human_design::ephemeris::{EphemerisCalculator, EPHE_MUTEX};
use libswisseph_sys::{
    tuple_result::{
        create::swe_calc_ut,
        simple::{swe_julday, swe_set_sid_mode},
    },
    SEFLG_JPLEPH, SEFLG_MOSEPH, SEFLG_SIDEREAL, SEFLG_SPEED, SEFLG_SWIEPH, SE_GREG_CAL, SE_MOON,
    SE_SIDM_LAHIRI,
};
use noesis_core::EngineError;

pub(crate) struct SiderealMoon {
    pub(crate) longitude: f64,
    flags: i32,
}

impl SiderealMoon {
    pub(crate) fn backend(&self) -> &'static str {
        if self.flags & SEFLG_MOSEPH as i32 != 0 {
            "swiss-ephemeris-lahiri-full-v1:moshier"
        } else if self.flags & SEFLG_JPLEPH as i32 != 0 {
            "swiss-ephemeris-lahiri-full-v1:jpl"
        } else {
            "swiss-ephemeris-lahiri-full-v1:swiss-files"
        }
    }

    /// Swiss Ephemeris silently substitutes the analytical Moshier ephemeris
    /// when the requested `.se1` files are unavailable; that is a fallback.
    pub(crate) fn fallback_used(&self) -> bool {
        self.flags & SEFLG_MOSEPH as i32 != 0
    }
}

pub(crate) fn lahiri_moon(
    birth: &DateTime<Utc>,
    ephe_path: &str,
) -> Result<SiderealMoon, EngineError> {
    // Initialize through the existing Once before acquiring its non-reentrant
    // mutex. No HD calculation, sidereal subtraction or second init is needed.
    let _calculator = EphemerisCalculator::new(ephe_path);
    let _guard = EPHE_MUTEX.lock().map_err(|_| {
        EngineError::CalculationError(
            "Swiss Ephemeris mutex poisoned during Lahiri calculation".into(),
        )
    })?;
    let hour = birth.hour() as f64 + birth.minute() as f64 / 60.0 + birth.second() as f64 / 3600.0;
    // SAFETY: valid chrono date components are passed by value; tuple_result
    // owns the six-coordinate and error buffers. EPHE_MUTEX covers all C calls,
    // including global sidereal mode selection and calculation atomically.
    let (flags, coordinates) = unsafe {
        let jd = swe_julday(
            birth.year(),
            birth.month() as i32,
            birth.day() as i32,
            hour,
            SE_GREG_CAL as i32,
        );
        swe_set_sid_mode(SE_SIDM_LAHIRI as i32, 0.0, 0.0);
        swe_calc_ut(
            jd,
            SE_MOON as i32,
            (SEFLG_SWIEPH | SEFLG_SPEED | SEFLG_SIDEREAL) as i32,
        )
    }
    .map_err(|(code, message)| {
        EngineError::CalculationError(format!(
            "Full Lahiri sidereal Moon failed ({code}): {message}"
        ))
    })?;
    if flags & SEFLG_SIDEREAL as i32 == 0 || coordinates.iter().any(|x| !x.is_finite()) {
        return Err(EngineError::CalculationError(
            "Swiss Ephemeris returned invalid full-sidereal Moon coordinates".into(),
        ));
    }
    Ok(SiderealMoon {
        longitude: coordinates[0].rem_euclid(360.0),
        flags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moshier_substitution_is_a_fallback() {
        let moon = SiderealMoon {
            longitude: 0.0,
            flags: (SEFLG_MOSEPH | SEFLG_SIDEREAL) as i32,
        };
        assert!(moon.fallback_used());
        assert_eq!(moon.backend(), "swiss-ephemeris-lahiri-full-v1:moshier");
    }

    #[test]
    fn swiss_files_and_jpl_are_not_fallbacks() {
        for flags in [SEFLG_SWIEPH | SEFLG_SIDEREAL, SEFLG_JPLEPH | SEFLG_SIDEREAL] {
            let moon = SiderealMoon {
                longitude: 0.0,
                flags: flags as i32,
            };
            assert!(!moon.fallback_used(), "flags {flags}");
        }
    }
}
