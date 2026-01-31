---
name: task-master-planner
description: Generate system-engineering task plans from repo specs and .context docs, including backend/frontend separation, phased sprints, dependencies, and hour estimates. Use when asked to create a task breakdown, sprint plan, or engineering workplan from DesignSpec.md, ProjectArchitecture.md, or .context.
---

# Task Master Planner

Follow this workflow to produce a task-master style JSON plan from repo specs.

## Inputs to load (minimum)
- `DesignSpec.md`
- `ProjectArchitecture.md`
- `.context/architecture/overview.md`
- `.context/architecture/patterns.md`
- `.context/auth/overview.md`
- `.context/testing.md`
- `.context/workflows.md`
- `.context/errors.md`
- `.context/api/headers.md`
- `.context/feature-flags.md`
- `.context/performance.md`
- `.context/monitoring.md`
- `.context/ui/patterns.md`

If the request mentions “process all files in .context”, enumerate files with `rg --files .context` and skim any additional files that define requirements or constraints.

## Output format
Produce JSON following the template in `assets/task-master-template.json`. Keep valid JSON and stable IDs.

Required properties per task:
- `id` (string, stable)
- `title` (short, action-oriented)
- `area` (frontend|backend|data|infra|qa|product)
- `owner_role` (e.g., Frontend Eng, Backend Eng, DevOps, QA)
- `est_hours` (number)
- `dependencies` (array of task ids)
- `deliverable` (1 sentence)
- `acceptance` (1 sentence, testable)

## Task planning rules
1. Separate backend and frontend responsibilities clearly via `area` and `owner_role`.
2. Use phases with sprints; each sprint has a clear focus and 10–16 tasks.
3. Produce 80–100 tasks total unless user specifies otherwise.
4. Include work for: auth/RBAC, database schema + RLS, edge functions, API routes, realtime, pages from DesignSpec, shared UI components, tests (unit/integration/e2e), monitoring/logging, performance, security, deployment.
5. Respect architecture principles (real-time, event sourcing, API-driven, RBAC).
6. Add dependencies where sequencing matters (e.g., schema before API, API before UI).
7. Ensure task hours are realistic (4–16 hours typical).
8. Include assumptions and risks in top-level metadata.

## Optional helpers
Use `scripts/collect_context.sh <repo>` to bundle a lightweight context manifest if needed.

Use `scripts/generate_task_plan.py <repo> --mode initial --output task_master_plan.json` to generate the initial plan.

Use `scripts/generate_task_plan.py <repo> --mode evolve --baseline task_master_plan.json --output task_master_plan.evolve.json` to append an evolution phase after the first delivery.
