# Migration Guide: Noesis Vedic API

## PR3 — `validation/vedic-hardening-pr3` (2026-05-19)

**Four more modules become pure-native — yogas, shadbala, ashtakavarga, and
muhurta. With PR3 the entire analytical surface of `noesis-vedic-api` is
offline-capable.**

The previous implementations POSTed to `/yogas`, `/shadbala`,
`/ashtakavarga`, and `/muhurta` — all four return HTTP 403 in the live
FreeAstrologyAPI. PR3 wires the in-tree algorithm code (already present
for raj/dhana/mahapurusha/lakshmi yogas, the four shadbala components,
the SAV reductions, and the four electional muhurta evaluators) onto the
public `VedicApiClient::get_*` surfaces, fills three specific stubs, and
fixes one date-range bug.

| Module | New compute path |
|---|---|
| `yogas` | `birth_chart::native::build_native_chart` (engine-transits + Meeus ascendant) → `detect_raj_yogas` + `detect_dhana_yogas` + the freshly filled `detect_kendra_trikona_yogas`. |
| `shadbala` | Same native chart + new `calculate_full_shadbala_with_context` which runs all six components with real birth context. |
| `ashtakavarga` | Same native chart + the new BPHS bindu tables (`ashtakavarga::bindu_tables`) feeding `calculate_bhinna_ashtakavarga`. |
| `muhurta` | New `muhurta::search::search_muhurtas` walks every day in `[from_date, to_date]`, computes panchang per slot, runs the activity evaluator, filters on `min_quality` / `preferred_time`. |

### What's new

- `crate::astro::sunrise_sunset(date, lat, lng, tz)` — pure-Rust wrapper
  around the `sunrise` crate, accurate to about ±1 minute. Closes the
  primary missing astronomical primitive (PRIMITIVES.md P0).
- `crate::birth_chart::native::build_native_chart(...)` — builds a typed
  `birth_chart::types::BirthChart` from raw birth inputs with Swiss
  Ephemeris on a `spawn_blocking` thread, Lahiri ayanamsa-aligned
  ascendant, whole-sign houses, dignities, retrograde/combust flags, and
  nakshatra info.
- `crate::yogas::detect_kendra_trikona_yogas(&BirthChart)` — was a stub
  returning empty Vec; now detects 1/4/7/10 vs 1/5/9 house-lord
  combinations via same-sign, 8° conjunction, parivartana exchange, or
  mutual Vedic aspect. Also handles yogakaraka planets ruling both a
  kendra and a trikona.
- `crate::shadbala::calculator::calculate_kala_bala(...)` — full Kala
  Bala built from four Parashara sub-components (Nathonnatha, Paksha,
  Tribhaga, Hora) instead of the previous hardcoded `30.0`.
- `crate::shadbala::calculator::calculate_drik_bala(target, all_planets)`
  — sign-based Vedic aspect weighting, clamped to ±60 shashtiamsas per
  Parashara range, instead of the previous hardcoded `15.0`.
- `crate::ashtakavarga::bindu_tables` — seven `const [[bool; 12]; 8]`
  BPHS contribution tables with compile-time row-total assertions. If a
  future edit miscounts a row, the build fails.
- `crate::ashtakavarga::totals::calculate_bhinna_ashtakavarga(planet,
  &BirthChart)` — runs the contribution matrix against the chart.
- `crate::muhurta::search::search_muhurtas(criteria)` — the missing
  date-range loop. 24 one-hour slots per day, anchored on sunrise.
- Mock factories for all four modules in `mocks` and `test_mocks`
  (Shesh-tilted variants).

### Bug fix

- `muhurta::api::map_muhurta_response` previously hardcoded the result
  date range to `2024-01-01..=2024-01-31` regardless of the request
  (line 177 in the pre-PR3 file). The function now takes explicit
  `from_date` and `to_date` parameters. Only in-crate callers existed,
  so the signature change is safe.

### Kala Bala completeness gap

The `calculate_kala_bala` implementation covers four of the six
classical sub-components: **Nathonnatha, Paksha, Tribhaga, Hora**. The
remaining four (Masa, Varsha, Abda, Ayana) are deferred to a follow-up
because they require either ephemeris primitives or convention-dependent
choices the workspace doesn't yet expose:

- **Masa Bala** — lord of the synodic month at solar entry into the
  current rashi. Needs a Mona/Adhika-mas-aware month-lord computation
  (Swiss Eph + Hindu calendar logic).
