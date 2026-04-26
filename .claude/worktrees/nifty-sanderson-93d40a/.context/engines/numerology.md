# Numerology Engine Documentation

## Overview

Western numerology calculates core numbers from birth date and name, revealing personality patterns, life path, and soul urges through number symbolism.

## Purpose

Generate a numerological profile including Life Path, Expression, Soul Urge, Personality, and Birthday numbers, providing archetypal insight into character and destiny.

## Calculation Type

**Natal**: Derived from birth date and optionally birth name.

## Input Parameters

```json
{
  "birth_data": {
    "date": "1990-03-15",
    "name": "John Michael Smith"
  },
  "precision": "Standard",
  "options": {
    "include_personal_year": true,
    "include_pinnacles": true
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| birth_data.date | string | Yes | ISO 8601 date |
| birth_data.name | string | No | Full birth name for Expression/Soul Urge |
| options.include_personal_year | boolean | No | Current year calculation |
| options.include_pinnacles | boolean | No | Life cycle pinnacles |

## Output Structure

```json
{
  "engine_id": "numerology",
  "success": true,
  "result": {
    "life_path": {
      "number": 9,
      "meaning": "Humanitarian",
      "description": "The compassionate leader..."
    },
    "expression": {
      "number": 7,
      "meaning": "Seeker",
      "description": "The analytical thinker..."
    },
    "soul_urge": {
      "number": 3,
      "meaning": "Creator",
      "description": "The expressive spirit..."
    },
    "personality": {
      "number": 4,
      "meaning": "Builder",
      "description": "The practical presence..."
    },
    "birthday": {
      "number": 15,
      "reduced": 6,
      "meaning": "Nurturer",
      "description": "The responsible caretaker..."
    },
    "personal_year": {
      "number": 5,
      "meaning": "Change",
      "description": "A year of transformation..."
    },
    "master_numbers": [11, 22, 33],
    "karmic_numbers": [13, 14, 16, 19]
  },
  "witness_prompt": "With Life Path 9 calling toward service, how does your inner Seeker (7) inform what wisdom you share?",
  "consciousness_level": 0
}
```

## Core Numbers

### Life Path (Most Important)
**Calculation**: Sum all digits of birth date until single digit (or master number).

```
Birth Date: March 15, 1990
3 + 15 + 1990 = 3 + 1 + 5 + 1 + 9 + 9 + 0 = 28 → 2 + 8 = 10 → 1 + 0 = 1
Life Path: 1
```

| Number | Archetype | Keywords |
|--------|-----------|----------|
| 1 | Pioneer | Leadership, independence, innovation |
| 2 | Diplomat | Partnership, balance, sensitivity |
| 3 | Creator | Expression, joy, communication |
| 4 | Builder | Foundation, discipline, stability |
| 5 | Explorer | Freedom, change, adventure |
| 6 | Nurturer | Responsibility, harmony, service |
| 7 | Seeker | Wisdom, introspection, analysis |
| 8 | Achiever | Power, abundance, mastery |
| 9 | Humanitarian | Compassion, completion, wisdom |

### Master Numbers
Numbers 11, 22, 33 are not reduced:
- **11**: Master Intuitive (heightened 2)
- **22**: Master Builder (heightened 4)
- **33**: Master Teacher (heightened 6)

### Expression Number
**Calculation**: Convert full birth name to numbers using Pythagorean system, sum and reduce.

```
A=1, B=2, C=3, D=4, E=5, F=6, G=7, H=8, I=9
J=1, K=2, L=3, M=4, N=5, O=6, P=7, Q=8, R=9
S=1, T=2, U=3, V=4, W=5, X=6, Y=7, Z=8
```

### Soul Urge Number
**Calculation**: Sum only vowels (A, E, I, O, U) in birth name.

### Personality Number
**Calculation**: Sum only consonants in birth name.

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| Life Path | "As a {meaning} (Life Path {number}), what {theme} calls to you?" | "As a Humanitarian (Life Path 9), what service calls to you?" |
| Integration | "How does your {num1} {meaning1} express through your {num2} {meaning2}?" | "How does your 9 Humanitarian express through your 7 Seeker?" |
| Personal Year | "In this {meaning} year ({number}), what {theme} emerges?" | "In this Change year (5), what must transform?" |

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Life Path | ✅ Full | Master number support |
| Expression | ✅ Full | Full name analysis |
| Soul Urge | ✅ Full | Vowel extraction |
| Personality | ✅ Full | Consonant extraction |
| Birthday | ✅ Full | Day number + reduction |
| Personal Year | ✅ Full | Current year calculation |
| Pinnacles | ⚠️ Stub | Structure only |
| Wisdom data | ✅ Full | All numbers 1-9, 11, 22, 33 |
| Witness prompts | ✅ Full | Number-specific |

## API Endpoint

```
POST /api/v1/engines/numerology/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/engines/numerology/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1990-03-15",
      "name": "John Michael Smith"
    }
  }'
```

## Dependencies

- None (pure mathematical calculations)
- Numerology wisdom data

## References

- Pythagorean numerology system
- Matthew Oliver Goodwin's work

---

**Engine Version**: 0.1.0
**Required Phase**: 0
**Crate**: `crates/engine-numerology/`
