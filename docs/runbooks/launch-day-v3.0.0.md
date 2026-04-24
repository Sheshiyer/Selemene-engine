# v3.0.0 Launch-Day Runbook

This runbook is the operator-facing sequence for a coordinated Selemene v3.0.0
release day.

It is built from the current repo-visible release, smoke, monitoring, and
rollback surfaces. It is **not** a substitute for a staging rollback drill log.

## Scope

Use this runbook when coordinating a release that touches:

- the main Rust API on Railway
- the TypeScript sidecar
- the admin web on Vercel
- release notes / launch docs

Supporting references:

- [docs/release/release-checklist-template.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/release/release-checklist-template.md)
- [docs/monitoring/README.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/monitoring/README.md)
- [docs/runbooks/README.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/README.md)
- [docs/drills/rollback-drill-plan.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/drills/rollback-drill-plan.md)

## Operator Inputs

Fill these in before release start:

- Railway project / environment:
  - `<RAILWAY_PROJECT_ID>`
  - `<RAILWAY_ENVIRONMENT>`
  - `<RAILWAY_SERVICE>`
- API URL:
  - `<API_BASE_URL>`
- Admin web URL:
  - `<ADMIN_WEB_URL>`
- Release owner:
  - `<name>`
- On-call / escalation contact:
  - `<name + channel>`
- Last known-good Railway deployment or rollback target:
  - `<deployment-id-or-release-ref>`

## T-30 Minutes

1. Confirm release freeze:
   - release notes updated
   - docs updated
   - must-ship issues closed or explicitly deferred
2. Confirm the pre-release command set is available:

```bash
railway status
bash scripts/railway-health-check.sh "${API_BASE_URL}/health/live"
SMOKE_TEST_JWT="$SMOKE_TEST_JWT" bash scripts/smoke-test-runner.sh "$API_BASE_URL"
```

3. Open the operational references:
   - [docs/monitoring/README.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/monitoring/README.md)
   - [docs/runbooks/incident-api-down.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-api-down.md)
   - [docs/runbooks/incident-ts-bridge-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-ts-bridge-failure.md)
   - [docs/runbooks/incident-db-pool-exhaustion.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-db-pool-exhaustion.md)
   - [docs/runbooks/incident-redis-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-redis-failure.md)

## Launch Sequence

### 1. Verify Railway State

```bash
railway link -p "$RAILWAY_PROJECT_ID" -e "$RAILWAY_ENVIRONMENT" -s "$RAILWAY_SERVICE"
railway status
railway logs
```

If the Railway service is already unhealthy before deployment, stop and resolve
that first.

### 2. Deploy the API Revision

Use the existing deployment path for the release candidate. After the deploy
starts, verify cold-start recovery and health:

```bash
bash scripts/railway-health-check.sh "${API_BASE_URL}/health/live"
curl -sS "${API_BASE_URL}/ready" | jq
```

### 3. Smoke the API

```bash
SMOKE_TEST_JWT="$SMOKE_TEST_JWT" bash scripts/smoke-test-runner.sh "$API_BASE_URL"
```

Expected minimum outcome:

- `/health/live` returns `200`
- `/ready` shows acceptable dependency state
- protected engine/workflow smoke checks pass

### 4. Verify Admin Web

```bash
ADMIN_WEB_URL="$ADMIN_WEB_URL" \
API_BASE_URL="$API_BASE_URL" \
bash scripts/smoke_admin_web.sh
```

### 5. DNS / Domain Cutover

If the release involves domain or traffic cutover, record the exact change here:

- DNS record changed by: `<owner>`
- timestamp: `<time>`
- old target: `<previous>`
- new target: `<new>`

If no DNS change is part of this release, explicitly mark:

- `No DNS cutover required for this launch`

### 6. CDN Warm / Cache Prime

The repo does not currently ship a dedicated CDN warming tool. The current
repo-visible warm path is to hit the public health and representative workload
surfaces immediately after deploy:

```bash
curl -sS "${API_BASE_URL}/health/live" >/dev/null
curl -sS "${API_BASE_URL}/health/ready" >/dev/null
curl -sS -H "X-API-Key: ${SMOKE_TEST_API_KEY}" "${API_BASE_URL}/api/v1/engines" >/dev/null
SMOKE_TEST_JWT="$SMOKE_TEST_JWT" bash scripts/smoke-test-runner.sh "$API_BASE_URL" >/dev/null
```

This warms the app path and representative engine/workflow routes. If your
deployment includes an external CDN layer, record the CDN-specific warm step as
an operator override.

## Monitoring Dashboard Links

Repo-visible dashboard/config sources:

- [docs/monitoring/README.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/monitoring/README.md)
- [monitoring/grafana/dashboards/selemene-engine.json](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/dashboards/selemene-engine.json)
- [monitoring/grafana/provisioning/dashboards/api-overview.json](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/provisioning/dashboards/api-overview.json)
- [monitoring/grafana/provisioning/dashboards/engine-performance.json](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/provisioning/dashboards/engine-performance.json)

Fill in the live links used on launch day:

- Grafana overview: `<grafana-url>`
- Grafana engine performance: `<grafana-url>`
- Prometheus: `<prometheus-url>`
- Jaeger: `<jaeger-url>`
- Sentry project: `<sentry-url>`

## Escalation Contacts

Fill in before release:

- Release owner: `<name>`
- Platform / Railway owner: `<name>`
- Admin-web owner: `<name>`
- On-call escalation channel: `<channel>`
- Executive/status update channel: `<channel>`

## Rollback Procedure

### Trigger Conditions

Rollback immediately if any of these occur after deployment:

- `bash scripts/railway-health-check.sh` fails after the retry window
- `bash scripts/smoke-test-runner.sh` returns failure
- admin web smoke fails in a release-blocking way
- critical incident follows the API/DB/Redis/TS bridge runbooks

### Railway Rollback

1. Identify the last known-good release or deployment.
2. Use the Railway dashboard or CLI rollback path for that deployment.
3. Re-run:

```bash
bash scripts/railway-health-check.sh "${API_BASE_URL}/health/live"
curl -sS "${API_BASE_URL}/ready" | jq
SMOKE_TEST_JWT="$SMOKE_TEST_JWT" bash scripts/smoke-test-runner.sh "$API_BASE_URL"
```

### Database Migration Revert

If the release included a schema change, follow the migration authority rules:

- [docs/deployment/DB_MIGRATION_AUTHORITY.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/deployment/DB_MIGRATION_AUTHORITY.md)

You must already know the exact revert step before release start. Record it here:

- migration forward step: `<id>`
- migration revert step: `<command or manual SQL>`

### Admin-Web Rollback

If the admin web is the failing surface:

- use the Vercel project rollback path described in [docs/deployment/VERCEL_ADMIN_WEB.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/deployment/VERCEL_ADMIN_WEB.md)
- re-run `bash scripts/smoke_admin_web.sh`

## Post-Launch Verification

Complete these after the deploy settles:

- [ ] Railway health check passes
- [ ] smoke runner passes
- [ ] admin web smoke passes
- [ ] monitoring dashboards reachable
- [ ] no critical incident runbook is active
- [ ] release owner posts status update

## Limitations

This runbook is useful now, but it does **not** prove the full issue
acceptance yet.

Specifically still unproven from repo-only evidence:

- staging rollback dry-run completed in under 5 minutes
- live dashboard URLs resolved in the actual deployed environment

Do not close the launch-day runbook issue until those two checks are completed
or the issue acceptance is narrowed.
