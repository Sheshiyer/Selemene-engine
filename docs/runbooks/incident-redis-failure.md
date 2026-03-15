# Runbook: Redis Connection Failure / Cache Degradation

This runbook covers a live Redis outage or connectivity failure in the Selemene cache stack.

Current cache topology:

- `L1`: in-memory cache
- `L2`: Redis distributed cache
- `L3`: disk-backed precomputed cache

Current degradation behavior is implemented in [crates/noesis-cache/src/l2_cache.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-cache/src/l2_cache.rs):

- Redis reads fall back to `Ok(None)` when a connection cannot be acquired.
- Redis writes fall back to `Ok(())` after logging a warning.
- API requests continue serving through `L1` and recomputation even when `L2` is unavailable.

The original issue text referred to an `Option<RedisPool>` fallback. The current codebase uses the equivalent runtime pattern in `L2Cache::conn()`, which returns `None` when Redis is unreachable and lets the cache cascade continue.

## Symptoms

Look for one or more of these signals:

- `GET /ready` shows `redis: "down"`
- `GET /ready` shows `overall_status: "not_ready"`
- admin cache status shows `redis_available: false`
- latency rises because requests lose `L2` cache hits
- logs contain one or more of:
  - `Redis connection unavailable: ...`
  - `Redis SET failed for ...`
  - `Redis DEL failed for ...`
  - `Redis bulk DEL failed: ...`
  - `Redis MGET failed: ...`
  - `Redis pipeline store failed: ...`

If alerting is wired, treat this as the operational equivalent of a `NoesisRedisConnectionFailure` alert.

## Diagnosis

1. Confirm readiness degradation.
   - `curl -s http://<noesis-host>/ready | jq`
   - verify `redis` is `down`
2. Confirm admin surface degradation.
   - inspect the cache/admin status payload for `redis_available: false`
3. Check Redis directly.
   - local or Docker: `redis-cli -u "$REDIS_URL" ping`
   - Railway: inspect the Redis add-on status in the Railway dashboard and verify the bound `REDIS_URL`
4. Inspect application logs for the Redis warning patterns above.
5. Rule out config drift.
   - verify `REDIS_URL` is present and points at the expected instance
   - run `cargo run -p noesis-api --bin validate_config -- --dry-run` locally if env parity is in doubt

## Mitigation

1. Keep the API serving in degraded mode.
   - do not bounce the API first unless the process is already unhealthy
   - the current cache design tolerates `L2` failure and continues through `L1` / recomputation
2. Restore Redis reachability.
   - restart the Redis service if self-hosted
   - restart or recover the Railway Redis add-on if managed
   - fix host/network/auth configuration if the URL or credentials changed
3. If Redis is still unavailable but the API must stay online:
   - continue running in `L1`-only degraded mode
   - expect lower hit rate and higher latency
   - communicate that cache sharing across instances is temporarily unavailable

## Verify L1-Only Mode Is Operational

Use these checks before deciding the incident is contained:

1. `GET /ready` still returns a JSON response and the API continues answering calculation requests.
2. New calculation requests succeed even while `redis` remains `down`.
3. Logs show Redis warnings but no cascading application crash.
4. Repeated same-node requests remain serviceable because `L1` continues operating.

This is the practical validation that the graceful degradation path is active.

## Recovery Verification

After Redis is restored:

1. `redis-cli -u "$REDIS_URL" ping` returns `PONG`
2. `GET /ready` reports:
   - `redis: "ok"`
   - `overall_status: "ready"`
3. admin cache status reports `redis_available: true`
4. logs stop emitting new Redis connection warnings
5. repeat a representative API calculation and confirm normal latency / cache behavior

If Redis is healthy but readiness does not recover, restart the API instance to force a clean reconnection path.

## Escalation

- Platform / infra owner:
  Redis service down, Railway add-on unhealthy, network/auth drift
- API owner:
  readiness remains degraded after Redis recovery
- Release owner:
  incident overlaps a deployment or blocks smoke checks / release gates

## Related References

- [docs/runbooks/redis-degradation.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/redis-degradation.md)
- [crates/noesis-cache/src/l2_cache.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-cache/src/l2_cache.rs)
- [crates/noesis-api/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/lib.rs)
- [crates/noesis-api/src/handlers/admin.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/handlers/admin.rs)
