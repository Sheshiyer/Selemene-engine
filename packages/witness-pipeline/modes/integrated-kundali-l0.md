---
mode: integrated-kundali-l0
report_level: L0
subject_count:
  min: 1
  max: 1
roles:
  - subject
target_words:
  min: 15000
  max: 21000
architecture: linear
pass_plan:
  - id: opening
    title: Opening
    target_words: 450
    template: opening-pass
  - id: convergence-map
    title: Part I — The Convergence Map
    target_words: 1600
    template: convergence-map-pass
  - id: vedic-foundation
    title: Part II — The Vedic Foundation
    target_words: 3800
    template: vedic-foundation-pass
  - id: karmic-architecture
    title: Part III — The Karmic Architecture
    target_words: 1500
    template: karmic-architecture-pass
  - id: career-dharma
    title: Part IV — Career and Dharma
    target_words: 1600
    template: career-dharma-pass
  - id: wealth
    title: Part V — The Wealth Architecture
    target_words: 1200
    template: wealth-pass
  - id: love-marriage
    title: Part VI — Love and Marriage
    target_words: 1500
    template: love-marriage-pass
  - id: health
    title: Part VII — Health
    target_words: 1100
    template: health-pass
  - id: family-lineage
    title: Part VIII — Family, Roots, and Lineage
    target_words: 1050
    template: family-lineage-pass
  - id: master-timeline
    title: Part IX — The Master Timeline
    target_words: 2200
    template: master-timeline-pass
  - id: remedies-practices
    title: Part X — Remedies and Practices
    target_words: 1700
    template: remedies-practices-pass
  - id: final-synthesis
    title: Part XI — The Final Synthesis
    target_words: 1300
    template: final-synthesis-pass
engine_overlay_weights:
  panchanga: 1.0
  vimshottari: 1.0
  human-design: 0.9
  gene-keys: 0.85
  transits: 0.8
  numerology: 0.35
house_overlay: [1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12]
bridge_mandates:
  - "Use deterministic chart data first; interpretation must follow cited facts."
  - "Preserve the premium kundali report shape: section-by-section, not one-shot."
  - "Keep Witness safeguards in money, marriage, children, and health sections."
  - "Frame synthesis as pattern-reading, not certainty, diagnosis, or fate."
  - "Use Aletheios for structure and Pichet for vitality, then braid only real convergence."
  - "Cite only facts present in the supplied engine results. Never invent or generalize chart positions."
  - "For every major claim (Lagna, key gates, current dasha, tithi/nakshatra), name the exact value from the engines."
svg_topology: dyad-arc
register_variants:
  l1_l3:
    target_words:
      min: 13000
      max: 17000
    overrides:
      - pass_id: final-synthesis
        template: final-synthesis-pass-l1-l3
  l4_l5:
    target_words:
      min: 17000
      max: 23000
---

## opening-pass

Write the opening for {{subject_names}}'s premium integrated kundali report.

Register: {{register}}.

Prior pass context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Open with a sacred but grounded tone. Name the five-system stack: Vedic astrology, Human Design, Gene Keys, Vimshottari timing, and transits. State that the report is a deterministic-data synthesis with interpretive layering, not a one-shot oracle. Do not make predictions in the opening.

## convergence-map-pass

