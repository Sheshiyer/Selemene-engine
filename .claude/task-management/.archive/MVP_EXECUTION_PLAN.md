# MVP Deployment Execution Plan - Wave-Based Parallel Dispatch

**Generated:** 2026-02-08 21:58 IST
**Current Status:** Railway DOWN (502), Phase 1 Sprint 1 incomplete
**Strategy:** Parallel agent dispatch for independent MVP tasks

---

## Current State Assessment

### ✅ Already Complete
- **P1-S1-01** Supabase project exists (credentials in .env)
- **P1-S1-02** Database schema created (migrations/004_api_keys.sql)
- **P1-S1-04** sqlx migrations embedded (needs verification)
- **P1-S1-05** AuthService refactored (postgres feature-gated code exists)
- **P1-S1-09** railway.toml configuration created
- **Phase 10 FAPI** All 18 FreeAstrologyAPI tasks complete (570 tests passing)

### 🔴 Blocking Issues (Root Cause of 502)
1. `postgres` feature not enabled in noesis-api/Cargo.toml dependencies
2. DATABASE_URL missing or incorrect in Railway environment
3. Application failing health check on startup

### ⏳ Pending MVP Tasks

**Phase 1 Sprint 1 (Foundation):**
- P1-S1-03: Add sqlx dependencies (likely missing feature enablement)
- P1-S1-06: Add DATABASE_URL to ApiConfig validation
- P1-S1-07: Create API key seeding script
- P1-S1-08: Write unit tests for Postgres-backed AuthService
- P1-S1-10: Optimize Dockerfile.prod for Railway cache
- P1-S1-11: Configure Railway environment variables
- P1-S1-12: Provision Railway Redis add-on
- P1-S1-13: Deploy to Railway and verify health endpoints
- P1-S1-14: Test authenticated API request
- P1-S1-15: Test workflow execution
- P1-S1-16: Document Phase 1 deployment

**Phase 2 Sprint 1 (Observability - 14 tasks):**
- DNS, Sentry, Posthog, BetterStack

**Phase 2 Sprint 2 (User Onboarding - 12 tasks):**
- Admin endpoints, user docs, onboarding

---

## Wave 1: Foundation Fixes (CRITICAL - Parallel)

**Duration:** 4-6 hours wall-clock
**Blocker:** Railway is DOWN, must fix before proceeding

### Track A: Feature Enablement (Engineer)
**Tasks:** P1-S1-03, P1-S1-06
**Goal:** Enable postgres feature, add DATABASE_URL validation
**Agent Type:** Engineer
**Deliverable:**
- `noesis-api/Cargo.toml` has `noesis-auth = { path = "../noesis-auth", features = ["postgres"] }`
- `ApiConfig::from_env()` validates DATABASE_URL presence when RUST_ENV=production
- `cargo build --release` succeeds
**Success:** Application compiles with Postgres support

### Track B: Dockerfile Optimization (Engineer)
**Tasks:** P1-S1-10
**Goal:** Speed up Railway builds with layer caching
**Agent Type:** Engineer
**Deliverable:**
- `Dockerfile.prod` restructured: copy Cargo files → dummy build → copy source → real build
- Railway build time <3 minutes (down from 10-15min)
**Success:** Incremental deploys are fast

### Track C: Railway Environment (Engineer)
**Tasks:** P1-S1-11, P1-S1-12
**Goal:** Set all required environment variables, provision Redis
**Agent Type:** Engineer (with Railway CLI access)
**Deliverable:**
- All env vars from MVP task JSON configured in Railway:
  - DATABASE_URL (Supabase with sslmode=require)
  - JWT_SECRET (64-char random)
  - RUST_ENV=production
  - LOG_FORMAT=json
  - RUST_LOG=info
  - FREE_ASTROLOGY_API_KEY (from .env)
  - VEDIC_ENGINE_PROVIDER=api
  - VEDIC_ENGINE_FALLBACK_ENABLED=true
- Redis add-on provisioned
**Success:** `railway env` shows all variables, Redis URL injected

### Wave 1 Gate
**Verify:**
- [ ] `cargo build --release` succeeds locally
- [ ] Railway environment has all required variables
- [ ] Redis add-on provisioned and REDIS_URL available

---

## Wave 2: Deployment & Testing (Sequential)

**Duration:** 4-6 hours wall-clock
**Blocker:** Requires Wave 1 completion

### Track A: Deploy to Railway (Engineer)
**Tasks:** P1-S1-13
**Goal:** Get Railway deployment to 200 OK
**Agent Type:** Engineer
**Deliverable:**
- Railway deployment successful
- `/health` returns 200 with engines_loaded=9
- `/health/ready` shows redis=ok, orchestrator=ready
- `/metrics` returns Prometheus format
- `/api/docs` loads SwaggerUI
**Success:** All health endpoints return 200

### Track B: API Key Seeding (Engineer)
**Tasks:** P1-S1-07
**Goal:** Generate initial test API keys
**Agent Type:** Engineer
**Deliverable:**
- `examples/seed_api_keys.rs` CLI tool
- Generates 32-char random keys with nsk_ prefix
- Stores SHA-256 hash in Supabase
- Outputs plaintext keys once
**Success:** 5 test keys seeded, plaintext keys available

### Track C: Integration Testing (QATester)
**Tasks:** P1-S1-08, P1-S1-14, P1-S1-15
**Goal:** Verify Postgres auth, API requests, workflows
**Agent Type:** QATester
**Deliverable:**
- Unit tests for AuthService::validate_from_postgres
- Authenticated engine calculation test (POST /api/v1/engines/numerology/calculate)
- Workflow execution test (POST /api/v1/workflows/self-inquiry/execute)
- Rate limiting test (101st request returns 429)
**Success:** All tests pass, API key auth works end-to-end

