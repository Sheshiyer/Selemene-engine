# GitHub Issue Mapping for Engine Integration Plan

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
- New: Deepened pre-execution artifacts issue (resources-and-assets, gaps-and-improvements, goal-understanding committed + linked)
- Phase/wave issues to be rolled out from detailed-task-list (P2-P7 + key contracts)

**Deepened Artifacts (now primary anti-drift references):**
- resources-and-assets.md — inventory of all existing code/docs/prior work
- gaps-and-improvements.md — concrete gaps (mocks, contract mismatches, missing wiring, providers)
- goal-understanding.md — locked objective + success + two-prong model

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

**Next (post-deepening):** 
- Create dedicated issue for the 3 extraction files + update this mapping with new numbers.
- Roll out phase issues for P2-P7.
- Begin P1 W1 contract tasks (T-002 etc) as issues.
- Update this file after each gh create.

**Project Views:** Use filters like `label:phase:integration-p1 label:area:engine-integration` in the Consciousness Engine GitHub Project.
