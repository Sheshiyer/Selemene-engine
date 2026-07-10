---
mode: business-partners
subject_count:
  min: 2
  max: 2
roles:
  - business-partner
  - business-partner
target_words:
  min: 3500
  max: 5500
architecture: linear
pass_plan:
  - id: opening
    title: Opening
    target_words: 350
    template: opening-template
  - id: partnership-field
    title: Partnership Field
    target_words: 1100
    template: partnership-field-template
  - id: decision-dynamics
    title: Decision Dynamics
    target_words: 1100
    template: decision-dynamics-template
  - id: synthesis
    title: Synthesis
    target_words: 700
    template: synthesis-template
engine_overlay_weights:
  human-design: 1.0
  gene-keys: 0.9
  numerology: 0.8
house_overlay: [1, 7, 10]
bridge_mandates:
  - "Use business-partner language only; never romantic framing"
  - "No investment or outcome guarantees"
svg_topology: dyad-arc
relationship_types:
  - business-partners
report_level: L2
---

## opening-template
# {{relationship_header}}

Subjects: {{subject_roles}}

Mapping goal: {{mapping_goal}}

Non-predictive pattern witness for the business partnership.

## partnership-field-template
Describe the structural and energetic field between the two business-partners using engine facts. Name roles explicitly.

## decision-dynamics-template
Witness complementary or frictional decision patterns between the business-partners. Stay descriptive.

## synthesis-template
Synthesize partnership observations. One open question. No prescriptions.

## lessons
### 2026-07-10 — Seeded from relationship mapping contract
**Question:** Can business dyads be framed without romantic or financial prediction language?
**Adopted:** Yes — explicit roles + guardrails.
