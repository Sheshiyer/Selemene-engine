// tests/load/scenario5-ratelimit.js
// Scenario 5: Rate Limit Testing - exceed configured rate limits
//
// Purpose: Validate rate limiter correctly enforces per-user request limits.
// The server uses 100 req/min default, but premium tier = 1000 req/min.
// We test by sending requests faster than the limit allows.

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Counter, Trend } from 'k6/metrics';
import { BASE_URL, authHeaders, birthData } from './helpers.js';

const rateLimitTriggered = new Rate('rate_limit_triggered');
const status200Count = new Counter('status_200');
const status429Count = new Counter('status_429');
const otherStatusCount = new Counter('other_status');
const requestLatency = new Trend('request_latency', true);

export const options = {
  scenarios: {
    rate_limit_test: {
      executor: 'constant-arrival-rate',
      rate: 200,             // 200 requests per timeUnit
      timeUnit: '10s',       // = 1200 req/min (exceeds 1000/min premium limit)
      duration: '2m',
      preAllocatedVUs: 20,
      maxVUs: 50,
    },
  },
  thresholds: {
    // We EXPECT 429s here - that means rate limiting works
    'rate_limit_triggered': ['rate>0'],  // At least some requests should be rate limited
  },
};

export default function () {
  const response = http.post(
    `${BASE_URL}/api/v1/engines/panchanga/calculate`,
    JSON.stringify(birthData),
    { headers: authHeaders() }
  );

  const is200 = response.status === 200;
  const is429 = response.status === 429;

  check(response, {
    'response is 200 or 429': (r) => r.status === 200 || r.status === 429,
    'rate limit returns proper error': (r) => {
      if (r.status !== 429) return true;
      try {
        const body = JSON.parse(r.body);
        return body.error_code === 'RATE_LIMIT_EXCEEDED';
      } catch (e) { return false; }
    },
    'rate limit has headers': (r) => {
      if (r.status !== 429) return true;
      return r.headers['X-Ratelimit-Limit'] !== undefined ||
             r.headers['x-ratelimit-limit'] !== undefined;
    },
  });

  rateLimitTriggered.add(is429);
  requestLatency.add(response.timings.duration);

  if (is200) {
    status200Count.add(1);
  } else if (is429) {
    status429Count.add(1);
  } else {
    otherStatusCount.add(1);
  }

  // No sleep - we want to overwhelm the rate limiter
}

export function handleSummary(data) {
  const total200 = data.metrics.status_200 ? data.metrics.status_200.values.count : 0;
  const total429 = data.metrics.status_429 ? data.metrics.status_429.values.count : 0;
  const totalOther = data.metrics.other_status ? data.metrics.other_status.values.count : 0;
  const totalRequests = total200 + total429 + totalOther;

  const summary = {
    scenario: 'rate_limit_test',
    target_rate: '1200 req/min',
    duration: '2m',
    total_requests: totalRequests,
    status_200: total200,
    status_429: total429,
    other_status: totalOther,
    rate_limit_percentage: totalRequests > 0 ? ((total429 / totalRequests) * 100).toFixed(2) + '%' : '0%',
    rate_limit_enforced: total429 > 0,
    p50_ms: data.metrics.request_latency ? data.metrics.request_latency.values['p(50)'] : 0,
    p95_ms: data.metrics.request_latency ? data.metrics.request_latency.values['p(95)'] : 0,
  };

  return {
    'stdout': JSON.stringify(summary, null, 2) + '\n',
    'tests/load/results/scenario5-ratelimit.json': JSON.stringify(data, null, 2),
  };
}
