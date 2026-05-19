# ENGINES.md — Vedic Engine API Map for noesis-vedic-api PR2

## Workspace Position

All three engines are declared workspace members in the root `Cargo.toml`. None of `engine-panchanga`, `engine-vimshottari`, or `engine-transits` appear in `noesis-vedic-api/Cargo.toml` `[dependencies]`. Adding them as `path = "../engine-panchanga"` etc. introduces no circular dependency: `noesis-vedic-api` currently depends only on `tokio`, `reqwest`, `serde`, `chrono`, `lru`, `config`, `tracing`, `tokio-retry`, and `backoff`.

---

## engine-panchanga

**File:** `crates/engine-panchanga/src/lib.rs` (single-file crate)

### Public surface

```
pub struct PanchangaResult         // lib.rs:179
pub struct PanchangaEngine         // lib.rs:446
pub fn compute_panchanga           // lib.rs:337
pub fn calculate_julian_day        // lib.rs:229
pub fn calculate_solar_position    // lib.rs:258
pub fn calculate_lunar_position    // lib.rs:270
pub fn calculate_tithi             // lib.rs:283
pub fn calculate_nakshatra         // lib.rs:292
pub fn calculate_yoga              // lib.rs:297
pub fn calculate_karana            // lib.rs:322
pub fn calculate_vara              // lib.rs:327
pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput}
```

### Primary "compute everything" function

```rust
pub fn compute_panchanga(date: &str, time: &str, tz_offset_hours: f64) -> PanchangaResult
```

- `date`: `"YYYY-MM-DD"` string
- `time`: `"HH:MM"` string
- `tz_offset_hours`: raw `f64` UTC offset (e.g. `5.5` for IST)
- Synchronous, no `async`

### PanchangaResult top-level fields

`tithi_index: u8`, `tithi_name: String`, `tithi_value: f64`,
`nakshatra_index: u8`, `nakshatra_name: String`, `nakshatra_value: f64`,
`yoga_index: u8`, `yoga_name: String`, `yoga_value: f64`,
`karana_index: u8`, `karana_name: String`, `karana_value: f64`,
`vara_index: u8`, `vara_name: String`,
`solar_longitude: f64`, `lunar_longitude: f64`, `julian_day: f64`

### ConsciousnessEngine conformance

`PanchangaEngine` implements `ConsciousnessEngine`. `engine_id = "panchanga"`, `required_phase = 0`.
Default mode reads `birth_data.{date,time,timezone}`.
`options["mode"] = "daily"` flips to `current_time` date.

### Nakshatra name string (PR1 landmine fix)

`compute_panchanga(...).nakshatra_name` → `String` e.g. `"Uttara Phalguni"`.
**Pada derivation:** `((result.nakshatra_value.fract() * 4.0).floor() as u8) + 1`

---

## engine-vimshottari

**Files:** `src/{lib,engine,calculator,models,wisdom,witness,wisdom_data}.rs`

### Public surface

```
pub use engine::VimshottariEngine
pub use calculator::{
    calculate_antardashas, calculate_birth_nakshatra, calculate_complete_timeline,
    calculate_dasha_balance, calculate_mahadashas, calculate_pratyantardashas,
    enrich_period_with_qualities, get_nakshatra, get_nakshatra_from_longitude,
}
pub use models::*
pub use wisdom_data::*
pub use witness::generate_witness_prompt
pub use noesis_core::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput}
```

### 120-year timeline pipeline

1. `get_nakshatra_from_longitude(moon_longitude_f64) -> &'static Nakshatra` (calculator.rs:341)
2. `calculate_dasha_balance(moon_longitude, &nakshatra) -> f64` (calculator.rs:360)
3. `calculate_mahadashas(birth_time: DateTime<Utc>, starting_planet: VedicPlanet, balance: f64) -> Vec<Mahadasha>` (calculator.rs:395)
4. `calculate_complete_timeline(mahadashas) -> Vec<Mahadasha>` (calculator.rs:566)

Alternative: `calculate_birth_nakshatra(birth_time: DateTime<Utc>, ephe_path: &str) -> Result<Nakshatra, EngineError>` derives Moon longitude internally via Swiss Ephemeris.

### Key models

`Mahadasha { planet: VedicPlanet, start_date: DateTime<Utc>, end_date: DateTime<Utc>, duration_years: f64, antardashas: Vec<Antardasha> }`
`Antardasha { planet, start_date, end_date, duration_years, pratyantardashas: Vec<Pratyantardasha> }`
`Pratyantardasha { planet, start_date, end_date, duration_days }`
`VimshottariChart { birth_date, mahadashas, current_period, upcoming_transitions }`

