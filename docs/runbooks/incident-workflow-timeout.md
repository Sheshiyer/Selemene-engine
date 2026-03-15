# Runbook: Orchestrator Workflow Timeout / Partial Engine Failure

This runbook covers workflow incidents where:

- a workflow succeeds but returns fewer `engine_outputs` than expected
- a full-spectrum execution records one or more timed-out engines
- the outer API request times out before the workflow finishes

Treat these as three related but distinct failure modes.

## Current Runtime Model

### Standard workflow execution

The primary workflow path in
[crates/noesis-orchestrator/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/lib.rs)
uses `WorkflowOrchestrator::execute_workflow()`:

- engines run concurrently via `join_all`
- engine failures are logged and omitted from `engine_outputs`
- missing engines are skipped
- phase-gated engines are skipped
- the workflow itself still returns `200` if at least the orchestrator path completes

Relevant warning messages already emitted by the runtime:

- `Engine failed, omitting from results`
- `Engine not found in registry, skipping`
- `Phase access denied, skipping engine`

### Full-spectrum execution

The full-spectrum path in
[crates/noesis-orchestrator/src/workflow/full_spectrum.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/workflow/full_spectrum.rs)
adds explicit per-engine timeout handling:

- `FullSpectrumConfig.engine_timeout` defaults to `5s`
- each engine is wrapped in `tokio::time::timeout(...)`
- timed-out engines are recorded in `failed_engines`
- successful engines still populate `successful_outputs`

### Outer API request timeout

The API also has an outer request timeout controlled by `REQUEST_TIMEOUT_SECS`
in
[crates/noesis-api/src/config.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/config.rs).
That timeout is separate from workflow engine handling:

- it applies to the entire HTTP request
- it returns `504 Gateway Timeout`
- it does not preserve partial workflow results once the request is aborted

Use this distinction during triage:

- `200` with missing `engine_outputs` => standard workflow graceful degradation
- `200` full-spectrum result with timeout entries in `failed_engines` => per-engine full-spectrum timeout
- `504` => outer API timeout, not orchestrator partial-result behavior

## Symptoms

- `POST /api/v1/workflows/:id/execute` returns `200` but one or more expected engines are missing from `engine_outputs`
- `full-spectrum` output contains `failed_engines` entries such as `Timeout after 5s`
- logs show repeated workflow warnings for the same engine ID
- request latency spikes and operators see slow workflow traces in Jaeger
- Sentry shows new `5xx` API events tied to a workflow request
- the API returns `504 Gateway Timeout` for a workflow route

## Diagnosis

### 1. Capture the incident context

Record:

- workflow ID
- approximate request time
- user phase if known
- whether the response was `200` or `504`
- request `trace_id` if an error response included one

`trace_id` correlation is created by
[crates/noesis-api/src/middleware.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/middleware.rs),
and `5xx`/`4xx` API errors are pushed into Sentry breadcrumbs and tags by
[crates/noesis-api/src/error_mapper.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/error_mapper.rs).

### 2. Distinguish the failure mode

Use the response shape first:

- standard workflow partial-result case:
  - HTTP `200`
  - `engine_outputs` missing one or more workflow engines
- full-spectrum timeout case:
  - HTTP `200`
  - `failed_engines` contains timeout strings
- outer request timeout case:
  - HTTP `504`
  - no workflow payload was returned

### 3. Identify the missing or slow engine

Compare the response against the workflow definition in
[crates/noesis-orchestrator/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/lib.rs).

Then correlate logs:

```logql
{app="noesis-api"} | json | trace_id="REPLACE_ME"
```

Watch for:

- `Engine failed, omitting from results`
- `Engine not found in registry, skipping`
- `Phase access denied, skipping engine`

If the issue is full-spectrum-specific, inspect the `failed_engines` map in the
response body first. That is the fastest path to the timing offender.

### 4. Use Jaeger to isolate the slow span

Jaeger is documented in
[docs/deployment/monitoring.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/deployment/monitoring.md).

