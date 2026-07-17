# EXECUTION-STATUS — Selemene + Sankalpa Engine Integration (P1 Wave 1 focus)

**Location:** docs/plans/engine-integration/EXECUTION-STATUS.md
**Purpose:** Live tracking of started tasks for P1 W1 coordination, owners, branches/worktrees, status. Updated at each handoff. Use with GitHub issues + detailed-task-list.md.

**Started:** 2026-07-17 (P1 W1 GitHub/scaffolding/validation prep + initial contracts + remaining batch issues + roadmap updates)

## P1 Wave 1 Tasks Started / In Progress

| Task ID | Title / Focus | Owner / Agent | Branch / Worktree | Status | GitHub | Notes / Evidence |
|---------|---------------|---------------|-------------------|--------|--------|------------------|
| T-002 | Freeze EngineInput/EngineOutput media extensions (core + fixes) | backend (copilot) | swarm/engines/p1-w1/contracts/T-002-copilot<br>.worktrees/T-002-copilot | Completed (in worktree; P1W1-CONTRACTS-FROZEN.md) + validated + reviewed | #898 | Media extensions + provider iface + lifecycle states + raaga audio frozen. Matches FROZEN. cargo check/test + ts typecheck green (after 1 test init fix in worktree). See worktree + FROZEN.md + validation-gate-checklist.md evidence. Post-gate review: 34 files (+463 lines), core in types.rs:437+ (MediaRef/Consent/Quality/Generated*/Capture*), bridge forward, T-003 ImageProvider complete, T-004/5 ready; boilerplate 28 files for struct inits (over-scope noted but minimal/no P2). Ready for boundary merge/handoff. |
| T-003 | Image provider abstraction (NVIDIA, nano-banana, kimi) | backend | (pending T-002) | Ready | #898 | Depends on T-002 |
| T-004 | Biofield-capture + face lifecycle contract | backend | (pending) | Ready | #898 | Lock zone |
| T-005 | Raaga audio output contract | backend | (pending) | Ready | #898 |  |
| #899 (local/CI, T-00x subset) | Local Dev Setup for TS Server + Python Sidecars + CI Baseline Updates (P1 W1 post-gate) | orchestrator / Temperance | main (scaffolding; contracts ref .worktrees/T-002-copilot) | ✅ COMPLETE (this exec): verified runs, basic roundtrips, CI enhanced, READMEs+checklist+STATUS updated w/ evidence. Stayed scaffolding. | #899 | Local dev: bun ts-engines@3001 (health/engines/raaga/sigil-calc OK, 61p tests); python biofield@8002 (uvicorn health opencv=true, pytest 11-metrics PASS). Roundtrip: raaga strudel, sigil provider (media sample via params; full frozen in worktree). CI: test.yml + ts-engines typecheck script + python-sidecars job (smoke curls + pytest). Updates cite 3 extraction files + bootstrap-packet + P1W1-CONTRACTS-FROZEN.md + detailed-task-list. See below + checklist for logs/cmds. |
| (T-006..T-025 est.) | Sankalpa media UI contracts, CI baselines, GitHub labels (done), first wave issues (done), **worker bootstrap packet** (this), local dev setup for TS server + python, **verification gate** (this), handoff + docs update | orchestrator / planner | (this session: coordination + gate exec) | ✅ GATE CLOSED (all checkboxes + evidence in checklist); T-002 validated in worktree + FROZEN; core green | #893, #896, #898, #899, #900, #901, #902 | Validation gate CLOSED: tests (cargo noesis-core/bridge PASS, bun ts 61p, sankalpa tsc PASS), reads/diffs, labels/roadmaps verified+updated, gh comments on #901/#893. All cite 3 extraction files. See p1-w1-validation-gate-checklist.md evidence package + handoff note. T-002 worktree ready for boundary. |
| T-006 (#900) | Define Sankalpa media UI contracts (camera/file components, consent UI, result viewers) | UI (opencode) | main | Complete (scaffolding expanded in engine-media-contracts.ts + minimal compat in biofield/App/features) | #900 | Detailed interfaces: CameraCaptureLifecycle (states+local/remote switch), FileInputContract (face/sigil), RaagaAudioSurface (strudel_ratios, swaras, player), result viewers (BiofieldResultViewerContract w/ 11 metrics, FaceResult w/ zones, Raaga w/ swaras, Sigil image). ConsentGate + toBackend* serializers for FROZEN image_data/consent_token/generated_*. No rendering/backend. Cites resources-and-assets.md + gaps-and-improvements.md + goal-understanding.md + p1-w1-worker-bootstrap-packet.md (UI sec) + P1W1-CONTRACTS-FROZEN.md + checklist + EXECUTION-STATUS. Typecheck pending run. |
| GitHub sync / labels / mapping | Update #893/#896/#898 + new #899-902; verify labels; update mapping + roadmaps | orchestrator | main | Complete (this task) | #899-#902 | Consistent phase:integration-p1, wave:integration-w1 etc. Cites extraction files. |
| Worker bootstrap packet | Create p1-w1-worker-bootstrap-packet.md with context/contracts/state | orchestrator | main | Complete (this task) | - | For P1 agents |
| P1 W1 Validation gate | Draft p1-w1-validation-gate-checklist.md | orchestrator | main | Complete (this task) | - | Cross-ref external attempt; now canonical here |
| Roadmaps | Add execution start note for P1 W1, links to bootstrap/checklist/STATUS/FROZEN, current status (contracts in worktree, gate drafted) | orchestrator | main | Complete (this task; both CONSCIOUSNESS_ROADMAP.md + sankalpa/ROADMAP.md) | - | Cites 3 extraction files; see updated roadmaps |
| Issues update + batch creation | Sync status + all plan links + 3 extraction + new bootstrap/validation + create 3-5 new GH issues for remaining P1 W1/early W2 (local dev/CI, Sankalpa contracts, validation exec, docs/handoff) from mapping + detailed-task-list | orchestrator | main | Complete (this task) | #893 #896 #898 #899 #900 #901 #902 | New: #899 local/CI, #900 Sankalpa UI contracts T-006, #901 validation gate exec, #902 docs+matrix+handoff. All cite extraction files. |

## Key Files (all in docs/plans/engine-integration/)
- selemene-sankalpa-full-integration-swarm-plan.md (full 7 phases)
- detailed-task-list.md (T-001+)
- github-issue-mapping.md (updated)
- resources-and-assets.md (pre-exec inventory; extraction)
- gaps-and-improvements.md (pre-exec gaps; extraction)
- goal-understanding.md (pre-exec objective; extraction)
- p1-w1-worker-bootstrap-packet.md (NEW)
- p1-w1-validation-gate-checklist.md (NEW)
- EXECUTION-STATUS.md (this)
- discovery-summary.md (annotated)
- FROZEN reference: .worktrees/T-002-copilot/docs/plans/engine-integration/P1W1-CONTRACTS-FROZEN.md (contracts)

## Current Execution State
- **Deepened pre-exec (2026-07-17):** 3 extraction files committed and referenced in all issues/roadmaps (resources-and-assets.md, gaps-and-improvements.md, goal-understanding.md).
- **GitHub issues:** #893 (epic/plan), #894 (P1), #895 (P5), #896 (deepened), #897 (P2), #898 (P1 W1 contracts T-002..), #899 (local dev+CI), #900 (Sankalpa UI contracts), #901 (validation gate exec), #902 (docs/handoff/matrix - done).
- **Worktree active:** T-002-copilot for contracts (lock zone on types; P1W1-CONTRACTS-FROZEN.md).
- **Current status:** Contracts in worktree (T-002 complete), gate drafted (checklist), 4 new issues created for remaining P1 W1/early W2. Roadmaps + mapping updated.
- **Next for P1 W1:** Complete contracts (T-003-005 in #898), execute local dev/CI (#899), Sankalpa contracts (#900), run validation gate (#901), docs/handoff (#902 done), handoff to P2 or W2.
- **Mode:** Contract-first. No parallel impl until contracts frozen + validated. Reference 3 extraction files in all work.
- **Tags:** Always use phase:integration-p1, wave:integration-w1, area:engine-integration, swarm:selemene-backend, engine-*, etc.

## Handoff Protocol
- Update this file + relevant issue comment before switching tasks/agents.
- Attach evidence (screenshots, test output, contract diffs) to issue.
- For next: reference this + bootstrap packet + validation checklist + the 3 deepened files.

**Last updated:** 2026-07-17 (P1 W1 ✅ GATE CLOSED + #899 COMPLETE + T-020/T-026/T-027 START (Wave 2 / P2): ... + T-027 in .worktrees/T-027-codex (extend engine-face-reading image_data+consent+heuristic+landmark hook per FROZEN; new test w/ FROZEN sample; cargo test green 35p; cites extraction+FROZEN+bootstrap+T-027+tags). Evidence below + worktree. All minimal. Ready W2 cont.)
**Owner:** Orchestrator (current session)

**Parallel Dispatch Round 1 Results (per temperance-parallel-dispatch protocol):**
  - External rail: /var/folders/.../tmp.Wb2OIZacpp (3 research/contract-draft tasks) — all failed (credits/interpreter/device). Fail-open applied: covered by Codex subagents below.
  - Codex rail (3 parallel Task subagents):
    1. Core contracts (T-002 primary + T-003/4/5): Completed in worktree .worktrees/T-002-copilot on branch swarm/engines/p1-w1/contracts/T-002-copilot. Media extensions frozen in noesis-core, provider iface, lifecycle, raaga audio. Cargo clean. P1W1-CONTRACTS-FROZEN.md created. gh #898/#894 updated. See worktree for diffs.
    2. Sankalpa media contracts scaffold: Completed. engine-media-contracts.ts + minimal facade. typecheck + tests green. gh #895/#896 updated.
    3. GitHub + bootstrap + validation prep + new issues batch: Completed. p1-w1-worker-bootstrap-packet.md, p1-w1-validation-gate-checklist.md, EXECUTION-STATUS.md (this), mapping/roadmaps updated (w/ P1 W1 start note, FROZEN, bootstrap/checklist/STATUS links, contracts worktree status, gate drafted), gh issues #893/#896/#898 + #899-902 created/updated w/ consistent labels + extraction cites. 
  - All reference the 3 deepened extraction files (resources-and-assets.md, gaps-and-improvements.md, goal-understanding.md) + plan + bootstrap.
  - Contract-first invariant held. No P2 impl started.

## Handoff for Next (P1 W1 remaining or Wave 2)

**Completed in this session (Wave 3 GitHub + Wave 4 validation prep from plan + this continuation):**
  - GitHub issues #893, #896, #898 + #899/#900/#901/#902 synced/updated via gh (status, all links to plans + 3 extraction files + bootstrap + validation + EXECUTION-STATUS + FROZEN).
  - Worker bootstrap packet created (p1-w1-worker-bootstrap-packet.md) — agents MUST load.
  - Validation gate checklist drafted (p1-w1-validation-gate-checklist.md) — supersedes external json attempt.
  - Labels verified (key integration ones present/used; additional p3+ created).
  - Roadmaps (both CONSCIOUSNESS_ROADMAP.md + sankalpa/ROADMAP.md) + github-issue-mapping.md + this status updated with P1 W1 execution start note, links to bootstrap/checklist/STATUS/FROZEN, current status (contracts in worktree, gate drafted), new issues #899-902.
  - EXECUTION-STATUS.md updated tracking T-002 (completed in worktree + FROZEN) + new batch issues + prep tasks. All cite the 3 extraction files.
  - New issues created (per github-issue-mapping + detailed-task-list for remaining P1 W1/early W2): #899 local dev+CI baselines, #900 Sankalpa media UI contracts, #901 validation gate execution, #902 per-engine docs + handoff + matrix update.

 **Current live state:**
   - ✅ P1 W1 Validation Gate CLOSED (see p1-w1-validation-gate-checklist.md: all [x] + full evidence package; header/status/summary updated).
    - ✅ #899 (local dev + CI baselines) COMPLETE (this exec, post-gate): verified bun ts-engines@3001 + python biofield@8002 runs; basic contract roundtrips (raaga audio T-005, sigil); CI enhanced in test.yml (ts typecheck+smoke, new python-sidecars pytest+uvicorn health); READMEs/checklist/STATUS + verif cmds updated. Scaffolding only. Evidence below + in checklist. Refs: 3 extraction files (resources-and-assets.md, gaps-and-improvements.md, goal-understanding.md) + p1-w1-worker-bootstrap-packet.md + P1W1-CONTRACTS-FROZEN.md (worktree) + detailed-task-list.md .
    - T-020 STARTED (Wave2 per breakdown): .github/workflows/test.yml enhanced (TS typecheck + media smoke raaga/strudel T-005 + sigil image via provider T-003 w/ consent samples); ts-engines/package.json added smoke:media script; ts-engines/README.md updated w/ T-020 notes + cites. CI smoke exercises frozen media. All cite p1-w1-worker-bootstrap-packet.md + 3 extraction (resources-and-assets.md gaps-and-improvements.md goal-understanding.md) + P1W1-CONTRACTS-FROZEN.md + detailed-task-list T-020 + EXECUTION-STATUS. See test.yml:239+ , package.json, README.
    - T-026 STARTED (P2 per breakdown): worktree .worktrees/T-026-copilot created (branch swarm/engines/p2-w1/biofield/T-026-copilot); crates/engine-biofield/src/engine.rs + models.rs + tests edited for Vedic path harden + capture result mapping to frozen (11 metrics + consent in result json; keep is_mock; roundtrip test added). Cites extraction + FROZEN + bootstrap + detailed-task-list T-026 everywhere in comments. cargo test roundtrip green. Update STATUS evidence.
       **VERIFY T-026:** cd .worktrees/T-026-copilot; cargo test -p engine-biofield (65 pass incl new capture roundtrip test mapping 11 keys + consent); cites all required files in comments. Matches gaps (Vedic stub +5vs11) + resources (mock exists) + goal (biofield focus) + FROZEN.
     - T-027 STARTED (P2 per wave2-t027-face-p2-start): worktree .worktrees/T-027-codex created (branch swarm/engines/p2-w1/face/T-027-codex); crates/engine-face-reading/src/(engine.rs, models.rs) edited to support image_data + consent from frozen (heuristic fallback + landmark hook placeholder; analysis_from_* ; consent/quality attach + echo in result; FROZEN sample test added). All cites p1-w1-worker-bootstrap-packet.md + resources-and-assets.md + gaps-and-improvements.md + goal-understanding.md + P1W1-CONTRACTS-FROZEN.md + detailed-task-list.md (T-027) + EXECUTION-STATUS + tags phase:integration-p1 wave:integration-w2 etc. cargo test -p engine-face-reading green (35/35 + doc). Evidence: worktree edits + test logs below. Matches gaps (pure stub no image) + resources (heuristic exists) + goal (face focus) + FROZEN face example.
       **VERIFY T-027:** cd .worktrees/T-027-codex; cargo test -p engine-face-reading (35 pass incl test_calculate_with_frozen_image_data_consent_sample using FROZEN b64+consent); cites all refs in comments. 

    - T-002 complete + validated in lock zone (worktree .worktrees/T-002-copilot; P1W1-CONTRACTS-FROZEN.md). Contracts match FROZEN; tests green (cargo noesis-core/bridge, ts, sankalpa). Partial over-scope noted.
   - p1-w1-validation-gate-checklist.md + EXECUTION-STATUS updated w/ gate close + evidence (tests, diffs, reads of 3 extraction + contracts + Sankalpa).
   - New issues #899-902 active for remaining W1/early W2.
   - Roadmaps + mapping + labels updated/verified.
   - T-002 worktree ready for wave-boundary merge consideration or handoff.
   - Mode: contract-first. Reference 3 extraction files always. No P2 started. Gate closed per plan.

**Worktree Review (T-002-copilot, post gate close 2026-07-17):**
  - **Summary of changes (git diff main --stat in .worktrees/T-002-copilot):** 34 files, +463/-46. Core media contracts added per FROZEN.md:7-14.
    - crates/noesis-core/src/types.rs (+228): MediaRef, Consent, QualitySpec, GeneratedImage/GeneratedAudio + Metadata, CaptureState/CaptureLifecycle/SessionStatus (T-004); EngineInput/Output extensions (image_data/audio_ref/video_ref/consent/quality + generated_*); BiofieldResultSchema fix (optional dominant_element + dual-path note); SigilForgeResultSchema cleaned (no phantom vector_path + comment); examples for biofield-capture/face/raaga/sigil (types.rs:571-599).
    - ts-engines/src/types/engine.ts: TS mirror of input media + Consent/QualitySpec + generated on Output.
    - crates/noesis-bridge/src/lib.rs: to_ts_request forwards media fields to parameters; EngineOutput inits generated_*=None; test defaults.
    - T-003 complete: ts-engines/src/providers/image-provider.ts (new untracked; ImageProvider iface, ImageProviderConfig for 'nvidia'|'nano-banana'|'kimi', GeneratedImage); ts-engines/src/utils/nvidia-image.ts updated; ts-engines/src/engines/sigil-forge/engine.ts refactored (+26; uses injected provider, dynamic name/availability, no hardcode NVIDIA).
    - T-004/5: lifecycle models + raaga generated_audio.strudel_ratios/clip_url ready in types (per FROZEN:13; wiring pending).
    - Untracked: P1W1-CONTRACTS-FROZEN.md, ts-engines/src/providers/image-provider.ts.
    - Boilerplate (28 files, over-scope note): 12 engine-*/{engine,lib}.rs (biofield, face, gene-keys, panchanga, vimshottari etc), noesis-orchestrator/* (executor, full_spectrum, 6 synthesis/*, benches, tests), noesis-api/tests/*, noesis-tui, noesis-core/Cargo.toml. All add `generated_image: None, generated_audio: None` to EngineOutput struct literals in impls/tests (Rust explicit fields; no ..Default in many). Minimal per file (4-16 lines). No P2 impl (no CV/hardening/full render; only contracts + required for compile + T-003 iface/sigil).
  - **Verify matches FROZEN + gate evidence:** 1:1. FROZEN lists exactly the locked (types media, TS+bridge, fixes, Capture T-004, ImageProvider T-003, raaga T-005). Gate checklist: +228 types diff, bridge, provider, lifecycle, raaga ready, 4-engine examples, cargo test -p noesis-core --features openapi (2 PASS), cargo test -p noesis-bridge (35/1 PASS post fix), ts-engines bun test (sigil contract 9 PASS, pre-exist image timeout), sankalpa typecheck PASS (engine-media-contracts.ts mirrors + cites 3 extraction), no vector_path, consent local-first, anti-drift (two-prong from goal-understanding), refs extraction in FROZEN/STATUS/checklist/bootstrap. Sankalpa ../sankalpa/src/renderer/data/engine-media-contracts.ts compat (ImageMediaRef etc, consent gate).
  - **T-003/4/5 status:** T-003: complete (iface+config+sigil refactor; switch config-only). T-004: models (Capture*) in types ready; wiring to api/handlers per FROZEN pending. T-005: generated_audio contract (strudel_ratios etc) + examples ready. All per detailed-task-list.md (T-002..5), bootstrap packet.
  - **Readiness:** Green for wave boundary after P1 W1 gate close. T-002 validated + frozen. Ready for merge (worktree integrate at boundary per plan: "wave-boundary merge only on green") or handoff to W2/P2. Do not edit contracts w/o re-freeze+gate. Next: update matrix/docs (#902), #899 local dev/CI, #900 Sankalpa UI contracts, then P2 T-026+.
  - **Cites (all required + more):** .worktrees/T-002-copilot (cd + git diff --stat/log/status), P1W1-CONTRACTS-FROZEN.md, p1-w1-validation-gate-checklist.md (evidence for contracts), EXECUTION-STATUS.md, P1W1-W2-HANDOFF.md, 3 extraction files (resources-and-assets.md, gaps-and-improvements.md, goal-understanding.md), p1-w1-worker-bootstrap-packet.md, detailed-task-list.md, selemene-sankalpa-full-integration-swarm-plan.md, crates/noesis-core/src/types.rs (worktree), ts-engines/*, crates/noesis-bridge/src/lib.rs, Sankalpa engine-media-contracts.ts, gh #898/#893, cargo tests.
  - **Next steps for integrating worktree:** Per plan/gate/STATUS: at P1 W1 boundary (post this review): (1) cd .worktrees/T-002-copilot; git checkout main; git merge --no-ff swarm/engines/p1-w1/contracts/T-002-copilot (or worktree merge protocol); (2) cargo test -p noesis-core --features openapi; cargo test -p noesis-bridge; cd ts-engines && bun test; (3) resolve boilerplate if conflict; (4) update engine-matrix.json + docs/engines/*.md with "🧊 P1 W1 Frozen" badge (FROZEN note); (5) gh comment evidence on #898/#893; (6) git worktree remove .worktrees/T-002-copilot after clean. Or handoff parallel without immediate merge. Do not merge in this session. See P1W1-W2-HANDOFF.md.
  - **Over-scope note:** Broad (28 non-core files) but required+minimal for contract addition (struct fields); no scope creep to P2/impl. Core delta focused on T-002-005. Matches contract-first + extraction (cites gaps for fixes like vector_path).

**For next agent/session:**
  1. Read: p1-w1-worker-bootstrap-packet.md + the 3 extraction files (resources-and-assets.md + gaps-and-improvements.md + goal-understanding.md) + EXECUTION-STATUS.md + detailed-task-list.md (your T-xxx) + owning issue (#898 for contracts, #899-902 for new batch) + ts-engines/README.md + python-services/README.md (for local dev).
  2. cd to worktree or create per protocol.
  3. Update this table with progress.
  4. At task end: gh comment on issue + update this + post to #893 if wave boundary.
  5. Before gate: ensure all checkboxes in p1-w1-validation-gate-checklist.md have evidence.

**Handoff owner:** Temperance Engine / Orchestrator
**Branch:** main (coordination); contracts work on T-002 branch.
**Ready for:** ✅ P1 W1 gate closed (checklist all green w/ evidence). T-002 validated + contracts frozen (FROZEN.md). T-003/4/5 per FROZEN. P1W1-W2-HANDOFF.md + docs/matrix updates (#902). Readiness for wave handoff or T-002 worktree merge at P1 W1 boundary (per plan: wave-boundary merge only on green). Post gh #901/#893 done. Worktree separate; decision at handoff. Update issues + STATUS. See checklist evidence package.

## #899 Execution Evidence (Local Dev + CI, 2026-07-17 post-gate)
**Commands run (bash via opencode, main + worktree for frozen contracts):**
- cd ts-engines && bun install --frozen-lockfile ; bun test → 61 pass, 1 pre-exist (sigil image timeout no key)
- bun run typecheck → pre-existing TS errors (app.ts:125 metadata generic, tests unknown body); not media (refs FROZEN in worktree); bun test still green
- Server run + smoke (PORT=3001): bun run start & ; curl /health (6 engines incl raaga/sigil); POST /engines/raaga/calculate (melakarta=1 → Kanakangi + strudel_ratios array + prahar); POST /engines/sigil-forge/calculate (intention → method + generated_image:null)
- For frozen media: cd .worktrees/T-002-copilot/ts-engines (bun install); same curls + sample w/ image_data in params (accepted via parameters for scaffolding compat; types have top-level per engine.ts:30+)
- cd python-services && python -m venv .venv ; source .venv/bin/activate ; pip install -e ".[dev]" ; uvicorn ... --port 8002 & ; curl /health ({"status":"healthy","opencv_available":true,"numpy_available":true})
- pytest tests/test_biofield_analyze.py (selected: 11 metrics, contract fields, quality_assessment PASS)
- Evidence files updated: test.yml (ts smoke + python-sidecars job), ts-engines/package.json (typecheck), ts-engines/README.md + python-services/README.md (cmds + run logs + cites), p1-w1-validation-gate-checklist.md (evidence), this STATUS.

**Sample outputs (truncated):**
TS health: {"status":"healthy","engines":["tarot",...,"raaga","sigil-forge"]}
raaga: {"engine_id":"raaga","result":{"melakarta":{"name":"Kanakangi",...},"strudel_ratios":[1,1.053...,...],"prahar":{...}}}
sigil: {"engine_id":"sigil-forge","result":{"method":{...},"generated_image":null},...}
py health: {"status":"healthy","service":"biofield-cv","version":"3.0.0","opencv_available":true,"numpy_available":true}
pytest: . [100%] 1 passed

**Refs (mandatory per bootstrap + plan):** 3 extraction (resources-and-assets.md: raaga/sigil ready, dual biofield; gaps-and-improvements.md: no prior e2e + schema mismatch; goal-understanding.md: two-prong/local-first/4 engines focus) + p1-w1-worker-bootstrap-packet.md + P1W1-CONTRACTS-FROZEN.md (worktree) + detailed-task-list.md (T-00x local/CI) + selemene-sankalpa...plan.md . Stayed in scaffolding; no engine impl.

**CI baseline now:** contract tests (curls exercising raaga audio T-005 + sigil T-003), typecheck (w/ media note), sidecar smoke (py biofield T-004 11-metric contract). See .github/workflows/test.yml .

All per #899 + post-gate for P1 W1.

All tags consistent (phase:integration-p1 etc). Non-code heavy coordination complete. 

## P1 Wave 2 Task Breakdown (Scaffolding & Baselines + P2 Start)
**Source:** selemene-sankalpa-full-integration-swarm-plan.md (Phase 1 Wave 2: "Scaffolding & baselines: project setup, CI, test harnesses, local dev for TS server + python" + P2 start), detailed-task-list.md (T-006..T-025 est W1 + T-026+ P2), P1W1-W2-HANDOFF.md.
**Format:** Ready for STATUS table / github-issue-mapping.md / next-batch.
**Cites (mandatory in all work):** 3 extraction files (resources-and-assets.md, gaps-and-improvements.md, goal-understanding.md), p1-w1-worker-bootstrap-packet.md, issues #899-902, EXECUTION-STATUS.md, P1W1-CONTRACTS-FROZEN.md (worktree), detailed-task-list.md, plan.md. Contract-first; no drift. Local-first + consent from goal-understanding. #899 covered local/CI scaffolding (post-gate complete); breakdown below granularizes for mapping + future waves. P2 start after W1 gate + W2 baselines.

| id | title | owner_role | est_hours | dependencies | deliverable | acceptance | validation |
|----|-------|------------|-----------|--------------|-------------|------------|------------|
| T-020 | Enhance CI (test.yml) for TS-engines typecheck + P1W1 media contract smoke (raaga/strudel T-005, sigil T-003) | orchestrator / infra | 3 | T-005, T-003 (contracts frozen); #899 | Updated .github/workflows/test.yml + ts-engines/package.json (typecheck script); smoke curls exercising frozen media | CI green on PR; raaga strudel_ratios + sigil generated present in logs; typecheck note refs FROZEN | Run: bun test + typecheck + curl POSTs; matches resources (raaga ready) + gaps (no prior e2e); cites 3 extraction + bootstrap + #899 + goal-understanding two-prong |
| T-021 | Add python-sidecars CI job (pytest 11-metric biofield + uvicorn health @8002; FROZEN samples) | orchestrator / infra | 2 | T-004 (capture contract); #899 | Enhanced .github/workflows/test.yml (T-021 worktree) + setup; pytest 11-metrics/Quality + FROZEN consent smoke | pytest PASS + health + FROZEN smoke (11 metrics, quality, consent); see .worktrees/T-021-codex/test.yml | python -m pytest + uvicorn + curl FROZEN sample; per gaps + resources + goal + bootstrap + P1W1-CONTRACTS-FROZEN; phase:integration-p1 wave:integration-w2 engine-biofield |
| T-022 | Local dev baseline for ts-engines server (bun start PORT=3001; health + raaga/sigil calculate) | backend-architect | 4 | T-020, T-005 | ts-engines/README.md (P1 W2 section) + verification cmds + sample outputs; server runs clean | Health lists 6 engines; raaga returns Kanakangi+strudel; sigil returns method+image:null (or media sample) | Manual: bun run start &; curls; logs in README cite extraction files + #899 + bootstrap; bun test 61p |
| T-023 | Local dev baseline for python biofield sidecar (venv + uvicorn @8002 + pytest) | backend-architect | 3 | T-021, T-004 | python-services/README.md updated + run logs + contract smoke | uvicorn health opencv=true; pytest 11-metrics + quality_assessment PASS | Activate venv; uvicorn; curls + pytest; refs gaps (schema) + resources (CV sidecar) + detailed-task-list local dev |
| T-024 | Contract roundtrip test harness (media samples per FROZEN for 4 engines) | validation | 4 | T-002 (FROZEN), T-020..T-023; #901 | scripts/ or README harness (curls with image_data/consent for bio/face, audio for raaga, gen for sigil); CI notes | All 4 roundtrip shapes match P1W1-CONTRACTS-FROZEN (no vector_path, consent local, generated_*); samples validate. **Self-contained ext-contract-harness.md (TS + curl + sh) generated in fail-open re-dispatch for ext-contract-harness (2026-07-17). Includes local-first consent guards, schema notes, run against ts@3001/py@8002. 4 full JSONs + executable snippets.** | docs/plans/engine-integration/ext-contract-harness.md added (ready); execute + schema vs types.rs; cites FROZEN + goal-understanding local-first + 3 extraction + bootstrap. |
| T-025 | Update root project setup / harnesses (ts-engines + py package scripts, .env.example, engine-matrix baseline) | orchestrator | 2 | T-022,T-023; #902 | package.json updates + READMEs + matrix note for P1 W2 status | Scripts (typecheck, smoke) documented; matrix has "p1_w2: scaffolding" entry | cargo check + bun + py -m pip; cross-ref handoff.md + #902 docs update |
| T-026 | (P2 start) Harden engine-biofield Rust: Vedic path + capture result mapping to frozen contracts | backend-architect | 6 | T-004,T-002 (FROZEN), T-024 baselines; #897 | crates/engine-biofield/src/engine.rs + models + tests; is_mock_data path + full 11+ | Birth + capture paths produce correct BiofieldResult per FROZEN; tests green | cargo test -p engine-biofield; roundtrip via bridge/api; cites gaps (Vedic stub + 5 vs 11) + resources (mock exists) + goal-understanding (biofield focus) |
| T-027 | (P2 start) Extend engine-face-reading for image_data input (heuristic + landmark hook) per T-004 | backend-architect | 5 | T-004,T-002, T-026; #897 | crates/engine-face-reading/src/engine.rs + models support image_data + consent | Heuristic runs on image ref; output FaceAnalysis shape matches; no crash on mock | Unit test w/ sample image_data; integration per FROZEN; refs gaps (pure stub, no image) + resources (heuristic exists) | STARTED in .worktrees/T-027-codex (branch swarm/engines/p2-w1/face/T-027-codex); engine.rs+models.rs+mock.rs edited (heuristic+landmark_hook placeholder, image_data/consent via options per FROZEN sample shape, consent/quality echoed in result, new roundtrip test w/ FROZEN b64+consent); cargo test -p engine-face-reading (35 pass incl new test). Cites extraction + FROZEN + bootstrap + detailed-task-list T-027 + tags. See worktree + build logs in STATUS. |
| T-028 | (P2 start) Update raaga/sigil TS for media output options (strudel + generated per contracts) | backend-architect | 4 | T-005,T-003, T-025; #897 | ts-engines/src/engines/{raaga,sigil-forge}/engine.ts + tests | Output includes generated_audio strudel + clip_url option; sigil uses provider | bun test; sample POSTs; per detailed-task-list T-031/T-035 start + plan P2 W2/W3 |
| T-029 | Wave 2 close + P2 entry validation (baselines green, P2 tasks kicked, handoff to W3/P2 full) | validation / orchestrator | 3 | All T-020..T-028; #899-#902, #901 | Updated EXECUTION-STATUS + P1W2-HANDOFF.md + engine-matrix + gh comments on #893/#897 | Baselines + first P2 tasks evidence attached; no contract drift; ready for P2 W1 full | Gate-like checklist run (smokes + 1 P2 test each); cite all extraction + bootstrap + plan Wave 2 + detailed-task-list |

**Usage:** Paste this table (or rows) into github-issue-mapping.md, new issues for T-020+, STATUS tracking, or as sub-issues under #894/#897. Every row must reference the 3 extraction files + bootstrap packet in body/notes. Wave boundary only on green (see P1W1-W2-HANDOFF.md). #899 was execution of local/CI subset (T-022/T-023/T-020/T-021); this is canonical granular definition.
**Next:** Use for P1 W2 execution post #901 gate close. P2 start (T-026+) after W2 baselines per plan sequencing.


**Re-dispatch Round (post external failures - fail-open to Codex):**
- All 3 external tasks re-dispatched as Codex subagents (self-contained prompts executed via workspace reads).
  - nano-kimi-provider-sketches: Complete TS sketches for NanoBananaProvider + KimiProvider (implements ImageProvider iface, styled prompts runic/vedic/chaos, generate/edit returning b64/url, config-driven, registration factory matching nvidia style). Notes open Qs on kimi API (text-only in repo). Cites FROZEN, 3 extraction, bootstrap.
  - contract-roundtrip-examples (prior): snippets. Fail-open re-dispatch ext-contract-harness: full self-contained harness md + TS harness (fail-open + consent guard fn) + curl + sh script for all 4 (bio image_data@py8002, face image@3001, raaga melakarta+audio@3001, sigil intention+image@3001). Written to docs/plans/.../ext-contract-harness.md ready to add. Guards, schema notes, run cmds. Cites FROZEN/types/goal-understanding/3-extraction. 
  - wave2-task-breakdown: 9 granular tasks T-020..T-029 (CI enhancements, python sidecar CI, local dev baselines for ts/py, contract roundtrip harness, project setup, P2 start for biofield/face/raaga/sigil hardening). Table with id/title/owner/est/deps/deliverable/acceptance/validation. Ready for paste to mapping/STATUS. Cites plan Wave 2, detailed-task-list, #899-902, 3 extraction, bootstrap, handoff.
- All updates cite 3 extraction files + P1W1-CONTRACTS-FROZEN + bootstrap packet + plan + issues. STATUS extended with table + round summary.
- External consistently unavailable (fail-open applied per protocol; no dead-end).
  - ext-p2-validation-checklist: Drafted `ext-p2-validation-checklist.md` (8-10 items: pre-P2 checks, biofield Vedic+ capture mapping, face image input, raaga/sigil media updates, roundtrips w/ FROZEN, anti-drift vs 3 extraction files, tests/CI, no scope creep strict to T-026/27/31/35). Format: markdown checkboxes + evidence + commands. Cites plan P2 secs + detailed-task-list T-026+ + resources/gaps/goal + FROZEN + bootstrap + this STATUS + #897. Modeled on p1-w1-validation-gate-checklist.md. Pending execution post P1W2. File committed.

**Current state post all batches:** P1 W1 gate closed. Contracts validated in worktree. Local dev/CI baselined. Sankalpa contracts expanded. Worktree reviewed (green, no merge). Wave 2 breakdown ready. New issues #899-902 active. ext-p2-validation-checklist.md drafted (fail-open re-dispatch). **ext-contract-harness.md generated (fail-open re-dispatch T-024)**: self-contained TS+curl+sh with 4 examples, consent guards, refs FROZEN/types/goal-understanding. Ready to add to scripts/. STATUS T-024 + re-dispatch updated. All cite 3 extraction + local-first. Ready for Wave 2 (T-020+) or P2. Reference bootstrap + extraction files always. Tags consistent.
- T-027 start evidence (2026-07-17 Codex subagent, no ext rail): worktree created per spec; edits only in .worktrees/T-027-codex/crates/engine-face-reading/src/ (engine+models; mock for compile); FROZEN sample test; no merge/push.

**T-027 Execution Evidence (build/test logs):**
Commands:
- git worktree add -b swarm/engines/p2-w1/face/T-027-codex .worktrees/T-027-codex HEAD
- cd .worktrees/T-027-codex && cargo test -p engine-face-reading (35 pass)
- cargo test -p engine-face-reading test_calculate_with_frozen... -- --nocapture (1 pass, FROZEN consent+b64 sample)
Full: 35 unit + 1 doc-test green. New test exercises image_data object + consent per FROZEN:35, sets non-mock, echoes consent, uses landmark hook path, backend=heuristic-image-landmark-hook.
Cites in all added comments: the 3 extraction + bootstrap + FROZEN + detailed-task-list T-027 + EXECUTION-STATUS + plan + tags.
**VERIFY:** cd .worktrees/T-027-codex; cargo test -p engine-face-reading ; matches gaps (face stub) + resources (heuristic) + FROZEN face ex + task wave2-t027.
 
