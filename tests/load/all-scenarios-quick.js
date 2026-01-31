// tests/load/all-scenarios-quick.js
// Quick combined load test - runs all scenarios in reduced duration
// Total runtime: ~5 minutes

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { BASE_URL, authHeaders, birthData, engines, workflows, sampleBirthData, buildInput, randomChoice } from './helpers.js';

// Custom metrics
const errorRate = new Rate('errors');
const engineLatency = new Trend('engine_latency', true);
const workflowLatency = new Trend('workflow_latency', true);
const rateLimitCount = new Counter('rate_limited');

export const options = {
  scenarios: {
    // Scenario 1: Steady load (60s, 100 VUs)
    steady_load: {
      executor: 'constant-vus',
      vus: 100,
      duration: '60s',
      exec: 'steadyLoad',
      startTime: '0s',
    },
    // Scenario 2: Spike test (10 -> 200 -> 10)
    spike_test: {
      executor: 'ramping-vus',
      startVUs: 10,
      stages: [
        { duration: '10s', target: 10 },
        { duration: '5s', target: 200 },
        { duration: '30s', target: 200 },
        { duration: '5s', target: 10 },
        { duration: '10s', target: 10 },
      ],
      exec: 'spikeTest',
      startTime: '65s',
    },
    // Scenario 3: Workflow load (30s, 50 VUs)
    workflow_load: {
      executor: 'constant-vus',
      vus: 50,
      duration: '30s',
      exec: 'workflowLoad',
      startTime: '130s',
    },
    // Scenario 4: Cache test (30s, 50 VUs)
    cache_test: {
      executor: 'constant-vus',
      vus: 50,
      duration: '30s',
      exec: 'cacheTest',
      startTime: '165s',
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<500'],
    'errors': ['rate<0.05'],
    'engine_latency': ['p(95)<500'],
    'workflow_latency': ['p(95)<2000'],
  },
};

// Scenario 1: Steady load across all engines
export function steadyLoad() {
  const engineId = randomChoice(engines);
  const response = http.post(
    `${BASE_URL}/api/v1/engines/${engineId}/calculate`,
    JSON.stringify(birthData),
    { headers: authHeaders() }
  );

  const ok = response.status === 200;
  check(response, {
    'steady: 200': (r) => r.status === 200,
    'steady: has body': (r) => r.body && r.body.length > 2,
  });
  errorRate.add(!ok);
  engineLatency.add(response.timings.duration);
  sleep(1);
}

// Scenario 2: Spike test
export function spikeTest() {
  const engineId = randomChoice(engines);
  const response = http.post(
    `${BASE_URL}/api/v1/engines/${engineId}/calculate`,
    JSON.stringify(birthData),
    { headers: authHeaders() }
  );

  const ok = response.status === 200 || response.status === 429;
  check(response, {
    'spike: 200 or 429': (r) => r.status === 200 || r.status === 429,
  });
  errorRate.add(response.status !== 200 && response.status !== 429);
  engineLatency.add(response.timings.duration);
  if (response.status === 429) rateLimitCount.add(1);
  sleep(0.5);
}

// Scenario 3: Workflow load
export function workflowLoad() {
  const workflowId = randomChoice(workflows);
  const response = http.post(
    `${BASE_URL}/api/v1/workflows/${workflowId}/execute`,
    JSON.stringify(birthData),
    { headers: authHeaders() }
  );

  check(response, {
    'workflow: 200': (r) => r.status === 200,
    'workflow: has engine_outputs': (r) => {
      try { return 'engine_outputs' in JSON.parse(r.body); } catch (e) { return false; }
    },
  });
  errorRate.add(response.status !== 200);
  workflowLatency.add(response.timings.duration);
  sleep(2);
}

// Scenario 4: Cache test (same inputs repeated)
export function cacheTest() {
  const sample = randomChoice(sampleBirthData);
  const input = buildInput(sample);
  const response = http.post(
    `${BASE_URL}/api/v1/engines/panchanga/calculate`,
    JSON.stringify(input),
    { headers: authHeaders() }
  );

  check(response, {
    'cache: 200': (r) => r.status === 200,
    'cache: fast (<200ms)': (r) => r.timings.duration < 200,
  });
  errorRate.add(response.status !== 200);
  engineLatency.add(response.timings.duration);
  sleep(0.5);
}

export function handleSummary(data) {
  const m = data.metrics;
  const summary = {
    test_date: new Date().toISOString(),
    total_requests: m.http_reqs ? m.http_reqs.values.count : 0,
    overall_rps: m.http_reqs ? m.http_reqs.values.rate : 0,
    http_req_duration: {
      p50: m.http_req_duration ? m.http_req_duration.values['p(50)'] : 0,
      p95: m.http_req_duration ? m.http_req_duration.values['p(95)'] : 0,
      p99: m.http_req_duration ? m.http_req_duration.values['p(99)'] : 0,
      avg: m.http_req_duration ? m.http_req_duration.values.avg : 0,
      max: m.http_req_duration ? m.http_req_duration.values.max : 0,
    },
    engine_latency: {
      p50: m.engine_latency ? m.engine_latency.values['p(50)'] : 0,
      p95: m.engine_latency ? m.engine_latency.values['p(95)'] : 0,
      p99: m.engine_latency ? m.engine_latency.values['p(99)'] : 0,
    },
    workflow_latency: {
      p50: m.workflow_latency ? m.workflow_latency.values['p(50)'] : 0,
      p95: m.workflow_latency ? m.workflow_latency.values['p(95)'] : 0,
      p99: m.workflow_latency ? m.workflow_latency.values['p(99)'] : 0,
    },
    error_rate: m.errors ? m.errors.values.rate : 0,
    rate_limited: m.rate_limited ? m.rate_limited.values.count : 0,
    thresholds: Object.fromEntries(
      Object.entries(data.thresholds || {}).map(([k, v]) => [k, v.thresholds])
    ),
  };

  return {
    'stdout': JSON.stringify(summary, null, 2) + '\n',
    '/Volumes/madara/2026/witnessos/Selemene-engine/tests/load/results/all-scenarios-quick.json': JSON.stringify(data, null, 2),
  };
}
