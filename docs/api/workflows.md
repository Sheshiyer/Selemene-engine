# Workflow API Endpoints

## Overview

Workflow endpoints orchestrate multiple engines in parallel, providing synthesized insights across consciousness systems.

## Base Path

```
/api/v1/workflows
```

## Available Workflows

| ID | Name | Engines | TTL |
|----|------|---------|-----|
| birth-blueprint | Birth Blueprint | numerology, human-design, gene-keys | 24h |
| daily-practice | Daily Practice | panchanga, vedic-clock, biorhythm | 1h |
| decision-support | Decision Support | tarot, i-ching, human-design | 15m |
| self-inquiry | Self-Inquiry | gene-keys, enneagram | 24h |
| creative-expression | Creative Expression | sigil-forge, sacred-geometry | 15m |
| full-spectrum | Full Spectrum | All 14 engines | 1h |

---

## Execute Workflow

### Endpoint
```
POST /api/v1/workflows/{workflow_id}/execute
```

### Request
```json
{
  "birth_data": {
    "date": "1990-03-15",
    "time": "14:30",
    "latitude": 40.7128,
    "longitude": -74.0060,
    "timezone": "America/New_York"
  },
  "current_time": "2025-01-15T12:00:00Z",
  "options": {
    "include_synthesis": true
  }
}
```

### Response
```json
{
  "workflow_id": "birth-blueprint",
  "engine_outputs": {
    "numerology": {
      "engine_id": "numerology",
      "result": {
        "life_path": {"number": 9, "meaning": "Humanitarian"},
        "expression": {"number": 7, "meaning": "Seeker"}
      },
      "witness_prompt": "With Life Path 9..."
    },
    "human-design": {
      "engine_id": "human-design",
      "result": {
        "hd_type": "Generator",
        "authority": "Sacral",
        "profile": "1/3"
      },
      "witness_prompt": "What does it feel like..."
    },
    "gene-keys": {
      "engine_id": "gene-keys",
      "result": {
        "activation_sequence": {...}
      },
      "witness_prompt": "How do your gifts..."
    }
  },
  "synthesis": {
    "primary_themes": [
      {
        "theme": "Service",
        "occurrences": 3,
        "sources": ["numerology", "human-design", "gene-keys"],
        "narrative": "Service emerges across all three systems..."
      }
    ],
    "secondary_themes": [
      {"theme": "Introspection", "occurrences": 2}
    ],
    "alignments": [
      {
        "type": "identity",
        "theme": "Seeking through Service",
        "sources": ["Life Path 9", "Generator Type", "Gene Key 17 Gift"],
        "narrative": "Your seeker nature finds expression through serving others..."
      }
    ],
    "tensions": [],
    "narrative": "Your birth blueprint reveals a pattern of service through seeking wisdom...",
    "witness_prompt": "With service appearing across all three systems, how do you already serve without trying?"
  },
  "total_time_ms": 45.2,
  "timestamp": "2025-01-15T12:00:01Z"
}
```

---

## Birth Blueprint Workflow

### Endpoint
```
POST /api/v1/workflows/birth-blueprint/execute
```

### Engines
- Numerology (Life Path, Expression, Soul Urge)
- Human Design (Type, Authority, Profile)
- Gene Keys (Activation Sequence)

### Required Input
- `birth_data` with date, time, coordinates, timezone

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/workflows/birth-blueprint/execute \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1990-03-15",
      "time": "14:30",
      "latitude": 40.7128,
      "longitude": -74.0060,
      "timezone": "America/New_York"
    }
  }'
```

### Synthesis Focus
- Identity patterns across systems
- Life purpose alignment
- Core gifts and challenges

---

## Daily Practice Workflow

### Endpoint
```
POST /api/v1/workflows/daily-practice/execute
```

### Engines
- Panchanga (Tithi, Nakshatra, Yoga)
- Vedic Clock (Dosha period, Muhurta)
- Biorhythm (Physical, Emotional, Intellectual)

### Required Input
- `birth_data.date` for biorhythm
- `current_time` (or defaults to now)
- `location` for Panchanga/Vedic Clock

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/workflows/daily-practice/execute \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {"date": "1990-03-15"},
    "current_time": "2025-01-15T08:00:00Z",
    "location": {"latitude": 28.6139, "longitude": 77.2090}
  }'
```

### Synthesis Focus
- Optimal timing windows
- Energy level alignment
- Daily rhythm recommendations

---

## Decision Support Workflow

### Endpoint
```
POST /api/v1/workflows/decision-support/execute
```

### Engines
- Tarot (Three-card or specified spread)
- I-Ching (Hexagram with changing lines)
- Human Design (Authority for decision-making)

