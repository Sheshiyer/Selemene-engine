# Plan-to-Issue Automation

This repo now supports automatic sync from taskmaster JSON plans to GitHub issues.

## What is automated

- Reads taskmaster plans from `docs/planning/*.json`
- Creates or updates one issue per task (idempotent)
- Tracks issue identity using `plan-task-id: <task_id>` marker in issue body
- Applies planning labels (plan, phase, sprint, wave, area, owner role)
- Optionally adds issues to a GitHub Project board
- Optionally auto-closes/reopens issues from `task.status`

## Source of truth contract

- JSON plan is the source of truth.
- Issue title/body/labels are generated from plan fields.
- To auto-close an issue, set:

```json
{
  "id": "P2-S1-05",
  "status": "completed"
}
```

Accepted closing values: `completed`, `done`, `closed`.

## Scripts

- Canonical script: `scripts/sync-plans-to-github-issues.sh`
- Backward-compatible wrapper: `scripts/import-tasks-to-github.sh`

### Local examples

Dry-run for all plans:

```bash
scripts/sync-plans-to-github-issues.sh --repo Sheshiyer/Selemene-engine
```

Apply only Wave 0-1 phases:

```bash
scripts/sync-plans-to-github-issues.sh \
  --apply \
  --repo Sheshiyer/Selemene-engine \
  --plan-file docs/planning/admin-panel-taskmaster-plan-2026-02-25.json \
  --phase-filter P0,P1
```

Apply and add to project board:

```bash
scripts/sync-plans-to-github-issues.sh \
  --apply \
  --repo Sheshiyer/Selemene-engine \
  --project-owner Sheshiyer \
  --project-number 1
```

## GitHub Action

Workflow: `.github/workflows/plan-issue-sync.yml`

Triggers:
- Push to `main` when files under `docs/planning/*.json` change
- Manual `workflow_dispatch`

Optional repo variables:
- `GH_ROADMAP_PROJECT_OWNER`
- `GH_ROADMAP_PROJECT_NUMBER`

Optional secret:
- `PLAN_SYNC_TOKEN` (if you need broader scopes than `GITHUB_TOKEN`)

## Recommended execution workflow

1. Create or update a plan JSON in `docs/planning/`.
2. Merge plan to `main`.
3. Workflow auto-syncs issues.
4. Implement work via PRs that include `Fixes #<issue_number>`.
5. On merge, issue closes by GitHub linkage.
6. Keep plan `task.status` updated for deterministic reconciliation.

## Long-term roadmap planning pattern

Use one plan file per strategic track:

- `docs/planning/mobile-app-taskmaster-plan-YYYY-MM-DD.json`
- `docs/planning/solana-payments-taskmaster-plan-YYYY-MM-DD.json`
- `docs/planning/nft-platform-taskmaster-plan-YYYY-MM-DD.json`

This keeps issue generation modular and lets you sync/execute tracks independently by phase filter.
