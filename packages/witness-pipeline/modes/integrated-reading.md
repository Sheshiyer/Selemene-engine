---
mode: integrated-reading
subject_count:
  min: 1
  max: 2
roles:
  - subject
target_words:
  min: 4200
  max: 5800
architecture: linear
pass_plan:
  - id: structural
    title: Structural Field
    target_words: 1800
    template: structural-pass
  - id: somatic
    title: Somatic Vitality
    target_words: 1200
    template: somatic-pass
  - id: synthesis
    title: Synthesis & Invitation
    target_words: 1500
    template: synthesis-pass
engine_overlay_weights:
  panchanga: 1.0
  vimshottari: 0.9
  human-design: 0.85
  gene-keys: 0.75
  numerology: 0.6
house_overlay: [1, 4, 7, 10]
bridge_mandates:
  - "Braid deterministic facts from multiple engines before any interpretation"
  - "Hold the tension between structure (Aletheios) and flow (Pichet) without forcing resolution"
  - "Surface one concrete, non-prescriptive invitation per major pass"
  - "Name shadow/gift dynamics only when consciousness register supports it"
svg_topology: dyad-arc
register_variants:
  l1_l3:
    target_words:
      min: 3800
      max: 5200
    overrides:
      - pass_id: synthesis
        template: synthesis-pass-l1-l3
  l4_l5:
    target_words:
      min: 4800
      max: 6500
---

## structural-pass

Compose the structural field for {{subject_names}}.

You are writing in register {{register}}.

Context from prior pass (if any):
{{prior_pass}}

Engine overlay weights and focus houses:
{{overlay_summary}}

Bridge mandates to honor:
{{bridge_mandates}}

Recent autoresearch lessons (use only to avoid repeating past dead-ends):
{{lessons_summary}}

Target length: ~{{target_words}} words (pass id: {{pass_id}}).

Write a precise, non-prescriptive mapping of the dominant structural patterns across the engines. Weave panchanga + vimshottari timing with Human Design gates and Gene Key activations. Identify 2-3 "load-bearing" facts that multiple engines converge on. Do not moralize. Leave tension visible.

## somatic-pass

Now layer the somatic and vitality currents for {{subject_names}}.

Register: {{register}}.

Prior structural field (use for braid, do not repeat verbatim):
{{prior_pass}}

Overlay and houses:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Lessons:
{{lessons_summary}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Describe the felt-sense, biorhythmic, and transiting vitality layer. Note where the structural field is supported or resisted somatically. Name one place where the body is being asked to metabolize a timing or gate activation. Stay close to observable data. Avoid therapy-speak.

## synthesis-pass

Synthesize the reading for {{subject_names}} into a coherent mirror.

Register: {{register}}.

Prior passes (structural then somatic):
{{prior_pass}}

Overlay summary:
{{overlay_summary}}

Bridge mandates (especially the non-resolution and invitation rules):
{{bridge_mandates}}

Lessons:
{{lessons_summary}}

Target ~{{target_words}} words. Pass id: {{pass_id}}.

Hold Aletheios (structure) and Pichet (vitality) in the same frame. Surface the primary tension or coherence. Offer 1-2 precise invitations that honor the subject's autonomy. Do not promise outcomes. End on an open question the subject can live into.

## synthesis-pass-l1-l3

Synthesize for {{subject_names}} at the l1_l3 register (earlier consciousness band).

Prior material:
{{prior_pass}}

Focus on clear pattern recognition. Use simpler language. Emphasize observable facts over deep shadow work. One small invitation only. Target ~{{target_words}} words.

## overlay-rules

When describing engine contributions, always cite the specific engine and the concrete data point (e.g., "Panchanga tithi X on date Y" or "HD gate 34 in the sacral center").

Never use the engines to diagnose, predict, or prescribe. The output is a mirror, not a verdict.

When register is l1_l3, avoid or soften Gene Keys shadow language and siddhi language. When l4_l5, you may surface shadow/gift/siddhi dynamics but still as possibilities, not identities.

## lessons

### 2026-06-12 — Over-braiding caused loss of subject voice

**Question:** In multi-engine passes, how do we prevent the synthesis from sounding like a committee report?
**Variants:** Heavy cross-citation / Lighter voice with selective braid / Let engines speak in turn then one final human-voiced pass
**Winner:** Lighter voice with selective braid
**Adopted:** Let each pass have a primary voice; only braid the load-bearing facts that genuinely converge. The final invitation must sound like one coherent person wrote it.
**Reference:** witness-agents v0.9.4 post-mortem

### 2026-06-20 — l1_l3 users disoriented by siddhi language

**Question:** Should register variants affect only length or also depth of shadow/siddhi language?
**Adopted:** Yes — l1_l3 synthesis template must explicitly instruct lighter language and forbid siddhi framing.
