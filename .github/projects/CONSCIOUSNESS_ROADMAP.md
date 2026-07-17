# Consciousness Engine Roadmap - GitHub Project Configuration

## Project Overview
This GitHub Project tracks the development of all consciousness engines and integration features in Selemene Engine.

## Interactive Timeline
- HTML roadmap artifact: [selemene-roadmap-timeline.html](../../selemene-roadmap-timeline.html)
- Recommended render in Craft Agent: use an `html-preview` block pointing to the absolute file path.

## Project Structure

### Views

#### 1. **By Engine** (Board View)
- Columns:
  - 🔴 Not Started
  - 🟡 In Progress
  - 🟢 Complete
  - 🔵 Validated
  - ⚫ Blocked
- Group by: `engine` label

#### 2. **By Wave/Phase** (Table View)
- Group by: `wave` label (wave1, wave2, wave3, wave4)
- Sort by: Priority
- Columns: Title, Status, Engine, Assignee, Priority, Due Date

#### 3. **By Status** (Board View)
- Columns: Backlog, Todo, In Progress, In Review, Done
- Auto-archive: Done items after 30 days

#### 4. **Timeline** (Roadmap View)
- Show milestones and releases
- Group by: Quarter
- Show dependencies

## Labels

### Engine Labels
- `engine-human-design` - Human Design calculations
- `engine-gene-keys` - Gene Keys system
- `engine-vimshottari` - Vimshottari Dasha
- `engine-panchanga` - Vedic Panchanga
- `engine-numerology` - Numerology system
- `engine-biorhythm` - Biorhythm calculations
- `engine-vedic-clock` - Vedic time systems
- `engine-biofield` - Biofield analysis
- `engine-face-reading` - Face reading system
- `engine-integration` - Integration layer
- `engine-core` - Core engine functionality

### Wave Labels
- `wave1` - Sprint 1 & 2 (Auth, Docker, CORS, Rate Limiting, Health)
- `wave2` - HD, Gene Keys, Vimshottari (complete)
- `wave3` - Vedic Clock enhancements
- `wave4` - Remaining engines + advanced features

### Type Labels
- `type-feature` - New feature
- `type-bug` - Bug fix
- `type-enhancement` - Improvement to existing feature
- `type-docs` - Documentation
- `type-test` - Testing
- `type-refactor` - Code refactoring
- `type-performance` - Performance improvement

### Priority Labels
- `priority-critical` - Blocking issue
- `priority-high` - High priority
- `priority-medium` - Medium priority
- `priority-low` - Low priority

### Status Labels
- `status-blocked` - Blocked by dependency
- `status-needs-review` - Needs code review
- `status-needs-testing` - Needs testing
- `status-needs-docs` - Needs documentation

## Automation Rules

### Auto-archive
- When: Status = "Done"
- After: 30 days
- Condition: No recent activity

### Auto-label
- When: Issue created with title starting with "[ENGINE]"
- Then: Add `new-engine` label

### Auto-assign
- When: PR created
- Then: Assign to PR creator
- Add to project

### Status sync
- When: PR merged
- Then: Move linked issues to "Done"

## Milestones

### Active Milestones
- **v2.1.0 - Integration Layer** (Current)
  - Due: Q1 2026
  - Focus: Vedic API integration, enhanced Vedic Clock
  
- **v2.2.0 - Specialized Engines** (Next)
  - Due: Q2 2026
  - Focus: Numerology, Biorhythm, Biofield
  
- **v3.0.0 - Platform Launch** (Future)
  - Due: Q3 2026
  - Focus: Complete platform with all engines

### Completed Milestones
- ✅ v2.0.0 - Wave 2 Complete (HD, Gene Keys, Vimshottari)
- ✅ v1.0.0 - Wave 1 Complete (Core API, Auth, Docker)

## Fields

### Custom Fields
- **Engine**: Single select (list of engines)
- **Wave**: Single select (wave1, wave2, wave3, wave4)
- **Priority**: Single select (Critical, High, Medium, Low)
- **Effort**: Number (story points or hours)
- **Test Coverage**: Number (percentage)
- **Dependencies**: Text (list of dependent issues)

## Integration with JSON Plans

The `docs/planning/*.json` taskmaster plan files map to GitHub issues:

1. **Load JSON plans** into GitHub Issues
2. **Create issues** for each task
3. **Link issues** to project
4. **Set metadata** (labels, milestone, assignee)
5. **Track progress** in project views

## Team Workflow

1. **Planning**: Create issues from JSON task plans
2. **Development**: Move to "In Progress", create feature branch
3. **Review**: Create PR, auto-linked to issue
4. **Testing**: Mark "Needs Testing", run test suite
5. **Merge**: PR merged → Issue auto-closed → Moved to "Done"
6. **Release**: Milestone complete → Create GitHub Release

## Queries