Build Part I, the convergence map for {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Create the top-level map before interpreting life areas. Include: five systems at a glance, master convergence, divergence/texture, timing spine, and a single-sentence reading. Require concrete facts from at least three deterministic systems before naming a convergence.

## vedic-foundation-pass

Build Part II, the Vedic foundation for {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Use D1/D9 chart facts, lagna, Moon, Sun, atmakaraka, darakaraka, planetary placements, yogas, functional benefics/malefics, Rahu-Ketu, and Sade Sati/transit state where present. Prefer table-like clarity for placements and yogas. Do not infer missing chart data.

## karmic-architecture-pass

Build Part III, karmic architecture for {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Read Rahu-Ketu, Moon nakshatra, atmakaraka curriculum, and lived mahadasha trail as pattern architecture. Use karmic language as symbolic interpretation, not literal proof of past lives. Distinguish deterministic timing from reflective meaning.

## career-dharma-pass

Build Part IV, career and dharma for {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Synthesize Vedic career houses/lords, Human Design mechanics, Gene Keys vocation/pearl, and timing periods. Give directional fields and fit patterns, not absolute job prescriptions. Separate dharma from career outcome.

## wealth-pass

Build Part V, wealth architecture for {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Read 2nd/11th houses, wealth karakas, dasha timing, and relevant HD/Gene Keys material. Do not guarantee financial outcomes, investment gains, inheritance, debt resolution, or property events. Frame money as capacity, pattern, timing pressure, and decision hygiene. Include cautions as reflective risk patterns, not financial advice.

## love-marriage-pass

Build Part VI, love and marriage for {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Read the 7th house, darakaraka, Venus/Mars/Jupiter, D9, relationship mechanics, and timing windows. Do not predict marriage inevitability, divorce, spouse identity, or fixed relationship outcomes. Present timing as relational weather and readiness windows. Preserve agency and consent.

## health-pass

Build Part VII, health for {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Read constitutional tendencies, 6th/8th/12th house indicators, planetary stressors, biofield/vitality context where available, and practical regulation themes. Do not diagnose, treat, forecast disease, replace medical care, or claim certainty about health events. Use non-medical language: tendency, load, rhythm, recovery, vitality, support.

## family-lineage-pass

Build Part VIII, family, roots, and lineage for {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Read mother, father, siblings, children, in-laws, roots, and inherited patterns from house and karaka indicators. Do not predict childbirth, infertility, child outcomes, parent death, estrangement, or family events. Treat lineage as symbolic patterning and relational inheritance, not fixed biography.

## master-timeline-pass

Build Part IX, the master timeline for {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Make timing the spine: current mahadasha/antardasha, upcoming transitions, Sade Sati/transit overlays, and major life chapters. Use cross-reference tables where helpful. Distinguish exact deterministic dates from interpretive strategic readings.

## remedies-practices-pass

Build Part X, remedies and practices for {{subject_names}}.

Register: {{register}}.

Prior context:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Offer Vedic remedies, HD practices, Gene Keys contemplation, and practical life corrections as optional supports. Avoid coercive language. Do not prescribe gemstones, fasting, donations, mantras, or rituals as guaranteed fixes. Include gentle cautions when strengthening a planet could over-amplify a chart tension.

## final-synthesis-pass

Build Part XI, final synthesis for {{subject_names}}.

Register: {{register}}.

Prior sections:
{{prior_pass}}

Overlay:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Close with the exemplar structure: single-sentence reading, biggest lesson, what to avoid, what to pursue, destiny strengths, karmic purpose, honest prediction framed as pattern-not-fate, and one integrating practice. Keep the final claim humble, grounded, and autonomy-preserving.

## final-synthesis-pass-l1-l3

Build the L1-L3 final synthesis for {{subject_names}}.

Prior sections:
{{prior_pass}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Use simpler language, fewer esoteric terms, and more observable pattern recognition. Avoid siddhi-heavy Gene Keys language. Keep the “honest prediction” as a grounded possibility statement, not a fate statement.

## overlay-rules

Every major section must name which deterministic system supplied each load-bearing fact. Do not let retrieved concepts or exemplar style override chart data.

Grounding rule: You are given deterministic engine results. Every specific claim about the chart (signs, gates, dashas, tithis, nakshatras) must be traceable to those results. If a fact is not in the engines, do not state it as true for this person.

Never use this report to diagnose, prescribe, promise outcomes, or remove agency. Do not guarantee financial outcomes. Do not predict marriage inevitability. Do not diagnose. Do not predict childbirth.

For l1_l3, use clear pattern language and soften shadow/siddhi terminology. For l4_l5, deeper shadow/gift language is allowed only as contemplative possibility, not identity.

## lessons

### 2026-07-04 — L0 kundali requires premium interpretation with Witness safeguards

**Question:** Should the L0 integrated kundali report be pure Witness Dyad or premium interpretation?
**Variants:** Pure mirror / Premium kundali report / One-shot generated report
**Winner:** Premium kundali report with Witness safeguards
**Adopted:** Preserve the 11-part exemplar structure and section-by-section generation while enforcing safeguards for health, marriage, children, and money.
**Reference:** Review of Kundali_Integrated_Chitra, Harshita, and Varsha DOCX exemplars.
