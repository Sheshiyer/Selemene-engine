// tests/load/scenario3-workflows.js
// Scenario 3: Workflow Load - multi-engine workflow execution
//
// Purpose: Validate workflow endpoint handles parallel engine execution under load.
// Target: p95 < 2000ms (workflows invoke multiple engines), error rate < 1%

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { BASE_URL, authHeaders, birthData, workflows, randomChoice } from './helpers.js';

const errorRate = new Rate('errors');
const workflowLatency = new Trend('workflow_latency', true);
const engineResultCount = new Counter('engine_results_returned');

export const options = {
  scenarios: {
    workflow_load: {
      executor: 'constant-vus',
      vus: 50,        // Lower VUs since workflows are heavier
      duration: '3m',
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<2000'],   // Workflows take longer (multi-engine)
    'errors': ['rate<0.01'],
    'workflow_latency': ['p(95)<2000'],
  },
};

export default function () {
  const workflowId = randomChoice(workflows);

  const response = http.post(
    `${BASE_URL}/api/v1/workflows/${workflowId}/execute`,
    JSON.stringify(birthData),
    { headers: authHeaders() }
  );

  const isSuccess = response.status === 200;

  check(response, {
    'status is 200': (r) => r.status === 200,
    'response is JSON': (r) => {
      try { JSON.parse(r.body); return true; } catch (e) { return false; }
    },
    'has results object': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.results !== undefined || body.engine_outputs !== undefined;
      } catch (e) { return false; }
    },
  });

  errorRate.add(!isSuccess);
  workflowLatency.add(response.timings.duration);

  // Count engine results returned
  if (isSuccess) {
    try {
      const body = JSON.parse(response.body);
      const results = body.results || body.engine_outputs || {};
      engineResultCount.add(Object.keys(results).length);
    } catch (e) {
      // Ignore parse errors in counting
    }
  }

  sleep(2); // Longer think time for heavier workflows
}

export function handleSummary(data) {
  const summary = {
    scenario: 'workflow_load',
    vus: 50,
    duration: '3m',
    total_requests: data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 0,
    rps: data.metrics.http_reqs ? data.metrics.http_reqs.values.rate : 0,
    p50_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(50)'] : 0,
    p95_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(95)'] : 0,
    p99_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(99)'] : 0,
    error_rate: data.metrics.errors ? data.metrics.errors.values.rate : 0,
  };

  return {
    'stdout': JSON.stringify(summary, null, 2) + '\n',
    'tests/load/results/scenario3-workflows.json': JSON.stringify(data, null, 2),
  };
}
