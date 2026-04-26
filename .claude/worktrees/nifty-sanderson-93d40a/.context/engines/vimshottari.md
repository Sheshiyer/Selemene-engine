# Vimshottari Dasha Engine Documentation

## Overview

The Vimshottari Dasha system is the most widely used planetary period system in Vedic astrology. It spans 120 years and divides life into planetary ruling periods (Mahadashas) based on the Moon's nakshatra position at birth.

## Purpose

Calculate and interpret planetary periods that influence different life phases, helping users understand cyclical timing patterns and current planetary influences.

## Calculation Type

**Natal + Temporal**: Derives initial Mahadasha from birth Moon nakshatra, then calculates progression through time.

## Input Parameters

```json
{
  "birth_data": {
    "date": "1990-03-15",
    "time": "14:30",
    "latitude": 40.7128,
    "longitude": -74.0060,
    "timezone": "America/New_York"
  },
  "current_time": "2025-01-15T12:00:00Z",
  "precision": "Standard",
  "options": {
    "include_antardashas": true,
    "include_pratyantardashas": false
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| birth_data.date | string | Yes | ISO 8601 date |
| birth_data.time | string | Yes | 24-hour format |
| birth_data.latitude | number | Yes | -90 to 90 |
| birth_data.longitude | number | Yes | -180 to 180 |
| current_time | string | No | Query time (defaults to now) |
| options.include_antardashas | boolean | No | Include sub-periods |
| options.include_pratyantardashas | boolean | No | Include sub-sub-periods |

## Output Structure

```json
{
  "engine_id": "vimshottari",
  "success": true,
  "result": {
    "birth_nakshatra": {
      "name": "Rohini",
      "lord": "Moon",
      "pada": 2,
      "degree_in_nakshatra": 8.5
    },
    "current_dasha": {
      "mahadasha": {
        "planet": "Jupiter",
        "start_date": "2020-03-15",
        "end_date": "2036-03-15",
        "duration_years": 16
      },
      "antardasha": {
        "planet": "Saturn",
        "start_date": "2024-06-15",
        "end_date": "2027-01-15",
        "duration_months": 31
      }
    },
    "dasha_sequence": [
      {"planet": "Moon", "duration_years": 10},
      {"planet": "Mars", "duration_years": 7},
      {"planet": "Rahu", "duration_years": 18},
      {"planet": "Jupiter", "duration_years": 16},
      {"planet": "Saturn", "duration_years": 19},
      {"planet": "Mercury", "duration_years": 17},
      {"planet": "Ketu", "duration_years": 7},
      {"planet": "Venus", "duration_years": 20},
      {"planet": "Sun", "duration_years": 6}
    ],
    "elapsed_at_birth": {
      "years": 4,
      "months": 3,
      "days": 12
    },
    "wisdom": {
      "mahadasha_meaning": "Jupiter Mahadasha brings expansion...",
      "antardasha_meaning": "Saturn sub-period within Jupiter..."
    }
  },
  "witness_prompt": "During Jupiter's expansive period tempered by Saturn's discipline, what growth requires patience?",
  "consciousness_level": 1
}
```

## Dasha Sequence

The 120-year cycle follows this fixed order starting from birth nakshatra's lord:

| Planet | Duration | Nature |
|--------|----------|--------|
| Sun | 6 years | Authority, soul |
| Moon | 10 years | Mind, emotions |
| Mars | 7 years | Energy, conflict |
| Rahu | 18 years | Desires, worldly |
| Jupiter | 16 years | Expansion, wisdom |
| Saturn | 19 years | Discipline, karma |
| Mercury | 17 years | Communication, intellect |
| Ketu | 7 years | Spirituality, release |
| Venus | 20 years | Relationships, pleasures |

## Nakshatra-Lord Mapping

| Nakshatra | Lord | | Nakshatra | Lord |
|-----------|------|-|-----------|------|
| Ashwini | Ketu | | Magha | Ketu |
| Bharani | Venus | | Purva Phalguni | Venus |
| Krittika | Sun | | Uttara Phalguni | Sun |
| Rohini | Moon | | Hasta | Moon |
| Mrigashira | Mars | | Chitra | Mars |
| Ardra | Rahu | | Swati | Rahu |
| Punarvasu | Jupiter | | Vishakha | Jupiter |
| Pushya | Saturn | | Anuradha | Saturn |
| Ashlesha | Mercury | | Jyeshtha | Mercury |

(Pattern repeats for remaining nakshatras)

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| Mahadasha | "During {planet}'s period of {nature}, what {theme} calls for attention?" | "During Jupiter's period of expansion, what growth calls for attention?" |
| Antardasha | "{planet} within {planet}: How do {quality1} and {quality2} interact in your current experience?" | "Saturn within Jupiter: How do discipline and expansion interact in your current experience?" |
| Transition | "As {old_planet}'s influence wanes and {new_planet} rises, what must be released and what embraced?" | "As Mars's influence wanes and Rahu rises..." |

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Nakshatra calculation | ✅ Full | Swiss Ephemeris Moon position |
| Mahadasha periods | ✅ Full | Full 120-year sequence |
| Antardasha periods | ✅ Full | Sub-periods calculated |
| Pratyantardasha | ✅ Full | Sub-sub-periods |
| Wisdom data | ✅ Full | All 9 planets + combinations |
| Witness prompts | ✅ Full | Phase-appropriate |

## API Endpoint

```
POST /api/v1/engines/vimshottari/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/engines/vimshottari/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1985-06-15",
      "time": "14:30",
      "latitude": 34.0522,
      "longitude": -118.2437,
      "timezone": "America/Los_Angeles"
    },
    "options": {
      "include_antardashas": true
    }
  }'
```

## Dependencies

- Swiss Ephemeris for Moon position
- Nakshatra wisdom data
- Planetary period wisdom data

## References

- B.V. Raman's works on Vedic astrology
- Classical Jyotish texts (Brihat Parashara Hora Shastra)

---

**Engine Version**: 0.1.0
**Required Phase**: 1
**Crate**: `crates/engine-vimshottari/`
