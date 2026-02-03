# noesis-vedic-api

FreeAstrologyAPI.com integration for accurate Vedic astrology calculations in the Tryambakam Noesis platform.

## Overview

This crate provides a Rust client for [FreeAstrologyAPI.com](https://freeastrologyapi.com), offering accurate Panchang, Vimshottari Dasha, Birth Charts, and advanced Vedic astrology features.

## Features

- **Panchang**: Tithi, Nakshatra, Yoga, Karana, Vara
- **Muhurtas**: Abhijit, Amrit Kaal, Rahu Kalam, Yama Gandam, Gulika, Hora, Choghadiya
- **Vimshottari Dasha**: All 4 levels (Maha, Antar, Pratyantar, Sookshma)
- **Birth Charts**: Rashi (D1), Navamsa (D9), and all Vargas (D1-D60)
- **Advanced**: Yogas, Shadbala, Ashtakavarga, Transits

## Quick Start

### 1. Get API Key

Sign up for a free API key at [FreeAstrologyAPI.com](https://freeastrologyapi.com)

### 2. Configure Environment

```bash
# .env
FREE_ASTROLOGY_API_KEY=your_api_key_here
FREE_ASTROLOGY_API_BASE_URL=https://json.freeastrologyapi.com
FREE_ASTROLOGY_API_TIMEOUT=30
```

### 3. Use the Client

```rust
use noesis_vedic_api::{VedicApiClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize from environment
    let client = VedicApiClient::from_env()?;
    
    // Get Panchang for birth date
    let panchang = client.get_panchang(
        1991, 8, 13,      // year, month, day
        13, 31, 0,        // hour, minute, second
        12.9716, 77.5946, // lat, lng (Bengaluru)
        5.5               // timezone (IST)
    ).await?;
    
    println!("Tithi: {}", panchang.tithi.name);
    println!("Nakshatra: {}", panchang.nakshatra.name);
    
    // Get Vimshottari Dasha
    let dasha = client.get_vimshottari_dasha(
        1991, 8, 13, 13, 31, 0,
        12.9716, 77.5946, 5.5,
        DashaLevel::Antar
    ).await?;
    
    for maha in &dasha.mahadashas {
        println!("{}: {} to {}", 
            maha.planet, maha.start_date, maha.end_date);
    }
    
    Ok(())
}
```

## Architecture

```
┌─────────────────────────────────────────┐
│         VedicApiClient                  │
│  ┌───────────────────────────────────┐  │
│  │  Cache (L1/L2)                    │  │
│  │  - Birth data: Infinite           │  │
│  │  - Daily data: 24 hours           │  │
│  └───────────────────────────────────┘  │
│  ┌───────────────────────────────────┐  │
│  │  Circuit Breaker                  │  │
│  │  - Opens after 5 failures         │  │
│  │  - Auto-recovery                  │  │
│  └───────────────────────────────────┘  │
│  ┌───────────────────────────────────┐  │
│  │  Retry Logic                      │  │
│  │  - Exponential backoff            │  │
│  │  - Respects rate limits           │  │
│  └───────────────────────────────────┘  │
│  ┌───────────────────────────────────┐  │
│  │  HTTP Client (reqwest)            │  │
│  │  - Async/Tokio                    │  │
│  │  - Timeout: 30s                   │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
                    │
                    ▼
        FreeAstrologyAPI.com
```

## Error Handling

The client provides detailed error types:

- `Configuration`: Missing API key or invalid config
- `Network`: Connection issues, timeouts
- `Api`: API errors with HTTP status codes
- `RateLimit`: Rate limiting with retry-after
- `Parse`: JSON parsing errors
- `CircuitBreakerOpen`: API temporarily unavailable

## Fallback Strategy

If the API is unavailable and `VEDIC_ENGINE_FALLBACK_ENABLED=true`, the client falls back to native calculations:

```rust
match client.get_panchang(...).await {
    Ok(result) => result,
    Err(e) if e.should_fallback() => {
        // Use native calculation
        native_calculate(...)
    }
    Err(e) => return Err(e),
}
```

## Caching

Aggressive caching reduces API calls:

- **Birth charts**: Cached forever (birth data never changes)
- **Dasha periods**: Cached forever
- **Panchang**: Cached 24 hours
- **Transits**: Cached 1 hour

Expected cache hit rate: >95%

## Testing

```bash
# Run unit tests
cargo test -p noesis-vedic-api

# Run with real API (requires API key)
FREE_ASTROLOGY_API_KEY=xxx cargo test -p noesis-vedic-api -- --ignored
```

## Rate Limits

- **Free tier**: 1,000 requests/day
- With caching, typical usage: 200-300 requests/day

## API Endpoints

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
- `POST /vimshottari-dasha` - All Dasha levels

### Charts
- `POST /horoscope-chart` - Rashi chart (D1)
- `POST /navamsa-chart` - Navamsa (D9)

## Validation

Results are validated against:
- JHora software
- Genetic Matrix (for birth charts)
- GeneKeys.com (for Nakshatra calculations)

## License

MIT - See LICENSE file
