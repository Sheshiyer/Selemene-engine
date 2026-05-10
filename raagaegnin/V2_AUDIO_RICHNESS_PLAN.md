# Nādashakti V2 — Audio Richness Plan

**Document type:** Swarm Architect phase→wave→swarm plan
**Companion:** `RAAGA_ENGINE.md`, `SHRUTI_THEORY.md`
**Author:** Claude (orchestrator), 2026-05-09
**Predecessor:** v1 — sine-wave just-intonation playback (shipped, verified end-to-end)
**Successor:** v2 — sonically rich, ritually-aware raga engine

---

## 1. Discovery Summary

| Field | Decision |
|---|---|
| Planning depth | **deeply detailed** (user invoked swarm-architect explicitly) |
| Delivery mode | **prototype → production** transition (v1 already ships in Selemene) |
| Release model | **phased rollout** — three waves per phase, swappable behind a feature flag |
| Quality bar | typecheck (strict TS) + browser audio smoke-tests + offline-render fidelity check |
| Team/agent topology | **solo-with-swarm**: Claude as planner/orchestrator, parallel sub-agents per swarm via the `dispatching-parallel-agents` skill |
| Constraints | Strudel `@strudel/web@1.3.0` is the audio engine (we just verified no `xen()` global; we use absolute Hz via `freq()`). All 22-shruti just-intonation precision must be preserved end-to-end. |

## 2. Assumptions and Constraints

