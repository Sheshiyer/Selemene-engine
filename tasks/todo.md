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
- Vectorize plan + implementation
- Location normalization

---

## Remaining Actionable Work

### Documentation Currency
- [ ] Verify docs/ENGINES.md covers current 16 engines and active surfaces only
- [ ] Update llms.txt for current branding and active consumers (admin-web, witness-pipeline, OpenClaw, Raycast)
- [ ] Audit docs/api/* for references to retired surfaces (noesis-web, biofield-web, Supabase, witness agents)

### API Surface
- [ ] Confirm OpenAPI /api/openapi.json reflects current Workers + D1 + Vectorize endpoints (no retired billing/admin if removed)
- [ ] Audit noesis-sdk-ts against live API (only active endpoints)

### Integrations (Active Only)
- [ ] Update docs/api/OPENCLAW_INTEGRATION.md for current platform
- [ ] Update docs/api/MCP_INTEGRATION.md for current platform
- [ ] Update Raycast extension docs for current API surface

### Platform Notes (Keep Lean)
- No new consumer docs for retired surfaces
- No tasks for noesis-web deployment, Hermes bridge, or Doc CI gates from 2026-05 plan

---

**Archive:** Full 2026-05 drift plan moved to `tasks/todo-legacy-2026-05.md` (203 lines). This file is now slim and reflects reality only. No active GH issues/PRs for retired P1-P4 items.

**Next reader note:** Everything below the "Current Status (2026-07)" header is the only actionable list. Treat all prior phases as historical.
