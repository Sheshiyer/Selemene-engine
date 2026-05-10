# Roadmap — Milestone m1 (2026-02-24 → 2026-05-03)

## Phases

| # | Phase | Sprint | Dates | Status |
|---|---|---|---|---|
| P1 | Stabilization Baseline | S1 | 2026-02-24 → 2026-03-08 | ✓ Completed |
| P2 | Workflow Hardening | S2 | 2026-03-09 → 2026-03-22 | ✓ Completed |
| P3 | Bridge Reliability | S3 | 2026-03-23 → 2026-04-05 | ✓ Completed |
| P4 | Performance & Observability | S4 | 2026-04-06 → 2026-04-19 | 🔄 In Progress |
| P5 | Release Readiness & Scale | S5 | 2026-04-20 → 2026-05-03 | 🔄 In Progress |

## Phase Goals

### P1 — Stabilization Baseline ✓

**Goal:** Lock production baseline, enforce orchestrator guardrails, normalize error/caching paths.

**Requirements covered:**
- Orchestrator-only routing test-enforced
- Error mapping normalized
- Cache key consistency

**Verification:** see `phases/P1/VERIFICATION.md` (TBD — backfill from `.context/planning/_archive-2026-03/PHASE_3_COMPLETION_SUMMARY.md`)

### P2 — Workflow Hardening ✓

**Goal:** Add contract tests + validation hardening for all 6 workflows and selection logic.

**Requirements covered:**
- Workflow-parity test (all 6 workflows + 16 engines)
- Selection logic deterministic
- Witness prompt contract added (#432)

**Verification:** see `phases/P2/VERIFICATION.md` (TBD)

### P3 — Bridge Reliability ✓

**Goal:** Harden TS sidecar bridge contracts, retries, circuit-breakers, and latency visibility.

**Requirements covered:**
- TS sidecar contract tests
- Retry/circuit-breaker patterns
- Bridge latency observability

**Verification:** see `phases/P3/VERIFICATION.md` (TBD)

### P4 — Performance & Observability 🔄

**Goal:** Improve fan-out throughput, load posture, auth soak coverage, idempotency behavior.

**Requirements (`PERF-*`):**
- [ ] **PERF-01**: p95 latency SLO validated under mixed workflow traffic (Gate D)
- [ ] **PERF-02**: Auth soak at 60-minute window
- [ ] **PERF-03**: Workflow-level + request-level tracing spans
- [ ] **PERF-04**: Idempotency behavior covered

**In-flight scaffolds:** PRs #569-#578 (V22-W1 + P4-W1 fixtures), #581-#590 (P4 benches + tracing)

### P5 — Release Readiness & Scale 🔄

**Goal:** Close rollout gates with canary + smoke automation, runbooks, release signoff.

**Requirements (`REL-*`):**
- [ ] **REL-01**: Canary + rollback drill passed (Gate E)
- [ ] **REL-02**: Smoke automation per workflow
- [ ] **REL-03**: Ephemeris cache + rate-limit runbooks
- [ ] **REL-04**: Final cargo audit + TruffleHog secret scan signoff

**In-flight scaffolds:** PRs #582-#588 (P5-W1 + W3 release gates, ephemeris cache)

## Milestone Gates

- **Gate A (S1):** Orchestrator-only routing checks + error mapping test-enforced ✓
- **Gate B (S2):** Workflow contracts covered with deterministic fixtures ✓
- **Gate C (S3):** Sidecar failure handling + schema parity production-safe ✓
- **Gate D (S4):** p95 SLO and load profile validated under mixed workflow traffic — open
- **Gate E (S5):** Canary + rollback drill passed; release checklist signed — open

## Hygiene Track (cross-cutting)

- [ ] **HYG-01**: reqwest 0.11 → 0.12 upgrade in `noesis-western-api` (drops 3 audit ignores)
- [ ] **HYG-02**: Backfill `phases/P{1,2,3}/VERIFICATION.md` from completion summaries in `.context/planning/_archive-2026-03/`
- [ ] **HYG-03**: Branch cleanup — 35 no-PR remote branches (wave*, bf*-clean, release/v3.0.0-*, fix/*, claude/musing-*)

## Forward-Looking (post-m1)

### Nādashakti V2 — Audio Richness (consumer-side, noesis-web)

**Status:** plan ready, awaiting milestone slot. Owner: `claude` orchestrator + 4 sub-agent swarms.
**Plan:** [`raagaegnin/V2_AUDIO_RICHNESS_PLAN.md`](../../raagaegnin/V2_AUDIO_RICHNESS_PLAN.md)
**Scope:** gamakas (5 ornaments) · sample-pack timbres (sitar/tanpura/mridangam/bansuri/sarangi) · 6 talas + breath-paced cps · server-side WAV render · tanpura drone bed.
**Foundation shipped:** v1 just-intonation 22-shruti playback verified end-to-end (Strudel `freq()` + `evaluate()` path). Engine output (raga#) → audio synthesis pipeline live in `apps/noesis-web/src/lib/raaga/`.
**Effort:** 78 tasks · 3 phases · 9 waves · 22 swarms · ~108 hrs.
**Acceptance:** all 72 melakartas play within ±5¢ of just-intonation Hz under 5 ornament types; offline-rendered WAV matches live audio (RMSE < 0.01); v1 callsites unaffected.

## References

- Source: `.context/planning/_archive-2026-03/noesis-roadmap-timeline-2026-02-24.md`
- Per-phase JSON plans: `.context/planning/_archive-2026-03/p{1..5}-*.json`
- Recent execution plans: `docs/plans/`
- Consumer-app v2 plan: `raagaegnin/V2_AUDIO_RICHNESS_PLAN.md`