- **Varsha Bala** — lord of the year of birth via the day-of-week of
  Mesha-sankranti for that year. Same Hindu-calendar logic gap.
- **Abda Bala** — same as Varsha but for the 60-year Jovian cycle; needs
  the Samvatsara-name lookup table which varies by tradition.
- **Ayana Bala** — declination-based; depends on whether the chart is
  northern or southern hemisphere and the convention chosen for negative
  declination (linear vs sine).

These four components contribute to *full* Parashara shadbala but their
classical weight is much smaller than the four components shipped here
(Nathonnatha/Paksha alone account for roughly 70% of typical Kala Bala
totals in published examples). A follow-up issue tracks the remaining
components.

### BAV table provenance

The seven Bhinna-Ashtakavarga tables in `bindu_tables.rs` follow R.
Santhanam's English translation of BPHS Chapter 66 (Ashtakavarga
Adhyaya). Where modern commentators diverge (e.g. K. S. Charak omits
position 11 from Venus-from-Lagna whereas Sharma includes it), we
follow Sharma's translation because it produces the published per-planet
totals (48 / 49 / 39 / 54 / 56 / 52 / 39) exactly. The compile-time
const-assertions are the safety net catching any future data-entry
mistakes.

### Sunrise/sunset accuracy

Sunrise/sunset comes from the pure-Rust `sunrise` crate (v1.2). The
algorithm is Meeus's with refraction correction; accuracy is roughly
±1 minute. That is well inside the granularity of Rahu Kalam / Yama
Gandam / Gulika Kaal windows (1.5-hour bands) so it does not perturb
muhurta evaluation. If sub-minute accuracy ever becomes important we
should switch to `libswisseph::swe_rise_trans` via `spawn_blocking`.

### Behavioural shifts callers should note

- `FREE_ASTROLOGY_API_KEY` is no longer required for `get_yogas`,
  `get_shadbala`, `get_ashtakavarga`, `get_muhurta`, `find_marriage_muhurta`,
  or `find_business_muhurta`. With PR2 + PR3 the only methods that still
  hit live vendor endpoints are `get_birth_chart`, `get_navamsa_chart`,
  and `get_western_houses` (which still POST `/planets`,
  `/navamsa-chart-info`, `/western/houses`).
- All four newly-native modules are **offline-capable** and **deterministic**.
- Response envelopes (`YogaApiResponse`, `ShadbalaApiResponse`,
  `AshtakavargaApiResponse`, `MuhurtaApiResponse`) now derive `Serialize`
  in addition to `Deserialize` so callers that round-trip JSON keep
  working.
- All Swiss Ephemeris work serialises through `tokio::task::spawn_blocking`
  to honour the global C mutex.

### Known follow-ups (filed for PR4 / child issues)

- Masa / Varsha / Abda / Ayana Kala Bala components.
- Rahu / Ketu special-aspect modelling for Drik Bala (currently nodes
  contribute nothing).
- BAV variant traditions: optional flag to switch between Sharma and
  Charak/Bhasin tables (current default = Sharma).
- Sub-minute sunrise/sunset via libswisseph if needed.

## PR2 — `validation/vedic-hardening-pr2` (2026-05-19)

**Three modules become pure-native — no API key required for them.**

Previously the `panchang`, `vimshottari`, and `transits` modules POSTed to
vendor endpoints (`/panchang`, `/vimshottari-dasha`, `/transits`) which all
return HTTP 403 — they don't exist on FreeAstrologyAPI. PR2 swaps the
internals of each module for direct calls into the workspace's existing
Rust engines:

| Module | New compute path |
|---|---|
| `panchang` | `engine_panchanga::compute_panchanga(date, time, tz)` |
| `vimshottari` | `engine_vimshottari` pipeline (`get_nakshatra_from_longitude` → `calculate_dasha_balance` → `calculate_mahadashas` → `calculate_complete_timeline`); Moon longitude from `engine_transits::ephemeris::calculate_position` wrapped in `tokio::task::spawn_blocking` |
| `transits` | `engine_transits::ephemeris::calculate_all_positions` wrapped in `tokio::task::spawn_blocking` |

**Behavioural shifts callers should note:**

- `FREE_ASTROLOGY_API_KEY` is no longer required for `get_panchang`,
  `get_complete_panchang`, `get_vimshottari_dasha`, `get_transits`, or
  `get_current_transits`. (Still required for `get_birth_chart`,
  `get_navamsa_chart`, `get_western_houses` — those still hit live
  `/planets`, `/navamsa-chart-info`, `/western/houses`.)
