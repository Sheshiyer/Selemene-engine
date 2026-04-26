// tests/load/scenario4-cache.js
// Scenario 4: Cache Performance - test cache hit/miss latency patterns
//
// Purpose: Validate caching reduces latency for repeated calculations.
// Target: Repeated requests < 50ms (cache hits), fresh requests < 200ms

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { BASE_URL, authHeaders, sampleBirthData, buildInput, randomChoice } from './helpers.js';

const errorRate = new Rate('errors');
const cacheLatency = new Trend('cache_request_latency', true);
const firstRequestLatency = new Trend('first_request_latency', true);
const repeatedRequestLatency = new Trend('repeated_request_latency', true);
const cacheHits = new Counter('cache_hits');
const cacheMisses = new Counter('cache_misses');

export const options = {
  scenarios: {
    cache_warmup: {
      executor: 'per-vu-iterations',
      vus: 10,
      iterations: 10,   // Each VU sends 10 requests (total 100)
      maxDuration: '1m',
      exec: 'warmup',
      startTime: '0s',
    },
    cache_test: {
      executor: 'constant-vus',
      vus: 50,
      duration: '2m',
      exec: 'cacheTest',
      startTime: '1m10s',   // Start after warmup
    },
  },
  thresholds: {
    'repeated_request_latency': ['p(95)<200'],  // Cache hits should be fast
    'errors': ['rate<0.01'],
  },
};

// Phase 1: Warm up cache with all sample data
export function warmup() {
  const sample = sampleBirthData[__ITER % sampleBirthData.length];
  const input = buildInput(sample);

  const response = http.post(
    `${BASE_URL}/api/v1/engines/panchanga/calculate`,
    JSON.stringify(input),
    { headers: authHeaders() }
  );

  check(response, {
    'warmup: status is 200': (r) => r.status === 200,
  });

  firstRequestLatency.add(response.timings.duration);
  cacheMisses.add(1);

  sleep(0.2);
}

// Phase 2: Test cache performance with same data
export function cacheTest() {
  // Pick from the same sample data (should hit cache)
  const sample = randomChoice(sampleBirthData);
  const input = buildInput(sample);

  const response = http.post(
    `${BASE_URL}/api/v1/engines/panchanga/calculate`,
    JSON.stringify(input),
    { headers: authHeaders() }
  );

  const isSuccess = response.status === 200;

  check(response, {
    'cache: status is 200': (r) => r.status === 200,
    'cache: response time < 200ms': (r) => r.timings.duration < 200,
  });

  errorRate.add(!isSuccess);
  cacheLatency.add(response.timings.duration);
  repeatedRequestLatency.add(response.timings.duration);

  // Heuristic: if response is fast, likely a cache hit
  if (response.timings.duration < 50) {
    cacheHits.add(1);
  } else {
    cacheMisses.add(1);
  }

  sleep(0.5);
}

export function handleSummary(data) {
  const summary = {
    scenario: 'cache_performance',
    warmup_vus: 10,
    test_vus: 50,
    duration: '2m',
    total_requests: data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 0,
    first_request_p50: data.metrics.first_request_latency ? data.metrics.first_request_latency.values['p(50)'] : 0,
    first_request_p95: data.metrics.first_request_latency ? data.metrics.first_request_latency.values['p(95)'] : 0,
    repeated_request_p50: data.metrics.repeated_request_latency ? data.metrics.repeated_request_latency.values['p(50)'] : 0,
    repeated_request_p95: data.metrics.repeated_request_latency ? data.metrics.repeated_request_latency.values['p(95)'] : 0,
    cache_hits: data.metrics.cache_hits ? data.metrics.cache_hits.values.count : 0,
    cache_misses: data.metrics.cache_misses ? data.metrics.cache_misses.values.count : 0,
    error_rate: data.metrics.errors ? data.metrics.errors.values.rate : 0,
  };

  return {
    'stdout': JSON.stringify(summary, null, 2) + '\n',
    'tests/load/results/scenario4-cache.json': JSON.stringify(data, null, 2),
  };
}
