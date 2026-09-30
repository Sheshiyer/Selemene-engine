---
mode: integrated-kundali-reference
subject_count:
  min: 1
  max: 1
roles:
  - primary
target_words:
  min: 12000
  max: 18000
architecture: linear
pass_plan:
  - id: opening
    title: "Opening — Scope and Available Systems"
    target_words: 600
    template: opening-pass
  - id: part1
    title: "Part I — The Convergence Map"
    target_words: 1200
    template: part1-pass
  - id: part2
    title: "Part II — The Vedic Foundation"
    target_words: 2000
    template: part2-pass
  - id: part3
    title: "Part III — The Karmic Architecture (Past → Present)"
    target_words: 1200
    template: part3-pass
  - id: part4
    title: "Part IV — Career & Dharma"
    target_words: 1400
    template: part4-pass
  - id: part5
    title: "Part V — Wealth & Money"
    target_words: 1100
    template: part5-pass
  - id: part6
    title: "Part VI — Love, Marriage, Spouse"
    target_words: 1200
    template: part6-pass
  - id: part7
    title: "Part VII — Health & Energy Body"
    target_words: 1000
    template: part7-pass
  - id: part8
    title: "Part VIII — Family, Roots, and Soul Lineage"
    target_words: 1000
    template: part8-pass
  - id: part9
    title: "Part IX — The Master Timeline"
    target_words: 1800
    template: part9-pass
  - id: part10
    title: "Part X — Practices and Anti-Dependency"
    target_words: 1000
    template: part10-pass
  - id: part11
    title: "Part XI — Final Synthesis"
    target_words: 900
    template: part11-pass
engine_overlay_weights:
  vimshottari: 1.0
  transits: 0.95
  human-design: 0.9
  gene-keys: 0.8
  numerology: 0.75
  panchanga: 0.7
house_overlay: [1, 2, 4, 5, 7, 8, 9, 10, 11, 12]
bridge_mandates:
  - "Write reader-facing report prose only. Never print mode names, register IDs, internal receipt sections, claimed Jev verdicts, acceptance status, or invented execution claims. Those are produced by software outside the narrative. Begin directly with section content; the stitcher supplies the section title."
  - "Section structure must follow the reference contract exactly: Opening + Parts I–XI. Never collapse or rename required sections; mark any section whose required evidence is absent as a coverage gap rather than silently omitting it."
  - "The Aletheios/Pichet witness dyad must be invoked for every section requiring structural/experiential synthesis. A missing dyad is a blocker, not a warning. Record dyad receipts per section."
  - "Grounding retrieval state must be recorded per section: success, empty, failure, and disabled are distinct states. A retrieval failure is not a silent pass. Sections dependent on retrieval that cannot be grounded must be marked as having unresolved retrieval dependency."
  - "Claim-to-source map: every factual claim must be traceable to a named engine result field or an attributed source reference. Do not derive facts from prior knowledge, infer missing values, or populate engine fields that were not supplied."
  - "Scope classification: count source records separately from independent systems. An empty Enneagram questions-mode result is unassessed, not a dated state or an inferred type. Transits contains separate natal_positions and transit_positions; label each by its actual scope. Count numerology fields from the supplied result before making a numerical inventory; a five-core-value summary must explicitly distinguish any additional Chaldean value."
  - "Birth-time uncertainty: preserve the supplied confidence flag without inventing a minute interval, alternate-time result, or comparative reliability ranking. Boundary proximity alone does not establish how small a time shift changes the chart. Do not assert that Human Design or Gene Keys is unaffected by approximate birth time; follow the recorded derivation and shared source."
  - "Cross-system comparisons require actual competing claims. An absent dignity field is not disagreement with a supplied dignity label, and a nakshatra description is not a dignity or strength ranking. Name supplied source records as records; authenticated captures are not additional engine calculations or independent confirmations."
  - "Recount all numerical inventories from the actual members, distinguishing seven planets from nine positions including the two nodes. Verify empty-house counts and each house membership separately. Report supplied retrograde flags without adding universal motion rules. An unavailable Atmakaraka or yoga ruleset remains a coverage gap; arithmetic alone does not establish a traditional classification."
  - "Reader-facing citations must use the supplied stable corpus ID or a named engine result field. Do not cite prompt-only ordinals such as Quoted Passage 1: readers do not have that prompt. A repeated or derived source is not independent corroboration, and absent precision metadata does not imply a measured accuracy level."
  - "Date discipline: no date for any somatic, biorhythm, transit, dosha, or body-related engine may be derived from birth data, the computer clock, another engine, or forecast cadence. Use only exact timestamps in that engine's own result, identified as saved snapshots."
  - "Symbolic labels (Gene Keys Shadow/Gift/Siddhi, HD channels, Tarot Arcana) must never be translated into physiological findings, diagnoses, or certain life-event predictions. State they are symbolic framework labels from their attributed source."
  - "Word-fit gate, deterministic-fact gate, integrated-layering gate, guardrail gate, dyad receipt gate, and retrieval receipt gate are ALL required for final acceptance. A PASS verdict that omits any gate is incomplete."
  - "Synthesis in Part XI must consume all accepted section artifacts. No unsupported facts may be introduced in Part XI. If a section artifact was marked as coverage gap or retrieval failure, note that in the synthesis rather than assuming its content."
  - "Section hashes must be recorded at the time the section is accepted. Post-review edits must update the hash. Final artifact hash must match the last accepted hash."
  - "Both French and English renderings must carry the same facts and coverage. Cross-language judge or deterministic diff check required before acceptance."
  - "Legacy reference examples (prior DOCX/PDF) are editorial references for structure, pacing, and navigability only; never for sourcing facts. Do not reproduce facts, placements, predictions, or remedies from the reference document."
  - "Jev judgment for every section: shadow or active mode. Ambiguous Jev judgments have an explicit disposition (not relabeled as passed). Blocked sections are blockers in final verification."
