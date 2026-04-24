# Root Binary Consolidation - Requirements

**Feature Name:** root-binary-consolidation  
**Created:** 2026-02-08  
**Status:** Draft  
**Priority:** High  
**Context:** Cargo.toml was edited to add dependencies to workspace root package

---

## Problem Statement

The codebase currently has **two parallel HTTP server implementations**:

1. **Root binary** (`src/main.rs`) - Uses `selemene-engine` package with basic Axum server
2. **noesis-api crate** (`crates/noesis-api`) - Production-grade API with full middleware stack

The MVP deployment plan (`.claude/task-management/mvp-deploy.md`) references `noesis-api` as the deployment target, but recent changes suggest development is happening on the root binary instead.

### Current State

**Root Binary (`src/main.rs`):**
- ✅ Basic Axum HTTP server
- ✅ Panchanga calculation endpoint
- ✅ Graceful shutdown handling
- ❌ No authentication
- ❌ No rate limiting
- ❌ No metrics/observability
- ❌ No database integration
- ❌ No Redis caching
- ❌ Not referenced in deployment docs

**noesis-api Crate:**
- ✅ Production-grade middleware (auth, rate limiting, CORS)
- ✅ JWT + API key authentication
- ✅ Prometheus metrics
- ✅ SwaggerUI documentation
- ✅ Health check endpoints
- ✅ Redis L2 cache integration
- ✅ Referenced in all deployment documentation
- ❓ Status unclear - is it being replaced?

### Risk

**Deployment confusion:** The MVP deployment tasks assume `noesis-api` is the binary to deploy, but development appears to be happening on the root binary. This creates:
- Wasted effort if the wrong binary is being developed
- Deployment failure if the wrong binary is deployed
- Documentation drift between code and deployment plans

---

## User Stories

### US-1: As a developer, I need clarity on which binary to develop
**Acceptance Criteria:**
- [ ] 1.1 There is ONE canonical HTTP server binary for production deployment
- [ ] 1.2 The chosen binary is clearly documented in README.md
- [ ] 1.3 The Dockerfile.prod builds the correct binary
- [ ] 1.4 The deployment docs reference the correct binary

### US-2: As a DevOps engineer, I need the deployment to use the production-ready binary
**Acceptance Criteria:**
- [ ] 2.1 Dockerfile.prod builds the binary with all production features
- [ ] 2.2 Railway deployment uses the correct binary
- [ ] 2.3 Health checks work on the deployed binary
- [ ] 2.4 All middleware (auth, rate limiting, metrics) is present

### US-3: As a maintainer, I need to avoid duplicate code
**Acceptance Criteria:**
- [ ] 3.1 There is no duplicate HTTP server implementation
- [ ] 3.2 Shared code is in library crates, not duplicated
- [ ] 3.3 The workspace structure is clean and logical

---

## Proposed Solutions

### Option A: Use noesis-api (Recommended)

**Rationale:** The deployment plan already assumes `noesis-api` is the production binary. It has all production features implemented.

**Changes Required:**
1. Remove or archive `src/main.rs` and `src/lib.rs` from workspace root
2. Update `Cargo.toml` to remove `[dependencies]` section (workspace root becomes pure workspace)
3. Verify `Dockerfile.prod` builds `noesis-api` binary (likely already does)
4. Update README.md to clarify `noesis-api` is the production binary
5. Move any unique features from root binary to `noesis-api` if needed

**Pros:**
- ✅ Aligns with existing deployment plan
- ✅ No need to reimplement production features
- ✅ Clean workspace structure
- ✅ Less code to maintain

**Cons:**
- ❌ Need to migrate any unique features from root binary
- ❌ Recent work on root binary may need to be redone

### Option B: Consolidate into Root Binary

**Rationale:** Continue developing the root binary and migrate production features from `noesis-api`.

**Changes Required:**
1. Migrate authentication from `noesis-auth` to root binary
2. Migrate rate limiting middleware
3. Migrate metrics/observability
4. Migrate health check endpoints
5. Migrate SwaggerUI documentation
6. Update deployment docs to reference root binary
7. Update Dockerfile.prod to build root binary

**Pros:**
- ✅ Preserves recent work on root binary
- ✅ Simpler workspace structure (fewer crates)

**Cons:**
- ❌ Duplicates work already done in `noesis-api`
- ❌ Requires updating all deployment documentation
- ❌ Higher risk of missing production features
- ❌ More work to reach MVP

### Option C: Dual Binary Strategy

**Rationale:** Keep both binaries for different purposes.

**Changes Required:**
1. Rename root binary to `selemene-simple` (lightweight demo server)
2. Keep `noesis-api` as production binary
3. Document the purpose of each binary clearly
4. Ensure Dockerfile.prod builds `noesis-api`

**Pros:**
- ✅ Preserves both implementations
- ✅ Provides a simple demo server for development
- ✅ Production binary remains unchanged

**Cons:**
- ❌ Maintenance burden of two servers
- ❌ Potential confusion about which to use
- ❌ Code duplication

---

## Recommendation

**Choose Option A: Use noesis-api**

### Reasoning

1. **Deployment plan alignment:** All MVP deployment tasks assume `noesis-api` is the production binary
2. **Feature completeness:** `noesis-api` already has all production features (auth, rate limiting, metrics, health checks)
3. **Time to MVP:** Using existing production-ready code is faster than rebuilding
4. **Risk reduction:** Less chance of missing critical production features

### Migration Path

1. **Audit root binary** - Identify any unique features not in `noesis-api`
2. **Migrate unique features** - Port any valuable code to `noesis-api` or supporting crates
3. **Archive root binary** - Move `src/` to `archive/` or delete entirely
4. **Clean Cargo.toml** - Remove `[dependencies]` from workspace root
5. **Verify deployment** - Ensure Dockerfile.prod builds `noesis-api`
6. **Update documentation** - Clarify in README.md that `noesis-api` is the production binary

---

## Questions to Resolve

1. **Why was the root binary being developed?** Was there a specific reason to avoid `noesis-api`?
2. **Are there unique features in the root binary?** Does it have anything `noesis-api` lacks?
3. **What is the intended deployment target?** Should Railway deploy the root binary or `noesis-api`?
4. **Is there a migration plan?** If switching binaries, what's the migration strategy?

---

## Next Steps

1. **User decision required:** Which option (A, B, or C) should we pursue?
2. **Create design document:** Once option is chosen, create detailed design
3. **Create task list:** Break down implementation into concrete tasks
4. **Update deployment docs:** Ensure all docs reference the correct binary

---

## Related Documents

- `.claude/task-management/mvp-deploy.md` - MVP deployment plan (assumes `noesis-api`)
- `.claude/task-management/mvp-deploy-tasks.json` - Deployment tasks
- `Dockerfile.prod` - Production Docker build
- `.context/CODEBASE_INDEX.md` - Codebase overview

---

## Acceptance Criteria Summary

This feature is complete when:

- [ ] There is ONE clearly documented production binary
- [ ] Dockerfile.prod builds the correct binary
- [ ] All deployment docs reference the correct binary
- [ ] No duplicate HTTP server implementations exist
- [ ] README.md clearly states which binary to use for production
- [ ] Railway deployment uses the correct binary
- [ ] All production features (auth, rate limiting, metrics) are present in the deployed binary
