# Sacred Geometry Engine Documentation

## Overview

The Sacred Geometry engine generates mathematical patterns and geometric forms considered to hold spiritual significance across traditions, including the Flower of Life, Metatron's Cube, and Platonic solids.

## Purpose

Create sacred geometric patterns for meditation, intention setting, and visual contemplation, with mathematical precision and symbolic meaning.

## Calculation Type

**Generative**: Mathematical pattern generation based on geometric principles.

## Runtime

**TypeScript** (runs on port 3001, bridged via HTTP)

## Input Parameters

```json
{
  "options": {
    "pattern_type": "flower_of_life",
    "iterations": 3,
    "color_scheme": "golden",
    "include_annotations": true,
    "output_format": "svg"
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| options.pattern_type | string | No | Pattern to generate (default: flower_of_life) |
| options.iterations | number | No | Complexity level (1-7) |
| options.color_scheme | string | No | Color palette |
| options.include_annotations | boolean | No | Include meaning labels |
| options.output_format | string | No | "svg", "json", or "description" |

## Available Patterns

| Pattern | Description | Symbolism |
|---------|-------------|-----------|
| seed_of_life | 7 overlapping circles | Creation, beginnings |
| flower_of_life | 19 overlapping circles | Unity, interconnection |
| fruit_of_life | 13 circles from Flower | Hidden knowledge |
| metatrons_cube | Lines connecting Fruit of Life | All Platonic solids |
| sri_yantra | 9 interlocking triangles | Cosmos, divine feminine |
| vesica_piscis | 2 overlapping circles | Creation, duality |
| golden_spiral | Fibonacci spiral | Natural growth |
| torus | Toroidal energy field | Energy flow |
| merkaba | Star tetrahedron | Light body |
| platonic_solids | 5 regular polyhedra | Elements |

## Output Structure

```json
{
  "engine_id": "sacred-geometry",
  "success": true,
  "result": {
    "pattern": {
      "type": "flower_of_life",
      "iterations": 3,
      "circle_count": 19,
      "center": {"x": 0, "y": 0},
      "radius": 100
    },
    "geometry": {
      "circles": [
        {"cx": 0, "cy": 0, "r": 100},
        {"cx": 100, "cy": 0, "r": 100},
        {"cx": 50, "cy": 86.6, "r": 100}
      ],
      "intersection_points": [
        {"x": 50, "y": 0},
        {"x": 25, "y": 43.3}
      ]
    },
    "symbolism": {
      "tradition": "Universal",
      "meaning": "The Flower of Life represents the interconnection of all life...",
      "meditation_focus": "Unity within diversity",
      "elements_present": ["Circle", "Vesica Piscis", "Hexagon"]
    },
    "mathematical_properties": {
      "golden_ratio_present": true,
      "phi_relationships": ["Circle spacing", "Petal proportions"],
      "symmetry": "6-fold rotational"
    },
    "svg": "<svg>...</svg>",
    "color_scheme": {
      "primary": "#FFD700",
      "secondary": "#4169E1",
      "background": "#1a1a2e"
    }
  },
  "witness_prompt": "Gazing at the Flower of Life's nineteen circles, where do you find yourself in this pattern of interconnection?",
  "consciousness_level": 2
}
```

## Platonic Solids

| Solid | Faces | Element | Symbolism |
|-------|-------|---------|-----------|
| Tetrahedron | 4 triangles | Fire | Transformation |
| Cube | 6 squares | Earth | Stability |
| Octahedron | 8 triangles | Air | Integration |
| Dodecahedron | 12 pentagons | Spirit/Ether | Cosmos |
| Icosahedron | 20 triangles | Water | Fluidity |

## Mathematical Foundations

### Golden Ratio (φ ≈ 1.618)
Present in:
- Flower of Life proportions
- Sri Yantra triangles
- Golden spiral construction
- Pentagon/Dodecahedron relationships

### Sacred Proportions
```
φ = (1 + √5) / 2 ≈ 1.618033988749895
1/φ = φ - 1 ≈ 0.618033988749895
φ² = φ + 1 ≈ 2.618033988749895
```

### Circle Relationships
- Vesica Piscis: Two circles, each passing through other's center
- Seed of Life: 7 circles in hexagonal arrangement
- Flower of Life: 19 circles with consistent overlap

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| Contemplation | "Gazing at {pattern}, what {theme} reveals itself?" | "Gazing at the Flower of Life, what interconnection reveals itself?" |
| Element | "The {solid} embodies {element}—where does {quality} appear in your life?" | "The Tetrahedron embodies Fire—where does transformation appear in your life?" |
| Proportion | "The golden ratio appears in {location}—what natural order do you sense?" | "The golden ratio appears in this spiral—what natural order do you sense?" |

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Seed of Life | ✅ Full | SVG generation |
| Flower of Life | ✅ Full | Variable iterations |
| Fruit of Life | ✅ Full | 13-circle pattern |
| Metatron's Cube | ✅ Full | Line connections |
| Sri Yantra | ✅ Full | Triangle construction |
| Vesica Piscis | ✅ Full | Two-circle base |
| Golden Spiral | ✅ Full | Fibonacci-based |
| Platonic Solids | ✅ Full | 2D projections |
| Merkaba | ✅ Full | Star tetrahedron |
| Torus | ⚠️ Partial | 2D representation only |
| Wisdom data | ✅ Full | All patterns |
| Witness prompts | ✅ Full | Pattern-specific |

## API Endpoint

```
POST /api/v1/engines/sacred-geometry/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/engines/sacred-geometry/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {
      "pattern_type": "flower_of_life",
      "iterations": 3,
      "output_format": "svg"
    }
  }'
```

## Dependencies

- SVG generation library
- Mathematical computation for proportions
- Pattern wisdom data

## References

- Drunvalo Melchizedek's sacred geometry work
- Mathematical traditions across cultures
- Golden ratio in nature and art

---

**Engine Version**: 0.1.0
**Required Phase**: 2
**Location**: `ts-engines/src/engines/sacred-geometry/`
