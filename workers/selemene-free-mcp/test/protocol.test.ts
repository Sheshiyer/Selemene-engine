import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StreamableHTTPClientTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js";
import worker from "../src/index.js";
import { PUBLIC_ORIGIN } from "../src/env.js";
import { SHARED_STORAGE_DISCLOSURE } from "../src/sanitize.js";
import { API_ORIGIN, SYNTHETIC_KEY, call, init, makeEnv, request } from "./helpers.js";

let fixture: ReturnType<typeof makeEnv>;
let phase = 5, responseStatus = 200;
let result: Record<string, unknown>;
const upstream = vi.fn(async (input: RequestInfo | URL, options?: RequestInit) => {
  const url = new URL(typeof input === "string" ? input : input instanceof URL ? input.href : input.url);
  expect(url.origin).toBe(API_ORIGIN);
  expect(new Headers(options?.headers).get("X-API-Key")).toBe(SYNTHETIC_KEY);
  expect(new Headers(options?.headers).get("authorization")).toBeNull();
  if (url.pathname === "/api/v1/users/me") return Response.json({
    id: "private-shared-account-id", email: "private-account@example.test", tier: "enterprise", consciousness_level: phase,
  });
  if (url.pathname === "/api/v1/engines") return Response.json({ engines: ["tarot", "human-design", "numerology"] });
  if (url.pathname === "/api/v1/engines/capabilities") return new Response(null, { status: 404 });
  if (url.pathname === "/api/v1/engines/tarot/calculate") return Response.json(result, { status: responseStatus });
  throw new Error("Unexpected upstream path");
});
beforeEach(() => {
  fixture = makeEnv(); phase = 5; responseStatus = 200;
  result = { output: { engine_id: "tarot", result: { cards: ["The Star"] }, consciousness_level: 5 } };
  upstream.mockClear(); vi.stubGlobal("fetch", upstream);
});
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks(); });
async function rpc(body: unknown) {
  const response = await worker.fetch(request(body), fixture.env);
  const raw = await response.text();
  expect(raw).not.toContain(SYNTHETIC_KEY);
  expect(raw).not.toContain("private-shared-account-id");
  expect(raw).not.toContain("private-account@example.test");
  return { response, raw, data: raw ? JSON.parse(raw) : null };
}
describe("official stateless MCP protocol", () => {
  it("connects the actual SDK client and preserves noauth discovery metadata", async () => {
    const client = new Client({ name: "sdk-proof", version: "1.0" }, { capabilities: {} });
    const mockedFetch = (async (input: RequestInfo | URL, options?: RequestInit) => {
      const incoming = new Request(input, options);
      incoming.headers.set("cf-connecting-ip", "203.0.113.8");
      return worker.fetch(incoming, fixture.env);
    }) as typeof fetch;
    const transport = new StreamableHTTPClientTransport(new URL(`${PUBLIC_ORIGIN}/mcp`), { fetch: mockedFetch });
    await client.connect(transport);
    const listed = await client.listTools();
    expect(listed.tools).toHaveLength(8);
    expect(listed.tools.map((tool) => tool.name)).toContain("selemene_vedic_chart");
    for (const tool of listed.tools) {
      expect(tool._meta?.securitySchemes).toEqual([{ type: "noauth" }]);
      expect(tool.description).not.toContain("mcp:calculate scope");
      expect(tool.inputSchema.additionalProperties).toBe(false);
    }
    expect(listed.tools.find((tool) => tool.name === "selemene_calculate")?.description).toContain(SHARED_STORAGE_DISCLOSURE);
    expect(upstream).not.toHaveBeenCalled();
    await client.close();
  });
  it("initializes and advertises both server-owned noauth scheme locations", async () => {
    expect((await rpc(init)).response.status).toBe(200);
    const listed = await rpc({ jsonrpc: "2.0", id: 2, method: "tools/list", params: { _meta: { progressToken: 1 } } });
    expect(listed.response.status).toBe(200);
    expect(listed.data.result.tools).toHaveLength(8);
    for (const tool of listed.data.result.tools) {
      expect(tool.securitySchemes).toEqual([{ type: "noauth" }]);
      expect(tool._meta).toEqual({ securitySchemes: [{ type: "noauth" }] });
    }
    expect(listed.response.headers.get("cache-control")).toBe("no-store");
  });
  it("accepts current SDK client metadata as ignored descriptive data", async () => {
    const initialized = await rpc({ ...init, params: { ...init.params, clientInfo: {
      name: "SDK client", version: "1.0", title: "Customer host",
      description: "Synthetic protocol client", websiteUrl: "https://example.test/client",
      icons: [{ src: "https://example.test/icon.png", mimeType: "image/png", sizes: ["48x48"], theme: "light" }],
    }, _meta: { ignored: "client metadata" } } });
    expect(initialized.response.status).toBe(200);
    expect(initialized.raw).not.toContain("https://example.test/client");
    expect(upstream).not.toHaveBeenCalled();
  });
  it("accepts a bounded tools/list cursor without changing the fixed single-page catalog", async () => {
    const listed = await rpc({ jsonrpc: "2.0", id: 2, method: "tools/list", params: {
      cursor: "opaque-client-cursor", _meta: { ignored: true },
    } });
    expect(listed.response.status).toBe(200);
    expect(listed.data.result.tools).toHaveLength(8);
    expect(listed.data.result.nextCursor).toBeUndefined();
    expect(upstream).not.toHaveBeenCalled();
  });
  it("allows only harmless initialized/cancelled lifecycle notifications", async () => {
    const initialized = await rpc({ jsonrpc: "2.0", method: "notifications/initialized", params: {} });
    expect(initialized.response.status).toBe(202);
    expect((await rpc({ jsonrpc: "2.0", method: "notifications/cancelled", params: { requestId: 1 } })).response.status).toBe(202);
    const notification = call("selemene_calculate", { engine_id: "tarot", options: { spread: "single_card" } });
    const { id: _id, ...withoutId } = notification;
    expect((await rpc(withoutId)).response.status).toBe(400);
    expect(upstream).not.toHaveBeenCalled();
  });
  it("rejects batches, extra envelope/operation fields and unsupported methods before upstream", async () => {
    for (const body of [
      [call("selemene_list_engines")], { ...init, api_key: SYNTHETIC_KEY },
      { jsonrpc: "2.0", id: 4, method: "admin/users", params: {} },
      { jsonrpc: "2.0", id: 4, method: "tools/call", params: { name: "selemene_list_engines", url: "https://evil.test" } },
    ]) expect((await rpc(body)).response.status).toBe(400);
    expect(upstream).not.toHaveBeenCalled();
  });
  it("rejects multibyte body oversize and wrong content type", async () => {
    const huge = JSON.stringify({ ...init, padding: "界".repeat(12000) });
    expect(huge.length).toBeLessThan(32768);
    expect((await rpc(huge)).response.status).toBe(413);
    const wrong = await worker.fetch(request(init, "203.0.113.8", { "content-type": "text/plain" }), fixture.env);
    expect(wrong.status).toBe(415);
    expect(upstream).not.toHaveBeenCalled();
  });
});
describe("free service tool boundaries", () => {
  it("discovers the reviewed catalog and prepares locally with shared account disclosure", async () => {
    const catalog = await rpc(call("selemene_list_engines"));
    expect(catalog.data.result.isError).not.toBe(true);
    expect(catalog.raw).toContain("numerology");
    upstream.mockClear();
    const prepared = await rpc(call("selemene_prepare_reading", { engine_id: "numerology" }));
    expect(prepared.data.result.structuredContent.can_calculate).toBe(false);
    expect(prepared.data.result.structuredContent.shared_storage_disclosure).toBe(SHARED_STORAGE_DISCLOSURE);
    expect(upstream).not.toHaveBeenCalled();
  });
  it("uses server-derived phase, sponsor header and fixed ordinary calculation path", async () => {
    const calculated = await rpc(call("selemene_calculate", { engine_id: "tarot", options: { spread: "single_card" } }));
    expect(calculated.response.status).toBe(200);
    expect(calculated.data.result.isError).not.toBe(true);
    expect(calculated.data.result.structuredContent.persistence_status).toBe("not_confirmed");
    expect(calculated.data.result.structuredContent.shared_storage_disclosure).toBe(SHARED_STORAGE_DISCLOSURE);
    expect(calculated.raw).toContain("The Star");
    const posts = upstream.mock.calls.filter(([, options]) => options?.method === "POST");
    expect(posts).toHaveLength(1);
    const body = JSON.parse(posts[0][1]!.body as string);
    expect(body.consciousness_level).toBe(5);
    expect(body.options).toEqual({ spread: "single_card" });
    expect(calculated.raw).not.toContain("consciousness_level");
    expect(calculated.raw).toContain("host ChatGPT");
  });
  it.each([
    ["selemene_admin", {}], ["selemene_witness", {}],
    ["selemene_calculate", { engine_id: "face-reading" }],
    ["selemene_calculate", { engine_id: "sigil-forge" }],
    ["selemene_calculate", { engine_id: "unknown" }],
    ["selemene_run_workflow", { workflow_id: "full-spectrum", options: { type: 1 } }],
    ["selemene_calculate", { engine_id: "tarot", consciousness_level: 5 }],
    ["selemene_calculate", { engine_id: "tarot", api_key: SYNTHETIC_KEY }],
    ["selemene_calculate", { engine_id: "tarot", url: "https://evil.test" }],
    ["selemene_calculate", { engine_id: "tarot", options: { admin: true } }],
  ])("denies %s %j without any upstream request", async (name, args) => {
    const response = await rpc(call(name as string, args as Record<string, unknown>));
    expect(response.data.result.isError).toBe(true);
    expect(upstream).not.toHaveBeenCalled();
  });
  it("fails closed on an invalid internal account phase without calculation", async () => {
    phase = 99;
    const response = await rpc(call("selemene_calculate", { engine_id: "tarot", options: { spread: "single_card" } }));
    expect(response.data.result.isError).toBe(true);
    expect(upstream).toHaveBeenCalledTimes(1);
    expect(String(upstream.mock.calls[0][0])).toContain("/api/v1/users/me");
    expect(upstream.mock.calls.filter(([, options]) => options?.method === "POST")).toHaveLength(0);
  });
  it("sanitizes sponsor 401/403 without an OAuth challenge or retry", async () => {
    for (const status of [401, 403, 503]) {
      responseStatus = status; upstream.mockClear();
      const response = await rpc(call("selemene_calculate", { engine_id: "tarot", options: { spread: "single_card" } }));
      expect(response.data.result.isError).toBe(true);
      expect(response.raw).not.toContain("mcp/www_authenticate");
      expect(response.raw).not.toContain("Reconnect");
      expect(upstream.mock.calls.filter(([, options]) => options?.method === "POST")).toHaveLength(1);
    }
  });
  it("redacts nested credential echoes and retains legitimate Human Design profile/authority", async () => {
    result = { output: { _auth: { user_id: "secret-account" }, token: "sensitive-token",
      profile: "2/4", authority: "Emotional", note: SYNTHETIC_KEY,
      redirect: "https://example.test/?access_token=sensitive-value",
      extra: { apiKey: SYNTHETIC_KEY, cookie: "sensitive-cookie", value: "kept" } } };
    const response = await rpc(call("selemene_calculate", { engine_id: "tarot", options: { spread: "single_card" } }));
    expect(response.raw).not.toContain("secret-account");
    expect(response.raw).not.toContain("sensitive-token");
    expect(response.raw).not.toContain("sensitive-cookie");
    expect(response.raw).not.toContain("sensitive-value");
    expect(response.raw).toContain("Emotional");
    expect(response.raw).toContain("2/4");
    expect(response.raw).toContain("kept");
  });
  it("rejects oversize upstream responses without leaking their body", async () => {
    result = { sensitive_body: "X".repeat(270000) };
    const response = await rpc(call("selemene_calculate", { engine_id: "tarot", options: { spread: "single_card" } }));
    expect(response.data.result.isError).toBe(true);
    expect(response.raw).not.toContain("X".repeat(100));
    expect(upstream.mock.calls.filter(([, options]) => options?.method === "POST")).toHaveLength(1);
  });
  it("returns atomic public 429 after calculation attempt budget and never makes a second POST", async () => {
    fixture = makeEnv({ GLOBAL_CALCULATIONS_PER_DAY: "1" });
    expect((await rpc(call("selemene_calculate", { engine_id: "tarot", options: { spread: "single_card" } }))).response.status).toBe(200);
    const denied = await rpc(call("selemene_calculate", { engine_id: "tarot", options: { spread: "single_card" } }));
    expect(denied.response.status).toBe(429);
    expect(Number(denied.response.headers.get("retry-after"))).toBeGreaterThan(0);
    expect(upstream.mock.calls.filter(([, options]) => options?.method === "POST")).toHaveLength(1);
  });
  it("fails closed without trusted edge IP, DO or valid service origins/secrets", async () => {
    for (const overrides of [
      { SELEMENE_API_ORIGIN: "https://evil.test" }, { SELEMENE_API_KEY: "" }, { RATE_LIMIT_SALT: "short" },
      { IP_REQUESTS_PER_MINUTE: "0" },
    ]) {
      const env = { ...fixture.env, ...overrides };
      const response = await worker.fetch(request(init), env);
      expect(response.status).toBe(503);
      expect(await response.text()).not.toContain(SYNTHETIC_KEY);
    }
    const incoming = request(init); incoming.headers.delete("cf-connecting-ip");
    expect((await worker.fetch(incoming, fixture.env)).status).toBe(503);
    fixture.env.FREE_RATE_LIMITER = { idFromName: () => { throw new Error(SYNTHETIC_KEY); } } as unknown as DurableObjectNamespace;
    expect((await rpc(init)).response.status).toBe(503);
    expect(upstream).not.toHaveBeenCalled();
  });
});
