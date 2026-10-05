---
mode: partner-synastry
report_level: L2
subject_count:
  min: 2
  max: 2
roles:
  - partner
  - partner
target_words:
  min: 3200
  max: 5000
architecture: linear
pass_plan:
  - id: opening
    title: Opening
    target_words: 300
    template: opening-template
  - id: structural-compatibility
    title: Structural Compatibility (Aletheios)
    target_words: 1100
    template: structural-template
  - id: energetic-dance
    title: Energetic Dance (Pichet)
    target_words: 1100
    template: energetic-template
  - id: synthesis
    title: Synthesis
    target_words: 700
    template: synthesis-template
engine_overlay_weights:
  human-design: 1.0
  gene-keys: 0.9
  numerology: 0.7
  vimshottari: 0.8
house_overlay: [1, 7]
bridge_mandates:
  - "Use only the relationship type the caller declared; never assume marriage, romance, or a future"
  - "Compare two fields; do not rank or score the partners against each other"
  - "Name growth edges as present tensions, not as outcomes"
  - "Consciousness level 2 minimum; level 3 and above may name shadow/gift dynamics"
svg_topology: dyad-arc
relationship_types:
  - unmarried-partners
  - married-partners
  - custom
---

## opening-template
# {{relationship_header}}

Subjects: {{subject_roles}}

Mapping goal: {{mapping_goal}}

A comparative pattern witness of two fields. Register: {{register}}.

## structural-template
Aletheios pillar. Compare the two partners' Human Design bodygraphs (types, authorities, definition, defined centres, channels shared or bridged), Gene Keys activation sequences, and numerology codes using the engine facts supplied. Name what is shared, what is complementary, and what is absent. Stay descriptive.

## energetic-template
Pichet pillar. Witness how the two fields move together: authority styles in decision moments, sacral or emotional timing, the current Vimshottari periods of each partner side by side. Describe the dance as it is present now; no forecast.

## synthesis-template
Braid the structural and energetic observations. Name the present tensions as growth edges. One open question. No prescriptions, no outcomes.

## lessons
### 2026-09-27 — Migrated from the legacy mode_id frontmatter to the mode-doc schema
**Question:** The original document declared engines, a register band and a consciousness gate but no pass plan, so `parseModeDoc` rejected it. What is the minimal faithful migration?
**Adopted:** Keep the four-engine selection as overlay weights, express the Aletheios/Pichet/synthesis flow as three passes after an opening, carry the level-2 gate as a bridge mandate, and let the caller's relationship_context decide the framing.
