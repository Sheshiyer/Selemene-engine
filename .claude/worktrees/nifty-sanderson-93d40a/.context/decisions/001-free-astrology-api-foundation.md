# ADR 001: FreeAstrologyAPI.com as Western Astrology Authority

## Status
Accepted

## Context
We require accurate planetary and house calculations for the Western astrology engine. The native Rust implementation (Swiss Ephemeris bindings) has been temporarily disabled or is currently insufficient for validation. We need a reliable oracle to validate our native calculations and serve as the primary backend logic until the native engine is fully verified.

## Decision
We will use [FreeAstrologyAPI.com](https://freeastrologyapi.com) as the:
1.  **Primary Data Source** for Western astrology calculations (Planets, Houses, Aspects) in the interim.
2.  **Validation Oracle** to back-test and verify our native Rust implementation.

## Constraints & Rules
The interaction with this API is subject to strict constraints:
1.  **Rate Limit**: 
    - Short-term: 1 request per second.
    - Long-term: **50 requests per day** (Free Tier).
2.  **Caching**: All responses MUST be cached aggressively.
    - Identical requests (same time/location) must strictly hit the cache.
    - In-memory caching (DashMap) is mandatory to survive the session.
    - Persistent caching (Redis/Disk) is recommended for the future.
3.  **Architecture**:
    - The `WesternApiClient` must encapsulate all rate limiting and caching logic.
    - Consumers of the API should not be aware of the rate limits but should expect potential "Quota Exceeded" errors.

## Implementation Details
- **Crate**: `noesis-western-api`
- **Caching**: In-memory `DashMap` keying by request parameters.
- **Throttling**: Token bucket or simple delay mechanism to enforce 1 req/s.
- **Quota Management**: Atomic counter reset daily to track the 50-call limit.

## Consequences
- **Pros**: Guaranteed accuracy from a verified third-party source.
- **Cons**: Extremely low daily volume (50 calls) limits high-throughput testing.
- **Mitigation**: We will only validation/reference data for specific test cases, not live high-load traffic, until the native engine is validated and switched back on.
