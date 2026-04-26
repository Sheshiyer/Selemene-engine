// tests/load/smoke-test.js
// Quick smoke test to verify server is up and API is responding before full load tests
//
// Usage: k6 run --env JWT_TOKEN=<token> tests/load/smoke-test.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { BASE_URL, authHeaders, birthData } from './helpers.js';

export const options = {
  vus: 1,
  iterations: 1,
};

export default function () {
  // 1. Health check (no auth needed)
  const healthRes = http.get(`${BASE_URL}/health`);
  check(healthRes, {
    'health: status 200': (r) => r.status === 200,
    'health: status ok': (r) => {
      try { return JSON.parse(r.body).status === 'ok'; } catch (e) { return false; }
    },
  });
  console.log(`Health: ${healthRes.status} - ${healthRes.body}`);

  // 2. List engines (auth required)
  const enginesRes = http.get(`${BASE_URL}/api/v1/engines`, { headers: authHeaders() });
  check(enginesRes, {
    'engines: status 200': (r) => r.status === 200,
    'engines: returns list': (r) => {
      try { return JSON.parse(r.body).engines.length > 0; } catch (e) { return false; }
    },
  });
  console.log(`Engines: ${enginesRes.status} - ${enginesRes.body}`);

  // 3. Calculate panchanga
  const calcRes = http.post(
    `${BASE_URL}/api/v1/engines/panchanga/calculate`,
    JSON.stringify(birthData),
    { headers: authHeaders() }
  );
  check(calcRes, {
    'calculate: status 200': (r) => r.status === 200,
  });
  console.log(`Calculate: ${calcRes.status} (${calcRes.timings.duration.toFixed(0)}ms)`);

  // 4. List workflows
  const wfRes = http.get(`${BASE_URL}/api/v1/workflows`, { headers: authHeaders() });
  check(wfRes, {
    'workflows: status 200': (r) => r.status === 200,
  });
  console.log(`Workflows: ${wfRes.status} - ${wfRes.body}`);

  console.log('\nSmoke test complete. Server is ready for load testing.');
}
