# Jev gate for the witness pipeline

Status: implemented in shadow mode and exercised live on 2026-09-27. The key lives in `~/.claude/.env` as `API_KEY` under the `####JEV-TYPESAFE-AI####` header; `loadTypesafeKey` resolves that section-scoped form (a plain `TYPESAFE_API_KEY` also works). The TypeSafe skill is still not installed locally; the pipeline uses the raw HTTP client, as urania does.

## What Jev is

TypeSafe's Jev (System One, `POST https://api.typesafe.ai/v1/systemone`, model `jev-latest`) answers typed questions only: `choice` among supplied options, `score` on a supplied scale, `noul` yes/no as a probability. It does not write prose. The recurring lesson across the ingested sources is placement, not model swapping: delete the LLM calls that only ever picked something, and hand those to Jev.

## Sources already ingested

Field Theory cache (`~/.fieldtheory/bookmarks/bookmarks.db`, 456 records, resynced 2026-09-27 14:30 UTC, no new bookmarks). Twelve records mention Jev; eight have local reading copies under `~/.fieldtheory/library/bookmarks/`. The curated set, dispositions and hashes live in `thoughtseed/factor/docs/research/2026-09-27-agent-knowledge/` (`source-index.md`, `sources.json`, `implementation-brief.md`, `primary-sources.md`, `sync-receipt.md`). Planning that already consumed them:

- `thoughtseed/factor/specs/003-jev/` and `skills/jev-gate/SKILL.md`: Jev picks, scores, or says yes/no from options that already exist in files; thresholds 0.5 (ask) and 0.85 (act); missing TypeSafe skill stops the card and prints `npx skills add typesafe-ai/skills --skill typesafe-ai`; Claude must never answer in Jev's place.
- `thoughtseed/factor/docs/research/2026-09-27-agent-knowledge/implementation-brief.md`: confidence is not permission (P0), Jev evaluated in shadow mode against a labeled set (P2), retained context bytes must match originals (P2).
- `tryambakam-noesis/urania-137` v0.7.1 (deployed, attested): phases A–E give a Workers client, engine-output validation (completeness noul, quality score, primary-kind choice, per-field nouls), Jev-assisted extraction for unknown engines, client quality tiers, and transit significance scoring. That client is the shape ported here.
- `thoughtseed-labs` skill-registry discovery log: "Jev stays a classify-vs-generate recommendation; no OmniRoute seat from this harvest."

## What was added to Selemene

- `packages/witness-pipeline/src/jev/client.ts`: System One client with injectable fetch, one retry on 429/5xx, `createJevClient` returning `null` without a key, `loadTypesafeKey` (env or `~/.claude/.env`, value never logged).
- `packages/witness-pipeline/src/jev/pass-gate.ts`: per-pass typed questions. `guardrail_clean` (noul: no prediction, diagnosis, guarantee), `relationship_framing_ok` (noul, only for declared relationship types; options come from the mode taxonomy), `fact_grounding` (score 0–3 against the engine facts block for every subject), `register_fit` (choice between the two register bands). Verdicts use exactly 0.5 and 0.85. Shadow mode records and lists disagreements with the regex rubric. Active mode blocks only on a confident safety failure and never approves anything.
- Orchestrator: optional `jevGate`; each `PassResult` gains `jev`, the output gains `jev_receipts`. `runFinalVerification` treats an active-mode block as a blocker.
- Runner flag `--jev off|shadow|active` on `scripts/business-partners-l2-runner.ts`; receipts land in `jev-receipts.json` and the summary.
- Tests: 13 (client shapes, retry, auth failure, thresholds, could-not-tell, shadow never blocks, active blocks only confident failures, malformed and transport errors recorded as error, orchestrator attaches one receipt per pass, off mode adds nothing).

## First live shadow run (2026-09-27, two-subject business-partners L2)

Probe: one `noul` question answered in 331 ms on `jev-1.13.0`. Full run: 4 passes judged, 0 skipped, 0 errors, 0 blocked, 274 to 386 ms per pass, about 310 input tokens each.

| Pass | guardrail_clean | framing_ok | grounding (score / conf) | register choice / conf | Jev vs rubric |
|---|---|---|---|---|---|
| opening | 0.57 could-not-tell | 0.95 pass | 2.96 / 0.96 pass | l4_l5 / 0.47 could-not-tell | agree |
| partnership-field | 0.36 fail | 0.96 pass | 2.98 / 0.98 pass | l4_l5 / 0.05 could-not-tell | guardrail: rubric pass, Jev fail |
| decision-dynamics | 0.53 could-not-tell | 0.96 pass | 2.93 / 0.93 pass | l1_l3 / 0.10 could-not-tell | agree |
| synthesis | 0.23 fail | 0.96 pass | 2.96 / 0.96 pass | l1_l3 / 0.41 could-not-tell | guardrail: rubric pass, Jev fail |

