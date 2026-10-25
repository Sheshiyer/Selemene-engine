---
mode: human-design-integrated-12
subject_count:
  min: 1
  max: 1
roles:
  - primary
target_words:
  min: 3600
  max: 5600
architecture: linear
pass_plan:
  - id: opening
    title: Opening and scope
    target_words: 300
    template: opening-pass
  - id: source-map
    title: Source map and limits
    target_words: 400
    template: source-map-pass
  - id: chart-overview
    title: Identity stack and chart overview
    target_words: 450
    template: chart-overview-pass
  - id: type-profile
    title: Type, profile, authority and definition
    target_words: 450
    template: type-profile-pass
  - id: centers-channels
    title: Centers and complete channels
    target_words: 450
    template: centers-channels-pass
  - id: gates-cross
    title: Gate inventory and incarnation cross
    target_words: 450
    template: gates-cross-pass
  - id: gene-keys-overlay
    title: Attributed Gene Keys overlay
    target_words: 350
    template: gene-keys-pass
  - id: panchanga
    title: Birth Panchanga
    target_words: 300
    template: panchanga-pass
  - id: vimshottari
    title: Vimshottari calculation snapshot
    target_words: 350
    template: vimshottari-pass
  - id: numerology
    title: Numerology and supporting systems
    target_words: 300
    template: numerology-pass
  - id: integration
    title: Integrated patterns and differences
    target_words: 500
    template: integration-pass
  - id: synthesis
    title: Synthesis and reflection
    target_words: 300
    template: synthesis-pass
engine_overlay_weights:
  human-design: 1.0
  humdes-reference: 1.0
  gene-keys: 0.8
  panchanga: 0.65
  vimshottari: 0.65
  numerology: 0.45
house_overlay: []
bridge_mandates:
  - "Use user-supplied legacy readings only as editorial references for pacing, navigability, layered section design, source tables, and cross-system comparison; never treat their facts as current source data."
  - "Aim for the supplied reference's depth through a clear identity/source map, system-by-system foundations, carefully labeled cross-system patterns and differences, dated snapshots where explicitly supplied, and a closing synthesis. Do not imitate its unsupported certainty, medical claims, life forecasts, or remedies."
  - "Make interpretations visibly interpretive: state the exact source labels or values they draw on, distinguish direct repetition from editorial juxtaposition, and use conditional language. Never present interpretation as a deterministic engine fact or as proof that two systems validate one another."
  - "A detailed reading should explain the supplied design and engine data in readable prose and tables, with concrete values, source labels, and limits. Do not meet depth targets by repeating facts or inventing significance."
  - "Do not make somatic, physiological, medical, emotional-state, or body-part claims from symbolic chart data. Include a body/rhythm engine only when that engine result is present, label it as a model output, and do not translate it into health advice or bodily fact."
  - "Never derive a date for a somatic, biorhythm, transit, dosha, or body-related engine from birth data, capture time, the computer clock, another engine, or a forecast cadence. A date may be stated only when that engine's own supplied result contains that exact date or calculation timestamp; identify it as a saved engine snapshot, not a current or future prediction."
  - "For Human Design, reproduce every exact personality_activations and design_activations row supplied in the engine-facts block, including planets beyond the four cross anchors. If the supplied record omits a row, say it is not supplied; never claim that a field is absent without checking the full block."
  - "Do not reproduce facts, biographical details, placements, periods, predictions, medical statements, remedies, or body mappings from editorial-reference documents. Use only the active supplied source pack and exact engine outputs for facts."
  - "Follow the requested language for every heading, sentence, explanation, table label, and question; for fr, write in French throughout while preserving exact source values and proper names."
  - "Treat deterministic Selemene engine results as the only chart-calculation authority."
  - "Attribute HumDes cross names and Gene Keys labels to their saved source reference."
  - "A gate overlap is not a complete channel; list only engine-confirmed complete channels."
  - "Never infer or describe candidate channel pairings from gate numbers, including hypothetical pairs; repeat only exact strings in active_channels."
  - "Separate each system's output and do not claim one system validates another."
  - "Use present-tense descriptive language; no prediction, diagnosis, health inference, or advice."
  - "Never invent Vedic houses, planetary placements, transits, birth confidence, or missing chart fields."
  - "Make factual claims only when the exact value appears in the supplied engine facts or attributed HumDes record."
  - "Do not infer Human Design mechanics, channel topology, traits, capabilities, decisions, or outcomes from labels. State labels and ask open questions instead."
  - "Do not pad sections to hit length. Use concise detail from the supplied values; omit unsupported exposition."
  - "Before naming any channel, verify the exact channel string exists in this subject's active_channels field; do not call unsupported gate pairs connections or pairings."
  - "Keep report section-by-section and let Jev judge every final pass."
