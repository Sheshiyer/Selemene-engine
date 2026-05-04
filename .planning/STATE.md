# State

## Current

- **Active milestone**: v3.1.0 — **all Gates A–E verified, release signoff complete**
- **Codebase version**: 3.1.0 (HEAD: ad810647)
- **Open feature PRs**: none — 0 open PRs
- **Remote branches**: `gate-e-report` and `fix/clippy-suppressions` (both merged, blocked from deletion by branch-protection rules — harmless)

## Recent Activity

- **2026-05-04** (session 2):
  - **PR #658** merged — Gate E verification report (`docs/release/gate-e-verification.md`)
    - Drill 1: healthy canary baseline → `canary_healthy: true` ✅
    - Drill 2: 5% error rate → `canary-promote.sh` rolls back at stage 25, hook called ✅
    - Drill 3: live smoke runner → `/health/live`, `/health/ready`, `/metrics` all pass ✅
  - **PR #659** merged — removed obsolete clippy suppressions from CI
    - CI now runs `cargo clippy --all-targets -- -D warnings` with zero `-A` overrides
    - All checks green at the stricter lint level

- **2026-05-04** (session 1):
  - **PR #652** merged — v3.1.0 security audits, input validation hardening, UsageRepository, API migration guide
  - **PR #654** merged — biorhythm `ForecastDay` aesthetic/spiritual cycles + `lib.rs`/`calculator.rs` split fix (rebased from closed #653)
  - **PR #656** merged — added `aesthetic: CycleResult` to `BiorhythmResult`; `overall_energy` now 6-cycle mean (was 3)
  - **PR #657** merged — bumped `test_health_check_no_auth_required` version assertion `"3.0.0"` → `"3.1.0"`
  - Gate D re-verified: 52/52 tests pass against v3.1.0 main

- **2026-04-28**: PR #651 merged — V22-W1-S2-09 (full V22 Week 1 sprint complete)

## Open Blockers

None.

## Next

Lower-priority follow-on items (no blocker on v3.1.0):

1. **Smoke test credentials** — provision `SMOKE_TEST_API_KEY` in Railway env to cover auth-gated checks in Drill 3 (engines list, panchanga calc, workflow exec, TS bridge)
2. **Version assertion fragility** — `test_health_check_no_auth_required` line 213 hardcodes `"3.1.0"`; will break on next version bump — consider making it semver-agnostic
3. **Rollback drill Scenarios 1 & 2** — broken env var + crashing init require a staging deploy; no staging environment currently exists on Railway
4. **Grafana annotation wiring** — `GRAFANA_ANNOTATION_CMD` hook exists but `GRAFANA_URL`/`GRAFANA_API_TOKEN` not configured; blocked on observability infrastructure

## Notes

- **Gate verification reports**: `docs/release/gate-verification.md` (Gates A–D, generated 2026-04-24) and `docs/release/gate-e-verification.md` (Gate E, generated 2026-05-04)
- **CI lint**: now strictest-ever — no `-A` suppressions; `cargo clippy --all-targets -- -D warnings` passes clean
- **Branch protection quirk**: `noesis-admin` ruleset blocks force-push on `feat/*`/`fix/*` and blocks branch deletion. Merged branches persist as orphans. Workaround for force-push: push to new branch name, close old PR, open new PR. `refs/heads/agent/**` is exempt from deletion protection.
- **Squash-merge divergence**: squash commits don't match individual commit SHAs. Never cut a fix branch from a feature branch post-merge; always cherry-pick onto `origin/main`.
- **types.rs dead file**: `crates/engine-biorhythm/src/types.rs` exists but is not referenced — leftover artifact, low priority.
- `.context/planning/` archived to `.context/planning/_archive-2026-03/` (25 files, frozen 2026-03-02).
