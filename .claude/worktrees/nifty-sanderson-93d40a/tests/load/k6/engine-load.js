// tests/load/k6/engine-load.js
// W2-S8-03: Load test individual engines
//
// Target: 1000 concurrent users, <1s p95 latency
// Run: k6 run tests/load/k6/engine-load.js

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import exec from 'k6/execution';

// Configuration
const BASE_URL = __ENV.API_URL || 'http://localhost:8080';
const JWT_TOKEN = __ENV.JWT_TOKEN || '';

// Custom metrics
const errorRate = new Rate('errors');
const engineLatency = new Trend('engine_latency_ms', true);
const engineSuccess = new Counter('engine_success');
const engineFailure = new Counter('engine_failure');

// Per-engine metrics
const panchangaLatency = new Trend('panchanga_latency_ms', true);
const numerologyLatency = new Trend('numerology_latency_ms', true);
const biorhythmLatency = new Trend('biorhythm_latency_ms', true);
const humanDesignLatency = new Trend('human_design_latency_ms', true);
const geneKeysLatency = new Trend('gene_keys_latency_ms', true);
const vimshottariLatency = new Trend('vimshottari_latency_ms', true);

// Test scenarios - ramp up to 1000 concurrent users
export const options = {
  scenarios: {
    // Scenario 1: Gradual ramp-up
    ramp_up: {
      executor: 'ramping-vus',
      startVUs: 10,
      stages: [
        { duration: '30s', target: 100 },   // Warm up
        { duration: '1m', target: 500 },    // Scale to 500
        { duration: '2m', target: 1000 },   // Target load
        { duration: '3m', target: 1000 },   // Sustain
        { duration: '1m', target: 0 },      // Ramp down
      ],
      gracefulRampDown: '30s',
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<1000', 'p(99)<2000'],
    'errors': ['rate<0.05'],
    'engine_latency_ms': ['p(95)<1000'],
    'panchanga_latency_ms': ['p(95)<500'],
    'numerology_latency_ms': ['p(95)<300'],
    'biorhythm_latency_ms': ['p(95)<200'],
    'human_design_latency_ms': ['p(95)<1500'],
    'gene_keys_latency_ms': ['p(95)<1000'],
    'vimshottari_latency_ms': ['p(95)<800'],
  },
};

// Rust engines
const RUST_ENGINES = ['panchanga', 'numerology', 'biorhythm', 'human-design', 'gene-keys', 'vimshottari'];

// Sample birth data variants for cache testing
const birthDataVariants = [
  { name: 'NYC 1990', date: '1990-01-15', time: '14:30', lat: 40.7128, lon: -74.006, tz: 'America/New_York' },
  { name: 'London 1985', date: '1985-06-20', time: '09:15', lat: 51.5074, lon: -0.1278, tz: 'Europe/London' },
  { name: 'Tokyo 1995', date: '1995-12-03', time: '18:45', lat: 35.6762, lon: 139.6503, tz: 'Asia/Tokyo' },
  { name: 'Sydney 1988', date: '1988-03-21', time: '06:00', lat: -33.8688, lon: 151.2093, tz: 'Australia/Sydney' },
  { name: 'Mumbai 1992', date: '1992-07-10', time: '22:30', lat: 19.076, lon: 72.8777, tz: 'Asia/Kolkata' },
  { name: 'Berlin 1979', date: '1979-11-25', time: '11:45', lat: 52.52, lon: 13.405, tz: 'Europe/Berlin' },
  { name: 'LA 1998', date: '1998-04-30', time: '20:15', lat: 34.0522, lon: -118.2437, tz: 'America/Los_Angeles' },
  { name: 'Delhi 1983', date: '1983-08-17', time: '07:45', lat: 28.6139, lon: 77.209, tz: 'Asia/Kolkata' },
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
    options: {},
  });
}

function recordEngineMetric(engine, duration, success) {
  engineLatency.add(duration);
  
  if (success) {
    engineSuccess.add(1);
  } else {
    engineFailure.add(1);
  }
  
  // Per-engine metrics
  switch (engine) {
    case 'panchanga':
      panchangaLatency.add(duration);
      break;
    case 'numerology':
      numerologyLatency.add(duration);
      break;
    case 'biorhythm':
      biorhythmLatency.add(duration);
      break;
    case 'human-design':
      humanDesignLatency.add(duration);
      break;
    case 'gene-keys':
      geneKeysLatency.add(duration);
      break;
    case 'vimshottari':
      vimshottariLatency.add(duration);
      break;
  }
}

export default function () {
  // Randomly select engine and birth data for distribution
  const engine = randomChoice(RUST_ENGINES);
  const sample = randomChoice(birthDataVariants);
  const payload = buildInput(sample);

  group(`Engine: ${engine}`, function () {
    const response = http.post(
      `${BASE_URL}/api/v1/engines/${engine}/calculate`,
      payload,
      { headers: authHeaders() }
    );

    const success = response.status === 200;
    
    check(response, {
      'status is 200': (r) => r.status === 200,
      'response has body': (r) => r.body && r.body.length > 0,
      'response is valid JSON': (r) => {
        try { JSON.parse(r.body); return true; }
        catch (e) { return false; }
      },
      'has witness_prompt': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.witness_prompt !== undefined;
        } catch (e) { return false; }
      },
      'latency under 1s': (r) => r.timings.duration < 1000,
    });

    errorRate.add(!success);
    recordEngineMetric(engine, response.timings.duration, success);
  });

  // Think time: randomized 0.5-1.5s
  sleep(0.5 + Math.random());
}

export function handleSummary(data) {
  const summary = {
    test: 'engine-load',
    timestamp: new Date().toISOString(),
    config: {
      target_vus: 1000,
      engines: RUST_ENGINES,
    },
    metrics: {
      total_requests: data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 0,
      rps: data.metrics.http_reqs ? data.metrics.http_reqs.values.rate : 0,
      p50_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(50)'] : 0,
      p95_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(95)'] : 0,
      p99_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(99)'] : 0,
      error_rate: data.metrics.errors ? data.metrics.errors.values.rate : 0,
    },
    engine_metrics: {
      panchanga_p95: data.metrics.panchanga_latency_ms ? data.metrics.panchanga_latency_ms.values['p(95)'] : null,
      numerology_p95: data.metrics.numerology_latency_ms ? data.metrics.numerology_latency_ms.values['p(95)'] : null,
      biorhythm_p95: data.metrics.biorhythm_latency_ms ? data.metrics.biorhythm_latency_ms.values['p(95)'] : null,
      human_design_p95: data.metrics.human_design_latency_ms ? data.metrics.human_design_latency_ms.values['p(95)'] : null,
      gene_keys_p95: data.metrics.gene_keys_latency_ms ? data.metrics.gene_keys_latency_ms.values['p(95)'] : null,
      vimshottari_p95: data.metrics.vimshottari_latency_ms ? data.metrics.vimshottari_latency_ms.values['p(95)'] : null,
    },
    thresholds_passed: Object.entries(data.thresholds || {})
      .every(([_, v]) => v.ok),
  };

  return {
    'stdout': JSON.stringify(summary, null, 2) + '\n',
    'tests/load/k6/results/engine-load-results.json': JSON.stringify(data, null, 2),
  };
}