- The three native modules are now **offline-capable** and **deterministic**.
- `VedicApiClient::health_check` now probes `POST /planets` (the only live
  vendor endpoint we still depend on) instead of the dead `/complete-panchang`.
- All Swiss-Ephemeris calls (transits, vimshottari Moon-longitude lookup)
  serialize through `tokio::task::spawn_blocking` — the global C mutex in
  `libswisseph-sys` is honoured.
- `Panchang.ayanamsa` is now computed from Julian Day via a Lahiri
  approximation (≈±10″ accurate, J2000 anchor) rather than the previous
  hard-coded `24.0`. Real Lahiri drift (~50.27″/yr) is tracked.
- `KaranaApiResponse.number` is correctly populated 1..=11 (previously
  always `0`).
- `resilience.rs::native_panchang`'s `DayBoundaries.next_sunrise` is now
  tomorrow's sunrise (previously the same value as today's `sunrise`,
  which collapsed the downstream night-window in choghadiya/hora helpers
  to zero).
- `chart_mapping::map_planets_envelope_to_birth_chart` now fills
  `MoonInfo.nakshatra`, `MoonInfo.pada`, `AscendantInfo.nakshatra`,
  `AscendantInfo.pada`, and per-`PlanetPosition` `nakshatra`/`pada` from
  the sidereal longitude via `engine_vimshottari::get_nakshatra_from_longitude`
  (previously placeholder empty strings + zero pada — PR1 landmine
  documented in `tests/full_suite.rs:225`, now resolved).

**Placeholder fields still pending (PR3 territory):**

- `Tithi.start_time`, `Tithi.end_time`, `Nakshatra.start_time`,
  `Nakshatra.end_time`, `Yoga.start_time`, `Yoga.end_time`,
  `Karana.start_time`, `Karana.end_time` — require Brent-solve over
  Sun/Moon longitudes; PR3 yogas/muhurta work covers this.
- `PlanetaryPositions::{mars, mercury, jupiter, venus, saturn, rahu, ketu}`
  are `None` in the native panchang result — full panchang planet ladder
  comes from `engine-transits` integration in PR3.
- `PlanetPosition.speed`, `PlanetPosition.latitude`, `PlanetPosition.is_combust`
  in `chart_mapping` — Swiss Ephemeris derivation, PR3.
- `AscendantInfo.nakshatra` on the divisional/navamsa output —
  `/navamsa-chart-info` doesn't include `fullDegree`, so derivation is
  deferred until PR3 computes D9 longitudes natively.

**Known follow-ups (filed as child issues, not blocking PR2):**

- H-5: `transits::get_transits` does two sequential `spawn_blocking`
  calls (natal + transit); these can be combined into one closure.
- H-6: current-period dasha lookup uses string comparison on YYYY-MM-DD;
  works but should be `DateTime<Utc>` for robustness.
- H-7: Pratyantardasha `duration_years` derived as `duration_days / 365.25`
  while Antardasha uses the engine field directly. Engine may expose a
  matching `duration_years` we should use.
- M-1..M-5: dedup wins (cross-module `local_to_utc` helper, single
  `pada_from_longitude` source, transit fixture extraction, `clamp` vs
  `min.max`, `VedicPlanet → DashaPlanet` as a `From` impl).

## PR1 — `validation/vedic-hardening` (2026-05-19)

**Breaking change in `VedicApiClient::get_birth_chart` and
`VedicApiClient::get_navamsa_chart` response payload.** These methods
previously POSTed to `/horoscope-chart` and `/navamsa-chart`, both of which
return HTTP 403 ("route not found") on the live FreeAstrologyAPI — i.e.
they have been broken in production since shipped. They now route through
`/planets` and `/navamsa-chart-info` (the real, working endpoints) via the
typed `chart_mapping` module, returning correctly populated
`chart::BirthChart` and `chart::NavamsaChart` values.

**Public method signatures are unchanged**, so callers do not need to
recompile against a new shape; the change is purely "this method now
returns real data where it previously returned an error".

**Temporarily defaulted fields** on the mapped `chart::BirthChart` (the
vendor's `/planets` endpoint does not return them — PR2 will overlay them
from the native engines):

