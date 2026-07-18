# P1 Worker Bootstrap Packet — Selemene + Sankalpa Engine Integration (Wave 1: Contracts & Foundation)

**Version:** 2026-07-17 (created during P1 W1 GitHub/scaffolding/validation prep)
**For:** Any agent (orchestrator, backend/copilot, validation, UI/codex) starting P1 tasks
**Scope:** Phase 1 Wave 1 only (T-001 to ~T-025). Contract-first, no drift.
**Handoff to:** Next wave/phase via validation gate + EXECUTION-STATUS.md + issue comments
**Related GitHub:** #893 (epic), #894 (P1), #896 (deepened), #898 (P1 W1 contracts)

---

## 1. Immediate Context (Read First — 5 min)

**Initiative:** Full integration of 4 media/embodiment engines (biofield BV-PIP capture, face-reading, raaga, sigil-forge) via two-prong architecture.
- **Prong 1 (Selemene-engine):** Rust (biofield/face), TS (raaga/sigil via bridge), python CV, image providers (NVIDIA → nano-banana + kimi), contracts, API, scaffolding.
- **Prong 2 (Sankalpa):** Electron desktop instrument — safe media capture (consent, local preview), engine surfaces, consumption of results. Local-first + opt-in backend.

**P1 Goal (Foundation):** Freeze media contracts, set up scaffolding + GitHub, establish baselines. NO implementation of engines/UI yet.
**P1 W1 Focus:** Discovery refresh + contract freeze (API/media types, image provider interface, capture lifecycle, raaga audio).

**Current Execution State (see EXECUTION-STATUS.md for live):**
- Deepened pre-execution complete (2026-07-17): 3 canonical files committed.
- GitHub issues created/updated for P1 W1.
- T-002 active in worktree: `swarm/engines/p1-w1/contracts/T-002-copilot` (lock zone on noesis-core types).
- This bootstrap + validation gate + EXECUTION-STATUS created.
- Labels verified, roadmaps + mapping to be updated in this session.

**Do not start without:**
- Reading the 3 deepened extraction files (below).
- Checking EXECUTION-STATUS.md for what's already started.
- Confirming your task ID from detailed-task-list.md.

---

## 2. The Three Canonical Pre-Execution Extraction Files (Anti-Drift — READ THESE)

Every P1 task **must** reference at least one.

1. **resources-and-assets.md**
   - Full inventory of what already exists (do not re-invent).
   - Key: raaga is production-ready (TS + Strudel), sigil functional (NVIDIA only), biofield has dual paths (server mock + Sankalpa local PIP), face is stub.
   - Sankalpa has strong local biofield scaffolding (biofieldDomain.ts, MetricsCalculator, consent model).
   - Bridge registration pattern exists for TS engines.