svg_topology: dyad-arc
relationship_types: []
report_level: L4
jev_guardrail: descriptive
---

## opening-pass
Write the Opening — "Scope and Available Systems" — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Establish what this report covers and how the available systems offer distinct interpretive lenses without claiming mutual validation. Name only systems present in the supplied engine records. Explain the reading method: structural precision and experiential reflection are reconciled. Reserve detailed placements and numerical chart values for later chapters; any scope table here should contain system, source, and scope only.

Use this exact chapter map, translated when needed: I The Convergence Map; II The Vedic Foundation; III The Karmic Architecture; IV Career and Dharma; V Wealth and Money; VI Love, Marriage, Spouse; VII Health and Energy Body; VIII Family, Roots, and Soul Lineage; IX The Master Timeline; X Practices and Anti-Dependency; XI Final Synthesis. Do not invent a system-by-system alternative outline. Health, money, and relationship chapters describe symbolic frameworks and evidence limits, without clinical or event claims.

Write 500–650 words. Do not include an execution receipt, fabricated review verdict, internal mode metadata, or duplicate opening title.

Do not derive biography, promise outcomes, or introduce facts not in the engine records. Target length: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part1-pass
Write Part I — The Convergence Map — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Subsections required:
- 1.1 The Identity Stack: list all present systems and their scope in a labeled table. Quote a source precision label only when explicitly supplied; never infer accuracy from the number of displayed digits. Missing precision is "not supplied".
- 1.2 Shared Observations — The Bedrock: list only claims supported by at least two separate engine results; label each with the engine IDs that support it. Shared source inputs are not independent validation.
- 1.3 What the Systems Disagree On — The Texture: list explicitly where two or more systems offer different or non-overlapping information; label each with the engine IDs

If a system result is absent or errored, mark that row in the table with "not supplied" rather than filling from prior knowledge.

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part2-pass
Write Part II — The Vedic Foundation — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Subsections required (mark as coverage gap if evidence absent):
- 2.1 Lagna — include exact sign, degree, and nakshatra pada from the supplied Vedic engine result
- 2.2 Moon — exact sign, degree, nakshatra pada from Vedic engine result
- 2.3 Sun — exact sign, degree, nakshatra pada from Vedic engine result
- 2.4 Planetary Placement Table — reproduce every planet row from the supplied engine result; do not omit or abbreviate
- 2.5 House Distribution — describe the house loading as supplied; do not invent missing house placements
- 2.6 Notable Yogas — list only yogas present in the supplied engine result with their exact engine labels
- 2.7 Yogas, Doshas, Strengths — from supplied engine facts and attributed retrieved framework passages
- 2.8 Functional Benefics & Malefics — from supplied engine facts and attributed retrieved framework passages; if absent, mark as not supplied
- 2.9 Atmakaraka — from supplied engine facts and attributed retrieved framework passages; if absent, mark as not supplied
- 2.10 The Cosmic Signature — one sentence from structural facts only, no prediction

