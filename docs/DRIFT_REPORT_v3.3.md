# Noesis API Drift Report — v3.3.0
**Generated:** 2026-05-08  
**Auditor:** P1-W1 Swarm  
**Status:** Complete — all 4 audits synthesized

---

## Executive Summary

| Category | Gap Count | Severity |
|----------|-----------|----------|
| Undocumented API routes | 44 of 84 routes (52%) | 🔴 Critical |
| SDK missing methods | 18+ missing client methods | 🔴 Critical |
| llms.txt accuracy | 3 categories of drift | 🟡 Medium |
| OpenAPI spec public access | Spec returns 404 in production | 🔴 Critical |
| Naming inconsistency (Selemene→Noesis) | 49+ files | 🟡 Medium |

---

## Audit 1: API Routes vs Docs Coverage

**Total registered routes:** 84  
**Documented routes (in docs/api/):** ~40  
**Coverage:** ~48%

### Completely Undocumented Route Groups

#### Admin Analytics (0% covered — 4 routes)
- `GET /admin/analytics/summary`
- `GET /admin/analytics/top-consumers`
- `GET /admin/analytics/usage-breakdown`
- `GET /admin/analytics/usage-timeseries`

#### Admin API Keys (0% covered — 3 routes)
- `GET /admin/api-keys`
- `DELETE /admin/api-keys/:key_id/revoke`
- `POST /admin/api-keys/:key_id/rotate`

#### Admin Audit Events (0% covered — 3 routes)
- `GET /admin/audit-events`
- `GET /admin/audit-events/:event_id`
- `GET /admin/audit-events/actions`

#### Admin Billing (0% covered — 9 routes)
- `GET /admin/billing/overview`
- `GET /admin/billing/plans`
- `GET /admin/billing/reconcile/drift`
- `POST /admin/billing/reconcile/run`
- `GET /admin/billing/subscriptions`
- `GET /admin/billing/subscriptions/:id`
- `POST /admin/billing/subscriptions/:id/cancel`
- `GET /admin/billing/webhook-events`
- `GET /admin/usage/summary` ← **fixed 500 in v3.3.0; make_interval::integer cast**

#### Admin System (0% covered — 4 routes)
- `GET /admin/system/health`
- `GET /admin/system/cache`
- `GET /admin/system/services`
- `GET /admin/system/workflows`

#### Admin History Sync (0% covered — 3 routes)
- `GET /admin/history-sync/devices`
- `GET /admin/history-sync/events`
- `GET /admin/history-sync/users`

#### Admin User Management (partial — 3 undocumented)
- `PUT /admin/users/:user_id/roles`
- `PUT /admin/users/:user_id/state`
- `PUT /admin/users/:user_id/tier`

#### Admin Misc (0% covered — 2 routes)
- `GET /admin/ephemeris/checksums`
- `GET /admin/session`

#### Biofield (0% covered — 9 routes)
- `GET /biofield/readings` / `GET /biofield/readings/:reading_id` / `POST /biofield/readings/:reading_id/reprocess`
- `GET /biofield/baselines`
- `POST /biofield/exports`
- `GET /biofield/sessions` / `GET /biofield/sessions/:session_id`
- `POST /biofield/sessions/:session_id/captures` / `POST /biofield/sessions/:session_id/close`

#### Billing Consumer (partial — 2 undocumented)
- `GET /billing/checkout`
- `GET /billing/portal`
- (billing/balance and billing/events ARE documented)

#### Auth (partial — 3 undocumented)
- `POST /auth/change-password`
- `POST /auth/forgot-password`
- `GET /auth/discord/authorize`
- `GET /auth/discord/callback`

#### Onboarding (0% covered — 3 routes)
- `GET /onboarding/:code/openclaw.txt`
- `POST /onboarding/invite`
- `GET /onboarding/invites`

#### Other Undocumented
- `POST /witness/interpret` ← reading-object contract not documented
- `GET /users/me` ← user profile endpoint
- `GET /engines/face-reading/upload` ← file upload endpoint

---

## Audit 2: noesis-sdk-ts vs v3.3.0 API

**SDK version:** ~1.0 (no admin, no biofield, minimal billing)

### Methods Present in SDK
| Method | Endpoint | Status |
|--------|----------|--------|
| `health()` | GET /health/live | ✅ Current |
| `calculate(engineId, input)` | POST /engines/:id/calculate | ✅ Current |
| `workflow(workflowId, input)` | POST /workflows/:id/execute | ✅ Current |
| `billing.ts` | (types only, no client methods) | ⚠️ Types only |

