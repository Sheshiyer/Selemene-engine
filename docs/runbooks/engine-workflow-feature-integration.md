# Engine And Workflow Feature Integration

## Purpose

Use this runbook when adding or extending an engine result shape, engine input option, or workflow passthrough. The goal is to touch only the owning surfaces and avoid re-scoping unrelated persistence or database systems.

## Decision Rule: Do We Need A Database Migration?

1. Check the storage model first.
2. If the new data only changes the shape of `result_data` or `input_data` and those fields are already `JSONB`, do not create a migration.
3. Only add a migration when one of these is true:
   - a new relational column is required for querying, filtering, or indexing
   - a uniqueness or foreign-key rule changes
   - a non-JSON API contract is persisted in typed columns

In this codebase, readings persist through:
- `crates/noesis-api/src/lib.rs`
- `crates/noesis-data/src/models/reading.rs`
- `crates/noesis-data/src/repositories/readings_repository.rs`
- `migrations/005_readings.sql`
- `supabase/migrations/20260225000005_005_readings.sql`

`result_data` and `input_data` are already `JSONB`, so most engine payload additions are append-only and do not need schema work.

## Minimal Integration Sequence

1. Update the owning engine input parser and result payload.
2. Add or update engine-level tests for the new shape.
3. If a workflow forwards the new option, update the workflow enum or mapper.
4. Add workflow-level tests for enum aliases and forwarded values.
5. Verify persistence only if the stored JSON shape changed.
6. Update public API docs and any internal context docs that show payload examples.
7. Update stale fixtures that describe result schemas.
8. Run only the targeted test commands for the touched engine/workflow.

## Surfaces To Check

### Engine feature
- TS engine or Rust engine implementation
- engine metadata / input schema
- HTTP integration tests
- docs for the engine
- expected output fixtures if present

### Workflow feature
- workflow input enum / option mapper
- workflow docs
- synthesis tests if output assumptions changed
- e2e request examples only if they rely on old option names

### Persistence feature
- confirm `JSONB` compatibility first
- add a repository regression test for nested payloads if needed
- avoid migrations unless queryable relational fields are changing

## Verification Commands

```bash
cd ts-engines && bun test src/engines/tarot/engine.test.ts tests/integration.test.ts
cargo test -p noesis-orchestrator decision_support -- --nocapture
cargo test -p noesis-data save_reading_preserves_nested_tarot_result_payloads -- --nocapture
```

If `DATABASE_URL` is not set, the persistence integration test may skip. That is acceptable for local verification, but the JSONB storage decision should still be documented in the PR or task summary.

## Anti-Patterns

- Do not design a new relational schema when `result_data` already stores the new shape.
- Do not update every engine fixture just because one engine changed.
- Do not widen to full-database discovery when the change is isolated to one engine/workflow path.
- Do not change docs without running the narrow tests that prove the payload.