Use a table for planetary placements. Every placement must cite the engine_id and field name it comes from.

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part3-pass
Write Part III — The Karmic Architecture (Past → Present) — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Subsections required (mark as coverage gap if evidence absent):
- 3.1 The Rahu-Ketu Axis — from Vedic engine; exact nodes, signs, and houses as supplied
- 3.2 The 8th House — from Vedic engine; placements only, no life-event prediction
- 3.3 The 12th House — from Vedic engine; placements only
- 3.4 The Lived Mahadasha Trail — from Vimshottari engine; list the supplied period history with dates; do not invent undated periods
- 3.5 Patterns Likely Repeating — interpretive section labeled as symbolic synthesis; cite the specific engine values it draws on; conditional language required

If Vimshottari historical periods are not in the supplied result, state that explicitly rather than fabricating a timeline.

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part4-pass
Write Part IV — Career & Dharma — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Subsections required:
- 4.1 The 10th House — from Vedic engine; exact placements as supplied
- 4.2 Relevant Raj Yoga or equivalent — from Vedic facts and attributed retrieved framework passages; if absent, mark not supplied
- 4.3 Career Path — Best-Fit and No-Fit — interpretive; labeled as synthesis; cite supporting engine values; use conditional language
- 4.4 Career Timeline by Mahadasha — from Vimshottari engine; only periods that appear in the supplied result with their supplied dates
- 4.5 Foreign Settlement / Travel — from Vedic engine or transits engine data only
- 4.6 Fame and Public Recognition — from supplied engine data; conditional and labeled as interpretive

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part5-pass
Write Part V — Wealth & Money — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Subsections required (mark as coverage gap if evidence absent):
- 5.1 The 2nd House — exact placements from Vedic engine
- 5.2 The 11th House — exact placements from Vedic engine
- 5.3 Property & Real Estate — interpretive; labeled; cite supporting engine values
- 5.4 Inheritance & Sudden Gains — from Vedic facts and attributed retrieved framework passages; conditional language
- 5.5 Best Earning Periods — from Vimshottari periods with supplied dates; not fabricated
- 5.6 Debt & Caution — from supplied engine data; conditional; no investment advice

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part6-pass
Write Part VI — Love, Marriage, Spouse — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Subsections required:
- 6.1 The 7th House and Its Lord — exact placements from Vedic engine
- 6.2 Darakaraka — from Vedic engine if present; mark not supplied if absent
- 6.3 Love Marriage vs Arranged — interpretive synthesis labeled as such; cite supporting values; conditional language
- 6.4 Marriage Timing Indicators — from Vimshottari periods with supplied dates only; never fabricate dates
- 6.5 Marriage Quality, Stability, Friction — interpretive; labeled; cite values; conditional
- 6.6 Divorce / Separation Indicators — if present in Vedic engine only; conditional language; no certainty
- 6.7 The Spouse Profile — interpretive synthesis; labeled; cite supporting values; conditional

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part7-pass
Write Part VII — Health & Energy Body — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Subsections required:
- 7.1 Constitution — from Vedic engine or Panchanga only; qualitative description; no biometric numbers
- 7.2 Scope of Supplied Body and Rhythm Records — from supplied Vedic engine data only; symbolic labels only; never diagnose
- 7.3 Mental & Emotional Health — from supplied Vedic engine data; symbolic/descriptive; no diagnosis
- 7.4 Dated Rhythm Records — identify supplied calculation dates; never infer health-sensitive ages from dasha periods
- 7.5 Self-Observation Questions — from Human Design engine or supplied framework data only; qualitative; no prescriptions

