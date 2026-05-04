---
title: Biorhythm
sidebar_position: 4
---

## Summary

- **Engine ID:** `biorhythm`
- **Required phase:** `0`
- **Description:** Multi-cycle biorhythm engine — primary, secondary, and compatibility modes.

## Endpoint

`POST /api/v1/engines/biorhythm/calculate`

## Input parameters

Uses shared `EngineInput`:

```json
{
  "birth_data": {
    "name": "Asha",
    "date": "1990-05-15",
    "time": "14:30",
    "latitude": 12.9716,
    "longitude": 77.5946,
    "timezone": "Asia/Kolkata"
  },
  "current_time": "2026-03-03T12:00:00Z",
  "precision": "standard",
  "options": {
    "forecast_days": 7,
    "partner_birth_date": "1992-08-14"
  }
}
```

### Options

| Option | Type | Default | Description |
|---|---|---|---|
| `forecast_days` | integer | `7` | Number of days to include in the forecast window. Set to `0` to disable. |
| `partner_birth_date` | string (`YYYY-MM-DD`) | — | If provided, calculates **biorhythm compatibility** between you and a partner. Adds a `compatibility` block to the response. |

## Response example

```json
{
  "engine_id": "biorhythm",
  "result": {
    "days_alive": 13077,
    "target_date": "2026-03-03",
    "physical":     { "value": 0.72, "percentage": 86.0, "phase": "Rising", "days_until_peak": 3, "days_until_critical": 8, "is_critical": false, "cycle_day": 19 },
    "emotional":    { "value": -0.43, "percentage": 28.5, "phase": "Falling", "days_until_peak": 18, "days_until_critical": 2, "is_critical": false, "cycle_day": 22 },
    "intellectual": { "value": 0.10, "percentage": 55.0, "phase": "Rising", "days_until_peak": 7, "days_until_critical": 1, "is_critical": false, "cycle_day": 9 },
    "intuitive":    { "value": -0.25, "percentage": 37.5, "phase": "Falling", "days_until_peak": 22, "days_until_critical": 4, "is_critical": false, "cycle_day": 30 },
    "spiritual":    { "value": 0.55, "percentage": 77.5, "phase": "Rising", "days_until_peak": 6, "days_until_critical": 12, "is_critical": false, "cycle_day": 16 },
    "mastery": 70.5,
    "passion": 57.3,
    "wisdom": 41.8,
    "overall_energy": 56.5,
    "critical_days": ["2026-03-05"],
    "forecast": [
      {
        "date": "2026-03-04",
        "days_alive": 13078,
        "physical": 88.2,
        "emotional": 25.1,
        "intellectual": 57.3,
        "intuitive": 35.0,
        "aesthetic": 62.4,
        "spiritual": 80.1,
        "overall_energy": 56.9
      }
    ],
    "compatibility": {
      "birth_date_a": "1990-05-15",
      "birth_date_b": "1992-08-14",
      "target_date": "2026-03-03",
      "physical":     { "score": 78.4, "period": 23.0, "days_diff": 821 },
      "emotional":    { "score": 45.2, "period": 28.0, "days_diff": 821 },
      "intellectual": { "score": 91.0, "period": 33.0, "days_diff": 821 },
      "intuitive":    { "score": 60.5, "period": 38.0, "days_diff": 821 },
      "overall": 71.5
    }
  },
  "witness_prompt": "A reflective prompt tuned to your current phase.",
  "metadata": {
    "consciousness_level": 0,
    "processing_time_ms": 1.2,
    "version": "3.0.0",
    "cache_hit": false
  }
}
```

## Output fields

### Primary cycles

All primary cycles (`physical`, `emotional`, `intellectual`) include:

| Field | Type | Description |
|---|---|---|
| `value` | float [-1, 1] | Raw sine value of the cycle |
| `percentage` | float [0, 100] | Mapped to 0–100 % |
| `phase` | string | `Peak`, `Rising`, `Falling`, `Low`, or `Critical` |
| `days_until_peak` | integer | Days until next positive peak |
| `days_until_critical` | integer | Days until next zero crossing |
| `is_critical` | boolean | Whether today is near a zero crossing |
| `cycle_day` | integer | Day within the current cycle period |

### Secondary cycles

`intuitive` (38-day) and `spiritual` (53-day) return the same per-field structure as primary cycles.

The 7-day `forecast` array also includes `aesthetic` (43-day) and `spiritual` as percentage values for each day.

### Composite scores

| Field | Type | Description |
|---|---|---|
| `mastery` | float [0, 100] | Average of physical + intellectual |
| `passion` | float [0, 100] | Average of physical + emotional |
| `wisdom` | float [0, 100] | Average of emotional + intellectual |
| `overall_energy` | float [0, 100] | Equal-weight average of three primary cycles |

### Compatibility block

Present only when `partner_birth_date` is supplied. Scores are computed as:

```
score = 50 × (1 + cos(2π × (|days_diff| mod period) / period))
```

This gives:
- **100** when both people share the same phase (days_diff ≡ 0 mod period)
- **0** when they are in opposite phases (days_diff ≡ period/2)
- **50** at quadrature (90° offset)

| Field | Description |
|---|---|
| `physical.score` | Physical cycle compatibility [0, 100] |
| `emotional.score` | Emotional cycle compatibility [0, 100] |
| `intellectual.score` | Intellectual cycle compatibility [0, 100] |
| `intuitive.score` | Intuitive cycle compatibility [0, 100] |
| `overall` | Equal-weighted average of physical, emotional, intellectual |

## Witness prompt examples by consciousness level

- **0 · Dormant:** grounding, concrete, low-friction prompts
- **1 · Glimpsing:** pattern-noticing prompts with simple experiments
- **2 · Practicing:** reflective prompts with behavioral commitments
- **3 · Integrated:** synthesis prompts spanning multiple mirrors
- **4 · Embodied:** action-integrated, relationally-aware prompts

