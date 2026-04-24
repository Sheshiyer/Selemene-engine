# Runbook Index

This is the quick-reference index for the current incident runbooks.

## Alert Map

| Alert / Incident | Runbook | Severity | Expected Resolution |
|---|---|---:|---:|
| `NoesisAPIDown` / public `502` or `503` | [incident-api-down.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-api-down.md) | High | 15-30 min |
| `NoesisRedisConnectionFailure` / cache degradation | [incident-redis-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-redis-failure.md) | Medium | 15-30 min |
| TS bridge timeout / sidecar degradation | [incident-ts-bridge-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-ts-bridge-failure.md) | High | 15-30 min |
| `NoesisPostgresConnectionsHigh` / DB pool exhaustion | [incident-db-pool-exhaustion.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-db-pool-exhaustion.md) | High | 15-30 min |
| Workflow partial results / workflow timeout symptoms | [incident-workflow-timeout.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-workflow-timeout.md) | Medium | 15-30 min |

## On-Call Quick Reference

### Railway

```bash
railway link -p "$RAILWAY_PROJECT_ID" -e "${RAILWAY_ENVIRONMENT:-production}" -s "${RAILWAY_SERVICE:-Selemene-engine}"
railway status
railway logs
```

### API Health

```bash
curl -sS "$NOESIS_URL/health/live"
curl -sS "$NOESIS_URL/ready" | jq
```

### Redis

```bash
redis-cli -u "$REDIS_URL" ping
```

### Postgres

```bash
psql "$DATABASE_URL" -c 'select 1'
psql "$DATABASE_URL" -c 'select state, count(*) from pg_stat_activity where datname = current_database() group by state order by count(*) desc;'
```

### TS Sidecar

```bash
curl -sS "$TS_ENGINES_URL/health"
curl -sS "$TS_ENGINES_URL/health/ready"
```

## Notes

- This index intentionally covers the five current failure-mode runbooks.
- Some individual issues may stay open if their acceptance criteria require richer observability or control-plane behavior than the current codebase exposes.

## Release Policy

- Canary rollout policy: [canary-rollout-policy.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/canary-rollout-policy.md)
- Launch-day operations: [launch-day-v3.0.0.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/launch-day-v3.0.0.md)

## Resilience Drills

- Rollback drill plan: [../drills/rollback-drill-plan.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/drills/rollback-drill-plan.md)
