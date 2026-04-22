# Parallel Worktree Scaffold Runbook

## Goal
Create isolated worktrees for Rust engines, workflows, and TypeScript engines so teams or agents can run in parallel with persistence and anti-drift controls.

## Inputs
- Stream inventory: `.context/worktrees/parallel-workstreams.yaml`
- Base branch: `main` by default
- Worktree root: `.worktrees` by default

## Step 1: Create worktrees

```bash
bash scripts/worktrees/create_parallel_worktrees.sh main .worktrees
```

This creates branch/worktree pairs:
- `parallel/rust-engine/<engine-id>`
- `parallel/workflow/<workflow-id>`
- `parallel/ts-engine/<engine-id>`

Each worktree contains:
- `.parallel/README.md`
- `.parallel/EXPERIMENTS.md`
- `.parallel/DECISIONS.md`
- `.parallel/REVIEW-CHECKLIST.md`

## Step 2: Run persistence loop per stream

For each stream:
1. Establish baseline behavior.
2. Run autoresearch loop with one change per iteration.
3. Log metric and keep/discard in `.parallel/EXPERIMENTS.md`.
4. Record decisions in `.parallel/DECISIONS.md`.

## Step 3: Apply documentation gate

Use the code-documentation flow after each accepted change:
- Update API or architecture notes.
- Add short explanation snippets for changed behavior.
- Keep docs synchronized with code.

## Step 4: Apply review gate

Use the code-review-ai flow before merge:
- Security checks
- Performance impact checks
- Maintainability checks
- Test coverage checks

Track completion in `.parallel/REVIEW-CHECKLIST.md`.

## Step 5: Prevent drift

Run periodic drift checks:

```bash
bash scripts/worktrees/check_parallel_drift.sh main .worktrees
```

Statuses:
- `in-sync`: no drift
- `needs-rebase`: behind base branch
- `ahead`: contains unmerged changes
- `diverged`: both ahead and behind, requires manual reconciliation

## Suggested cadence
- Drift check every 2-4 hours during active parallel development.
- Rebase `needs-rebase` streams before adding new experiments.
- Merge small accepted batches frequently to reduce long-lived divergence.
