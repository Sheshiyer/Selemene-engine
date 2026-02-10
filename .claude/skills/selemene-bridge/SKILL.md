---
name: selemene-bridge
description: "Selemene Engine Bridge: Query 14 consciousness engines and 6 workflows for witnessing cosmic patterns. Non-prescriptive mirror — generates inquiry, not advice."
tools:
  - name: selemene_health
    description: "Check Selemene engine health and loaded engines/workflows"
    parameters:
      type: object
      properties: {}
  - name: selemene_list_engines
    description: "List all available consciousness engines with metadata"
    parameters:
      type: object
      properties: {}
  - name: selemene_engine_info
    description: "Get detailed metadata for a specific engine"
    parameters:
      type: object
      properties:
        engine_id:
          type: string
          description: "Engine ID: panchanga, numerology, biorhythm, human-design, gene-keys, vimshottari, biofield, vedic-clock, face-reading, tarot, i-ching, enneagram, sacred-geometry, sigil-forge"
      required: [engine_id]
  - name: selemene_calculate
    description: "Run a calculation on a specific consciousness engine"
    parameters:
      type: object
      properties:
        engine_id:
          type: string
          description: "Engine ID to calculate with"
        parameters:
          type: object
          description: "Engine-specific input parameters (birth data, location, etc.)"
        consciousness_level:
          type: integer
          description: "Required consciousness phase (0-5). Higher phases unlock deeper engines."
      required: [engine_id, parameters, consciousness_level]
  - name: selemene_workflow_execute
    description: "Execute a multi-engine workflow (birth-blueprint, daily-practice, decision-support, self-inquiry, creative-expression, full-spectrum)"
    parameters:
      type: object
      properties:
        workflow_id:
          type: string
          description: "Workflow ID to execute"
        parameters:
          type: object
          description: "Workflow parameters including birth data, location, question"
        consciousness_level:
          type: integer
          description: "Required consciousness phase for this workflow"
      required: [workflow_id, parameters, consciousness_level]
---

# Selemene Engine Bridge

Selemene is a consciousness engine platform with 14 engines spanning Vedic astrology, numerology, Human Design, Gene Keys, and more. This skill lets you query it directly.

## Quick Start

```bash
# Check engine health
python .claude/skills/selemene-bridge/scripts/bridge.py selemene_health

# Calculate panchanga for Bengaluru
python .claude/skills/selemene-bridge/scripts/bridge.py selemene_calculate '{"engine_id":"panchanga","parameters":{"latitude":12.9716,"longitude":77.5946},"consciousness_level":0}'

# Run self-inquiry workflow
python .claude/skills/selemene-bridge/scripts/bridge.py selemene_workflow_execute '{"workflow_id":"self-inquiry","parameters":{"birth_date":"1991-08-13","birth_time":"13:31","latitude":12.97,"longitude":77.59,"timezone":"Asia/Kolkata","question":"What patterns emerge today?"},"consciousness_level":2}'
```

## Engines (14 total)

| Engine | Phase | Type | Description |
|--------|-------|------|-------------|
| panchanga | 0 | Rust | Vedic time qualities (Tithi, Nakshatra, Yoga, Karana) |
| numerology | 0 | Rust | Pythagorean + Chaldean number analysis |
| biorhythm | 0 | Rust | Physical, emotional, intellectual cycles |
| human-design | 0 | Rust | Human Design bodygraph from birth data |
| gene-keys | 1 | Rust | Gene Keys profile and shadow/gift/siddhi |
| vimshottari | 0 | Rust | Vimshottari dasha planetary periods |
| biofield | 1 | Rust | Biofield analysis |
| vedic-clock | 0 | Rust | Vedic time divisions (muhurta, ghati) |
| face-reading | 2 | Rust | Physiognomy analysis |
| tarot | 1 | TS | Tarot card draws and spreads |
| i-ching | 1 | TS | I Ching hexagram casting |
| enneagram | 1 | TS | Enneagram type analysis |
| sacred-geometry | 1 | TS | Sacred geometry patterns |
| sigil-forge | 1 | TS | Sigil creation and analysis |

## Workflows (6 total)

| Workflow | Phase | Engines Used |
|----------|-------|-------------|
| birth-blueprint | 0 | numerology, human-design, vimshottari |
| daily-practice | 0 | panchanga, vedic-clock, biorhythm |
| decision-support | 1 | tarot, i-ching, human-design |
| self-inquiry | 2 | gene-keys, enneagram |
| creative-expression | 1 | sigil-forge, sacred-geometry |
| full-spectrum | 3 | All 14 engines |

## Environment Variables

- `SELEMENE_URL` — Rust server URL (default: http://localhost:8080)
- `SELEMENE_TS_URL` — TS engines URL (default: http://localhost:3001)
- `SELEMENE_API_KEY` — API key or JWT token for authenticated endpoints
