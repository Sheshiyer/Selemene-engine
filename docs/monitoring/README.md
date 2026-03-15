# Monitoring and Alerting Reference

This document maps the Selemene monitoring stack to the actual repo-visible
configuration files and clarifies what is implemented versus what is only
configured.

## Stack Map

| Component | Purpose | Primary config | Notes |
| --- | --- | --- | --- |
| Prometheus | Scrapes API and infra metrics, evaluates alert rules | [monitoring/prometheus.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus.yml) | Stable API scrape target is active; canary scrape target is documented but commented out by default. |
| Alert rules | API, cache, database, and canary divergence alerts | [monitoring/prometheus/alerts/noesis-alerts.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus/alerts/noesis-alerts.yml) | Canary divergence rule coverage is tested locally. |
| Alertmanager | Groups, inhibits, and routes alerts to receivers | [monitoring/alertmanager/alertmanager.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/alertmanager/alertmanager.yml) | Webhook receiver URLs are configured, but downstream webhook handlers are not repo-visible here. |
| Grafana datasources | Provisions Prometheus, Loki, and Jaeger datasources | [monitoring/grafana/provisioning/datasources/datasources.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/provisioning/datasources/datasources.yml) | Legacy datasource example also exists in `monitoring/grafana/datasources/prometheus.yml`. |
| Grafana dashboards | Dashboards for API, cache, engine performance, and release/canary views | [monitoring/grafana/provisioning/dashboards/dashboard.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/provisioning/dashboards/dashboard.yml) | Includes `api-overview`, `cache-efficiency`, `engine-performance`, and `selemene-engine`. |
| Sentry | Error capture, breadcrumbs, and trace/tag correlation for API failures | [crates/noesis-api/src/main.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/main.rs) and [crates/noesis-api/src/error_mapper.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/error_mapper.rs) | Active only when `SENTRY_DSN` is set. |
| Jaeger | OTLP-compatible distributed tracing backend | [monitoring/jaeger/jaeger-config.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/jaeger/jaeger-config.yml) | Repo has Jaeger config and OTLP tracing support, but trace export must be enabled by deployment/runtime wiring. |
| Loki / Promtail | Structured log aggregation with `trace_id` label extraction | [monitoring/loki/loki-config.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/loki/loki-config.yml) and [monitoring/loki/promtail-config.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/loki/promtail-config.yml) | Promtail expects structured JSON logs and extracts `trace_id` for log/trace correlation. |

## Alert Flow

1. The API exposes liveness/readiness and Prometheus metrics on:
   - `GET /health/live`
   - `GET /health/ready`
   - `GET /ready`
   - `GET /metrics`
2. Prometheus scrapes the configured targets from [monitoring/prometheus.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus.yml).
3. Prometheus evaluates alert expressions from [monitoring/prometheus/alerts/noesis-alerts.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus/alerts/noesis-alerts.yml).
4. Alertmanager groups and routes those alerts using [monitoring/alertmanager/alertmanager.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/alertmanager/alertmanager.yml).
5. Grafana reads the same Prometheus data and overlays release/canary annotations from the dashboard JSON files.

### Current Limitation

Alertmanager receivers point at:

- `http://noesis-api:8080/webhooks/alerts`
- `http://noesis-api:8080/webhooks/alerts/critical`
- `http://noesis-api:8080/webhooks/alerts/warning`
- `http://noesis-api:8080/webhooks/alerts/infrastructure`
- `http://noesis-api:8080/webhooks/alerts/database`

Those receiver URLs are configured, but this repository does not currently show
repo-visible handler implementations for that webhook leg. Treat the
detection-and-routing path as configured, and the final notification delivery
path as unverified unless your deployment proves those endpoints.

## Prometheus

Primary scrape and rule config:

- [monitoring/prometheus.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus.yml)
- [monitoring/prometheus/alerts/noesis-alerts.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus/alerts/noesis-alerts.yml)
- [monitoring/prometheus/tests/noesis-canary-alerts.test.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus/tests/noesis-canary-alerts.test.yml)

Key targets currently defined:

- `noesis-api`
- `noesis-api-health`
- `postgres`
- `redis`
- `node-exporter`
- `prometheus`
- `jaeger`
- `loki`

The `noesis-api-canary` target is documented inline but intentionally commented
out in the default config until a real canary service exists.

## Grafana

Provisioned dashboards:

- [monitoring/grafana/provisioning/dashboards/api-overview.json](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/provisioning/dashboards/api-overview.json)
- [monitoring/grafana/provisioning/dashboards/cache-efficiency.json](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/provisioning/dashboards/cache-efficiency.json)
- [monitoring/grafana/provisioning/dashboards/engine-performance.json](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/provisioning/dashboards/engine-performance.json)
- [monitoring/grafana/dashboards/selemene-engine.json](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/dashboards/selemene-engine.json)

Datasource provisioning:

- [monitoring/grafana/provisioning/datasources/datasources.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/provisioning/datasources/datasources.yml)

## Sentry

Sentry initialization and enrichment live in:

- [crates/noesis-api/src/main.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/main.rs)
- [crates/noesis-api/src/error_mapper.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/error_mapper.rs)

What is repo-visible:

- API startup initializes Sentry when `SENTRY_DSN` is present.
- Structured API errors attach `trace_id` and tags to Sentry events.
- 5xx errors are captured as Sentry events; 4xx errors record breadcrumbs.

What is not guaranteed by repo-only review:

- live DSN wiring in production
- alerting from Sentry to external channels

## Jaeger and Loki

Jaeger config:

- [monitoring/jaeger/jaeger-config.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/jaeger/jaeger-config.yml)

Loki / Promtail config:

- [monitoring/loki/loki-config.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/loki/loki-config.yml)
- [monitoring/loki/promtail-config.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/loki/promtail-config.yml)

Runtime support:

- [crates/noesis-metrics/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-metrics/src/lib.rs) includes OTLP tracing support compatible with Jaeger.
- [crates/noesis-api/src/logging.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/logging.rs) contains the logging/tracing hooks for structured observability.

Important distinction:

- Jaeger, Loki, and Promtail configs are present and Grafana datasources are provisioned.
- The API entrypoint currently always initializes logging and Sentry.
- OTLP trace export is supportable from the codebase, but only active when your deployment enables the tracing path.

## Runbooks

Current incident runbooks:

- [docs/runbooks/README.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/README.md)
- [docs/runbooks/incident-api-down.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-api-down.md)
- [docs/runbooks/incident-redis-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-redis-failure.md)
- [docs/runbooks/incident-ts-bridge-failure.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-ts-bridge-failure.md)
- [docs/runbooks/incident-db-pool-exhaustion.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-db-pool-exhaustion.md)
- [docs/runbooks/incident-workflow-timeout.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-workflow-timeout.md)

## Quick Validation

Use these checks to prove the documented surfaces still exist:

```bash
# API health and metrics
curl -sS "$NOESIS_URL/health/live"
curl -sS "$NOESIS_URL/ready" | jq
curl -sS "$NOESIS_URL/metrics" | head

# Alert rule validation
bash scripts/test_canary_alerts.sh

# Grafana dashboard JSON validity
jq empty monitoring/grafana/dashboards/selemene-engine.json
jq empty monitoring/grafana/provisioning/dashboards/api-overview.json

# Confirm webhook receiver URLs exist in config
rg -n "webhooks/alerts" monitoring/alertmanager/alertmanager.yml
```

If any of those checks drift, update this document before using it as a release
readiness artifact.
