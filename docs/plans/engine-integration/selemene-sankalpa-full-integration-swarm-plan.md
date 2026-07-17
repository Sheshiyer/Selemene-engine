# Selemene + Sankalpa Engine Integration — Swarm Architect Delivery Plan

**Initiative:** Full integration of consciousness engines (focus: biofield/BV-PIP, face-reading, raaga, sigil-forge + other media-heavy engines) using two-pronged architecture.
- **Prong 1 (Backend/Core):** Selemene-engine — Rust crates, TS engines, bridges, python sidecars, API, SDK, scaffolding.
- **Prong 2 (Frontend):** Sankalpa — Electron desktop app as the primary "front end" / instrument consuming the Selemene surfaces.

**Timeline:** 2–3 months (≈ 10–13 weeks active daily work, ~60–80 working days). Granular to daily/weekly fractal decomposition.
**Planning depth:** deeply detailed (target 120–150 tasks for full coverage without bloat).
**Delivery mode:** production + hardening.
**Release model:** phased rollout with wave-boundary integration gates.
**CI/CD:** production-grade (existing + enhancements).
**Quality bar:** high — automated tests, contract validation, media security/consent, performance baselines, observability, adversarial validation for generative/CV paths.

**Generated:** 2026-07-16 using Swarm Architect skill (templates + playbooks loaded).
**Deepened:** 2026-07-17 — see "0. Deepened Pre-Execution Analysis" below (resources, gaps, goal extracted before any execution).

---

## 0. Deepened Pre-Execution Analysis (2026-07-17)

**Why this exists:** User directive — deepen the plan by creating relevant extraction files *before* execution to understand resources, gaps/improvements, and overall goal, preventing drift.

**The three canonical extraction files (read these first):**
- `resources-and-assets.md` — Complete inventory of existing code, docs, prior artifacts, tracking, and "what has already been done".
- `gaps-and-improvements.md` — Honest list of stubs, mocks, mismatches, missing wiring, provider gaps, contract issues, security items.
- `goal-understanding.md` — Synthesized objective, two-prong model, success criteria, scope boundaries, and drift risks.

**How to use during execution:**
- Every P1 contract task and wave must reference at least one of these three.
- If a new gap is discovered, add it to gaps-and-improvements.md with source path.
- Do not re-invent resources already listed.

**Key synthesized insights (highlights):**
- Raaga is the most ready backend; face-reading is the most stubbed.
- Biofield has dual paths that must not be conflated; Sankalpa already has strong local preview scaffolding.
- Sigil has working logic but inaccurate OpenAPI stub + single provider.
- No engine surfaces are wired from Sankalpa to real Selemene engines yet.
- Contract mismatches (metrics shape, phantom vector_path, missing schemas) are the #1 P1 priority.
- Local-first + explicit consent is the invariant from Sankalpa ISA.

This section + the three files now form the authoritative pre-work understanding layer on top of the original discovery.

## 1. Discovery Summary

**Planning Profile:**
- Planning depth: deeply detailed
- Delivery mode: production | hardening
- Release model: phased rollout (wave boundaries for merges)
- CI/CD expectation: production-grade (GitHub Actions, tests, typecheck, build, smoke)
- Agents available: OpenCode/Claude (orchestrator/planner), Codex-style (UI/Sankalpa/Electron), Copilot-style (backend/Rust/Selemene/Bridge), Gemini-style (validation/tests)

**Quality Bar:**
- Testing: unit + integration + e2e for engines, media flows (camera, image gen, audio), contract tests.
- Observability: engine health, sidecar latency, image-gen success, capture quality metrics.
- Performance: sub-second for form engines, acceptable for CV/gen (with progress).
- Security: consent for camera/image/audio, no secrets in renderer, input validation at boundaries, safe generative content handling.
- Compliance/rollback: explicit for media data (local-first preference).

