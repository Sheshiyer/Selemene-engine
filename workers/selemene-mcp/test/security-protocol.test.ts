import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { handleMcp, type McpRequestContext } from "../src/mcp.js";

// Exercise the real SDK transport/serialization. All identities are synthetic;
// unexpected upstream traffic fails locally rather than reaching the network.
const origin = "https://connector.example.test";
const context: McpRequestContext = {
  env: {
    publicOrigin: origin,
    mcpResource: `${origin}/mcp`,
    authorizeEndpoint: `${origin}/authorize`,
    tokenEndpoint: `${origin}/oauth/token`,
    clientRegistrationEndpoint: `${origin}/oauth/register`,
    selemeneApiOrigin: "https://upstream.example.test",
  },
  props: {
    userId: "00000000-0000-4000-8000-000000000001",
    tier: "free",
    consciousnessLevel: 0,
    apiKey: "nk_synthetic_protocol_test_only",
    linkedAt: 1,
  },
  scope: ["mcp:read", "mcp:calculate"],
};

interface ToolDescriptor {
  name: string;
  inputSchema: { type: string };
  annotations: {
    readOnlyHint: boolean;
    destructiveHint: boolean;
    openWorldHint: boolean;
  };
  securitySchemes: Array<{ type: string; scopes: string[] }>;
  _meta: { securitySchemes: ToolDescriptor["securitySchemes"] };
}

interface RpcResponse {
  jsonrpc: string;
  id: number | null;
  result?: {
    protocolVersion?: string;
    serverInfo?: { name: string };
    capabilities?: { tools?: object };
    tools?: ToolDescriptor[];
    isError?: boolean;
    _meta?: { "mcp/www_authenticate"?: string[] };
  };
  error?: { code: number; message: string };
}

const upstream = vi.fn(async () => {
  throw new Error("Unexpected upstream request in protocol boundary test");
});

function request(raw: string): Request {
  return new Request(context.env.mcpResource, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      accept: "application/json, text/event-stream",
      "mcp-protocol-version": "2025-03-26",
    },
    body: raw,
  });
}

async function rpc(body: unknown, scope = context.scope) {
  const response = await handleMcp(request(JSON.stringify(body)), {
    ...context,
    scope,
  });
  const raw = await response.text();
  expect(raw).not.toContain(context.props.apiKey);
  expect(raw).not.toContain(context.props.userId);
  return { response, data: JSON.parse(raw) as RpcResponse };
}

const calculate = {
  jsonrpc: "2.0",
  id: 10,
  method: "tools/call",
  params: {
    name: "selemene_calculate",
    arguments: { engine_id: "numerology", input: { name: "Synthetic Person" } },
  },
};

beforeEach(() => {
  upstream.mockClear();
  vi.stubGlobal("fetch", upstream);
});
afterEach(() => vi.unstubAllGlobals());

describe("MCP SDK protocol security boundaries", () => {
  it("initializes and serializes seven tools with both security scheme locations", async () => {
    const initialized = await rpc({
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        protocolVersion: "2025-03-26",
        capabilities: {},
        clientInfo: { name: "synthetic-security-test", version: "1.0.0" },
      },
    });
    expect(initialized.response.status).toBe(200);
    expect(initialized.data.error).toBeUndefined();
    expect(initialized.data.result).toMatchObject({
      protocolVersion: "2025-03-26",
      serverInfo: { name: "selemene-mcp" },
      capabilities: { tools: {} },
    });

    const listed = await rpc({ jsonrpc: "2.0", id: 2, method: "tools/list", params: {} });
    expect(listed.response.status).toBe(200);
    expect(listed.data.error).toBeUndefined();
    expect(listed.response.headers.get("cache-control")).toBe("no-store");
    const tools = listed.data.result?.tools;
    expect(tools).toHaveLength(7);
    expect(tools?.map((tool) => tool.name).sort()).toEqual([
      "selemene_calculate", "selemene_engine_info", "selemene_list_engines",
      "selemene_list_workflows", "selemene_prepare_reading",
      "selemene_run_workflow", "selemene_workflow_info",
    ]);
    for (const tool of tools ?? []) {
      const writes = ["selemene_calculate", "selemene_run_workflow"].includes(tool.name);
      expect(tool.inputSchema.type).toBe("object");
      expect(tool.securitySchemes).toEqual([{
        type: "oauth2", scopes: writes ? ["mcp:read", "mcp:calculate"] : ["mcp:read"],
      }]);
      expect(tool._meta.securitySchemes).toEqual(tool.securitySchemes);
      expect(tool.annotations).toMatchObject({
        readOnlyHint: !writes, destructiveHint: false, openWorldHint: false,
      });
    }
    expect(upstream).not.toHaveBeenCalled();
  });

  it.each([
    { label: "read-only", scope: ["mcp:read"] },
    { label: "calculate-only", scope: ["mcp:calculate"] },
    { label: "neither", scope: [] },
  ])("denies $label calculation with one complete challenge and no upstream request", async ({ scope }) => {
    const { response, data } = await rpc(calculate, scope);
    expect(response.status).toBe(200);
    expect(data.result?.isError).toBe(true);
    const challenges = data.result?._meta?.["mcp/www_authenticate"];
    expect(challenges).toHaveLength(1);
    expect(challenges?.[0]).toMatch(/^Bearer /);
    expect(challenges?.[0]).toContain(`resource_metadata="${origin}/.well-known/oauth-protected-resource/mcp"`);
    expect(challenges?.[0]).toContain('error="insufficient_scope"');
    expect(challenges?.[0]).toContain('scope="mcp:read mcp:calculate"');
    expect(upstream).not.toHaveBeenCalled();
  });

  it("rejects a batch before any upstream dispatch", async () => {
    const { response, data } = await rpc([calculate, { ...calculate, id: 11 }]);
    expect(response.status).toBe(400);
    expect(data.error?.code).toBe(-32600);
    expect(upstream).not.toHaveBeenCalled();
  });

  it("rejects a tool-call notification before any upstream dispatch", async () => {
    const { response, data } = await rpc({
      jsonrpc: calculate.jsonrpc, method: calculate.method, params: calculate.params,
    });
    expect(response.status).toBe(400);
    expect(data.error?.code).toBe(-32600);
    expect(upstream).not.toHaveBeenCalled();
  });

  it("enforces the byte bound for multibyte JSON without Content-Length", async () => {
    const raw = JSON.stringify({ ...calculate, padding: "界".repeat(12_000) });
    expect(raw.length).toBeLessThan(32_768);
    expect(new TextEncoder().encode(raw).byteLength).toBeGreaterThan(32_768);
    const incoming = request(raw);
    expect(incoming.headers.has("content-length")).toBe(false);
    const response = await handleMcp(incoming, context);
    expect(response.status).toBe(413);
    const data = await response.json() as RpcResponse;
    expect(data.error?.code).toBe(-32600);
    expect(upstream).not.toHaveBeenCalled();
  });
});
