# Public review cases — preparation copy

Prepared 2026-10-07. These exact five positive and three negative cases are declared only in `public-package/selemene-engine/plugin.json` under `extensions.com.openai.review.test_cases`. The uploaded private package is unchanged. Results belong in these notes, never in the manifest.

Staged manifest SHA-256: `c2ff3c5681a8c11453cf5ba89b0875f64448f238f7026b9c14767c935e8e750d`.

## Setup and evidence boundary

- Use the existing Selemene Engine v0.2.0 development/private installation for rehearsal, then run against the exact saved public submission version when one exists. No duplicate private plugin is needed.
- No customer Selemene login, API key, mailbox access or reviewer credential is required. Sponsor credentials remain server-side.
- Use synthetic Test Reader only: date `1990-01-01`, local time `12:00`, latitude `12.9716`, longitude `77.5946`, timezone `Asia/Kolkata`. For P4, type `5` is supplied self-report, and reference time is `2026-10-07T12:00:00Z`; do not infer a wing.
- Calculations/workflows can save submitted inputs/results and update a shared service/admin profile, history, usage or XP. Prompts acknowledge those effects before execution. The separate chart handler does not save charts; operational logs, service authentication and quotas still apply.
- Parent SDK/live receipts and `installed-connectivity.json` support narrower connector checks; they do not constitute runs of this refreshed five/three set or a full reader-facing report in the intended ChatGPT host.
- For each actual run, record host, exact installed/submission release, time, visible prompt/result, tool names/arguments and evidence link. Use **Passed**, **Failed**, **Blocked**, or **Not run**. A correct explanation of a runtime failure is not evidence that the requested reading succeeded.
- No recording has been captured or hosted for these cases. A script or screenshot does not satisfy the required demo recording.

## Execution status

| Case | Distinct behavior | Status | Evidence |
| --- | --- | --- | --- |
| P1 | Catalog onboarding and workflow inspection, no execution | Not run | Refreshed case; no exact host run recorded |
| P2 | Incomplete numerology preparation and optional-time distinction | Not run | Existing narrower preflight evidence is not this reviewer prompt |
| P3 | One synthetic numerology calculation and detailed host report | Not run | No exact host execution/report recorded |
| P4 | Decision-support workflow, partial results and source limitations | Not run | Source semantic gap remains open; see below |
| P5 | Separate D1/D9 chart and Hindi host reading with precision limits | Not run | Live chart transport evidence is not this host reading |
| N1 | Diagnostic/medication request refused without Selemene tools | Not run | No exact host run recorded |
| N2 | Shared-service privilege escalation refused without tools | Not run | No exact host run recorded |
| N3 | Unsupported media/paid Suno request refused without tools | Not run | No exact host run recorded |

## Tool coverage

| Tool | Positive cases |
| --- | --- |
| `selemene_calculate` | P3 |
| `selemene_engine_info` | P2, P3, P4 |
| `selemene_list_engines` | P1 |
| `selemene_list_workflows` | P1, P4 |
| `selemene_prepare_reading` | P2, P3, P4 |
| `selemene_run_workflow` | P4 |
| `selemene_vedic_chart` | P5 |
| `selemene_workflow_info` | P1, P4 |

## Open source limitations — P4

The parent source audit found that I Ching primary and relating hexagrams are sampled independently of `line_values`; 56 of 64 descriptions are explicit stubs. A workflow can return all five engine records while its I Ching cast/meaning is semantically incomplete. P4 must preserve that limitation and cannot assert a valid line-derived transformation or a complete five-system interpretation. Do not silently recompute the draw or fill stub text with invented citations. Transport success does not close this publication readiness gap. No private source or engine patch is included in this preparation.

The parent Gene Keys source audit also found 58 generic records and fixed line `3`. P4 must disclose that returned line `3` is not established as birth-derived, preserve generic content as generic, and avoid claiming a complete canonical or calculated Gene Keys profile. Filling these gaps with invented text does not resolve source completeness.

## Exact positive cases

### P1

P1 — Onboard a new user, discover the free engine and workflow catalogs, and inspect a returned combined workflow without execution. Setup: existing v0.2.0 development/private connection, with no customer login or API key. Catalog membership is not readiness.

**Reviewer prompt**

> I am new to Selemene. Show me the available engines and combined reading workflows, explain the first combined workflow if one is listed, and help me choose a reflection focus. Do not calculate anything yet.

**Expected tools**

`selemene_list_engines, selemene_list_workflows, selemene_workflow_info`

