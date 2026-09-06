# CodeGraph recovery — 2026-09-05

Source: `9a05f5cc90777f083d7065e491484ef727aeada1` in the main Selemene checkout. Initial MCP status returned `CodeGraph not initialized`; `codegraph init` and `codegraph index` completed. Fresh MCP status reported 859 indexed files, 14,850 nodes, 36,207 edges, 41.21 MB, node:sqlite WAL/FTS5. `codegraph sync` then returned `Already up to date`.

Language entries: Rust 477, TypeScript 207, TSX 43, Python 67, JavaScript 19, YAML 46. There are 813 code file nodes; YAML/config indexing explains the distinction from the total file count. Excluded files and non-code assets are not certified by these counts.

A functional symbol probe resolved `engine_capabilities` to `crates/noesis-api/src/handlers/admin.rs:4643`, the route to `crates/noesis-api/src/lib.rs:888`, and `listCapabilities` to `ts-engines/src/server/registry.ts:43`. A context probe returned the actual handler and its permission check. It calls `state.bridge().engines()`; it is not a native engine capability inventory.

Use CodeGraph context first, then one focused explore/impact/trace for the selected boundary. Query the correct `projectPath`; independent worktrees need their own index. Re-sync after source edits. The index is navigation evidence, not compilation, semantic correctness, deployed revision or complete documentation coverage.

The graph database remains local and ignored. Runtime index receipts are retained separately from source changes. Main checkout WIP, existing worktrees and the planning stash were preserved.

## Recovery worktree verification

The isolated Superset recovery worktree has its own index. Incremental sync reported up-to-date but a targeted query missed the newly added merge-lane test. A forced full index was therefore run and checked, rather than trusting that status alone. Fresh MCP status then reported 863 files, 14,905 nodes, 36,284 edges, 42.85MB, 70 Python and 47 YAML entries. Exact symbol probes resolve `engine_capabilities` and the new `test_merge_lane_safety` at `tests/scripts/test_merge_lane_safety.py:117`.

Use exact symbol queries before broad natural-language context: the broad context request returned unrelated API-key symbols, while the exact handler query resolved correctly. Keep this retrieval limitation explicit; graph availability is not a guarantee that every semantic search is relevant.

## Final source-content check

At candidate `9b618de`, incremental sync again reported up-to-date while `nextConfig` still showed its old standalone value. A second forced full index fixed the stale entry. Fresh symbol queries then showed the conditional Vercel packaging comment and the merge-lane safety test. The index reports 863 files, 14,905 nodes, 36,284 edges and 44.00 MB. Treat forced indexing plus a changed-symbol readback as the reliable update procedure until incremental invalidation is repaired.

After the capability fixture repair, the final forced index reports 863 files, 14,909 nodes and 36,292 edges (44.00 MB). `test_runtime` resolves to `crates/noesis-api/tests/capability_route_tests.rs:20`, and its static runtime is at line 18. Both the changed Next configuration and the new runtime helper resolve from the current source.