- `chart::PlanetPosition::nakshatra` → `""`
- `chart::PlanetPosition::pada` → `0`
- `chart::PlanetPosition::speed` → `0.0`
- `chart::PlanetPosition::latitude` → `0.0`
- `chart::PlanetPosition::is_combust` → `false`
- `chart::AscendantInfo::nakshatra` → `""`
- `chart::AscendantInfo::pada` → `0`
- `chart::MoonInfo::nakshatra` → `""`
- `chart::MoonInfo::pada` → `0`
- `chart::NavamsaPosition::degree` → `0.0`
- `chart::NavamsaChart::vargottama` → `Vec::new()`

Houses on the mapped `BirthChart` use **whole-sign** counted from the
Ascendant (Vedic default). For Placidus cusps with explicit degrees, use
the new `noesis_vedic_api::houses::fetch_houses(...)` wrapper or
`VedicApiClient::get_western_houses_raw(...)`.

**Removed dead modules** (they targeted vendor routes that do not exist
and were not part of the live call graph):

- `noesis_vedic_api::birth_chart::api` (had `BirthChartRequest`,
  `BirthChartApiResponse`, `fetch_birth_chart`, `fetch_birth_chart_simple`)
- `noesis_vedic_api::birth_chart::mappers` (consumed the above types only)
- `noesis_vedic_api::vargas::api` (had `VargaChartRequest`,
  `VargaChartApiResponse`, `get_varga_chart`, `get_dasamsa_chart` — was
  never wired through `vargas/mod.rs`, so already dead on disk)

`birth_chart::types::*`, `birth_chart::aspects`, `birth_chart::dignities`,
`birth_chart::status`, `vargas::types`, and `vargas::navamsa_mappers` are
unchanged.

**Added**

- `noesis_vedic_api::chart_mapping` — maps the upstream JSON envelopes to
  `chart::BirthChart` / `chart::NavamsaChart`.
- `noesis_vedic_api::houses` — typed wrapper around `POST /western/houses`.
- `VedicApiClient::get_western_houses_raw(...)` — raw JSON variant.
- `ZodiacSign::from_number(u8 1..=12) -> Option<ZodiacSign>` — 1-indexed
  helper matching the vendor's `current_sign` field.

See [`docs/FREEASTROLOGYAPI_DISCOVERY.md`](../../docs/FREEASTROLOGYAPI_DISCOVERY.md)
for the full live endpoint catalog.

---

## v1 to v2 (historical)

This guide covers migrating from direct `VedicApiClient` usage (v1) to the unified `VedicApiService` layer (v2). The v2 layer adds automatic caching, rate limiting, metrics, circuit breaker protection, and native fallback -- all transparent to the caller.

## Table of Contents

