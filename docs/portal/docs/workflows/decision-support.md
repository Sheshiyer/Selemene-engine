---
title: Decision Support
sidebar_position: 4
---

## Overview

- **Workflow ID:** `decision-support`
- **Required phase:** `1`
- **Purpose:** Multi-perspective guidance for choices.

## Engine composition

- `tarot`
- `i-ching`
- `human-design`
- `enneagram`
- `gene-keys`

## Endpoint

`POST /api/v1/workflows/decision-support/execute`

## Request example

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
    "question": "What should I prioritize this week?",
    "spread": "yes_no"
  }
}
```

## Response example

```json
{
  "workflow_id": "decision-support",
  "engine_outputs": {
    "tarot": {
      "engine_id": "tarot",
      "result": {
        "spread": {
          "type": "yes_no",
          "name": "Yes or No",
          "card_count": 1
        },
        "cards": [
          {
            "position": 0,
            "cardName": "The Fool"
          }
        ],
        "decision": {
          "answer": "yes",
          "confidence": 0.74
        }
      },
      "witness_prompt": "...",
      "metadata": {"processing_time_ms": 12.3}
    }
  },
  "synthesis": {
    "themes": ["clarity through structure"],
    "alignments": ["timing supports intent"],
    "tensions": ["certainty vs exploration"]
  },
  "total_time_ms": 48.1,
  "timestamp": "2026-03-03T12:00:01Z"
}
```

## Consciousness phase gating

This workflow requires **phase 1** or higher. If user phase is lower, API returns a phase access error.

## Tarot spread options

- `single_card`
- `three_card`
- `celtic_cross`
- `horseshoe`
- `relationship`
- `career`
- `yes_no`
