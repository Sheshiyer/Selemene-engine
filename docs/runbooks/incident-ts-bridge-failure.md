# Runbook: TS Sidecar Bridge Timeout / Circuit Breaker Open

This runbook covers TS sidecar outages or timeouts that make the five bridged engines unavailable while the Rust-native engine surface remains online.

Affected TS sidecar engines:

- `tarot`
- `i-ching`
- `enneagram`
- `sacred-geometry`
- `sigil-forge`

Rust-native engines that remain operational during a TS sidecar outage:

- `biofield`
- `biorhythm`
- `face-reading`
- `gene-keys`
- `human-design`
- `nadabrahman`
- `numerology`
- `panchanga`
- `transits`
- `vedic-clock`
- `vimshottari`

The current engine inventory is locked in [docs/baseline/engine-matrix.json](/Volumes/madara/2026/witnessos/Selemene-engine/docs/baseline/engine-matrix.json).

## Current Bridge Reality

The issue text refers to a "circuit breaker open" state. The current `noesis-bridge` implementation does **not** expose a dedicated circuit breaker object or explicit breaker state machine.

Operationally, use these current bridge states instead:

- `available`
  - `BridgeManager::health_check()` succeeds
  - sidecar `/health/ready` is healthy
- `degraded`
  - `BridgeManager::readiness_status()` returns some failed engines
  - `/ready` includes `bridge_status: "degraded"` and `bridge_failed_engines`
- `unavailable` / effective open state
  - `BridgeManager::health_check()` fails due to timeout or connection refusal
  - the TS sidecar is unreachable and all five bridged engines should be treated as unavailable

This is the operational equivalent of an "open" circuit for the current release.

## Symptoms

Look for one or more of these signals:

- TS-engine-backed calculations fail or time out
- `/ready` reports `bridge_status: "degraded"`
- `/ready` includes one or more IDs in `bridge_failed_engines`
- the sidecar `/health` endpoint stops responding
- logs or errors include:
  - bridge timeout
  - connection refused to the TS engines base URL
  - partial workflow results with TS engines missing

Relevant surfaces:

- `BridgeManager::health_check()` in [crates/noesis-bridge/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-bridge/src/lib.rs)
- `BridgeManager::readiness_status()` in [crates/noesis-bridge/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-bridge/src/lib.rs)
- `/ready` bridge fields in [crates/noesis-api/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/lib.rs)

## Top 5 Failure Modes

This runbook is intended to cover the most common bridge failure modes for on-call:

1. bridge timeout spike
2. sidecar crash loop
3. effective "circuit breaker open" state
4. TS engine memory leak / resource exhaustion
5. bridge schema mismatch

### 1. Bridge Timeout Spike

Symptoms:

- bridge requests start timing out
- `BridgeError::Timeout` / timeout-style failures increase
- sidecar `/health` may still respond, but engine calculations stall

Actions:

1. confirm `/health` and `/health/ready`
2. inspect sidecar logs for slow requests
3. check resource pressure before restarting
4. restart the sidecar if it is wedged and not recovering

### 2. Sidecar Crash Loop

Symptoms:

- `/health` is unreachable
- connection refused to `TS_ENGINES_URL`
- container/process repeatedly exits and restarts

Actions:

1. inspect process/container restart history
2. inspect boot logs for dependency or runtime failure
3. correct env/config drift if startup is failing
4. restart only after the root startup error is identified

### 3. Effective "Circuit Breaker Open" State

Current note:

- no dedicated circuit breaker object is exposed in the bridge today
- treat repeated timeout/connection-refused states plus bridge unavailability as the operational equivalent

Symptoms:

- all five TS engines are effectively unavailable
- `/ready` trends from `degraded` to a full sidecar-unavailable state

Actions:

1. treat the sidecar as unavailable
2. communicate partial platform degradation
3. rely on the 11 Rust-native engines while sidecar recovery is in progress

### 4. TS Engine Memory Leak / Resource Exhaustion

