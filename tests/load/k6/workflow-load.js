// tests/load/k6/workflow-load.js
// W2-S8-03: Load test workflows
//
// Target: 500 concurrent users, <2s p95 latency for workflows
// Run: k6 run tests/load/k6/workflow-load.js

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Configuration
const BASE_URL = __ENV.API_URL || 'http://localhost:8080';
const JWT_TOKEN = __ENV.JWT_TOKEN || '';

// Custom metrics
const errorRate = new Rate('errors');
const workflowLatency = new Trend('workflow_latency_ms', true);
const workflowSuccess = new Counter('workflow_success');
const workflowFailure = new Counter('workflow_failure');

// Per-workflow metrics
const birthBlueprintLatency = new Trend('birth_blueprint_latency_ms', true);
const dailyPracticeLatency = new Trend('daily_practice_latency_ms', true);
const selfInquiryLatency = new Trend('self_inquiry_latency_ms', true);

export const options = {
  scenarios: {
    workflow_load: {
      executor: 'ramping-vus',
      startVUs: 5,
      stages: [
        { duration: '30s', target: 50 },
        { duration: '1m', target: 200 },
        { duration: '2m', target: 500 },
        { duration: '3m', target: 500 },
        { duration: '1m', target: 0 },
      ],
      gracefulRampDown: '30s',
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<2000', 'p(99)<5000'],
    'errors': ['rate<0.10'],
    'workflow_latency_ms': ['p(95)<2000'],
    'birth_blueprint_latency_ms': ['p(95)<2500'],
    'daily_practice_latency_ms': ['p(95)<1500'],
    'self_inquiry_latency_ms': ['p(95)<2500'],
  },
};

const WORKFLOWS = ['birth-blueprint', 'daily-practice', 'self-inquiry'];

const birthDataVariants = [
  { name: 'Workflow NYC', date: '1990-01-15', time: '14:30', lat: 40.7128, lon: -74.006, tz: 'America/New_York' },
  { name: 'Workflow London', date: '1985-06-20', time: '09:15', lat: 51.5074, lon: -0.1278, tz: 'Europe/London' },
  { name: 'Workflow Tokyo', date: '1995-12-03', time: '18:45', lat: 35.6762, lon: 139.6503, tz: 'Asia/Tokyo' },
  { name: 'Workflow Mumbai', date: '1992-07-10', time: '22:30', lat: 19.076, lon: 72.8777, tz: 'Asia/Kolkata' },
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

function recordWorkflowMetric(workflow, duration, success) {
  workflowLatency.add(duration);
  
  if (success) {
    workflowSuccess.add(1);
  } else {
    workflowFailure.add(1);
  }
  
  switch (workflow) {
    case 'birth-blueprint':
      birthBlueprintLatency.add(duration);
      break;
    case 'daily-practice':
      dailyPracticeLatency.add(duration);
      break;
    case 'self-inquiry':
      selfInquiryLatency.add(duration);
      break;
  }
}

export default function () {
  const workflow = randomChoice(WORKFLOWS);
  const sample = randomChoice(birthDataVariants);
  const payload = buildInput(sample);

  group(`Workflow: ${workflow}`, function () {
    const response = http.post(
      `${BASE_URL}/api/v1/workflows/${workflow}/execute`,
      payload,
      { headers: authHeaders(), timeout: '30s' }
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
      'has total_time_ms': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.total_time_ms !== undefined;
        } catch (e) { return false; }
      },
      'latency under 2s': (r) => r.timings.duration < 2000,
    });

    errorRate.add(!success);
    recordWorkflowMetric(workflow, response.timings.duration, success);
  });

  // Think time: 1-2s between workflow executions
  sleep(1 + Math.random());
}

export function handleSummary(data) {
  const summary = {
    test: 'workflow-load',
    timestamp: new Date().toISOString(),
    config: {
      target_vus: 500,
      workflows: WORKFLOWS,
    },
    metrics: {
      total_requests: data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 0,
      rps: data.metrics.http_reqs ? data.metrics.http_reqs.values.rate : 0,
      p50_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(50)'] : 0,
      p95_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(95)'] : 0,
      p99_ms: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(99)'] : 0,
      error_rate: data.metrics.errors ? data.metrics.errors.values.rate : 0,
    },
    workflow_metrics: {
      birth_blueprint_p95: data.metrics.birth_blueprint_latency_ms ? 
        data.metrics.birth_blueprint_latency_ms.values['p(95)'] : null,
      daily_practice_p95: data.metrics.daily_practice_latency_ms ? 
        data.metrics.daily_practice_latency_ms.values['p(95)'] : null,
      self_inquiry_p95: data.metrics.self_inquiry_latency_ms ? 
        data.metrics.self_inquiry_latency_ms.values['p(95)'] : null,
    },
    thresholds_passed: Object.entries(data.thresholds || {})
      .every(([_, v]) => v.ok),
  };

  return {
    'stdout': JSON.stringify(summary, null, 2) + '\n',
    'tests/load/k6/results/workflow-load-results.json': JSON.stringify(data, null, 2),
  };
}
