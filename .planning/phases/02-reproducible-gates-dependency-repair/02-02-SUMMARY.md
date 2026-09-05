---
phase: 02-reproducible-gates-dependency-repair
plan: "02"
status: complete_source_scope
completed: 2026-09-05
source_commit: f6d777e3c7050e14999b884279745047fe1dc41f
requirements_partial: ["GATE-01", "GATE-05"]
---
# Required CI and release-path repair

All 77 third-party Action references use verified 40-character SHAs. Actionlint passes. All 67 script tests pass, including 14 mocked executions of the real merge-lane workflow. CI Gate includes 13 required jobs, with additional Python version/image matrix legs. Admin smoke now fails on missing URL configuration; both tag and CD release paths use the same complete CI workflow.

## Verification limits

Action pin scope is uses refs only. Candidate remote CI and required-check ruleset are not yet proved/applied. This plan completes source wiring, not every original GATE-05 registry or deployment obligation.

These summaries close their bounded source tasks. Phase 2 and original Wave 0/1 exit criteria remain open until all requirements are independently verified. The remote CI and critical promotion decision are tracked by 02-04 / 02-05 and the recovery PR.
