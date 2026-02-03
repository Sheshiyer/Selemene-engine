# Migration Guide: Noesis Vedic API v1 to v2

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
