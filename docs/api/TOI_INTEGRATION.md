# TOI Integration Guide — Tool of Intent

**TOI**: Tool of Intent — the structured intent-setting and decision support surface for Noesis.

**Backend API**: `https://selemene.tryambakam.space`  
**Web viewer**: `https://noesis.tryambakam.space`

---

## Overview

The TOI integrates with Noesis to provide intent-aware consciousness calculations. It uses the `decision-support` and `self-inquiry` workflows, combined with the witness interpretation endpoint, to ground intent in multi-engine cosmic context.

**Key workflows for TOI**:
- `decision-support` — 5-engine cross-validation for major decisions
- `self-inquiry` — 4-engine depth probe for shadow and gift work
- `daily-practice` — 3-engine daily ritual anchor

---

## Authentication

```bash
export NOESIS_API_KEY="nk_your_key_here"
```

Send as `X-Api-Key: <key>` header.

---

## Workflow: Decision Support

Best for: major life decisions, crossroads, timing questions.

```
POST https://selemene.tryambakam.space/api/v1/workflows/decision-support/execute
X-Api-Key: nk_...

{
  "birth_data": {
    "date": "1991-08-13",
    "time": "13:19",
    "timezone": "Asia/Kolkata",
    "latitude": 12.9716,
    "longitude": 77.5946
  },
  "options": {
    "intent": "Should I launch the product this month?"
  }
}
```

---

## Workflow: Self-Inquiry

Best for: shadow work, gift activation, purpose exploration.

```
POST https://selemene.tryambakam.space/api/v1/workflows/self-inquiry/execute
X-Api-Key: nk_...
```

---

## Witness Interpretation

Send any text or reading ID to get a witness-layer interpretation:

```
POST https://selemene.tryambakam.space/api/v1/witness/interpret
X-Api-Key: nk_...

{
  "text": "I feel blocked in creative expression",
  "context": { "engine": "gene-keys" }
}
```

Response:

```json
{
  "interpretation": "...",
  "suggestions": ["...", "..."]
}
```

---

## Intent + Timing Pattern

Combine `transits` (current cosmic weather) with `vedic-clock` (timing) for intent anchoring:

```typescript
// Get timing context
const timing = await client.calculate("vedic-clock", { birth_data });
const transits = await client.calculate("transits", { birth_data });

// Get intent support
const intent = await client.workflow("decision-support", { birth_data });

// Synthesize with witness
const witness = await client.interpretWitness({
  text: `Timing: ${timing.result.muhurta}. Intent: launch product.`,
  context: { transit_summary: transits.result }
});
```

---

## v3.3.0 Reading Object Contract

All workflow responses return a persisted reading:

```json
{
  "reading_id": "r_abc123",
  "reading_url": "https://noesis.tryambakam.space/readings/r_abc123",
  "subject": "Decision Support Reading",
  "evidence": ["Mars trine Saturn — good for structure", "HD Sacral defined"],
  "witness_layer": {
    "title": "The Pattern",
    "summary": "Your body knows before your mind does.",
    "convergences": ["Gate 34 active", "Ketu cycle ending"],
    "frictions": ["Saturn square Midheaven — delay signal"],
    "practice": "Rest before the decision. Let the sacral respond.",
    "question": "What would you do if the timing were perfect?"
  }
}
```

---

## Readings API (Intent Archive)

Retrieve past readings for review:

```
GET https://selemene.tryambakam.space/api/v1/readings?workflow_id=decision-support
X-Api-Key: nk_...
```

```json
[
  {
    "id": "r_abc123",
    "workflow_id": "decision-support",
    "created_at": "2026-05-08T10:00:00Z",
    "subject": "Decision Support Reading",
    "reading_url": "..."
  }
]
```

---

## SDK Usage

```typescript
import { NoesisClient } from "@noesis/sdk";

const client = new NoesisClient("https://selemene.tryambakam.space", {
  authToken: process.env.NOESIS_API_KEY,
});

// Decision support workflow
const reading = await client.workflow("decision-support", { birth_data });
console.log(reading.witness_layer?.question);

// Archive
const history = await client.listReadings({ workflow_id: "decision-support" });
```

---

## Related

- [TUI Integration](./TUI_INTEGRATION.md)
- [Workflows Guide](./workflows.md)
- [LLM Agent Guide](./LLM_AGENT_GUIDE.md)
