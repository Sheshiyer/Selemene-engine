//! Swiss Ephemeris wrapper for Human Design planetary calculations
//!
//! Provides clean API for calculating all 13 planetary positions needed for HD charts.

use chrono::{DateTime, Datelike, Timelike, Utc};
use noesis_core::EngineError;
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::sync::{LockResult, Mutex, MutexGuard, OnceLock};

/// The ephemeris path chosen for this process ("" = built-in Moshier): the
/// first [`EphemerisCalculator::new`] or [`lock_ephemeris`] decides it.
static ACTIVE_EPHE_PATH: OnceLock<String> = OnceLock::new();

/// `SEFLG_MOSEPH`: set in the flags Swiss Ephemeris returns when it computed
/// with the built-in Moshier ephemeris instead of the requested `.se1` files.
pub const SEFLG_MOSEPH: i32 = 4;

thread_local! {
    /// libswisseph keeps its state (`swed`, including the ephemeris path) in
    /// thread-local storage on Linux (`TLS __thread` in sweodef.h; not on
    /// macOS), so `swe_set_ephe_path` must run on every thread that calculates.
    static THREAD_EPHE_PATH_SET: Cell<bool> = const { Cell::new(false) };
    /// Whether a calculation on this thread returned [`SEFLG_MOSEPH`] since the
    /// last [`reset_moshier_observed`].
    static MOSHIER_OBSERVED: Cell<bool> = const { Cell::new(false) };
}

/// Environment variables naming the Swiss Ephemeris data directory, in
/// priority order. `SWISS_EPHEMERIS_PATH` is what the production image sets;
/// `EPHEMERIS_DIR` is the API's checksum directory.
pub const EPHEMERIS_PATH_ENV_VARS: [&str; 3] =
    ["SWISS_EPHE_PATH", "SWISS_EPHEMERIS_PATH", "EPHEMERIS_DIR"];

/// Directories tried relative to the current working directory when no
/// environment variable names a usable one.
const CWD_CANDIDATES: [&str; 4] = [
    "data/ephemeris",
    "../data/ephemeris",
    "../../data/ephemeris",
    "../../../data/ephemeris",
];

/// A directory is usable only if it holds Swiss Ephemeris `.se1` files;
/// anything else would silently degrade to Moshier.
fn has_ephemeris_files(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .any(|entry| entry.path().extension().is_some_and(|ext| ext == "se1"))
        })
        .unwrap_or(false)
}

/// Resolution with injectable inputs (environment lookup, working directory,
/// executable directory) so it can be tested without mutating process state.
fn resolve_ephemeris_path_from(
    env: impl Fn(&str) -> Option<String>,
    cwd: Option<&Path>,
    exe_dir: Option<&Path>,
) -> Option<PathBuf> {
    let from_env = EPHEMERIS_PATH_ENV_VARS
        .iter()
        .filter_map(|name| env(name))
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from);
    let from_cwd = cwd
        .into_iter()
        .flat_map(|cwd| CWD_CANDIDATES.iter().map(move |rel| cwd.join(rel)));
    let from_exe = exe_dir.map(|dir| dir.join("data/ephemeris"));

    from_env
        .chain(from_cwd)
        .chain(from_exe)
        .find(|dir| has_ephemeris_files(dir))
        .map(|dir| dir.canonicalize().unwrap_or(dir))
}

/// Absolute path of the Swiss Ephemeris data directory, or `None` when no
/// candidate holds `.se1` files (the library then falls back to Moshier).
///
/// Order: [`EPHEMERIS_PATH_ENV_VARS`], then `data/ephemeris` relative to the
/// working directory and its parents, then `data/ephemeris` next to the
/// executable. Every engine that touches Swiss Ephemeris goes through
/// [`EphemerisCalculator::new`], which uses this, so whichever engine
/// initialises the process-global path first sets the real one.
pub fn resolve_ephemeris_path() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    resolve_ephemeris_path_from(
        |name| std::env::var(name).ok(),
        cwd.as_deref(),
        exe_dir.as_deref(),
    )
}

/// The ephemeris path chosen for this process: `None` before the first
/// [`EphemerisCalculator::new`] or [`lock_ephemeris`], `Some("")` when no data
/// files were found (built-in Moshier).
pub fn active_ephemeris_path() -> Option<&'static str> {
    ACTIVE_EPHE_PATH.get().map(String::as_str)
}

fn active_or_resolved_path() -> &'static str {
    ACTIVE_EPHE_PATH.get_or_init(|| {
        resolve_ephemeris_path()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default()
    })
}

