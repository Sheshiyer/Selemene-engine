# Runbook: Noesis API Down (502/503 from Railway)

This runbook covers a production outage where the main Rust API is unreachable or returns `502` / `503`.

## Symptoms

- public API returns `502` or `503`
- Railway health checks fail
- `GET /health/live` does not return `200`
- `GET /ready` reports degraded or unavailable dependencies
- admin and auth routes fail along with engine/workflow routes

Treat this as the operational equivalent of a `NoesisAPIDown` alert.

## Primary Checks

1. Check liveness:
   - `curl -sS -o /tmp/noesis-live.json -w "%{http_code}" "$NOESIS_URL/health/live"`
2. Check readiness:
   - `curl -sS "$NOESIS_URL/ready" | jq`
3. Check Railway service status and logs:
   - `railway link -p "$RAILWAY_PROJECT_ID" -e "${RAILWAY_ENVIRONMENT:-production}" -s "${RAILWAY_SERVICE:-Selemene-engine}"`
   - `railway status`
   - `railway logs`
4. Run config sanity audit if env drift is suspected:
   - `cargo run -p noesis-api --bin validate_config -- --dry-run`

## Railway CLI Procedure

The current deploy pipeline in [/.github/workflows/deploy.yaml](/Volumes/madara/2026/witnessos/Selemene-engine/.github/workflows/deploy.yaml) already uses these commands:

```bash
railway link -p "$RAILWAY_PROJECT_ID" -e "${RAILWAY_ENVIRONMENT:-production}" -s "${RAILWAY_SERVICE:-Selemene-engine}"
railway status
railway logs
railway up --ci --detach
```

Use them in this order during an incident:

1. `railway status`
   - confirm the service URL and current deploy health
2. `railway logs`
   - inspect startup failures, OOMs, bind errors, missing env vars
3. `curl "$NOESIS_URL/health/live"`
   - verify the container is actually serving
4. `curl "$NOESIS_URL/ready"`
   - determine whether the API is up but degraded on DB / Redis / bridge

## Root Cause Checklist

Check these first because they map directly to current runtime behavior:

- missing or malformed env vars
  - `JWT_SECRET`
  - `DATABASE_URL`
  - `REDIS_URL`
  - `TS_ENGINES_URL`
- database connection timeout during boot
- Redis degradation causing readiness failure
- TS sidecar unavailable causing bridge degradation
- OOM / crash loop during deploy

Relevant code and docs:

- [crates/noesis-api/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/lib.rs)
- [crates/noesis-api/src/bin/validate_config.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/bin/validate_config.rs)
- [docs/deployment/RAILWAY.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/deployment/RAILWAY.md)

## Mitigation

1. If the service is crash-looping after a deploy:
   - inspect `railway logs`
   - roll back to the previous healthy deploy if the new release is clearly bad
2. If `/health/live` is healthy but `/ready` is failing:
   - treat this as dependency degradation, not total API loss
   - follow the dependency-specific runbook:
     - Redis: [incident-redis-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-redis-failure.md)
     - TS bridge: [incident-ts-bridge-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-ts-bridge-failure.md)
     - DB pool: [incident-db-pool-exhaustion.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-db-pool-exhaustion.md)
3. If env drift is suspected:
   - compare Railway variables against [docs/baseline/env-parity.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/baseline/env-parity.md)
4. If the API is completely unavailable and logs indicate bad startup:
   - restart or redeploy only after identifying the failing layer

## Grafana / Monitoring References

Use the monitoring references in [docs/deployment/monitoring.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/deployment/monitoring.md):

- `Noesis Overview`
- `Engine Performance`
- `Workflow Metrics`

Most relevant signals during API-down incidents:

- request rate
- request latency p95
- engine calculation time
- cache hit rate

## Recovery Verification

1. `GET /health/live` returns `200`
2. `GET /ready` reports healthy or known degraded dependency state
3. one engine route succeeds
4. one workflow route succeeds
5. Railway logs stop showing new startup failures

## Escalation Contacts

Use role-based escalation:

- platform / deploy owner
- API owner
- release owner if the outage overlaps a deployment window

