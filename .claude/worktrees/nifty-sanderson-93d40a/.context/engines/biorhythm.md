# Biorhythm Engine Documentation

## Overview

Biorhythm theory calculates three fundamental cycles beginning at birth: Physical (23-day), Emotional (28-day), and Intellectual (33-day), tracking their sine wave patterns to predict daily energy levels.

## Purpose

Provide daily energy forecasting across physical, emotional, and mental dimensions to support activity planning and self-awareness.

## Calculation Type

**Natal + Temporal**: Birth date establishes cycle start; current date determines position.

## Input Parameters

```json
{
  "birth_data": {
    "date": "1990-03-15"
  },
  "current_time": "2025-01-15T12:00:00Z",
  "options": {
    "include_secondary_cycles": true,
    "forecast_days": 30
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| birth_data.date | string | Yes | ISO 8601 date |
| current_time | string | No | Query date (defaults to now) |
| options.include_secondary_cycles | boolean | No | Include derived cycles |
| options.forecast_days | number | No | Days to forecast (max 90) |

## Output Structure

```json
{
  "engine_id": "biorhythm",
  "success": true,
  "result": {
    "days_since_birth": 12725,
    "current_values": {
      "physical": {
        "value": 0.73,
        "percentage": 73,
        "phase": "High",
        "days_until_peak": 3,
        "days_until_critical": 8
      },
      "emotional": {
        "value": -0.15,
        "percentage": -15,
        "phase": "Descending",
        "days_until_peak": 12,
        "days_until_critical": 5
      },
      "intellectual": {
        "value": 0.45,
        "percentage": 45,
        "phase": "Ascending",
        "days_until_peak": 6,
        "days_until_critical": 14
      }
    },
    "secondary_cycles": {
      "passion": {
        "value": 0.29,
        "percentage": 29,
        "formula": "(Physical + Emotional) / 2"
      },
      "wisdom": {
        "value": 0.59,
        "percentage": 59,
        "formula": "(Emotional + Intellectual) / 2"
      },
      "mastery": {
        "value": 0.59,
        "percentage": 59,
        "formula": "(Physical + Intellectual) / 2"
      }
    },
    "critical_days": {
      "upcoming": ["2025-01-20", "2025-01-23"],
      "triple_critical": null
    },
    "forecast": [
      {"date": "2025-01-16", "physical": 0.65, "emotional": -0.22, "intellectual": 0.52},
      {"date": "2025-01-17", "physical": 0.55, "emotional": -0.29, "intellectual": 0.58}
    ]
  },
  "witness_prompt": "With physical energy high (73%) and emotions descending, how might body wisdom guide decisions today?",
  "consciousness_level": 0
}
```

## Primary Cycles

### Physical Cycle (23 days)
Governs physical strength, endurance, and coordination.

| Phase | Days | State |
|-------|------|-------|
| Ascending | 1-5.75 | Building strength |
| High | 5.75-11.5 | Peak performance |
| Descending | 11.5-17.25 | Recovery period |
| Low | 17.25-23 | Rest needed |

**Critical Day**: Day 1 (cycle start) and day ~11.5 (crossing zero)

### Emotional Cycle (28 days)
Governs mood, creativity, and sensitivity.

| Phase | Days | State |
|-------|------|-------|
| Ascending | 1-7 | Optimism rising |
| High | 7-14 | Emotional clarity |
| Descending | 14-21 | Introspection |
| Low | 21-28 | Vulnerability |

**Critical Day**: Day 1 and day 14 (crossing zero)

### Intellectual Cycle (33 days)
Governs learning, concentration, and problem-solving.

| Phase | Days | State |
|-------|------|-------|
| Ascending | 1-8.25 | Mental clarity rising |
| High | 8.25-16.5 | Peak cognition |
| Descending | 16.5-24.75 | Integration |
| Low | 24.75-33 | Mental rest |

**Critical Day**: Day 1 and day ~16.5 (crossing zero)

## Secondary (Derived) Cycles

| Cycle | Formula | Meaning |
|-------|---------|---------|
| Passion | (P + E) / 2 | Drive, motivation |
| Wisdom | (E + I) / 2 | Intuitive insight |
| Mastery | (P + I) / 2 | Skilled execution |
| Perception | (P + E + I) / 3 | Overall awareness |

## Critical Days

**Definition**: Days when a cycle crosses zero (transitioning from positive to negative or vice versa).

**Types**:
- **Single Critical**: One cycle at zero
- **Double Critical**: Two cycles at zero
- **Triple Critical**: All three at zero (rare, approximately every 21,252 days)

**Significance**: Critical days indicate instability and potential for accidents or poor decisions.

## Calculation Formula

```
value = sin(2π × days_since_birth / period)

Where:
- Physical period = 23 days
- Emotional period = 28 days
- Intellectual period = 33 days
```

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| High Energy | "With {cycle} at {value}% ({phase}), what {activity} serves you?" | "With physical at 73% (High), what physical challenges serve you?" |
| Critical Day | "On this critical day for {cycle}, what extra awareness does {domain} require?" | "On this critical day for emotional, what extra care does your mood require?" |
| Multi-Cycle | "Physical {p_phase} ({p_val}%), emotional {e_phase} ({e_val}%): how do body and heart collaborate?" | "Physical high (73%), emotional descending (-15%): how do body and heart collaborate?" |

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Physical cycle | ✅ Full | 23-day sine wave |
| Emotional cycle | ✅ Full | 28-day sine wave |
| Intellectual cycle | ✅ Full | 33-day sine wave |
| Secondary cycles | ✅ Full | 4 derived cycles |
| Critical day detection | ✅ Full | Single/double/triple |
| Forecasting | ✅ Full | Up to 90 days |
| Wisdom data | ✅ Full | Phase descriptions |
| Witness prompts | ✅ Full | Cycle-aware |

## API Endpoint

```
POST /api/v1/engines/biorhythm/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/engines/biorhythm/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1990-03-15"
    },
    "options": {
      "forecast_days": 7
    }
  }'
```

## Dependencies

- None (pure mathematical calculations)
- Biorhythm wisdom data

## References

- Wilhelm Fliess's original biorhythm theory
- Modern computational biorhythm models

---

**Engine Version**: 0.1.0
**Required Phase**: 0
**Crate**: `crates/engine-biorhythm/`
