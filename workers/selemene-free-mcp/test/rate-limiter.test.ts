import { afterEach, describe, expect, it, vi } from "vitest";
import { CLIENT_TTL_MS, MAX_CLIENTS, clientHash } from "../src/rate-limiter.js";
import { makeEnv, request, init, SYNTHETIC_KEY } from "./helpers.js";
afterEach(() => vi.restoreAllMocks());
const hash = (n: number) => n.toString(16).padStart(64, "0");
async function take(fixture: ReturnType<typeof makeEnv>, n: number, kind = "calculation") {
  const response = await fixture.object.fetch(new Request("https://rate.internal/reserve", {
    method: "POST", body: JSON.stringify({ clientHash: hash(n), kind }),
  }));
  return response.json() as Promise<{ allowed: boolean; retryAfter: number; reason: string }>;
}
describe("persistent atomic sponsor quotas", () => {
  it("enforces one global daily ceiling under concurrent distinct clients", async () => {
    const fixture = makeEnv({ GLOBAL_CALCULATIONS_PER_DAY: "3" });
    const decisions = await Promise.all(Array.from({ length: 12 }, (_, i) => take(fixture, i)));
    expect(decisions.filter((item) => item.allowed)).toHaveLength(3);
    expect((fixture.records.get("counters") as { dailyCalculations: number }).dailyCalculations).toBe(3);
    expect(fixture.records.size).toBe(1);
  });
  it("separates request and calculation budgets and resets fixed minute windows under continuous traffic", async () => {
    let now = Date.parse("2026-01-01T00:00:00Z");
    vi.spyOn(Date, "now").mockImplementation(() => now);
    const fixture = makeEnv({ IP_REQUESTS_PER_MINUTE: "2", IP_CALCULATIONS_PER_MINUTE: "1" });
    expect((await take(fixture, 1, "request")).allowed).toBe(true);
    expect((await take(fixture, 1, "request")).allowed).toBe(true);
    expect((await take(fixture, 1, "request")).allowed).toBe(false);
    expect((await take(fixture, 1)).allowed).toBe(true);
    expect((await take(fixture, 1)).allowed).toBe(false);
    now += 61000;
    expect((await take(fixture, 1, "request")).allowed).toBe(true);
    expect((await take(fixture, 1)).allowed).toBe(true);
    expect((fixture.records.get("counters") as { dailyCalculations: number }).dailyCalculations).toBe(2);
  });
  it("resets daily attempts only at the next UTC day, independently of IP TTL", async () => {
    let now = Date.parse("2026-01-01T12:00:00Z");
    vi.spyOn(Date, "now").mockImplementation(() => now);
    const fixture = makeEnv({ GLOBAL_CALCULATIONS_PER_DAY: "1" });
    expect((await take(fixture, 1)).allowed).toBe(true);
    now += CLIENT_TTL_MS;
    await fixture.object.alarm();
    expect((await take(fixture, 2)).allowed).toBe(false);
    now = Date.parse("2026-01-02T00:00:00Z");
    expect((await take(fixture, 2)).allowed).toBe(true);
  });
  it("bounds512 identities below128KiB, prunesTTL and never stores raw IP/input/secret", async () => {
    let now = Date.parse("2026-01-01T00:00:00Z");
    vi.spyOn(Date, "now").mockImplementation(() => now);
    const fixture = makeEnv({ GLOBAL_REQUESTS_PER_MINUTE: "10000" });
    for (let i = 0; i < MAX_CLIENTS; i++) expect((await take(fixture, i, "request")).allowed).toBe(true);
    expect((await take(fixture, MAX_CLIENTS, "request")).reason).toBe("capacity");
    const stored = JSON.stringify([...fixture.records.values()]);
    expect(new TextEncoder().encode(stored).byteLength).toBeLessThan(128 * 1024);
    expect(stored).not.toContain("203.0.113");
    expect(stored).not.toContain(SYNTHETIC_KEY);
    expect(stored).not.toContain("birth_data");
    now += CLIENT_TTL_MS;
    await fixture.object.alarm();
    expect(Object.keys((fixture.records.get("counters") as { clients: object }).clients)).toHaveLength(0);
    expect((await take(fixture, MAX_CLIENTS, "request")).allowed).toBe(true);
  });
  it("uses salted HMAC only on trusted Cloudflare IP and rejects caller-supplied limiter fields", async () => {
    const req = request(init);
    const a = await clientHash(req, "a".repeat(32)), b = await clientHash(req, "b".repeat(32));
    expect(a).toMatch(/^[a-f0-9]{64}$/);
    expect(a).not.toBe(b);
    req.headers.delete("cf-connecting-ip"); req.headers.set("x-forwarded-for", "203.0.113.8");
    expect(await clientHash(req, "a".repeat(32))).toBeNull();
    const fixture = makeEnv();
    const denied = await fixture.object.fetch(new Request("https://rate.internal/reserve", {
      method: "POST", body: JSON.stringify({ clientHash: hash(1), kind: "request", dailyLimit: 999999 }),
    }));
    expect(denied.status).toBe(400);
    expect(fixture.records.size).toBe(0);
  });
});
