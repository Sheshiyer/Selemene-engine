# noesis-vedic-api PR1 Internals

> Read-only analysis for the Engineer rewriting `birth_chart`, `vargas`, and adding `houses`.

## VedicApiClient surface

**File:** `src/client.rs`

`VedicApiClient` holds `config: Config`, `http_client: reqwest::Client`, and `rate_limit_handler: Arc<Mutex<RateLimitHandler>>`.

**`post` signature** (line 578):
```rust
pub async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
    &self, path: &str, body: B,
) -> Result<T>
```
The method serializes `body` to `serde_json::Value`, then calls `execute_with_retry`, and finally calls `response.json::<T>()`. Callers that do `serde_json::from_value(self.post(...).await?)` are instantiating `T = serde_json::Value` — that is the current pattern in `birth_chart/api.rs` and `vargas/api.rs`.

**Retry/rate-limit integration:** `execute_with_retry` loops on HTTP 429, delegating to `RateLimitHandler` (from `src/rate_limit.rs`) for backoff delays. Non-429 non-success responses are mapped to `VedicApiError::Api { status_code, message }` at line 180 of `client.rs`. Timeout/connection errors become `VedicApiError::Network`. Circuit breaker (`src/circuit_breaker.rs`) is wired independently at the `CachedVedicClient` / `VedicApiService` layer, not inside `post` itself.

## Error variants to use

All variants are in `src/error.rs`. For PR1 purposes:

- **Vendor non-success envelope** (HTTP 4xx/5xx): use `VedicApiError::Api { status_code: u16, message: String }`. Already raised by `execute_request` before `post` returns.
- **Vendor malformed JSON / field missing**: use `VedicApiError::Parse { message: String }`. The `From<serde_json::Error>` impl maps directly to this variant. Call sites in `birth_chart/api.rs` and `vargas/api.rs` currently use the legacy alias `VedicApiError::ParseError(String)` — prefer the structured `Parse` variant for new code.
- `is_retryable()` returns true for `Network` and `RateLimit` and `Api` with status >= 500. `Parse` is **not** retryable; do not use it for network failure.

## birth_chart call graph + types

**API-layer request/response (replaceable in PR1):**

- `BirthChartRequest` — `src/birth_chart/api.rs:14`. Fields: `birth_date: String`, `birth_time: String`, `latitude: f64`, `longitude: f64`, `timezone: f64`, `ayanamsa: Option<String>`, `house_system: Option<String>`.
- `BirthChartApiResponse` — `src/birth_chart/api.rs:67`. Fields: `planets: Vec<PlanetApiResponse>`, `houses: Vec<HouseApiResponse>`, `ascendant: AscendantApiResponse`, `ayanamsa: Option<AyanamsaApiResponse>`. The flat `planets[*].house: u8` and `ascendant.longitude: f64` fields are what the old schema expected.

**Methods on `VedicApiClient` added by this module:**
- `fetch_birth_chart(&self, request: &BirthChartRequest) -> VedicApiResult<BirthChartApiResponse>` — calls `self.post("/horoscope-chart", request)` then `serde_json::from_value`.
- `fetch_birth_chart_simple(birth_datetime, lat, lng, tz)` — convenience wrapper.

**Internal callers consuming `BirthChartApiResponse`:**
- `src/birth_chart/mappers.rs` — `map_birth_chart_response(BirthChartApiResponse) -> VedicApiResult<BirthChart>` consumes `.planets`, `.houses`, `.ascendant.{sign,degree}`, `.ayanamsa`. This is the only place `PlanetApiResponse` / `HouseApiResponse` fields are field-accessed.
- `src/client.rs:342` — the older `get_birth_chart(...)` method bypasses `birth_chart/api.rs` entirely; it POSTs directly to `"horoscope-chart"` and deserializes into `chart::BirthChart` (the `chart.rs` type, not the `birth_chart/types.rs` type).
- `src/cached_client.rs:161` — `get_birth_chart(...)` wraps `self.inner.get_birth_chart(...)` (the `client.rs` variant) with cache and rate-limit checks; returns `chart::BirthChart`.
- `src/cached_client.rs:204` — `get_birth_chart_raw(...)` calls `self.inner.get_birth_chart_raw(...)` → `serde_json::Value`.

## vargas call graph + types

**API-layer request/response (replaceable in PR1):**

- `VargaChartRequest` — `src/vargas/api.rs:14`. Fields: `birth_date`, `birth_time`, `latitude`, `longitude`, `timezone`, `varga: String` (e.g. `"D9"`), `ayanamsa`.
- `VargaChartApiResponse` — `src/vargas/api.rs:55`. Fields: `varga: String`, `planets: Vec<VargaPlanetResponse>`, `ascendant: VargaAscendantResponse`.