### ConsciousnessEngine

`engine_id = "vimshottari"`, `required_phase = 2`.
Two modes:
- `birth_data` present → Swiss Ephemeris derives Moon longitude, converts to Lahiri sidereal
- `options["moon_longitude"]: f64` + `options["birth_date"]: String`

---

## engine-transits

**Files:** `src/{lib,engine,ephemeris,models,aspects,sade_sati,witness}.rs`

### Public surface

```
pub mod aspects, engine, ephemeris, models, sade_sati, witness
pub use engine::TransitsEngine
```

Access pattern: `engine_transits::ephemeris::calculate_all_positions`.

### Primary position function

```rust
pub fn calculate_all_positions(
    calculator: &EphemerisCalculator,
    datetime: &DateTime<Utc>,
) -> Result<Vec<PlanetaryPosition>, EngineError>
```

Returns 12 sidereal positions (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Rahu, Ketu). Synchronous.

### PlanetaryPosition shape

`{ planet: TransitPlanet, longitude: f64, latitude: f64, speed: f64, sign: ZodiacSign, degree_in_sign: f64, is_retrograde: bool }`

### Backend

Uses `libswisseph-sys` C FFI + `engine-human-design`'s `EphemerisCalculator` and `EPHE_MUTEX` to serialize global Swiss Ephemeris state. `TransitsEngine::new()` creates `EphemerisCalculator` internally, holds it as a field → `TransitsEngine` is **not** `Clone`.

### ConsciousnessEngine

`engine_id = "transits"`, `required_phase = 0`. Input parses `birth_data.timezone` as IANA tz via `chrono_tz`.

---

## External Callers (within Selemene-engine)

**engine-panchanga:**
- `crates/noesis-integration/tests/panchanga_snapshot_tests.rs`
- `crates/noesis-integration/tests/biorhythm_workflow_tests.rs`
- `crates/noesis-api/tests/edge_case_snapshot_tests.rs`
- `crates/noesis-orchestrator/tests/trait_conformance_tests.rs`

**engine-vimshottari:**
- `crates/noesis-integration/src/lib.rs:64` — re-exports `VimshottariEngine` types as public integration surface
- `crates/noesis-api/benches/engine_comparison.rs`
- `crates/noesis-orchestrator/tests/trait_conformance_tests.rs`

**engine-transits:**
- `crates/noesis-orchestrator/tests/trait_conformance_tests.rs`
- `crates/engine-transits/benches/transits_bench.rs`

---

## Wiring Plan for noesis-vedic-api Facade (PR2)

### Cargo.toml additions

```toml
engine-panchanga    = { path = "../engine-panchanga" }
engine-vimshottari  = { path = "../engine-vimshottari" }
engine-transits     = { path = "../engine-transits" }
```

No circular dependencies.

### Minimum facade surface

1. **PR1 placeholder fill** — `chart_mapping.rs` lines 81/88/123: replace `nakshatra: String::new(), pada: 0` with `engine_panchanga::compute_panchanga(date, time, tz_offset).nakshatra_name` and pada derivation.
2. **Vimshottari passthrough** — `vimshottari/api.rs::get_vimshottari_dasha` delegates to `engine_vimshottari::VimshottariEngine` (full pipeline) or directly to `calculate_mahadashas` + `calculate_complete_timeline`.
3. **Transits passthrough** — `transits/api.rs::get_transits` delegates to `engine_transits::TransitsEngine` or `calculate_all_positions`.

### Type bridging

| noesis-vedic-api type | Engine native type | Bridge |
|---|---|---|
| `dasha::DashaPlanet` | `engine_vimshottari::VedicPlanet` | enum name mapping |
| `dasha::DashaPeriod` | `engine_vimshottari::Mahadasha` | field rename + RFC3339 dates |
| `panchang::Nakshatra` | `PanchangaResult::nakshatra_name: String` | wrap |
| `chart::MoonInfo.nakshatra` | `PanchangaResult::nakshatra_name` | direct |
| `transits::types::*` | `engine_transits::models::TransitAnalysisResult` | serde flatten |

### Sync vs async

All three engines expose sync hot paths. `noesis-vedic-api` is a `tokio` async crate. Two options:
- Use the `async ConsciousnessEngine::calculate()` path (cleaner error handling)
- Wrap sync calls with `tokio::task::spawn_blocking` (mandatory for `engine-transits` Swiss Ephemeris C FFI calls)
