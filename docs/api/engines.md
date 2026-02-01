# Engine API Endpoints

## Overview

Engine endpoints provide direct access to individual consciousness calculation systems. Each engine accepts specific input parameters and returns structured calculation results with witness prompts.

## Base Path

```
/api/v1/engines/{engine_id}/calculate
```

## Available Engines

### Rust Engines (Native)

| Engine ID | Name | Required Phase |
|-----------|------|----------------|
| human-design | Human Design | 1 |
| gene-keys | Gene Keys | 1 |
| vimshottari | Vimshottari Dasha | 1 |
| panchanga | Panchanga | 0 |
| numerology | Numerology | 0 |
| biorhythm | Biorhythm | 0 |
| vedic-clock | Vedic Clock | 0 |
| biofield | Biofield | 2 |
| face-reading | Face Reading | 2 |

### TypeScript Engines (Bridged)

| Engine ID | Name | Required Phase |
|-----------|------|----------------|
| tarot | Tarot | 1 |
| i-ching | I-Ching | 1 |
| enneagram | Enneagram | 1 |
| sacred-geometry | Sacred Geometry | 2 |
| sigil-forge | Sigil Forge | 2 |

---

## Human Design Engine

### Endpoint
```
POST /api/v1/engines/human-design/calculate
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
  "precision": "Standard"
}
```

### Response
```json
{
  "engine_id": "human-design",
  "success": true,
  "result": {
    "hd_type": "Generator",
    "authority": "Sacral",
    "profile": "1/3",
    "definition": "Single",
    "defined_centers": ["Root", "Sacral", "Solar Plexus", "G-Center", "Throat"],
    "active_channels": ["3-60", "9-52"],
    "personality_activations": {
      "sun": {"gate": 35, "line": 1},
      "earth": {"gate": 5, "line": 1}
    },
    "design_activations": {
      "sun": {"gate": 20, "line": 3},
      "earth": {"gate": 34, "line": 3}
    }
  },
  "witness_prompt": "What does it feel like when you wait to respond rather than initiating?",
  "consciousness_level": 1,
  "metadata": {
    "calculation_time_ms": 45.2,
    "backend": "native",
    "cached": false
  }
}
```

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/engines/human-design/calculate \
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

---

## Gene Keys Engine

### Endpoint
```
POST /api/v1/engines/gene-keys/calculate
```

### Request (Mode 1: Birth Data)
```json
{
  "birth_data": {
    "date": "1990-03-15",
    "time": "14:30",
    "latitude": 40.7128,
    "longitude": -74.0060,
    "timezone": "America/New_York"
  },
  "options": {
    "consciousness_level": 3
  }
}
```

### Request (Mode 2: HD Gates)
```json
{
  "options": {
    "hd_gates": {
      "personality_sun": 17,
      "personality_earth": 18,
      "design_sun": 45,
      "design_earth": 26
    },
    "consciousness_level": 3
  }
}
```

### Response
```json
{
  "engine_id": "gene-keys",
  "result": {
    "activation_sequence": {
      "lifes_work": [17, 18],
      "evolution": [45, 26],
      "radiance": [17, 45],
      "purpose": [18, 26]
    },
    "active_keys": [
      {
        "key_number": 17,
        "shadow": "Opinion",
        "gift": "Far-Sightedness",
        "siddhi": "Omniscience"
      }
    ],
    "frequency_assessments": [...]
  },
  "witness_prompt": "How do Far-Sightedness (17) and Synergy (45) create your core magnetism?"
}
```

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/engines/gene-keys/calculate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {
      "hd_gates": {
        "personality_sun": 17,
        "personality_earth": 18,
        "design_sun": 45,
        "design_earth": 26
      }
    }
  }'