/// Acquire [`EPHE_MUTEX`] for Swiss Ephemeris calls and make sure this thread's
/// libswisseph state has the process ephemeris path. Every Swiss Ephemeris
/// calculation in the workspace must run while holding this guard; locking
/// `EPHE_MUTEX` directly skips the per-thread path and silently gives Moshier
/// on Linux worker threads.
pub fn lock_ephemeris() -> LockResult<MutexGuard<'static, ()>> {
    let guard = EPHE_MUTEX.lock();
    if guard.is_ok() {
        THREAD_EPHE_PATH_SET.with(|set| {
            if !set.get() {
                swisseph::swe::set_ephe_path(active_or_resolved_path());
                set.set(true);
            }
        });
    }
    guard
}

/// Record the flags a Swiss Ephemeris calculation returned on this thread.
pub fn note_returned_flags(flags: i32) {
    if flags & SEFLG_MOSEPH != 0 {
        MOSHIER_OBSERVED.with(|seen| seen.set(true));
    }
}

/// Start observing Moshier substitution on this thread (call before a
/// synchronous run of calculations).
pub fn reset_moshier_observed() {
    MOSHIER_OBSERVED.with(|seen| seen.set(false));
}

/// Whether any calculation on this thread since [`reset_moshier_observed`]
/// fell back to the built-in Moshier ephemeris.
pub fn moshier_observed() -> bool {
    MOSHIER_OBSERVED.with(Cell::get)
}

/// Global mutex serializing ALL Swiss Ephemeris C library access.
/// The swisseph C library uses static buffers and global state that are not
/// thread-safe. Concurrent calls to set_ephe_path or calc_ut will corrupt
/// memory and cause SIGSEGV/SIGABRT.
pub static EPHE_MUTEX: Mutex<()> = Mutex::new(());

/// Planet identifiers for Swiss Ephemeris
#[derive(Debug, Clone, Copy)]
pub enum HDPlanet {
    Sun = 0,
    Moon = 1,
    Mercury = 2,
    Venus = 3,
    Mars = 4,
    Jupiter = 5,
    Saturn = 6,
    Uranus = 7,
    Neptune = 8,
    Pluto = 9,
    NorthNode = 11,  // True Node
    SouthNode = -11, // Calculated as opposite of North Node
    Earth = 14,      // Geo-centric Earth position
}

/// Planetary position result
#[derive(Debug, Clone)]
pub struct PlanetPosition {
    pub longitude: f64, // 0-360 degrees
    pub latitude: f64,  // degrees
    pub distance: f64,  // AU
    pub speed: f64,     // degrees per day
}

/// Swiss Ephemeris calculator for Human Design
pub struct EphemerisCalculator {
    data_path: String,
}

impl EphemerisCalculator {
    /// Create new calculator
    ///
    /// # Arguments
    /// * `data_path` - Path to Swiss Ephemeris data files (use "" for default)
    ///
    /// If path is empty, resolves it with [`resolve_ephemeris_path`]
    /// (environment, then working directory, then executable directory) and
    /// stays empty (built-in Moshier) only when no `.se1` files are found.
    ///
    /// The first call in the process chooses the ephemeris path for every
    /// engine; later calls cannot change it. The path is applied to each
    /// calculating thread by [`lock_ephemeris`].
    pub fn new(data_path: impl Into<String>) -> Self {
        let mut data_path = data_path.into();

        if data_path.is_empty() {
            if let Some(resolved) = resolve_ephemeris_path() {
                data_path = resolved.to_string_lossy().into_owned();
            }
        }

        let active = ACTIVE_EPHE_PATH.get_or_init(|| data_path);
        Self {
            data_path: active.clone(),
        }
    }

    /// Convert DateTime to Julian Day using Swiss Ephemeris
    fn datetime_to_jd(dt: &DateTime<Utc>) -> f64 {
        let year = dt.year();
        let month = dt.month() as i32;
        let day = dt.day() as i32;
        let hour = dt.hour() as f64 + dt.minute() as f64 / 60.0 + dt.second() as f64 / 3600.0;

        // Use Swiss Ephemeris built-in Julian Day calculation
        swisseph::swe::julday(year, month, day, hour, 1)
    }

