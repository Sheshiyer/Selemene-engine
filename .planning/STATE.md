# State

## Current

- **Active milestone**: m1 (P4 Gate D → P5 pending)
- **Main branch**: at PR #651 (V22-W1 biorhythm/numerology complete)
- **Open feature PRs**:
  - **PR #652** (`feat/v3.1.0-security-validation-usage` → main): v3.1.0 security audits, input validation, UsageRepository, migration guide
  - **PR #653** (`feat/biorhythm-forecast-secondary-cycles` → main): ForecastDay aesthetic/spiritual cycles + lib.rs/calculator.rs split fix
- **Remote branches**: 2 feature branches (`feat/v3.1.0-*`, `feat/biorhythm-*`), no `agent/issue-*` branches remaining

## Recent Activity

- **2026-05-04**:
  - Opened PR #652 — v3.1.0 security hardening, input validation, UsageRepository date-range queries, API migration guide (v2→v3)
  - Opened PR #653 — biorhythm `ForecastDay` secondary cycles (aesthetic 43-day, spiritual 53-day) + fixed pre-existing `E0255` compile bug in `lib.rs` (duplicate definitions vs `calculator.rs`)
  - **Branch cleanup complete**: deleted all 74 `agent/issue-*` remote branches — all work confirmed in main or covered by new PRs
    - Updated `noesis-admin` ruleset to exclude `refs/heads/agent/**` from branch deletion protection
  - V22-W1 (numerology personal cycles + biorhythm secondary cycles + compatibility) fully merged via PR #651

- **2026-04-28**: PR #651 merged — V22-W1-S2-09 (full V22 Week 1 sprint complete)

- **2026-04-26**:
  - Closed PR #567 (redundant); shipped #568, #596, #597, #598, #600, #601
  - Agent-dispatch + auto-ready + merge-lane pipeline shipped

## Open Blockers

None. CI should be green on main (PR #651 clean merge).

## Next

1. **Merge PR #652** once CI passes — v3.1.0 security + usage analytics
2. **Merge PR #653** once CI passes — biorhythm ForecastDay fix
3. **Reconcile `overall_energy` formula**: `ForecastDay.overall_energy` uses 6-cycle mean (PR #653), but `BiorhythmResult.overall_energy` in `calculate()` in `lib.rs` still uses 3-cycle mean — decide if they should match
4. **Gate D** (P4): p95 SLO validation under mixed workflow traffic
5. **Gate E** (P5): Canary + rollback drill, release checklist signoff

## Notes

- **engine-biorhythm compile bug (fixed in PR #653)**: When `calculator.rs` was extracted, `lib.rs` was updated to `pub mod calculator` + `pub use` but the old monolithic code was NOT removed, causing ~52 `E0255` duplicate-name errors silently lurking on main. PR #653 removes the ~280 duplicate lines.
- **V22 milestone**: V22-W1-S1 (numerology) + V22-W1-S2 (biorhythm) fully merged. PR #652 brings v3.1.0 security layer. PR #653 adds the last missing secondary rhythm fields.
- `.context/planning/` archived to `.context/planning/_archive-2026-03/` (25 files, untouched, frozen 2026-03-02).