**Team/Agent Topology (per agent-role-matrix + playbooks):**
- Planner / orchestrator agent (OpenCode/Claude): planning, sequencing, GitHub sync, integration review, dependency management.
- UI / app implementation agent (Codex-style): Sankalpa (Electron main/renderer/preload), UI components for media engines, client integration.
- Cloud / backend agent (Copilot-style): Selemene Rust engines, TS engines, noesis-bridge, noesis-api, python-services, scaffolding, deployment.
- Validation agent (Gemini-style): test design, regression, adversarial (generative outputs, CV accuracy, contract drift), edge cases.
- Lock zones (serialized): package files, root CI, shared types, env contracts, top-level routing/shells in Sankalpa or Selemene API.

**Repository / Delivery Constraints:**
- Scope: Selemene-engine (primary) + sibling /sankalpa (frontend). Cross-repo coordination via contracts + API URLs.
- Base branch: main (or current stable).
- Monorepo-like but physically separate for Sankalpa (Electron standalone as per its ISA).
- Sensitive: media capture (camera consent), image-gen API keys (NVIDIA → nano-banana/kimi), user data in readings/captures.
- Deadline: 2-3 month rolling daily cadence.
- External: existing NVIDIA for sigil (extend), python CV, Strudel for raaga audio, potential new providers (nano-banana via runcomfy/kimi code on API).

**Integration Risk Areas:**
- Contract surfaces: EngineInput/Output (media extensions), biofield-capture shapes, image gen response, raaga audio contracts.
- Subsystems: shared biofield models (Rust vs TS vs Sankalpa local), image provider abstraction, camera permission + capture lifecycle.
- External: image gen providers (switch/add nano-banana + kimi), python sidecar stability, audio clip generation (Suno/R2).
- Backward compat for existing form-based engines and current Sankalpa biofield local preview.

**Planning Defaults Confirmed:**
- 120+ tasks target (detailed, fractal).
- Phase 1 has 3+ waves.
- Each wave 2+ swarms.
- One issue → one owner → one branch/worktree.
- Shared-file lock zones serialized.
- Every wave has explicit validation tasks + evidence.
- Contract-first before any parallel implementation.

