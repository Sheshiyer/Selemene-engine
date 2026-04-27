# Noesis Engineering Timeline (as of 2026-02-24)

## Assumptions
- 16 engines and 6 workflows are already live in production.
- All calculation entrypoints remain orchestrator-first (no direct handler-to-engine coupling).
- Sprint cadence is 2 weeks.

## Phase Timeline

| Phase | Dates | Sprint | Objective |
|---|---|---|---|
| P1 — Stabilization Baseline | 2026-02-24 → 2026-03-08 | S1 | Lock production baseline, enforce orchestrator guardrails, normalize error/caching paths. |
| P2 — Workflow Hardening | 2026-03-09 → 2026-03-22 | S2 | Add contract tests + validation hardening for all 6 workflows and selection logic. |
| P3 — Bridge Reliability | 2026-03-23 → 2026-04-05 | S3 | Harden TS sidecar bridge contracts, retries, circuit-breakers, and latency visibility. |
| P4 — Performance & Observability | 2026-04-06 → 2026-04-19 | S4 | Improve fan-out throughput, load posture, auth soak coverage, and idempotency behavior. |
| P5 — Release Readiness & Scale | 2026-04-20 → 2026-05-03 | S5 | Close rollout gates with canary + smoke automation, runbooks, and release signoff. |

## Milestone Gates
- **Gate A (end of S1):** orchestrator-only routing checks and error mapping are test-enforced.
- **Gate B (end of S2):** workflow contracts are covered with deterministic fixtures.
- **Gate C (end of S3):** sidecar failure handling + schema parity are production-safe.
- **Gate D (end of S4):** p95 SLO and load profile validated under mixed workflow traffic.
- **Gate E (end of S5):** canary + rollback drill passed; release checklist signed.

## Risks to Track
- Bridge schema drift between Rust orchestrator envelope and TS sidecar responses.
- Cache key inconsistency causing false misses and stale response reuse.
- Observability gaps that hide fan-out bottlenecks during mixed workflow loads.
