# TUI Integration Guide — Noesis Terminal Interface

**TUI**: The Noesis Terminal User Interface — a rich, interactive terminal viewer for all 16 consciousness engines.

**Production URLs**: `3437.tryambakam.space` | `noesis.tryambakam.space`  
**Backend API**: `https://selemene.tryambakam.space`

---

## Overview

The TUI exposes the full Noesis engine stack in a terminal-native experience. It consumes the same REST API as all other consumers and maps each engine's output to rich terminal primitives (tables, bars, sparklines, panels).

The name `3437` is a gematria derivation — this is the canonical URL. `noesis.tryambakam.space` is the alias.

---

## Authentication

The TUI uses the same API key mechanism as the REST API:

```bash
# Set your API key
export NOESIS_API_KEY="nk_your_key_here"
```

Keys are sent as `X-Api-Key: <key>` header for `nk_` prefixed keys, or `Authorization: Bearer <token>` for JWT.

---

## Core API Contract

### Health check

```
GET https://selemene.tryambakam.space/health/live
```

```json
{
  "status": "ok",
  "version": "3.3.0",
  "engines_loaded": 17,
  "workflows_loaded": 6
}
```

### Single engine calculation

```
POST https://selemene.tryambakam.space/api/v1/engines/{engine_id}/calculate
Content-Type: application/json
X-Api-Key: nk_...

{
  "birth_data": {
    "date": "1991-08-13",
    "time": "13:19",
    "timezone": "Asia/Kolkata",
    "latitude": 12.9716,
    "longitude": 77.5946
  }
}
```

### Full-spectrum workflow (all 16 engines)

```
POST https://selemene.tryambakam.space/api/v1/workflows/full-spectrum/execute
Content-Type: application/json
X-Api-Key: nk_...
```

Response includes `reading_id`, `reading_url`, `witness_layer`, and per-engine `engine_outputs`.

---

## Engine ID Reference

| Engine ID | Name | Key output fields |
|---|---|---|
| `panchanga` | Vedic Calendar | `tithi`, `nakshatra`, `yoga`, `karana`, `masa` |
| `human-design` | Human Design | `type`, `profile`, `centers`, `gates`, `channels` |
| `gene-keys` | Gene Keys | `shadow`, `gift`, `siddhi`, `sequence` |
| `vimshottari` | Vimshottari Dasha | `maha_dasha`, `antar_dasha`, `pratyantar_dasha` |
| `numerology` | Numerology | `life_path`, `expression`, `soul_urge`, `birthday` |
| `biorhythm` | Biorhythm | `physical`, `emotional`, `intellectual`, `cycles` |
| `vedic-clock` | Vedic Clock | `muhurta`, `hora`, `prana` |
| `biofield` | Biofield | `chakras`, `aura_layer`, `dominant_element` |
| `face-reading` | Face Reading | `dominant_element`, `regions`, `insights` |
| `nadabrahman` | Nadabrahman | `base_frequency`, `overtones`, `sound_archetype` |
| `transits` | Transits | `current_transits`, `aspects`, `sade_sati` |
| `tarot` | Tarot | `drawn_cards`, `positions`, `synthesis` |
| `i-ching` | I-Ching | `hexagram`, `changing_lines`, `judgment` |
| `enneagram` | Enneagram | `type`, `wing`, `instinct`, `levels` |
| `sacred-geometry` | Sacred Geometry | `primary_form`, `phi_ratio`, `geometric_sequence` |
| `sigil-forge` | Sigil Forge | `intent_sigil`, `power_nodes`, `activation_method` |

---

## v3.3.0 Reading Object Contract

All workflow responses include:

```json
{
  "reading_id": "r_abc123",
  "reading_url": "https://noesis.tryambakam.space/readings/r_abc123",
  "created_at": "2026-05-08T10:00:00Z",
  "subject": "Birth Blueprint Reading",
  "evidence": ["Sun in Leo", "HD Type: Generator", "Life Path 3"],
  "witness_layer": {
    "title": "The Field Sees",
    "summary": "...",
    "convergences": ["..."],
    "frictions": ["..."],
    "practice": "...",
    "question": "What are you not saying?"
  },
  "engine_outputs": [...]
}
```

---

## Rate Limits

Rate limit state is returned in response headers:

| Header | Description |
|---|---|
| `X-RateLimit-Limit` | Requests per minute |
| `X-RateLimit-Remaining` | Remaining this window |
| `X-RateLimit-Reset` | Unix timestamp of reset |
| `X-RateLimit-Daily-Remaining` | Daily engine calls remaining |

---

## SDK Usage

```typescript
import { NoesisClient } from "@noesis/sdk";

const client = new NoesisClient("https://selemene.tryambakam.space", {
  authToken: process.env.NOESIS_API_KEY,
});

// Full-spectrum (all 16 engines)
const result = await client.workflow("full-spectrum", {
  birth_data: { date: "1991-08-13", time: "13:19", timezone: "Asia/Kolkata" }
});

// Access reading-object
console.log(result.reading_id, result.witness_layer?.question);
```

---

## Related

- [API Reference](./README.md)
- [Workflows Guide](./workflows.md)
- [Engines Guide](./engines.md)
- [TOI Integration](./TOI_INTEGRATION.md)
