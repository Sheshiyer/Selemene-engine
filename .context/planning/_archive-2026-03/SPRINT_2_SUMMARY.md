# Sprint 2: Aleph Launch — Cleanup & Pruning Summary

**Date:** 2026-02-11
**Tasks:** ALEPH-12 through ALEPH-24 (13 tasks)
**Status:** Complete

---

## Metrics

| Metric | Value |
|--------|-------|
| Files deleted | 40 |
| Files formatted (Rust) | 2 |
| Files formatted (TypeScript) | 0 (already clean) |
| Compilation errors introduced | 0 |
| Test regressions introduced | 0 |

---

## Completed Tasks

### Deletions (ALEPH-12, 13, 17, 20, 21)

| What | Size | Reason |
|------|------|--------|
| `legacy/` directory | 16KB (Cargo.toml + 3 symlinks) | Not in workspace, dead code |
| `archive/root-binary-prototype/` | 348KB (33 Rust source files) | Historical prototype, preserved in git history |
| `archive/` parent directory | Empty after removal | Cleaned up |
| `scripts/test_gene_keys_frequency.sh` | 1.8KB | Deprecated test script |
| `scripts/verify_phase3.sh` | 5KB | Deprecated verification script |
| `.claude/crystalline-giggling-trinket.md` | 13KB | Old session file, no longer needed |

### Formatting (ALEPH-18, 19)

- **Rust (`cargo fmt --all`):** 2 active crate files reformatted
  - `crates/noesis-api/src/lib.rs`
  - `crates/noesis-data/src/repositories/readings_repository.rs`
- **TypeScript (Biome):** 32 files checked — all already clean, 0 fixes needed

### Audits (ALEPH-14, 22, 23)

| Area | Finding | Decision |
|------|---------|----------|
| `.gitkeep` files (4) | All in empty or near-empty directories | KEEP — serving intended purpose |
| `data/` duplicates | `types.json` appears in enneagram/ and human-design/ — different domains, not duplicates | No action needed |
| `k8s/` manifests (64KB) | Full Kubernetes setup (deployments, services, ingress, HPA, cert-manager) | KEEP — future K8s migration option; Railway remains primary |

### Crate Evaluations (ALEPH-15, 16)

| Crate | LOC | Purpose | Decision |
|-------|-----|---------|----------|
| `noesis-witness` | 34 | Consciousness-level witness prompt generation | KEEP as separate crate |
| `noesis-western-api` | 303 | HTTP client for freeastrologyapi.com | KEEP as separate crate |

---

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check --workspace` | PASS (clean compile) |
| `cargo fmt --all -- --check` | PASS (no diffs) |
| `cargo test --workspace` | Pre-existing: `engine-vimshottari` test compile error (missing `VedicPlanet` import in test code — not caused by Sprint 2) |
| `bun run lint` (ts-engines/) | PASS (0 issues) |
| Deleted dirs gone | PASS (legacy/, archive/, scripts removed, session file removed) |

---

## Pre-existing Issues (Not Sprint 2 Scope)

- `engine-vimshottari`: Test code has undeclared `VedicPlanet` type in `witness.rs:143` — needs `use crate::VedicPlanet;` import
- `sqlx-postgres v0.7.4`: Future-incompat warning (known)
