---
mode: birth-blueprint
report_level: L1
subject_count:
  min: 1
  max: 1
roles:
  - subject
target_words:
  min: 1600
  max: 2400
architecture: linear
pass_plan:
  - id: natal
    title: Natal Field
    target_words: 900
    template: natal-pass
  - id: invitation
    title: One Living Invitation
    target_words: 700
    template: invitation-pass
engine_overlay_weights:
  panchanga: 1.0
  vimshottari: 0.95
  human-design: 0.8
  numerology: 0.65
house_overlay: [1, 4, 10]
bridge_mandates:
  - "Stay strictly with the birth data; do not import current transits unless explicitly part of the mode"
  - "One clear invitation only — no laundry list"
  - "Name the dominant structural signature without overclaiming"
svg_topology: dyad-arc
---

## natal-pass

Map the birth signature for {{subject_names}}.

Register: {{register}}.

Overlay: {{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Identify the strongest 2-3 interlocking facts from the deterministic engines at the moment of birth. Note the primary house emphasis and any tight conjunction of timing + type. Keep language factual and spare.

## invitation-pass

From the natal field, surface the single most alive invitation.

Prior pass:
{{prior_pass}}

Register: {{register}}.

Target ~{{target_words}} words.

Offer one precise, autonomy-honoring question or micro-practice the subject can meet the pattern with. No promises. No "you are a X". End with the question living in the body.

## overlay-rules

Birth-blueprint is natal-only. Do not weave current sky unless the data explicitly includes a transit set (this mode does not).

When register l1_l3 keep language simple and descriptive. l4_l5 may add one layer of "how this pattern tends to feel when lived unconsciously".

## lessons

### 2026-05-28 — Birth data modes kept trying to become full life readings

**Question:** How do we keep a birth-blueprint from ballooning into a full integrated reading?
**Adopted:** Strict pass budget (max 2 passes), hard cap on engine set, and the "one living invitation" rule in the final pass.