Search by:

- service: `noesis-api`
- time window around the incident
- trace attributes:
  - `workflow.id`
  - `engine.id`

Expected workflow trace behavior:

- one parent request / workflow span
- child spans for engine calculations or bridge calls
- the failing engine will show the longest span or a missing completion path

Goal for tabletop triage:

- identify the slow or failing engine in Jaeger within 2 minutes

### 5. Use Prometheus for per-engine latency and error trends

The reliable repo-visible engine metric today is:

- `noesis_engine_calculation_duration_seconds`

Useful queries:

```promql
histogram_quantile(
  0.95,
  rate(noesis_engine_calculation_duration_seconds_bucket{engine_id="human-design"}[5m])
)
```

```promql
sum(rate(noesis_engine_calculation_errors_total{engine_id="human-design"}[5m]))
```

Use these to answer:

- is one engine slower than its baseline?
- is the problem isolated to a single engine or systemic?

### 6. Correlate Sentry when the incident includes a `5xx`

For API errors, `ErrorMapper` already:

- adds an `api.error` breadcrumb
- tags captured events with `trace_id`
- tags captured events with `error_code`

Use the error response `trace_id` or the request log `trace_id` to filter the
event in Sentry and confirm whether the workflow issue also triggered:

- bridge failures
- internal errors
- config/runtime drift

## Mitigation

### Single-engine failure inside a standard workflow

If one engine is failing but the workflow is otherwise usable:

1. keep serving partial results
2. declare the workflow degraded, not down
3. route the failing engine to the domain owner
4. if it is a bridged TS engine, switch to
   [incident-ts-bridge-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-ts-bridge-failure.md)

### Full-spectrum per-engine timeout

If the incident is limited to `full-spectrum`:

1. confirm the timed-out engine from `failed_engines`
2. prefer partial results over total failure
3. reduce incident scope by advising operators to use a narrower workflow if possible
4. note that the per-engine timeout is code-configured via `FullSpectrumConfig.engine_timeout`
   and there is no separate live control plane for changing it during an incident

### Outer `504 Gateway Timeout`

If the request itself timed out:

1. treat this as an API timeout incident first
2. check `REQUEST_TIMEOUT_SECS` configuration and overall latency pressure
3. inspect whether the slowness is:
   - one pathological engine
   - bridge latency
   - DB / Redis degradation
4. if the route is broadly timing out, pivot to:
   - [incident-api-down.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-api-down.md)
   - [incident-db-pool-exhaustion.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-db-pool-exhaustion.md)
   - [incident-redis-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-redis-failure.md)

## Recovery Verification

1. Re-run the affected workflow.
2. Confirm expected engines are present in `engine_outputs`, or confirm the workflow is intentionally degraded with one known missing engine.
3. For `full-spectrum`, confirm `failed_engines` no longer records timeout entries for the remediated engine.
4. Confirm per-engine latency returns to baseline in Prometheus.
5. Confirm Jaeger traces no longer show a pathological workflow/engine span.
6. If the incident involved a `5xx`, confirm Sentry stops receiving new workflow-related error events for the same `trace_id` pattern.

## Runtime Proof Points

These tests already support the incident model above:

- partial workflow failure still succeeds:
  - [crates/noesis-api/tests/workflow_execution_tests.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/tests/workflow_execution_tests.rs)
  - `test_workflow_partial_failure_graceful_degradation`
- all engines failing still yields a workflow result:
  - [crates/noesis-api/tests/workflow_execution_tests.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/tests/workflow_execution_tests.rs)
  - `test_workflow_all_engines_fail_still_succeeds`
- full-spectrum timeout handling:
  - [crates/noesis-orchestrator/src/workflow/full_spectrum.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/workflow/full_spectrum.rs)
  - `test_timeout_handling`

## Escalation

Escalate in this order:

1. on-call engineer
2. workflow/orchestrator owner
3. specific engine owner if the offender is isolated
4. platform owner if the incident is actually an API timeout or dependency incident
