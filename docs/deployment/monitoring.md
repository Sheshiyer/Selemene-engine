# Monitoring Setup Guide

## Overview

Selemene Engine supports comprehensive observability through:
- **Prometheus** - Metrics collection
- **Grafana** - Dashboards and visualization
- **Jaeger** - Distributed tracing
- **Loki** - Log aggregation

## Quick Start

### Docker Compose

```bash
# Start with monitoring stack
docker-compose -f docker-compose.yml -f docker-compose.monitoring.yml up -d

# Access dashboards
# Grafana: http://localhost:3000 (admin/admin)
# Prometheus: http://localhost:9090
# Jaeger: http://localhost:16686
```

---

## Prometheus Metrics

### Available Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `noesis_requests_total` | Counter | Total API requests |
| `noesis_request_duration_seconds` | Histogram | Request latency |
| `noesis_engine_calculation_duration_seconds` | Histogram | Engine calculation time |
| `noesis_engine_calculations_total` | Counter | Total calculations by engine |
| `noesis_cache_hits_total` | Counter | Cache hits by layer |
| `noesis_cache_misses_total` | Counter | Cache misses by layer |
| `noesis_active_connections` | Gauge | Current active connections |
| `noesis_workflow_duration_seconds` | Histogram | Workflow execution time |
| `noesis_ts_bridge_duration_seconds` | Histogram | TS engine bridge latency |

### Prometheus Configuration

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'noesis-api'
    static_configs:
      - targets: ['noesis-api:8080']
    metrics_path: /metrics

  - job_name: 'ts-engines'
    static_configs:
      - targets: ['ts-engines:3001']
    metrics_path: /metrics

  - job_name: 'redis'
    static_configs:
      - targets: ['redis:6379']

