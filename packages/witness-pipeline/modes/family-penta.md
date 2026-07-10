---
mode: family-penta
subject_count:
  min: 3
  max: 7
roles:
  - mother
  - father
  - child1
  - child2
  - child3
target_words:
  min: 6000
  max: 9000
architecture: hierarchical
pass_plan:
  - id: opening
    title: Opening
    target_words: 500
    template: opening-template
  - id: field-overview
    title: Family Field Overview
    target_words: 1500
    template: field-overview-template
  - id: transmission-vectors
    title: Transmission Vectors
    target_words: 1500
    template: transmission-vectors-template
  - id: sibling-dynamics
    title: Sibling Dynamics
    target_words: 1200
    template: sibling-dynamics-template
  - id: synthesis
    title: Synthesis
    target_words: 1000
    template: synthesis-template
engine_overlay_weights:
  human-design: 1.0
  gene-keys: 0.95
  vimshottari: 0.85
  numerology: 0.75
house_overlay: [1, 4, 5, 7, 10]
bridge_mandates:
  - "Always label subjects by their declared roles (mother, father, childN)"
  - "Strictly non-predictive for all children"
  - "No parent-outcome language"
svg_topology: pentagon
relationship_types:
  - family
report_level: L3
---

## opening-template
# {{relationship_header}}

Subjects: {{subject_roles}}

Mapping goal: {{mapping_goal}}

Non-predictive pattern witness for the family penta.

## field-overview-template
Map the overall family field using the five subjects. Reference concrete engine activations. Use role labels.

## transmission-vectors-template
Describe observable transmission patterns (activations, timings, emphases) across generations in this penta. Stay descriptive.

## sibling-dynamics-template
Witness dynamics among the children (child1, child2, child3) and between parents and children. Explicit roles only.

## synthesis-template
Synthesize penta observations into one integrative view. One open witness question. No prescriptions.

## lessons
### 2026-07-10 — Seeded from relationship mapping contract
**Question:** Can a 5-person family penta be modeled with explicit roles and pentagon topology?
**Adopted:** Yes — roles + pentagon + family type.