    /// Calculate position for a single planet
    ///
    /// All Swiss Ephemeris C library calls are serialized via EPHE_MUTEX
    /// to prevent SIGSEGV from concurrent access to static C buffers.
    #[allow(clippy::only_used_in_recursion)]
    pub fn get_planet_position(
        &self,
        planet: HDPlanet,
        datetime: &DateTime<Utc>,
    ) -> Result<PlanetPosition, EngineError> {
        let jd = Self::datetime_to_jd(datetime);
        let planet_id = planet as i32;
        let flags = 258; // SEFLG_SPEED | SEFLG_SWIEPH

        // Handle Earth as opposite of Sun (geocentric)
        if planet_id == 14 {
            let sun = self.get_planet_position(HDPlanet::Sun, datetime)?;
            let earth_longitude = (sun.longitude + 180.0) % 360.0;
            return Ok(PlanetPosition {
                longitude: earth_longitude,
                latitude: -sun.latitude,
                distance: sun.distance,
                speed: sun.speed,
            });
        }

        // Handle South Node as opposite of North Node
        if planet_id == -(HDPlanet::NorthNode as i32) {
            let north_node = self.get_planet_position(HDPlanet::NorthNode, datetime)?;
            let south_longitude = (north_node.longitude + 180.0) % 360.0;
            return Ok(PlanetPosition {
                longitude: south_longitude,
                latitude: -north_node.latitude,
                distance: north_node.distance,
                speed: north_node.speed,
            });
        }

        // Serialize access to the Swiss Ephemeris C library (and set this
        // thread's ephemeris path on first use).
        let _guard = lock_ephemeris().unwrap();
        match swisseph::swe::calc_ut(jd, planet_id as u32, flags) {
            Ok(result) => {
                note_returned_flags(result.code);
                Ok(PlanetPosition {
                    longitude: result.out[0],
                    latitude: result.out[1],
                    distance: result.out[2],
                    speed: result.out[3],
                })
            }
            Err(e) => Err(EngineError::CalculationError(format!(
                "Swiss Ephemeris calculation failed for planet {:?}: {:?}",
                planet, e
            ))),
        }
    }

    /// Calculate all 13 planetary positions for Human Design
    pub fn get_all_planets(
        &self,
        datetime: &DateTime<Utc>,
    ) -> Result<Vec<(HDPlanet, PlanetPosition)>, EngineError> {
        let planets = [
            HDPlanet::Sun,
            HDPlanet::Earth,
            HDPlanet::Moon,
            HDPlanet::NorthNode,
            HDPlanet::SouthNode,
            HDPlanet::Mercury,
            HDPlanet::Venus,
            HDPlanet::Mars,
            HDPlanet::Jupiter,
            HDPlanet::Saturn,
            HDPlanet::Uranus,
            HDPlanet::Neptune,
            HDPlanet::Pluto,
        ];

        planets
            .iter()
            .map(|&planet| {
                let pos = self.get_planet_position(planet, datetime)?;
                Ok((planet, pos))
            })
            .collect()
    }

    /// The process ephemeris path ("" = built-in Moshier).
    pub fn data_path(&self) -> &str {
        &self.data_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh scratch directory under the system temp dir.
    fn scratch_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "hd-ephe-{label}-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn with_se1(dir: &Path) -> &Path {
        std::fs::write(dir.join("semo_18.se1"), b"stub").unwrap();
        dir
    }

    #[test]
    fn resolver_prefers_env_in_documented_order() {
        let first = scratch_dir("first");
        let second = scratch_dir("second");
        with_se1(&first);
        with_se1(&second);
        let (first_s, second_s) = (
            first.to_string_lossy().into_owned(),
            second.to_string_lossy().into_owned(),
        );
        // SWISS_EPHEMERIS_PATH (production) is used when SWISS_EPHE_PATH is unset.
        let resolved = resolve_ephemeris_path_from(
            |name| (name == "SWISS_EPHEMERIS_PATH").then(|| first_s.clone()),
            None,
            None,
        );
        assert_eq!(resolved, Some(first.canonicalize().unwrap()));
        // SWISS_EPHE_PATH wins over SWISS_EPHEMERIS_PATH.
        let resolved = resolve_ephemeris_path_from(
            |name| match name {
                "SWISS_EPHE_PATH" => Some(second_s.clone()),
                "SWISS_EPHEMERIS_PATH" => Some(first_s.clone()),
                _ => None,
            },
            None,
            None,
        );
        assert_eq!(resolved, Some(second.canonicalize().unwrap()));
    }

    #[test]
    fn resolver_skips_dirs_without_se1_files_and_empty_env() {
        let empty = scratch_dir("empty");
        let cwd = scratch_dir("cwd");
        let real = cwd.join("data/ephemeris");
        std::fs::create_dir_all(&real).unwrap();
        with_se1(&real);
        let empty_s = empty.to_string_lossy().into_owned();
        let resolved = resolve_ephemeris_path_from(
            |name| match name {
                "SWISS_EPHE_PATH" => Some(String::new()),
                "SWISS_EPHEMERIS_PATH" => Some(empty_s.clone()),
                _ => None,
            },
            Some(&cwd),
            None,
        );
        assert_eq!(resolved, Some(real.canonicalize().unwrap()));
    }

