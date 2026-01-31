// tests/load/helpers.js
// Shared helpers for Noesis API load tests
//
// Usage: import { BASE_URL, authHeaders, birthData, engines } from './helpers.js';

import { SharedArray } from 'k6/data';
import exec from 'k6/execution';

// Base URL - override with API_URL env var
export const BASE_URL = __ENV.API_URL || 'http://localhost:8080';

// Single JWT token (for smoke/simple tests)
export const JWT_TOKEN = __ENV.JWT_TOKEN || '';

// Multiple JWT tokens loaded from file (for load tests with per-VU rate limit buckets)
// Each line is a separate enterprise-tier token with unique user_id
// SharedArray MUST be initialized in init context (module scope)
const tokens = __ENV.TOKEN_FILE
  ? new SharedArray('tokens', function () {
      return open(__ENV.TOKEN_FILE).trim().split('\n');
    })
  : null;

export function getTokenForVU() {
  if (tokens && tokens.length > 0) {
    const vuIndex = (exec.vu.idInTest - 1) % tokens.length;
    return tokens[vuIndex];
  }
  return JWT_TOKEN;
}

// Standard auth headers - uses per-VU token if available
export function authHeaders() {
  const token = getTokenForVU();
  return {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${token}`,
  };
}

// Standard birth data payload for engine calculations
// Note: current_time is required by EngineInput (DateTime<Utc>)
export const birthData = {
  birth_data: {
    name: 'Load Test User',
    date: '1990-01-15',
    time: '14:30',
    latitude: 40.7128,
    longitude: -74.006,
    timezone: 'America/New_York',
  },
  current_time: new Date().toISOString(),
  precision: 'Standard',
  options: {},
};

// Sample birth data array for cache testing (varied inputs)
export const sampleBirthData = [
  { name: 'User NYC', date: '1990-01-15', time: '14:30', latitude: 40.7128, longitude: -74.006, timezone: 'America/New_York' },
  { name: 'User London', date: '1985-06-20', time: '09:15', latitude: 51.5074, longitude: -0.1278, timezone: 'Europe/London' },
  { name: 'User Tokyo', date: '1995-12-03', time: '18:45', latitude: 35.6762, longitude: 139.6503, timezone: 'Asia/Tokyo' },
  { name: 'User Sydney', date: '1988-03-21', time: '06:00', latitude: -33.8688, longitude: 151.2093, timezone: 'Australia/Sydney' },
  { name: 'User Mumbai', date: '1992-07-10', time: '22:30', latitude: 19.076, longitude: 72.8777, timezone: 'Asia/Kolkata' },
  { name: 'User Berlin', date: '1979-11-25', time: '11:45', latitude: 52.52, longitude: 13.405, timezone: 'Europe/Berlin' },
  { name: 'User Sao Paulo', date: '2000-02-14', time: '03:15', latitude: -23.5505, longitude: -46.6333, timezone: 'America/Sao_Paulo' },
  { name: 'User Cairo', date: '1975-09-08', time: '16:00', latitude: 30.0444, longitude: 31.2357, timezone: 'Africa/Cairo' },
  { name: 'User LA', date: '1998-04-30', time: '20:15', latitude: 34.0522, longitude: -118.2437, timezone: 'America/Los_Angeles' },
  { name: 'User Delhi', date: '1983-08-17', time: '07:45', latitude: 28.6139, longitude: 77.209, timezone: 'Asia/Kolkata' },
];

// Available engine IDs
export const engines = [
  'panchanga',
  'numerology',
  'biorhythm',
  'human-design',
  'gene-keys',
  'vimshottari',
];

// Workflow IDs that exist in the server
export const workflows = [
  'birth-blueprint',
  'daily-practice',
  'self-inquiry',
];

// Build engine input from sample birth data
export function buildInput(sample) {
  return {
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
  };
}

// Random element from array
export function randomChoice(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}