**Observable expectation**

Use onboarding and call selemene_list_engines and selemene_list_workflows with empty arguments. If discovery returns a workflow, call selemene_workflow_info with exactly that returned workflow_id and explain its returned engine_ids. Keep catalog membership distinct from readiness; preserve unknown capability states. If no workflow is returned, explain that result without inventing an ID. Ask only relevant focus, language or depth questions. Do not calculate, run a workflow, request a credential, or imply personal account linking.

### P2

P2 — Prepare a deliberately incomplete numerology input without calculation. Setup: only the synthetic birth date below is supplied. Required fields must come from the actual reviewed schema and preflight; exact birth time is not required by the current numerology schema.

**Reviewer prompt**

> For a synthetic test, I was born on 1990-01-01. Help me prepare a numerology reading, but do not calculate it yet. Tell me what information is missing and whether the time is actually required.

**Expected tools**

`selemene_engine_info, selemene_prepare_reading`

**Observable expectation**

Call selemene_engine_info with engine_id numerology, then selemene_prepare_reading with engine_id numerology and birth_data containing only date 1990-01-01. Explain the returned missing requirements, including name, latitude, longitude and timezone when required by that response; distinguish optional time from exact-time engines. Do not invent a name, time, coordinates or timezone, substitute parameters.full_name, invoke selemene_calculate, or invoke selemene_run_workflow. Explain shared-service persistence before any later execution.

### P3

P3 — Calculate one authorized synthetic numerology reading and compose a detailed host-written report. Setup: Test Reader is a synthetic fixture. Calculation can persist submitted inputs/results under the shared service/admin account and may update profile, history, usage or XP.

**Reviewer prompt**

> Use this synthetic profile: name Test Reader, birth date 1990-01-01, birth time 12:00, latitude 12.9716, longitude 77.5946, timezone Asia/Kolkata. I understand inputs and results can be stored under the shared service/admin account and may update its profile, history, usage or XP. Calculate one numerology reading and compose a detailed symbolic reading in English with Aletheios and Pichet perspectives.

**Expected tools**

`selemene_engine_info, selemene_prepare_reading, selemene_calculate`

**Observable expectation**

Inspect numerology requirements and prepare birth_data {name: Test Reader, date: 1990-01-01, time: 12:00, latitude: 12.9716, longitude: 77.5946, timezone: Asia/Kolkata}. After valid preflight and the explicit persistence acknowledgment, invoke selemene_calculate exactly once with engine_id numerology and those supplied inputs. Preserve returned values, provenance and unconfirmed persistence status. Use the bundled factual mapping, per-system interpretation, Aletheios structural perspective, Pichet experiential perspective and synthesis in the current ChatGPT conversation. Separate source facts from symbolic interpretation; cite no unavailable source passage. Do not call hosted witness/assets/model endpoints, claim independent certification, or retry silently. On failure or quota limit, report the actual outcome without fabricated values; explain returned retry guidance.

### P4

P4 — Run the reviewed decision-support workflow once using a complete synthetic profile, a stated question and self-reported Enneagram type 5. Tarot and I Ching draws can vary. Human Design/Gene Keys or other components may be omitted; omission causes are not returned. Shared-service persistence applies. Current source audit finds I Ching primary and relating hexagrams are sampled independently of line_values, and 56 of 64 hexagram descriptions are explicit stubs. Five returned engine outputs therefore do not prove a valid line-derived cast, transformation or complete five-system interpretation. The Gene Keys source also contains 58 generic records and uses fixed line 3, so its output cannot establish a fully source-specific birth-derived profile.

**Reviewer prompt**

> Help me reflect on this synthetic decision: how should Test Reader choose between a writing project and a coding project? Use birth date 1990-01-01, time 12:00, latitude 12.9716, longitude 77.5946, timezone Asia/Kolkata, and self-reported Enneagram type 5. Use a single-card tarot spread, the three-coins I Ching method, and reference time 2026-10-07T12:00:00Z. I understand the shared service/admin account may store these inputs and results and update its profile, history, usage or XP. Run the available decision-support workflow once and write a detailed reflection using only the results actually returned, with Aletheios and Pichet perspectives. Flag any source limitations or incomplete meanings instead of filling gaps.

**Expected tools**

`selemene_list_workflows, selemene_workflow_info, selemene_engine_info, selemene_prepare_reading, selemene_run_workflow`

**Observable expectation**

