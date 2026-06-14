# Swarm Plan — humdes Validation Hardening

> Generated via Swarm Architect skill from the four follow-ups identified by
> the initial humdes ground-truth validation (89-person corpus, see
> [`tests/fixtures/humdes/README.md`](../../tests/fixtures/humdes/README.md)).
>
> Source skill: `swarm-architect`
> Companion playbooks: `multi-agent-boundaries.md`, `worktree-strategy.md`,
> `verification-gates.md`, `github-sync.md`

## 1. Discovery Summary

| Field | Value |
|---|---|
| Planning depth | standard |
| Delivery mode | production (touches engine validation + persistent storage) |
| Release model | phased rollout (4 issues → 1 milestone) |
| Quality bar | tests pass, validation report stays ≥ current baseline, new fields added are measurable |
| Team / agent topology | small squad — 1 human lead + 4 specialist coding agents |
| Coding agents available | claude (orchestrator), codex (impl), copilot (cloud/backend), gemini (validation/QA) |
| External constraints | no external dependencies; all work fits in the existing workspace |

## 2. Assumptions and Constraints

- **A1** — The humdes capture pipeline (`~/Downloads/humdes-extractor/`) is stable. New sub-tab capture extends it; it doesn't replace it.
- **A2** — Selemene engine API (`EngineInput` + `HumanDesignEngine::calculate`) is the integration contract. No changes to that contract are in scope.
- **A3** — Test fixtures live at `tests/fixtures/humdes/` and that path is canonical (referenced from Rust tests + Python normaliser).
- **C1** — Cannot modify `claude/depth-reading` branch work (parallel WIP).
- **C2** — Issues numbered 853-856; no other validation issues open. Don't conflate with the unrelated `plan-admin-panel-taskmaster-plan-2026-02-25` epic.
- **C3** — Validation report must remain runnable in <2 minutes (current ~7s with 89 fixtures).

## 3. Agent Ownership Model

| Concern | Primary owner | Issue | Secondary reviewer |
|---|---|---|---|
| MG/G investigation + decision | **Claude (orchestrator)** — judgment-heavy, holistic | #853 | Human lead |
| Sub-tab capture + parser extension | **Codex (impl)** — Python script extension | #854 | Claude |
| Cross-engine validation tests | **Copilot (cloud/backend)** — pattern replication | #855 | Gemini |
| Tier-2 storage migration + writer | **Gemini (validation/qa)** — schema + feature gating | #856 | Copilot |

Rationale: choosing owners by the **dominant skill** the task needs, not by stack alignment. The MG/G investigation is judgement and code-reading; sub-tab capture is web-scraping code; the cross-engine tests are pattern application; the storage migration is schema design and careful gating.

## 4. Phase Map

### Phase 1 — Hardening (4 issues, 2 waves)

Single phase. All work fits within one milestone (no v-bump, no cross-cutting refactor).

**Goal:** raise humdes validation from 8 fields covering 1 engine to 11+ fields covering 3 engines, with results persisted for trend tracking.

**Exit criteria:**
- All 4 issues closed with merged PRs.
- `cargo test --package engine-human-design --test humdes_validation_tests -- --ignored --nocapture` shows ≥11 fields evaluated; type at 100% OR documented divergence.
- `cargo test --package engine-panchanga` and `--package engine-numerology` include passing humdes_smoke_tests.
- At least one row exists in `humdes_validation_runs` table (proof of writer).

### Wave 1 — Investigation + extraction (parallel)

Two swarms run fully in parallel; no shared files; no contract changes.

#### Swarm A — Engine-side decision (#853, MG/G)
- **Owner:** Claude
- **Branch:** `swarm/humdes-validation/p1-w1/engine/853-claude`
- **Worktree:** `.worktrees/853-claude`
- **Edit surface:**
  - `crates/engine-human-design/src/analysis.rs` (only if fix needed)
  - `tests/fixtures/humdes/README.md` (always, to document outcome)
- **Inputs:** 7 mismatching fixtures listed in #853, `analysis.rs::determine_type`, mechanics-tab HTML bodies in `~/Downloads/humdes-extractor/output/<latest>_bulk2/`
- **Outputs:** EITHER a code fix that brings type validation to 100% OR a documented divergence in the README
- **Validation:** re-run `humdes_engine_full_validation_report` and assert outcome matches commit message

