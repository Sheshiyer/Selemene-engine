# FACADE_TARGETS.md — noesis-vedic-api PR2

## Module: `panchang`

### Public type surface (crate-root re-exports)

| Type | Top-level fields |
|---|---|
| `CompletePanchang` | `panchang, muhurtas, hora_timings, choghadiya, metadata` |
| `Panchang` | `date, location, tithi, nakshatra, yoga, karana, vara, paksha, planets, day_boundaries, ayanamsa: f64` |
| `Tithi` | `number: u8, name_tithi: TithiName, start_time/end_time: String, is_complete: bool` |
| `Nakshatra` | `number: u8, name_nakshatra: NakshatraName, pada: u8, start_time/end_time: String, longitude: f64` |
| `Yoga` | `number: u8, name_yoga: YogaName, start_time/end_time` |
| `Karana` | `name_karana: KaranaName, karana_type: KaranaType, start_time/end_time` |
| `DayBoundaries` | `sunrise/sunset/next_sunrise/day_duration/night_duration: String` (HH:MM format — **contract**) |
| `MuhurtaCollection` | 8 `Option<Muhurta>` fields |
| `HoraTimings`, `ChoghadiyaTimings` | locally-computed, don't break post-swap |
| Enums | `Vara`, `Paksha`, `TithiName` (16), `NakshatraName` (27), `YogaName` (27), `KaranaName` (11), `KaranaType` |

### API-side surface (`VedicApiClient`)

| Method | Endpoint | Returns |
|---|---|---|
| `get_panchang_raw(&PanchangApiRequest)` | GET `/panchang` (dead) | `PanchangApiResponse` |
| `get_sunrise_sunset` | GET `/sunrise-sunset` (dead) | `SunriseSunsetResponse` |
| `get_panchang_for_date/_datetime` | delegates to above | `PanchangApiResponse` |

### Cached-client + service

`CachedVedicClient`: `get_panchang(9)`, `get_complete_panchang(9)`, `get_panchang_with_query(&PanchangQuery)`, `get_muhurtas(6)`, `get_hora_timings(6)`, `get_choghadiya(6)`, `get_current_muhurta(8)`, `get_favorable_muhurtas(6)`, `prefetch_panchang(7)`.

`VedicApiService`: `panchang`, `complete_panchang`, `panchang_with_query`, `panchang_with_fallback`, `batch_panchang`.

**`get_complete_panchang` already calls `calculate_muhurtas`, `calculate_hora_timings`, `calculate_choghadiya` locally** — only the base `Panchang` retrieval is broken.

---

## Module: `vimshottari`

### Public type surface

`vimshottari/mod.rs` re-exports `enrichment`, `query` submodules. Canonical contract types live in `crate::dasha`:

| Type | Top-level fields |
|---|---|
| `VimshottariDasha` | `birth_date: String, moon_nakshatra: String, moon_longitude: f64, balance: DashaBalance, mahadashas: Vec<DashaPeriod>, current_mahadasha: DashaPeriod, current_antardasha/pratyantardasha/sookshma: Option<DashaPeriod>` |
| `DashaPeriod` | `planet: DashaPlanet, level: DashaLevel, start_date/end_date: String (YYYY-MM-DD!), duration_years: f64, duration_days: i64, sub_periods: Option<Vec<DashaPeriod>>` |
| `DashaBalance` | `planet, years/months/days_remaining: f64, total_period_years: f64` |
| `DashaPlanet` enum | Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury, Ketu, Venus |
| `DashaLevel` enum | Mahadasha, Antardasha, Pratyantardasha, Sookshma, Prana |

**Parallel engine-native types** in `vimshottari/types.rs::DashaPeriod` use `NaiveDate` — these are NOT the API contract.

### API-side

`get_vimshottari_dasha(&VimshottariRequest)` → POST `/vimshottari-dasha` (dead).
`get_mahadasha_only`, `get_antardasha_level`, `get_pratyantardasha_level`, `get_sookshma_level` — all delegate.

### Cached + service

`CachedVedicClient::get_vimshottari_dasha(9 args + DashaLevel) -> Result<VimshottariDasha>` (caches by birth_key+level, infinite TTL).
`VedicApiService::vimshottari_dasha`, `vimshottari_dasha_with_fallback`.

---

## Module: `transits`

### Public type surface

`transits/mod.rs` re-exports `api::*, aspects::*, jupiter::*, predictions::*, sade_sati::*, types::*`. **NOT re-exported at crate root.**

| Type | Top-level fields |
|---|---|
| `TransitAnalysis` | `analysis_date: NaiveDate, current_transits, significant_aspects, sade_sati_status: Option, jupiter_transit: Option, period_quality: PeriodQuality, upcoming_dates` |
| `TransitEvent` | `transiting_planet/sign: String, degree: f64, start_date/end_date, is_retrograde, aspects` |
| `TransitAspect` | `natal_planet/natal_sign, aspect_type: AspectType, is_applying, orb, nature` |
| `SadeSatiStatus` | `is_active, phase, start/end_date, saturn_sign/moon_sign` |
| `JupiterTransitStatus` | `current_sign, from_ascendant/from_moon, quality, affected_areas` |

### API-side

`get_transits(&TransitRequest)` → POST `/transits` (dead).
`get_current_transits(birth_dt, lat, lng, tz)` delegates.

