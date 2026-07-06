# HV-T01 — MG/G classification investigation

> Self-contained handoff packet. Read this only, then begin work.

**Status:** Resolved — documented as known divergence (option b). Selemene's
strict classifier is canonically correct; humdes' MG label for 7 fixtures is
internally inconsistent with humdes' own type-mechanics prose. See the new
"MG/G boundary decision" section in `tests/fixtures/humdes/README.md` for the
full trace and reasoning. No code change to `analysis.rs` was needed.

## Issue
[#853 — HD engine: investigate 7 MG-vs-Generator classification mismatches](https://github.com/Sheshiyer/Selemene-engine/issues/853)

## Identity
- **Agent:** Claude (orchestrator role)
- **Branch:** `swarm/humdes-validation/p1-w1/engine/853-claude`
- **Worktree:** `.worktrees/853-claude` (create with `git worktree add .worktrees/853-claude swarm/humdes-validation/p1-w1/engine/853-claude`)
- **Wave:** Phase 1, Wave 1, Swarm A

## Goal
Hand-trace one MG-vs-G mismatching fixture through `analysis.rs::determine_type` and decide one of:
- **(a) Fix:** Selemene logic misses a valid MG configuration → fix + add unit test
- **(b) Document:** humdes is permissive vs Selemene strict → document divergence in `tests/fixtures/humdes/README.md`

Either outcome is acceptable. Reason about which definition matches Ra Uru Hu's original system most faithfully.

## Allowed edit surface
| File | Reason |
|---|---|
| `crates/engine-human-design/src/analysis.rs` | Only if option (a): fix `determine_type` |
| `crates/engine-human-design/tests/analysis_tests.rs` | Only if option (a): add unit test for the new MG case |
| `tests/fixtures/humdes/README.md` | Always: document decision under a new "## MG/G boundary decision" section |
| `.swarm/humdes-validation/handoffs/HV-T01-claude.md` | This file: status field |

## Forbidden surface
- Any other crate
- `tests/fixtures/humdes/_index.json` or any fixture file (those are HV-T02's lock zone)
- `tests/reference_charts.json` (separate test corpus)

## Required reads (in order)
1. [`tests/fixtures/humdes/README.md`](../../../tests/fixtures/humdes/README.md) — context for the validation harness
2. [`crates/engine-human-design/src/analysis.rs`](../../../crates/engine-human-design/src/analysis.rs) — function `determine_type` and surrounding helpers
3. [`crates/engine-human-design/src/models.rs`](../../../crates/engine-human-design/src/models.rs) — `HDType`, `Channel`, `Center` definitions
4. The mechanics HTML body for one mismatch — open
   `~/Downloads/humdes-extractor/output/2026-05-16_124939_bulk2/readings/personal/c517b6613503abfe886f2850356ba135_*/05_*_tabs_mec.json`
   then read the `data.body` string
5. Issue [#853](https://github.com/Sheshiyer/Selemene-engine/issues/853) full body

## Suggested first commands
```bash
cd /Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/Selemene-engine
git worktree add .worktrees/853-claude swarm/humdes-validation/p1-w1/engine/853-claude
cd .worktrees/853-claude

# Reproduce baseline failure
cargo test --package engine-human-design --test humdes_validation_tests \
    -- --ignored --nocapture 2>&1 | tee /tmp/baseline.log

# Look at the source-of-truth for one mismatch
LATEST=~/Downloads/humdes-extractor/output/2026-05-16_124939_bulk2
DIR=$(find "$LATEST/readings/personal" -name 'c517b66135*' -type d)
cat "$DIR/_row.json"                                         # what humdes says
cat "$LATEST/readings/personal/$(basename $DIR)/05_"*"_tabs_mec.json" | jq -r '.data.body' > /tmp/mec.html

# Read Selemene's logic
grep -n "determine_type" crates/engine-human-design/src/analysis.rs
```

## Verification before opening PR
```bash
cargo fmt --package engine-human-design --check
cargo clippy --package engine-human-design -- -D warnings
cargo test --package engine-human-design                                       # smoke
cargo test --package engine-human-design --test humdes_validation_tests \
    -- --ignored --nocapture 2>&1 | tee /tmp/after.log

# If option (a): diff /tmp/baseline.log /tmp/after.log should show type=89/89
# If option (b): diff is identical, but README section is added/updated
```

## PR template
```
Title: [HV-T01 #853] HD MG/G boundary — <fix|documented as divergence>

Closes #853

## Summary
- Outcome: <fix landed | documented as known divergence>
- Fixtures affected: 7 (all 7 of the MG-vs-G mismatches from validation report)

## Decision
<one-paragraph explanation of which option taken and why>

## Evidence
- Baseline report: `<paste baseline section>`
- After report: `<paste after section>`

## Files touched
- `crates/engine-human-design/src/analysis.rs` (only if fix)
- `crates/engine-human-design/tests/analysis_tests.rs` (only if fix; new test)
- `tests/fixtures/humdes/README.md` (always; new section on MG/G decision)
```

## Handoff signal
On merge, post on the issue:
> HV-T01 merged on `<commit>`. Validation now <100% on type | documents
> a known divergence>. Next: HV-T02 (#854) can land independently.

## Escalation triggers
- If the fix would touch `crates/engine-human-design/src/activations.rs` or `src/chart.rs` (more than `analysis.rs`), STOP and reopen #853 with a Phase-2 child issue. This packet only covers `analysis.rs`.
- If the bug affects > 7 charts (you find new mismatches in the existing report), STOP and escalate to human lead.
