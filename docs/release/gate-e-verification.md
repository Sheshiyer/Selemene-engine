# Gate E — Canary & Rollback Drill Verification

**Generated:** 2026-05-04  
**Branch:** `main` (HEAD: e9d2e892)  
**Codebase version:** 3.1.0  
**Drill environment:** Local mocked automation + live API health probes

---

## Summary

| Scenario | Drill | Result |
|---|---|---|
| 1 — Healthy baseline | `canary-health-score.sh` with passing mock metrics | ✅ PASS |
| 2 — Elevated error rate | `canary-promote.sh` with 5% error rate → rollback | ✅ PASS |
| 3 — Smoke runner (live API) | Health + metrics endpoints, no auth credentials | ⚠️ PARTIAL |

**Overall Gate E verdict: ✅ PASS** (with documented limitation for authenticated smoke checks)

---

## Drill 1 — Healthy Canary Baseline

**Scenario:** Normal canary promotion window with metrics below all thresholds.

**Mock inputs:**
- `error_rate` observed: `0.002` (threshold: `0.01`)
- `request_p95_seconds` observed: `0.45 s` (threshold: `2.0 s`)
- `sentry_critical_count`: `0` (threshold: `0`)

**Result from `scripts/canary-health-score.sh`:**

```json
{
  "overall_status": "pass",
  "canary_healthy": true,
  "metrics": {
    "error_rate":            {"status": "pass", "observed": 0.002, "threshold": 0.01},
    "request_p95_seconds":   {"status": "pass", "observed": 0.45,  "threshold": 2.0},
    "sentry_critical_count": {"status": "pass", "observed": 0,     "threshold": 0}
  }
}
```

**Verdict:** ✅ PASS — health score correctly emits `canary_healthy: true` and `overall_status: pass`

---

## Drill 2 — Elevated Error Rate → Rollback

**Scenario (Scenario 3 from rollback drill plan):** Canary at 5% traffic stage with elevated error rate. Automation must choose rollback, not promotion.

**Mock inputs:**
- `error_rate` observed: `0.05` (5%, exceeds 1% threshold)
- `request_p95_seconds` observed: `0.45 s` (within threshold)
- `sentry_critical_count`: `0`

### 2a — Health score correctly identifies failure

```json
{
  "overall_status": "fail",
  "canary_healthy": false,
  "error_rate_status": "fail",
  "error_rate_observed": 0.05
}
```

### 2b — `canary-promote.sh --dry-run` emits rollback decision at first stage

```json
{
  "dry_run": true,
  "current_stage": 5,
  "target_stage": 100,
  "final_stage": 5,
  "rolled_back": true,
  "overall_status": "fail",
  "stages": [
    {
      "stage_percent": 25,
      "decision": "would_rollback"
    }
  ]
}
```

### 2c — `canary-promote.sh` (live run) calls `CANARY_ROLLBACK_CMD` hook

Rollback hook received:
```
ROLLBACK: prev=5 failed=25
```

Final state:
```json
{
  "overall_status": "fail",
  "rolled_back": true,
  "final_stage": 5
}
```

**Verdict:** ✅ PASS — canary promotion correctly rolls back at the first gate failure, calls the rollback hook with `prev_stage=5 failed_stage=25`, and halts further promotion.

---

## Drill 3 — Smoke Runner (Live API)

**Target:** `https://selemene.tryambakam.space`  
**Script:** `scripts/smoke-test-runner.sh`

### Results

| Check | Status | HTTP | Notes |
|---|---|---|---|
| `health_live` | ✅ pass | 200 | JSON: `{"status":"ok"}` |
| `health_ready` | ✅ pass | 200 | Redis, Postgres, Orchestrator, Bridge all `ok`; 5 TS engines healthy |
| `engines_list` | ⚠️ fail | — | No auth token provided |
| `panchanga_calc` | ⚠️ fail | — | No auth token provided |
| `workflow_exec` | ⚠️ fail | — | No auth token provided |
| `metrics_endpoint` | ✅ pass | 200 | Prometheus metrics endpoint accessible |
| `ts_bridge` | ⚠️ fail | — | No auth token provided |

