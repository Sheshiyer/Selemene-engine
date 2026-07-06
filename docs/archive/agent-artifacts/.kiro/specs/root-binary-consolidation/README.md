# Root Binary Consolidation Spec

**Status:** 🟡 In Progress  
**Created:** 2026-02-08  
**Priority:** High  
**Blocks:** MVP Deployment (Phase 1)

---

## Quick Summary

Your codebase has **two HTTP server binaries** and the recent `Cargo.toml` edit caused a build failure. This spec documents the issue and provides a decision framework.

### What Happened
1. Someone added dependencies to root `Cargo.toml`
2. This caused a dependency conflict with `time-core v0.1.8`
3. Railway build failed with `edition2024` error
4. **Fix:** Removed dependencies from root `Cargo.toml` ✅

### What Needs to Happen
1. **Decide which binary to use** for production deployment
2. **Update Dockerfile.prod** to build the correct binary
3. **Proceed with MVP deployment**

---

## Documents in This Spec

| Document | Purpose | Read When |
|----------|---------|-----------|
| **RESOLUTION_SUMMARY.md** | What was fixed and next steps | First - start here |
| **URGENT_DECISION_NEEDED.md** | Decision matrix with 3 options | Need to decide which binary to use |
| **BUILD_FAILURE_ANALYSIS.md** | Technical deep-dive on the error | Want to understand what went wrong |
| **AFFECTED_FILES.md** | List of files that need changes | Ready to implement the fix |
| **requirements.md** | Full requirements document | Need complete context |

---

## The Two Binaries

### Binary #1: `selemene-engine` (Workspace Root)
- **Location:** `src/main.rs`
- **Built by:** `Dockerfile.prod` ✅
- **Status:** Basic Axum server
- **Features:** Panchanga endpoint, graceful shutdown
- **Missing:** Auth, rate limiting, metrics, Redis, database, SwaggerUI

### Binary #2: `noesis-server` (noesis-api crate)
- **Location:** `crates/noesis-api/src/main.rs`
- **Built by:** `Dockerfile` (not Dockerfile.prod)
- **Status:** Production-ready
- **Features:** JWT auth, API keys, rate limiting, Prometheus metrics, SwaggerUI, health checks, Redis L2 cache
- **Referenced by:** ALL deployment documentation

---

## The Decision

**You need to choose ONE of these options:**

### Option A: Use noesis-api (RECOMMENDED) ⭐
- Update `Dockerfile.prod` to build `noesis-server`
- Remove or archive `src/` directory
- Proceed with MVP deployment as planned
- **Time:** 30 minutes
- **Risk:** Low
- **Benefit:** Fastest path to MVP, all features already implemented

### Option B: Use selemene-engine
- Migrate production features from `noesis-api` to root binary
- Update all deployment docs
- **Time:** 2-3 weeks
- **Risk:** High (missing features, documentation drift)
- **Benefit:** Simpler workspace structure

### Option C: Keep both
- Rename root binary to `selemene-simple` (demo server)
- Use `noesis-server` for production
- **Time:** 1 hour
- **Risk:** Medium (maintenance burden)
- **Benefit:** Preserves both implementations

---

## Current Status

### ✅ Completed
- [x] Build failure fixed (removed dependencies from root Cargo.toml)
- [x] Workspace structure validated (cargo check succeeds)
- [x] Spec created with decision framework
- [x] All affected files documented

### ⏳ Pending
- [ ] **Decision:** Which option (A, B, or C)?
- [ ] Update `Dockerfile.prod` to build correct binary
- [ ] Archive or delete unused binary
- [ ] Update README.md with binary documentation
- [ ] Retry Railway deployment
- [ ] Proceed with Phase 1 MVP tasks

---

## How to Use This Spec

### Step 1: Understand the Problem
Read: `RESOLUTION_SUMMARY.md`

### Step 2: Make a Decision
Read: `URGENT_DECISION_NEEDED.md`
Choose: Option A, B, or C

### Step 3: Implement the Fix
Read: `AFFECTED_FILES.md`
Follow: The implementation checklist

### Step 4: Verify
Run: `docker build -f Dockerfile.prod -t test .`
Test: `docker run test /app/noesis-server --help`

### Step 5: Deploy
Proceed with Phase 1 MVP tasks

---

## Key Files

**Workspace Configuration:**
- `Cargo.toml` - Workspace root (fixed ✅)
- `Cargo.lock` - Dependency lock file

**Binaries:**
- `src/main.rs` - Root binary (selemene-engine)
- `crates/noesis-api/src/main.rs` - Production binary (noesis-server)

**Docker:**
- `Dockerfile` - Builds noesis-server
- `Dockerfile.prod` - Currently builds selemene-engine (NEEDS UPDATE)

**Deployment:**
- `.claude/task-management/mvp-deploy.md` - Assumes noesis-api
- `.claude/task-management/mvp-deploy-tasks.json` - 42 tasks for MVP

---

## Timeline

**Today (2026-02-08):**
- ✅ Build failure fixed
- ⏳ Decision needed

**Tomorrow (2026-02-09):**
- [ ] Implement chosen option
- [ ] Update Dockerfile.prod
- [ ] Retry Railway deployment

**This Week:**
- [ ] Proceed with Phase 1 MVP tasks
- [ ] First test users onboarded

---

## Questions?

**Q: Why are there two binaries?**  
A: The root binary appears to be a prototype/demo server. The production binary is `noesis-server` in the `noesis-api` crate.

**Q: Which should I use?**  
A: Option A (use noesis-api) is recommended. It's production-ready and all deployment docs assume it.

**Q: What if I want to keep both?**  
A: Option C is possible. Rename the root binary to `selemene-simple` and use it as a demo server.

**Q: How long will this take?**  
A: Option A takes 30 minutes. Option B takes 2-3 weeks. Option C takes 1 hour.

**Q: Will this delay MVP?**  
A: Option A won't delay anything. Options B and C will delay by 2-3 weeks and 1 hour respectively.

---

## Next Action

👉 **Read `URGENT_DECISION_NEEDED.md` and choose Option A, B, or C**

Once you decide, I'll implement the changes and update all documentation.

