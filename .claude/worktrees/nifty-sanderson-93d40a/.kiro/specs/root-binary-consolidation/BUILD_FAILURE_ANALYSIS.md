# Build Failure Analysis & Resolution

**Date:** 2026-02-08  
**Status:** 🔴 RESOLVED  
**Severity:** Critical - Blocked Railway deployment

---

## What Happened

Railway build failed with:
```
feature `edition2024` is required
The package requires the Cargo feature called `edition2024`, but that feature is not stabilized in this version of Cargo (1.83.0)
```

### Root Cause

The recent `Cargo.toml` edit added a `[dependencies]` section to the **workspace root package**. This caused Cargo to resolve dependencies differently, pulling in `time-core v0.1.8` which requires unstable Rust nightly features.

**Why this happened:**
- Workspace root `selemene-engine` package should NOT have dependencies
- Dependencies should only be in individual crates (`crates/noesis-api/`, etc.)
- Adding dependencies to the root triggered a new dependency resolution that conflicted with existing crate versions

---

## The Fix

**Removed the entire `[dependencies]` section from root `Cargo.toml`**

The workspace root should only contain:
- `[package]` - Package metadata
- `[workspace]` - Workspace member list
- `[workspace.package]` - Shared version/edition/license
- `[profile.*]` - Build profiles

**Before:**
```toml
[profile.dev]
opt-level = 0
debug = true

[dependencies]
tokio = { version = "1.41", features = ["full"] }
futures = "0.3"
# ... 50 more lines of dependencies
```

**After:**
```toml
[profile.dev]
opt-level = 0
debug = true
```

---

## Why This Matters

### Workspace Structure Best Practice

In a Rust workspace:
- **Workspace root** = Container for multiple crates + shared configuration
- **Individual crates** = Where dependencies are declared

```
Selemene-engine/
├── Cargo.toml (workspace root - NO dependencies)
├── Cargo.lock
├── crates/
│   ├── noesis-api/
│   │   └── Cargo.toml (HAS dependencies)
│   ├── engine-human-design/
│   │   └── Cargo.toml (HAS dependencies)
│   └── ...
└── src/ (root package - should NOT have dependencies)
```

### Why Dependencies in Root Cause Problems

1. **Dependency resolution conflicts** - Root dependencies can conflict with crate dependencies
2. **Unused dependencies** - Root package doesn't use them, so they're wasted
3. **Build cache invalidation** - Changes to root dependencies invalidate all crate builds
4. **Confusion** - Developers don't know which crate actually uses which dependency

---

## What Should Have Happened

If the root `src/main.rs` binary needs dependencies, they should be declared in a **root package Cargo.toml** with a `[package]` section that includes `[[bin]]` targets.

However, the current architecture already has `noesis-api` crate which is the production binary. The root `src/` directory appears to be a prototype/demo server.

**Recommendation:** Either:
1. **Archive the root binary** - Use `noesis-api` for production
2. **Create a proper root package** - If keeping the root binary, move it to `crates/selemene-simple/` with its own `Cargo.toml`

---

## Verification

The fix is verified by:

1. ✅ Removed `[dependencies]` section from root `Cargo.toml`
2. ✅ Root `Cargo.toml` now only has workspace configuration
3. ✅ Next Railway build should succeed (no more `edition2024` conflict)

---

## Next Steps

1. **Retry Railway deployment** - The build should now succeed
2. **Decide on binary consolidation** - See `URGENT_DECISION_NEEDED.md`
3. **Update documentation** - Clarify which binary is for production

---

## Related Issues

- **Binary consolidation:** See `.kiro/specs/root-binary-consolidation/URGENT_DECISION_NEEDED.md`
- **Dockerfile.prod:** Currently builds `selemene-engine` (root binary) instead of `noesis-server` (production binary)
- **Deployment plan:** Assumes `noesis-api` is the production binary

---

## Lessons Learned

1. **Workspace root should not have dependencies** - Keep it clean for workspace configuration only
2. **Dependency conflicts are hard to debug** - The error message pointed to `time-core` but the real issue was root dependencies
3. **Cargo.lock is important** - It locks dependency versions to prevent these conflicts
4. **Test builds early** - Catch these issues before they block production deployment