Readings of the receipts:

- Framing and grounding are confident and agree with the regex rubric: business-partner language held, and the same-day engine-facts injection shows up to Jev as dense grounding for both subjects.
- The guardrail question is where Jev earns its place. The regex rubric only knows four phrase lists and passed every section. Jev rated two sections as more predictive than descriptive, and inspection confirms it: "Success likely emerges through...", "ensure diverse approaches...", "creates potential for breakthrough innovations". The mode mandates descriptive witness only, so these are real drift, and the regex is the weaker gate.
- The register question is not usable as asked: confidence never cleared 0.5. The two criteria read too alike for a short section. Drop it or rewrite the options with concrete markers (jargon density, second-person address, shadow/gift vocabulary).


## Retry loop and all-modes matrix (2026-09-27, live)

Implemented: `register_fit` rewritten with concrete markers; `findPredictiveSentences` asks one `noul` per sentence (max 40) and returns those at or above 0.5; the orchestrator's `jevRetry.maxRetries` regenerates a pass whose guardrail or framing verdict is a confident fail, quoting the flagged sentences, and keeps the revision only when Jev's guardrail score rises without framing dropping. `scripts/mode-matrix-runner.ts` drives every mode from one manifest. One defect found and fixed during the run: an accepted retry stored its own receipt inside its history (JSON cycle); pinned by a test.

Every mode doc in `packages/witness-pipeline/modes/` was run live (OpenRouter `anthropic/claude-sonnet-4` narrative, Jev shadow, `jev_retry: 1` except L0) on real engine data from production: the two Thoughtseed co-founders; the operator solo; the operator's mother and father from the `Fam` fixture; and the humdes compatibility pairing for the two partner modes, run as mode-contract tests rather than readings about a declared relationship.

| Run | Passes | Words (target) | Register | Rubric w/f/l/g | Jev guardrail final | Retries acc/attempted | Notes |
|---|---|---|---|---|---|---|---|
| integrated-kundali-l0 | 12 | 13,601 (15–21k) | l4_l5 | 6/12/12/12 | 11 fail, 1 unsure (0.07–0.51) | 0/0 | forecasting by design; see below |
| business-partners-l2 (retry) | 4 | 2,898 (3.5–5.5k) | l1_l3 | 3/4/4/4 | all unsure (0.53–0.66) | 1/1 | no confident fail left |
| mother-son-lineage-l2 | 4 | 3,479 (4–6k) | l1_l3 | 4/4/4/4 | unsure (0.55–0.76) | 3/3 | strongest lift: 0.42→0.76 opening |
| family-penta-l2 | 5 | 4,626 (6–9k) | l1_l3 | 4/5/5/5 | 2 fail, 3 unsure | 1/2 | field-overview stuck at 0.28 |
| birth-blueprint-l1 | 2 | 1,422 (1.6–2.4k) | l1_l3 | 2/2/2/2 | 2 fail (0.48, 0.31) | 2/2 | "invitation" pass is inherently forward-looking |
| integrated-reading-l3 | 3 | 3,725 (4.2–5.8k) | l1_l3 | 2/3/3/3 | 1 unsure, 2 fail (0.48) | 1/2 | somatic pass: 0 sentences flagged yet fails |
| integrated-reading-l4 | 2 | 2,536 (4.8–6.5k) | l4_l5 | 1/2/2/2 | 1 fail (0.32) | 1/1 | short of target words |
| unmarried-partners-l2 (contract) | 4 | 2,770 (3.5–5.5k) | l1_l3 | 3/4/4/4 | all unsure (0.55–0.84) | 1/1 | |
| married-partners-l2 (contract) | 4 | 3,018 (3.5–5.5k) | l1_l3 | 3/4/4/4 | all unsure (0.63–0.84) | 2/2 | synthesis 0.27→0.69 |
| partner-synastry | – | – | – | – | – | – | mode doc uses the legacy `mode_id` frontmatter; `parseModeDoc` rejects it |

Business-partners, same subjects, before and after the retry loop (guardrail_clean):

| Pass | Run 2 (no retry) | Run 3 first draft | Run 3 final |
|---|---|---|---|
| opening | 0.57 | 0.66 | 0.66 |
| partnership-field | 0.36 fail | 0.38 fail | 0.53 (revision accepted) |
| decision-dynamics | 0.53 | 0.61 | 0.61 |
| synthesis | 0.23 fail | 0.54 | 0.54 |

Readings:

