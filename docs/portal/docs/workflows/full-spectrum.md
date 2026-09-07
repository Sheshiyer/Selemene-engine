---
title: Full Spectrum
sidebar_position: 7
---

## Overview

- **Workflow ID:** `full-spectrum`
- **Required phase:** `2`
- **Purpose:** Comprehensive consciousness portrait.

## Engine composition

- `all 19 runtime identities`

## Endpoint

There is no executable route for this workflow while its lossless adapter is unproven.

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
    "question": "What should I prioritize this week?"
  }
}
```

## Response example

```json
{
  "workflow_id": "full-spectrum",
  "engine_outputs": {
    "example-engine": {
      "engine_id": "example-engine",
      "result": {},
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

This workflow requires **phase 2** or higher. If user phase is lower, API returns a phase access error.
## Contract v1 support

`full-spectrum` is **unsupported** in the canonical workflow registry. Authenticated requests use exactly one `X-API-Key` or `Authorization: Bearer` header on the maintained `/api/v1/workflows/full-spectrum/execute` route. Outcomes follow `contracts/v1/schemas/workflow-outcome.schema.json` and preserve `complete`, `partial`, and `failed` status with one output or bounded failure per requested engine. The runtime catalogue has 19 identities and 17 public mirrors; Full Spectrum remains explicitly `unsupported` pending a lossless adapter test.