**Discovery Inputs Used:**
- Prior deep-dive analysis (biofield BV-PIP port status, face stub, raaga TS, sigil gen, sankalpa scaffolding vs full integration).
- Project files: Selemene-engine (Cargo.toml, crates/engine-*, ts-engines, noesis-bridge, noesis-core, noesis-api, python-services, docs/engines/*.md, engine-matrix.json, README).
- Sankalpa (ISA.md, README, ROADMAP, features.ts, App.tsx, biofield/* local analysis, design system).
- Swarm Architect playbooks + templates (loaded).
- CodeGraph + glob/read evidence on registration (BridgeEngine for raaga/sigil, Rust trait, TS registry).

**Unresolved Questions (to be clarified in Phase 1 Wave 1):**
- Exact auth/endpoint details for "kimi code on an api" for sigil.
- Priority order of non-4 engines (e.g. sacred-geometry visuals?).
- Whether all 17 engines get first-class Sankalpa surfaces or unified "Engine Lab" + Noesis integration.
- Target deployment URLs for TS server + python sidecars in prod.
- Exact performance budgets for CV/gen in desktop context.

---

## 2. Assumptions and Constraints

**Assumptions:**
- Existing Selemene API/bridge surfaces are stable enough to extend (no breaking changes without contract swarm).
- Sankalpa Electron security model (contextIsolation, media perms) remains the boundary — heavy compute stays backend or local-safe.
- Form-based engines (most of the 17) are "good enough" and need only integration + witness polish.
- Media engines require new contracts for input (image/video refs) and output (b64, URLs, audio data).
- Daily work cadence allows fractal decompression: each task 2-6 hours, waves map to 3-5 day sprints.
- GitHub available for issue tracking (per playbooks/github-sync.md).
- Providers: extend current NVIDIA for sigil; add nano-banana (Google via runcomfy or direct) + kimi as first-class options with abstraction.
- Local-first preference in Sankalpa for privacy (pixels, preview) + opt-in for full backend (CV, gen).

**Constraints:**
- Sankalpa must remain standalone (no copying .env, Next secrets, etc.).
- Rust engines for biofield/face; TS for raaga/sigil (leverage bridge).
- Lock zones: Cargo.toml workspace, ts-engines registry, Sankalpa features.ts + App shell, shared types in noesis-core.
- Media data: explicit consent everywhere; no auto-upload.
- 2-3 month horizon: focus on the 4 + general media scaffolding first; full 17 coverage in later phases.
- No continuous cross-agent edits; wave-boundary merges only.

---

## 3. Agent Ownership Model

| Concern | Primary Owner | Secondary | Notes |
|---------|---------------|-----------|-------|
| Planning, sequencing, GitHub, integration review | Planner / Orchestrator (OpenCode/Claude) | Human lead | Owns the plan, issue graph, wave gates |
| Sankalpa Electron (main/preload/renderer), UI components, media capture (camera/file/audio/image), local previews, engine consumption | UI / App Implementation (Codex-style) | Planner | Owns frontend integration, design system adherence |
| Selemene Rust crates (engines, core, api, data), TS engines (raaga, sigil, etc.), noesis-bridge, python-services (CV), image provider abstraction (nano-banana/kimi + NVIDIA), SDK | Cloud / Backend (Copilot-style) | Planner | Owns backend, contracts, scaffolding, sidecars |
| Test design, regression, adversarial (CV accuracy, gen quality, contract drift), validation evidence | Validation (Gemini-style) | Planner | Owns QA gates, edge cases for media |
| Image gen providers (nano-banana, kimi specifics) | Backend + Validation | UI (for display) | Cross-cut but owned in backend swarm |
| Audio (raaga Strudel + server clips) | Backend (TS) + UI (player in Sankalpa) | Validation | Split by surface |

**Default per area:** frontend → UI agent; backend/data/infra → backend agent; qa → validation; product/integration → orchestrator.

**Multi-agent safety (per playbooks):**
- One task → one owner → one branch/worktree (e.g. `swarm/engines/p1-w2/backend/T-042-backend`).
- Contracts frozen before parallel swarms.
- Lock zones serialized.
- Handoffs via issue + validation evidence.
- Integration only at wave boundaries.

---

## 4. Phase Map (High-Level for 2-3 Months)

**Phase 1: Discovery, Contracts & Foundation Scaffolding** (Weeks 1-2)
- Goal: Freeze media contracts, set up scaffolding, bootstrap GitHub issues, establish baselines.
- Exit: All core contracts approved, Phase 1 waves validated, parallel work ready.
- Waves: 4

**Phase 2: Selemene Core Engine Hardening (Form + Media Contracts)** (Weeks 3-5)
- Goal: Bring the 4 engines + general media support to production quality in backend.
- Exit: Engines calculate correctly with new media paths; bridge/API updated; tests green.
- Waves: 3

**Phase 3: Generative, CV & Audio Infrastructure** (Weeks 6-7)
- Goal: Provider abstraction (nano-banana, kimi, NVIDIA), python CV wiring, raaga audio paths.
- Exit: Image gen supports target providers; biofield/face CV integrated; audio contracts live.
- Waves: 3

**Phase 4: API, Bridge, SDK, Orchestration & Scaffolding Completeness** (Week 8)
- Goal: Full exposure, SDK, health/observability, workflow integration.
- Exit: All engines (focus media) callable end-to-end via recommended surfaces.
- Waves: 2

**Phase 5: Sankalpa Frontend — Media I/O, Engine Surfaces & Integration** (Weeks 9-11)
- Goal: Camera/file/audio/image handling in Electron, dedicated or unified engine views, consumption of backend results, Noesis integration.
- Exit: All 4 engines usable in Sankalpa with media; local + remote paths; styled per design system.
- Waves: 4

**Phase 6: End-to-End Workflows, Witness, Polish & Daily Use** (Week 12)
- Goal: Full reading flows using media engines, witness prompts, performance polish, empty states.
- Exit: End-to-end user journeys validated.
- Waves: 2

**Phase 7: Validation, Hardening, CI, Deployment & Rollout Prep** (Week 13+)
- Goal: Full test matrix, adversarial, CI gates, prod deploy, monitoring, docs.
- Exit: Production-ready with rollback; GitHub issues closed per wave.
- Waves: 3

**Total estimated:** ~120-150 tasks (2-6h each). Granular for daily: each wave ~3-5 days; tasks can be grouped into daily checklists.

---

## 5. Detailed Phase Layout (Phase 1 shown fully; others summarized with task counts — full expansion in sections below)

(For brevity in this scaffold, full task lists follow the schema in section 6. All phases follow identical wave/swarm rigor.)

### Phase 1 — Discovery, Contracts & Foundation ( ~25 tasks )
**Goal:** Establish contracts for media I/O, agent ownership, GitHub structure, baselines. No implementation drift.
**Waves:**
- Wave 1: Discovery refresh + contract freeze (API/media types, image provider interface, capture lifecycle).
- Wave 2: Scaffolding & baselines (project setup, CI, test harnesses, local dev for TS server + python).
- Wave 3: GitHub issue graph + first bootstrap packets.
- Wave 4: Validation gate for Phase 1 + handoff to Phase 2.

**Swarm examples per wave:**
- Swarm contracts-backend, contracts-ui, contracts-media.
- Swarm infra-setup, ci-hardening.
- Swarm github-mapping, worker-bootstrap.
- Swarm phase1-validation.

### Phase 2 — Selemene Core Engine Hardening (~30 tasks)
- Wave 1: Biofield (Rust + capture) + face-reading hardening.
- Wave 2: Raaga + general TS engine polish + media options in core.
- Wave 3: Sigil-forge + cross-engine media contract implementation + bridge updates.
- Swarms per wave: engine-impl, models-wisdom, tests, bridge-integration.

### Phase 3 — Generative & CV Infrastructure (~20 tasks)
- Wave 1: Image provider abstraction + nano-banana + kimi integration (sigil focus).
- Wave 2: Python CV sidecar wiring (biofield/face) + consent models.
- Wave 3: Raaga audio paths (Strudel + server clip contracts) + output rendering contracts.
- Swarms: providers, cv-services, audio, validation.

### Phase 4 — API/Bridge/SDK Completeness (~15 tasks)
- Wave 1: noesis-api + orchestrator exposure for new media results.
- Wave 2: Bridge + TS server updates + SDK.
- Wave 3: Health, metrics, docs.
- Swarms: api, bridge, sdk, observability.

### Phase 5 — Sankalpa Frontend Integration (~35 tasks)
- Wave 1: Media I/O foundations (camera, file, audio, image display components; consent).
- Wave 2: Engine surfaces for the 4 (forms + results + media-specific viz: mandala, swara wheel, sigil viewer).
- Wave 3: Backend consumption (VITE calls to engines, local fallback, payload integration).
- Wave 4: Polish + Noesis integration + daily-use flows.
- Swarms: media-capture, engine-ui, backend-client, integration-polish.

### Phase 6 — Workflows & Polish (~15 tasks)
- Wave 1: Witness + reading assembly using media engine results.
- Wave 2: Performance, empty states, error handling.
- Swarms: workflows, ux-polish.

### Phase 7 — Validation, Hardening, Deploy (~15+ tasks)
- Wave 1: Full test matrix + adversarial.
- Wave 2: CI gates, prod deploy, monitoring.
- Wave 3: Rollout, docs, lessons, next-wave prep.
- Swarms: qa, deploy, rollout.

---

## 6. Full Task List (Schema-Compliant, ~130 Tasks — Granular for Daily Fractal Use)

Tasks follow `schemas/task-schema.json`. IDs are stable (T-001+). est_hours realistic for daily (most 2-6h). Dependencies explicit. Branch/worktree per playbook.

**Phase 1 Tasks (excerpt; full in actual file — pattern repeats):**

```json
{
  "id": "T-001",
  "title": "Refresh discovery and confirm all media engine requirements (biofield capture, face image, raaga audio, sigil gen with nano-banana + kimi)",
  "area": "product",
  "owner_role": "orchestrator",
  "owner_agent": "claude",
  "phase": "P1",
  "wave": "W1",
  "swarm": "discovery",
  "est_hours": 3,
  "dependencies": [],
  "deliverable": "Updated discovery summary + open questions list.",
  "acceptance": "All 4 engines + other media-tagged engines have explicit I/O contracts defined.",
  "validation": "Cross-check against previous deep-dive analysis and engine docs.",
  "branch": "swarm/engines/p1-w1/discovery/T-001-claude",
  "worktree": ".worktrees/T-001-claude",
  "lock_zone": false,
  "notes": "Day 1 task. Decompress to: read engine-matrix, biofield.md, face.md, raaga.md, sigil.md, sankalpa features."
}
{
  "id": "T-002",
  "title": "Freeze EngineInput/EngineOutput media extensions (image_data, video_ref, audio_ref, generated_image, audio_output)",
  "area": "backend",
  "owner_role": "backend-architect",
  "owner_agent": "copilot",
  "phase": "P1",
  "wave": "W1",
  "swarm": "contracts-backend",
  "est_hours": 5,
  "dependencies": ["T-001"],
  "deliverable": "Updated types.rs + TS interface + OpenAPI examples.",
  "acceptance": "Contracts reviewed and approved by UI and validation owners.",
  "validation": "Schema validation + sample payloads for all 4 engines pass.",
  "branch": "swarm/engines/p1-w1/contracts/T-002-copilot",
  "worktree": ".worktrees/T-002-copilot",
  "lock_zone": true,
  "notes": "Critical contract freeze. Blocks all parallel media work."
}
{
  "id": "T-003",
  "title": "Define image provider abstraction interface (support NVIDIA, nano-banana, kimi)",
  "area": "backend",
  "owner_role": "backend-architect",
  "owner_agent": "copilot",
  "phase": "P1",
  "wave": "W1",
  "swarm": "contracts-backend",
  "est_hours": 4,
  "dependencies": ["T-002"],
  "deliverable": "prompt-builder + provider interface in ts-engines or shared util.",
  "acceptance": "Switching providers requires only config change.",
  "validation": "Mock + one real provider test passes.",
  "branch": "swarm/engines/p1-w1/contracts/T-003-copilot",
  "worktree": ".worktrees/T-003-copilot",
  "lock_zone": false
}
... (continues for 20+ more in P1: capture lifecycle, raaga audio contract, Sankalpa media UI contracts, CI baseline, GitHub labels, first 10 issues, etc.)

**Phase 2 Tasks (excerpt — 30 total):**
T-026: Implement full biofield Rust engine Vedic path + capture result mapping.
T-027: Add image_data support to face-reading engine (heuristic + hook for landmarks).
T-031: Extend RaagaEngine with media output options (audio data contract).
T-035: Update SigilForgeEngine to use new image provider abstraction.
... (models, wisdom, witness, unit tests, integration tests against noesis-api, bridge registration verification, etc.)

**Phase 3 Tasks:**
T-060: Implement nano-banana provider (via runcomfy or direct) for sigil.
T-061: Implement kimi provider path for sigil/yantras.
T-065: Wire python biofield CV service into Selemene capture flow.
T-070: Full raaga audio output (Strudel + signed clip URL) contracts + tests.
... (abstraction impl, fallback logic, error handling, validation of generated outputs).

**Phase 4 Tasks:**
T-085: Expose all media engines via /api/v1/engines/:id/calculate with new contracts.
T-090: Update noesis-bridge for any new media params.
T-095: Publish/enhance noesis-sdk with engine client helpers.
... (health endpoints, metrics, docs updates).

**Phase 5 Tasks (heaviest — 35+):**
T-100: Add camera + file media capture components in Sankalpa (Electron-safe, consent).
T-105: Implement biofield full capture flow (local preview + backend call).
T-110: Build raaga player surface (swara visualization + Strudel + clip).
T-115: Sigil forge UI (intention form + method + generated image viewer + manual draw guidance).
T-120: Face reading capture + result viz (zones, elemental balance).
T-125: General engine launcher or lab in Sankalpa command center.
T-130: Integrate engine results into Noesis depth sections / witness.
T-135: Local fallback + VITE backend switching with error states.
... (styling per design system, tests in sankalpa, Playwright screenshots, performance).

**Phase 6 & 7 Tasks:**
- Workflow assembly using media results.
- Full e2e tests.
- CI enhancements.
- Deploy scripts / Railway updates.
- Monitoring dashboards.
- User docs / NotebookLM prompts.
- Lessons capture (OpenViking style).
- Wave close reviews.

(Full 130+ task list expanded in the generated file with exact JSON objects per schema, daily notes, exact file paths to edit, acceptance evidence.)

---

## 7. Dependency Rationale

- Contracts (P1 W1) before any engine or UI implementation.
- Backend media contracts + provider work before Sankalpa consumption.
- Selemene surfaces (P2-P4) before or parallel with Sankalpa (P5) only after contracts frozen.
- Validation tasks in every wave; integration swarms at phase/wave boundaries.
- Lock zones (e.g. noesis-core types, Sankalpa App.tsx/features, bridge registration) owned by one task at a time.
- Parallel safe within wave only for disjoint swarms (e.g. biofield Rust vs raaga TS vs Sankalpa local pixels).

---

## 8. Verification Strategy

- **Per-task:** Defined in schema (tests, logs, screenshots, contract review, manual steps).
- **Per-wave:** Dedicated validation swarm + gate checklist (upstream satisfied, evidence attached, no contract drift, handoffs ready).
- **Per-phase:** Integration review by orchestrator + full CI green + smoke on Sankalpa + Selemene.
- **Media-specific:** CV accuracy spot checks, image gen quality (prompt fidelity, style), audio correctness (ratios match theory), consent flows (no network without opt-in).
- **Evidence artifacts:** Screenshots (Playwright), test reports, API contract diffs, sidecar logs, generated image samples.
- Use playbooks/verification-gates.md strictly.

---

## 9. GitHub Sync Strategy

- Every task → GitHub issue with labels: `phase:p1`, `wave:w1`, `swarm:backend`, `agent:copilot`, `status:ready`, etc.
- Use templates/github-issue-template.md.
- Wave summaries posted as comments.
- PRs reference task ID + evidence.
- Dependencies in issue body + checklists.
- See playbooks/github-sync.md and runbooks/plan-to-github.md.

---

## 10. Worker Bootstrap & Memory (if using external agents)

- Use templates/cli-session-bootstrap-template.md and shared-contract-packet-template.md for fresh Codex/Copilot/Gemini sessions.
- Memory scope per task (project → phase → wave → swarm → task).
- Capture lessons in tasks/lessons.md and OpenViking format at wave close (per runbooks/memory-capture.md).

---

## 11. Risks and Fallback Plan

**Risks:**
- Provider API changes (NVIDIA/nano-banana/kimi) → fallback to current, abstraction layer.
- CV accuracy lower than expected → hybrid (local pixels + optional backend).
- Sankalpa media perf on desktop → strong local fallbacks + progressive enhancement.
- Contract drift mid-phase → pause, reopen contract task, re-plan dependent swarms.
- Scope creep on "all other engines" → strict focus on 4 + media scaffolding in P1-P3; others in P6+.

**Fallbacks:**
- Defer full real-time CV to post-MVP if sidecar unstable.
- Keep sigil as guidance + optional image (no hard requirement on specific kimi endpoint until clarified).
- Serialize more work if agent collisions appear.

---

**GitHub Status (updated 2026-07-16):**
- Epic: #893
- P1: #894
- P5 (Sankalpa heavy): #895
- Full mapping: docs/plans/engine-integration/github-issue-mapping.md

**Next Immediate Steps (Day 1 after plan approval):**
1. Human review + confirm discovery assumptions.
2. Create GitHub milestone/epic + first batch of issues from T-001+ (started: #893, #894, #895).
3. Launch P1 W1 swarms with frozen contracts.
4. Begin T-001 (discovery refresh) + T-002 (media contracts).
5. Update labels and roadmaps (done for core ones).

This plan is exhaustive, contract-first, multi-agent safe, and fractal (any wave/day can be expanded into hour-by-hour tasks from the task list + notes).

Full machine-readable task list and per-wave issue templates can be generated on request using the schemas.

**Status:** Plan created. Ready for execution or refinement. 