#### Swarm B — Capture-side extraction (#854, sub-tabs)
- **Owner:** Codex
- **Branch:** `swarm/humdes-validation/p1-w1/extractor/854-codex`
- **Worktree:** `.worktrees/854-codex`
- **Edit surface:**
  - `~/Downloads/humdes-extractor/bulk_fetch_v2.py` (extend OR fork to v3)
  - `~/Downloads/humdes-extractor/humdes_to_selemene.py` (consume new fields)
  - `tests/fixtures/humdes/_index.json` (regenerated)
  - `tests/fixtures/humdes/readings/**/*_expected.json` (regenerated)
  - `tests/fixtures/humdes/README.md` (note new fields)
- **Inputs:** ravecard `tabs[*].childs` tree (described in issue), existing tab JSONs in latest bulk2 folder
- **Outputs:** ≥3 new ground-truth fields (`definition`, `active_channels`, `defined_centers`) populated for all 89 persons
- **Validation:** updated `_index.json` shows new fields populated; HD validation re-run quantifies match on new fields

**No shared edits.** Swarm A touches only engine source + README. Swarm B touches only the extractor (outside the repo) + regenerated fixtures + README. README is the **lock zone** for this wave — serialise the README touch via the wave-close commit (see Verification).

### Wave 2 — Coverage expansion (parallel)

Wave 1 must complete before Wave 2 — Wave 2's storage writer references field names locked by Wave 1.

#### Swarm A — Cross-engine smoke (#855, panchanga + numerology)
- **Owner:** Copilot
- **Branch:** `swarm/humdes-validation/p1-w2/engines/855-copilot`
- **Worktree:** `.worktrees/855-copilot`
- **Edit surface:**
  - `crates/engine-panchanga/tests/humdes_smoke_tests.rs` (new file)
  - `crates/engine-numerology/tests/humdes_smoke_tests.rs` (new file)
  - `tests/fixtures/humdes/README.md` (Tier-4 section)
- **Inputs:** existing `tests/humdes_validation_tests.rs` as template
- **Outputs:** both engines get a passing smoke test + an ignored long-running report
- **Validation:** both `cargo test --package engine-{panchanga,numerology}` pass; per-engine reports print

#### Swarm B — Tier-2 storage (#856, drift tracking)
- **Owner:** Gemini
- **Branch:** `swarm/humdes-validation/p1-w2/storage/856-gemini`
- **Worktree:** `.worktrees/856-gemini`
- **Edit surface:**
  - `crates/noesis-data/migrations/<timestamp>_humdes_validation.sql` (new file)
  - `crates/noesis-data/src/humdes_validation.rs` (new module + writer)
  - `crates/noesis-data/src/lib.rs` (re-export)
  - `crates/noesis-data/Cargo.toml` (add `record-validation` feature)
  - `tests/fixtures/humdes/README.md` (Tier-2 section: add trend query)
- **Inputs:** existing noesis-data migration patterns + the schema in #856
- **Outputs:** migration applies cleanly, writer compiles under feature flag, README has trend query
- **Validation:** `cargo check --package noesis-data --features record-validation` passes; migration up + down both work in a scratch Postgres

## 5. Detailed Phase 1 Wave Layout

See section 4 above. All Phase-1 work fits in 2 waves with 2 swarms each.

## 6. Task List