svg_topology: dyad-arc
relationship_types: []
report_level: L3
jev_guardrail: descriptive
---

## opening-pass
Write an inviting, source-bounded opening for {{subject_names}}. Establish what this report covers, how its systems are kept distinct, and how to read fact tables versus interpretive commentary. Name only systems present in supplied records. Do not derive biography or promise outcomes.

## source-map-pass
Use source-labeled subsections and bullet lists. Inventory values exactly as provided. State date, time, location, confidence, and timezone only when present in a supplied source note or engine fact. List missing inputs only when they affect a requested calculation. Do not add generic chart-data categories or meanings.

## chart-overview-pass
Open with a compact fact table for type, profile, authority, definition, defined/undefined centres when supplied, complete channels, and gate-line activations. Add a separate row for any HumDes cross name. Follow it with a short reading guide that describes what is directly present in the chart and what is left uninterpreted; attribute every sourced cross-reference.

## type-profile-pass
Explain the supplied type, profile, authority, strategy, and definition labels using only authorized interpretation text in the source pack. Put exact values and provenance in a table, then offer a clearly marked reflective reading that distinguishes source wording from interpretation. If the source pack contains labels only, say what remains unknown instead of assigning traits, capacity, or decision rules.

## centers-channels-pass
Show defined and undefined centres and complete channels in labeled tables, copying identifiers exactly. Explain only sourced mechanics or interpretive descriptions present in the source pack, with attribution and an explicit distinction between chart structure and possible lived reflection. Do not add topology, bridging, health, capacity, compatibility, or professional-fit claims.

## gates-cross-pass
Present the complete personality and design activation table (planet/point, exact gate.line) using every exact row in the engine-facts block, active-gate inventory, P-Sun/P-Earth/D-Sun/D-Earth lines, and source-attributed cross name in separate tables. Do not omit values to shorten the table or claim rows are absent when present in the facts. If source-pack descriptions are available, explain them as symbolic prompts, not traits or destiny. Distinguish HumDes cross labels from engine-calculated activations; do not infer channels, roles, traits, or outcomes from gate intersections.

## gene-keys-pass
Use a table with the supplied Gene Keys activation position, key-line, and Shadow/Gift/Siddhi labels. Attribute the exact source. Do not paraphrase, explain, compare, group, or claim these labels describe the person. Keep this table separate from Human Design facts.

## panchanga-pass
List only the Panchanga values present in the engine output: tithi, nakshatra, vara, yoga, karana. State the backend and supplied input limits. No traditional meanings, traits, mood, behaviour, physiology, or future events.

## vimshottari-pass
Give the Vimshottari birth-nakshatra value and the supplied Mahadasha, Antardasha, and Pratyantardasha planet/date pairs in a table. Identify this as a dated engine snapshot. Do not add period themes, causal explanations, forecasts, or advice.

## numerology-pass
List only the named numerology values, numbers, and meanings literally present in the engine output. Attribute each to that engine. Do not translate a number into a trait or add meanings.

## integration-pass
Create a convergence-and-difference map with columns for system, exact source evidence, and editorial reading. Call a direct overlap only when a source value or label is literally repeated. Any thematic juxtaposition must be labeled interpretation, name its exact inputs, remain conditional, and never claim validation, causation, proof, or predictive significance. Include one open reflection question.

## synthesis-pass
Close with a layered synthesis: stable chart facts; dated calculation snapshots that the relevant engines explicitly supply; and the report's bounded interpretive threads. Introduce no new source values. Keep predictions, diagnoses, medical claims, body mappings, guarantees, and advice out. End with one open question and a complete sentence.

## lessons
### 2026-09-28 — Human Design integration contract
**Question:** How do we integrate multiple symbolic layers without letting them overwrite the chart source?
**Adopted:** Keep engine facts, HumDes attributions, and reflection visibly separate; judge each generated section with Jev in shadow before assembling the report.