### Useful filters:
```
# All blocked issues
is:issue is:open label:status-blocked

# High priority in current wave
is:issue is:open label:priority-high label:wave3

# Engine-specific backlog
is:issue is:open label:engine-vedic-clock no:assignee

# Ready for review
is:pr is:open label:status-needs-review

# Stale items
is:issue is:open updated:<2026-01-01
```

## Setup Commands

### Create labels
```bash
gh label create "engine-human-design" --color "FF6B6B"
gh label create "wave1" --color "4ECDC4"
gh label create "priority-critical" --color "FF0000"
# ... (see full script in scripts/setup-github-labels.sh)
```

### Create project
```bash
gh project create "Consciousness Engine Roadmap" \
  --owner Sheshiyer \
  --body "Development tracking for all consciousness engines"
```

### Import tasks from JSON
```bash
# Canonical script
scripts/sync-plans-to-github-issues.sh --apply --repo Sheshiyer/Selemene-engine

# Backward-compatible wrapper
scripts/import-tasks-to-github.sh --apply --repo Sheshiyer/Selemene-engine
```

## Selemene + Sankalpa Engine Integration Plan (2026)

**Epic:** #893  
**Focus:** Two-prong integration of media-heavy consciousness engines (biofield/BV-PIP capture, face-reading, raaga, sigil-forge outputting runic/vedic sigils/yantras) + general engines.

- **Prong 1 (Core):** Selemene-engine — complete Rust/TS engines, bridges (noesis-bridge for TS), python CV sidecars, image gen providers (extend to nano-banana + kimi via API), core contracts for media I/O, SDK, API exposure.
- **Prong 2 (Frontend):** Sankalpa (Electron) — local-safe media capture (camera for biofield/face, file for sigil), rich output rendering (mandalas, swara wheels, image viewers), engine consumption surfaces, integration into Noesis workflows.

**Phases & Tags (aligned with existing flow):**
- phase:integration-p1 : Contracts, Discovery & Foundation (see #894)
- phase:integration-p2 : Selemene Core Engine Hardening
- phase:integration-p3 : Infrastructure & Providers (nano-banana, kimi, CV)
- phase:integration-p4 : API / Bridge / Orchestration
- phase:integration-p5 : Sankalpa Frontend Media I/O + Engine Surfaces (see #895)
- phase:integration-p6 : Integration, Testing, Witness Flows
- phase:integration-p7 : Validation, Hardening, Rollout
- Waves: wave:integration-w1, w2, ... per phase
- Swarms: swarm:selemene-backend, swarm:sankalpa-frontend, etc.

**Labels in use (consistent with project):**
- `engine-biofield`, `engine-face-reading`, `engine-raaga`, `engine-sigil-forge`
- `area:engine-integration`, `area:backend`, `area:frontend`
- `swarm:selemene-backend`, `swarm:sankalpa-frontend`
- `phase:integration-p1`, `phase:integration-p5` (extending phase:Px pattern)
- `wave:integration-w1` etc.
- `roadmap`, `enhancement`

**Plan artifacts:**
- Full Swarm Architect plan: `docs/plans/engine-integration/selemene-sankalpa-full-integration-swarm-plan.md`
- Deepened pre-execution extraction (2026-07-17, before any code):
  - `docs/plans/engine-integration/resources-and-assets.md` (what exists: code, docs, prior work)
  - `docs/plans/engine-integration/gaps-and-improvements.md` (stubs, mismatches, wiring gaps)
  - `docs/plans/engine-integration/goal-understanding.md` (objective, success criteria, drift risks)
- Discovery: `docs/plans/engine-integration/discovery-summary.md`
- Detailed tasks (130+ granular): `docs/plans/engine-integration/detailed-task-list.md`
- GitHub mapping: `docs/plans/engine-integration/github-issue-mapping.md`

**GitHub Mapping Strategy:**
- One issue per key phase/wave or task bundle.
- All use the tags above for filtering in project views (By Wave/Phase, By Engine).
- Dependencies tracked in issue bodies.
- Validation evidence required before close (per swarm-architect playbooks).

**Current Status (as of 2026-07-17):** 
- Deepened pre-execution analysis complete (resources, gaps, goal extracted from all prior work to prevent drift).
- All planning artifacts (plan + 3 extraction files + mapping + task list) committed to repo.
- GitHub issues: #893 (epic), #894 (P1), #895 (P5) + new issues for deepened artifacts + phase waves.
- Labels/tagging standardized (phase:integration-*, wave:integration-*, swarm:*, engine-*, area:engine-integration).
- Next: Roll out remaining phase issues from detailed-task-list, begin P1 contract work (wave-boundary merges only).
- Sankalpa side: see sibling repo ROADMAP Milestone 4b.
- No stale planning docs for this initiative (old web-app refs are historical/retired and correctly annotated in engine docs).

See also sankalpa/ROADMAP.md for frontend-specific milestones.
