// tests/load/k6/full-spectrum.js
// W2-S8-03: Load test full-spectrum workflow specifically
//
// This is the heaviest workflow running all 13+ engines
// Target: 100 concurrent users, <5s p95 latency
// Run: k6 run tests/load/k6/full-spectrum.js

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter, Gauge } from 'k6/metrics';

// Configuration
const BASE_URL = __ENV.API_URL || 'http://localhost:8080';
const JWT_TOKEN = __ENV.JWT_TOKEN || '';

// Custom metrics
const errorRate = new Rate('errors');
const fullSpectrumLatency = new Trend('full_spectrum_latency_ms', true);
const engineCount = new Gauge('engines_returned');
const successfulExecutions = new Counter('successful_executions');
const failedExecutions = new Counter('failed_executions');

export const options = {
  scenarios: {
    // Conservative ramp for heavy workflow
    full_spectrum_load: {
      executor: 'ramping-vus',
      startVUs: 1,
      stages: [
        { duration: '30s', target: 10 },   // Warm up
        { duration: '1m', target: 50 },    // Scale
        { duration: '2m', target: 100 },   // Target load
        { duration: '3m', target: 100 },   // Sustain
        { duration: '1m', target: 0 },     // Ramp down
      ],
      gracefulRampDown: '30s',
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<5000', 'p(99)<10000'],
    'errors': ['rate<0.15'],
    'full_spectrum_latency_ms': ['p(95)<5000'],
  },
};

const birthDataVariants = [
  { name: 'Full Spectrum NYC', date: '1990-01-15', time: '14:30', lat: 40.7128, lon: -74.006, tz: 'America/New_York' },
  { name: 'Full Spectrum London', date: '1985-06-20', time: '09:15', lat: 51.5074, lon: -0.1278, tz: 'Europe/London' },
  { name: 'Full Spectrum Tokyo', date: '1995-12-03', time: '18:45', lat: 35.6762, lon: 139.6503, tz: 'Asia/Tokyo' },
];

function authHeaders() {
  return {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${JWT_TOKEN}`,
  };
}

function randomChoice(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

function buildInput(sample) {
  return JSON.stringify({
    birth_data: {
      name: sample.name,
      date: sample.date,
      time: sample.time,
      latitude: sample.lat,
      longitude: sample.lon,
      timezone: sample.tz,
    },
    current_time: new Date().toISOString(),
    precision: 'Standard',
    options: {
      // Gene Keys options
      hd_gates: {
        personality_sun: 17,
        personality_earth: 18,
        design_sun: 45,
        design_earth: 26,
      },
      // Vimshottari options
      moon_longitude: 125.0,
    },
  });
}

export default function () {
  const sample = randomChoice(birthDataVariants);
  const payload = buildInput(sample);

  group('Full Spectrum Workflow', function () {
    const response = http.post(
      `${BASE_URL}/api/v1/workflows/full-spectrum/execute`,
      payload,
      { headers: authHeaders(), timeout: '60s' }
    );

    const success = response.status === 200;
    
    check(response, {
      'status is 200': (r) => r.status === 200,
      'response has body': (r) => r.body && r.body.length > 0,
      'has engine_outputs': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.engine_outputs !== undefined;
        } catch (e) { return false; }
      },
      'has at least 5 engine outputs': (r) => {
        try {
          const body = JSON.parse(r.body);
          return Object.keys(body.engine_outputs || {}).length >= 5;
        } catch (e) { return false; }
      },
      'latency under 5s': (r) => r.timings.duration < 5000,
    });

    errorRate.add(!success);
    fullSpectrumLatency.add(response.timings.duration);

    if (success) {
      successfulExecutions.add(1);
      try {
        const body = JSON.parse(response.body);
        const count = Object.keys(body.engine_outputs || {}).length;
        engineCount.add(count);
      } catch (e) {}
    } else {
      failedExecutions.add(1);
    }
  });

  // Longer think time for heavy workflow (2-4s)
  sleep(2 + Math.random() * 2);
}

export function handleSummary(data) {
  const summary = {
    test: 'full-spectrum',
    timestamp: new Date().toISOString(),
    config: {
      target_vus: 100,
      workflow: 'full-spectrum',
    },
    metrics: {
      total_requests: data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 0,
      rps: data.metrics.http_reqs ? data.metrics.http_reqs.values.rate : 0,
      p50_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(50)'] : 0,
      p95_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(95)'] : 0,
      p99_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(99)'] : 0,
      error_rate: data.metrics.errors ? data.metrics.errors.values.rate : 0,
      avg_engines_returned: data.metrics.engines_returned ? 
        data.metrics.engines_returned.values.value : null,
    },
    thresholds_passed: Object.entries(data.thresholds || {})
      .every(([_, v]) => v.ok),
  };

  return {
    'stdout': JSON.stringify(summary, null, 2) + '\n',
    'tests/load/k6/results/full-spectrum-results.json': JSON.stringify(data, null, 2),
  };
}