2. **gaps-and-improvements.md**
   - Honest list of stubs/mocks/mismatches/wiring gaps.
   - Critical for P1: schema mismatches (sigil claims vector_path that doesn't exist; biofield shapes inaccurate; raaga no OpenAPI; face no image input).
   - Media contracts missing entirely (image_data, audio_ref, consent, generative outputs).
   - Only NVIDIA provider; no abstraction.
   - No Sankalpa → Selemene calls yet for any engine.
   - Consent model exists for biofield local but must generalize.

3. **goal-understanding.md**
   - Locked objective: two-prong, 4 focus engines first, contract-driven.
   - Success for P1: frozen accurate EngineInput/Output + per-engine results (fix mismatches); media extensions; bridge/API roundtrip; engine-matrix as truth.
   - Drift risks: UI before contracts, conflate biofield paths, hard-code providers, lose local-first/consent, treat stubs as done.
   - Unresolved (carry into P1 W1): kimi details, scope for non-4 engines, perf budgets, prod URLs.

**How to use:** In every commit/PR/issue comment for P1 work, cite e.g. "Addresses gap X from gaps-and-improvements.md; aligns with goal-understanding two-prong."

---

## 3. Core Contracts & Deliverables for P1 W1 (from detailed-task-list + plan)

**Primary (T-002 to T-005 — lock zones on shared types):**
- **T-002 (lock):** Freeze EngineInput + EngineOutput extensions for media (image_data b64/ref, video_ref, audio_ref, generated outputs, consent_token, quality, local_only).
  - Files: crates/noesis-core/src/types.rs + TS mirror + OpenAPI.
  - Acceptance: UI + validation approve; samples for 4 engines; schema validates.
- **T-003:** Image provider abstraction (NVIDIA current + nano-banana + kimi). Config-only switch. Update sigil prompt-builder.
- **T-004 (lock):** Biofield-capture + face image capture lifecycle (requested/uploaded/analyzed/persisted, consent, quality). Matches sankalpa biofieldDomain + extends.
- **T-005:** Raaga audio output contract (strudel_ratios, swaras, prahar + optional server clip + timbre/gamaka).

**Other P1 W1 (T-006+ est.):**
- Sankalpa media UI contracts (camera/file components, consent UI) — define, do not fully impl yet.
- CI baseline updates, local dev for TS server + python.
- GitHub labels/issue graph (this packet helps).
- Worker bootstrap + this validation gate + handoff.
- Full list + JSON schema in detailed-task-list.md.

**Lock Zones (serialized — one task at a time):**
- noesis-core types (T-002, T-004)
- Sankalpa top-level (features.ts, App.tsx shell, shared media components)
- ts-engines registry + bridge registration
- Root CI / package files

---

## 4. Agent Instructions (Role-Specific)

**General (all agents):**
- Contract-first. Do not write engine impl or UI rendering code until contracts frozen + this gate green.
- Always start session by: read this packet + 3 extraction files + EXECUTION-STATUS.md + your task in detailed-task-list.md + owning GitHub issue.
- Use consistent tags in all artifacts: phase:integration-p1 wave:integration-w1 area:engine-integration swarm:selemene-backend|swarm:sankalpa-frontend engine-biofield|... 
- One task → one owner → one branch/worktree (e.g. swarm/engines/p1-w1/contracts/T-002-copilot).
- Evidence before claim: tests, diffs, samples, screenshots, logs.
- Update issue + EXECUTION-STATUS + handoff note on completion.
- If gap discovered: append to gaps-and-improvements.md with source path.
- Preserve local-first + explicit consent (Sankalpa ISA invariant).
- Reference engine docs/*.md and engine-matrix.json as spec.

**Backend / Copilot-style (Rust/TS contracts, providers, bridge):**
- Own Selemene contracts, types.rs, TS mirrors, provider abstraction, raaga/sigil updates for media, noesis-api models.
- For image providers: make pluggable; keep NVIDIA working; sketch nano-banana/kimi (use external research if needed but prefer repo patterns).
- Test roundtrips in core + ts-engines.
- Do not touch Sankalpa renderer code (UI agent owns).

**UI / Codex-style (Sankalpa contracts):**
- Define media I/O contracts for Sankalpa (camera consent lifecycle, file upload for image, audio player interface, result viz contracts).
- Ensure compatibility with existing biofieldDomain.ts + local PIP.
- Do not call real backend yet (define client surface / VITE switch).
- Style per frozen design system (Kha/Ba/La, Goethe).
- Provide samples that match the backend contract shapes.

**Validation / Gemini-style:**
- Own this gate checklist execution.
- Design contract tests, adversarial (future gen/CV), edge (bad image, no consent, provider fail).
- Verify anti-drift vs goal-understanding.
- Run/define schema validation + roundtrips.
- Spot-check Sankalpa local compat without full impl.

**Orchestrator / Planner:**
- GitHub sync, wave gates, EXECUTION-STATUS, bootstrap packets, roadmap updates, sequencing, handoffs.
- Enforce lock zones + wave boundaries.
- Cross-repo (Selemene + sankalpa) coordination via contracts.

**Memory / Session:**
- Scope: project → phase:integration-p1 → wave:integration-w1 → swarm:xxx → task:T-xxx
- Capture lessons in tasks/lessons.md or per-wave (OpenViking style) at close.
- Use templates from swarm-architect if present; otherwise this packet.

---

## 5. Current State Snapshot (2026-07-17)

**What is solid (from resources-and-assets):**
- Raaga TS engine + wisdom + Strudel full.
- Sigil logic + NVIDIA image (but contract mismatch).
- Sankalpa shell + biofield local preview + consent pattern + visual system (50 ISCs).
- Bridge registration for TS engines.
- Per-engine .md files as spec.
- GitHub labels + initial issues + this planning dir.

**What is stubbed / mismatched (from gaps-and-improvements — fix in P1):**
- Biofield server: always mock; only 5 metrics vs 11 in capture.
- Face: pure stub, no image input.
- Sigil: phantom vector_path in OpenAPI; NVIDIA only.
- Raaga: no OpenAPI presence; no Sankalpa surface yet.
- No media contracts at all.
- Zero end-to-end Sankalpa → Selemene for engines.
- Provider + CV + audio clip paths missing.

**Success for this wave (from goal-understanding + plan):**
- Frozen media-extended contracts + per-engine samples.
- Abstraction started.
- Capture lifecycle defined.
- All 4 have I/O defined.
- No drift on two-prong / local-first / consent.
- This packet + validation gate + status file live.
- GitHub graph + labels current.

**Unresolved (flag in work):**
- Exact kimi "code on an api" (endpoint/auth/prompts for yantras) — research if task requires.
- Unified Engine Lab vs per-engine in Sankalpa.
- Exact perf budgets.
- Prod deploy URLs.

---

## 6. Execution Rules for P1

- **Branching:** `swarm/engines/p1-w1/<swarm>/<T-xxx>-<agent>`
- **Worktree:** `.worktrees/T-xxx-<agent>` (use `git worktree add`)
- **Daily cadence:** Load memory/this packet + 3 files → execute deliverable → validate → update issue + EXECUTION-STATUS + handoff → push.
- **PRs:** Reference task ID + owning issue + evidence. Title: `[P1][W1][swarm] T-xxx — ...`
- **Validation:** Per task acceptance in detailed-task-list + this packet. Full gate at wave end.
- **CI:** Must pass typecheck/tests/build before merge.
- **If blocked:** Update issue, note in EXECUTION-STATUS, escalate to orchestrator. Do not bypass contracts.
- **Handoff:** When task or wave done: update this packet if needed, post summary to epic, mark in EXECUTION-STATUS.

---

## 7. Quick Start Commands

```bash
# Always
cd /path/to/Selemene-engine
git checkout main
git pull

# For a task (example T-002)
git worktree add .worktrees/T-002-copilot swarm/engines/p1-w1/contracts/T-002-copilot || true
cd .worktrees/T-002-copilot
# ... work ...

# Validate contracts example
cargo test -p noesis-core
cd ts-engines && npm test

# Update status
# edit EXECUTION-STATUS.md
# gh issue comment 898 --body "T-002 progress: ..."
```

For Sankalpa work (later in P1 or P5): cd ../sankalpa ; follow its package.json.

---

## 8. Links & References

- Full plan + waves: selemene-sankalpa-full-integration-swarm-plan.md
- Granular tasks: detailed-task-list.md
- GitHub mapping + status: github-issue-mapping.md + EXECUTION-STATUS.md
- Validation gate: p1-w1-validation-gate-checklist.md
- Roadmaps: Selemene .github/projects/CONSCIOUSNESS_ROADMAP.md + sankalpa/ROADMAP.md (Milestone 4b)
- Engine specs: docs/engines/{biofield,face-reading,raaga,sigil-forge}.md + engine-matrix.json
- Sankalpa foundation: sankalpa/ISA.md + biofield/* + features.ts

**This packet supersedes generic templates for P1 work.** Load it at start of every P1 session.

**Next after P1 W1:** Complete contracts + pass validation gate → update EXECUTION-STATUS → handoff packet for P2 or W2.

**Produced as part of:** P1 W1 coordination tasks (GitHub, scaffolding, validation prep).