- Framing and grounding passed on every judged pass in every mode: 40 of 40. The engine-facts injection holds across solo, dyad and triad shapes; the Folio header appears in every relationship run.
- The retry loop works as a mirror, not a censor: 12 of 14 retried passes improved, 2 were rejected because the revision scored no better, and the reading kept the original. Jev never approved anything; it only ranked drafts.
- Guardrail scores cluster in the 0.5–0.7 band even after revision. Sonnet's default register for these modes is mildly modal ("suggests", "supports", "creates potential"). That is a prompt and template matter for the mode docs, and the receipts now give a per-sentence list to fix it against.
- The L0 kundali is a different contract. Jev judged its Master Timeline at 28 of 40 sentences predictive, with dated dasha forecasts; Wealth and Health at 9 and 7. The mode asks for exactly that. The guardrail question must be mode-aware (forecast allowed, promise forbidden) before Jev is used on L0.
- The rewritten register question now clears 0.85 on the L0 (nine of twelve passes) and on several dyad passes, but stays unsure on short sections. Usable, not yet decisive.
- Word counts run 10–45% under target across modes; the regex word-fit gate already shows it. Not a Jev finding.

Artifacts: `matrix-summary.{json,md}`, per-run `reading.md`, `result.json`, `jev-receipts.json` under the session scratchpad `run/matrix-live/`.


## Mode-aware guardrail, voice rules, lessons wiring, partner-synastry migration (2026-09-27, follow-up)

- `jev_guardrail: descriptive | forecast-allowed` is an optional mode-doc key (validated by the parser). `integrated-kundali-l0.md` declares `forecast-allowed`; every other mode stays `descriptive`. The gate's guardrail question and the per-sentence flagging clause both follow the policy: under `forecast-allowed`, dated periods described as tendencies or invitations are not flagged, while guarantees, certainties, diagnoses and promises still are.
- `VOICE_RULES` (per policy) are appended to every system prompt. The "avoid" list is the set of phrases Jev flagged in the matrix: "will", "likely", "success emerges", "ensures", "creates potential for", "positions for", "supports future", "over the coming years"; for timed readings also "will bring", "optimal conditions for", "major expansion".
- Each mode doc gained a dated lesson naming the phrases Jev flagged in that mode. Finding while wiring it: `{{lessons_summary}}` is declared by one template in ten, so lessons were reaching no prompt. The orchestrator now appends the lessons summary when a template lacks the placeholder, the same rule already used for engine facts.
- `partner-synastry.md` migrated from the legacy `mode_id` frontmatter to the mode-doc schema: four passes (opening, structural-compatibility, energetic-dance, synthesis), two `partner` roles, four-engine overlay weights, level-2 gate carried as a bridge mandate, framing decided by the caller's `relationship_context`. Every mode doc now parses; a test pins that.
- Tests: 133 total in witness-pipeline, 130 passing; the 3 failures are the pre-existing vault-path ones.


Live comparison after these changes (same subjects, same provider, Jev shadow):

| Run | Guardrail before (first draft) | Guardrail after (first draft) | Retries needed | Framing / grounding |
|---|---|---|---|---|
| business-partners L2 | 0.66 / 0.38 / 0.61 / 0.54 | 0.91 / 0.87 / 0.81 / 0.85 | 1 → 0 | 4/4, 4/4 |
| integrated-kundali L0 (12 passes) | 0.07–0.51, 11 fail | 0.72–0.90, 0 fail, 3 pass | 0 → 0 | 12/12, 12/12 |
| partner-synastry L2 (migrated, contract test) | did not parse | 0.93 / 0.85 / 0.85 / 0.77 | 0 | 4/4, 4/4 |

Readings: three of four business-partners passes now clear 0.85 on the first draft, so the retry loop did not fire; the voice rules and the lessons block did the work. The L0 kundali under `forecast-allowed` moved from eleven confident failures to none, with the Master Timeline and final synthesis the two lowest at 0.72, which is where the dated forecasts are densest. The migrated partner-synastry doc produced a four-pass dyad reading with the Folio header and grounding on both subjects. Register confidence also rose sharply once the drafts stopped hedging (business-partners 0.80–0.99). Word counts remain under target; that is the next template concern.

## What is deliberately not done

- Active mode was not switched on. Two confident guardrail failures in shadow are the calibration signal the factor brief asks for; a labeled set comes first.
- Jev does not replace the regex rubric yet. Shadow receipts are for calibration against it first.
- No engine-output validation port (urania phase B) yet; that is the natural next slice once the key exists, using the same client.

## Next steps

1. Make the guardrail question mode-aware: forecasts allowed where the mode declares them (L0 kundali, birth-blueprint invitation), promises and guarantees forbidden everywhere.
2. Tighten the mode templates against the flagged-sentence lists so first drafts land above 0.85 instead of relying on the retry.
3. Migrate `partner-synastry.md` to the current frontmatter schema or retire it.
4. Label a small set of passes, compare Jev verdicts with the rubric, then consider `--jev active` for guardrail and framing only.
5. Port urania's engine-output validation questions to gate `engineResultsBySubject` before a reading starts.