### Wave 2 Gate
**Verify:**
- [ ] Railway deployment healthy (200 OK)
- [ ] 5 test API keys exist in Supabase
- [ ] Authenticated requests work
- [ ] Rate limiting enforced
- [ ] Workflows execute successfully

---

## Wave 3: Documentation (Single Agent)

**Duration:** 4 hours
**Blocker:** Requires Wave 2 completion

### Track A: Phase 1 Runbook (Engineer)
**Tasks:** P1-S1-16
**Goal:** Document deployment process
**Agent Type:** Engineer
**Deliverable:**
- `docs/DEPLOYMENT_RUNBOOK_PHASE1.md`
- Covers: Railway setup, Supabase config, env vars, health checks, troubleshooting
**Success:** Another engineer can replicate deployment from scratch

---

## Wave 4: Phase 2 Observability (Parallel - 3 tracks)

**Duration:** 8-10 hours wall-clock
**Blocker:** Requires Phase 1 complete

### Track A: DNS + Domain (Engineer)
**Tasks:** P2-S1-01, P2-S1-02, P2-S1-03
**Goal:** Configure tryambakam.space with Cloudflare
**Deliverable:**
- Domain registered, CNAME → Railway
- Cloudflare caching rules (/health 30s, /api/docs 1h, /api/* bypass)
- ALLOWED_ORIGINS updated
**Success:** https://tryambakam.space accessible with SSL

### Track B: Sentry Integration (Engineer)
**Tasks:** P2-S1-04 through P2-S1-09
**Goal:** Error tracking operational
**Deliverable:**
- Sentry project created
- sentry-rust + sentry-tower integrated
- Error context (engine_id, tier, consciousness_level)
- SENTRY_DSN in Railway env
**Success:** Sentry captures errors from production

### Track C: Posthog + BetterStack (Engineer)
**Tasks:** P2-S1-10 through P2-S1-14
**Goal:** Analytics and uptime monitoring
**Deliverable:**
- Posthog middleware (async events)
- POSTHOG_API_KEY in Railway
- BetterStack monitors (/health 3min, /health/ready 5min)
- Public status page
**Success:** Posthog shows events, BetterStack shows uptime

### Wave 4 Gate
**Verify:**
- [ ] Production domain SSL works
- [ ] Sentry captures test error
- [ ] Posthog shows usage events
- [ ] BetterStack monitors active

---

## Wave 5: Admin Endpoints (Parallel - 2 tracks)

**Duration:** 12-14 hours wall-clock
**Blocker:** Requires Phase 2 Sprint 1 complete

### Track A: Admin API (Engineer)
**Tasks:** P2-S2-01 through P2-S2-05
**Goal:** API key management endpoints
**Deliverable:**
- POST /api/v1/admin/keys (generate)
- POST /api/v1/admin/keys/:id/revoke
- GET /api/v1/admin/keys (list)
- Integration tests
**Success:** Admin can manage keys via API

### Track B: User Onboarding (Product + QA)
**Tasks:** P2-S2-06 through P2-S2-12
**Goal:** Onboard first test users
**Deliverable:**
- Test user documentation
- Usage monitoring dashboard
- End-to-end journey test
- 24-hour uptime verification
- Deployment retrospective
- 3 test users onboarded
**Success:** 3+ users making successful requests

---

## Agent Spawn Commands

### Wave 1 (Parallel - 3 agents)
```typescript
Task("Wave 1 Track A: Enable postgres feature", {
  type: "Engineer",
  tasks: ["P1-S1-03", "P1-S1-06"],
  deliverable: "noesis-api Cargo.toml with postgres feature, DATABASE_URL validation"
})

Task("Wave 1 Track B: Optimize Dockerfile", {
  type: "Engineer",
  tasks: ["P1-S1-10"],
  deliverable: "Dockerfile.prod with layer caching, <3min builds"
})

Task("Wave 1 Track C: Configure Railway", {
  type: "Engineer",
  tasks: ["P1-S1-11", "P1-S1-12"],
  deliverable: "All env vars set, Redis provisioned"
})
```

### Wave 2 (Sequential - 3 agents)
```typescript
Task("Wave 2 Track A: Deploy Railway", {
  type: "Engineer",
  tasks: ["P1-S1-13"],
  blocker: "Wave 1 complete",
  deliverable: "Railway deployment 200 OK"
})

Task("Wave 2 Track B: Seed API keys", {
  type: "Engineer",
  tasks: ["P1-S1-07"],
  blocker: "Track A deployed",
  deliverable: "5 test keys in Supabase"
})

Task("Wave 2 Track C: Integration tests", {
  type: "QATester",
  tasks: ["P1-S1-08", "P1-S1-14", "P1-S1-15"],
  blocker: "Track B seeded",
  deliverable: "Auth + workflow tests passing"
})
```

---

## Success Metrics

**Phase 1 Complete:**
- ✅ Railway deployment 200 OK
- ✅ API keys persist across restarts
- ✅ Redis L2 cache operational
- ✅ Authenticated requests work
- ✅ Workflows execute successfully

**Phase 2 Complete:**
- ✅ Production domain SSL
- ✅ Sentry capturing errors
- ✅ Posthog tracking events
- ✅ BetterStack 100% uptime (24h)
- ✅ 3+ test users onboarded

---

## Next Action

**IMMEDIATE:** Launch Wave 1 parallel agents to fix Railway 502