```

---

## Numerology Engine

### Endpoint
```
POST /api/v1/engines/numerology/calculate
```

### Request
```json
{
  "birth_data": {
    "date": "1990-03-15",
    "name": "John Michael Smith"
  }
}
```

### Response
```json
{
  "engine_id": "numerology",
  "result": {
    "life_path": {"number": 9, "meaning": "Humanitarian"},
    "expression": {"number": 7, "meaning": "Seeker"},
    "soul_urge": {"number": 3, "meaning": "Creator"},
    "personality": {"number": 4, "meaning": "Builder"},
    "personal_year": {"number": 5, "meaning": "Change"}
  },
  "witness_prompt": "With Life Path 9 calling toward service, how does your Seeker nature inform what wisdom you share?"
}
```

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/engines/numerology/calculate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1990-03-15",
      "name": "John Michael Smith"
    }
  }'
```

---

## Biorhythm Engine

### Endpoint
```
POST /api/v1/engines/biorhythm/calculate
```

### Request
```json
{
  "birth_data": {
    "date": "1990-03-15"
  },
  "current_time": "2025-01-15T12:00:00Z",
  "options": {
    "forecast_days": 7
  }
}
```

### Response
```json
{
  "engine_id": "biorhythm",
  "result": {
    "days_since_birth": 12725,
    "current_values": {
      "physical": {"value": 0.73, "phase": "High"},
      "emotional": {"value": -0.15, "phase": "Descending"},
      "intellectual": {"value": 0.45, "phase": "Ascending"}
    },
    "critical_days": {
      "upcoming": ["2025-01-20"]
    }
  },
  "witness_prompt": "With physical energy high and emotions descending, how might body wisdom guide decisions?"
}
```

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/engines/biorhythm/calculate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {"date": "1990-03-15"},
    "options": {"forecast_days": 7}
  }'
```

---

## Panchanga Endpoints

### Calculate Panchanga
```
POST /api/v1/panchanga/calculate
```

### Request
```json
{
  "current_time": "2025-01-15T12:00:00Z",
  "location": {
    "latitude": 28.6139,
    "longitude": 77.2090
  }
}
```

### Response
```json
{
  "result": {
    "tithi": {"name": "Shukla Saptami", "paksha": "Shukla"},
    "nakshatra": {"name": "Pushya", "lord": "Saturn"},
    "yoga": {"name": "Shiva", "nature": "Auspicious"},
    "karana": {"name": "Bava"},
    "vara": {"name": "Wednesday", "lord": "Mercury"}
  },
  "witness_prompt": "On this Shukla Saptami in Pushya, what seeds planted now bear fruit?"
}
```

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/panchanga/calculate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "current_time": "2025-01-15T12:00:00Z",
    "location": {"latitude": 28.6139, "longitude": 77.2090}
  }'
```

### Batch Panchanga
```
POST /api/v1/panchanga/batch
```

Calculate for multiple dates.

---

## Ghati Time Endpoints

### Calculate Ghati Time
```
POST /api/v1/ghati/calculate
```

### Request
```json
{
  "current_time": "2025-01-15T12:00:00Z",
  "location": {
    "latitude": 28.6139,
    "longitude": 77.2090
  }
}
```

### Response
```json
{
  "result": {
    "ghati": 24,
    "pala": 30,
    "vipala": 15,
    "display": "24:30:15",
    "day_percentage": 40.8
  }
}
```

### Other Ghati Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/ghati/current` | POST | Current ghati time |
| `/ghati/boundaries` | POST | Ghati day boundaries |
| `/ghati/utc-to-ghati` | POST | Convert UTC to ghati |
| `/ghati/ghati-to-utc` | POST | Convert ghati to UTC |
| `/ghati/methods` | GET | Available calculation methods |

---

## Tarot Engine

### Endpoint
```
POST /api/v1/engines/tarot/calculate
```

### Request
```json
{
  "options": {
    "spread_type": "three_card",
    "question": "What should I focus on this month?",
    "reversed_enabled": true
  }
}
```