Do not translate symbolic chart data into physiological findings, medical diagnoses, biometric claims, or somatic instructions not derived from a supplied engine result. If an engine is absent, mark the subsection as not supplied.

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part8-pass
Write Part VIII — Family, Roots, and Soul Lineage — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Subsections required:
- 8.1 Mother — from Vedic engine (Moon, 4th house) placements; no life-event prediction
- 8.2 Father — from Vedic engine (Sun, 9th house) placements; no life-event prediction
- 8.3 Siblings — from Vedic engine (3rd house) placements; no life-event prediction
- 8.4 Children — from Vedic engine (5th house) placements; conditional language; no prediction
- 8.5 Family Karma & Inherited Patterns — interpretive synthesis; labeled; cite supporting values; conditional
- 8.6 Spiritual Growth — from supplied engine facts and attributed retrieved framework passages; no prescribed practices not in engine output

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part9-pass
Write Part IX — The Master Timeline — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Subsections required:
- 9.1 Where You Stand Right Now — from Vimshottari current period (mahadasha + antardasha) with supplied dates
- 9.2 Any named upcoming pivot — from transits engine or Vimshottari supplied data only; state the exact engine source and date if present; if absent mark not supplied
- 9.3 Supplied Mahadasha Antardasha Map — from Vimshottari engine; reproduce the supplied period map with dates; do not extrapolate beyond supplied data
- 9.4 Sade Sati Overlay — from transits engine if present; exact dates as supplied; if absent mark not supplied
- 9.5 Dated Period-by-Period Map — from Vimshottari antardasha periods with supplied dates only; one paragraph per period that appears in the engine result; do not fabricate periods not in the result
- 9.6 The Master Cross-Reference Table — table of planetary period × theme; themes labeled as interpretive synthesis; cite specific engine values used

Use jev_guardrail: descriptive — dated periods may be described as tendencies and invitations; never as guarantees, certainties, or promises about life events.

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part10-pass
Write Part X — Practices and Anti-Dependency — for {{subject_names}}.

Engine facts:
{{engine_facts}}

Subsections required (mark as coverage gap if engine data is absent):
- 10.1 Vedic Reflection Practices — from Vedic facts and attributed retrieved framework passages; no shopping lists; build decoding capacity (anti-dependency contract)
- 10.2 Numerology Coordinates — from numerology engine data only
- 10.3 HD-Aligned Practices — from Human Design facts and attributed retrieved framework passages; qualitative; no prescriptions
- 10.4 Gene Keys — from Gene Keys engine data only; symbolic labels as supplied; no trait claims
- 10.5 Independent Reflection and Follow-Through — interpretive synthesis; labeled; cite engine values; no advice not grounded in supplied data

Anti-dependency rule: every remedy or practice must be formulated as a self-decoding capacity milestone the reader can verify in their own life. Do not prescribe products, mantras, gemstones, or actions that require external purchase or service, unless explicitly sourced from the engine result.

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## part11-pass
Write Part XI — Final Synthesis — for {{subject_names}}.

Engine facts:
{{engine_facts}}

This section must consume all accepted section artifacts from Parts I–X. Do not introduce new engine facts or source claims not covered in the accepted prior sections.

Subsections required:
- 11.1 The Single Sentence — one sentence distilling the structural signature from accepted facts only
- 11.2 The Biggest Life Lesson — interpretive synthesis; labeled; cite the specific accepted section evidence
- 11.3 What to Avoid — from accepted section artifacts; conditional language; cite section IDs
- 11.4 What to Strongly Pursue — from accepted section artifacts; conditional language; cite section IDs
- 11.5 Source-Grounded Strength Themes — from accepted Vedic + HD + Gene Keys artifacts; cite section IDs
- 11.6 Core Karmic Purpose — from accepted section artifacts; labeled as interpretive synthesis
- 11.7 Sourced Timing Summary — from Vimshottari + transits data in accepted section artifacts; jev_guardrail: descriptive; state tendencies, not certainties
- 11.8 A Shared Reflection Practice Across Available Systems — anti-dependency; one practice verifiable by the reader without external products
- 11.9 Closing — one paragraph; no new facts; reference the reading method

If any required section artifact was marked as coverage gap or retrieval failure, note that explicitly here rather than assuming its content.

Target: ~{{target_words}} words (pass id: {{pass_id}}).

Prior context:
{{prior_pass}}

Bridge mandates:
{{bridge_mandates}}

## lessons

### 2026-09-30 — Reference-aligned route contract
**Question:** What makes the integrated-kundali-reference mode distinct from the existing linear modes?
**Adopted:** Strict Opening + Parts I–XI section manifest matching the reference DOCX; mandatory dyad invocation per section; explicit retrieval receipts; claim-to-source mapping; required section coverage with visible gap markers; all rubric dimensions enforced in final verification.

### 2026-09-30 — Dyad receipt required per section
**Question:** Is a dyad receipt recorded even when the dyad provider is unavailable?
**Adopted:** Yes — the receipt records DYAD_UNAVAILABLE with the reason. Missing dyad is a blocker in final verification, not a silent pass or warning. The section output contains the DYAD_UNAVAILABLE_MARKER so downstream inspection can find it.
