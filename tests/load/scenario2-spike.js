// tests/load/scenario2-spike.js
// Scenario 2: Spike Test - sudden traffic surge from 10 to 200 users
//
// Purpose: Validate API handles traffic spikes without degradation.
// Target: p95 < 1000ms during spike, graceful recovery

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { BASE_URL, authHeaders, birthData, engines, randomChoice } from './helpers.js';

const errorRate = new Rate('errors');
const engineLatency = new Trend('engine_latency', true);
const spikeLatency = new Trend('spike_phase_latency', true);

export const options = {
  scenarios: {
    spike: {
      executor: 'ramping-vus',
      startVUs: 10,
      stages: [
        { duration: '30s', target: 10 },    // Baseline: warm up
        { duration: '10s', target: 200 },   // Spike: ramp to 200 users
        { duration: '1m', target: 200 },    // Hold: sustained spike
        { duration: '10s', target: 10 },    // Recovery: ramp down
        { duration: '30s', target: 10 },    // Stabilize: verify recovery
      ],
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<1000'],   // More lenient during spike
    'errors': ['rate<0.05'],                // Allow up to 5% errors during spike
    'engine_latency': ['p(95)<1000'],
  },
};

export default function () {
  const engineId = randomChoice(engines);

  const response = http.post(
    `${BASE_URL}/api/v1/engines/${engineId}/calculate`,
    JSON.stringify(birthData),
    { headers: authHeaders() }
  );

  const isSuccess = response.status === 200;

  check(response, {
    'status is 200 or 429 (rate limited)': (r) => r.status === 200 || r.status === 429,
    'response has body': (r) => r.body && r.body.length > 0,
  });

  // Only count non-429 as errors (rate limiting is expected during spike)
  errorRate.add(response.status !== 200 && response.status !== 429);
  engineLatency.add(response.timings.duration);

  // Track spike phase latency separately
  if (__ITER > 0) {
    spikeLatency.add(response.timings.duration);
  }

  sleep(0.5); // Faster think time during spike test
}

export function handleSummary(data) {
  const summary = {
    scenario: 'spike_test',
    max_vus: 200,
    total_requests: data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 0,
    max_rps: data.metrics.http_reqs ? data.metrics.http_reqs.values.rate : 0,
    p50_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(50)'] : 0,
    p95_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(95)'] : 0,
    p99_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(99)'] : 0,
    error_rate: data.metrics.errors ? data.metrics.errors.values.rate : 0,
  };

  return {
    'stdout': JSON.stringify(summary, null, 2) + '\n',
    'tests/load/results/scenario2-spike.json': JSON.stringify(data, null, 2),
  };
}
