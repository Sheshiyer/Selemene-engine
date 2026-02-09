// tests/load/baseline.js
// MVP-21: Baseline performance test for the Selemene/Noesis API
//
// Establishes a performance baseline across four scenarios:
//   1. Health check     — 100 VUs, 30s, p95 < 100ms
//   2. Engine list      — 50 VUs,  30s, p95 < 200ms
//   3. Panchanga calc   — 20 VUs,  60s, p95 < 500ms
//   4. Workflow execute  — 10 VUs,  60s, p95 < 500ms
//
// Usage:
//   k6 run --env BASE_URL=http://localhost:8080 --env JWT_TOKEN=<token> tests/load/baseline.js
//   k6 run --env BASE_URL=https://selemene.railway.app --env JWT_TOKEN=<token> tests/load/baseline.js

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const JWT_TOKEN = __ENV.JWT_TOKEN || '';

// ---------------------------------------------------------------------------
// Custom metrics — one Trend per scenario for independent threshold tracking
// ---------------------------------------------------------------------------

const healthLatency = new Trend('health_latency_ms', true);
const enginesLatency = new Trend('engines_list_latency_ms', true);
const calculateLatency = new Trend('calculate_latency_ms', true);
const workflowLatency = new Trend('workflow_latency_ms', true);

const healthErrors = new Rate('health_errors');
const enginesErrors = new Rate('engines_errors');
const calculateErrors = new Rate('calculate_errors');
const workflowErrors = new Rate('workflow_errors');

const totalRequests = new Counter('total_requests');

// ---------------------------------------------------------------------------
// Scenarios & Thresholds
// ---------------------------------------------------------------------------

