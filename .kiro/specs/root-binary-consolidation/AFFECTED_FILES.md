# Files Affected by Binary Consolidation

**Last Updated:** 2026-02-08

---

## Files That Need Changes (Option A - Recommended)

### 1. Dockerfile.prod ⚠️ CRITICAL
**Current state:** Builds `selemene-engine` (wrong binary)  
**Required change:** Build `noesis-server` instead

**Lines to change:**
- Line 53: `cargo build --release --bin selemene-engine` → `--bin noesis-server`
- Line 55: `strip /build/target/release/selemene-engine` → `/noesis-server`
- Line 68: `COPY --from=builder /build/target/release/selemene-engine` → `/noesis-server`
- Line 99: `ENTRYPOINT ["/app/selemene-engine"]` → `["/app/noesis-server"]`

### 2. Cargo.toml (Workspace Root) ✅ COMPLETED
**Current state:** `[dependencies]` section removed  
**Status:** Workspace root is now a pure workspace manifest

**Verified:**
- ✅ `[package]` section present
- ✅ `[workspace]` section present with all 22 member crates
- ✅ `[workspace.package]` section present
- ✅ `[profile.release]` section present (LTO enabled)
- ✅ `[profile.dev]` section present
- ✅ No `[dependencies]` section

### 3. src/ directory (Workspace Root) 📦 OPTIONAL
**Current state:** Contains root binary implementation  
**Required change:** Archive or delete

**Options:**
- **Archive:** `mv src/ archive/root-binary-prototype/`
- **Delete:** `rm -rf src/`
- **Keep:** Rename binary to `selemene-simple` for demo purposes

### 4. README.md 📝 DOCUMENTATION
**Current state:** May not clearly state which binary is for production  
**Required change:** Add section clarifying binary usage

**Add section:**
```markdown
## Binaries

This workspace produces the following binaries:

- **noesis-server** (Production) - Full-featured API server with authentication, rate limiting, metrics, and observability. Located in `crates/noesis-api/`. This is the binary deployed to production.

- **selemene-engine** (Demo/Development) - Lightweight demo server for development and testing. Located in workspace root `src/`. Not intended for production use.
```

### 5. .dockerignore 📝 OPTIONAL
**Current state:** May reference root src/  
**Required change:** Verify it doesn't exclude `crates/noesis-api/src/`

---

## Files That Reference the Binary

### Deployment Documentation
- `.claude/task-management/mvp-deploy.md` - ✅ Already references `noesis-api`
- `.claude/task-management/mvp-deploy-tasks.json` - ✅ Already references `noesis-api`
- `.claude/task-management/MVP_DEPLOYMENT_SUMMARY.md` - ✅ Already references `noesis-api`
- `.claude/RAILWAY_SETUP_CHECKLIST.md` - ✅ Already references `noesis-api`
- `.claude/RAILWAY_CONFIG_GUIDE.md` - ✅ Already references `noesis-api`

**Status:** All deployment docs already assume `noesis-api` is the production binary. No changes needed.

### Docker Files
- `Dockerfile` - ✅ Already builds `noesis-server`
- `Dockerfile.prod` - ❌ Currently builds `selemene-engine` (NEEDS FIX)
- `docker-compose.yml` - ✅ Uses `noesis-api` service name

### Codebase Documentation
- `.context/CODEBASE_INDEX.md` - ✅ References `noesis-api` as main HTTP server
- `.context/architecture/overview.md` - May need verification
- `.context/architecture/system-overview.md` - May need verification

---

## Verification Checklist

After making changes, verify:

- [ ] ✅ Root `Cargo.toml` has no `[dependencies]` section (COMPLETED)
- [ ] `Dockerfile.prod` builds `noesis-server` binary (NEXT)
- [ ] `docker build -f Dockerfile.prod -t test .` succeeds
- [ ] `docker run test /app/noesis-server --help` works
- [ ] `cargo build --workspace` succeeds
- [ ] README.md clarifies which binary is for production
- [ ] All deployment docs still reference correct binary

---

## Testing the Fix

```bash
# 1. Build production Docker image
docker build -f Dockerfile.prod -t selemene-test .

# 2. Verify correct binary is present
docker run selemene-test ls -lh /app/

# Expected output should show:
# -rwxr-xr-x 1 appuser appuser 45M Feb  8 10:00 noesis-server

# 3. Test binary runs
docker run -p 8080:8080 selemene-test

# 4. In another terminal, test health endpoint
curl http://localhost:8080/health

# Expected: {"status":"healthy","engines_loaded":9,"workflows_loaded":6}
```

---

## Rollback Plan

If changes cause issues:

```bash
# 1. Revert Dockerfile.prod
git checkout HEAD -- Dockerfile.prod

# 2. Revert Cargo.toml
git checkout HEAD -- Cargo.toml

# 3. Rebuild
docker build -f Dockerfile.prod -t selemene-rollback .
```

---

## Related Specs

- `requirements.md` - Full requirements document
- `URGENT_DECISION_NEEDED.md` - Decision matrix and recommendations
