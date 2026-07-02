# Requirements — Milestone m1

Brownfield: P1–P3 capabilities are validated; P4 + P5 + Hygiene tracks are open.

## v1 Requirements (Active)

### Performance (P4)

- [ ] **PERF-01**: p95 latency SLO validated under mixed workflow traffic (Gate D)
- [ ] **PERF-02**: Auth soak coverage at 60-minute window
- [ ] **PERF-03**: Workflow-level + request-level tracing spans
- [ ] **PERF-04**: Idempotency behavior covered for write-path workflows

### Release (P5)

- [ ] **REL-01**: Canary + rollback drill passed (Gate E)
- [ ] **REL-02**: Smoke automation for all 6 workflows
- [ ] **REL-03**: Runbooks for ephemeris cache + rate limits
- [ ] **REL-04**: Final cargo audit + TruffleHog secret scan signoff

### Hygiene (cross-cutting)

- [ ] **HYG-01**: reqwest 0.11 → 0.12 upgrade in `noesis-western-api` (drops 3 audit ignores)
- [ ] **HYG-02**: Backfill `phases/P{1,2,3}/VERIFICATION.md`
- [ ] **HYG-03**: Branch cleanup — 35 no-PR remote branches

## v2 Requirements (Deferred)

- Canonical Copilot bootstrap install (canonical contract from swarm-architect-skill — deferred for sister repos; Selemene's homegrown pipeline already implements equivalent contract)
- Replace unmaintained `backoff` crate with `backon` (RUSTSEC-2025-0012 warning)
- Pin `rust-toolchain` to a specific minor (avoid future stable-clippy regressions)

## Out of Scope

- Prescriptive output ("you should X") — banned by witness contract; enforced in CI
- Direct handler-to-engine coupling — orchestrator-only routing is a hard architectural rule
- Re-implementing canonical Copilot bootstrap files in this repo — already covered by homegrown pipeline (#568, #596, #597)

## Validated Capabilities (P1–P3 inheritance)

- ✓ Orchestrator-only routing checks + error mapping (Gate A)
- ✓ Workflow-parity contract tests for all 6 workflows + 16 engines (Gate B)
- ✓ TS sidecar bridge reliability + schema parity (Gate C)
- ✓ Witness prompt quality contract — all 16 engines (#432)
- ✓ Agent-dispatch + auto-ready + merge-lane + post-merge pipeline (#568, #596, #597)
- ✓ Cloudflare Zero Trust + API key auth
- ✓ Native Vedic charts (D1/D9) via Swiss Ephemeris

## Traceability

(Filled in by `gsd-roadmapper` during phase planning. Currently mapped via `ROADMAP.md` phase requirement lists.)
