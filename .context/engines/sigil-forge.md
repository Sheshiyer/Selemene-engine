# Sigil Forge Engine Documentation

## Overview

The Sigil Forge engine creates magical sigils by encoding intentions into visual symbols using traditional occult methods like the Rose Cross and chaos magic techniques.

## Purpose

Transform written intentions into abstract visual symbols (sigils) for magical practice, meditation, or personal symbolism.

## Calculation Type

**Generative**: Transforms text intentions into visual sigil designs.

## Runtime

**TypeScript** (runs on port 3001, bridged via HTTP)

## Input Parameters

```json
{
  "options": {
    "intention": "I am confident and creative",
    "method": "rose_cross",
    "style": "angular",
    "include_breakdown": true,
    "output_format": "svg"
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| options.intention | string | Yes | Intention statement to encode |
| options.method | string | No | Encoding method (default: rose_cross) |
| options.style | string | No | Visual style (angular, curved, mixed) |
| options.include_breakdown | boolean | No | Show encoding steps |
| options.output_format | string | No | "svg", "json", or "description" |

## Encoding Methods

### Rose Cross Method
Traditional Golden Dawn technique using the Rose Cross (22-letter Hebrew wheel):

1. Remove duplicate letters from intention
2. Remove vowels (optional, chaos magic variant keeps them)
3. Connect letters on Rose Cross diagram
4. Simplify resulting path into sigil

### Chaos Magic Method
Austin Osman Spare's technique:

1. Write intention as statement of will
2. Remove duplicate letters
3. Combine remaining letters into monogram
4. Abstract into pleasing design

### Planetary Squares
Encode numbers onto magic squares:

| Planet | Square Size | Total |
|--------|-------------|-------|
| Saturn | 3×3 | 15 per row |
| Jupiter | 4×4 | 34 per row |
| Mars | 5×5 | 65 per row |
| Sun | 6×6 | 111 per row |
| Venus | 7×7 | 175 per row |
| Mercury | 8×8 | 260 per row |
| Moon | 9×9 | 369 per row |

## Output Structure

```json
{
  "engine_id": "sigil-forge",
  "success": true,
  "result": {
    "sigil": {
      "method": "rose_cross",
      "style": "angular",
      "intention": "I am confident and creative",
      "encoded": "MCNFDTV"
    },
    "breakdown": {
      "original": "I AM CONFIDENT AND CREATIVE",
      "vowels_removed": "M CNFDNT ND CRTV",
      "duplicates_removed": "MCNFDTV",
      "letter_positions": [
        {"letter": "M", "x": 120, "y": 80},
        {"letter": "C", "x": 180, "y": 95},
        {"letter": "N", "x": 160, "y": 140}
      ],
      "path_points": [
        {"x": 120, "y": 80},
        {"x": 180, "y": 95},
        {"x": 160, "y": 140}
      ]
    },
    "geometry": {
      "bounding_box": {"width": 200, "height": 200},
      "center": {"x": 100, "y": 100},
      "line_count": 6,
      "point_count": 7
    },
    "svg": "<svg viewBox='0 0 200 200'>...</svg>",
    "activation_suggestions": [
      "Meditate on the sigil while holding intention",
      "Draw the sigil in a ritual context",
      "Visualize the sigil before sleep"
    ],
    "symbolism": {
      "dominant_angles": "Sharp angles suggest decisive action",
      "flow_direction": "Upward movement indicates aspiration",
      "enclosed_spaces": "Circles represent completion and protection"
    }
  },
  "witness_prompt": "Gazing at this sigil encoded from 'confident and creative,' what does this abstract form evoke before the mind names it?",
  "consciousness_level": 2
}
```

## Sigil Creation Process

### Step 1: Intention Formation
Write clear, present-tense, positive statement:
- ✅ "I am confident and creative"
- ❌ "I want to be confident" (future tense)
- ❌ "I am not anxious" (negative framing)

### Step 2: Letter Extraction
```
Original:    I AM CONFIDENT AND CREATIVE
No Vowels:   M CNFDNT ND CRTV
No Dupes:    M C N F D T R V
Final:       MCNFDTRV
```

### Step 3: Path Generation
Connect letters on chosen diagram (Rose Cross, planetary square, etc.)

### Step 4: Stylization
- Angular: Sharp turns, straight lines
- Curved: Flowing, organic curves
- Mixed: Combination of both

### Step 5: Simplification
Abstract the path into pleasing, unified symbol.

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| Pre-cognitive | "Before naming it, what does this form evoke?" | "Gazing at this sigil, what does this abstract form evoke before the mind names it?" |
| Intention Echo | "This sigil encodes '{intention}'—where in your body does this intention resonate?" | "This sigil encodes 'confident'—where in your body does this intention resonate?" |
| Activation | "How might you bring this sigil into daily awareness?" | - |

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Rose Cross method | ✅ Full | 22-letter Hebrew wheel |
| Chaos magic method | ✅ Full | Letter combination |
| Letter extraction | ✅ Full | Vowel/duplicate removal |
| Path generation | ✅ Full | Point-to-point paths |
| SVG output | ✅ Full | Scalable vector output |
| Style variants | ✅ Full | Angular/curved/mixed |
| Planetary squares | ⚠️ Partial | Saturn, Jupiter, Mars only |
| Wisdom data | ✅ Full | Activation suggestions |
| Witness prompts | ✅ Full | Intention-aware |

## API Endpoint

```
POST /api/v1/engines/sigil-forge/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/engines/sigil-forge/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {
      "intention": "I manifest abundance easily",
      "method": "rose_cross",
      "output_format": "svg"
    }
  }'
```

## Dependencies

- SVG generation library
- Rose Cross diagram data
- Planetary square definitions

## References

- Austin Osman Spare's sigil magic
- Golden Dawn Rose Cross system
- Chaos magic traditions

---

**Engine Version**: 0.1.0
**Required Phase**: 2
**Location**: `ts-engines/src/engines/sigil-forge/`