### Critical Missing SDK Methods
| Missing Method | Endpoint | Priority |
|----------------|----------|----------|
| `listEngines()` | GET /engines | High |
| `engineInfo(id)` | GET /engines/:id/info | High |
| `validateEngine(id, input)` | POST /engines/:id/validate | Medium |
| `listWorkflows()` | GET /workflows | High |
| `workflowInfo(id)` | GET /workflows/:id/info | High |
| `getMe()` | GET /users/me | High |
| `getMyUsage()` | GET /users/me/usage | High |
| `getBillingBalance()` | GET /billing/balance | High |
| `getBillingSubscription()` | GET /billing/subscription | High |
| `subscribe(plan)` | POST /billing/checkout | Medium |
| `listReadings()` | GET /readings | High |
| `getReading(id)` | GET /readings/:id | High |
| `getReadingStats()` | GET /readings/stats | Medium |
| `interpretWitness(input)` | POST /witness/interpret | High |
| All admin methods | /admin/* | Low (admin-only) |
| All biofield methods | /biofield/* | Medium |

### SDK Type Gaps
- `WorkflowResult` missing: `reading_id`, `reading_url`, `created_at`, `subject`, `evidence` (reading-object contract v3.3.0)
- `EngineOutput` missing: `witness_layer` object with `title`, `summary`, `convergences`, `frictions`, `practice`, `question`
- No admin types at all (UsageSummary, BillingOverview, etc.)
- No biofield types

---

## Audit 3: llms.txt Accuracy

**Current state:** 49 lines, accurate for engine list and workflow list.

### Gaps Found
| Issue | Details | Fix |
|-------|---------|-----|
| **Product name** | File says "Selemene Engine" (2 occurrences) | → "Noesis" |
| **Missing surface** | No billing endpoints listed | Add billing section |
| **Missing surface** | No admin endpoints listed | Add admin section (at least overview/usage) |
| **OpenAPI URL** | Listed as `https://selemene.tryambakam.space/api/openapi.json` but returns 404 (Swagger gated) | Either enable or document how to access |
| **Version** | No version number anywhere | Add `version: 3.3.0` |
| **Missing endpoints** | readings, witness/interpret, users/me/usage | Add |

### What's Correct
- All 16 engine IDs and info URLs ✅
- All 6 workflow IDs and info URLs ✅  
- Swagger UI URL (`/api/docs`) ✅
- Base API URL ✅

---

## Audit 4: OpenAPI Spec Completeness

**Finding: `/api/openapi.json` returns 404 in production.**

**Root cause:** Swagger UI is gated behind `ENABLE_SWAGGER_UI=true` env var (lib.rs line 946-950). This env var is not set in Railway production.

This means:
- The `openapi.yaml` in `docs/api/` is the only spec reference — and it's not auto-generated from code, so it drifts
- The llms.txt URL for the spec is broken
- LLM agents that try to discover the API via spec get a 404

**Partial coverage in `docs/api/openapi.yaml`:** File exists but needs audit against registered routes.

### Routes Definitely Missing from openapi.yaml
All admin endpoints (analytics, billing, API keys, audit events, system, usage)  
All biofield endpoints  
All auth endpoints (change-password, forgot-password, Discord)  
All onboarding endpoints  
`/witness/interpret`  
`/users/me`  

---

## Naming Inconsistency Sweep

```
grep -r "Selemene Engine" docs/ --include="*.md" | wc -l → 23 occurrences
grep -r "Selemene Engine" llms.txt → 2 occurrences
grep -r "selemene.tryambakam.space" docs/ → correct (keep — this is the URL)
grep -r "SelemeneError" → SDK class name (intentional, keep for BC)
```

**Product name:** "Noesis" everywhere in user-facing docs.  
**Repo/code name:** "Selemene Engine" acceptable in internal/code context.  
**`SelemeneError`** in SDK: intentional class name — document as breaking change if renamed.

---

## Priority Order for P1-W2 and P1-W3

1. **Fix OpenAPI spec access** — enable `ENABLE_SWAGGER_UI=true` in Railway OR export spec to file; breaks all LLM agent discovery (PR-immediate)
2. **Update SDK types for reading-object** — contracts already shipped, SDK is wrong today
3. **Write admin-analytics.md + admin-billing docs** — most-used admin surface
4. **Update llms.txt** — LLM consumers see wrong info
5. **Add missing SDK methods** — listEngines, listWorkflows, listReadings, interpretWitness, getMe
6. **Document biofield endpoints** — used in Noesis Web Stage 2

---

## Files to Update in P1-W2 / P1-W3

| File | Action |
|------|--------|
| `docs/api/README.md` | Add billing, admin, biofield, witness sections |
| `docs/API_QUICKSTART.md` | Update with v3.3.0; add reading-object shape |
| `docs/api/billing.md` | CREATE — full billing endpoint reference |
| `docs/api/admin-analytics.md` | CREATE — usage/summary, analytics/* |
| `docs/api/admin-reconcile.md` | CREATE — billing/reconcile/* |
| `docs/api/admin-billing.md` | CREATE — billing/overview, subscriptions, plans |
| `docs/api/biofield.md` | CREATE — all biofield endpoints |
| `docs/api/witness.md` | CREATE — witness/interpret + reading-object contract |
| `llms.txt` | Rename product, add billing/admin/version, fix OpenAPI URL |
| `docs/api/openapi.yaml` | Major update — add all missing routes |
| `packages/noesis-sdk-ts/src/index.ts` | Add 14+ missing methods |
| `packages/noesis-sdk-ts/src/billing.ts` | Add client methods (not just types) |