rule_files:
  - /etc/prometheus/rules/*.yml

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']
```

### Alerting Rules

```yaml
# monitoring/prometheus/rules/noesis.yml
groups:
  - name: noesis
    rules:
      - alert: HighErrorRate
        expr: rate(noesis_requests_total{status="error"}[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High error rate detected
          description: Error rate is {{ $value }} errors/sec

      - alert: SlowCalculations
        expr: histogram_quantile(0.95, rate(noesis_engine_calculation_duration_seconds_bucket[5m])) > 1
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: Engine calculations are slow

      - alert: LowCacheHitRate
        expr: rate(noesis_cache_hits_total[5m]) / (rate(noesis_cache_hits_total[5m]) + rate(noesis_cache_misses_total[5m])) < 0.5
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: Cache hit rate below 50%

      - alert: TSEngineBridgeDown
        expr: up{job="ts-engines"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: TypeScript engine bridge is down
```

---

## Grafana Dashboards

### Default Dashboards

1. **Noesis Overview** - Request rates, latencies, error rates
2. **Engine Performance** - Per-engine calculation times
3. **Cache Analytics** - Hit rates, memory usage
4. **Workflow Metrics** - Workflow execution stats

### Dashboard JSON

```json
{
  "dashboard": {
    "title": "Noesis Engine Overview",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(noesis_requests_total[5m])",
            "legendFormat": "{{method}} {{path}}"
          }
        ]
      },
      {
        "title": "Request Latency (p95)",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(noesis_request_duration_seconds_bucket[5m]))",
            "legendFormat": "p95"
          }
        ]
      },
      {
        "title": "Cache Hit Rate",
        "type": "gauge",
        "targets": [
          {
            "expr": "rate(noesis_cache_hits_total[5m]) / (rate(noesis_cache_hits_total[5m]) + rate(noesis_cache_misses_total[5m]))"
          }
        ]
      },
      {
        "title": "Engine Calculation Time",
        "type": "heatmap",
        "targets": [
          {
            "expr": "rate(noesis_engine_calculation_duration_seconds_bucket[5m])",
            "legendFormat": "{{engine_id}}"
          }
        ]
      }
    ]
  }
}
```

---

## Jaeger Tracing

### Trace Configuration

```bash
# Environment variables for tracing
OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
OTEL_SERVICE_NAME=noesis-api
OTEL_TRACES_SAMPLER=parentbased_traceidratio
OTEL_TRACES_SAMPLER_ARG=0.1  # Sample 10% of traces
```

### Traced Operations

- HTTP request handling
- Engine calculations
- Cache operations
- TypeScript bridge calls
- Database queries
- Workflow orchestration

### Trace Attributes

| Attribute | Description |
|-----------|-------------|
| `engine.id` | Engine identifier |
| `engine.calculation_type` | Type of calculation |
| `cache.hit` | Whether cache was hit |
| `cache.layer` | Cache layer (L1/L2/L3) |
| `workflow.id` | Workflow identifier |
| `workflow.engines` | Engines in workflow |

---

## Loki Logging

### Log Configuration

```bash
# Environment variables
LOG_FORMAT=json
RUST_LOG=info,tower_http=debug
```

### Log Query Examples

```logql
# All errors
{app="noesis-api"} |= "error"

# Slow calculations (>100ms)
{app="noesis-api"} | json | calculation_time_ms > 100

# Specific engine logs
{app="noesis-api"} |= "engine_id" | json | engine_id="human-design"

# Rate of errors
rate({app="noesis-api"} |= "error" [5m])
```

### Structured Log Fields

```json
{
  "timestamp": "2025-01-15T12:00:00Z",
  "level": "INFO",
  "target": "noesis_api::handlers",
  "message": "Engine calculation completed",
  "engine_id": "human-design",
  "calculation_time_ms": 45.2,
  "cached": false,
  "trace_id": "abc123",
  "span_id": "def456"
}
```

---

## Docker Compose Monitoring Stack

```yaml
# docker-compose.monitoring.yml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:v2.45.0
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - ./monitoring/prometheus/rules:/etc/prometheus/rules
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.enable-lifecycle'

  grafana:
    image: grafana/grafana:10.0.0
    ports:
      - "3000:3000"
    volumes:
      - ./monitoring/grafana/provisioning:/etc/grafana/provisioning
      - ./monitoring/grafana/dashboards:/var/lib/grafana/dashboards
      - grafana-data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false

  jaeger:
    image: jaegertracing/all-in-one:1.47
    ports:
      - "16686:16686"  # UI
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP
    environment:
      - COLLECTOR_OTLP_ENABLED=true

  loki:
    image: grafana/loki:2.8.0
    ports:
      - "3100:3100"
    volumes:
      - ./monitoring/loki/config.yml:/etc/loki/config.yml
      - loki-data:/loki
    command: -config.file=/etc/loki/config.yml

  promtail:
    image: grafana/promtail:2.8.0
    volumes:
      - ./monitoring/promtail/config.yml:/etc/promtail/config.yml
      - /var/log:/var/log
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
    command: -config.file=/etc/promtail/config.yml

  alertmanager:
    image: prom/alertmanager:v0.25.0
    ports:
      - "9093:9093"
    volumes:
      - ./monitoring/alertmanager/config.yml:/etc/alertmanager/config.yml

volumes:
  prometheus-data:
  grafana-data:
  loki-data:
```

---

## Alertmanager Configuration

```yaml
# monitoring/alertmanager/config.yml
global:
  resolve_timeout: 5m

route:
  group_by: ['alertname', 'severity']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'default'
  routes:
    - match:
        severity: critical
      receiver: 'critical'

receivers:
  - name: 'default'
    email_configs:
      - to: 'team@example.com'
        from: 'alertmanager@example.com'
        smarthost: 'smtp.example.com:587'
        
  - name: 'critical'
    email_configs:
      - to: 'oncall@example.com'
    pagerduty_configs:
      - service_key: '<pagerduty-key>'
```

---

## Key Performance Indicators (KPIs)

| KPI | Target | Alert Threshold |
|-----|--------|-----------------|
| Request Latency (p95) | <200ms | >500ms |
| Request Latency (p99) | <500ms | >1s |
| Error Rate | <0.1% | >1% |
| Cache Hit Rate | >80% | <50% |
| Engine Calculation Time | <100ms | >500ms |
| Workflow Time (14 engines) | <2s | >5s |
| Availability | 99.9% | <99% |

---

## Troubleshooting Monitoring

### Prometheus Not Scraping

```bash
# Check targets
curl http://localhost:9090/api/v1/targets

# Verify metrics endpoint
curl http://localhost:8080/metrics
```

### Grafana Dashboard Not Loading

```bash
# Check Grafana logs
docker-compose logs grafana

# Verify data source
curl http://localhost:3000/api/datasources
```

### Missing Traces

```bash
# Check Jaeger connectivity
curl http://localhost:16686/api/services

# Verify OTLP endpoint
curl http://localhost:4318/v1/traces
```

---

**Last Updated**: 2026-01
