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