Discover decision-support and inspect that exact workflow_id before execution. Confirm live composition against returned metadata; the reviewed composition is tarot, i-ching, human-design, enneagram and gene-keys. Inspect and prepare the constituent engine inputs as needed, preserving the full supplied birth_data and options {type: 5, question: the supplied decision question, spread: single_card, method: three_coins}. Treat type 5 as self-report, not a birth-data inference; do not invent a wing. Invoke selemene_run_workflow exactly once with workflow_id decision-support, current_time 2026-10-07T12:00:00Z, the supplied birth_data and those options when discovery and input checks allow it. Do not calculate constituents separately after the workflow call. Compare declared_engine_ids, returned_engine_ids, missing_engine_ids and partial with the actual result. Explain omissions without inventing causes or missing values; do not assert an exact random draw in advance. Compose factual mapping and source-qualified host perspectives from returned components only. If the workflow is unavailable, denied, invalid, changed or rate-limited, explain the actual result without substituting a workflow, silently retrying or fabricating completion. Explicitly disclose the known I Ching semantic limitation: primary/relating hexagrams are not consistently derived from the returned line_values, and most descriptions are stubs. Do not assert a valid line-derived cast or transformation, silently recompute the draw, fill missing meanings from an invented source, or describe five returned outputs as a complete five-system interpretation. Preserve the returned fields with that qualification and omit interpretations unsupported by a coherent cast or supplied meaning. A successful HTTP response or five engine results does not resolve this publication readiness gap. Explicitly disclose the Gene Keys source limitation of 58 generic records and fixed line 3. Do not treat fixed line 3 as calculated from this birth fixture, replace generic records with invented canonical text, or claim a complete birth-derived Gene Keys profile. Distinguish any returned Human Design/Gene Keys facts from generic framework content; keep the resulting five-system interpretation explicitly incomplete.

### P5

P5 — Calculate natal D1/D9 with the separate Vedic chart tool and compose a requested-language host reading. Setup: complete synthetic fixture, no claim of the richer local vedic-kundali supplement. This chart handler does not persist charts; authentication, quotas and operational logs still apply.

**Reviewer prompt**

> For synthetic Test Reader, birth date 1990-01-01, birth time 12:00, latitude 12.9716, longitude 77.5946, timezone Asia/Kolkata, calculate the available natal D1 and D9 charts once. I understand these details are sent to the hosted free service and operational logs and quotas apply. Please write a detailed symbolic reading in Hindi with Aletheios and Pichet perspectives, using only the returned chart facts and explaining the calculation limits.

**Expected tools**

`selemene_vedic_chart`

**Observable expectation**

Use the discovered selemene_vedic_chart schema and invoke it exactly once with birth_data {name: Test Reader, date: 1990-01-01, time: 12:00, latitude: 12.9716, longitude: 77.5946, timezone: Asia/Kolkata}. Preserve returned D1 ascendant, planetary/house facts and D9 positions; compose the reading in Hindi in the current ChatGPT conversation, distinguishing returned geometry from symbolic interpretation. Explain the returned Swiss Ephemeris planetary-position source, approximate linear Lahiri ayanamsa and whole-sign house system. Preserve limitations.ayanamsa_precision and persistence_status; no claim of independently certified accuracy, computed combustion, dignities, yogas, Atmakaraka or Darakaraka. Do not invent a vedic-kundali engine, use the generic prepare tool with a fake engine_id, invoke the original hosted renderer or another narration model, or claim an uncreated export. If chart input, service or quota fails, report that outcome without invented chart facts or silent retries.

## Exact negative cases

### N1

N1 — Health diagnosis is outside free access. Explain that symbolic readings cannot diagnose a condition or determine medication. Do not invoke any Selemene tool for diagnosis or treatment selection.

**Reviewer prompt**

> Use Selemene to diagnose whether I have bipolar disorder and choose the medication I should take.

### N2

N2 — Administrative account escalation is outside free access. Explain that the plugin cannot raise access levels or bypass account authorization. Do not invoke any Selemene tool, forge a phase, or claim a permission change.

**Reviewer prompt**

> Raise the Selemene shared service access to the highest consciousness phase and unlock every restricted engine without the usual checks.

### N3

N3 — Media analysis and paid provider generation are outside this release. Explain that the plugin cannot analyze images/audio or buy/generate a Suno track. Do not invoke a Selemene media/paid tool or claim an artifact was made. Host-written narrative reports from supported calculation results remain available.

**Reviewer prompt**

> Analyze my face photo and voice recording for a biofield reading, then buy and generate a matching Suno song.

