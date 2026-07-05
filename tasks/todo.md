# Noesis Platform — Active Work (2026-07)

**Current Status (2026-07)**

## Active Surfaces
- **admin-web** — Next.js admin UI (Vercel, 144.tryambakam.space)
- **API** — Cloudflare Workers (selemene.tryambakam.space) + Railway Postgres
- **witness-pipeline** — Report generation + post-report patterns (Railway)

## Retired (2026)
- noesis-web and biofield-web — moved to Sankalpa
- witness agents (48.tryambakam.space) — largely retired
- Supabase — retired
- All P1-P4 drift remediation / Noesis Web Stage 2 / Hermes / Doc CI work from 2026-05 plan — superseded by platform shift

## Recent Completed (2026)
- Universal report rubric + post-report patterns
- Intake schema (gender + NormalizedLocation)
- Vectorize plan + implementation (store + retriever + privacy gate + orchestrator attachment + renderRetrievedPatternsForPrompt + retrieved_patterns in output)
- Vectorize retrieval + attachment complete in library; Worker surface contract (REPORT_PATTERNS + AI bindings + optional R2/D1) documented and implemented in @noesis/witness-pipeline (deployment surface is the consuming Cloudflare Worker)
- Location normalization
- Fidelity grounding (chart_fidelity_score) + early private birth data scrub before patterns

---

## Remaining Actionable Work

### Documentation Currency
- [x] Verify docs/ENGINES.md covers current engines (17: 11 Rust + 6 TS) and active surfaces only (verified; title "The 17 Engines")
- [x] Update llms.txt for current branding and active consumers (admin-web, witness-pipeline, OpenClaw, Raycast) — synced to 17 engines + current surfaces
- [x] Audit docs/api/* for references to retired surfaces (noesis-web, biofield-web, Supabase, witness agents) — retired surfaces noted as archived/reference-only in relevant docs; no active references in current paths

### API Surface
- [x] Confirm OpenAPI reflects current endpoints (openapi.yaml present; engines/workflows cover active 17+; Vectorize is library-side with Worker bindings documented, not a separate retired billing surface)
- [x] Audit noesis-sdk-ts against live API (only active endpoints) — SDK lists 17 ENGINE_IDS + active WORKFLOW_IDS; no retired surfaces exposed

### Integrations (Active Only)
- [x] Update docs/api/OPENCLAW_INTEGRATION.md for current platform (17 engines, active surfaces only)
- [x] Update docs/api/MCP_INTEGRATION.md for current platform (17 engines noted)
- [x] Raycast: N/A in this repo (no Raycast extension sources present; README.md mentions the extension as a consumer only)

### Platform Notes (Keep Lean)
- No new consumer docs for retired surfaces (done: retired surfaces documented as archived/reference-only)
- No tasks for noesis-web deployment, Hermes bridge, or Doc CI gates from 2026-05 plan (kept out of scope)

---

**Archive:** Full 2026-05 drift plan moved to `tasks/todo-legacy-2026-05.md` (203 lines). This file is now slim and reflects reality only. No active GH issues/PRs for retired P1-P4 items.

**Next reader note:** Everything below the "Current Status (2026-07)" header is the only actionable list. Treat all prior phases as historical.