**Methods on `VedicApiClient` added by this module:**
- `get_varga_chart(&self, request: &VargaChartRequest) -> VedicApiResult<VargaChartApiResponse>` — POSTs to `"/horoscope-chart/varga"`.
- `get_navamsa_chart(birth_datetime, lat, lng, tz) -> VedicApiResult<VargaChartApiResponse>` — builds a `VargaType::Navamsa` request and delegates.
- `get_dasamsa_chart(birth_datetime, lat, lng, tz) -> VedicApiResult<VargaChartApiResponse>`.

**`cached_client.rs` public signature that MUST be preserved** (`src/cached_client.rs:252`):
```rust
pub async fn get_navamsa_chart(
    &self,
    year: i32, month: u32, day: u32,
    hour: u32, minute: u32, second: u32,
    lat: f64, lng: f64, tzone: f64,
) -> Result<NavamsaChart>
```
Note: this returns `chart::NavamsaChart`, not `VargaChartApiResponse`. The cached client wraps `self.inner.get_navamsa_chart(...)` which is defined on `client.rs:390` and returns `chart::NavamsaChart` (POSTing to `"navamsa-chart"`). The `vargas/api.rs` methods are separate from this chain.

`src/vargas/mod.rs` exposes only `dwadasamsa`, `navamsa_mappers`, `navamsa_types`, `saptamsa` — the `api.rs` and `types.rs` sub-modules are present on disk but **not declared in mod.rs** and therefore currently dead code.

## Untouchable public types

These are re-exported from `src/lib.rs` and consumed by downstream crates. Shape must not change:

- `chart::BirthChart`, `chart::PlanetPosition`, `chart::HousePosition`, `chart::NavamsaChart`, `chart::NavamsaPosition`, `chart::ZodiacSign` — re-exported via `pub use chart::{BirthChart, HousePosition, NavamsaChart, PlanetPosition, ZodiacSign}` at `lib.rs:203`.
- `birth_chart::types::*` (via `pub use types::*` in `birth_chart/mod.rs`): `BirthChart` (the module-local one, distinct from `chart::BirthChart`), `PlanetPosition`, `HouseCusp`, `ZodiacSign`, `Planet`, `Dignity`, `calculate_dignity`.
- `vargas::types::{VargaType, VargaChart, VargaPosition, VargaBala, VargaPoint}` — in `src/vargas/types.rs` but **not re-exported** through `lib.rs` or `vargas/mod.rs` currently. Safe to restructure module visibility, but the type shapes must stay.

**Note:** `chart.rs` and `birth_chart/types.rs` both define types named `BirthChart`, `PlanetPosition`, and `ZodiacSign` with different shapes. Downstream callers import through `lib.rs` re-exports, which point to `chart.rs` versions.

## Test gating pattern

- All inline tests live inside module-level `#[cfg(test)] mod tests` blocks within source files.
- The two integration test files (`tests/integration_tests.rs`, `tests/full_suite.rs`) exist on disk; the live tests are gated with `#[cfg(feature = "real-api")]`. Run with `cargo test --features real-api`.
- Without the flag, all tests use `wiremock` (`MockServer`) to intercept HTTP calls. The `test_client(base_url)` helper in integration tests points `Config.base_url` at the wiremock server URI.
- `dev-dependencies` block in `Cargo.toml` activates `mocks` + `test-mocks` features for tests:
  `noesis-vedic-api = { path = ".", features = ["mocks", "test-mocks"] }`.

---

## Essential file list for the Engineer

- `crates/noesis-vedic-api/src/client.rs` — `VedicApiClient`, `post`, `execute_with_retry`, `get_birth_chart` (LIVE PATH used by cached_client), `get_navamsa_chart` (LIVE PATH used by cached_client), raw variants
- `crates/noesis-vedic-api/src/error.rs` — all `VedicApiError` variants
- `crates/noesis-vedic-api/src/birth_chart/api.rs` — `BirthChartRequest`, `BirthChartApiResponse`, `fetch_birth_chart` (DEAD/parallel; not on production path)
- `crates/noesis-vedic-api/src/birth_chart/mappers.rs` — consumes `BirthChartApiResponse` fields
- `crates/noesis-vedic-api/src/birth_chart/types.rs` — untouchable domain types
- `crates/noesis-vedic-api/src/vargas/api.rs` — `VargaChartRequest`, `VargaChartApiResponse` (NOT WIRED via vargas/mod.rs — dead code on disk)
- `crates/noesis-vedic-api/src/vargas/types.rs` — untouchable varga domain types
- `crates/noesis-vedic-api/src/vargas/mod.rs` — currently missing `api` and `types` declarations
- `crates/noesis-vedic-api/src/cached_client.rs` — public `get_navamsa_chart` sig to preserve; also `get_birth_chart`, raw variants
- `crates/noesis-vedic-api/src/chart.rs` — `BirthChart`, `NavamsaChart`, `ZodiacSign` (the lib.rs-exported shapes)
- `crates/noesis-vedic-api/src/lib.rs` — all re-exports
- `crates/noesis-vedic-api/Cargo.toml` — feature flags (`mocks`, `test-mocks`, `real-api`)
