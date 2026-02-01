# Face Reading Engine Documentation

## Overview

The Face Reading engine analyzes facial features according to Chinese physiognomy (Mian Xiang) and Western characterology, correlating facial structure with personality traits and life patterns.

## Purpose

Provide character and personality insights based on facial feature analysis, supporting self-understanding through physiognomic traditions.

## Calculation Type

**Feature-based**: Analyzes provided facial feature descriptions or measurements.

## Implementation Status

⚠️ **STUB IMPLEMENTATION**: This engine provides mock/placeholder data. Full implementation would require image analysis capabilities or detailed manual feature input.

## Input Parameters

```json
{
  "options": {
    "face_shape": "oval",
    "forehead": "high",
    "eyebrows": "thick_straight",
    "eyes": "large_round",
    "nose": "straight_medium",
    "mouth": "full_lips",
    "chin": "rounded",
    "ears": "medium_attached"
  }
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| options.face_shape | string | No | oval, round, square, oblong, heart, diamond |
| options.forehead | string | No | high, medium, low, wide, narrow |
| options.eyebrows | string | No | thick, thin, arched, straight |
| options.eyes | string | No | large, small, wide-set, close-set |
| options.nose | string | No | straight, aquiline, upturned, wide |
| options.mouth | string | No | full, thin, wide, small |
| options.chin | string | No | pointed, rounded, square, receding |
| options.ears | string | No | large, small, attached, detached |

## Output Structure

```json
{
  "engine_id": "face-reading",
  "success": true,
  "result": {
    "face_shape_analysis": {
      "shape": "Oval",
      "interpretation": "Balanced, diplomatic nature",
      "element": "Wood",
      "strengths": ["Adaptable", "Diplomatic", "Balanced perspective"],
      "challenges": ["May struggle with firm decisions"]
    },
    "feature_analysis": {
      "forehead": {
        "feature": "High",
        "interpretation": "Strong analytical mind",
        "age_period": "15-30 years"
      },
      "eyebrows": {
        "feature": "Thick, Straight",
        "interpretation": "Direct communication style",
        "trait": "Assertive"
      },
      "eyes": {
        "feature": "Large, Round",
        "interpretation": "Open, expressive nature",
        "trait": "Empathetic"
      },
      "nose": {
        "feature": "Straight, Medium",
        "interpretation": "Methodical approach to goals",
        "age_period": "40-50 years"
      },
      "mouth": {
        "feature": "Full Lips",
        "interpretation": "Generous, sensual nature",
        "trait": "Expressive"
      },
      "chin": {
        "feature": "Rounded",
        "interpretation": "Friendly, approachable",
        "trait": "Sociable"
      }
    },
    "three_zones": {
      "upper": {"forehead": "High", "interpretation": "Strong intellect"},
      "middle": {"nose": "Medium", "interpretation": "Steady career path"},
      "lower": {"chin": "Rounded", "interpretation": "Good later years"}
    },
    "five_elements": {
      "dominant": "Wood",
      "supporting": "Fire",
      "interpretation": "Growth-oriented with passionate expression"
    },
    "overall_reading": "A balanced individual with strong analytical abilities and empathetic nature..."
  },
  "witness_prompt": "Your high forehead suggests analytical strength—how does this mental capacity serve your life's purpose?",
  "consciousness_level": 2
}
```

## Face Regions (Three Zones)

| Zone | Area | Age Period | Governs |
|------|------|------------|---------|
| Upper | Forehead to eyebrows | 15-30 | Intellect, early life |
| Middle | Eyebrows to nose tip | 30-50 | Career, middle life |
| Lower | Nose tip to chin | 50+ | Will, later life |

## Face Shapes

| Shape | Element | Traits |
|-------|---------|--------|
| Oval | Wood | Balanced, diplomatic, adaptable |
| Round | Water | Sociable, intuitive, nurturing |
| Square | Earth | Practical, reliable, determined |
| Oblong | Wood/Metal | Ambitious, organized, methodical |
| Heart | Fire | Romantic, creative, passionate |
| Diamond | Fire/Metal | Intellectual, unique, independent |

## Feature Interpretations

### Forehead
| Type | Interpretation |
|------|----------------|
| High | Analytical, intellectual |
| Wide | Good judgment, leadership |
| Rounded | Creative thinking |
| Flat | Practical approach |

### Eyes
| Type | Interpretation |
|------|----------------|
| Large | Open, expressive |
| Small | Detail-oriented |
| Wide-set | Independent thinking |
| Close-set | Focused concentration |

### Nose
| Type | Interpretation |
|------|----------------|
| Straight | Methodical, principled |
| Aquiline | Leadership, ambition |
| Upturned | Optimistic, friendly |
| Wide | Generous, grounded |

## Witness Prompt Patterns

| Pattern | Template | Example |
|---------|----------|---------|
| Feature | "Your {feature} suggests {trait}—how does this {quality} express in your life?" | "Your high forehead suggests analytical strength—how does this mental capacity express in your life?" |
| Element | "With {element} dominant, how does {quality} manifest in your relationships?" | "With Wood dominant, how does growth-orientation manifest in your relationships?" |
| Zone | "The {zone} zone indicates {reading}—what does this reveal about your {life_period}?" | "The middle zone indicates steady progress—what does this reveal about your career?" |

## Implementation Notes

**Current Status**: Mock implementation returning pattern-based interpretations.

**Future Implementation Would Require**:
- Image analysis / computer vision integration
- Detailed facial landmark detection
- Validated physiognomy knowledge base
- Cultural sensitivity considerations

## API Endpoint

```
POST /api/v1/engines/face-reading/calculate
```

## Example Request

```bash
curl -X POST http://localhost:8080/api/v1/engines/face-reading/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {
      "face_shape": "oval",
      "forehead": "high",
      "eyes": "large_round"
    }
  }'
```

## Dependencies

- Physiognomy wisdom data
- (Future) Computer vision integration

## References

- Chinese physiognomy (Mian Xiang)
- Western characterology traditions

---

**Engine Version**: 0.1.0 (Stub)
**Required Phase**: 2
**Crate**: `crates/engine-face-reading/`
