# Runbook: Orchestrator Workflow Timeout / Partial Engine Failure

This runbook covers workflows that return partial engine results or miss expected engine outputs.

## Current Runtime Behavior

The orchestrator in [crates/noesis-orchestrator/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/lib.rs) currently behaves like this:

- engines in a workflow are executed concurrently
- individual engine failures are logged and omitted
- the workflow can still succeed with partial `engine_outputs`

This is already covered by the orchestrator tests for:

- engine failure handling
- missing engine skip behavior
- phase-gated engine omission

## Symptoms

- workflow returns fewer engine outputs than expected
- one engine silently missing from a synthesis response
- warnings in logs for:
  - `Engine failed, omitting from results`
  - `Engine not found in registry, skipping`
  - `Phase access denied, skipping engine`

## Diagnosis

1. Identify the workflow ID and expected engines.
2. Compare the response `engine_outputs` against the workflow definition.
3. Inspect logs using the request `trace_id`.
4. Use monitoring references in [docs/deployment/monitoring.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/deployment/monitoring.md):
   - Jaeger tracing for workflow spans
   - Grafana `Workflow Metrics`
   - engine calculation timing

## Important Current Limitation

This runbook exists so the operational set is complete, but the current codebase does **not** expose:

- per-engine timeout settings in the orchestrator
- a dedicated workflow-timeout control plane

Current timeout-related behavior is mostly global request timeout / bridge timeout behavior, not orchestrator-specific per-engine timeouts.

## Mitigation

1. Determine whether the missing output is due to:
   - engine failure
   - phase gating
   - missing engine registration
   - sidecar degradation for a bridged engine
2. If the failure is isolated to one engine:
   - treat the workflow as partially degraded
   - continue serving partial results if acceptable
3. If multiple engines are missing:
   - escalate as a broader workflow incident
4. If the missing engine is a TS sidecar engine:
   - use [incident-ts-bridge-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-ts-bridge-failure.md)

## Recovery Verification

1. re-run the affected workflow
2. confirm expected engine IDs are present in `engine_outputs`
3. confirm logs no longer show repeated engine omission warnings

