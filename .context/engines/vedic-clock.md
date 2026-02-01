# Vedic Clock Engine Documentation

## Overview

The Vedic Clock engine calculates traditional Ayurvedic time divisions (doshas, muhurtas) and Ghati time, providing recommendations based on the quality of each time period.

## Purpose

Map clock time to Vedic time categories to support daily routine alignment with natural energy cycles according to Ayurvedic wisdom.

## Calculation Type

**Temporal**: Based on current time and solar position (sunrise/sunset).

## Input Parameters

```json
{
  "current_time": "2025-01-15T06:30:00Z",
  "location": {
    "latitude": 28.6139,
    "longitude": 77.2090
  },
  "options": {
    "include_muhurtas": true,
    "include_recommendations": true
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| current_time | string | Yes | ISO 8601 datetime |
| location.latitude | number | Yes | -90 to 90 |
| location.longitude | number | Yes | -180 to 180 |
| options.include_muhurtas | boolean | No | Include muhurta details |
| options.include_recommendations | boolean | No | Include activity suggestions |

## Output Structure

```json
{
  "engine_id": "vedic-clock",
  "success": true,
  "result": {
    "ghati_time": {
      "ghati": 24,
      "pala": 30,
      "vipala": 15,
      "display": "24:30:15",
      "day_percentage": 40.8
    },
    "current_dosha": {
      "dosha": "Kapha",
      "period": "Morning",
      "start_time": "06:00",
      "end_time": "10:00",
      "qualities": ["Heavy", "Slow", "Steady", "Cool"],
      "recommendations": {
        "activities": ["Exercise", "Routine tasks", "Grounding practices"],
        "avoid": ["Heavy meals", "Sleeping late", "Lethargy"]
      }
    },
    "muhurta": {
      "name": "Brahma",
      "number": 1,
      "nature": "Auspicious",
      "deity": "Brahma",
      "duration_minutes": 48,
      "suitable_for": ["Spiritual practice", "Study", "Planning"]
    },
    "solar_data": {
      "sunrise": "06:52",
      "sunset": "17:45",
      "solar_noon": "12:18",
      "day_length_hours": 10.88
    },
    "daily_schedule": {
      "brahma_muhurta": "04:24-05:12",
      "sunrise": "06:52",
      "kapha_morning": "06:00-10:00",
      "pitta_midday": "10:00-14:00",
      "vata_afternoon": "14:00-18:00",
      "kapha_evening": "18:00-22:00",
      "pitta_night": "22:00-02:00",
      "vata_night": "02:00-06:00"
    }
  },
  "witness_prompt": "In this Kapha morning period, what steady, grounding action aligns with the heavy, slow quality of this time?",
  "consciousness_level": 0
}
```

## Dosha Time Periods

The 24-hour day is divided into six 4-hour periods, each ruled by a dosha (repeated twice).

| Period | Time | Dosha | Quality |
|--------|------|-------|---------|
| Morning | 06:00-10:00 | Kapha | Heavy, slow, steady |
| Midday | 10:00-14:00 | Pitta | Hot, sharp, transformative |
| Afternoon | 14:00-18:00 | Vata | Light, mobile, creative |
| Evening | 18:00-22:00 | Kapha | Heavy, slow, settling |
| Late Night | 22:00-02:00 | Pitta | Hot, digestive (of day) |
| Early Morning | 02:00-06:00 | Vata | Light, subtle, spiritual |

## Ghati Time System

Traditional Vedic time measurement:

| Unit | Western Equivalent |
|------|-------------------|
| 1 Day | 60 Ghatis |
| 1 Ghati | 24 minutes |
| 1 Pala | 24 seconds |
| 1 Vipala | 0.4 seconds |

**Calculation**: `ghati = (minutes_since_sunrise / 24) mod 60`

## Muhurta System

30 muhurtas per day (each ~48 minutes), half during day and half at night.

| Muhurta | Number | Nature | Suitable Activities |
|---------|--------|--------|---------------------|
| Brahma | 1 | Auspicious | Spiritual practice, study |
| Pratar | 2 | Auspicious | Beginning new activities |
| Savitri | 3 | Mixed | Routine work |
| ... | ... | ... | ... |
| Abhijit | 8 | Very Auspicious | All important activities |
| ... | ... | ... | ... |

**Brahma Muhurta**: Special period ~96-48 minutes before sunrise, ideal for spiritual practice.

## Dosha Recommendations

### Kapha (06:00-10:00, 18:00-22:00)
**Optimize**: Counter heaviness with activity
- ✅ Exercise, vigorous movement
- ✅ Stimulating activities
- ✅ Light, warm foods
- ❌ Heavy meals, sleeping

### Pitta (10:00-14:00, 22:00-02:00)
**Optimize**: Use transformative energy wisely
- ✅ Complex problem-solving
- ✅ Important decisions
- ✅ Moderate, cooling foods
- ❌ Overheating, arguments

### Vata (14:00-18:00, 02:00-06:00)
**Optimize**: Ground creative energy
- ✅ Creative work
- ✅ Light activities
- ✅ Warm, grounding foods
- ❌ Overstimulation, anxiety

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| Dosha Period | "In this {dosha} period ({quality}), what {activity_type} aligns with this energy?" | "In this Kapha period (heavy, slow), what grounding activity aligns with this energy?" |
| Muhurta | "During {muhurta} muhurta ({nature}), what {action} serves your intentions?" | "During Abhijit muhurta (very auspicious), what important action serves your intentions?" |
| Brahma Muhurta | "In Brahma muhurta's stillness, what spiritual practice calls?" | - |

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Ghati calculation | ✅ Full | Sunrise-based |
| Dosha periods | ✅ Full | 6 periods/day |
| Muhurta calculation | ✅ Full | 30 muhurtas |
| Sunrise/sunset | ✅ Full | Swiss Ephemeris |
| Recommendations | ✅ Full | Dosha-specific |
| Wisdom data | ✅ Full | All doshas/muhurtas |
| Witness prompts | ✅ Full | Time-appropriate |

## API Endpoint

```
POST /api/v1/engines/vedic-clock/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/engines/vedic-clock/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "current_time": "2025-01-15T08:30:00Z",
    "location": {
      "latitude": 28.6139,
      "longitude": 77.2090
    }
  }'
```

## Dependencies

- Swiss Ephemeris for sunrise/sunset
- Ayurvedic wisdom data

## References

- Ayurvedic clock systems
- Classical Jyotish muhurta texts

---

**Engine Version**: 0.1.0
**Required Phase**: 0
**Crate**: `crates/engine-vedic-clock/`