Symptoms:

- sidecar latency climbs over time
- intermittent timeouts turn into crash loops
- memory usage climbs until container restart

Actions:

1. inspect container/process memory usage
2. identify whether one engine endpoint is responsible
3. restart the sidecar to restore service
4. capture logs and repro context for follow-up engineering work

### 5. Bridge Schema Mismatch

Symptoms:

- sidecar responds, but Rust bridge deserialization fails
- readiness may be healthy while calculations fail with payload/shape errors
- new sidecar deployment introduced response shape drift

Actions:

1. inspect recent TS sidecar deployment/change history
2. inspect Rust bridge error logs for deserialization context
3. compare the failing response shape against the expected bridge contract
4. roll back the sidecar or align the schema before restoring traffic confidence

## Diagnosis

1. Check the API readiness surface.
   - `curl -s http://<noesis-host>/ready | jq`
   - inspect:
     - `bridge_status`
     - `bridge_engines`
     - `bridge_failed_engines`
2. Check the sidecar health endpoints directly.
   - `curl -s http://<ts-engines-host>:3001/health`
   - `curl -s http://<ts-engines-host>:3001/health/ready`
3. Confirm the configured sidecar base URL.
   - inspect `TS_ENGINES_URL`
4. Inspect TS sidecar logs.
   - local: `cd ts-engines && bun run dev`
   - containerized: inspect the `ts-engines` container logs
5. Determine the outage shape.
   - if `/health` fails entirely, treat the sidecar as unavailable
   - if `/health/ready` returns degraded data, treat this as a partial TS engine outage

## Mitigation

1. Preserve service on the Rust-native path.
   - the eleven Rust-native engines remain operational during TS sidecar outage
   - communicate partial degradation rather than full platform outage
2. Restore the sidecar.
   - restart the `ts-engines` process or container
   - fix the `TS_ENGINES_URL` configuration if it drifted
   - investigate Bun/runtime crashes or dependency issues in the sidecar logs
3. If the sidecar is timing out repeatedly:
   - verify the container is healthy and accepting connections
   - check resource pressure before repeatedly restarting it
4. If failures point to schema mismatch rather than availability:
   - stop treating the issue as pure infra
   - compare sidecar response payloads with the Rust bridge contract before rolling forward

## Partial-Service Communication

When the sidecar is down, user-facing impact is limited to the five bridged engines:

- unavailable or degraded:
  - `tarot`
  - `i-ching`
  - `enneagram`
  - `sacred-geometry`
  - `sigil-forge`
- still available:
  - the eleven Rust-native engines listed above

Operational messaging should explicitly say:

- the API is partially degraded
- TS bridge-backed engines are affected
- Rust-native engines remain available

## Recovery Verification

After recovery:

1. `BridgeManager::health_check()` equivalent succeeds through `GET /health`
2. sidecar `GET /health/ready` returns healthy
3. API `GET /ready` reports:
   - `bridge_status: "available"`
   - `bridge_failed_engines: []`
4. a representative TS engine request succeeds
5. a representative Rust-native engine request still succeeds
6. if the incident involved schema mismatch, confirm deserialization errors are gone after the fix or rollback

## Validation Reference

Current degraded bridge behavior is already exercised in [crates/noesis-api/tests/bridge_readiness_tests.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/tests/bridge_readiness_tests.rs), including:

- available sidecar readiness
- degraded sidecar readiness with explicit failed-engine details

Use that as the tabletop reference when simulating a sidecar crash or partial outage.

## Related References

- [docs/troubleshooting.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/troubleshooting.md)
- [docs/baseline/engine-matrix.json](/Volumes/madara/2026/witnessos/Selemene-engine/docs/baseline/engine-matrix.json)
- [crates/noesis-bridge/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-bridge/src/lib.rs)
- [crates/noesis-api/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/lib.rs)
- [crates/noesis-api/tests/bridge_readiness_tests.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/tests/bridge_readiness_tests.rs)
