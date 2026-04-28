// tests/load/scenario6-auth-soak.js
// Scenario 6: Auth Soak Test - 60-minute sustained load rotating JWT and API key auth
//
// Purpose: Validate that authentication remains stable under prolonged traffic,
// rotating between JWT bearer tokens and X-API-Key header auth methods.
// Target: 50 VUs for 60 minutes, auth failure rate < 0.1%
//
// Usage:
//   k6 run tests/load/scenario6-auth-soak.js
//
// Environment variables:
//   API_URL      - Base URL (default: http://localhost:8080)
//   JWT_TOKEN    - Single JWT bearer token (used when TOKEN_FILE is absent)
//   TOKEN_FILE   - Path to newline-separated file of JWT tokens (one per line)
//   API_KEY      - Single API key for X-API-Key header auth
//   API_KEY_FILE - Path to newline-separated file of API keys (one per line)

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { SharedArray } from 'k6/data';
import exec from 'k6/execution';
import { BASE_URL, sampleBirthData, engines, buildInput, randomChoice } from './helpers.js';

// ---------------------------------------------------------------------------
// Custom metrics
// ---------------------------------------------------------------------------

const authFailureRate = new Rate('auth_failures');
const authLatency = new Trend('auth_latency', true);
const jwtLatency = new Trend('jwt_auth_latency', true);
const apiKeyLatency = new Trend('api_key_auth_latency', true);
const tokenValidationFailures = new Counter('token_validation_failures');
const jwtRequests = new Counter('jwt_requests');
const apiKeyRequests = new Counter('api_key_requests');

// ---------------------------------------------------------------------------
// Token pools (loaded once in init context)
// ---------------------------------------------------------------------------

const jwtTokens = __ENV.TOKEN_FILE
  ? new SharedArray('jwt_tokens', function () {
      return open(__ENV.TOKEN_FILE).trim().split('\n').filter(t => t.length > 0);
    })
  : null;

const apiKeys = __ENV.API_KEY_FILE
  ? new SharedArray('api_keys', function () {
      return open(__ENV.API_KEY_FILE).trim().split('\n').filter(k => k.length > 0);
    })
  : null;

// ---------------------------------------------------------------------------
// Auth header helpers
// ---------------------------------------------------------------------------

function jwtHeaders() {
  let token;
  if (jwtTokens && jwtTokens.length > 0) {
    token = jwtTokens[(exec.vu.idInTest - 1) % jwtTokens.length];
  } else {
    token = __ENV.JWT_TOKEN || '';
  }
  return {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${token}`,
  };
}

function apiKeyHeaders() {
  let key;
  if (apiKeys && apiKeys.length > 0) {
    key = apiKeys[(exec.vu.idInTest - 1) % apiKeys.length];
  } else {
    key = __ENV.API_KEY || '';
  }
  return {
    'Content-Type': 'application/json',
    'X-API-Key': key,
  };
}

// Alternate auth method per-VU iteration so each VU rotates independently
function pickHeaders() {
  const useJwt = exec.vu.iterationInInstance % 2 === 0;
  return { headers: useJwt ? jwtHeaders() : apiKeyHeaders(), method: useJwt ? 'jwt' : 'api_key' };
}

// ---------------------------------------------------------------------------
// k6 options
// ---------------------------------------------------------------------------

export const options = {
  scenarios: {
    auth_soak: {
      executor: 'constant-vus',
      vus: 50,
      duration: '60m',
    },
  },
  thresholds: {
    // Auth failure rate must stay below 0.1%
    'auth_failures': ['rate<0.001'],
    // Overall HTTP error rate below 1%
    'http_req_failed': ['rate<0.01'],
    // p95 auth latency under 1 second
    'auth_latency': ['p(95)<1000'],
    // JWT and API key latency individually
    'jwt_auth_latency': ['p(95)<1000'],
    'api_key_auth_latency': ['p(95)<1000'],
  },
};

// ---------------------------------------------------------------------------
// Default function
// ---------------------------------------------------------------------------

export default function () {
  const { headers, method } = pickHeaders();
  const engineId = randomChoice(engines);
  const sample = randomChoice(sampleBirthData);
  const input = buildInput(sample);

  const response = http.post(
    `${BASE_URL}/api/v1/engines/${engineId}/calculate`,
    JSON.stringify(input),
    { headers }
  );

  const isSuccess = response.status === 200;
  // 401/403 indicate token validation failures
  const isAuthFailure = response.status === 401 || response.status === 403;

  check(response, {
    'auth: status is 200': (r) => r.status === 200,
    'auth: not 401 unauthorized': (r) => r.status !== 401,
    'auth: not 403 forbidden': (r) => r.status !== 403,
    'auth: has response body': (r) => r.body && r.body.length > 0,
  });

  authFailureRate.add(isAuthFailure);
  authLatency.add(response.timings.duration);

  if (isAuthFailure) {
    tokenValidationFailures.add(1);
  }

  if (method === 'jwt') {
    jwtLatency.add(response.timings.duration);
    jwtRequests.add(1);
  } else {
    apiKeyLatency.add(response.timings.duration);
    apiKeyRequests.add(1);
  }

  // Realistic think time between requests (1-3 seconds)
  sleep(1 + Math.random() * 2);
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

export function handleSummary(data) {
  const authFailRate = data.metrics.auth_failures
    ? data.metrics.auth_failures.values.rate
    : 0;

  const summary = {
    scenario: 'auth_soak',
    vus: 50,
    duration: '60m',
    total_requests: data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 0,
    jwt_requests: data.metrics.jwt_requests ? data.metrics.jwt_requests.values.count : 0,
    api_key_requests: data.metrics.api_key_requests ? data.metrics.api_key_requests.values.count : 0,
    token_validation_failures: data.metrics.token_validation_failures
      ? data.metrics.token_validation_failures.values.count
      : 0,
    auth_failure_rate: authFailRate,
    auth_failure_rate_pct: (authFailRate * 100).toFixed(4) + '%',
    auth_failure_threshold: '< 0.1%',
    auth_failure_threshold_passed: authFailRate < 0.001,
    auth_latency_p50_ms: data.metrics.auth_latency
      ? data.metrics.auth_latency.values['p(50)']
      : 0,
    auth_latency_p95_ms: data.metrics.auth_latency
      ? data.metrics.auth_latency.values['p(95)']
      : 0,
    auth_latency_p99_ms: data.metrics.auth_latency
      ? data.metrics.auth_latency.values['p(99)']
      : 0,
    jwt_latency_p95_ms: data.metrics.jwt_auth_latency
      ? data.metrics.jwt_auth_latency.values['p(95)']
      : 0,
    api_key_latency_p95_ms: data.metrics.api_key_auth_latency
      ? data.metrics.api_key_auth_latency.values['p(95)']
      : 0,
    http_req_failed_rate: data.metrics.http_req_failed
      ? data.metrics.http_req_failed.values.rate
      : 0,
  };

  return {
    'stdout': JSON.stringify(summary, null, 2) + '\n',
    'tests/load/results/scenario6-auth-soak.json': JSON.stringify(data, null, 2),
  };
}
