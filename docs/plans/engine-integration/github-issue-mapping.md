# GitHub Issue Mapping for Engine Integration Plan

**Execution Start Note (2026-07-17, P1 W1 continued):** P1 Wave 1 GitHub/scaffolding/validation prep + 4 new issues for remaining tasks executed (see EXECUTION-STATUS.md). Contracts work started (T-002 complete in worktree; gate drafted). All updates use phase:integration-p1 / wave:integration-w1 etc. Cites 3 extraction files (resources-and-assets.md, gaps-and-improvements.md, goal-understanding.md). See updated issues #893/#896/#898/#899/#900/#901/#902 + bootstrap p1-w1-worker-bootstrap-packet.md, checklist p1-w1-validation-gate-checklist.md, STATUS, FROZEN.

**Epic:** #893 — [PLAN] Selemene + Sankalpa Full Engine Integration

**Tags used (to fit existing project flow):**
- phase:integration-p1 (P1: Foundation/Contracts)
- phase:integration-p2 , phase:integration-p5 (key heavy phases)
- wave:integration-w1 etc.
- area:engine-integration , area:backend , area:frontend
- swarm:selemene-backend , swarm:sankalpa-frontend
- engine-biofield , engine-face-reading , engine-raaga , engine-sigil-forge
- roadmap , enhancement

**Created Issues (as of 2026-07-17):**
  - #893: Main plan epic (links to docs/plans/engine-integration/* + deepened files)
  - #894: P1 Contracts & Foundation (see detailed-task-list.md for T-001..)
  - #895: P5 Sankalpa Frontend Media + Surfaces
  - #896: Deepened pre-execution artifacts (resources-and-assets.md, gaps-and-improvements.md, goal-understanding.md committed + linked)
  - #897: P2 Selemene Core Engine Hardening
  - #898: P1 W1 Media Contracts & Provider Abstraction (T-002..T-005)
  - #899: P1 W1 infra — Local Dev Setup + CI Baselines (T-0xx)
  - #900: P1 W1 sankalpa-frontend — Define Sankalpa media UI contracts (T-006 est.)
  - #901: P1 W1 validation — Execute P1 Wave 1 Validation Gate Checklist + evidence
  - #902: P1 W1 / early W2 docs — Update per-engine docs + engine-matrix + handoff notes
  - More phase/wave/task issues to be rolled out from detailed-task-list (P3-P7 + remaining)

**P1 W1 GitHub/Scaffolding/Validation Prep + Batch Issues Complete (2026-07-17 coordination task):**
  - Updated #893, #896, #898, #899, #900, #901, #902 with status + links to all plans + 3 extraction files (resources-and-assets.md, gaps-and-improvements.md, goal-understanding.md) + new scaffolding (bootstrap, checklist, STATUS, FROZEN).
  - Created: p1-w1-worker-bootstrap-packet.md (context/contracts/agent instructions/current state)
  - Created: p1-w1-validation-gate-checklist.md (cross-ref external attempt; now canonical)
  - Created: EXECUTION-STATUS.md (tracks started tasks, owners, branches)
  - Labels verified present and consistent (phase:integration-p1, wave:integration-w1, swarm:*, area:engine-integration, engine-* etc.)
  - Roadmaps + this mapping updated with execution start note.
  - T-002 worktree active on contracts (lock zone; P1W1-CONTRACTS-FROZEN.md); T-003.. ready; new issues #899-902 for remaining P1 W1 / early W2 (local dev/CI, Sankalpa contracts, validation exec, docs/handoff). Gate drafted.

**Deepened Artifacts (now primary anti-drift references):**
- resources-and-assets.md — inventory of all existing code/docs/prior work
- gaps-and-improvements.md — concrete gaps (mocks, contract mismatches, missing wiring, providers)
- goal-understanding.md — locked objective + success + two-prong model

**P1 Scaffolding Artifacts (created/updated 2026-07-17):**
- p1-w1-worker-bootstrap-packet.md — full context for agents on P1 W1 (load at every session start)
- p1-w1-validation-gate-checklist.md — Phase 1 Wave 1 gate (contracts + anti-drift + evidence)
- EXECUTION-STATUS.md — live tracking of started tasks, owners, branches/worktrees, handoff notes

**Mapping Strategy (per swarm-architect/playbooks/github-sync.md and plan-to-github.md):**
- Tasks from detailed-task-list.md map 1:1 or bundled to issues.
- Title format: `[P1][W1][swarm] T-xxx — Title`
- Labels: phase:integration-p1, wave:integration-w1, swarm:xxx, area:xxx, engine-xxx, agent:xxx (when assigned)
- Body uses github-issue-template.md structure: Context, Deliverable, Acceptance, Validation, Dependencies, Execution Envelope (branch/worktree), Completion Protocol.
- Dependencies listed in body + checklists.
- Wave summaries posted as comments on epic or phase issues.
- PRs reference owning task/issue.
- All new planning docs linked in epic and referenced in roadmap files.

**Full Task to Issue Rollout:**
Use the detailed-task-list.md (130+ tasks) to batch-create via gh or scripts.
Example for T-002 (media contracts):
gh issue create --title "[P1][W1][contracts-backend] T-002 — Freeze EngineInput/EngineOutput media extensions" --label "phase:integration-p1,wave:integration-w1,swarm:selemene-backend,area:backend,engine-biofield,engine-raaga" ...

See main plan for complete list and how daily work decomposes from waves.

**Next (post P1 W1 prep + this batch):** 
  - Complete remaining P1 W1 contracts (T-003..T-005 in #898) + run validation gate execution (#901, see p1-w1-validation-gate-checklist.md + #898).
  - Execute local dev/CI (#899), Sankalpa UI contracts (#900), docs/handoff/matrix (#902).
  - Roll out phase issues for P2-P7 from detailed-task-list (as waves complete).
  - Update EXECUTION-STATUS.md + this mapping + issues on every handoff.
  - Begin P1 contract tasks in worktrees (T-002 complete; see FROZEN).
  - Update this file after each gh create / status change.
  - All cite the 3 extraction files + bootstrap/checklist/STATUS.

**Project Views:** Use filters like `label:phase:integration-p1 label:area:engine-integration` in the Consciousness Engine GitHub Project.
