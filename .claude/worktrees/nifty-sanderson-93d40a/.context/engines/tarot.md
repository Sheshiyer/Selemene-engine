# Tarot Engine Documentation

## Overview

The Tarot engine performs card draws and spreads from the 78-card Rider-Waite-Smith tradition, providing archetypal interpretation and symbolic guidance.

## Purpose

Generate divinatory readings using tarot card draws, offering symbolic mirrors for self-reflection and decision support through archetypal imagery.

## Calculation Type

**Oracular**: Random card selection with position-based interpretation.

## Runtime

**TypeScript** (runs on port 3001, bridged via HTTP)

## Input Parameters

```json
{
  "options": {
    "spread_type": "celtic_cross",
    "question": "What do I need to know about my career path?",
    "reversed_enabled": true,
    "deck": "rider_waite"
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| options.spread_type | string | No | Spread layout (default: three_card) |
| options.question | string | No | Question for the reading |
| options.reversed_enabled | boolean | No | Allow reversed cards (default: true) |
| options.deck | string | No | Deck variant (default: rider_waite) |

## Available Spreads

| Spread | Cards | Positions | Use Case |
|--------|-------|-----------|----------|
| single | 1 | Card | Quick insight |
| three_card | 3 | Past, Present, Future | General reading |
| celtic_cross | 10 | Full spread | Comprehensive analysis |
| horseshoe | 7 | Past to outcome | Situation overview |
| relationship | 6 | Two perspectives | Partnership questions |

## Output Structure

```json
{
  "engine_id": "tarot",
  "success": true,
  "result": {
    "spread_type": "three_card",
    "question": "What do I need to know about my career path?",
    "cards": [
      {
        "position": "Past",
        "card": {
          "name": "The Hermit",
          "arcana": "Major",
          "number": 9,
          "suit": null,
          "reversed": false,
          "keywords": ["Introspection", "Solitude", "Inner guidance"],
          "meaning_upright": "A period of withdrawal and inner reflection...",
          "meaning_reversed": "Isolation, loneliness, withdrawal..."
        },
        "interpretation": "Your past career path involved significant introspection..."
      },
      {
        "position": "Present",
        "card": {
          "name": "The Wheel of Fortune",
          "arcana": "Major",
          "number": 10,
          "suit": null,
          "reversed": false,
          "keywords": ["Cycles", "Fate", "Turning point"]
        },
        "interpretation": "You are currently at a turning point..."
      },
      {
        "position": "Future",
        "card": {
          "name": "Three of Wands",
          "arcana": "Minor",
          "number": 3,
          "suit": "Wands",
          "reversed": false,
          "keywords": ["Expansion", "Foresight", "Opportunity"]
        },
        "interpretation": "Expansion and new opportunities await..."
      }
    ],
    "synthesis": "The reading suggests a journey from introspection through change toward expansion...",
    "advice": "Trust the turning point you're experiencing as preparation for growth."
  },
  "witness_prompt": "The Hermit's past solitude meets the Wheel's present change—what wisdom from your inner journey prepares you for this turning point?",
  "consciousness_level": 1
}
```

## Card Structure

### Major Arcana (22 cards)
The Fool (0) through The World (21) - archetypal life journey.

| Card | Number | Theme |
|------|--------|-------|
| The Fool | 0 | New beginnings |
| The Magician | 1 | Manifestation |
| The High Priestess | 2 | Intuition |
| The Empress | 3 | Abundance |
| The Emperor | 4 | Authority |
| The Hierophant | 5 | Tradition |
| The Lovers | 6 | Choice |
| The Chariot | 7 | Willpower |
| Strength | 8 | Inner strength |
| The Hermit | 9 | Introspection |
| Wheel of Fortune | 10 | Cycles |
| Justice | 11 | Balance |
| The Hanged Man | 12 | Surrender |
| Death | 13 | Transformation |
| Temperance | 14 | Moderation |
| The Devil | 15 | Shadow |
| The Tower | 16 | Upheaval |
| The Star | 17 | Hope |
| The Moon | 18 | Illusion |
| The Sun | 19 | Joy |
| Judgement | 20 | Renewal |
| The World | 21 | Completion |

### Minor Arcana (56 cards)
Four suits of 14 cards each (Ace through 10 + Page, Knight, Queen, King).

| Suit | Element | Domain |
|------|---------|--------|
| Wands | Fire | Creativity, action |
| Cups | Water | Emotions, relationships |
| Swords | Air | Thoughts, conflict |
| Pentacles | Earth | Material, practical |

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| Single Card | "What does {card}'s presence invite you to notice about {theme}?" | "What does The Hermit's presence invite you to notice about your inner world?" |
| Position Pair | "{card1} in {position1} meets {card2} in {position2}—what connection emerges?" | "The Hermit in Past meets Wheel of Fortune in Present—what connection emerges?" |
| Synthesis | "Across this reading, {theme} appears—how does this pattern relate to your question?" | "Across this reading, transformation appears—how does this pattern relate to your question?" |

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Card database | ✅ Full | 78 cards with meanings |
| Single card draw | ✅ Full | Random with optional reversals |
| Three card spread | ✅ Full | Past/Present/Future |
| Celtic Cross | ✅ Full | 10 positions |
| Horseshoe | ✅ Full | 7 positions |
| Relationship | ✅ Full | 6 positions |
| Card wisdom | ✅ Full | Upright/reversed meanings |
| Synthesis | ✅ Full | Cross-card theme detection |
| Witness prompts | ✅ Full | Spread-appropriate |

## API Endpoint

```
POST /api/v1/engines/tarot/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/engines/tarot/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {
      "spread_type": "three_card",
      "question": "What should I focus on this month?",
      "reversed_enabled": true
    }
  }'
```

## Dependencies

- Tarot card database (78 cards)
- Spread position definitions
- Interpretation wisdom data

## References

- Rider-Waite-Smith tarot tradition
- A.E. Waite's "Pictorial Key to the Tarot"

---

**Engine Version**: 0.1.0
**Required Phase**: 1
**Location**: `ts-engines/src/engines/tarot/`