### Cached + service

**Neither `CachedVedicClient` nor `VedicApiService` expose transit-specific public methods.** Cleanest module to swap — no cache layer to preserve.

---

## Downstream callers (outside the crate)

| File:line | Call | Notes |
|---|---|---|
| `crates/noesis-integration/src/analysis.rs:241` | `client.get_complete_panchang(date.year, ..., 5.5)` | tz HARDCODED to 5.5 IST — see Risk 3 |
| `crates/noesis-integration/src/lib.rs:159` | `client.get_complete_panchang(now.year, ..., lat, lng, tz)` | proper tz pass-through |

**No external callers of `get_transits` or `get_vimshottari_dasha`.**

---

## `noesis-integration::fetch_panchang` call chain

```
fetch_panchang(profile)
  → CachedVedicClient::get_complete_panchang(y,mo,d,h,mi,0, lat,lng, 5.5)
      → cache miss path:
          → CachedVedicClient::get_panchang(...) → VedicApiClient::get_panchang(...)
              → POST /panchang   ← 403, dead
          → calculate_muhurtas(day_of_week, sunrise, sunset)   ← local, works
          → calculate_hora_timings(...)                          ← local, works
          → calculate_choghadiya(...)                            ← local, works
      → CompletePanchang { panchang, muhurtas, hora_timings, choghadiya, metadata }
```

PR2 must replace `VedicApiClient::get_panchang(...)` internals with native `engine-panchanga::compute_panchanga` + mapping to the `Panchang` struct.

---

## Test surface

### Inline `#[cfg(test)]`

- `panchang/mod.rs`: parse_tithi/parse_nakshatra tests
- `panchang/data.rs`: enum display tests
- `vimshottari/api.rs`: request creation, parse_dasha_lord
- `vimshottari/types.rs`: dasha lord arithmetic
- `transits/api.rs`: request creation
- `transits/types.rs`: enum tests

### `tests/integration_tests.rs` wiremock tests (need rewrite or deletion)

- `panchang_tests::*` (6 tests) — mock `/panchang` HTTP
- `dasha_tests::*` (4 tests) — mock `/vimshottari-dasha` HTTP
- `fallback_service_tests::*` (3 tests) — `panchang_with_fallback`, `vimshottari_with_fallback`

**No wiremock tests exist for `/transits`.**

### `tests/full_suite.rs` (mock factory based, safe post-swap)

`mock_data_integrity::*` — uses `mocks::` factories directly.

---

## Mock layer

**`mocks.rs`** provides: `mock_panchang`, `mock_complete_panchang`, `mock_vimshottari_dasha`, plus JSON serializers. **No `mock_transits`.**

**`test_mocks.rs`** Shesh-specific: `shesh_panchang`, `shesh_vimshottari_dasha`, `shesh_birth_chart`. **`MockApiClient::get_transits()` does not exist.**

---

## Wire-up risks (top 3)

### Risk 1 — `DayBoundaries` string format contract
`CachedVedicClient::get_complete_panchang` passes `panchang.day_boundaries.{sunrise,sunset,next_sunrise}` as **`String` HH:MM** into local `calculate_muhurtas`/`hora_timings`/`choghadiya`. If the native swap returns `NaiveTime`, empty strings, or a different format, downstream silently produces wrong timings. **No panic, no compile error.** Engineer must verify HH:MM format.

### Risk 2 — `crate::dasha::DashaPeriod` (`String` dates) vs `vimshottari/types::DashaPeriod` (`NaiveDate`)
The façade must return the `crate::dasha` variant. `vimshottari/query.rs::dasha_period_by_date` does string comparison; if the façade accidentally emits the engine-native NaiveDate type, all date lookups silently return `None`. No compile error.

### Risk 3 — IST hardcode at `analysis.rs:241`
`tz_offset = 5.5` is hardcoded. PR2 façade swap does not touch this. If PR2 additionally fixes `analysis.rs:241` to use `profile.timezone`, IST users (who currently get correct results via the hardcode) silently regress if their `profile.timezone` field is wrong. **Recommendation: do NOT touch analysis.rs:241 in PR2. File a separate child issue.**

---

## Key files for Engineer

- `crates/noesis-vedic-api/src/panchang/{mod,api,data,muhurtas,hora,choghadiya}.rs`
- `crates/noesis-vedic-api/src/vimshottari/{mod,api,types,query,enrichment}.rs`
- `crates/noesis-vedic-api/src/transits/{mod,api,types,aspects,jupiter,predictions,sade_sati}.rs`
- `crates/noesis-vedic-api/src/dasha.rs` (canonical dasha types)
- `crates/noesis-vedic-api/src/cached_client.rs` (preserve get_panchang/get_complete_panchang/get_vimshottari_dasha signatures)
- `crates/noesis-vedic-api/src/service.rs` (preserve VedicApiService methods)
- `crates/noesis-vedic-api/src/lib.rs` (re-exports)
- `crates/noesis-vedic-api/src/mocks.rs` (no transit mock — must add)
- `crates/noesis-vedic-api/src/test_mocks.rs` (Shesh mocks)
- `crates/noesis-vedic-api/tests/integration_tests.rs` (wiremock tests need rewrite)
- `crates/noesis-integration/src/analysis.rs:228-255` (key downstream caller)
