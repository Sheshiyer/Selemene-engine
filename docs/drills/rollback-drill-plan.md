# Rollback Drill Plan

This document defines the first rollback drill set for release-readiness Wave 2.

It is a design and rehearsal artifact, not an execution log. Use it to prepare
staging or isolated drill runs before attempting any production-grade exercise.

## Scope

The initial drill wave covers three scenarios:

1. deploy with a broken environment variable
2. deploy with crashing init code
3. canary with elevated error rate

Each drill must capture:

- setup steps
- expected automated response
- rollback trigger
- target detection time
- manual verification checklist
- success criteria

## Prerequisites

- run drills in staging or an explicitly isolated environment
- keep a known-good image tag / deployment revision ready for rollback
- have access to:
  - deploy workflow logs in [/.github/workflows/deploy.yaml](/Volumes/madara/2026/witnessos/Selemene-engine/.github/workflows/deploy.yaml)
  - smoke runner in [scripts/smoke-test-runner.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/smoke-test-runner.sh)
  - canary helpers in [scripts/canary-health-score.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/canary-health-score.sh) and [scripts/canary-promote.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/canary-promote.sh)
  - the relevant runbooks in [docs/runbooks/README.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/README.md)

## Scenario Matrix

| Scenario | Primary Signal | Target Detection Time | Rollback Trigger | Owner |
|---|---|---:|---|---|
| Broken env var deploy | readiness / smoke failure | <= 3 min | `/health/live` stays unhealthy or smoke runner fails | On-call + Release Owner |
| Crashing init code | rollout never stabilizes | <= 3 min | pod/service crash loop or Railway health never passes | On-call + Platform Owner |
| Canary elevated error rate | canary error divergence | <= 5 min | canary health score fails or divergence alert fires | On-call + Release Owner |

## Scenario 1: Broken Environment Variable Deploy

### Goal

Prove that a deploy with invalid runtime configuration is detected quickly and
rolled back before it becomes a prolonged outage.

### Setup

1. Deploy to staging with one intentionally bad required variable:
   - preferred: invalid `DATABASE_URL`
   - acceptable alternative: invalid `JWT_SECRET` or `REDIS_URL`
2. Record the expected healthy revision before the drill starts.
3. Start a timer when the bad revision is deployed.

### Expected Automated Response

- the deployment workflow health verification fails:
  - Railway path: `Verify Railway health` fails after repeated `/health/live` checks
  - Kubernetes path: rollout status does not complete or readiness remains red
- the API smoke runner returns `overall_status=fail`
- the deploy workflow stops progressing to a healthy release outcome

### Rollback Trigger

Trigger rollback when either condition is met:

- `/health/live` is still not `200` after the deploy health retry window
- or the smoke runner reports failure for the deployed revision

### Manual Verification Checklist

- [ ] confirm the broken variable value was the only intentional fault
- [ ] confirm the failing revision is the one currently serving
- [ ] capture deploy logs and the first failing health/smoke result
- [ ] execute rollback to the last known-good revision
- [ ] re-run `/health/live`, `/ready`, and the smoke runner

### Success Criteria

- failure is detected within 3 minutes
- rollback is initiated without ambiguity about root cause
- the previous healthy revision serves traffic again
- post-rollback smoke checks return `overall_status=pass`

## Scenario 2: Crashing Init Code

### Goal

Prove that a release which crashes before the API becomes healthy is detected
and rolled back quickly.

### Setup

1. Create a drill build that exits during startup before normal request serving:
   - startup panic
   - explicit `exit(1)`
   - failing mandatory init path
2. Deploy it to staging or an isolated drill environment.
3. Start timing at deployment start.

### Expected Automated Response

- rollout fails to stabilize:
  - Kubernetes rollout does not become ready
  - or Railway health checks never pass
- `/health/live` never returns `200`
- the smoke runner cannot complete successfully against the target URL

### Rollback Trigger

Trigger rollback when:

- rollout status remains unhealthy past the deploy timeout window
- or health verification never reaches a passing state

### Manual Verification Checklist

- [ ] capture the startup failure log excerpt
- [ ] confirm no application endpoint became healthy during the drill window
- [ ] execute rollback to the previous healthy revision
- [ ] verify pod/service recovery or Railway health recovery
- [ ] re-run smoke checks after rollback

### Success Criteria

- the crash loop is identified within 3 minutes
- rollback is initiated using the pre-recorded healthy revision
- the restored revision passes `/health/live`, `/ready`, and smoke checks

## Scenario 3: Canary with Elevated Error Rate

### Goal

Prove that a canary release with materially worse error behavior is identified
and stopped before full promotion.

### Setup

1. Prepare a canary-only fault injection that increases request or engine error
   rate without affecting the stable path.
2. Run the drill against the canary automation surface:
   - alert rule in [monitoring/prometheus/alerts/noesis-alerts.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus/alerts/noesis-alerts.yml)
   - promotion logic in [scripts/canary-promote.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/canary-promote.sh)
3. If a real weighted canary environment is not available yet, execute the drill
   in the mocked automation harness and record that limitation.

### Expected Automated Response

- `NoesisCanaryErrorDivergence` becomes eligible to fire when canary error rate
  materially diverges from stable
- `scripts/canary-health-score.sh` reports unhealthy output for the canary
- `scripts/canary-promote.sh` chooses rollback instead of promotion
- Grafana receives a rollback annotation when live hooks are configured

### Rollback Trigger

Trigger rollback when either condition is met:

- the canary health score fails at a promotion gate
- or the canary divergence alert crosses its threshold

### Manual Verification Checklist

- [ ] capture the canary health score output
- [ ] capture the alert or mocked promtool result
- [ ] confirm promotion stopped before full traffic rollout
- [ ] verify rollback hook or rollback decision was emitted
- [ ] verify stable service remained healthy throughout the drill

### Success Criteria

- elevated canary error rate is detected within 5 minutes
- automation chooses rollback rather than further promotion
- rollback decision is visible in logs and, when configured, as a Grafana annotation
- no full-rollout promotion occurs after the failure signal

## Approval Checklist

Before closing the design issue for this plan:

- [ ] Tech Lead reviews the scenarios
- [ ] SRE / on-call owner reviews the automation expectations
- [ ] dependencies and current limitations are explicitly noted
- [ ] follow-on execution issues are linked before scheduling the first drill

## Current Limitations

- real weighted canary traffic is still blocked by the unresolved canary infra
  work, so Scenario 3 may need to begin in mocked or pre-prod form first
- this document does not replace the execution logs that later drill issues will
  need to produce
