import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import worker from "../src/index.js";
import { PUBLIC_ORIGIN, type Env } from "../src/env.js";
import { init, makeEnv, request } from "./helpers.js";

const CHALLENGE_PATH = "/.well-known/openai-apps-challenge";
const PUBLIC_TOKEN = "synthetic-public-domain-verification-token";
const upstream = vi.fn(() => { throw new Error("Unexpected upstream request"); });

function tokenOnlyEnv(token: unknown): Env {
  return new Proxy({}, { get(_target, field) {
    if (field === "OPENAI_APPS_CHALLENGE_TOKEN") return token;
    throw new Error("Challenge route accessed service configuration");
  } }) as Env;
}

beforeEach(() => { upstream.mockClear(); vi.stubGlobal("fetch", upstream); });
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks(); });

describe("public domain ownership challenge route", () => {
  it("serves only the trimmed public token without service configuration or trusted IP", async () => {
    const response = await worker.fetch(new Request(`${PUBLIC_ORIGIN}${CHALLENGE_PATH}`), tokenOnlyEnv(`  ${PUBLIC_TOKEN}\n`));
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("text/plain; charset=utf-8");
    expect(response.headers.get("cache-control")).toBe("no-store");
    expect(response.headers.get("location")).toBeNull();
    expect(await response.text()).toBe(PUBLIC_TOKEN);
    expect(upstream).not.toHaveBeenCalled();
  });

  it.each([undefined, "", " \n\t", 123])("returns 404 for an absent or invalid public token (%s)", async (token) => {
    const response = await worker.fetch(new Request(`${PUBLIC_ORIGIN}${CHALLENGE_PATH}`), tokenOnlyEnv(token));
    expect(response.status).toBe(404);
    expect(response.headers.get("cache-control")).toBe("no-store");
    expect(await response.text()).toBe("");
    expect(upstream).not.toHaveBeenCalled();
  });

  it.each(["POST", "HEAD", "PUT", "OPTIONS"])("rejects %s before even reading the public token", async (method) => {
    const env = new Proxy({}, { get() { throw new Error("Unexpected environment access"); } }) as Env;
    const response = await worker.fetch(new Request(`${PUBLIC_ORIGIN}${CHALLENGE_PATH}`, { method }), env);
    expect(response.status).toBe(405);
    expect(response.headers.get("allow")).toBe("GET");
    expect(response.headers.get("cache-control")).toBe("no-store");
    expect(await response.text()).toBe("");
    expect(upstream).not.toHaveBeenCalled();
  });

  it.each([`${CHALLENGE_PATH}/`, `${CHALLENGE_PATH}-extra`])("does not expose the token at a different path (%s)", async (path) => {
    const response = await worker.fetch(new Request(`${PUBLIC_ORIGIN}${path}`), tokenOnlyEnv(PUBLIC_TOKEN));
    expect(response.status).toBe(404);
    expect(await response.text()).toBe("");
    expect(upstream).not.toHaveBeenCalled();
  });

  it("makes no quota reservations for verification and preserves health/MCP handling", async () => {
    const fixture = makeEnv({ OPENAI_APPS_CHALLENGE_TOKEN: PUBLIC_TOKEN });
    expect((await worker.fetch(new Request(`${PUBLIC_ORIGIN}${CHALLENGE_PATH}`), fixture.env)).status).toBe(200);
    expect(fixture.fetchObject).not.toHaveBeenCalled();
    expect((await worker.fetch(new Request(`${PUBLIC_ORIGIN}/health`), fixture.env)).status).toBe(200);
    expect(fixture.fetchObject).not.toHaveBeenCalled();
    const healthPost = await worker.fetch(new Request(`${PUBLIC_ORIGIN}/health`, { method: "POST" }), fixture.env);
    expect(healthPost.status).toBe(405);
    expect(healthPost.headers.get("allow")).toBe("GET");
    const mcpGet = await worker.fetch(new Request(`${PUBLIC_ORIGIN}/mcp`), fixture.env);
    expect(mcpGet.status).toBe(405);
    expect(mcpGet.headers.get("allow")).toBe("POST");
    const initialized = await worker.fetch(request(init), fixture.env);
    expect(initialized.status).toBe(200);
    expect(fixture.fetchObject).toHaveBeenCalledTimes(1);
    expect(await initialized.text()).not.toContain(PUBLIC_TOKEN);
    expect(upstream).not.toHaveBeenCalled();
  });
});
