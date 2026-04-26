---
title: Nadabrahman
sidebar_position: 11
---

## Summary

- **Engine ID:** `nadabrahman`
- **Required phase:** `0`
- **Description:** Sound and vibration-oriented reflective mirror.

## Endpoint

`POST /api/v1/engines/nadabrahman/calculate`

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
  "options": {}
}
```

## Response example

```json
{
  "engine_id": "nadabrahman",
  "timestamp": "2026-03-03T12:00:00Z",
  "result": {"example": "typed-result-fields-see-openapi"},
  "witness_prompt": "A reflective prompt tuned to your current phase.",
  "metadata": {
    "consciousness_level": 0,
    "processing_time_ms": 23.4,
    "version": "3.0.0",
    "cache_hit": false
  }
}
```

## Witness prompt examples by consciousness level

- **0 · Dormant:** grounding, concrete, low-friction prompts
- **1 · Glimpsing:** pattern-noticing prompts with simple experiments
- **2 · Practicing:** reflective prompts with behavioral commitments
- **3 · Integrated:** synthesis prompts spanning multiple mirrors
- **4 · Embodied:** action-integrated, relationally-aware prompts

## Common options

```json
{
  "options": {
    "question": "What should I focus on now?",
    "intent": "clarity",
    "intent_text": "Create aligned momentum"
  }
}
```
