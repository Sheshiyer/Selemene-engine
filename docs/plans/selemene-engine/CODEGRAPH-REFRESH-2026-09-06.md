# CodeGraph refresh — 2026-09-06

Scope: recovery candidate `4305265acee96461c40594fbb2689306d357f59d` plus the uncommitted review and evidence documents that will accompany it. This receipt records a local structural index. It does not establish deployed source, runtime health, or semantic engine correctness.

## Synchronization receipt

| Step | Result |
|---|---|
| `codegraph index . --force` | Cleared the prior index, scanned 867 files, indexed 820 supported source files, and completed successfully |
| `codegraph sync .` | Already up to date after the forced rebuild |
| `codegraph status .` | Up to date; 867 files, 15,164 nodes, 37,101 edges, 44.63 MB, `node:sqlite` WAL backend |

The refreshed language inventory contains 477 Rust, 208 TypeScript, 72 Python, 47 YAML, 43 TSX, and 20 JavaScript files. The main symbol inventory contains 5,073 functions, 2,029 methods, 1,031 structs, 409 interfaces, and 65 classes.

## Functional queries and impact checks

All queries resolved the intended current-source declaration.

| Symbol | Resolved declaration | Depth-three impact |
|---|---|---|
| `registerTypeScriptRuntimeEngines` | `ts-engines/src/server/registry.ts:79` | 3 nodes / 2 edges; includes the server registry and public TS entry point |
| `register_database_conditional_engines` | `crates/noesis-api/src/lib.rs:3977` | 73 nodes / 72 edges; includes both app-state construction and API/e2e/security paths |
| `WorkflowOrchestrator::register_native_runtime_engines` | `crates/noesis-orchestrator/src/lib.rs:306` | 161 nodes / 160 edges; includes runtime construction, API entry, and integration/security tests |
| `validate_repo_reference` | `scripts/validate_contracts.py:564` | 41 nodes / 40 edges; includes registry authority validation and the adversarial resolver suite |
| `validate_release_receipt` | `scripts/validate_release_receipt.py:148` | 19 nodes / 18 edges; includes fixture validation and operational receipt tests |

The graph confirms the three runtime-registration seams and both release-authority validators are present and connected to their expected consumers. The wider impact counts are planning inputs for Phase 3 contract convergence; they are not evidence that every affected path has executed successfully.

## Boundary

The graph contains the final parser-backed source-anchor validator and its expanded tests. The independent exact-source review remains the authority for the resolver behavior, while the full repository gate and exact-head GitHub Actions run remain the executable acceptance gates.
