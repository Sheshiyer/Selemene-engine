# Spec Kit Roadmap Alignment

Date: 2026-04-18

This note maps the current Selemene GitHub milestones and issue hygiene into a local planning
artifact so the codebase, roadmap, and release work do not drift.

Reference process: `github/spec-kit`

- Constitution first
- Spec before implementation
- Plan before task generation
- Tasks mapped to executable work
- Issue sync only after local planning artifacts exist

## Current GitHub milestone inventory

Repository: `Sheshiyer/Selemene-engine`

| Milestone | Open issues | Closed issues | Intent |
|---|---:|---:|---|
| `P1-Stabilization` | 41 | 29 | reliability, break-fix, hardening |
| `P2-Workflow-Hardening` | 64 | 6 | workflow correctness and durability |
| `P3-Bridge-Reliability` | 52 | 0 | TS bridge and sidecar stability |
| `P4-Performance-Observability` | 70 | 0 | telemetry, perf, QA visibility |
| `P5-Release-Readiness` | 45 | 25 | release prep and operational readiness |
| `v2.2.0-Specialized-Engines` | 35 | 26 | engine feature delivery |
| `v3.0.0-Platform-Launch` | 19 | 39 | launch execution |

## Current drift signal

GitHub snapshot on 2026-04-18:

- `200` open issues scanned
- `186` already have a milestone
- `14` open roadmap issues have no milestone

Representative missing-milestone roadmap issues:

- `#550` Biofield Phase 1: Web-first vertical slice
- `#551` BF1-01 — Merge and stabilize biofield scaffolding, contracts, and persistence seam
- `#552` BF1-02 — Align Python sidecar contract with Rust expectations
- `#553` BF1-03 — Implement authenticated biofield session lifecycle in Rust API
- `#554` BF1-04 — Wire biofield-web auth and session bootstrap
- `#555` BF1-05 — Implement capture upload path and sidecar proxy
- `#556` BF1-06 — Persist readings and artifacts and expose history/detail APIs
- `#557` BF1-07 — Render real history and reading detail in biofield-web
- `#558` BF1-08 — Add vertical-slice integration tests, smoke checks, and runbook proof
- `#544` through `#548` P4 ADR tasks are also open roadmap work without milestone assignment

## Operating rules to stop roadmap drift

These are the local rules to follow even before a full Spec Kit bootstrap lands in the repo.

1. Every new roadmap body of work gets a local planning artifact before code starts.
2. Every GitHub roadmap issue must have exactly one milestone or an explicit documented reason for none.
3. Every implementation PR must link the local planning artifact and the GitHub issue.
4. Every release-sensitive auth, routing, or deployment change must add or update a runbook note and regression tests.
5. `roadmap` issues without milestones are treated as process debt and should be triaged before new feature expansion.

## Suggested local mapping shape

Use a Spec Kit-style artifact chain for work that can break production paths:

```text
specs/<feature>/spec.md
specs/<feature>/plan.md
specs/<feature>/tasks.md
```

Minimum mapping fields to include in each local artifact:

- GitHub issue number
- GitHub milestone
- owner
- affected services
- required tests
- required runbooks or rollout notes

## Immediate alignment actions

1. Assign the Biofield Phase 1 issue set (`#550` to `#558`) to a single milestone instead of leaving them unscoped.
2. Assign the open P4 ADR issues (`#544` to `#548`) to `P4-Performance-Observability` unless product intent changed.
3. Treat auth-flow changes like Discord OAuth as `P1-Stabilization` or `P2-Workflow-Hardening` work, never as undocumented hotfixes.
4. When a bug is fixed in production, land the matching tests and runbook note in the same source change before considering the work done.

## How this connects to the current hardening work

This change set adds the first concrete example of the rule above:

- backend regression tests for callback path validation
- frontend regression tests for stable-host versus preview-host override behavior
- a runbook note for the canonical Discord callback policy

That is the minimum bar for future auth and redirect fixes.