### Response
```json
{
  "engine_id": "tarot",
  "result": {
    "spread_type": "three_card",
    "cards": [
      {
        "position": "Past",
        "card": {"name": "The Hermit", "arcana": "Major", "reversed": false}
      },
      {
        "position": "Present",
        "card": {"name": "Wheel of Fortune", "arcana": "Major", "reversed": false}
      },
      {
        "position": "Future",
        "card": {"name": "Three of Wands", "arcana": "Minor", "reversed": false}
      }
    ],
    "synthesis": "A journey from introspection through change toward expansion..."
  },
  "witness_prompt": "The Hermit's solitude meets the Wheel's change—what wisdom prepares you for this turning point?"
}
```

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/engines/tarot/calculate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {
      "spread_type": "celtic_cross",
      "question": "Career guidance"
    }
  }'
```

---

## I-Ching Engine

### Endpoint
```
POST /api/v1/engines/i-ching/calculate
```

### Request
```json
{
  "options": {
    "method": "coin",
    "question": "How should I approach this decision?"
  }
}
```

### Response
```json
{
  "engine_id": "i-ching",
  "result": {
    "hexagram": {
      "number": 4,
      "name": "Meng",
      "english_name": "Youthful Folly",
      "judgment": "Youthful Folly has success..."
    },
    "changing_hexagram": {
      "number": 18,
      "name": "Gu",
      "english_name": "Work on What Has Been Spoiled"
    },
    "lines": [
      {"position": 3, "changing": true, "text": "Take not a maiden..."}
    ]
  },
  "witness_prompt": "Hexagram 4 asks: where are you the student seeking wisdom?"
}
```

---

## Enneagram Engine

### Endpoint
```
POST /api/v1/engines/enneagram/calculate
```

### Request
```json
{
  "options": {
    "type": 4,
    "wing": 5,
    "instinctual_variant": "sx"
  }
}
```

### Response
```json
{
  "engine_id": "enneagram",
  "result": {
    "type": {
      "number": 4,
      "name": "The Individualist",
      "basic_fear": "Being without identity",
      "fixation": "Melancholy",
      "virtue": "Equanimity"
    },
    "wing": {"number": 5, "combined_name": "4w5: The Bohemian"},
    "growth_direction": {"integration": 1, "stress": 2}
  },
  "witness_prompt": "When melancholy arises, is it your depths speaking or the fixation obscuring equanimity?"
}
```

---

## Sacred Geometry Engine

### Endpoint
```
POST /api/v1/engines/sacred-geometry/calculate
```

### Request
```json
{
  "options": {
    "pattern_type": "flower_of_life",
    "iterations": 3,
    "output_format": "svg"
  }
}
```

### Response
```json
{
  "engine_id": "sacred-geometry",
  "result": {
    "pattern": {"type": "flower_of_life", "circle_count": 19},
    "svg": "<svg>...</svg>",
    "symbolism": {
      "meaning": "Interconnection of all life",
      "meditation_focus": "Unity within diversity"
    }
  },
  "witness_prompt": "Where do you find yourself in this pattern of interconnection?"
}
```

---

## Sigil Forge Engine

### Endpoint
```
POST /api/v1/engines/sigil-forge/calculate
```

### Request
```json
{
  "options": {
    "intention": "I am confident and creative",
    "method": "rose_cross",
    "output_format": "svg"
  }
}
```

### Response
```json
{
  "engine_id": "sigil-forge",
  "result": {
    "sigil": {"method": "rose_cross", "encoded": "MCNFDTV"},
    "svg": "<svg>...</svg>",
    "activation_suggestions": ["Meditate on the sigil while holding intention"]
  },
  "witness_prompt": "What does this abstract form evoke before the mind names it?"
}
```

---

## List All Engines

### Endpoint
```
GET /api/v1/engines
```

### Response
```json
{
  "engines": [
    {"id": "human-design", "name": "Human Design", "required_phase": 1, "runtime": "rust"},
    {"id": "gene-keys", "name": "Gene Keys", "required_phase": 1, "runtime": "rust"},
    {"id": "tarot", "name": "Tarot", "required_phase": 1, "runtime": "typescript"}
  ]
}
```

---

**Last Updated**: 2026-01
