# I-Ching Engine Documentation

## Overview

The I-Ching (Book of Changes) engine generates hexagram readings from the ancient Chinese oracle, providing wisdom through the 64 hexagrams and their transformations via changing lines.

## Purpose

Provide oracular guidance through hexagram generation and interpretation, offering philosophical frameworks for understanding change and making decisions.

## Calculation Type

**Oracular**: Random or yarrow stalk simulation for hexagram generation.

## Runtime

**TypeScript** (runs on port 3001, bridged via HTTP)

## Input Parameters

```json
{
  "options": {
    "method": "coin",
    "question": "How should I approach this decision?",
    "include_changing_lines": true
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| options.method | string | No | "coin" or "yarrow" (default: coin) |
| options.question | string | No | Question for contemplation |
| options.include_changing_lines | boolean | No | Show line transformations (default: true) |

## Output Structure

```json
{
  "engine_id": "i-ching",
  "success": true,
  "result": {
    "hexagram": {
      "number": 4,
      "name": "Meng",
      "english_name": "Youthful Folly",
      "chinese": "蒙",
      "judgment": "Youthful Folly has success. It is not I who seek the young fool...",
      "image": "A spring wells up at the foot of the mountain...",
      "trigram_upper": {
        "name": "Gen",
        "english": "Mountain",
        "attribute": "Keeping Still"
      },
      "trigram_lower": {
        "name": "Kan",
        "english": "Water",
        "attribute": "The Abysmal"
      },
      "lines": [
        {"position": 1, "value": 7, "type": "yang", "changing": false},
        {"position": 2, "value": 8, "type": "yin", "changing": false},
        {"position": 3, "value": 6, "type": "yin", "changing": true, "text": "Take not a maiden..."},
        {"position": 4, "value": 8, "type": "yin", "changing": false},
        {"position": 5, "value": 8, "type": "yin", "changing": false},
        {"position": 6, "value": 9, "type": "yang", "changing": true, "text": "In punishing folly..."}
      ]
    },
    "changing_hexagram": {
      "number": 18,
      "name": "Gu",
      "english_name": "Work on What Has Been Spoiled",
      "judgment": "Work on what has been spoiled has supreme success..."
    },
    "interpretation": "The reading suggests approaching this decision with beginner's mind...",
    "advice": "Seek guidance with humility; the answer will come when you stop forcing it."
  },
  "witness_prompt": "Hexagram 4 'Youthful Folly' asks: where in this situation are you the student seeking wisdom, and where the teacher waiting to be approached?",
  "consciousness_level": 1
}
```

## The 64 Hexagrams

Organized by traditional King Wen sequence:

| # | Name | Chinese | Theme |
|---|------|---------|-------|
| 1 | The Creative | 乾 | Pure yang, initiating |
| 2 | The Receptive | 坤 | Pure yin, responsive |
| 3 | Difficulty at the Beginning | 屯 | Birth pains |
| 4 | Youthful Folly | 蒙 | Inexperience, teaching |
| ... | ... | ... | ... |
| 11 | Peace | 泰 | Harmony, prosperity |
| 12 | Standstill | 否 | Stagnation |
| ... | ... | ... | ... |
| 63 | After Completion | 既濟 | Transition, vigilance |
| 64 | Before Completion | 未濟 | Incompleteness, potential |

## Eight Trigrams

| Trigram | Chinese | Element | Attribute | Family |
|---------|---------|---------|-----------|--------|
| Qian | 乾 | Heaven | Creative | Father |
| Kun | 坤 | Earth | Receptive | Mother |
| Zhen | 震 | Thunder | Arousing | First Son |
| Kan | 坎 | Water | Abysmal | Second Son |
| Gen | 艮 | Mountain | Keeping Still | Third Son |
| Xun | 巽 | Wind | Gentle | First Daughter |
| Li | 離 | Fire | Clinging | Second Daughter |
| Dui | 兌 | Lake | Joyous | Third Daughter |

## Line Values

| Value | Type | Stability | Changes To |
|-------|------|-----------|------------|
| 6 | Old Yin | Changing | Yang (7) |
| 7 | Young Yang | Stable | - |
| 8 | Young Yin | Stable | - |
| 9 | Old Yang | Changing | Yin (8) |

## Generation Methods

### Coin Method
Three coins tossed six times:
- Heads = 3, Tails = 2
- Total 6 = Old Yin (changing)
- Total 7 = Young Yang
- Total 8 = Young Yin
- Total 9 = Old Yang (changing)

### Yarrow Stalk Method
Traditional 49-stalk division process, more weighted toward changing lines.

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| Hexagram | "Hexagram {number} '{name}' asks: {inquiry}" | "Hexagram 4 'Youthful Folly' asks: where do you need to become a student?" |
| Changing Line | "Line {position} changing: '{text}'—what in your situation resonates?" | "Line 3 changing: 'Take not a maiden'—what attachment clouds your judgment?" |
| Transformation | "From '{hex1}' to '{hex2}': how does {theme1} transform into {theme2}?" | "From 'Folly' to 'Work': how does inexperience transform into correction?" |

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| 64 hexagrams | ✅ Full | Complete Wilhelm/Baynes translation basis |
| 8 trigrams | ✅ Full | All attributes |
| Coin method | ✅ Full | Random generation |
| Yarrow method | ✅ Full | Traditional probability |
| Changing lines | ✅ Full | Line texts + transformation |
| Changing hexagram | ✅ Full | Automatic derivation |
| Wisdom data | ✅ Full | Judgment, Image, Lines |
| Witness prompts | ✅ Full | Hexagram-specific |

## API Endpoint

```
POST /api/v1/engines/i-ching/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/engines/i-ching/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {
      "method": "coin",
      "question": "What is the best approach to this challenge?"
    }
  }'
```

## Dependencies

- Hexagram database (64 hexagrams)
- Line interpretation data
- Trigram relationship data

## References

- Wilhelm/Baynes translation of the I-Ching
- Traditional Confucian commentaries

---

**Engine Version**: 0.1.0
**Required Phase**: 1
**Location**: `ts-engines/src/engines/i-ching/`
