// tests/load/scenario1-steady.js
// Scenario 1: Steady Load - 100 concurrent virtual users for 5 minutes
//
// Purpose: Validate API handles sustained traffic with acceptable latency.
// Target: p95 < 500ms, error rate < 1%

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { BASE_URL, JWT_TOKEN, authHeaders, birthData, engines, randomChoice } from './helpers.js';

// Custom metrics
const errorRate = new Rate('errors');
const engineLatency = new Trend('engine_latency', true);
const successCount = new Counter('successful_requests');
const failCount = new Counter('failed_requests');

export const options = {
  scenarios: {
    steady_load: {
      executor: 'constant-vus',
      vus: 100,
      duration: '5m',
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],
    'errors': ['rate<0.01'],
    'engine_latency': ['p(95)<500'],
  },
};

export default function () {
  // Rotate through engines to distribute load
  const engineId = randomChoice(engines);

  const response = http.post(
    `${BASE_URL}/api/v1/engines/${engineId}/calculate`,
    JSON.stringify(birthData),
    { headers: authHeaders() }
  );

  const isSuccess = response.status === 200;

  check(response, {
    'status is 200': (r) => r.status === 200,
    'has response body': (r) => r.body && r.body.length > 0,
    'response is JSON': (r) => {
      try { JSON.parse(r.body); return true; } catch (e) { return false; }
    },
    'has witness_prompt': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.witness_prompt !== undefined;
      } catch (e) { return false; }
    },
  });

  errorRate.add(!isSuccess);
  engineLatency.add(response.timings.duration);

  if (isSuccess) {
    successCount.add(1);
  } else {
    failCount.add(1);
  }

  // Think time: 1 second between requests (simulates real user behavior)
  sleep(1);
}

export function handleSummary(data) {
  const summary = {
    scenario: 'steady_load',
    vus: 100,
    duration: '5m',
    total_requests: data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 0,
    rps: data.metrics.http_reqs ? data.metrics.http_reqs.values.rate : 0,
    p50_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(50)'] : 0,
    p95_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(95)'] : 0,
    p99_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(99)'] : 0,
    error_rate: data.metrics.errors ? data.metrics.errors.values.rate : 0,
    thresholds_passed: !Object.values(data.root_group.checks || {}).some(c => c.fails > 0),
  };

  return {
    'stdout': JSON.stringify(summary, null, 2) + '\n',
    'tests/load/results/scenario1-steady.json': JSON.stringify(data, null, 2),
  };
}
