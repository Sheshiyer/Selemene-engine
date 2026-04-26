# Panchanga Engine Documentation

## Overview

Panchanga (Sanskrit: "five limbs") is the traditional Vedic calendar system calculating five essential time elements for any given moment: Tithi, Nakshatra, Yoga, Karana, and Vara.

## Purpose

Determine auspicious and inauspicious timing based on Vedic cosmology, supporting decisions about activities, rituals, and daily planning according to traditional wisdom.

## Calculation Type

**Temporal**: Pure time-based calculation for any given moment and location.

## Input Parameters

```json
{
  "current_time": "2025-01-15T14:30:00Z",
  "location": {
    "latitude": 28.6139,
    "longitude": 77.2090
  },
  "precision": "Standard",
  "options": {
    "include_muhurta": true,
    "include_hora": true
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| current_time | string | Yes | ISO 8601 datetime |
| location.latitude | number | Yes | -90 to 90 |
| location.longitude | number | Yes | -180 to 180 |
| precision | string | No | "Standard" or "High" |
| options.include_muhurta | boolean | No | Include 48 daily muhurtas |
| options.include_hora | boolean | No | Include planetary hours |

## Output Structure

```json
{
  "engine_id": "panchanga",
  "success": true,
  "result": {
    "tithi": {
      "name": "Shukla Saptami",
      "number": 7,
      "paksha": "Shukla",
      "lord": "Sun",
      "percentage_elapsed": 65.3,
      "end_time": "2025-01-15T22:30:00Z"
    },
    "nakshatra": {
      "name": "Pushya",
      "number": 8,
      "lord": "Saturn",
      "pada": 2,
      "percentage_elapsed": 45.2,
      "end_time": "2025-01-16T04:15:00Z"
    },
    "yoga": {
      "name": "Shiva",
      "number": 10,
      "nature": "Auspicious",
      "percentage_elapsed": 78.1,
      "end_time": "2025-01-15T18:45:00Z"
    },
    "karana": {
      "name": "Bava",
      "number": 1,
      "nature": "Movable",
      "percentage_elapsed": 30.8
    },
    "vara": {
      "name": "Wednesday",
      "lord": "Mercury",
      "sunrise": "2025-01-15T07:12:00+05:30",
      "sunset": "2025-01-15T17:45:00+05:30"
    },
    "auspiciousness": {
      "overall": "Good",
      "score": 72,
      "factors": {
        "tithi_quality": "Favorable",
        "nakshatra_quality": "Very Auspicious",
        "yoga_quality": "Auspicious",
        "karana_quality": "Neutral"
      }
    }
  },
  "witness_prompt": "On this Shukla Saptami in Pushya nakshatra, what seeds planted now will bear fruit?",
  "consciousness_level": 0
}
```

## The Five Limbs

### 1. Tithi (Lunar Day)
30 tithis per lunar month, each spanning 12° of Moon-Sun elongation.

| Paksha | Tithis | Moon Phase |
|--------|--------|------------|
| Shukla (Waxing) | 1-15 | New to Full |
| Krishna (Waning) | 1-15 | Full to New |

**Special Tithis**: Purnima (Full), Amavasya (New), Ekadashi (11th)

### 2. Nakshatra (Lunar Mansion)
27 nakshatras of 13°20' each across the zodiac.

| Group | Nakshatras | Nature |
|-------|------------|--------|
| Dhruva | Rohini, Uttara (3), Uttara Bhadrapada | Fixed |
| Chara | Punarvasu, Swati, Shravana, Dhanishtha | Movable |
| Ugra | Bharani, Magha, P.Phalguni, P.Ashadha, P.Bhadra | Fierce |
| Kshipra | Ashwini, Pushya, Hasta, Abhijit | Swift |

### 3. Yoga (Sun-Moon Combination)
27 yogas from combined Sun+Moon longitudes (each 13°20').

| Auspicious | Neutral | Inauspicious |
|------------|---------|--------------|
| Siddha, Amrita, Shubha | Priti, Vriddhi, Dhruva | Vishkumbha, Atiganda, Vyaghata |

### 4. Karana (Half-Tithi)
11 karanas cycling through tithis.

| Type | Karanas | Activity |
|------|---------|----------|
| Movable | Bava, Balava, Kaulava, Taitila, Gara | Dynamic actions |
| Fixed | Shakuni, Chatushpada, Naga, Kimsthugna | Avoid new ventures |

### 5. Vara (Weekday)
Seven weekdays ruled by planets.

| Day | Lord | Suitable For |
|-----|------|--------------|
| Sunday | Sun | Authority matters |
| Monday | Moon | Mind, emotions |
| Tuesday | Mars | Courage, conflict |
| Wednesday | Mercury | Learning, trade |
| Thursday | Jupiter | Spirituality, expansion |
| Friday | Venus | Arts, relationships |
| Saturday | Saturn | Service, austerity |

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| Tithi Focus | "On {tithi} in {paksha}, what {theme} aligns with this lunar energy?" | "On Shukla Saptami, what beginnings align with this waxing energy?" |
| Nakshatra Quality | "Under {nakshatra}'s influence ({quality}), what {action} serves you?" | "Under Pushya's influence (nourishing), what nurturing serves you?" |
| Combined Reading | "Today's {yoga} yoga in {nakshatra} nakshatra invites: {inquiry}" | "Today's Shiva yoga in Pushya nakshatra invites: what transforms?" |

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Tithi calculation | ✅ Full | Sun-Moon elongation based |
| Nakshatra calculation | ✅ Full | Moon position based |
| Yoga calculation | ✅ Full | Sun+Moon combination |
| Karana calculation | ✅ Full | Half-tithi derived |
| Vara calculation | ✅ Full | Weekday with sunrise |
| Muhurta system | ✅ Full | 48 daily periods |
| Hora system | ✅ Full | Planetary hours |
| Wisdom data | ✅ Full | All elements |
| Witness prompts | ✅ Full | Element-specific |

## API Endpoint

```
POST /api/v1/panchanga/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/panchanga/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "current_time": "2025-01-15T14:30:00Z",
    "location": {
      "latitude": 28.6139,
      "longitude": 77.2090
    }
  }'
```

## Dependencies

- Swiss Ephemeris for Sun/Moon positions
- Sunrise/sunset calculations
- Panchanga wisdom data

## References

- Classical Jyotish texts
- Lahiri ayanamsa for sidereal calculations

---

**Engine Version**: 0.1.0
**Required Phase**: 0
**Crate**: `crates/engine-panchanga/`
