# Lessons

## 2026-02-24 — CI Gate Discipline
- Before declaring a branch merge-ready, run the same lint gate locally as CI:
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets -- -D warnings`
- If CI lint fails, pull exact job logs first and fix only what blocks the gate.
- Keep a dedicated follow-up commit for CI-only fixes so reviewers can isolate behavior vs formatting/tooling changes.
