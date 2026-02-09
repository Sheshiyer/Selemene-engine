# MVP Deployment Task Plan Summary

**Generated:** 2026-02-08
**Execution Started:** 2026-02-08 16:01 IST
**Target:** Railway (Hobby) + Supabase Pro
**Budget:** $37-47/month (revised from $31-41)
**Scale:** <50 test users
**Status:** 🔄 **Wave 1 In Progress**

---

## Pre-Flight Completed (Wave 0) ✅

- [x] **Secret leak fixed** - Removed Supabase password from .env.example line 77
- [x] **Railway plan corrected** - Updated to Hobby tier in task file
- [x] **Supabase verified** - Project already exists with credentials
- [x] **Railway verified** - Hobby tier, GitHub connected
- [x] **Cloudflare verified** - Domain tryambakam.space accessible
- [x] **Phase 2 scoped** - Observability deferred (Sentry/Posthog/BetterStack)

---

## Overview

This task plan breaks down the MVP deployment into **2 phases, 3 sprints, 42 tasks** with an estimated **126 hours** of work over **3 weeks**.

### Phase Breakdown

| Phase | Focus | Duration | Tasks | Hours |
|-------|-------|----------|-------|-------|
| **Phase 1** | Foundation - Database & Core Infrastructure | 1 week | 16 tasks | 57 hours |
| **Phase 2** | Production Readiness - Observability & Onboarding | 2 weeks | 26 tasks | 69 hours |

---

## Phase 1: Foundation (Week 1)

### Sprint 1: Supabase Integration & Railway Deployment

**Objective:** Migrate auth from in-memory to persistent Postgres storage, deploy to Railway with Redis

**Key Deliverables:**
1. **Database Layer** (Tasks 01-08)
   - Supabase project provisioned
   - Schema created: `api_keys`, `users`, `usage_logs`
   - AuthService refactored to use `sqlx::PgPool`
   - API key seeding script created
   - Unit tests for Postgres-backed auth

2. **Railway Deployment** (Tasks 09-16)
   - `railway.toml` configuration
   - Dockerfile optimized for build cache (2-3 min deploys)
   - Environment variables configured
   - Redis add-on provisioned
   - Health endpoints verified
   - Authenticated requests tested
   - Workflow execution tested
   - Deployment runbook documented

**Success Criteria:**
- ✅ API accessible at https://tryambakam.space
- ✅ API keys persist across restarts
- ✅ Redis L2 cache operational
- ✅ All health checks passing
- ✅ Authenticated requests work end-to-end

---

## Phase 2: Production Readiness (Weeks 2-3)

### Sprint 1: DNS, Error Tracking, and Analytics (Week 2)

**Objective:** Add production-grade observability and configure custom domain

**Key Deliverables:**
1. **DNS & Domain** (Tasks 01-03)
   - Domain registered and Cloudflare configured
   - Caching rules optimized
   - CORS updated for production domain

2. **Sentry Integration** (Tasks 04-09)
   - Sentry project created
   - `sentry-rust` + `sentry-tower` integrated
   - Error capture with full context (engine_id, tier, consciousness_level)
   - Fingerprinting for error grouping
   - Forced error test verified

3. **Posthog Analytics** (Tasks 10-12)
   - Posthog project created
   - Analytics middleware implemented (async, non-blocking)
   - Events: `engine_calculation`, `workflow_execution`, `auth_failure`, `rate_limit_hit`
   - Production events flowing to dashboard

4. **Uptime Monitoring** (Tasks 13-14)
   - BetterStack monitors configured
   - Public status page enabled

**Success Criteria:**
- ✅ Production domain (tryambakam.space) accessible with SSL
- ✅ Sentry capturing errors with context
- ✅ Posthog tracking usage events
- ✅ BetterStack monitoring uptime

### Sprint 2: User Onboarding & API Key Management (Week 3)

**Objective:** Build admin tools for key management and onboard first test users

**Key Deliverables:**
1. **Admin Endpoints** (Tasks 01-05)
   - `POST /api/v1/admin/keys` - Generate new API keys
   - `POST /api/v1/admin/keys/:key_id/revoke` - Revoke keys
   - `GET /api/v1/admin/keys` - List all keys
   - Integration tests for admin endpoints

2. **User Onboarding** (Tasks 06-12)
   - Test user documentation created
   - Initial 5 API keys generated
   - Usage monitoring dashboard query
   - End-to-end user journey tested
   - 24-hour uptime verification
   - Deployment retrospective
   - First 3 test users onboarded

**Success Criteria:**
- ✅ Admin can generate/revoke keys via API
- ✅ Test users can self-serve with documentation
- ✅ Usage monitoring operational
- ✅ 24-hour uptime achieved
- ✅ 3+ test users making successful requests

---

## Task Distribution by Area

| Area | Tasks | Hours | % of Total |
|------|-------|-------|------------|
| **Backend** | 15 tasks | 54 hours | 43% |
| **Infrastructure** | 13 tasks | 30 hours | 24% |
| **QA** | 8 tasks | 23 hours | 18% |
| **Product** | 4 tasks | 15 hours | 12% |
| **DevOps** | 2 tasks | 4 hours | 3% |

---

## Critical Path

The following tasks are on the critical path and must complete on schedule:

1. **P1-S1-02** → Database schema (blocks all auth work)
2. **P1-S1-05** → AuthService refactor (blocks deployment)
3. **P1-S1-13** → Railway deployment (blocks Phase 2)
4. **P2-S1-01** → Domain configuration (blocks production access)
5. **P2-S2-02** → Admin key generation (blocks user onboarding)

---

## Risk Mitigation

| Risk | Mitigation | Task |
|------|------------|------|
| Auth migration database dependency | 5s timeout + 503 fallback | P1-S1-05 |
| Railway cold starts | BetterStack keep-alive pings | P2-S1-13 |
| FreeAstrologyAPI rate limits | Caching + native fallback | Existing |
| Supabase connection limits | Supavisor pooling (port 6543) | P1-S1-01 |
| Docker build time | Layer caching optimization | P1-S1-10 |

---

## Success Metrics

### Technical Metrics
- **Uptime:** >99.9% (BetterStack)
- **Response Time:** p95 <500ms (Prometheus)
- **Error Rate:** <1% (Sentry)
- **Cache Hit Rate:** >80% (Redis metrics)

### Business Metrics
- **Test Users Onboarded:** ≥3 users
- **API Requests:** >100 successful requests
- **Engine Usage:** All 9 Rust engines tested
- **Workflow Usage:** ≥2 workflows executed

---

## Out of Scope (Deferred)

The following are explicitly **NOT** included in MVP:

- ❌ User-facing frontend (API-only)
- ❌ Supabase Auth integration (API keys sufficient)
- ❌ Kubernetes deployment (Railway sufficient)
- ❌ CI/CD changes (existing pipeline works)
- ❌ Performance optimization (already sub-2ms)
- ❌ Multi-region deployment (single region OK)
- ❌ TypeScript engines (unless required)

---

## Next Steps After MVP

1. **Collect feedback** from first 3 test users
2. **Monitor metrics** for 1 week (uptime, errors, usage patterns)
3. **Iterate on documentation** based on user questions
4. **Scale to 10-20 users** if metrics are healthy
5. **Plan Phase 3** (frontend, expanded features)

---

## Files Generated

- `.claude/task-management/mvp-deploy-tasks.json` - Complete task plan (42 tasks)
- `.claude/task-management/MVP_DEPLOYMENT_SUMMARY.md` - This summary document

---

**Ready to execute!** 🚀
