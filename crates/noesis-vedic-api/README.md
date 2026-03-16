# noesis-vedic-api

FreeAstrologyAPI.com integration for accurate Vedic astrology calculations in the Tryambakam Noesis platform.

## Overview

This crate provides a production-grade Rust client for [FreeAstrologyAPI.com](https://freeastrologyapi.com), offering Panchang, Vimshottari Dasha, Birth Charts, and advanced Vedic astrology features. It remains available for optional provider integrations and experiments, but it is not the active production Vedic runtime path in Selemene Engine.

**Key capabilities:**

- Panchang (Tithi, Nakshatra, Yoga, Karana, Vara)
- Muhurtas (Abhijit, Amrit Kaal, Rahu Kalam, Yama Gandam, Gulika, Hora, Choghadiya)
- Vimshottari Dasha (all 4 levels: Maha, Antar, Pratyantar, Sookshma)
- Birth Charts: Rashi (D1), Navamsa (D9), and all Vargas (D1-D60)
- Advanced: Yogas, Shadbala, Ashtakavarga, Transits, Eclipses, Festivals
- Automatic fallback to native calculations when the API is unavailable
- Aggressive caching (95%+ hit rate in typical usage)
- Circuit breaker, rate limiting, and exponential backoff
- Prometheus-compatible metrics (11 metric families)

## Quick Start

### 1. Get an API Key

Sign up for a free API key at [FreeAstrologyAPI.com](https://freeastrologyapi.com).

### 2. Set Environment Variables

```bash
# Required
export FREE_ASTROLOGY_API_KEY="your_api_key_here"

# Optional (shown with defaults)
export FREE_ASTROLOGY_API_BASE_URL="https://json.freeastrologyapi.com"
export FREE_ASTROLOGY_API_TIMEOUT=30
export FREE_ASTROLOGY_API_RETRY_COUNT=3
export VEDIC_ENGINE_FALLBACK_ENABLED=true
export VEDIC_ENGINE_PROVIDER=api
```

### 3. Use VedicApiService (Recommended Entry Point)

```rust
use noesis_vedic_api::{VedicApiService, DashaLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize from environment -- sets up caching, rate limiting,
    // metrics, fallback, and circuit breaker automatically.
    let service = VedicApiService::from_env()?;

    // Get Panchang with automatic fallback on API failure
    let panchang = service.panchang_with_fallback(
        2026, 2, 8,       // year, month, day
        12, 0, 0,          // hour, minute, second
        12.9716, 77.5946,  // lat, lng (Bengaluru)
        5.5                // timezone offset (IST)
    ).await?;

    println!("Tithi: {}", panchang.tithi.name());
    println!("Nakshatra: {}", panchang.nakshatra.name());
    println!("Yoga: {}", panchang.yoga.name());

    // Get Vimshottari Dasha with fallback
    let dasha = service.vimshottari_dasha_with_fallback(
        1991, 8, 13, 13, 31, 0,
        12.9716, 77.5946, 5.5,
        DashaLevel::Mahadasha
    ).await?;

    for period in &dasha.periods {
        println!("{}: {} to {}", period.planet, period.start_date, period.end_date);
    }

    // Get Birth Chart with fallback
    let chart = service.birth_chart_with_fallback(
        1991, 8, 13, 13, 31, 0,
        12.9716, 77.5946, 5.5,
    ).await?;

    println!("Ascendant: {:?}", chart.ascendant);

    Ok(())
}
```

### 4. Use CachedVedicClient (Lower-Level Access)

```rust
use noesis_vedic_api::{CachedVedicClient, DashaLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CachedVedicClient::from_env()?;

    // Get Panchang (cached, rate-limited, but no automatic fallback)
    let panchang = client.get_panchang(
        2026, 2, 8, 12, 0, 0,
        12.9716, 77.5946, 5.5
    ).await?;

    println!("Tithi: {}", panchang.tithi.name());

    // Get Complete Panchang (includes Muhurtas, Hora, Choghadiya)
    let complete = client.get_complete_panchang(
        2026, 2, 8, 12, 0, 0,
        12.9716, 77.5946, 5.5
    ).await?;

    if let Some(ref abhijit) = complete.muhurtas.abhijit {
        println!("Abhijit Muhurta: {} to {}", abhijit.start, abhijit.end);
    }

    // Check rate limit status
    let status = client.status_report().await;
    println!("Remaining API calls today: {}", status.rate_limit.remaining_today);

    Ok(())
}
```

## Configuration

All configuration is via environment variables, read by `Config::from_env()`.

### Required

| Variable | Description |
|----------|-------------|
| `FREE_ASTROLOGY_API_KEY` | API key from [freeastrologyapi.com](https://freeastrologyapi.com) |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `FREE_ASTROLOGY_API_BASE_URL` | `https://json.freeastrologyapi.com` | API base URL |
| `FREE_ASTROLOGY_API_TIMEOUT` | `30` | Request timeout in seconds |
| `FREE_ASTROLOGY_API_RETRY_COUNT` | `3` | Max retry attempts on failure |
| `FREE_ASTROLOGY_CACHE_BIRTH_TTL` | `0` (infinite) | Cache TTL for birth data (seconds) |
| `FREE_ASTROLOGY_CACHE_DAILY_TTL` | `86400` (24h) | Cache TTL for daily data (seconds) |
| `VEDIC_ENGINE_PROVIDER` | `api` | Provider: `api` or `native` |
| `VEDIC_ENGINE_FALLBACK_ENABLED` | `true` | Enable native fallback on API failure |
| `FREE_ASTROLOGY_API_RATE_LIMIT_MAX_RETRIES` | `3` | Max retries on 429 response |
| `FREE_ASTROLOGY_API_RATE_LIMIT_BASE_DELAY` | `1000` | Base delay (ms) for exponential backoff |
| `FREE_ASTROLOGY_API_RATE_LIMIT_MAX_DELAY` | `60000` | Max delay cap (ms) for backoff |

## Architecture

```
┌─────────────────────────────────────────────┐
│            VedicApiService                  │
│   (Metrics + Fallback + Resilience)         │
│  ┌───────────────────────────────────────┐  │
│  │         CachedVedicClient             │  │
│  │  ┌─────────────────────────────────┐  │  │
│  │  │  Cache (LRU)                    │  │  │
│  │  │  - Birth data: Infinite TTL     │  │  │
│  │  │  - Daily data: 24-hour TTL      │  │  │
│  │  └─────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────┐  │  │
│  │  │  Rate Limiter                   │  │  │
│  │  │  - 50/day, 1/sec throttle       │  │  │
│  │  │  - 5-request safety buffer      │  │  │
│  │  └─────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────┐  │  │
│  │  │  Circuit Breaker                │  │  │
│  │  │  - Opens after repeated fails   │  │  │
│  │  │  - Auto-recovery after cooldown │  │  │
│  │  └─────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────┐  │  │
│  │  │  RateLimitHandler (429)         │  │  │
│  │  │  - Exponential backoff          │  │  │
│  │  │  - Respects Retry-After header  │  │  │
│  │  └─────────────────────────────────┘  │  │
│  └───────────────────────────────────────┘  │
│  ┌───────────────────────────────────────┐  │
│  │  FallbackCalculator (Native)          │  │
│  │  - Approximate Panchang               │  │
│  │  - Approximate Dasha                  │  │
│  │  - Approximate Birth Chart            │  │
│  └───────────────────────────────────────┘  │
│  ┌───────────────────────────────────────┐  │
│  │  NoesisMetrics (Prometheus)           │  │
│  │  - API call counts by endpoint        │  │
│  │  - Cache hit/miss ratios              │  │
│  │  - Response time histograms           │  │
│  │  - Error counts by type               │  │
│  │  - Fallback trigger counts            │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
                      │
                      ▼
          FreeAstrologyAPI.com
```

## Features

### Fallback to Native Calculation

When the API is unavailable, the service automatically falls back to local astronomical calculations. Fallback triggers on:

| Condition | Fallback? |
|-----------|-----------|
| Network timeout | Yes |
| API 5xx errors | Yes |
| Rate limit exceeded (429) | Yes |
| Circuit breaker open | Yes |
| API 4xx errors (client error) | No |
| JSON parse errors | No |

```rust
// Fallback is automatic with _with_fallback methods
let panchang = service.panchang_with_fallback(
    2026, 2, 8, 12, 0, 0, 12.97, 77.59, 5.5
).await?;
// If API fails, you get native calculation transparently.
// Only errors if BOTH API and native fail.

// Disable fallback via environment:
// VEDIC_ENGINE_FALLBACK_ENABLED=false
```

### Caching

Aggressive caching minimizes API calls against the free tier's 50/day limit:

| Data Type | TTL | Expected Hit Rate | Rationale |
|-----------|-----|-------------------|-----------|
| Birth Chart | Infinite | 99%+ | Birth data never changes |
| Dasha Periods | Infinite | 99%+ | Birth data never changes |
| Panchang | 24 hours | 90%+ | Same date queried repeatedly |
| Transits | 1 hour | 80%+ | Positions change slowly |

With caching, typical daily API usage drops to 20-30 actual calls even with hundreds of queries.

### Circuit Breaker

Prevents cascading failures when the API is experiencing issues:

- Opens after repeated consecutive failures
- Automatically enters half-open state after a cooldown period
- Routes all requests to fallback while open
- Resets on successful API response

### Rate Limiting (429 Handling)

Two layers of rate limit protection:

1. **Client-side quota tracking**: Tracks the 50 calls/day limit with a 5-request safety buffer (45 usable calls)
2. **Server-side 429 handling**: Exponential backoff with Retry-After header support

```
Backoff sequence: 1s -> 2s -> 4s -> 8s -> 16s -> 32s -> 60s (capped)
```

### Prometheus Metrics

11 metric families exported in Prometheus text exposition format:

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `noesis_api_calls_total` | counter | `endpoint` | Total API calls per endpoint |
| `noesis_cache_hits_total` | counter | `endpoint` | Cache hits per endpoint |
| `noesis_cache_misses_total` | counter | `endpoint` | Cache misses per endpoint |
| `noesis_fallback_triggers_total` | counter | `endpoint` | Fallback triggers per endpoint |
| `noesis_fallback_by_reason_total` | counter | `reason` | Fallback by reason (network, 5xx, 429, etc.) |
| `noesis_errors_total` | counter | `error_type` | Errors by classification |
| `noesis_responses_total` | counter | `status` | Total responses (success/error) |
| `noesis_response_time_seconds` | histogram | `endpoint` | Response time distribution |
| `noesis_retries_total` | counter | `result` | Retry outcomes |
| `noesis_rate_limit_remaining` | gauge | - | Remaining daily API calls |
| `noesis_circuit_breaker_state` | gauge | - | Circuit breaker state |

```rust
// Export for Prometheus scraping
let prometheus_text = service.export_prometheus_metrics().await;

// Or get JSON summary for logging
let json = service.export_metrics_json().await;

// Serve via axum
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

## API Reference

### VedicApiService (Primary Interface)

| Method | Returns | Fallback? | Description |
|--------|---------|-----------|-------------|
| `from_env()` | `Result<Self>` | - | Create from environment variables |
| `panchang(...)` | `Result<Panchang>` | No | Basic Panchang data |
| `panchang_with_fallback(...)` | `Result<Panchang>` | Yes | Panchang with native fallback |
| `complete_panchang(...)` | `Result<CompletePanchang>` | No | Full Panchang + Muhurtas + Hora + Choghadiya |
| `panchang_with_query(query)` | `Result<CompletePanchang>` | No | Query builder interface |
| `vimshottari_dasha(...)` | `Result<VimshottariDasha>` | No | Dasha periods at specified level |
| `vimshottari_dasha_with_fallback(...)` | `Result<VimshottariDasha>` | Yes | Dasha with native fallback |
| `birth_chart(...)` | `Result<BirthChart>` | No | Rashi chart (D1) |
| `birth_chart_with_fallback(...)` | `Result<BirthChart>` | Yes | Birth chart with native fallback |
| `navamsa_chart(...)` | `Result<NavamsaChart>` | No | Navamsa chart (D9) |
| `batch_panchang(requests)` | `Vec<Result<Panchang>>` | No | Batch Panchang requests |
| `export_prometheus_metrics()` | `String` | - | Prometheus text format |
| `export_metrics_json()` | `serde_json::Value` | - | JSON metric summary |
| `reset_metrics()` | `()` | - | Reset all counters |
| `resilience_snapshot()` | `MetricsSnapshot` | - | Resilience status |

### Error Types

```rust
use noesis_vedic_api::VedicApiError;

match result {
    Err(VedicApiError::Configuration { field, message }) => { /* missing config */ }
    Err(VedicApiError::Network { message }) => { /* connection/timeout */ }
    Err(VedicApiError::Api { status_code, message }) => { /* HTTP error */ }
    Err(VedicApiError::RateLimit { retry_after }) => { /* 429 response */ }
    Err(VedicApiError::CircuitBreakerOpen) => { /* too many failures */ }
    Err(VedicApiError::Parse { message }) => { /* JSON parse error */ }
    Err(VedicApiError::FallbackFailed { api_error, native_error }) => { /* both failed */ }
    Err(VedicApiError::InvalidInput { field, message }) => { /* bad parameters */ }
    Err(VedicApiError::Cache { message }) => { /* cache error */ }
    _ => {}
}

// Classification helpers
if err.is_retryable() { /* worth retrying */ }
if err.should_fallback() { /* native calculation appropriate */ }
if let Some(code) = err.status_code() { /* HTTP status */ }
```

## Testing

### Run Unit Tests (No API Key Required)

```bash
cargo test -p noesis-vedic-api
```

### Run with Mock Data

Tests use mock factories when the `mocks` or `test-mocks` feature is enabled. All CI/CD tests run without an API key.

```bash
# Run tests with mock feature enabled
cargo test -p noesis-vedic-api --features test-mocks
```

### Run Against Real API

```bash
# Requires a valid API key
FREE_ASTROLOGY_API_KEY=your_key cargo test -p noesis-vedic-api --features real-api -- --ignored
```

### Test Categories

| Category | Count | Description |
|----------|-------|-------------|
| Mock tests | 37 | Client behavior with mock API responses |
| Validation tests | 51 | Accuracy against JHora, ephemeris, Shesh's profile |
| Integration tests | 86 | End-to-end flows, resilience, error handling |
| Rate limit tests | 10 | 429 backoff, quota tracking |
| Metrics tests | 7 | Prometheus export, counter accuracy |

### Validation References

Results have been validated against:

- **JHora** - Panchang accuracy (Tithi, Nakshatra within 1 unit)
- **Swiss Ephemeris** - Planetary positions
- **Shesh's birth profile** (1991-08-13, Bengaluru) - Known reference chart

## Troubleshooting

### "Configuration error for 'FREE_ASTROLOGY_API_KEY'"

The API key is not set. Set it in your environment:

```bash
export FREE_ASTROLOGY_API_KEY="your-key-from-freeastrologyapi.com"
```

### "Rate limit exceeded"

You have exhausted your daily API budget (50 calls/day on free tier). Options:

1. Wait until the daily reset (midnight UTC)
2. Ensure fallback is enabled: `VEDIC_ENGINE_FALLBACK_ENABLED=true`
3. Check cache is working: `service.client().cache_stats().await`
4. Pre-warm cache for upcoming dates

### "Circuit breaker is open"

The API has failed too many consecutive requests. The circuit breaker auto-resets after cooldown. During this time, requests route to native fallback (if enabled).

### "Fallback failed"

Both the API and native calculation failed. Check:

1. Network connectivity to `json.freeastrologyapi.com`
2. API key validity (may have expired)
3. Input parameter validity (dates, coordinates in valid ranges)
4. Native engine availability (`VEDIC_ENGINE_FALLBACK_ENABLED=true`)

### High error rates in metrics

```rust
let json = service.export_metrics_json().await;
println!("{}", serde_json::to_string_pretty(&json)?);
// Check "errors" key for breakdown by type:
// network -> connectivity to freeastrologyapi.com
// rate_limit -> too many calls (check cache hit ratio)
// parse -> API response format may have changed
```

### Low cache hit rate

Verify that you are querying the same data across calls. Cache keys include the full parameter set (date + location + timezone). Different timezone values for the same location will create separate cache entries.

## API Endpoints

The client maps to these FreeAstrologyAPI.com endpoints:

### Panchang
- `POST /panchang` - Complete Panchang
- `GET /sunrise-sunset` - Day boundaries
- `GET /abhijit-muhurta` - Victorious midday
- `GET /amrit-kaal` - Nectar time
- `GET /rahu-kalam` - Rahu period
- `GET /yama-gandam` - Yama period
- `GET /gulika-kaal` - Gulika time
- `GET /hora-timings` - Planetary hours
- `GET /choghadiya` - Muhurtas

### Dasha
- `POST /vimshottari-dasha` - All Dasha levels (Maha, Antar, Pratyantar, Sookshma)

### Charts
- `POST /horoscope-chart` - Rashi chart (D1)
- `POST /navamsa-chart` - Navamsa (D9)

## Crate Feature Flags

| Feature | Description |
|---------|-------------|
| `mocks` | Enable mock data factories for testing |
| `test-mocks` | Enable Shesh-specific test mocks for CI/CD |
| `real-api` | Enable tests that hit the real API (requires key) |

## Modules

| Module | FAPI Task | Description |
|--------|-----------|-------------|
| `service` | - | Unified service with metrics and fallback |
| `cached_client` | - | Cached client with rate limiting |
| `client` | - | Raw HTTP client |
| `config` | - | Environment-based configuration |
| `error` | - | Typed error handling |
| `fallback` | FAPI-098 | Native calculation fallback |
| `resilience` | FAPI-098, FAPI-105 | Fallback chain and backoff |
| `metrics` | FAPI-099 | Prometheus-compatible metrics |
| `rate_limit` | FAPI-105 | 429 handling with exponential backoff |
| `rate_limiter` | - | Daily quota tracking (50/day) |
| `circuit_breaker` | FAPI-008 | Circuit breaker pattern |
| `cache` | - | LRU cache with TTL |
| `batch` | FAPI-106 | Batch request optimization |
| `versioning` | FAPI-107 | API version routing |
| `panchang` | - | Panchang types and data |
| `dasha` | - | Vimshottari Dasha types |
| `chart` | - | Birth/Navamsa chart types |
| `vimshottari` | - | Dasha calculation helpers |
| `vargas` | - | Divisional charts (D1-D60) |
| `transits` | FAPI-073-078 | Transit calculations |
| `yogas` | FAPI-063-066 | Yoga identification |
| `shadbala` | FAPI-067-069 | Planetary strength |
| `ashtakavarga` | FAPI-070-072 | Ashtakavarga system |
| `muhurta` | FAPI-081-086 | Muhurta calculations |
| `mocks` | FAPI-093 | Test mock factories |
| `test_mocks` | FAPI-093 | Shesh profile test data |

## Performance

| Operation | Cached | Uncached (API) | Native Fallback |
|-----------|--------|----------------|-----------------|
| Panchang | <1ms | 200-500ms | ~5ms |
| Birth Chart | <1ms | 300-600ms | ~10ms |
| Dasha | <1ms | 200-400ms | ~3ms |
| Complete Panchang | <1ms | 800-1500ms (multi-call) | N/A |

## Related Documentation

- [Migration Guide: Native to FreeAstrologyAPI](../../docs/MIGRATION_TO_FREE_ASTROLOGY_API.md) - How to migrate from native engines
- [Migration Guide: v1 to v2 (internal)](./MIGRATION.md) - Migrating from VedicApiClient to VedicApiService
- [Deployment Guide](../../docs/deployment/) - Docker and Railway deployment
- [CHANGELOG](../../CHANGELOG.md) - Version history

## License

MIT - See LICENSE file
