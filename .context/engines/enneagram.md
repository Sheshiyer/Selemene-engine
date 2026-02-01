# Enneagram Engine Documentation

## Overview

The Enneagram engine identifies and analyzes personality types according to the nine-point Enneagram system, including wings, stress/growth directions, and instinctual variants.

## Purpose

Provide personality typing and growth path insights based on Enneagram wisdom, supporting self-understanding through the nine type framework.

## Calculation Type

**Assessment-based**: Derives type from questionnaire responses or stated type.

## Runtime

**TypeScript** (runs on port 3001, bridged via HTTP)

## Input Parameters

```json
{
  "options": {
    "type": 4,
    "wing": 5,
    "instinctual_variant": "sx",
    "tritype": [4, 7, 1],
    "consciousness_level": 3
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| options.type | number | Yes | Primary type (1-9) |
| options.wing | number | No | Wing type (adjacent to primary) |
| options.instinctual_variant | string | No | "sp", "sx", or "so" |
| options.tritype | array | No | Three-type combination |
| options.consciousness_level | number | No | For witness prompt adaptation |

## Output Structure

```json
{
  "engine_id": "enneagram",
  "success": true,
  "result": {
    "type": {
      "number": 4,
      "name": "The Individualist",
      "alias": "The Romantic",
      "center": "Heart",
      "basic_fear": "Being without identity or significance",
      "basic_desire": "To find themselves and their significance",
      "core_motivation": "To express their individuality",
      "fixation": "Melancholy",
      "virtue": "Equanimity",
      "passion": "Envy"
    },
    "wing": {
      "number": 5,
      "name": "The Investigator",
      "influence": "Adds intellectual depth and introspection",
      "combined_name": "4w5: The Bohemian"
    },
    "instinctual_variant": {
      "code": "sx",
      "name": "Sexual/One-to-One",
      "focus": "Intensity, attraction, fusion",
      "stacking": "sx/sp/so"
    },
    "growth_direction": {
      "integration": {
        "type": 1,
        "qualities": ["Principled", "Disciplined", "Purposeful"],
        "path": "Moving toward objectivity and action"
      },
      "stress": {
        "type": 2,
        "qualities": ["Needy", "Manipulative", "Possessive"],
        "path": "Under stress, may become clingy and demanding"
      }
    },
    "levels_of_development": {
      "healthy": ["Creative", "Inspired", "Self-renewing"],
      "average": ["Self-absorbed", "Moody", "Self-indulgent"],
      "unhealthy": ["Depressed", "Alienated", "Self-destructive"]
    },
    "relationship_with_types": {
      "complementary": [9, 7],
      "challenging": [8, 3],
      "similar": [5, 2]
    }
  },
  "witness_prompt": "As a 4w5, when melancholy arises, is it the authentic voice of your depths or the fixation obscuring equanimity?",
  "consciousness_level": 1
}
```

## The Nine Types

| Type | Name | Center | Core Fear | Core Desire |
|------|------|--------|-----------|-------------|
| 1 | The Reformer | Body | Being corrupt | Being good |
| 2 | The Helper | Heart | Being unwanted | Being loved |
| 3 | The Achiever | Heart | Being worthless | Being valuable |
| 4 | The Individualist | Heart | Without identity | Finding self |
| 5 | The Investigator | Head | Being useless | Being capable |
| 6 | The Loyalist | Head | Without support | Having security |
| 7 | The Enthusiast | Head | Being trapped | Being satisfied |
| 8 | The Challenger | Body | Being controlled | Self-protection |
| 9 | The Peacemaker | Body | Loss/separation | Inner peace |

## Three Centers

| Center | Types | Focus | Emotion |
|--------|-------|-------|---------|
| Body (Gut) | 8, 9, 1 | Action, instinct | Anger |
| Heart (Feeling) | 2, 3, 4 | Image, emotion | Shame |
| Head (Thinking) | 5, 6, 7 | Analysis, security | Fear |

## Wings

Each type can be influenced by adjacent types:
- Type 1: 1w9 (The Idealist) or 1w2 (The Advocate)
- Type 2: 2w1 (The Servant) or 2w3 (The Host)
- Type 3: 3w2 (The Charmer) or 3w4 (The Professional)
- Type 4: 4w3 (The Aristocrat) or 4w5 (The Bohemian)
- Type 5: 5w4 (The Iconoclast) or 5w6 (The Problem Solver)
- Type 6: 6w5 (The Defender) or 6w7 (The Buddy)
- Type 7: 7w6 (The Entertainer) or 7w8 (The Realist)
- Type 8: 8w7 (The Maverick) or 8w9 (The Bear)
- Type 9: 9w8 (The Referee) or 9w1 (The Dreamer)

## Instinctual Variants

| Variant | Code | Focus | Drive |
|---------|------|-------|-------|
| Self-Preservation | sp | Security, comfort | Survival needs |
| Sexual/One-to-One | sx | Intensity, connection | Merger, attraction |
| Social | so | Group, status | Belonging, contribution |

## Growth/Stress Directions

| Type | Integration (Health) | Disintegration (Stress) |
|------|---------------------|------------------------|
| 1 → 7 | Spontaneous | 1 → 4 Moody |
| 2 → 4 | Self-nurturing | 2 → 8 Aggressive |
| 3 → 6 | Committed | 3 → 9 Withdrawn |
| 4 → 1 | Principled | 4 → 2 Needy |
| 5 → 8 | Assertive | 5 → 7 Scattered |
| 6 → 9 | Relaxed | 6 → 3 Competitive |
| 7 → 5 | Focused | 7 → 1 Critical |
| 8 → 2 | Caring | 8 → 5 Withdrawn |
| 9 → 3 | Active | 9 → 6 Anxious |

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| Fixation | "As a {type}, when {fixation} arises, what lies beneath it?" | "As a 4, when melancholy arises, what lies beneath it?" |
| Growth | "Moving toward {integration_type}'s {quality}, what opens up?" | "Moving toward 1's principled action, what opens up?" |
| Core Fear | "When {core_fear} threatens, what remains true?" | "When 'being without identity' threatens, what remains true?" |
| Virtue | "What would {virtue} look like in this situation?" | "What would equanimity look like in this situation?" |

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| 9 types | ✅ Full | Complete descriptions |
| Wings | ✅ Full | 18 wing combinations |
| Instinctual variants | ✅ Full | sp/sx/so |
| Growth/Stress | ✅ Full | Directions mapped |
| Levels of development | ✅ Full | Healthy/average/unhealthy |
| Tritype | ✅ Full | 27 combinations |
| Wisdom data | ✅ Full | Comprehensive |
| Witness prompts | ✅ Full | Type-specific |

## API Endpoint

```
POST /api/v1/engines/enneagram/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/engines/enneagram/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {
      "type": 4,
      "wing": 5
    }
  }'
```

## Dependencies

- Enneagram type database
- Wing combination descriptions
- Growth/stress direction data

## References

- Riso-Hudson Enneagram Type Indicator (RHETI)
- Don Richard Riso & Russ Hudson's work
- Oscar Ichazo's original teachings

---

**Engine Version**: 0.1.0
**Required Phase**: 1
**Location**: `ts-engines/src/engines/enneagram/`