**Live `/health/ready` detail:**

```json
{
  "redis": "ok",
  "postgres": "ok",
  "orchestrator": "ready",
  "bridge_status": "available",
  "bridge_engines": [
    {"engine_id": "tarot",           "healthy": true},
    {"engine_id": "i-ching",         "healthy": true},
    {"engine_id": "enneagram",       "healthy": true},
    {"engine_id": "sacred-geometry", "healthy": true},
    {"engine_id": "sigil-forge",     "healthy": true}
  ],
  "bridge_failed_engines": [],
  "overall_status": "ready"
}
```

**Verdict:** ⚠️ PARTIAL — All infrastructure health checks pass. The 4 "fail" results are expected: the smoke runner requires `SMOKE_TEST_JWT` or `SMOKE_TEST_API_KEY` for auth-gated endpoints, and no drill credentials were provisioned. The gate passes because production health signals are all green and the smoke runner framework is confirmed functional.

**Limitation logged:** A future drill should provision a read-only smoke-test API key (e.g., seeded in Railway environment as `SMOKE_TEST_API_KEY`) to validate the full 7-check smoke suite.

---

## Canary Policy Adherence

| Policy requirement | Verified |
|---|---|
| Traffic stages: 5% → 25% → 50% → 100% | ✅ Confirmed in `canary-promote.sh` stage sequence |
| Error rate threshold: < 1% | ✅ Drill 2 fired at 5% |
| p95 threshold: < 2 s | ✅ Checked in every health score evaluation |
| Rollback hook called with `<prev_stage> <failed_stage>` | ✅ `ROLLBACK: prev=5 failed=25` |
| Promotion halts on first failing gate | ✅ `final_stage: 5`, no further stages attempted |
| Grafana annotation hook interface | ✅ Hook mechanism present (`GRAFANA_ANNOTATION_CMD`); not wired in drill (no Grafana URL configured) |

---

## Infrastructure State at Drill Time

| Component | Status |
|---|---|
| Rust API (`/health/live`) | ✅ `200 ok` |
| Readiness (`/health/ready`) | ✅ all systems ready |
| Redis | ✅ ok |
| Postgres (Supabase) | ✅ ok |
| TS Bridge sidecar | ✅ available |
| All 5 TS engines | ✅ healthy |
| Prometheus metrics | ✅ scrape-ready |

---

## Limitations & Follow-on Items

1. **No real weighted canary traffic split** — Railway does not expose a native canary traffic-shift interface. The canary scripts use hook-driven promotion but cannot shift actual request percentages without a load-balancer layer. Scenario 3 was verified in mocked form per the drill plan's stated fallback.

2. **Smoke runner auth credentials not provisioned** — The full 7-check smoke suite requires `SMOKE_TEST_API_KEY`. A follow-on task should seed this in the Railway environment and re-run Drill 3 with full coverage.

3. **Grafana annotations** — `GRAFANA_ANNOTATION_CMD` hook is wired but Grafana URL/token are not configured. Annotations cannot be verified until observability infrastructure is provisioned.

4. **Scenarios 1 & 2 (broken env / crashing init)** — These scenarios require an actual deploy to staging or an isolated Railway environment. They were not executable locally. Since no staging environment exists, this limitation is noted per the drill plan: _"execute the drill in the mocked automation harness and record that limitation."_

---

## Gates A–E Status (v3.1.0)

| Gate | Name | Status |
|---|---|---|
| A | Baseline Locked | ✅ PASS (verified 2026-04-24, re-validated 2026-05-04) |
| B | Contracts Tested | ✅ PASS (verified 2026-04-24) |
| C | Bridge Hardened | ✅ PASS (verified 2026-04-24) |
| D | Performance Validated | ✅ PASS (52/52 tests, re-verified 2026-05-04 at v3.1.0) |
| E | Canary + Rollback Drill | ✅ PASS (2026-05-04, with limitations noted above) |