### Assumptions
- **A1.** Strudel's `samples()` API can load arbitrary sample packs from CDN URL or `strudel.json` manifest.
- **A2.** `OfflineAudioContext` can capture Strudel output for server-side rendering (no Strudel-native WAV export yet — upstream PR #1414 still WIP).
- **A3.** Gamakas can be approximated via `penv()` pitch envelopes + `pattack`/`pdecay` controls without writing a custom DSP module.
- **A4.** The Selemene Rust engine emits melakarta metadata only; all audio synthesis remains browser-side.

### Constraints
- **C1.** No 12-TET fallback path — every swara must hit its just-intonation Hz exactly.
- **C2.** Audio must initialize from a user gesture (browser policy); no auto-play.
- **C3.** Existing v1 callsites in [Nadabrahman.tsx](Selemene-engine/apps/noesis-web/src/components/engines/Nadabrahman.tsx), [strudel-demo.html](raagaegnin/strudel-demo.html), [melakarta_body_map.html](raagaegnin/melakarta_body_map.html) must keep working — v2 is additive, not breaking.
- **C4.** Bundle size: avoid pulling in heavyweight DSP libs; prefer Strudel-native primitives.

## 3. Agent Ownership Model

| Concern | Primary owner | Secondary | Notes |
|---|---|---|---|
| Planning / orchestration | Claude (planner) | Human lead | This document is the contract |
| Audio DSL & gamaka grammar | `gamaka-engineer` sub-agent | Claude | Pure TS, no UI |
| Sample manifest & timbre | `timbre-curator` sub-agent | Claude | Asset selection + manifest |
| Tala / breath wiring | `tala-engineer` sub-agent | Claude | Reads `RAAGA_ENGINE.md` chakra metadata |
| Server-render pipeline | `render-engineer` sub-agent | Claude | OfflineAudioContext + WAV encoder |
| UI integration | `ui-integrator` sub-agent | Claude | Wires player options into existing pages |
| Validation | `qa-validator` sub-agent | Claude | Browser tests + ear-test scripts |

Sub-agents are spawned via `Agent` tool with isolated worktrees per swarm where work overlaps shared files (e.g., `RaagaPlayer.ts`).

## 4. Phase Map

### Phase 1 — Contracts & Primitives (foundation)
- **Goal:** Freeze types/APIs for gamakas, tala, timbres, render — so all swarms can build in parallel without merge collisions.
- **Exit criteria:** Type contracts published; sample manifest schema agreed; `RaagaPlayer` interface extended (additively); zero v1 regressions; foundation tests green.
- **Waves:** 1.1 Audio DSL freeze · 1.2 Sample manifest & CDN · 1.3 Tala/breath data model

### Phase 2 — Parallel Feature Implementation
- **Goal:** All four feature areas built simultaneously against the frozen contracts.
- **Exit criteria:** Each feature passes its own swarm-level acceptance; integration smoke-tests green per wave; no contract drift.
- **Waves:** 2.1 Gamakas & Timbres · 2.2 Tala & Breath-tempo · 2.3 Server-side render pipeline

### Phase 3 — Integration, Polish & Verification
- **Goal:** End-to-end ritual experience: click body zone → hear breath-paced raga with gamakas, sitar timbre, optional WAV download.
- **Exit criteria:** All five v2 features composable on a single playback; offline render fidelity ≥ 99% match to live; docs updated; UAT recorded.
- **Waves:** 3.1 Integration smoke · 3.2 Render fidelity · 3.3 Docs + UAT

---

## 5. Detailed Phase 1 Wave Layout

### Wave 1.1 — Audio DSL Freeze
**Goal:** Lock the TypeScript types every swarm will consume so 2.x swarms can run independently.

#### Swarm A — Gamaka grammar
- Owner: `gamaka-engineer`
- Inputs: `RAAGA_ENGINE.md` therapeutic taxonomy; Strudel `penv` API
- Outputs: `Gamaka.ts` discriminated union (`kampita`, `andolana`, `kurula`, `nokku`, `sphurita`); `applyGamaka(pattern, gamaka)` signature
- Validation: TS compiles; spec unit-tests for each gamaka shape

#### Swarm B — Player options surface
- Owner: `ui-integrator`
- Inputs: existing `RaagaPlayer.play()` signature
- Outputs: extended `PlayOptions` (timbre, gamaka, tala, breath, render); v1 callsites unaffected
- Validation: existing browser smoke tests still pass with no opt-in

### Wave 1.2 — Sample Manifest & CDN
**Goal:** Decide where Indian-classical samples live and how they load.

#### Swarm A — Sample manifest schema
- Owner: `timbre-curator`
- Inputs: Strudel `samples()` API, VCSL catalog, dirt-samples
- Outputs: `samples-manifest.json` with sitar/tanpura/mridangam/bansuri/sarangi entries; CDN strategy doc (jsdelivr from a vault repo)
- Validation: `samples('https://...manifest.json')` resolves all entries in dev REPL

#### Swarm B — License & attribution audit
- Owner: `timbre-curator`
- Inputs: each sample source's license
- Outputs: `SAMPLES_ATTRIBUTION.md`; only AGPL/CC0/CC-BY entries shipped
- Validation: zero proprietary samples in manifest

### Wave 1.3 — Tala & Breath Data Model
**Goal:** Codify rhythmic + breath structures consumed by Phase 2.

#### Swarm A — Tala primitives
- Owner: `tala-engineer`
- Inputs: classical Carnatic talas (Adi 8, Rupakam 7, Misra Chapu 3.5, Khanda Chapu 5)
- Outputs: `talas.ts` with each as `{ beats, structure, euclid }`; mapping to Strudel `euclid()` / `every()`
- Validation: each tala plays correctly via metronome test

#### Swarm B — Breath-tempo extraction
- Owner: `tala-engineer`
- Inputs: `RAAGA_ENGINE.md` chakra `breath` field (4-4-4-4, Bhastrika, Brahmari…)
- Outputs: `breaths.ts` mapping breath name → `{ cps, articulation, durationCycle }`
- Validation: 12 breaths × correct cps; no nulls

---

## 6. Phase 2 Wave Layout (overview)

### Wave 2.1 — Gamakas & Timbres
- **Swarm A:** Gamaka envelope library (`pkampita.ts`, `pandolana.ts`, …)
- **Swarm B:** Sample-pack loader + bank chooser (default sitar; sarangi/bansuri/tanpura optional)
- **Swarm C:** ADSR per-swara articulation (pluck vs gamak vs sustain)

### Wave 2.2 — Tala & Breath-tempo
- **Swarm A:** Tala-aware pattern emitter (raga arohana laid into tala beats with samam emphasis)
- **Swarm B:** Breath-driven tempo modulation (chakra dictates cps + cycle length)
- **Swarm C:** Tanpura drone layer (Sa-Pa-Sa low-frequency bed under the raga)

### Wave 2.3 — Server-side Render
- **Swarm A:** OfflineAudioContext capture wrapper around Strudel
- **Swarm B:** PCM → WAV encoder (16-bit/48kHz)
- **Swarm C:** Download endpoint + signed URL pattern for noesis-web `/api/raaga/render`

---

## 7. Full Task List (78 tasks)

> Schema per [`task-schema.json`](https://… swarm-architect): `{id, title, area, owner_role, est_hours, dependencies, deliverable, acceptance, validation, phase, wave, swarm}`. Compacted as a table for readability.

### Phase 1 — Contracts & Primitives

| ID | Title | Area | Owner | Hrs | Deps | Deliverable | Acceptance | Validation |
|---|---|---|---|---|---|---|---|---|
| T-001 | Define `Gamaka` discriminated union | frontend | gamaka-engineer | 1 | — | `lib/raaga/gamakas/types.ts` | Covers 5 named gamakas + `none` | TS compiles; exhaustiveness check passes |
| T-002 | Spec `applyGamaka()` signature | frontend | gamaka-engineer | 1 | T-001 | typedef in `gamakas/index.ts` | Signature: `(pattern, gamaka, opts) => pattern` | Lint pass |
| T-003 | Document each gamaka mathematically | product | gamaka-engineer | 2 | T-001 | `docs/gamakas.md` | Pitch curve eqn + cents range per gamaka | Cross-ref Sangita Ratnakara IV |
| T-004 | Extend `PlayOptions` interface (additive) | frontend | ui-integrator | 1 | — | RaagaPlayer.ts patch | All new fields optional | v1 callers unchanged |
| T-005 | Add v2 feature flag `RAAGA_V2_ENABLED` | frontend | ui-integrator | 0.5 | T-004 | env wiring in noesis-web | Default `false` | Build green with flag off |
| T-006 | Sample manifest JSON schema | product | timbre-curator | 1 | — | `samples-manifest.schema.json` | JSON Schema draft 2020-12 | `ajv validate` passes on stub |
| T-007 | Curate sitar sample pack | data | timbre-curator | 3 | T-006 | manifest entry `sitar` (3+ velocity layers) | CC-BY/CC0 only | License doc per sample |
| T-008 | Curate tanpura drone sample | data | timbre-curator | 1 | T-006 | manifest entry `tanpura-c` (Sa drone, 4 strings) | Loops cleanly | No clicks at loop point |
| T-009 | Curate mridangam pack | data | timbre-curator | 2 | T-006 | manifest entry `mridangam` (tha, dhi, ki, ta) | 4 strikes | Spectrogram check |
| T-010 | Curate bansuri/sarangi alts | data | timbre-curator | 2 | T-006 | manifest entries `bansuri`, `sarangi` | Optional, loadable | Smoke test |
| T-011 | Publish manifest to CDN | infra | timbre-curator | 1 | T-007..T-010 | `samples-manifest.json` on jsdelivr | Reachable from any origin | curl 200 |
| T-012 | License attribution doc | product | timbre-curator | 1 | T-007..T-010 | `SAMPLES_ATTRIBUTION.md` | All sources cited | License compatibility table |
| T-013 | Define `Tala` interface | frontend | tala-engineer | 1 | — | `talas/types.ts` | `{ name, beats, structure, euclid }` | TS compiles |
| T-014 | Encode 6 canonical talas | frontend | tala-engineer | 2 | T-013 | `talas/data.ts` (Adi, Rupakam, Misra/Khanda Chapu, Tisra, Jhampa) | All 6 entries | Beat counts verified |
| T-015 | Map talas → Strudel `euclid()` | frontend | tala-engineer | 1 | T-014 | helper in `talas/strudel.ts` | Each tala → working pattern | Metronome test plays |
| T-016 | Extract breath table from RAAGA_ENGINE.md | data | tala-engineer | 1 | — | `breaths/data.ts` (12 entries) | All breaths from chakra map | Snapshot test |
| T-017 | Map breaths → cps & articulation | frontend | tala-engineer | 2 | T-016 | `breaths/strudel.ts` | Each breath → `{ cps, attack, sustain }` | Audible difference between Bhastrika / Brahmari |
| T-018 | Phase-1 contract review checkpoint | qa | qa-validator | 1 | T-001..T-017 | review report | All types frozen, no breaking changes | Sign-off in `.context/v2-contracts.md` |

### Phase 2 — Parallel Implementation

#### Wave 2.1 — Gamakas & Timbres

| ID | Title | Area | Owner | Hrs | Deps | Deliverable | Acceptance | Validation |
|---|---|---|---|---|---|---|---|---|
| T-019 | Implement `kampita` (light vibrato) | frontend | gamaka-engineer | 2 | T-002 | `gamakas/kampita.ts` | ±20 cents, 5–7 Hz LFO | Pitch tracker stays in band |
| T-020 | Implement `andolana` (slow oscillation) | frontend | gamaka-engineer | 2 | T-002 | `gamakas/andolana.ts` | ±50 cents, 1–2 Hz | Visual envelope check |
| T-021 | Implement `kurula` (curved slide) | frontend | gamaka-engineer | 2 | T-002 | `gamakas/kurula.ts` | Glide between adjacent shrutis | A→B traversal smooth |
| T-022 | Implement `nokku` (grace ahead) | frontend | gamaka-engineer | 2 | T-002 | `gamakas/nokku.ts` | Grace note 1 shruti above target | Heard before main note |
| T-023 | Implement `sphurita` (bent attack) | frontend | gamaka-engineer | 2 | T-002 | `gamakas/sphurita.ts` | Quick rise from below | Attack < 50ms |
| T-024 | Compose gamakas via `penv` chains | frontend | gamaka-engineer | 3 | T-019..T-023 | `applyGamaka()` real impl | Returns Strudel pattern | `evaluate()` runs without err |
| T-025 | Per-swara gamaka annotations for Mayamalavagaula | frontend | gamaka-engineer | 1 | T-024 | preset in `presets/raga15.ts` | Canonical Carnatic ornamentation | Ear-test against reference |
| T-026 | Sample loader: `loadRaagaSamples()` | frontend | timbre-curator | 2 | T-011 | `lib/raaga/samples.ts` | Calls Strudel `samples(url)` | All packs reachable post-load |
| T-027 | Bank selector in PlayOptions | frontend | timbre-curator | 1 | T-004, T-026 | `timbre: 'sine'\|'sitar'\|...` | Default `sitar` when v2 on | Default heard |
| T-028 | ADSR profiles per timbre | frontend | timbre-curator | 2 | T-027 | `timbres/adsr.ts` | sitar=pluck, bansuri=breath | Audible attack difference |
| T-029 | LPF ladder for sarangi warmth | frontend | timbre-curator | 1 | T-027 | filter chain in `timbres/sarangi.ts` | 800Hz LPF + Q=2 | Spectrogram check |
| T-030 | Smoke: play raga #15 with sitar+kampita | qa | qa-validator | 1 | T-024, T-027 | screen-recorded clip | 8s output, valid | preview_console_logs clean |
| T-031 | Smoke: gamaka catalog playback | qa | qa-validator | 1 | T-024 | demo page `gamakas-demo.html` | All 5 audible | Browser test passes |

#### Wave 2.2 — Tala & Breath-tempo

| ID | Title | Area | Owner | Hrs | Deps | Deliverable | Acceptance | Validation |
|---|---|---|---|---|---|---|---|---|
| T-032 | Tala-aware pattern emitter | frontend | tala-engineer | 3 | T-015 | `talaize(swaras, tala)` | Lays N swaras into tala beats | Output beats == tala.beats |
| T-033 | Samam (downbeat) emphasis on Sa | frontend | tala-engineer | 2 | T-032 | gain modulation on beat 0 | +6dB on samam | Visible in waveform |
| T-034 | Edam / anti-samam accent | frontend | tala-engineer | 1 | T-033 | secondary accent | +3dB on edam | Heard |
| T-035 | Tisra-Adi (3-3-2 grouping) | frontend | tala-engineer | 1 | T-032 | preset | Grouping reflected in accents | Listen test |
| T-036 | Breath modulation: cps from chakra | frontend | tala-engineer | 2 | T-017 | `applyBreath(pattern, breath)` | Pattern plays at chakra-matched cps | Bhastrika fast, Śītalī slow |
| T-037 | Breath modulation: articulation | frontend | tala-engineer | 2 | T-036 | attack/release per breath | Brahmari = sustained hum | Audible distinction |
| T-038 | Tanpura drone layer | frontend | tala-engineer | 3 | T-026 | `playTanpura(rootHz)` | Sa-Pa-Sa-Sa cycle, slow | Layered under raga |
| T-039 | Tanpura auto-tunes to raga root | frontend | tala-engineer | 1 | T-038 | reads PlayOptions.rootHz | Drone follows root | A/B test |
| T-040 | Compose: raga + tala + breath + drone | frontend | ui-integrator | 2 | T-032..T-039 | `playRaagaFull()` | One call, all features | Manual playback |
| T-041 | Smoke: full Mayamalavagaula at 5AM breath | qa | qa-validator | 1 | T-040 | clip + spectrogram | 30s, all layers present | preview_screenshot |
| T-042 | Smoke: 6 chakras × 6 ragas × correct breath | qa | qa-validator | 2 | T-040 | matrix test report | 36 combinations | All play without error |

#### Wave 2.3 — Server-side Render

| ID | Title | Area | Owner | Hrs | Deps | Deliverable | Acceptance | Validation |
|---|---|---|---|---|---|---|---|---|
| T-043 | OfflineAudioContext wrapper for Strudel | frontend | render-engineer | 4 | T-040 | `RaagaRenderer.ts` | Renders N seconds offline | OfflineAC.startRendering resolves |
| T-044 | PCM → WAV encoder (16-bit, 48kHz) | frontend | render-engineer | 2 | T-043 | `wavEncoder.ts` | Float32 → WAV blob | File plays in QuickTime |
| T-045 | Render request → Blob URL | frontend | render-engineer | 1 | T-044 | `renderToBlob(opts)` | Returns downloadable URL | Click-to-download works |
| T-046 | UI: "⬇ Download WAV" button | frontend | ui-integrator | 1 | T-045 | button on raga card | Triggers render + download | File saved |
| T-047 | Server-side render service (Node) | backend | render-engineer | 4 | T-044 | `apps/raaga-render/` Node service | Headless Chromium + Strudel | Same WAV as browser |
| T-048 | `/api/raaga/render` Next route | backend | render-engineer | 2 | T-047 | route in `noesis-web/app/api/...` | POST raga#, returns wav URL | curl roundtrip |
| T-049 | Caching layer (Redis or filesystem) | backend | render-engineer | 2 | T-048 | LRU 100 renders | Same params → cached hit | Second call < 50ms |
| T-050 | Signed URLs for renders | backend | render-engineer | 1 | T-049 | HMAC token | 5-min expiry | Tamper test rejects |
| T-051 | Smoke: render #15 → diff against live | qa | qa-validator | 2 | T-046, T-048 | comparison report | RMSE < 0.01 between live + offline | Audacity diff |

### Phase 3 — Integration & Verification

| ID | Title | Area | Owner | Hrs | Deps | Deliverable | Acceptance | Validation |
|---|---|---|---|---|---|---|---|---|
| T-052 | Wire v2 into Nadabrahman.tsx | frontend | ui-integrator | 2 | T-040, T-046 | timbre+gamaka selectors in UI | Toggleable per recommendation | Renders |
| T-053 | Wire v2 into melakarta_body_map.html | frontend | ui-integrator | 2 | T-040 | same controls in standalone | Parity with TSX path | Both UIs identical |
| T-054 | Wire v2 into strudel-demo.html | frontend | ui-integrator | 1 | T-040 | preset library dropdown | 6 presets | Listen test |
| T-055 | Feature flag rollout: 10% → 100% | infra | ui-integrator | 1 | T-052..T-054 | env var + canary | A/B harness in noesis-web | Telemetry shows opt-in |
| T-056 | Cross-browser test (Chrome/Safari/FF) | qa | qa-validator | 2 | T-052 | matrix in CI | Audio plays in all 3 | Headless Chromium snapshot |
| T-057 | Latency budget verification | qa | qa-validator | 1 | T-052 | ms per step | First-play < 4s | preview_eval timing |
| T-058 | Render fidelity check vs reference | qa | qa-validator | 3 | T-051 | report comparing each of 72 ragas | Pitch tracker tolerance ±2 cents | All 72 within tol |
| T-059 | Pitch tracker audit (verify shrutis correct in audio) | qa | qa-validator | 3 | T-058 | YIN/CREPE pitch detection on rendered WAVs | Each swara within ±5¢ of expected | Per-raga histogram |
| T-060 | Vivadi-pair audibility test | qa | qa-validator | 1 | T-059 | A/B blind test plan | R3 vs G1 (same shruti #5) sound identical only when same gamaka applied | Test recording |
| T-061 | Update RAAGA_ENGINE.md with v2 section | product | claude | 1 | T-052 | new section | All 5 features documented | Renders cleanly |
| T-062 | Update SHRUTI_THEORY.md with gamaka math | product | gamaka-engineer | 1 | T-024 | new section | Cites Sangita Ratnakara | Cross-link works |
| T-063 | Write `V2_USAGE.md` consumer guide | product | ui-integrator | 1 | T-052 | examples + screenshots | Covers all PlayOptions | Skim test |
| T-064 | Add SAMPLES_ATTRIBUTION.md to repo | product | timbre-curator | 0.5 | T-012 | shipped doc | All licenses listed | License linter |
| T-065 | Update verify.mjs with gamaka assertions | qa | qa-validator | 1 | T-024 | test extension | New gamaka math verified | Node run passes |
| T-066 | Update verify.mjs with tala assertions | qa | qa-validator | 1 | T-015 | test extension | All 6 talas verified | Node run passes |
| T-067 | Browser smoke test suite (Playwright) | qa | qa-validator | 3 | T-052 | `tests/browser/raaga.spec.ts` | 10 scenarios | All green in CI |
| T-068 | Performance: bundle size delta | qa | qa-validator | 1 | T-052 | webpack-bundle-analyzer report | < 500KB v1→v2 delta | Within budget |
| T-069 | Memory: no leaks across plays | qa | qa-validator | 1 | T-052 | DevTools heap snapshot | Stable after 100 plays | Snapshots match |
| T-070 | A11y: keyboard play / stop | frontend | ui-integrator | 1 | T-052 | tab + space triggers | All controls keyboardable | axe-core pass |
| T-071 | UAT recording: 5-min ritual demo | qa | qa-validator | 2 | T-052..T-058 | screen+audio capture | One full meditation cycle | Posted to docs |
| T-072 | Risk: Strudel API churn | infra | claude | 0.5 | — | pinned version + LICENSE | `@strudel/web@1.3.0` locked | package-lock.json |
| T-073 | Risk: sample CDN downtime | infra | timbre-curator | 1 | T-011 | fallback CDN | jsdelivr + unpkg + GH raw | Failover test |
| T-074 | Risk: AudioContext mute on idle tab | frontend | ui-integrator | 1 | T-052 | auto-resume on focus | Audio resumes < 100ms | Tab switch test |
| T-075 | Telemetry: anonymized play counts per raga | infra | claude | 1 | T-055 | event stream | Privacy-preserving | Verified no PII |
| T-076 | Rollback runbook | infra | claude | 0.5 | T-055 | `docs/v2-rollback.md` | Flag flip + asset purge | Tabletop drill |
| T-077 | Final integration sign-off | qa | qa-validator | 0.5 | T-051..T-076 | sign-off doc | All gates pass | Recorded |
| T-078 | Tag release `nadashakti-v2.0.0` | infra | claude | 0.5 | T-077 | git tag + GH release | Notes link to UAT | Release published |

**Total: 78 tasks · est. ~108 hours · 3 phases · 9 waves · 22 swarms**

## 8. Dependency Rationale

- **Serial entry-point:** T-001..T-018 (Phase 1) must complete before any Phase 2 swarm starts — they freeze the contracts every parallel swarm depends on. No exception.
- **Independent Phase 2 swarms:** 2.1 (Gamakas/Timbres), 2.2 (Tala/Breath), 2.3 (Render) touch disjoint files after the contract freeze. They can run in three parallel worktrees.
- **Integration choke points:** T-040 (`playRaagaFull`) is the integration node — it consumes outputs from all three Phase 2 waves. Schedule it after 2.1 and 2.2 land. T-051 (render fidelity) gates Phase 3.
- **Test gates:** T-018 closes Phase 1; T-041, T-042, T-051 close Phase 2 waves; T-077 closes Phase 3.
- **Lock-zone serialization:** `RaagaPlayer.ts`, `Nadabrahman.tsx`, `melakarta_body_map.html` are shared. Edits to these are serialized via single owner per wave (`ui-integrator` for v2 wiring, `claude` for v1 bug fixes only).

## 9. Verification Strategy

| Level | Gate | Evidence |
|---|---|---|
| **Task** | per-task `validation` field | Test output, screenshot, log, or diff |
| **Swarm** | swarm-level smoke test | Demo page or unit-test suite green |
| **Wave** | wave-close runbook (`runbooks/wave-close.md`) | Checklist from `verification-gates.md` ticked |
| **Phase** | phase-exit review | All wave gates closed; contract review (T-018, T-077) signed |
| **Project** | UAT recording (T-071) + rollout flag (T-055) | Telemetry green for 48h post-flip |

**Audio-specific verifications:**
- **Pitch correctness (T-059):** YIN/CREPE pitch detection on every rendered raga. Each swara must land within ±5 cents of its just-intonation Hz. Anything beyond is a regression.
- **Vivadi audibility (T-060):** Blind A/B confirms R3 (#5) and G1 (#5) sound *identical* (same shruti, same Hz) only when same gamaka applied — but raga *context* (which neighboring swaras) makes them feel different. This is the test that proves shrutis still drive the system after gamakas attach.
- **Render fidelity (T-051, T-058):** Live browser audio vs OfflineAudioContext-rendered WAV must match RMSE < 0.01.

## 10. GitHub Sync Strategy

- **Issue mapping:** one GitHub issue per task (T-001 → `Issue #N`). Phase/wave/swarm encoded as labels: `phase:1`, `wave:1.2`, `swarm:samples-cdn`.
- **Dependencies:** issue body contains "Blocked by: #M" lines mirroring `dependencies` array. GH Projects v2 board groups by phase column.
- **Wave summary comments:** at each wave boundary, post a single comment summarizing closed tasks, evidence links, and downstream readiness — per `playbooks/github-sync.md`.
- **PR linkage:** PR title `[T-NNN] <title>`; PRs auto-close their issue. Wave-merge happens via a single integration PR per wave (`integration/v2-wave-2.1`).
- **Branch naming:** `v2/<swarm-id>/<short-slug>` per `playbooks/worktree-strategy.md`.

## 11. Worker Bootstrap Packets

For each Phase 2 swarm, generate a `.swarm-handoff/<swarm-id>.md` packet using `templates/agent-handoff-template.md`. Each packet includes:
- frozen contract URLs (Phase 1 outputs)
- worktree command (`git worktree add ../v2-<swarm> v2/<swarm>`)
- environment vars (`RAAGA_V2_ENABLED=true`)
- expected deliverables and acceptance
- validation script path
- escalation path (back to planner Claude)

Spawn order: 2.1 + 2.2 in parallel, 2.3 starts when 2.1 lands `T-040`.

## 12. Risks & Fallback Plan

| Risk | Probability | Impact | Trigger | Fallback |
|---|---|---|---|---|
| Strudel `penv` doesn't reach pitch precision needed for gamakas | M | H | Pitch tracker fails T-059 | Custom OscillatorNode bank with manual envelope automation |
| Sample CDN goes down mid-rollout | L | M | T-073 alarm | Switch to GH raw URL fallback; pre-bundle minimal pack in app |
| OfflineAudioContext doesn't replicate live (timing drift) | M | H | T-051 RMSE > 0.01 | Live-only render; capture via MediaRecorder instead |
| Bundle size blows past 500KB budget | L | M | T-068 fail | Code-split by feature flag; lazy-load gamaka/sample modules |
| Browser AudioContext autoplay restrictions tighten | L | H | Future browser update | Always-on user-gesture init pattern; audio status pill stays |
| Sample licenses ambiguous on second pass | M | M | License audit fails | Drop ambiguous samples; ship sitar+tanpura+mridangam minimum |

## 13. Definition of Done

The **Nādashakti v2** ships when:
- [x] (this document) Discovery complete; assumptions/constraints captured
- [ ] All 78 tasks closed with evidence
- [ ] All 9 waves closed with `wave-close` checklist
- [ ] T-077 sign-off recorded
- [ ] T-071 UAT recording posted
- [ ] T-078 release tagged + notes published
- [ ] v1 callsites still pass smoke tests (no regressions)
- [ ] `RAAGA_V2_ENABLED=true` for 48h with no rollback

---

🎵 *"नाद से रस — From sound, the essence."*
