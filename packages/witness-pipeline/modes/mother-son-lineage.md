---
mode: mother-son-lineage
subject_count:
  min: 2
  max: 2
roles:
  - mother
  - son
target_words:
  min: 4000
  max: 6000
architecture: linear
pass_plan:
  - id: opening
    title: Opening
    target_words: 400
    template: opening-template
  - id: lineage-field
    title: Lineage Field
    target_words: 1200
    template: lineage-field-template
  - id: transmission-patterns
    title: Transmission Patterns
    target_words: 1200
    template: transmission-patterns-template
  - id: synthesis
    title: Synthesis
    target_words: 800
    template: synthesis-template
engine_overlay_weights:
  human-design: 1.0
  gene-keys: 0.9
  vimshottari: 0.8
  numerology: 0.7
house_overlay: [1, 4, 7, 10, 5]
bridge_mandates:
  - "Explicitly name mother and son roles in every pass"
  - "Never predict outcomes for the child"
svg_topology: dyad-arc
relationship_types:
  - family
report_level: L2
---

## opening-template
# {{relationship_header}}

Subjects: {{subject_roles}}

Mapping goal: {{mapping_goal}}

This is a non-predictive pattern witness for the mother-son lineage field.

## lineage-field-template
Map the structural patterns between {{subject_roles}} using the provided engine data. Reference concrete facts only. Do not predict the son's future or the mother's outcomes.

## transmission-patterns-template
Witness what is being transmitted (patterns, activations, timings) from mother to son or across the dyad. Use explicit role language. Stay descriptive.

## synthesis-template
Synthesize the lineage field observations. End with one open witness question. No prescriptions.

## lessons
### 2026-07-10 — Seeded from relationship mapping contract
**Question:** Can we declare explicit mother-son roles without romantic presumption?
**Adopted:** Yes — roles + relationship_context + header.
