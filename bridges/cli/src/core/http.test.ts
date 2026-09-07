// @ts-nocheck Bun supplies the test runtime; this package intentionally keeps
// its production dependency surface unchanged.
import { afterEach, describe, expect, test } from "bun:test";
import { checkHealth, fetchJSON } from "./http.js";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
});

describe("typed CLI HTTP transport", () => {
  test("sends API keys only in X-API-Key", async () => {
    let request: RequestInit | undefined;
    globalThis.fetch = (async (_url, init) => {
      request = init;
      return new Response(JSON.stringify({ ok: true }), { status: 200 });
    }) as typeof fetch;

    await fetchJSON("https://example.test/capabilities", { apiKey: "key-secret" });

    const headers = new Headers(request?.headers);
    expect(headers.get("X-API-Key")).toBe("key-secret");
    expect(headers.get("Authorization")).toBeNull();
  });

  test("sends bearer tokens only in Authorization", async () => {
    let request: RequestInit | undefined;
    globalThis.fetch = (async (_url, init) => {
      request = init;
      return new Response(JSON.stringify({ ok: true }), { status: 200 });
    }) as typeof fetch;

    await fetchJSON("https://example.test/capabilities", { bearerToken: "jwt-secret" });

    const headers = new Headers(request?.headers);
    expect(headers.get("Authorization")).toBe("Bearer jwt-secret");
    expect(headers.get("X-API-Key")).toBeNull();
  });

  test("rejects ambiguous credentials before fetch", async () => {
    let calls = 0;
    globalThis.fetch = (async () => {
      calls += 1;
      return new Response("{}", { status: 200 });
    }) as typeof fetch;

    await expect(
      fetchJSON("https://example.test/capabilities", {
        apiKey: "key-secret",
        bearerToken: "jwt-secret",
      }),
    ).rejects.toThrow("ambiguous authentication");
    expect(calls).toBe(0);
  });

  test("bounds empty, malformed and upstream error bodies", async () => {
    for (const response of [
      new Response("", { status: 502 }),
      new Response("raw upstream body token=super-secret", { status: 502 }),
      new Response("{malformed", { status: 502 }),
    ]) {
      globalThis.fetch = (async () => response) as typeof fetch;
      await expect(fetchJSON("https://example.test/capabilities")).rejects.toThrow(/\(502\)/);
      try {
        await fetchJSON("https://example.test/capabilities");
      } catch (error) {
        expect(String(error)).not.toContain("super-secret");
        expect(String(error)).not.toContain("upstream body");
      }
    }
  });

  test("health errors expose a bounded reason", async () => {
    globalThis.fetch = (async () => {
      throw new Error("connect ECONNREFUSED https://example.test");
    }) as typeof fetch;
    await expect(checkHealth("https://example.test/health")).resolves.toEqual({
      ok: false,
      error: "request_failed",
    });
  });
});