    #[test]
    fn resolver_finds_data_next_to_the_executable_regardless_of_cwd() {
        let app = scratch_dir("app");
        let data = app.join("data/ephemeris");
        std::fs::create_dir_all(&data).unwrap();
        with_se1(&data);
        let elsewhere = scratch_dir("elsewhere");
        let resolved = resolve_ephemeris_path_from(|_| None, Some(&elsewhere), Some(&app));
        assert_eq!(resolved, Some(data.canonicalize().unwrap()));
    }

    #[test]
    fn resolver_returns_none_without_any_ephemeris_files() {
        let elsewhere = scratch_dir("none");
        assert_eq!(
            resolve_ephemeris_path_from(|_| None, Some(&elsewhere), Some(&elsewhere)),
            None
        );
    }

    #[test]
    fn repository_data_directory_resolves() {
        // The checked-in data/ephemeris is found from this crate's directory.
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let resolved = resolve_ephemeris_path_from(|_| None, Some(crate_dir), None)
            .expect("repository data/ephemeris must resolve");
        assert!(resolved.is_absolute());
        assert!(resolved.join("semo_18.se1").exists());
    }

    #[test]
    fn calculation_on_a_fresh_thread_uses_the_data_files() {
        // Initialise on this thread, calculate on a newly spawned one: on Linux
        // libswisseph's path is thread-local, so a thread that never called
        // swe_set_ephe_path would silently use Moshier.
        let calc = EphemerisCalculator::new("");
        assert!(!calc.data_path().is_empty(), "repository ephemeris must resolve");
        let dt = DateTime::parse_from_rfc3339("1991-08-13T08:01:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let _ = calc.get_planet_position(HDPlanet::Sun, &dt).unwrap();

        let (moshier, raw_flags) = std::thread::spawn(move || {
            reset_moshier_observed();
            calc.get_planet_position(HDPlanet::Moon, &dt).unwrap();
            let raw = {
                let _guard = lock_ephemeris().unwrap();
                let jd = swisseph::swe::julday(1991, 8, 13, 8.0 + 1.0 / 60.0, 1);
                swisseph::swe::calc_ut(jd, HDPlanet::Moon as u32, 258)
                    .unwrap()
                    .code
            };
            (moshier_observed(), raw)
        })
        .join()
        .unwrap();
        assert_eq!(raw_flags & SEFLG_MOSEPH, 0, "flags {raw_flags} include MOSEPH");
        assert!(!moshier, "fresh thread fell back to Moshier");
    }

    #[test]
    fn moshier_observation_tracks_returned_flags_per_thread() {
        reset_moshier_observed();
        note_returned_flags(258);
        assert!(!moshier_observed());
        note_returned_flags(258 | SEFLG_MOSEPH);
        assert!(moshier_observed());
        // Another thread starts clean.
        assert!(!std::thread::spawn(moshier_observed).join().unwrap());
        reset_moshier_observed();
        assert!(!moshier_observed());
    }

    #[test]
    fn test_sun_position_j2000() {
        let calc = EphemerisCalculator::new("");
        let dt = DateTime::parse_from_rfc3339("2000-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let pos = calc.get_planet_position(HDPlanet::Sun, &dt).unwrap();

        // Sun should be around 280° on Jan 1, 2000
        assert!((pos.longitude - 280.0).abs() < 5.0);
        assert!(pos.latitude.abs() < 1.0);
    }

    #[test]
    fn test_all_planets() {
        let calc = EphemerisCalculator::new("");
        let dt = DateTime::parse_from_rfc3339("2000-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let positions = calc.get_all_planets(&dt).unwrap();

        // Should get exactly 13 planets
        assert_eq!(positions.len(), 13);

        // All longitudes should be 0-360
        for (planet, pos) in positions {
            assert!(
                pos.longitude >= 0.0 && pos.longitude < 360.0,
                "{:?} longitude out of range: {}",
                planet,
                pos.longitude
            );
        }
    }

    #[test]
    fn test_south_node_opposite_north_node() {
        let calc = EphemerisCalculator::new("");
        for timestamp in ["2000-01-01T12:00:00Z", "1990-01-01T06:30:00Z"] {
            let dt = DateTime::parse_from_rfc3339(timestamp)
                .unwrap()
                .with_timezone(&Utc);

            let north = calc.get_planet_position(HDPlanet::NorthNode, &dt).unwrap();
            let south = calc.get_planet_position(HDPlanet::SouthNode, &dt).unwrap();

            let diff = (north.longitude - south.longitude + 360.0) % 360.0;
            assert!((diff - 180.0).abs() < 0.1);
            // Adding a constant 180° preserves longitudinal velocity and motion direction.
            assert_eq!(south.speed, north.speed, "node speed at {timestamp}");
        }
    }
}
