# Redis Degradation Runbook

This runbook covers L2 Redis degradation for the Selemene cache stack (`L1 in-memory -> L2 Redis -> L3 disk`).

For the operator-facing incident procedure requested by the release-readiness backlog, see [incident-redis-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-redis-failure.md).

## Trigger Threshold

Escalate when either condition is met:

1. More than **5 consecutive L2 Redis failures** are observed in a rolling 5 minute window.
2. The readiness surface reports Redis as unavailable for **3 consecutive probes**.

## Detection Signals

### Readiness and admin surfaces

- `GET /ready` returns JSON with `redis: "down"` and `overall_status: "not_ready"`
- Admin cache status exposes `redis_available: false`

### Log patterns

These are the concrete messages emitted by the L2 cache path today:

- `Redis connection unavailable: ...`
- `Redis SET failed for <key>: ...`
- `Redis DEL failed for <key>: ...`
- `Redis bulk DEL failed: ...`
- `Redis MGET failed: ...`
- `Redis pipeline store failed: ...`

The cache layer degrades gracefully:

- reads return `None` from L2 and continue to L3 / recomputation
- writes return `Ok(())` after warning and continue without blocking the request path

## Impact

- cache hit ratio drops because Redis is bypassed
- horizontally scaled nodes stop sharing cached results
- latency rises because calculations fall back to L1-only or recomputation
- readiness can remain degraded even if API routes still serve traffic

## Immediate Response

1. Check `GET /ready` and confirm the `redis` field is `down`.
2. Inspect recent application logs for the L2 failure patterns above.
3. Verify Redis process / add-on health:
   - local or Docker: `redis-cli ping`
   - Railway / hosted: inspect add-on status and connection URL
4. Confirm `REDIS_URL` is still present in the runtime environment.

## Recovery Procedure

1. Restore Redis reachability.
   - restart the Redis service or fix the managed add-on
   - validate network policy / service discovery if the host changed
2. Re-run the readiness check until:
   - `redis` becomes `ok`
   - `overall_status` becomes `ready`
3. Watch logs for a clean period with no new L2 warnings.
4. If degradation persists after Redis is back, recycle the API instance so it re-establishes connections cleanly.

## Escalation Path

1. Platform / infra owner if Redis is unreachable or the URL changed
2. API owner if readiness remains degraded after Redis recovers
3. Release owner if degradation overlaps a deployment or blocks smoke checks

## Notes

- The current runtime is intentionally tolerant of Redis outages; the system should keep serving requests with reduced cache effectiveness.
- This runbook documents the current tracing and readiness fields rather than introducing new alerting primitives.
