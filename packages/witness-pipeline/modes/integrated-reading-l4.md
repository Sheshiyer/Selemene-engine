---
mode: integrated-reading-l4
report_level: L4
subject_count:
  min: 1
  max: 2
roles:
  - subject
target_words:
  min: 4800
  max: 6500
architecture: linear
pass_plan:
  - id: structural
    title: Structural Field
    target_words: 1800
    template: structural-pass
  - id: synthesis
    title: Synthesis & Invitation
    target_words: 1500
    template: synthesis-pass
engine_overlay_weights:
  panchanga: 1.0
  vimshottari: 0.9
  human-design: 0.85
house_overlay: [1, 4, 7, 10]
bridge_mandates:
  - "Braid deterministic facts from multiple engines before any interpretation"
  - "Hold the tension between structure (Aletheios) and flow (Pichet) without forcing resolution"
svg_topology: dyad-arc
---

## structural-pass

Compose the structural field for {{subject_names}} at L4 depth.

Register: {{register}}.

Context from prior pass (if any):
{{prior_pass}}

Engine overlay weights and focus houses:
{{overlay_summary}}

Bridge mandates to honor:
{{bridge_mandates}}

Target length: ~{{target_words}} words (pass id: {{pass_id}}).

Write a precise, non-prescriptive mapping of the dominant structural patterns across the engines.

## synthesis-pass

Synthesize the reading for {{subject_names}} into a coherent mirror at L4.

Register: {{register}}.

Prior passes:
{{prior_pass}}

Overlay summary:
{{overlay_summary}}

Bridge mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass id: {{pass_id}}.

Hold structure and vitality. Surface primary tension. Offer precise invitations. Do not promise outcomes.

## lessons

### 2026-07-10 — L4 explicit coverage
**Question:** Do we need a dedicated L4 mode doc for matrix enumeration?
**Adopted:** Yes — thin alias with L4 frontmatter for parser and matrix coverage.
