---
mode: composite-dyad
subject_count:
  min: 2
  max: 2
roles:
  - subject-a
  - subject-b
target_words:
  min: 9000
  max: 11000
architecture: linear
pass_plan:
  - id: alpha
    title: Structural Field
    target_words: 3000
    template: pass-alpha-template
engine_overlay_weights:
  panchanga: 1.0
house_overlay: [1, 4, 7, 10]
bridge_mandates:
  - "Braid Vedic and HD data"
svg_topology: dyad-arc
register_variants:
  l1_l3:
    target_words:
      min: 8000
      max: 10000
    overrides:
      - pass_id: alpha
        template: pass-alpha-template-l1-l3
---

## pass-alpha-template
This is the alpha prompt for {{subject_names}}.

## pass-alpha-template-l1-l3
This is the L1-L3 alpha prompt for {{subject_names}}.

## lessons

### 2026-06-01 — Test lesson
**Question:** Does this work?
**Adopted:** Yes.
