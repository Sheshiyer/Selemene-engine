---
title: Tarot
sidebar_position: 13
---

## Summary

- **Engine ID:** `tarot`
- **Required phase:** `0`
- **Description:** Card-spread guidance for symbolic decision framing.

## Endpoint

`POST /api/v1/engines/tarot/calculate`

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
  "engine_id": "tarot",
  "result": {
    "spread": {
      "type": "horseshoe",
      "name": "Horseshoe Spread",
      "description": "A 7-card spread for situational clarity from past influences to likely outcome",
      "card_count": 7,
      "available_types": ["single_card", "three_card", "celtic_cross", "horseshoe", "relationship", "career", "yes_no"]
    },
    "question": "What should I focus on now?",
    "positions": [
      {
        "position": 0,
        "name": "Past Influence",
        "meaning": "What from the past still shapes this situation",
        "card": {
          "id": "m09",
          "name": "The Hermit",
          "arcana": "major",
          "number": 9,
          "isReversed": false,
          "interpretation": {
            "meaning": "A period of inner guidance and reflection",
            "keywords": ["solitude", "wisdom", "reflection"]
          }
        }
      }
    ],
    "cards": [
      {
        "position": 0,
        "name": "Past Influence",
        "cardName": "The Hermit"
      }
    ]
  },
  "witness_prompts": [
    {
      "prompt": "What does this spread ask you to notice?"
    }
  ],
  "calculated_at": "2026-03-03T12:00:00Z",
  "processing_time_ms": 23
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
    "spread": "three_card",
    "question": "What should I focus on now?",
    "spread_type": "three_card"
  }
}
```

## Supported spreads

- `single_card`
- `three_card`
- `celtic_cross`
- `horseshoe`
- `relationship`
- `career`
- `yes_no`