Tasks are 1:1 with GitHub issues — already filed at issues
[#853](https://github.com/Sheshiyer/Selemene-engine/issues/853),
[#854](https://github.com/Sheshiyer/Selemene-engine/issues/854),
[#855](https://github.com/Sheshiyer/Selemene-engine/issues/855),
[#856](https://github.com/Sheshiyer/Selemene-engine/issues/856).

| Task ID | Issue | Owner | Wave | Swarm | Status |
|---|---|---|---|---|---|
| HV-T01 | #853 | claude  | W1 | A engine | done (merged via PR #858) |
| HV-T02 | #854 | codex   | W1 | B extractor | done (merged via PR #859) |
| HV-T03 | #855 | copilot | W2 | A engines | done (merged via PR #860) |
| HV-T04 | #856 | gemini  | W2 | B storage | done (merged via PR #861) |

Per-task agent handoff packets in `.swarm/humdes-validation/handoffs/`:
- [`HV-T01-claude.md`](handoffs/HV-T01-claude.md)
- [`HV-T02-codex.md`](handoffs/HV-T02-codex.md)
- [`HV-T03-copilot.md`](handoffs/HV-T03-copilot.md)
- [`HV-T04-gemini.md`](handoffs/HV-T04-gemini.md)

## 7. Dependency Rationale

- **HV-T01 & HV-T02 — independent.** Engine fix vs extractor extension touch disjoint surfaces. Run together.
- **HV-T03 — soft-blocked on Wave 1.** Cross-engine tests can be written against existing fixtures NOW, but if HV-T02 reshapes `_index.json`, HV-T03 needs a refresh. Cleanest: start HV-T03 once HV-T02 lands.
- **HV-T04 — soft-blocked on Wave 1.** The validation_records `field` column should know the canonical field names after HV-T02 enriches them. Migration itself is independent; the writer's column-mapping references the names.
- **All four merge cadence:** wave boundary — Wave-1 PRs (T01 + T02) merge together, then Wave-2 PRs (T03 + T04) merge together. Avoids `_index.json` rebases.

## 8. Verification Strategy

| Wave | Proof artifact | Runner |
|---|---|---|
| W1 close | HD validation report PR comment showing type=100% or documented divergence | `cargo test ... -- --ignored --nocapture` |
| W1 close | `_index.json` diff in PR: ≥3 new keys per entry | review |
| W2 close | Cross-engine reports show non-zero runs for panchanga + numerology | `cargo test ...` |
| W2 close | Migration applies + writer compiles under feature flag | `cargo check --features record-validation` |
| Phase exit | Single integration commit on `main` re-runs all three validation harnesses; results captured as a row in `humdes_validation_runs` (manual one-time invocation) | human lead |

## 9. GitHub Sync and Dispatch Strategy

- Issues exist: #853, #854, #855, #856 — all labelled `humdes-validation`.
- Optionally apply this milestone: create milestone `humdes-validation-p1` and assign all four.
- **Each PR title prefix:** `[HV-T0X #issue]` so it's auto-linkable in summary.
- **PR labels:** copy the issue's labels + add `status:in-review` on open, `status:done` on merge.
- **Wave summary comments:** the orchestrator (claude or human lead) posts:
  - on Wave-1 launch — link both PRs, restate contract
  - on Wave-1 close — paste validation diff
  - on Wave-2 launch — link Wave-1 evidence + Wave-2 PRs
  - on Wave-2 close — final report + close milestone

## 10. Worker Bootstrap Packet Strategy

Each agent gets a self-contained handoff packet (`.swarm/humdes-validation/handoffs/HV-T0X-<agent>.md`) covering:
1. Goal & acceptance (copied from issue body)
2. Allowed edit surface (file paths)
3. Forbidden surface (lock zones)
4. Required reads (specific files + line numbers)
5. Suggested first commands
6. Verification commands
7. Open-PR template

The packets are designed so a fresh CLI session of the named agent can begin work without re-reading this PLAN — they are self-sufficient bootstrap docs.

## 11. Risks and Fallback Plan

| Risk | Likelihood | Mitigation |
|---|---|---|
| Sub-tab capture (T02) hits new humdes anti-bot | medium | Fork to v3 keeping v2 working; if blocked, scope T02 to "definition" only via existing mechanics HTML |
| MG/G investigation (T01) finds engine bug that affects more than 7 charts | medium | Treat as Phase 2 — open a child issue, scope T01 to documentation-only, revisit fix in next milestone |
| `noesis-data` already has a migration scheme this PR doesn't match | medium | T04 owner inspects existing migrations first and adapts; if blocker, escalate to human lead |
| Cross-engine tests (T03) reveal one engine panics on certain charts | low | T03 owner files a child issue per crash; smoke test treats them as known failures with a fixture-exclusion list |
| Wave merge sequencing slips, causes `_index.json` rebase pain | low | Wave-close commits go to `main` first; Wave-2 branches rebase before merge |
