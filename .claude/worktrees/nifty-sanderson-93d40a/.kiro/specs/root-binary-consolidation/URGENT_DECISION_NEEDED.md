# 🚨 URGENT: Binary Consolidation Decision Required

**Date:** 2026-02-08  
**Severity:** High - Blocks MVP Deployment  
**Decision Deadline:** Before Phase 1 deployment

---

## The Problem

Your codebase has **TWO HTTP server binaries** and the recent Cargo.toml edit suggests confusion about which one to use:

### Binary #1: `selemene-engine` (Workspace Root)
- **Location:** `src/main.rs`
- **Built by:** `Dockerfile.prod` ✅
- **Status:** Recently edited (dependencies just added)
- **Features:** Basic Axum server, Panchanga endpoint, graceful shutdown
- **Missing:** Auth, rate limiting, metrics, Redis, database, SwaggerUI

### Binary #2: `noesis-server` (noesis-api crate)
- **Location:** `crates/noesis-api/src/main.rs`
- **Built by:** `Dockerfile` (not Dockerfile.prod) ❌
- **Status:** Production-ready with full middleware stack
- **Features:** JWT auth, API keys, rate limiting, Prometheus metrics, SwaggerUI, health checks, Redis L2 cache
- **Referenced by:** ALL deployment documentation

---

## The Conflict

**Your deployment plan says:**
> "Deploy `noesis-api` to Railway with full production features"

**Your Dockerfile.prod does:**
> Builds `selemene-engine` (the basic binary without production features)

**Your recent edit suggests:**
> Development is happening on `selemene-engine` (workspace root)

---

## Impact on MVP Deployment

The MVP deployment tasks (42 tasks, 126 hours) assume:
- ✅ `noesis-api` has JWT authentication → **Actually in noesis-api, NOT selemene-engine**
- ✅ `noesis-api` has rate limiting → **Actually in noesis-api, NOT selemene-engine**
- ✅ `noesis-api` has Prometheus metrics → **Actually in noesis-api, NOT selemene-engine**
- ✅ `noesis-api` has health check endpoints → **Actually in noesis-api, NOT selemene-engine**
- ✅ `noesis-api` has SwaggerUI → **Actually in noesis-api, NOT selemene-engine**

**If you deploy `selemene-engine` (what Dockerfile.prod currently builds):**
- ❌ Phase 1 Task P1-S1-05 (AuthService refactor) will fail - no auth exists
- ❌ Phase 1 Task P1-S1-14 (test authenticated request) will fail - no auth
- ❌ Phase 2 Task P2-S1-07 (Sentry integration) will be incomplete - no middleware
- ❌ Phase 2 Task P2-S1-11 (Posthog analytics) will be incomplete - no middleware
- ❌ All observability tasks will fail - no metrics endpoint

---

## Decision Options

### Option A: Use noesis-api (RECOMMENDED) ⭐

**What to do:**
1. Update `Dockerfile.prod` to build `noesis-server` instead of `selemene-engine`
2. Remove or archive `src/main.rs` and `src/lib.rs` from workspace root
3. Remove `[dependencies]` section from root `Cargo.toml`
4. Proceed with MVP deployment as planned

**Why this is best:**
- ✅ All production features already implemented
- ✅ Deployment plan already written for this binary
- ✅ Fastest path to MVP
- ✅ Lowest risk

**Time to fix:** 30 minutes

### Option B: Migrate noesis-api features to selemene-engine

**What to do:**
1. Copy authentication from `noesis-auth` to root binary
2. Copy rate limiting middleware
3. Copy metrics/observability
4. Copy health check endpoints
5. Copy SwaggerUI setup
6. Update all deployment docs to reference `selemene-engine`

**Why this is NOT recommended:**
- ❌ Duplicates 2-3 weeks of work already done
- ❌ High risk of missing production features
- ❌ Delays MVP by 2-3 weeks
- ❌ Requires rewriting all deployment documentation

**Time to fix:** 2-3 weeks

### Option C: Keep both binaries

**What to do:**
1. Rename root binary to `selemene-simple` (demo/development server)
2. Update `Dockerfile.prod` to build `noesis-server`
3. Document that `noesis-server` is for production, `selemene-simple` is for demos

**Why this might work:**
- ✅ Preserves both implementations
- ✅ Provides simple demo server
- ⚠️ Maintenance burden of two servers

**Time to fix:** 1 hour

---

## Recommended Action Plan

**IMMEDIATE (Today):**

1. **✅ COMPLETED: Clean Cargo.toml**
   - The `[dependencies]` section has been successfully removed
   - Workspace root is now a pure workspace manifest

2. **NEXT: Fix Dockerfile.prod** to build the correct binary:
   ```dockerfile
   # Change line 53 from:
   RUN cargo build --release --bin selemene-engine
   
   # To:
   RUN cargo build --release --bin noesis-server
   
   # Change line 55 from:
   strip /build/target/release/selemene-engine
   
   # To:
   strip /build/target/release/noesis-server
   
   # Change line 68 from:
   COPY --from=builder /build/target/release/selemene-engine /app/selemene-engine
   
   # To:
   COPY --from=builder /build/target/release/noesis-server /app/noesis-server
   
   # Change line 99 from:
   ENTRYPOINT ["/app/selemene-engine"]
   
   # To:
   ENTRYPOINT ["/app/noesis-server"]
   ```

3. **Archive root binary** (optional but recommended):
   ```bash
   mkdir -p archive/root-binary-prototype
   mv src/ archive/root-binary-prototype/
   ```

4. **Verify the fix:**
   ```bash
   docker build -f Dockerfile.prod -t test .
   docker run test /app/noesis-server --version
   ```

**NEXT (Before deployment):**

5. Update README.md to clarify `noesis-server` is the production binary
6. Verify all deployment docs reference `noesis-server`
7. Proceed with Phase 1 deployment tasks

---

## Questions for You

1. **Why were you editing the root binary?** Was there a specific feature you needed that `noesis-api` lacks?

2. **Did you know about noesis-api?** It has all the production features already implemented.

3. **What should we do with the root binary?** Archive it, delete it, or keep it as a demo server?

---

## Files to Review

- `src/main.rs` - Root binary (basic server)
- `crates/noesis-api/src/main.rs` - Production binary (full features)
- `Dockerfile.prod` - Currently builds wrong binary
- `.claude/task-management/mvp-deploy.md` - Assumes noesis-api

---

## Next Steps

**Please decide:**
- [ ] Option A: Use noesis-api (recommended)
- [ ] Option B: Migrate features to selemene-engine (not recommended)
- [ ] Option C: Keep both binaries

Once you decide, I'll implement the changes and update all documentation.
