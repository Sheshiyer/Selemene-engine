# Selemene Engine — Noesis

## What This Is

A consciousness intelligence platform delivering 16 reflection engines across 6 integrated workflows, exposed via Rust API + TypeScript sidecar bridge. Engines span Vedic astrology (transits, vimshottari, panchanga, vedic-clock, nadabrahman), Western reflection (biofield, biorhythm, gene-keys, human-design, numerology, face-reading), and symbolic systems (enneagram, i-ching, sacred-geometry, sigil-forge, tarot).

## Core Value

Reflective consciousness work without prescriptive output. Every engine surface presents witness prompts (questions, not directions) that invite self-inquiry. The witness prompt quality contract (#432) enforces this across all 16 engines.

## Tech Stack

- **API**: Rust workspace (axum, tokio, sqlx) — 11 native engines + 5 TS-bridge engines
- **Sidecar**: TypeScript engines via Bun (enneagram, i-ching, sacred-geometry, sigil-forge, tarot)
- **Storage**: PostgreSQL (sqlx), Redis (caching, queues)
- **Auth**: Discord OAuth + API keys
- **Frontends**: admin-web, biofield-web (Next.js)
- **Charts**: Native Swiss Ephemeris (libswisseph-sys) — D1/D9 raw passthrough
- **Deploy**: Docker + Kubernetes; Railway sidecar
- **CI/CD**: agent-dispatch + agent-auto-ready + agent-merge-lane + agent-post-merge pipeline (homegrown Copilot dispatch via #568, #596, #597)

## Project Phase

Brownfield, milestone m1 in progress. P1-P3 sprints complete; P4 (Performance & Observability) and P5 (Release Readiness) active. See `ROADMAP.md`.

## Methodology Notes

This repo runs **two complementary documentation systems**:

1. `.context/` — **Substrate Methodology** (architecture/, auth/, api/, database/, decisions/, ui/, etc.) — code-as-context patterns, ADRs, ai-rules. Stays as the canonical engineering reference.
2. `.planning/` — **GSD/swarm-architect** — phase tracking, requirements, roadmap, verification. Initialized 2026-04-26 for compatibility with `task-master-planner` skill and the agent-ready dispatch loop.

`docs/plans/` — human-readable execution plans, exported from `.planning/` artifacts. Stale `.context/planning/` (frozen 2026-03-02, 25 files) archived to `.context/planning/_archive-2026-03/`.

## Requirements

### Validated (existing capabilities)

- ✓ 16 engines registered with workflow-parity contract (`cargo test -p noesis-api workflow_parity`)
- ✓ Witness prompt quality contract (#432) — all 16 engines question-formatted, non-prescriptive, ≥24 chars
- ✓ Agent-dispatch pipeline (`agent-ready` label triggers scaffold + Copilot kickoff)
- ✓ Discord OAuth + API key auth
- ✓ Native Vedic charts (D1/D9 via Swiss Ephemeris, replacing FreeAstrologyAPI)
- ✓ Admin dashboard (api-keys, audit, history-sync, system, users)
- ✓ Biofield web app (capture → readings → comparison/export)

### Active (gates D + E open)

- [ ] P4 — Performance & Observability soak coverage + tracing
- [ ] P5 — Release Readiness + canary automation
- [ ] reqwest 0.11 → 0.12 upgrade in `noesis-western-api` (drops 3 audit ignores)

### Out of Scope

- Prescriptive output ("you should X") — banned by witness contract enforced in CI
- Direct handler-to-engine coupling — orchestrator-only routing
- Re-implementing canonical Copilot bootstrap files — homegrown pipeline already in production

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Witness contract enforced in CI | Reflection-first philosophy; non-prescriptive language | Hard gate added 2026-04-26 (#598) |
| Homegrown agent-dispatch over canonical install | Already shipped, production-validated, full loop (dispatch → auto-ready → merge-lane → post-merge) | Coexists; canonical install deferred for sister repos |
| Substrate `.context/` + GSD `.planning/` | Substrate for code patterns/ADRs, GSD for phase tracking + agent compatibility | Both maintained, distinct scopes |
| Pin `dtolnay/rust-toolchain@stable` (not version-pinned) | Catches new lints early but introduces stable-clippy regressions | Tracked separately; `.cargo/audit.toml` carries advisory ignores |

---
*Last updated: 2026-04-26 — minimal init from existing roadmap and context*

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Project Phase with current state
