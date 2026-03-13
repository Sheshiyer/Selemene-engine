# Canary Rollout Policy

This policy defines the staged rollout path for production releases and the
conditions for promotion, hold, or rollback.

It is grounded in the current observability and deploy surfaces:

- Prometheus metrics in [docs/deployment/monitoring.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/deployment/monitoring.md)
- Railway deploy and health checks in [/.github/workflows/deploy.yaml](/Volumes/madara/2026/witnessos/Selemene-engine/.github/workflows/deploy.yaml)
- API health/runbook procedures in [incident-api-down.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-api-down.md)
- TS bridge degradation procedures in [incident-ts-bridge-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-ts-bridge-failure.md)

## Traffic Stages

Use this exact progression unless an incident commander freezes the rollout:

1. 5%
2. 25%
3. 50%
4. 100%

Each stage must meet the promotion gates below before advancing.

## Promotion Gates

Advance to the next stage only when all gates are green for the full
observation window.

### Observation Window

- Minimum hold at each stage: 15 minutes
- Preferred hold for high-risk releases: 30 minutes

### Required Gates

- Error rate below 1%
  - Derived from `noesis_requests_total`
  - Query:
    ```promql
    rate(noesis_requests_total{status="error"}[5m])
    /
    rate(noesis_requests_total[5m])
    < 0.01
    ```
- Request p95 latency below 2 seconds
  - Metric: `noesis_request_duration_seconds`
  - Query:
    ```promql
    histogram_quantile(0.95, rate(noesis_request_duration_seconds_bucket[5m])) < 2
    ```
- No active Sentry critical incidents for the release window
  - Check the active release or environment in Sentry
  - Confirm no new critical or crash-looping errors tied to the deploy
- Health endpoints remain green
  - `/health/live` returns `200`
  - `/ready` returns healthy or intentionally degraded-only states that were already accepted before rollout
- No smoke-test regression
  - Post-deploy smoke checks and operator spot checks stay green

## Rollback Triggers

Rollback immediately if any of these occur:

- Error rate reaches or exceeds 1% for 5 minutes
- Request p95 reaches or exceeds 2 seconds for 5 minutes
- `/health/live` fails or `/ready` indicates an unhealthy API or dependency state not seen before the rollout
- Sentry reports new critical release errors
- TS bridge is unavailable and the release materially impacts TS-engine flows
- DB/Redis degradation appears only after the release and persists after one quick retry/restart decision point

## Manual Override Procedure

Manual override is allowed only when all of the following are true:

- the incident commander and release owner agree the signal is noisy or non-user-impacting
- the affected metrics are understood and documented in the release thread
- rollback risk is higher than holding the canary in place

If override is used:

1. Record the reason in the release channel or release issue.
2. Note the exact metric/query that was overridden.
3. Extend the observation window by at least 15 minutes.
4. Re-check Sentry, health, and smoke results before any further promotion.

## Escalation Path

Escalate in this order:

1. On-call engineer
2. Platform/deploy owner
3. Release owner
4. Domain owner if the failure is engine-specific

Escalate immediately instead of waiting out the observation window when:

- the API is unavailable
- readiness breaks across multiple subsystems
- rollback is likely required

## Operator Procedure

1. Deploy the release through the normal deployment path.
2. Confirm the release is serving traffic and health checks are passing.
3. At each stage, check:
   - Prometheus/Grafana for error rate and p95
   - Sentry for new critical errors
   - `/health/live` and `/ready`
   - smoke or spot-check results for the changed surface
4. Promote only when all gates are green for the full hold window.
5. Roll back immediately on any rollback trigger.

## Reference Queries

### Error Rate

```promql
rate(noesis_requests_total{status="error"}[5m])
/
rate(noesis_requests_total[5m])
```

### Request p95

```promql
histogram_quantile(0.95, rate(noesis_request_duration_seconds_bucket[5m]))
```

### Workflow p95

```promql
histogram_quantile(0.95, rate(noesis_workflow_duration_seconds_bucket[5m]))
```

### TS Bridge p95

```promql
histogram_quantile(0.95, rate(noesis_ts_bridge_duration_seconds_bucket[5m]))
```
