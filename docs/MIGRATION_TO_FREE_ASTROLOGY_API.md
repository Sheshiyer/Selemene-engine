# Migration Guide: Native Engines to FreeAstrologyAPI

This guide covers migrating from Selemene Engine's native Vedic calculation engines (`engine-panchanga`, `engine-vimshottari`) to the FreeAstrologyAPI integration provided by the `noesis-vedic-api` crate.

The migration goal is higher astronomical fidelity for reflection workflows, not prescriptive automation.

## Table of Contents

- [Why Migrate](#why-migrate)
- [Breaking Changes](#breaking-changes)
- [Migration Steps](#migration-steps)
  - [1. Configuration](#1-configuration)
  - [2. Panchang Calculation](#2-panchang-calculation)
  - [3. Vimshottari Dasha](#3-vimshottari-dasha)
  - [4. Birth Charts](#4-birth-charts)
  - [5. Error Handling](#5-error-handling)
- [Feature Parity Matrix](#feature-parity-matrix)
- [Performance Comparison](#performance-comparison)
- [Fallback Behavior](#fallback-behavior)
- [Rollback Plan](#rollback-plan)
- [Environment Variable Migration](#environment-variable-migration)
- [FAQ](#faq)

---

## Why Migrate

### Benefits of FreeAstrologyAPI over Native Calculation

| Aspect | Native Engine | FreeAstrologyAPI |
|--------|---------------|------------------|
| **Accuracy** | Approximate (simplified formulas) | High-precision (Swiss Ephemeris backend) |
| **Panchang depth** | Basic (Tithi, Nakshatra) | Complete (Tithi, Nakshatra, Yoga, Karana, Vara + Muhurtas) |
| **Dasha levels** | Mahadasha only | All 4 levels (Maha, Antar, Pratyantar, Sookshma) |
| **Birth charts** | Sun/Moon only | All 9 planets + Rahu/Ketu |
| **Validation** | Not validated against reference | Validated against JHora, Swiss Ephemeris, known profiles |
| **Maintenance** | Must maintain astronomical algorithms | Upstream maintains accuracy |
| **Dependencies** | Swiss Ephemeris binary (platform-specific) | HTTP only (cross-platform) |
| **Muhurtas** | Not available | Abhijit, Rahu Kalam, Yama Gandam, Gulika, Hora, Choghadiya |

### When to Keep Native

Native calculation remains appropriate when:
- You have no network access (air-gapped environments)
- You need sub-millisecond response times without caching
- You need to exceed 50 API calls/day without a paid tier
- You need planetary positions for non-standard calculations

The recommended approach is to **use FreeAstrologyAPI as primary with native as fallback** -- this is what `VedicApiService` does automatically.

---

## Breaking Changes

The `noesis-vedic-api` crate does **not** remove the native engines. It adds a new, preferred calculation path. There are no forced breaking changes.

However, if you switch from native to API, note these differences:

| Area | Native | FreeAstrologyAPI |
|------|--------|------------------|
| Function signatures | Synchronous | Async (`async fn ... .await`) |
| Error types | Engine-specific errors | `VedicApiError` enum |
| Return types | Engine-specific structs | `Panchang`, `VimshottariDasha`, `BirthChart` |
| Network dependency | None | Requires internet (or fallback) |
| Configuration | No config needed | Requires `FREE_ASTROLOGY_API_KEY` |

---

## Migration Steps

### 1. Configuration

**Before (Native):**

No API configuration needed. The native engines use only local computation.

```bash
# No environment variables required
```

**After (FreeAstrologyAPI):**

```bash
# Required: Get a free key at https://freeastrologyapi.com
export FREE_ASTROLOGY_API_KEY="your_api_key_here"

# Optional (shown with defaults)
export FREE_ASTROLOGY_API_BASE_URL="https://json.freeastrologyapi.com"
export FREE_ASTROLOGY_API_TIMEOUT=30
export VEDIC_ENGINE_FALLBACK_ENABLED=true
```

**Add to Cargo.toml:**

```toml
[dependencies]
noesis-vedic-api = { path = "../crates/noesis-vedic-api" }
tokio = { version = "1", features = ["full"] }
```

---

### 2. Panchang Calculation

**Before (Native - engine-panchanga):**

```rust
use engine_panchanga::{calculate_panchang, PanchangaInput};

let input = PanchangaInput {
    year: 2026,
    month: 2,
    day: 8,
    hour: 12.0,
    latitude: 12.9716,
    longitude: 77.5946,
    timezone: 5.5,
};

let result = calculate_panchang(&input);
println!("Tithi: {}", result.tithi);
println!("Nakshatra: {}", result.nakshatra);
```

**After (FreeAstrologyAPI):**

```rust
use noesis_vedic_api::VedicApiService;

let service = VedicApiService::from_env()?;

// Basic Panchang with automatic fallback
let panchang = service.panchang_with_fallback(
    2026, 2, 8,       // year, month, day
    12, 0, 0,          // hour, minute, second
    12.9716, 77.5946,  // lat, lng
    5.5                // timezone
).await?;

println!("Tithi: {}", panchang.tithi.name());
println!("Nakshatra: {}", panchang.nakshatra.name());
println!("Yoga: {}", panchang.yoga.name());       // NEW: not in native
println!("Karana: {}", panchang.karana.name());     // NEW: not in native
println!("Vara: {}", panchang.vara);                // NEW: not in native

// Complete Panchang includes Muhurtas, Hora, Choghadiya (all new)
let complete = service.complete_panchang(
    2026, 2, 8, 12, 0, 0, 12.9716, 77.5946, 5.5
).await?;

if let Some(ref abhijit) = complete.muhurtas.abhijit {
    println!("Abhijit: {} to {}", abhijit.start, abhijit.end);
}
```

**Key differences:**
- Function is `async` -- requires `.await`
- Hour is split into hour/minute/second (not a single float)
- Returns richer data (Yoga, Karana, Vara, Muhurtas)
- If API fails, falls back to native automatically (via `_with_fallback`)

---

### 3. Vimshottari Dasha

**Before (Native - engine-vimshottari):**

```rust
use engine_vimshottari::{calculate_vimshottari, VimshottariInput};

let input = VimshottariInput {
    birth_year: 1991,
    birth_month: 8,
    birth_day: 13,
    birth_hour: 13.5,  // 1:30 PM as float
    latitude: 12.9716,
    longitude: 77.5946,
    timezone: 5.5,
};

let dasha = calculate_vimshottari(&input);
for period in &dasha.periods {
    println!("{}: {} to {}", period.planet, period.start, period.end);
}
```

**After (FreeAstrologyAPI):**

```rust
use noesis_vedic_api::{VedicApiService, DashaLevel};

let service = VedicApiService::from_env()?;

// Mahadasha level (same as native)
let dasha = service.vimshottari_dasha_with_fallback(
    1991, 8, 13,       // birth date
    13, 31, 0,          // birth time (hour, min, sec)
    12.9716, 77.5946,   // location
    5.5,                // timezone
    DashaLevel::Mahadasha
).await?;

println!("Moon Nakshatra: {}", dasha.moon_nakshatra);
for period in &dasha.periods {
    println!("{}: {} to {}", period.planet, period.start_date, period.end_date);
}

// NEW: Sub-dasha levels not available in native
let antardasha = service.vimshottari_dasha_with_fallback(
    1991, 8, 13, 13, 31, 0,
    12.9716, 77.5946, 5.5,
    DashaLevel::Antardasha
).await?;

let pratyantar = service.vimshottari_dasha_with_fallback(
    1991, 8, 13, 13, 31, 0,
    12.9716, 77.5946, 5.5,
    DashaLevel::Pratyantar
).await?;
```

**Key differences:**
- Birth time is hour/minute/second (not a float)
- `DashaLevel` enum selects depth (Maha, Antar, Pratyantar, Sookshma)
- Sub-dasha levels are new -- native only supported Mahadasha
- Results are cached with infinite TTL (birth data is immutable)

---

### 4. Birth Charts

**Before (Native):**

Native engines did not provide a dedicated birth chart API. Planetary positions were limited to Sun and Moon via the Panchang engine.

```rust
// No dedicated birth chart in native engines
// Only Sun/Moon positions available from Panchang
```

**After (FreeAstrologyAPI):**

```rust
use noesis_vedic_api::VedicApiService;

let service = VedicApiService::from_env()?;

// Full birth chart with all planets
let chart = service.birth_chart_with_fallback(
    1991, 8, 13, 13, 31, 0,
    12.9716, 77.5946, 5.5,
).await?;

// Ascendant
println!("Ascendant: {:?}", chart.ascendant);

// Planetary positions (Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu)
for planet in &chart.planets {
    println!("{}: {} in house {}", planet.name, planet.sign, planet.house);
}

// Navamsa chart (D9)
let navamsa = service.navamsa_chart(
    1991, 8, 13, 13, 31, 0,
    12.9716, 77.5946, 5.5,
).await?;
```

**This is entirely new functionality** -- no native equivalent existed.

---

### 5. Error Handling

**Before (Native):**

```rust
// Native engines returned simple Result types
match calculate_panchang(&input) {
    Ok(result) => println!("{:?}", result),
    Err(e) => eprintln!("Calculation error: {}", e),
}
```

**After (FreeAstrologyAPI):**

```rust
use noesis_vedic_api::VedicApiError;

match service.panchang_with_fallback(2026, 2, 8, 12, 0, 0, 12.97, 77.59, 5.5).await {
    Ok(panchang) => println!("{}", panchang.tithi.name()),
    Err(VedicApiError::RateLimit { retry_after }) => {
        // API quota exhausted; fallback also failed (if this propagated)
        println!("Rate limited, retry after {:?}s", retry_after);
    }
    Err(VedicApiError::FallbackFailed { api_error, native_error }) => {
        // Both API and native failed
        println!("API: {}, Native: {}", api_error, native_error);
    }
    Err(e) => {
        if e.is_retryable() {
            println!("Retryable error: {}", e);
        }
        if e.should_fallback() {
            println!("Should trigger fallback: {}", e);
        }
    }
}
```

**Note:** With `_with_fallback` methods, most API errors are handled transparently. You only see errors when **both** API and native fail.

---

## Feature Parity Matrix

| Feature | Native Engine | FreeAstrologyAPI | Notes |
|---------|:------------:|:----------------:|-------|
| Tithi | Yes | Yes | API more accurate |
| Nakshatra | Yes | Yes | API more accurate |
| Yoga | No | Yes | New in API |
| Karana | No | Yes | New in API |
| Vara (weekday) | No | Yes | New in API |
| Muhurtas | No | Yes | Abhijit, Rahu Kalam, etc. |
| Hora timings | No | Yes | Planetary hours |
| Choghadiya | No | Yes | Auspicious time slots |
| Mahadasha | Yes | Yes | Same accuracy |
| Antardasha | No | Yes | New in API |
| Pratyantar Dasha | No | Yes | New in API |
| Sookshma Dasha | No | Yes | New in API |
| Birth chart (all planets) | No | Yes | New in API |
| Navamsa chart (D9) | No | Yes | New in API |
| Yogas | No | Yes | New in API |
| Shadbala | No | Yes | New in API |
| Ashtakavarga | No | Yes | New in API |
| Transits | No | Yes | New in API |
| Caching | Manual | Automatic | LRU with smart TTLs |
| Rate limiting | N/A | Automatic | 50/day quota managed |
| Metrics | None | Prometheus | 11 metric families |
| Circuit breaker | None | Automatic | Prevents cascading failure |
| Works offline | Yes | Via fallback | Degrades gracefully |

---

## Performance Comparison

| Operation | Native | API (uncached) | API (cached) | API + Cache Hit Rate |
|-----------|--------|----------------|-------------|---------------------|
| Panchang | ~5ms | 200-500ms | <1ms | 90%+ |
| Dasha (Maha) | ~3ms | 200-400ms | <1ms | 99%+ |
| Birth Chart | N/A | 300-600ms | <1ms | 99%+ |
| Complete Panchang | N/A | 800-1500ms | <1ms | 90%+ |

**Summary:** The API is slower on first call, but with 95%+ cache hit rates the effective latency is sub-millisecond for the vast majority of requests. The API provides substantially richer and more accurate data.

---

## Fallback Behavior

The `_with_fallback` methods on `VedicApiService` automatically use native calculation when the API fails. This means:

1. First attempt goes to FreeAstrologyAPI (with caching and rate limiting)
2. If API fails with a fallback-eligible error, native calculation runs
3. If native also fails, `VedicApiError::FallbackFailed` is returned

**Fallback-eligible errors:**
- Network timeout / connection error
- API 5xx server errors
- Rate limit exceeded (429)
- Circuit breaker open

**Non-fallback errors (caller must handle):**
- Configuration errors (missing API key)
- API 4xx client errors (bad input)
- JSON parse errors

**Enable/disable fallback:**

```bash
# Enabled by default
export VEDIC_ENGINE_FALLBACK_ENABLED=true

# Disable fallback (API-only, errors propagate directly)
export VEDIC_ENGINE_FALLBACK_ENABLED=false
```

---

## Rollback Plan

If you need to revert to native-only calculation:

### Option A: Environment Variable (Instant, No Code Change)

```bash
# Switch provider to native -- bypasses API entirely
export VEDIC_ENGINE_PROVIDER=native
```

This tells the Config to use native calculation, effectively disabling API calls.

### Option B: Disable API Key (Forces Fallback)

```bash
# Remove or unset the API key
unset FREE_ASTROLOGY_API_KEY

# Ensure fallback is enabled
export VEDIC_ENGINE_FALLBACK_ENABLED=true
```

Without an API key, the service cannot be initialized via `from_env()`. Use native engines directly instead.

### Option C: Code-Level Rollback

Revert your import and function calls:

```rust
// Revert to native engine
use engine_panchanga::{calculate_panchang, PanchangaInput};

let input = PanchangaInput {
    year: 2026,
    month: 2,
    day: 8,
    hour: 12.0,
    latitude: 12.9716,
    longitude: 77.5946,
    timezone: 5.5,
};

let result = calculate_panchang(&input);
```

### Rollback Verification

After rollback, verify:
1. Panchang calculations return data (even if less detailed)
2. Dasha periods are computed (Mahadasha level only)
3. No network calls are made to `freeastrologyapi.com`
4. Existing tests pass: `cargo test -p engine-panchanga && cargo test -p engine-vimshottari`

---

## Environment Variable Migration

| Old (Native) | New (FreeAstrologyAPI) | Required? | Notes |
|--------------|----------------------|-----------|-------|
| *(none)* | `FREE_ASTROLOGY_API_KEY` | Yes | Get at freeastrologyapi.com |
| *(none)* | `FREE_ASTROLOGY_API_BASE_URL` | No | Default: `https://json.freeastrologyapi.com` |
| *(none)* | `FREE_ASTROLOGY_API_TIMEOUT` | No | Default: 30s |
| *(none)* | `FREE_ASTROLOGY_API_RETRY_COUNT` | No | Default: 3 |
| *(none)* | `VEDIC_ENGINE_PROVIDER` | No | Default: `api` |
| *(none)* | `VEDIC_ENGINE_FALLBACK_ENABLED` | No | Default: `true` |
| *(none)* | `FREE_ASTROLOGY_CACHE_BIRTH_TTL` | No | Default: infinite |
| *(none)* | `FREE_ASTROLOGY_CACHE_DAILY_TTL` | No | Default: 86400 (24h) |
| *(none)* | `FREE_ASTROLOGY_API_RATE_LIMIT_MAX_RETRIES` | No | Default: 3 |
| *(none)* | `FREE_ASTROLOGY_API_RATE_LIMIT_BASE_DELAY` | No | Default: 1000ms |
| *(none)* | `FREE_ASTROLOGY_API_RATE_LIMIT_MAX_DELAY` | No | Default: 60000ms |

Native engines had no environment configuration. All new variables are additive -- setting them will not affect native engine behavior.

---

## FAQ

### Can I use both native and API simultaneously?

Yes. The `_with_fallback` methods on `VedicApiService` do exactly this: API-first with native fallback. This is the recommended approach.

### Will my existing native engine code break?

No. The native engines (`engine-panchanga`, `engine-vimshottari`) are untouched. You can continue using them alongside or instead of the API integration.

### How many API calls will I use per day?

With caching enabled (default), typical usage is 20-30 API calls/day even with hundreds of queries. Birth data is cached with infinite TTL, and Panchang data is cached for 24 hours.

### What happens when the API is down?

If `VEDIC_ENGINE_FALLBACK_ENABLED=true` (default), the service automatically uses native calculation. You get slightly less accurate results, but the service stays available.

### Do I need to update my Docker configuration?

If deploying via Docker, ensure the `FREE_ASTROLOGY_API_KEY` environment variable is set in your container configuration. The existing `Dockerfile.prod` and `docker-compose.yml` support environment variable injection. See `docs/deployment/docker.md`.

### How do I monitor the API integration?

The `VedicApiService` exports Prometheus-compatible metrics. Expose the `/metrics` endpoint and scrape with Prometheus. See the [README metrics section](../crates/noesis-vedic-api/README.md#prometheus-metrics) for details.
