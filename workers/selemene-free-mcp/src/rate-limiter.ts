import { readLimits, type Env, type FreeContext, type Limits } from "./env.js";
export const CLIENT_TTL_MS = 120_000;
export const MAX_CLIENTS = 512;
const STATE_KEY = "counters";
type Kind = "request" | "calculation";
interface ClientCounters { minute: number; requests: number; calculations: number; lastSeen: number }
interface CounterState {
  minute: number; globalRequests: number; globalCalculations: number;
  day: number; dailyCalculations: number; clients: Record<string, ClientCounters>;
}
export interface RateDecision { allowed: boolean; retryAfter: number; reason?: string }
function reset(state: CounterState | undefined, now: number): CounterState {
  const minute = Math.floor(now / 60_000), day = Math.floor(now / 86_400_000);
  if (!state) return { minute, day, globalRequests: 0, globalCalculations: 0, dailyCalculations: 0, clients: {} };
  if (state.minute !== minute) { state.minute = minute; state.globalRequests = 0; state.globalCalculations = 0; }
  if (state.day !== day) { state.day = day; state.dailyCalculations = 0; }
  for (const [hash, client] of Object.entries(state.clients)) {
    if (now - client.lastSeen >= CLIENT_TTL_MS) delete state.clients[hash];
  }
  return state;
}
function take(state: CounterState, hash: string, kind: Kind, limits: Limits, now: number): RateDecision {
  let client = state.clients[hash];
  if (!client) {
    if (Object.keys(state.clients).length >= MAX_CLIENTS) return { allowed: false, retryAfter: 120, reason: "capacity" };
    client = { minute: state.minute, requests: 0, calculations: 0, lastSeen: now };
    state.clients[hash] = client;
  }
  if (client.minute !== state.minute) { client.minute = state.minute; client.requests = 0; client.calculations = 0; }
  client.lastSeen = now;
  const retryAfter = Math.max(1, Math.ceil(((state.minute + 1) * 60_000 - now) / 1000));
  if (kind === "request") {
    if (client.requests >= limits.ipRequests || state.globalRequests >= limits.globalRequests) return { allowed: false, retryAfter, reason: "minute" };
    client.requests++; state.globalRequests++;
  } else {
    if (state.dailyCalculations >= limits.dailyCalculations) return {
      allowed: false, retryAfter: Math.max(1, Math.ceil(((state.day + 1) * 86_400_000 - now) / 1000)), reason: "day",
    };
    if (client.calculations >= limits.ipCalculations || state.globalCalculations >= limits.globalCalculations) return { allowed: false, retryAfter, reason: "minute" };
    client.calculations++; state.globalCalculations++; state.dailyCalculations++;
  }
  return { allowed: true, retryAfter: 0 };
}
// One persistent instance supplies an atomic service ceiling across Worker isolates.
// Only salted hashes, counters and timestamps are retained.
export class FreeRateLimiter {
  constructor(private state: DurableObjectState, private env: Env) {}
  async fetch(request: Request): Promise<Response> {
    if (request.method !== "POST") return new Response(null, { status: 405 });
    try {
      const raw = await request.text();
      if (new TextEncoder().encode(raw).byteLength > 512) return new Response(null, { status: 400 });
      const data = JSON.parse(raw) as Record<string, unknown>;
      if (!data || Array.isArray(data) || Object.keys(data).some((key) => key !== "clientHash" && key !== "kind") ||
          typeof data.clientHash !== "string" || !/^[a-f0-9]{64}$/.test(data.clientHash) ||
          (data.kind !== "request" && data.kind !== "calculation")) return new Response(null, { status: 400 });
      const limits = readLimits(this.env), now = Date.now();
      const result = await this.state.storage.transaction(async (tx) => {
        const counters = reset(await tx.get<CounterState>(STATE_KEY), now);
        const decision = take(counters, data.clientHash as string, data.kind as Kind, limits, now);
        await tx.put(STATE_KEY, counters);
        return decision;
      });
      await this.state.storage.setAlarm(now + CLIENT_TTL_MS);
      return Response.json(result, { headers: { "cache-control": "no-store" } });
    } catch { return Response.json({ allowed: false, retryAfter: 60, reason: "unavailable" }, { status: 503 }); }
  }
  async alarm(): Promise<void> {
    const now = Date.now();
    const active = await this.state.storage.transaction(async (tx) => {
      const counters = reset(await tx.get<CounterState>(STATE_KEY), now);
      await tx.put(STATE_KEY, counters);
      return Object.keys(counters.clients).length > 0;
    });
    if (active) await this.state.storage.setAlarm(now + CLIENT_TTL_MS);
  }
}
export async function clientHash(request: Request, salt: string): Promise<string | null> {
  const ip = request.headers.get("cf-connecting-ip");
  // Cloudflare supplies/overwrites this reserved header at the public edge.
  // Never accept X-Forwarded-For or caller-supplied quota identities.
  if (!ip || !/^[0-9a-f:.]{3,45}$/i.test(ip)) return null;
  const encoder = new TextEncoder();
  const key = await crypto.subtle.importKey("raw", encoder.encode(salt), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  const bytes = new Uint8Array(await crypto.subtle.sign("HMAC", key, encoder.encode(`selemene-free|${ip}`)));
  return [...bytes].map((n) => n.toString(16).padStart(2, "0")).join("");
}
export async function reserve(ctx: FreeContext, hash: string, kind: Kind): Promise<RateDecision> {
  try {
    const id = ctx.limiter.idFromName("sponsor-global");
    const response = await ctx.limiter.get(id).fetch("https://rate.internal/reserve", {
      method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ clientHash: hash, kind }),
    });
    if (!response.ok) return { allowed: false, retryAfter: 60, reason: "unavailable" };
    const result = await response.json() as RateDecision;
    if (typeof result.allowed !== "boolean" || !Number.isFinite(result.retryAfter) || result.retryAfter < 0) return { allowed: false, retryAfter: 60, reason: "unavailable" };
    return result;
  } catch { return { allowed: false, retryAfter: 60, reason: "unavailable" }; }
}
