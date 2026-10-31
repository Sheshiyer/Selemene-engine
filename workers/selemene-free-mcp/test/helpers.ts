import { vi } from "vitest";
import { FreeRateLimiter } from "../src/rate-limiter.js";
import { PUBLIC_ORIGIN, type Env } from "../src/env.js";
export const SYNTHETIC_KEY = "nk_synthetic_free_connector_test_only";
export const API_ORIGIN = "https://selemene-engine-production.up.railway.app";
export const birth = { name: "Synthetic Person", date: "1990-01-15", time: "12:00", latitude: 0, longitude: 0, timezone: "UTC" };
export function makeEnv(overrides: Partial<Env> = {}) {
  const records = new Map<string, unknown>();
  const alarms: number[] = [];
  let queue: Promise<unknown> = Promise.resolve();
  const tx = {
    async get<T>(key: string): Promise<T | undefined> { const value = records.get(key); return value === undefined ? undefined : structuredClone(value) as T; },
    async put(key: string, value: unknown) {
      if (new TextEncoder().encode(JSON.stringify(value)).byteLength > 128 * 1024) throw new Error("Record exceeds DO KV value limit");
      records.set(key, structuredClone(value));
    },
  };
  const storage = {
    ...tx,
    transaction<T>(callback: (transaction: typeof tx) => Promise<T>): Promise<T> {
      const result = queue.then(() => callback(tx));
      queue = result.then(() => undefined, () => undefined);
      return result;
    },
    async setAlarm(at: number) { alarms.push(at); },
  };
  const state = { storage } as unknown as DurableObjectState;
  const env: Env = {
    PUBLIC_ORIGIN, SELEMENE_API_ORIGIN: API_ORIGIN, SELEMENE_API_KEY: SYNTHETIC_KEY,
    RATE_LIMIT_SALT: "synthetic-rate-salt-never-production-12345",
    FREE_RATE_LIMITER: {} as DurableObjectNamespace, ...overrides,
  };
  const object = new FreeRateLimiter(state, env);
  const fetchObject = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) =>
    object.fetch(new Request(input, init)));
  const namespace = {
    idFromName: vi.fn(() => ({ toString: () => "sponsor-global" })),
    get: vi.fn(() => ({ fetch: fetchObject })),
  } as unknown as DurableObjectNamespace;
  env.FREE_RATE_LIMITER = namespace;
  return { env, records, alarms, object, namespace, fetchObject };
}
export function request(body: unknown, ip = "203.0.113.8", extra: Record<string, string> = {}, path = "/mcp") {
  return new Request(`${PUBLIC_ORIGIN}${path}`, { method: "POST", headers: {
    "content-type": "application/json", accept: "application/json, text/event-stream",
    "mcp-protocol-version": "2025-03-26", "cf-connecting-ip": ip, ...extra,
  }, body: typeof body === "string" ? body : JSON.stringify(body) });
}
export const init = { jsonrpc: "2.0", id: 1, method: "initialize", params: {
  protocolVersion: "2025-03-26", capabilities: {}, clientInfo: { name: "synthetic-client", version: "1.0" },
} };
export function call(name: string, args: Record<string, unknown> = {}) {
  return { jsonrpc: "2.0", id: 3, method: "tools/call", params: { name, arguments: args } };
}