### Required Input
- `birth_data` for Human Design Authority
- `options.question` (optional but recommended)

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/workflows/decision-support/execute \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1990-03-15",
      "time": "14:30",
      "latitude": 40.7128,
      "longitude": -74.0060,
      "timezone": "America/New_York"
    },
    "options": {
      "question": "Should I accept this job offer?"
    }
  }'
```

### Synthesis Focus
- Cross-oracle theme detection
- Alignment with personal authority
- Multiple perspective integration

---

## Self-Inquiry Workflow

### Endpoint
```
POST /api/v1/workflows/self-inquiry/execute
```

### Engines
- Gene Keys (Shadows, Gifts, Siddhis)
- Enneagram (Type, Fixation, Virtue)

### Required Input
- `birth_data` for Gene Keys
- `options.enneagram_type` (if known)

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/workflows/self-inquiry/execute \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1990-03-15",
      "time": "14:30",
      "latitude": 40.7128,
      "longitude": -74.0060,
      "timezone": "America/New_York"
    },
    "options": {
      "enneagram_type": 4,
      "consciousness_level": 3
    }
  }'
```

### Synthesis Focus
- Shadow-fixation mapping
- Growth edge identification
- Transformation pathways

---

## Creative Expression Workflow

### Endpoint
```
POST /api/v1/workflows/creative-expression/execute
```

### Engines
- Sigil Forge (Intention encoding)
- Sacred Geometry (Pattern generation)

### Required Input
- `options.intention` for sigil
- `options.pattern_type` for geometry (optional)

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/workflows/creative-expression/execute \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {
      "intention": "I manifest abundance with ease",
      "pattern_type": "flower_of_life"
    }
  }'
```

### Synthesis Focus
- Visual symbol creation
- Geometric-intention alignment
- Ritual/meditation support

---

## Full Spectrum Workflow

### Endpoint
```
POST /api/v1/workflows/full-spectrum/execute
```

### Engines
All 14 engines execute in parallel:
- numerology, human-design, gene-keys, vimshottari
- panchanga, vedic-clock, biorhythm
- tarot, i-ching, enneagram
- biofield, face-reading
- sacred-geometry, sigil-forge

### Required Input
- `birth_data` (complete)
- `current_time` (or defaults to now)
- `location` for temporal engines
- Various `options` for specific engines

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/workflows/full-spectrum/execute \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1990-03-15",
      "time": "14:30",
      "latitude": 40.7128,
      "longitude": -74.0060,
      "timezone": "America/New_York"
    },
    "current_time": "2025-01-15T12:00:00Z",
    "location": {"latitude": 40.7128, "longitude": -74.0060},
    "options": {
      "enneagram_type": 4,
      "tarot_spread": "three_card",
      "intention": "clarity and purpose"
    }
  }'
```

### Synthesis Focus
- Cross-category theme detection (5 categories)
- Primary themes (3+ engine agreement)
- Secondary themes (2 engine agreement)
- Alignments and tensions
- Comprehensive narrative

### Response Size
Full spectrum responses can be large. Consider:
- Using `Accept-Encoding: gzip`
- Requesting specific categories via options

---

## List Workflows

### Endpoint
```
GET /api/v1/workflows
```

### Response
```json
{
  "workflows": [
    {
      "id": "birth-blueprint",
      "name": "Birth Blueprint",
      "description": "Core identity mapping through birth data",
      "engines": ["numerology", "human-design", "gene-keys"],
      "ttl_seconds": 86400,
      "required_input": ["birth_data"]
    },
    {
      "id": "daily-practice",
      "name": "Daily Practice",
      "description": "Daily rhythm and awareness tools",
      "engines": ["panchanga", "vedic-clock", "biorhythm"],
      "ttl_seconds": 3600,
      "required_input": ["birth_data.date", "location"]
    }
  ]
}
```

---

## Get Workflow Definition

### Endpoint
```
GET /api/v1/workflows/{workflow_id}
```

### Response
```json
{
  "id": "birth-blueprint",
  "name": "Birth Blueprint",
  "description": "Core identity mapping through birth data",
  "engines": ["numerology", "human-design", "gene-keys"],
  "ttl_seconds": 86400,
  "required_input": ["birth_data"],
  "synthesis": {
    "type": "birth_blueprint",
    "themes": ["identity", "purpose", "gifts"],
    "output_format": "narrative + themes"
  }
}
```

---

## Custom Workflows (Future)

Custom workflow creation is planned for future releases:

```
POST /api/v1/workflows
```

```json
{
  "id": "my-custom-workflow",
  "name": "My Custom Workflow",
  "engines": ["numerology", "tarot"],
  "synthesis_type": "basic"
}
```

---

**Last Updated**: 2026-01
