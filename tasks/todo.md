# Task Plan — Merge-Ready Docs + Bridge Test Alignment

## Checklist
- [x] Create clean branch from `main` with `codex/` prefix
- [x] Apply only correct docs changes from PR #2/#3 and drop incorrect workflow claims
- [x] Fix stale `noesis-bridge` phase test expectations to match runtime constructors
- [x] Run full repository test suite
- [x] Push branch to remote
- [x] Open/update PR targeting `main`
- [x] Add verification summary and final review notes

## Scope Rules
- Keep docs aligned to API runtime source-of-truth in `crates/noesis-orchestrator/src/lib.rs` and route table in `crates/noesis-api/src/lib.rs`.
- Do not include PR #2 incorrect claims (`birth-blueprint` using vimshottari, `full-spectrum` as 14 engines).
- Preserve valid additions (status endpoint docs, phase tables, XP/promotion docs) with corrected wording where needed.

## Review (to fill after execution)
- Branch: `codex/merge-ready-docs-phase-tests`
- Commits:
  - `663c6e38` docs: align API docs with runtime workflows and phases
- Tests run:
  - `cargo test -p noesis-api test_birth_blueprint_workflow_includes_gene_keys -- --nocapture` (pass)
  - `cargo test -p noesis-bridge bridge_engine_factory_ -- --nocapture` (pass)
  - `cargo test -p noesis-bridge` (pass)
  - `cargo test --workspace` (fails in pre-existing `engine-human-design` reference validation tests)
- Test result:
  - Package-level tests for touched area pass.
  - Full workspace currently not green due unrelated failing tests in `crates/engine-human-design/tests/reference_validation_tests.rs`.
- PR URL: https://github.com/Sheshiyer/Selemene-engine/pull/4
- Remaining risks:
  - `cargo test --workspace` remains red because `engine-human-design` reference validation tests fail independently of this PR.