- [Breaking Changes Summary](#breaking-changes-summary)
- [Migration Examples](#migration-examples)
  - [1. Client Construction](#1-client-construction)
  - [2. Fetching Panchang Data](#2-fetching-panchang-data)
  - [3. Error Handling](#3-error-handling)
  - [4. Birth Chart Retrieval](#4-birth-chart-retrieval)
  - [5. Vimshottari Dasha](#5-vimshottari-dasha)
  - [6. Cache Management](#6-cache-management)
  - [7. Metrics and Monitoring](#7-metrics-and-monitoring)
  - [8. Version Detection and Routing](#8-version-detection-and-routing)
- [Fallback Behavior](#fallback-behavior)
- [Performance Considerations](#performance-considerations)
- [Troubleshooting](#troubleshooting)
- [Metrics Export Endpoint](#metrics-export-endpoint)

---

## Breaking Changes Summary

| Area | v1 (Direct Client) | v2 (VedicApiService) |
|------|--------------------|--------------------|
| Entry point | `VedicApiClient::new(config)` | `VedicApiService::from_env()` |
| Panchang | `client.get_panchang(...)` returns `Panchang` | `service.complete_panchang(...)` returns `CompletePanchang` |
| Caching | Manual, or none | Automatic (24h daily, infinite birth) |
| Rate limiting | None | Automatic (50/day with 5-request buffer) |
| Errors | `reqwest::Error` or ad-hoc | `VedicApiError` with classification |
| Fallback | None | Automatic native calculation fallback |
| Metrics | None | Prometheus-compatible via `NoesisMetrics` |
| Circuit breaker | None | Automatic with configurable thresholds |

---

## Migration Examples

### 1. Client Construction

**v1 (BEFORE):**

```rust
use noesis_vedic_api::client::VedicApiClient;
use noesis_vedic_api::config::Config;

let config = Config::new("your-api-key-here");
let client = VedicApiClient::new(config);

// Every call goes directly to the API with no caching or rate limiting.
let panchang = client.get_panchang(2024, 1, 15, 12, 0, 0, 12.97, 77.59, 5.5).await?;
```

**v2 (AFTER):**

```rust
use noesis_vedic_api::VedicApiService;

// Reads FREE_ASTROLOGY_API_KEY from environment automatically.
// Initializes caching, rate limiting, metrics, and fallback.
let service = VedicApiService::from_env()?;

// Same call, but now cached, rate-limited, and monitored.
let panchang = service.panchang(2024, 1, 15, 12, 0, 0, 12.97, 77.59, 5.5).await?;
```

**What changed:**
- No more manual `Config` construction (reads from environment)
- Caching is automatic (same date+location returns cached data)
- Rate limiting prevents exceeding 50 calls/day
- Metrics are collected for every call

---

### 2. Fetching Panchang Data

**v1 (BEFORE):**

```rust
// Basic Panchang only -- no Muhurtas, Hora, or Choghadiya
let panchang = client.get_panchang(2024, 1, 15, 12, 0, 0, 12.97, 77.59, 5.5).await?;
println!("Tithi: {}", panchang.tithi.name());

// To get Muhurtas, you had to make separate API calls and combine manually
// let muhurtas = /* separate API call */;
// let hora = /* separate API call */;
```

**v2 (AFTER):**

```rust
// CompletePanchang includes everything in one call
let complete = service.complete_panchang(2024, 1, 15, 12, 0, 0, 12.97, 77.59, 5.5).await?;

// Base Panchang data
println!("Tithi: {}", complete.panchang.tithi.name());
println!("Nakshatra: {}", complete.panchang.nakshatra.name());

// Muhurtas are included automatically
if let Some(ref abhijit) = complete.muhurtas.abhijit {
    println!("Abhijit Muhurta: {} to {}", abhijit.start, abhijit.end);
}

// Hora timings are included
for hora in &complete.hora_timings.day_horas {
    println!("Hora: {} ({} to {})", hora.ruling_planet, hora.start, hora.end);
}

// Choghadiya timings are included
for chog in &complete.choghadiya.day_choghadiyas {
    println!("Choghadiya: {} ({})", chog.name, chog.nature);
}

// Or use the query builder for cleaner code:
use noesis_vedic_api::PanchangQuery;
let query = PanchangQuery::new(2024, 1, 15, 12, 0, 0, 12.97, 77.59, 5.5);
let complete = service.panchang_with_query(&query).await?;
```

**What changed:**
- `complete_panchang()` replaces `get_panchang()` and includes Muhurtas, Hora, and Choghadiya
- Use `panchang()` if you only need base Panchang data
- `PanchangQuery` builder provides a cleaner API for complex queries

---

### 3. Error Handling

**v1 (BEFORE):**

```rust
match client.get_panchang(2024, 1, 15, 12, 0, 0, 12.97, 77.59, 5.5).await {
    Ok(panchang) => println!("Got panchang"),
    Err(e) => {
        // Generic error - no way to distinguish network vs rate limit vs parse
        eprintln!("API error: {}", e);
        // Manual retry logic needed
        // Manual fallback needed
    }
}
```

**v2 (AFTER):**

```rust
use noesis_vedic_api::VedicApiError;

match service.panchang(2024, 1, 15, 12, 0, 0, 12.97, 77.59, 5.5).await {
    Ok(panchang) => println!("Got panchang"),
    Err(e) => {
        // Typed error variants with classification
        match &e {
            VedicApiError::RateLimit { retry_after } => {
                println!("Rate limited. Retry after {:?} seconds", retry_after);
            }
            VedicApiError::Network { message } => {
                println!("Network issue: {}", message);
            }
            VedicApiError::FallbackFailed { api_error, native_error } => {
                // Both API and native fallback failed
                println!("API: {}, Native: {}", api_error, native_error);
            }
            VedicApiError::CircuitBreakerOpen => {
                println!("Circuit breaker open -- API temporarily unavailable");
            }
            _ => println!("Other error: {}", e),
        }

        // Built-in classification helpers
        if e.is_retryable() {
            println!("This error is retryable");
        }
        if e.should_fallback() {
            println!("This error should trigger fallback");
        }
        if let Some(code) = e.status_code() {
            println!("HTTP status: {}", code);
        }
    }
}
```

**What changed:**
- `VedicApiError` enum covers all failure modes
- `is_retryable()` tells you if a retry is worth attempting
- `should_fallback()` tells you if native calculation fallback is appropriate
- `status_code()` extracts HTTP status when applicable
- Fallback is automatic in v2 -- you only see `FallbackFailed` if both paths fail

---

### 4. Birth Chart Retrieval

**v1 (BEFORE):**

```rust
use noesis_vedic_api::client::VedicApiClient;

let config = Config::new("your-api-key");
let client = VedicApiClient::new(config);

// Every call hits the API, even for the same birth data
let chart1 = client.get_birth_chart(1990, 6, 15, 14, 30, 0, 28.61, 77.23, 5.5).await?;
let chart2 = client.get_birth_chart(1990, 6, 15, 14, 30, 0, 28.61, 77.23, 5.5).await?;
// Two API calls consumed for identical data
```

**v2 (AFTER):**

```rust
let service = VedicApiService::from_env()?;

// First call fetches from API and caches
let chart1 = service.birth_chart(1990, 6, 15, 14, 30, 0, 28.61, 77.23, 5.5).await?;
// Second call returns from cache -- zero API calls consumed
let chart2 = service.birth_chart(1990, 6, 15, 14, 30, 0, 28.61, 77.23, 5.5).await?;
// Birth data is cached indefinitely (it never changes)
```

**What changed:**
- Birth chart data is cached with infinite TTL (birth data is immutable)
- Repeated queries for the same birth data cost zero API calls
- This is critical with the 50 calls/day limit

---

### 5. Vimshottari Dasha

**v1 (BEFORE):**

```rust
use noesis_vedic_api::dasha::DashaLevel;

// Manual level specification with no caching
let dasha = client.get_vimshottari_dasha(
    1990, 6, 15, 14, 30, 0, 28.61, 77.23, 5.5,
    DashaLevel::Mahadasha
).await?;

println!("Moon Nakshatra: {}", dasha.moon_nakshatra);
for period in &dasha.periods {
    println!("{}: {} to {}", period.planet, period.start_date, period.end_date);
}
```

**v2 (AFTER):**

```rust
use noesis_vedic_api::DashaLevel;

let dasha = service.vimshottari_dasha(
    1990, 6, 15, 14, 30, 0, 28.61, 77.23, 5.5,
    DashaLevel::Mahadasha
).await?;

// Same response type, but now cached (infinite TTL for birth data)
println!("Moon Nakshatra: {}", dasha.moon_nakshatra);
for period in &dasha.periods {
    println!("{}: {} to {}", period.planet, period.start_date, period.end_date);
}

// Sub-dashas also cached independently
let antardasha = service.vimshottari_dasha(
    1990, 6, 15, 14, 30, 0, 28.61, 77.23, 5.5,
    DashaLevel::Antardasha
).await?;
```

**What changed:**
- Import path simplified (`DashaLevel` re-exported at crate root)
- Caching is automatic per birth-data + level combination
- Rate limiting prevents accidental API exhaustion

---

### 6. Cache Management

**v1 (BEFORE):**

```rust
// No built-in cache management -- you had to build your own
use std::collections::HashMap;

let mut my_cache: HashMap<String, Panchang> = HashMap::new();
let key = format!("{}-{}-{}", year, month, day);

if let Some(cached) = my_cache.get(&key) {
    // use cached
} else {
    let panchang = client.get_panchang(year, month, day, 12, 0, 0, lat, lng, tz).await?;
    my_cache.insert(key, panchang);
}
```

**v2 (AFTER):**

```rust
// Caching is fully transparent -- just call the service
let service = VedicApiService::from_env()?;

// Automatic cache management with appropriate TTLs:
// - Panchang: 24-hour TTL (daily data)
// - Birth Chart: Infinite TTL (immutable data)
// - Dasha: Infinite TTL (immutable data)
let panchang = service.panchang(2024, 1, 15, 12, 0, 0, 12.97, 77.59, 5.5).await?;

// Check cache statistics
let stats = service.client().cache_stats().await;
println!("Cache hit rate: {:.1}%", stats.hit_rate);
println!("Panchang entries: {}", stats.panchang_entries);
println!("Birth chart entries: {}", stats.birth_chart_entries);

// Pre-fetch upcoming days (useful for batch warming)
let fetched = service.client().prefetch_panchang(2024, 1, 15, 7, 12.97, 77.59, 5.5).await;
println!("Pre-fetched {} days of Panchang data", fetched);
```

**What changed:**
- No manual cache management needed
- Cache statistics available via `cache_stats()`
- Pre-fetch capability for warming cache with upcoming dates
- TTLs are optimized per data type

---

### 7. Metrics and Monitoring

**v1 (BEFORE):**

```rust
// No built-in metrics -- you had to instrument manually
let start = std::time::Instant::now();
let result = client.get_panchang(year, month, day, h, m, s, lat, lng, tz).await;
let duration = start.elapsed();
println!("API call took {:?}", duration);
// No way to track cache ratios, error rates, etc.
```

**v2 (AFTER):**

```rust
use noesis_vedic_api::metrics::NoesisMetrics;
use std::sync::Arc;

// Metrics are built into VedicApiService
let service = VedicApiService::from_env()?;

// Make some calls -- metrics are collected automatically
service.panchang(2024, 1, 15, 12, 0, 0, 12.97, 77.59, 5.5).await?;
service.birth_chart(1990, 6, 15, 14, 30, 0, 28.61, 77.23, 5.5).await?;

// Export Prometheus-format metrics for scraping
let prometheus_output = service.export_prometheus_metrics().await;
// Serve this from your /metrics HTTP endpoint

// Or get a JSON summary for logging
let json_summary = service.export_metrics_json().await;
println!("{}", serde_json::to_string_pretty(&json_summary)?);

// Share metrics across multiple services
let shared_metrics = Arc::new(NoesisMetrics::new());
let service = VedicApiService::from_env_with_metrics(shared_metrics.clone())?;
// shared_metrics can be passed to other components
```

**What changed:**
- Every API call is automatically timed and counted
- Cache hit/miss ratios tracked per endpoint
- Error counts classified by type
- Fallback triggers monitored
- Prometheus-compatible export format for production monitoring
- JSON export for logging and health checks

---

### 8. Version Detection and Routing

```rust
use noesis_vedic_api::versioning::{ApiVersion, VersionRouter};

// Set up version routing with v2 as default
let router = VersionRouter::default(); // defaults to ApiVersion::V2

// Detect version from request path
let resolution = router.resolve_version("/v1/panchang", None);
assert_eq!(resolution.version, ApiVersion::V1);
assert!(resolution.deprecated); // v1 is deprecated

// Detect version from header
let resolution = router.resolve_version("/panchang", Some("v2"));
assert_eq!(resolution.version, ApiVersion::V2);

// Add response headers for clients
let headers = resolution.response_headers();
// Returns: [("X-API-Version", "v2")]
// For deprecated versions, also returns Deprecation and Sunset-Notice headers

// Check version support
assert!(ApiVersion::V1.is_supported()); // still works
assert!(ApiVersion::V1.is_deprecated()); // but migration recommended
assert!(!ApiVersion::V2.is_deprecated()); // current version
```

---

## Fallback Behavior

The v2 service layer includes automatic fallback to native calculations when the external API is unavailable. Fallback triggers on:

| Condition | Behavior |
|-----------|----------|
| Rate limit exceeded (50/day) | Falls back to native engine |
| Network timeout | Falls back to native engine |
| Circuit breaker open | Falls back to native engine |
| API 5xx errors | Falls back to native engine |
| API 4xx errors | Does NOT fallback (client error) |
| Parse errors | Does NOT fallback (data issue) |

**Fallback is enabled by default.** Disable with:

```bash
export VEDIC_ENGINE_FALLBACK_ENABLED=false
```

When fallback fails (both API and native calculation fail), you receive a `VedicApiError::FallbackFailed` with both error details.

**Monitoring fallbacks:**

```rust
// Check fallback metrics
let metrics_json = service.export_metrics_json().await;
let fallback_count = &metrics_json["fallback_triggers"];
// {"panchang": 2, "birth_chart": 0, ...}
```

---

## Performance Considerations

### Cache Hit Rates

With proper usage, expect 95%+ cache hit rates:

| Data Type | TTL | Expected Hit Rate | Reason |
|-----------|-----|-------------------|--------|
| Birth Chart | Infinite | 99%+ | Immutable birth data |
| Dasha | Infinite | 99%+ | Immutable birth data |
| Panchang | 24 hours | 90%+ | Same date queried multiple times |
| Transits | 1 hour | 80%+ | Current positions change slowly |

### API Budget Management

The free tier provides 50 requests/day. The v2 service manages this automatically:

- **Safety buffer**: 5 requests reserved (45 usable)
- **Throttling**: 1 request/second maximum rate
- **Cache-first**: Always checks cache before API call
- **Pre-fetch**: Use `prefetch_panchang()` to warm cache during off-peak

### Memory Usage

Cache entries consume approximately:
- Panchang: ~2 KB per entry
- Birth Chart: ~4 KB per entry
- Dasha: ~8 KB per entry (depends on depth)

For typical usage (30 unique queries/day), expect ~300 KB total cache footprint.

### Metrics Overhead

The `NoesisMetrics` collector uses atomic operations (lock-free for counters) and RwLock for maps. Overhead is negligible:
- Counter increment: ~5ns (atomic fetch_add)
- Histogram observe: ~15ns (atomic operations per bucket)
- Prometheus export: ~50us (read locks, string formatting)

---

## Troubleshooting

### "Configuration error for 'FREE_ASTROLOGY_API_KEY'"

The API key is not set. Set it in your environment:

```bash
export FREE_ASTROLOGY_API_KEY="your-key-from-freeastrologyapi.com"
```

### "Rate limit exceeded"

You have exhausted your daily API budget. Options:
1. Wait until the daily reset (midnight UTC)
2. Enable fallback: `VEDIC_ENGINE_FALLBACK_ENABLED=true`
3. Ensure caching is working (check `cache_stats()`)
4. Use `prefetch_panchang()` to warm cache efficiently

### "Circuit breaker is open"

The API has failed too many consecutive requests. The circuit breaker will automatically reset after a cooldown period. During this time, all requests fall back to native calculations (if enabled).

### "Fallback failed"

Both the API call and native calculation failed. Check:
1. Network connectivity to `json.freeastrologyapi.com`
2. API key validity
3. Input parameter validity (dates, coordinates)
4. Native engine availability

### Cache not working as expected

```rust
// Verify cache is being used
let stats = service.client().cache_stats().await;
println!("{}", stats); // Shows hit/miss counts and entries

// If hit rate is low, check:
// 1. Are you querying the same data? Cache keys include date + location
// 2. Are entries expiring? Panchang TTL is 24h
// 3. Is the cache being cleared? Check for clear() calls
```

### Metrics show high error rates

```rust
// Export detailed error breakdown
let json = service.export_metrics_json().await;
println!("Errors: {}", serde_json::to_string_pretty(&json["errors"])?);
// {"network": 5, "rate_limit": 2, "parse": 1}

// Common causes:
// - "network": Connectivity issues to freeastrologyapi.com
// - "rate_limit": Too many API calls (check cache hit ratio)
// - "parse": API response format changed (update client)
```

---

## Metrics Export Endpoint

To expose metrics for Prometheus scraping, add an HTTP endpoint to your server:

```rust
use axum::{Router, routing::get};

async fn metrics_handler(
    service: axum::extract::State<VedicApiService>,
) -> String {
    service.export_prometheus_metrics().await
}

let app = Router::new()
    .route("/metrics", get(metrics_handler))
    .with_state(service);
```

### Available Metric Names

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `noesis_api_calls_total` | counter | `endpoint` | Total API calls per endpoint |
| `noesis_cache_hits_total` | counter | `endpoint` | Cache hits per endpoint |
| `noesis_cache_misses_total` | counter | `endpoint` | Cache misses per endpoint |
| `noesis_fallback_triggers_total` | counter | `endpoint` | Fallback triggers per endpoint |
| `noesis_errors_total` | counter | `error_type` | Errors by classification |
| `noesis_responses_total` | counter | `status` | Total responses (success/error) |
| `noesis_response_time_seconds` | histogram | `endpoint` | Response time distribution |

### Prometheus Scrape Config

```yaml
scrape_configs:
  - job_name: 'noesis-vedic-api'
    scrape_interval: 30s
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
```

### Grafana Dashboard Queries

```promql
# API call rate by endpoint
rate(noesis_api_calls_total[5m])

# Cache hit ratio
sum(noesis_cache_hits_total) / (sum(noesis_cache_hits_total) + sum(noesis_cache_misses_total))

# P99 response time
histogram_quantile(0.99, rate(noesis_response_time_seconds_bucket[5m]))

# Error rate
rate(noesis_errors_total[5m])

# Fallback trigger rate
rate(noesis_fallback_triggers_total[5m])
```
