# Resources & Assets — Selemene + Sankalpa Engine Integration (Deepened Extraction)

**Purpose:** Extracted inventory of everything that already exists (code, docs, prior artifacts, tracking) as of 2026-07-17. This is the source of truth for "what we have" before execution or further planning. All items are pulled from actual files and prior work.

## 1. Code Resources (Prong 1 — Selemene-engine Backend)

### Rust Engines (form + analysis)
- `crates/engine-biofield/` — Server-side birth biofield (currently mock `generate_mock_metrics`; Vedic path stub exists). See `src/engine.rs:36`, `models.rs:98`, `mock.rs`.
- `crates/engine-face-reading/` — Stub + heuristic. `src/engine.rs` (heuristic_from_seed, serialize_analysis), `models.rs`, `wisdom.rs`. No real CV/image processing yet.
- Other Rust engines (from matrix): biorhythm, gene-keys, human-design, nadabrahman, numerology, panchanga, transits, vedic-clock, vimshottari (most phase 0/2, not in current focus).

### TS Engines (media + generative)
- `ts-engines/src/engines/raaga/` — **Production-ready**. `engine.ts`, `wisdom.ts` (72 melakartas, SHRUTIS, dosha/prahar tables), `witness.ts`. Strudel-compatible output. Registered in bridge.
- `ts-engines/src/engines/sigil-forge/` — Functional. `engine.ts`, `wisdom.ts` (SIGIL_METHODS, CHARGING_METHODS), `prompt-builder.ts`, `witness.ts`, `utils/nvidia-image.ts` (NVIDIA NIM only today). Outputs intention + method steps + optional b64 image. No vector_path despite OpenAPI stub.
- `ts-engines/src/index.ts` — Registry.
- Bridge registration: `crates/noesis-bridge/src/lib.rs:297-303` (BridgeEngine::raaga), similar for sigil, manager at :542.

### Core Contracts & Scaffolding
- `crates/noesis-core/src/types.rs` — EngineInput/Output, BiofieldResultSchema (inaccurate for live), FaceReadingResultSchema, SigilForgeResultSchema (mismatch on vector_path).
- `crates/noesis-bridge/` — TS engine proxy, registration, API surface.
- `noesis-api/` + `python-services/` — Biofield capture sidecar client (`biofield_client.rs`), CV pipeline for 11-metric `BiofieldMetrics` + quality.
- Image providers: Current = NVIDIA NIM (sigil). Target to abstract + add nano-banana (Google) + kimi.

### Supporting
- `docs/baseline/engine-matrix.json` — Canonical list: 11 Rust + 6 TS engines.
- Engine registration patterns in bridge + TS.

## 2. Code Resources (Prong 2 — Sankalpa Frontend)

- `sankalpa/src/renderer/biofield/` — **Existing local implementation**:
  - `biofieldDomain.ts` (11 metrics + CompositeScores)
  - `pip/MetricsCalculator.ts` (live per-frame 5 scores from camera)
  - PIP portal, mandala, compass, score ring components.
  - Capture flow with explicit opt-in before any upload.
- `sankalpa/src/App.tsx`, `features.ts` — Shell + route/feature catalog (Noesis + Biofield surfaces present).
- Local analysis only today; no calls to Selemene engines yet.
- Design system: frozen Kha/Ba/La tokens, Goethe palette, visual ledger sheets 00-11 (except 01).
- Security model: contextIsolation true, nodeIntegration false, sandbox true (per ISA).

## 3. Documentation & Reference Assets

### Per-Engine Data References (source of truth for schemas + status)
- `Selemene-engine/docs/engines/biofield.md` (detailed dual-engine note, metrics mismatch, bridge explanation)
- `docs/engines/raaga.md` (full TS impl, no OpenAPI)
- `docs/engines/sigil-forge.md` (NVIDIA only, stub mismatch warning)
- `docs/engines/face-reading.md` (stub warning, no image proc)
- Parallel copies in `docs/portal/docs/engines/`
- `docs/engines/README.md`, `docs/engines/index.md`

### Project-Level
- `sankalpa/ISA.md` — 50+ signed-off ISC criteria (shell, biofield local, Noesis depth viewer, visual system, deferred risks). Phase complete, 50/50 progress.
- `sankalpa/README.md` — Current state summary, commands, migration notes.
- `sankalpa/ROADMAP.md` (updated) — Milestone 4b for engine integration.
- `.github/projects/CONSCIOUSNESS_ROADMAP.md` (updated) — Project config + new integration section.
- `Selemene-engine/README.md` + crate READMEs.

### Prior Planning Artifacts (this initiative)
- `docs/plans/engine-integration/selemene-sankalpa-full-integration-swarm-plan.md` — 7 phases, waves, 130+ tasks, agent topology, risks.
- `docs/plans/engine-integration/detailed-task-list.md` — Schema tasks (P1 contract-first etc.).
- `docs/plans/engine-integration/discovery-summary.md` — Inputs, assumptions, unresolved.
- `docs/plans/engine-integration/github-issue-mapping.md` — Issue sync.

## 4. Tracking & Coordination Assets
- GitHub Epic: #893 (engine-integration)
- GitHub P1: #894
- GitHub P5 (raaga): #895
- Labels applied: phase:integration-p1, wave:integration-w1, area:engine-integration, engine-biofield, engine-face-reading, engine-raaga, engine-sigil-forge, swarm:selemene-backend, swarm:sankalpa-frontend + others.
- engine-matrix.json as single source for engine list + phases.

## 5. External / Tooling Resources (known)
- NVIDIA NIM (current for sigil image)
- Strudel (raaga audio, client-side)
- Python CV sidecar (biofield capture metrics)
- Target: nano-banana, kimi (via runcomfy or direct API in Selemene)
- Potential: MediaPipe (deferred, per Sankalpa ISA)

## 6. What Has Already Been Done (Recent)
- Swarm-architect plan generation + task list (2026-07-16).
- GitHub issue creation + label mapping + body updates with plan links.
- Roadmap updates in both repos with two-prong model and tags.
- Deep-dive analysis feeding the initial plan (engine status, contract review, Sankalpa scaffolding audit).
- Consistent tag vocabulary established and applied.

**Extraction note:** All above pulled directly via reads/globs of the listed paths + gh issue views + prior session artifacts. No assumptions added without file evidence.

**Next use of this file:** Reference when filling gaps, avoiding duplicate work, or scoping P1 contracts.
