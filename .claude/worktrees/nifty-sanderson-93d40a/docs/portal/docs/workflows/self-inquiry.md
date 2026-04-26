---
title: Self Inquiry
sidebar_position: 5
---

## Overview

- **Workflow ID:** `self-inquiry`
- **Required phase:** `2`
- **Purpose:** Shadow/work pattern inquiry synthesis.

## Engine composition

- `gene-keys`
- `enneagram`

## Endpoint

`POST /api/v1/workflows/self-inquiry/execute`

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
  "workflow_id": "self-inquiry",
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