export const options = {
  scenarios: {
    // Scenario 1: Health check — lightweight, high concurrency
    health_check: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '5s', target: 50 },   // ramp up
        { duration: '20s', target: 100 },  // sustain
        { duration: '5s', target: 0 },     // ramp down
      ],
      exec: 'healthCheck',
      gracefulRampDown: '5s',
    },

    // Scenario 2: Engine list — moderate concurrency, auth required
    engine_list: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '5s', target: 25 },
        { duration: '20s', target: 50 },
        { duration: '5s', target: 0 },
      ],
      exec: 'engineList',
      startTime: '0s', // runs in parallel with health check
      gracefulRampDown: '5s',
    },

    // Scenario 3: Panchanga calculate — compute-heavy, lower concurrency
    calculate: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '10s', target: 10 },
        { duration: '40s', target: 20 },
        { duration: '10s', target: 0 },
      ],
      exec: 'calculatePanchanga',
      startTime: '35s', // start after health/engines finish ramping
      gracefulRampDown: '10s',
    },

    // Scenario 4: Workflow execution — heaviest operation, lowest concurrency
    workflow: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '10s', target: 5 },
        { duration: '40s', target: 10 },
        { duration: '10s', target: 0 },
      ],
      exec: 'workflowExecute',
      startTime: '35s', // parallel with calculate
      gracefulRampDown: '10s',
    },
  },

  thresholds: {
    // Per-scenario latency thresholds
    'health_latency_ms': ['p(95)<100'],
    'engines_list_latency_ms': ['p(95)<200'],
    'calculate_latency_ms': ['p(95)<500'],
    'workflow_latency_ms': ['p(95)<500'],

    // Per-scenario error rate thresholds (< 1%)
    'health_errors': ['rate<0.01'],
    'engines_errors': ['rate<0.01'],
    'calculate_errors': ['rate<0.01'],
    'workflow_errors': ['rate<0.01'],

    // Global HTTP thresholds
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],
  },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function headers() {
  return {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${JWT_TOKEN}`,
  };
}

// Diverse birth data to avoid cache-only measurements
const BIRTH_DATA_SAMPLES = [
  { name: 'Baseline NYC',    date: '1990-01-15', time: '14:30', latitude: 40.7128,  longitude: -74.006,   timezone: 'America/New_York' },
  { name: 'Baseline London', date: '1985-06-20', time: '09:15', latitude: 51.5074,  longitude: -0.1278,   timezone: 'Europe/London' },
  { name: 'Baseline Tokyo',  date: '1995-12-03', time: '18:45', latitude: 35.6762,  longitude: 139.6503,  timezone: 'Asia/Tokyo' },
  { name: 'Baseline Mumbai', date: '1992-07-10', time: '22:30', latitude: 19.076,   longitude: 72.8777,   timezone: 'Asia/Kolkata' },
  { name: 'Baseline Sydney', date: '1988-03-21', time: '06:00', latitude: -33.8688, longitude: 151.2093,  timezone: 'Australia/Sydney' },
  { name: 'Baseline Berlin', date: '1979-11-25', time: '11:45', latitude: 52.52,    longitude: 13.405,    timezone: 'Europe/Berlin' },
];

function randomSample() {
  return BIRTH_DATA_SAMPLES[Math.floor(Math.random() * BIRTH_DATA_SAMPLES.length)];
}

function buildEngineInput(sample) {
  return JSON.stringify({
    birth_data: {
      name: sample.name,
      date: sample.date,
      time: sample.time,
      latitude: sample.latitude,
      longitude: sample.longitude,
      timezone: sample.timezone,
    },
    current_time: new Date().toISOString(),
    precision: 'Standard',
    options: {},
  });
}

// ---------------------------------------------------------------------------
// Scenario functions
// ---------------------------------------------------------------------------

// Scenario 1: Health check
export function healthCheck() {
  const res = http.get(`${BASE_URL}/health`);

  const ok = check(res, {
    'health: status is 200': (r) => r.status === 200,
    'health: body has status ok': (r) => {
      try { return JSON.parse(r.body).status === 'ok'; }
      catch (_) { return false; }
    },
  });

  healthLatency.add(res.timings.duration);
  healthErrors.add(!ok);
  totalRequests.add(1);

  sleep(0.1); // 100ms think time for health checks
}

// Scenario 2: Engine list
export function engineList() {
  const res = http.get(`${BASE_URL}/api/v1/engines`, { headers: headers() });

  const ok = check(res, {
    'engines: status is 200': (r) => r.status === 200,
    'engines: returns array': (r) => {
      try { return Array.isArray(JSON.parse(r.body).engines); }
      catch (_) { return false; }
    },
    'engines: has at least 1 engine': (r) => {
      try { return JSON.parse(r.body).engines.length > 0; }
      catch (_) { return false; }
    },
  });

  enginesLatency.add(res.timings.duration);
  enginesErrors.add(!ok);
  totalRequests.add(1);

  sleep(0.5); // 500ms think time
}

// Scenario 3: Panchanga calculate
export function calculatePanchanga() {
  const sample = randomSample();
  const payload = buildEngineInput(sample);

  const res = http.post(
    `${BASE_URL}/api/v1/engines/panchanga/calculate`,
    payload,
    { headers: headers(), timeout: '10s' }
  );

  const ok = check(res, {
    'calculate: status is 200': (r) => r.status === 200,
    'calculate: has engine_id': (r) => {
      try { return JSON.parse(r.body).engine_id === 'panchanga'; }
      catch (_) { return false; }
    },
    'calculate: has result': (r) => {
      try { return JSON.parse(r.body).result !== undefined; }
      catch (_) { return false; }
    },
    'calculate: has witness_prompt': (r) => {
      try { return typeof JSON.parse(r.body).witness_prompt === 'string'; }
      catch (_) { return false; }
    },
  });

  calculateLatency.add(res.timings.duration);
  calculateErrors.add(!ok);
  totalRequests.add(1);

  sleep(1 + Math.random()); // 1-2s think time for calculations
}

// Scenario 4: Workflow execution (birth-blueprint)
export function workflowExecute() {
  const sample = randomSample();
  const payload = buildEngineInput(sample);

  const res = http.post(
    `${BASE_URL}/api/v1/workflows/birth-blueprint/execute`,
    payload,
    { headers: headers(), timeout: '15s' }
  );

  const ok = check(res, {
    'workflow: status is 200': (r) => r.status === 200,
    'workflow: has engine_outputs': (r) => {
      try { return JSON.parse(r.body).engine_outputs !== undefined; }
      catch (_) { return false; }
    },
    'workflow: has total_time_ms': (r) => {
      try { return typeof JSON.parse(r.body).total_time_ms === 'number'; }
      catch (_) { return false; }
    },
  });

  workflowLatency.add(res.timings.duration);
  workflowErrors.add(!ok);
  totalRequests.add(1);

  sleep(2 + Math.random()); // 2-3s think time for workflows
}

// ---------------------------------------------------------------------------
// Summary handler — structured JSON output for CI / comparison
// ---------------------------------------------------------------------------

export function handleSummary(data) {
  const scenarioMetric = (name) => {
    const m = data.metrics[name];
    if (!m) return null;
    return {
      min: m.values.min,
      avg: m.values.avg,
      med: m.values.med,
      p90: m.values['p(90)'],
      p95: m.values['p(95)'],
      p99: m.values['p(99)'],
      max: m.values.max,
    };
  };

  const errorMetric = (name) => {
    const m = data.metrics[name];
    if (!m) return null;
    return m.values.rate;
  };

  const summary = {
    test: 'baseline',
    timestamp: new Date().toISOString(),
    base_url: BASE_URL,
    scenarios: {
      health_check: {
        latency: scenarioMetric('health_latency_ms'),
        error_rate: errorMetric('health_errors'),
      },
      engine_list: {
        latency: scenarioMetric('engines_list_latency_ms'),
        error_rate: errorMetric('engines_errors'),
      },
      calculate: {
        latency: scenarioMetric('calculate_latency_ms'),
        error_rate: errorMetric('calculate_errors'),
      },
      workflow: {
        latency: scenarioMetric('workflow_latency_ms'),
        error_rate: errorMetric('workflow_errors'),
      },
    },
    global: {
      total_requests: data.metrics.total_requests
        ? data.metrics.total_requests.values.count
        : 0,
      http_req_duration: scenarioMetric('http_req_duration'),
    },
    thresholds_passed: Object.entries(data.thresholds || {}).every(
      ([_, v]) => v.ok
    ),
  };

  return {
    stdout: JSON.stringify(summary, null, 2) + '\n',
    'tests/load/results/baseline-results.json': JSON.stringify(data, null, 2),
  };
}
