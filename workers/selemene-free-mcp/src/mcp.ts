import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { WebStandardStreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/webStandardStreamableHttp.js";
import { CallToolRequestSchema, ListToolsRequestSchema, ImplementationSchema } from "@modelcontextprotocol/sdk/types.js";
import { zodToJsonSchema } from "zod-to-json-schema";
import { z } from "zod";
import { ALL_TOOLS, type ToolContext, type ToolRegistration, type ToolResult } from "../../selemene-mcp/src/tools/index.js";
import { isEngineAllowed, isWorkflowAllowed } from "../../selemene-mcp/src/allowlist.js";
import { validateEngineInput } from "../../selemene-mcp/src/engine-schemas.js";
import { callSelemene, MAX_RESPONSE_BYTES } from "../../selemene-mcp/src/upstream.js";
import type { FreeContext } from "./env.js";
import type { RateDecision } from "./rate-limiter.js";
import { sanitizeValue, SHARED_STORAGE_DISCLOSURE } from "./sanitize.js";
import { VEDIC_CHART_TOOL } from "./vedic-chart.js";

export const FREE_TOOLS: readonly ToolRegistration[] = [...ALL_TOOLS, VEDIC_CHART_TOOL];
const MAX_BYTES = 32 * 1024;
const INTERPRETATION = "Interpretation is composed in the user's current host ChatGPT conversation. This connector makes no hosted inference calls.";
const OPERATION_DESCRIPTIONS: Readonly<Record<string, string>> = {
  selemene_list_engines: "Return the reviewed Selemene engine catalog and capability/readiness metadata only. Catalog membership is not proof of operational readiness.",
  selemene_engine_info: "Return input requirements and metadata for exactly one reviewed engine selected by engine_id. Use these requirements to supply the required birth_data, options, and parameters.",
  selemene_prepare_reading: "Validate inputs locally for the selected reviewed engine_id and report missing or invalid fields. This tool makes no upstream call and executes or saves nothing. It uses the same input validation rules as selemene_calculate.",
  selemene_calculate: "Execute exactly one calculation for the selected reviewed engine_id using its validated input.",
};
const error = (status: number, message: string, id: string | number | null = null, code = -32600) =>
  Response.json({ jsonrpc: "2.0", id, error: { code, message } }, { status, headers: { "cache-control": "no-store" } });
function failed(message: string): ToolResult {
  return { isError: true, content: [{ type: "text", text: message }] };
}
export interface McpContext { service: FreeContext; reserveCalculation: () => Promise<RateDecision> }
const idSchema = z.union([z.string().max(128), z.number().finite()]);
const envelope = z.object({
  jsonrpc: z.literal("2.0"), id: idSchema.optional(), method: z.string().max(64),
  params: z.record(z.unknown()).optional(),
}).strict();
// Standard MCP request metadata is accepted and ignored; it never supplies credentials,
// phase, destinations, tool input or rate-limit identity.
const requestMeta = z.record(z.unknown()).optional();
const noParams = z.object({ _meta: requestMeta }).strict();
const listParams = z.object({ cursor: z.string().max(256).optional(), _meta: requestMeta }).strict();
const callParams = z.object({ name: z.string().min(1).max(64), arguments: z.record(z.unknown()).optional(), _meta: requestMeta }).strict();
// The current SDK uses Zod 4 internally; adapter inputs use the reviewed Zod 3
// contracts. Validate across this boundary instead of mixing schema instances.
const clientImplementation = z.unknown().refine((value) => {
  const checked = ImplementationSchema.strict().safeParse(value);
  return checked.success && checked.data.name.length <= 128 && checked.data.version.length <= 64;
}, "Invalid client implementation.");
const initializeParams = z.object({
  protocolVersion: z.string().min(1).max(32),
  capabilities: z.record(z.unknown()),
  clientInfo: clientImplementation,
  _meta: requestMeta,
}).strict();
const cancellationParams = z.object({ requestId: idSchema, reason: z.string().max(256).optional() }).strict();

async function readBody(request: Request): Promise<Uint8Array | Response> {
  const reader = request.body?.getReader();
  if (!reader) return error(400, "Invalid request body.");
  const chunks: Uint8Array[] = []; let size = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<never>((_, reject) => { timer = setTimeout(() => reject(new Error("timeout")), 15000); });
  try {
    while (true) {
      const { value, done } = await Promise.race([reader.read(), timeout]);
      if (done) break;
      size += value.byteLength;
      if (size > MAX_BYTES) { void reader.cancel().catch(() => {}); return error(413, "Request too large."); }
      chunks.push(value);
    }
    const bytes = new Uint8Array(size); let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    return bytes;
  } catch { void reader.cancel().catch(() => {}); return error(400, "Invalid request body."); }
  finally { if (timer !== undefined) clearTimeout(timer); }
}
async function serviceProps(service: FreeContext): Promise<ToolContext["props"] | null> {
  const outcome = await callSelemene(service.upstream, { method: "GET", path: "/api/v1/users/me", apiKey: service.apiKey });
  if (outcome.kind !== "ok" || !outcome.data || typeof outcome.data !== "object") return null;
  const data = outcome.data as Record<string, unknown>, phase = data.consciousness_level;
  if (typeof phase !== "number" || !Number.isInteger(phase) || phase < 0 || phase > 5 ||
      typeof data.tier !== "string") return null;
  // The real account ID/email never enter tool context or the public response.
  return { userId: "shared-service", tier: data.tier, consciousnessLevel: phase,
    apiKey: service.apiKey, linkedAt: 0 };
}
function storageDisclosure(tool: ToolRegistration): string | undefined {
  if (tool.name === "selemene_prepare_reading") return "For a later calculation only: " + SHARED_STORAGE_DISCLOSURE;
  return tool.annotations.readOnlyHint === false ? SHARED_STORAGE_DISCLOSURE : undefined;
}
function publicDescription(tool: ToolRegistration): string {
  const operation = OPERATION_DESCRIPTIONS[tool.name] ?? tool.description.replace(/Requires mcp:[a-z]+ scope\./g, "").trim();
  const disclosure = storageDisclosure(tool);
  return operation + (disclosure ? " " + disclosure : "") + " " + INTERPRETATION;
}
function publicResult(result: ToolResult, tool: ToolRegistration, secret: string): ToolResult {
  if (result._meta?.["mcp/www_authenticate"]) result = failed("The sponsored Selemene service is unavailable. Try again later.");
  const denied = result.content.some((part) => {
    try { return JSON.parse(part.text).error === "upstream_permission_denied"; } catch { return false; }
  });
  if (denied) result = failed("The sponsored Selemene service cannot run this calculation.");
  const clean = sanitizeValue(result, secret) as ToolResult;
  const disclosure = storageDisclosure(tool);
  const structured = { ...(clean.structuredContent ?? {}), interpretation_policy: INTERPRETATION,
    ...(disclosure ? { shared_storage_disclosure: disclosure } : {}) };
  clean.structuredContent = structured;
  clean.content = [...clean.content, { type: "text", text: JSON.stringify({
    interpretation_policy: INTERPRETATION,
    ...(disclosure ? { shared_storage_disclosure: disclosure } : {}),
  }) }];
  return clean;
}

export async function handleMcp(request: Request, ctx: McpContext): Promise<Response> {
  if (request.method !== "POST") return new Response(null, { status: 405, headers: { Allow: "POST" } });
  if (request.headers.get("content-type")?.split(";")[0].trim().toLowerCase() !== "application/json") return error(415, "Expected JSON.");
  const origin = request.headers.get("origin");
  if (origin && ![ctx.service.upstream.publicOrigin, "https://chatgpt.com", "https://chat.openai.com"].includes(origin)) return error(403, "Origin denied.");
  const bytes = await readBody(request);
  if (bytes instanceof Response) return bytes;
  let unknownBody: unknown;
  try { unknownBody = JSON.parse(new TextDecoder().decode(bytes)); } catch { return error(400, "Invalid JSON."); }
  const checked = envelope.safeParse(unknownBody);
  if (!checked.success) return error(400, "One valid operation per request is required.");
  const body = checked.data;
  if (body.id === undefined) {
    if (body.method === "notifications/initialized" && noParams.safeParse(body.params ?? {}).success) return new Response(null, { status: 202, headers: { "cache-control": "no-store" } });
    if (body.method === "notifications/cancelled" && cancellationParams.safeParse(body.params ?? {}).success) return new Response(null, { status: 202, headers: { "cache-control": "no-store" } });
    return error(400, "Tool calls and protocol operations require an id.");
  }
  const params = body.params ?? {};
  if (body.method === "initialize") {
    if (!initializeParams.safeParse(params).success) return error(400, "Invalid initialization.", body.id);
  } else if (body.method === "tools/list") {
    if (!listParams.safeParse(params).success) return error(400, "Invalid operation parameters.", body.id);
  } else if (body.method === "ping") {
    if (!noParams.safeParse(params).success) return error(400, "Invalid operation parameters.", body.id);
  } else if (body.method === "tools/call") {
    if (!callParams.safeParse(params).success) return error(400, "Invalid tool request.", body.id);
  } else return error(400, "Unsupported operation.", body.id, -32601);

  const server = new Server({ name: "selemene-free-mcp", version: "0.2.0" }, { capabilities: { tools: {} } });
  const schemes = [{ type: "noauth" }];
  let rateDenied: RateDecision | undefined;
  server.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: FREE_TOOLS.map((tool) => ({
      name: tool.name,
      description: publicDescription(tool),
      inputSchema: zodToJsonSchema(tool.inputSchema, { $refStrategy: "none" }) as { type: "object"; [key: string]: unknown },
      annotations: tool.annotations, securitySchemes: schemes, _meta: { securitySchemes: schemes },
    })),
  }));
  server.setRequestHandler(CallToolRequestSchema, async ({ params: toolParams }) => {
    const tool = FREE_TOOLS.find((item) => item.name === toolParams.name);
    if (!tool) return failed("Unknown tool.");
    const input = tool.inputSchema.safeParse(toolParams.arguments ?? {});
    if (!input.success) return publicResult(failed("Invalid tool input."), tool, ctx.service.apiKey);
    const args = input.data as Record<string, unknown>;
    if (typeof args.engine_id === "string" && !isEngineAllowed(args.engine_id)) return publicResult(failed("Engine is not in the reviewed allowlist."), tool, ctx.service.apiKey);
    if (typeof args.workflow_id === "string" && !isWorkflowAllowed(args.workflow_id)) return publicResult(failed("Workflow is not in the reviewed allowlist."), tool, ctx.service.apiKey);
    if (tool.name === "selemene_calculate" && typeof args.engine_id === "string" && isEngineAllowed(args.engine_id) &&
        !validateEngineInput(args.engine_id, args).valid) return publicResult(failed("Input validation failed."), tool, ctx.service.apiKey);
    const scope = { read: true, calculate: true };
    let props: ToolContext["props"] = { userId: "shared-service", tier: "service",
      consciousnessLevel: 0, apiKey: ctx.service.apiKey, linkedAt: 0 };
    if (tool.requiredScope === "calculate") {
      // Count sponsored attempts only after known-tool and outer/input validation.
      // Backend unavailability or later workflow preflight failure does not refund.
      const decision = await ctx.reserveCalculation();
      if (!decision.allowed) { rateDenied = decision; return failed("Free service quota reached. Try again later."); }
      const trusted = await serviceProps(ctx.service);
      if (!trusted) return publicResult(failed("The sponsored Selemene service is unavailable. Try again later."), tool, ctx.service.apiKey);
      props = trusted;
    }
    try {
      const result = await tool.handler({ env: ctx.service.upstream, props, scope }, args);
      return publicResult(result, tool, ctx.service.apiKey);
    } catch { return publicResult(failed("Calculation service failed. Try again later."), tool, ctx.service.apiKey); }
  });
  const transport = new WebStandardStreamableHTTPServerTransport({
    sessionIdGenerator: undefined, enableJsonResponse: true, maxRequestBodySize: MAX_BYTES,
  });
  try {
    await server.connect(transport);
    const headers = new Headers(request.headers); headers.delete("content-length");
    const forwarded = new Request(request.url, { method: "POST", headers, body: bytes });
    const response = await transport.handleRequest(forwarded, { parsedBody: body });
    if (rateDenied) return Response.json({
      jsonrpc: "2.0", id: body.id, error: { code: -32029, message: "Free service quota reached. Try again later." },
    }, { status: rateDenied.reason === "unavailable" ? 503 : 429, headers: { "cache-control": "no-store",
      "retry-after": String(rateDenied.retryAfter) } });
    // SDK responses, including protocol errors, pass through the same bounded sanitizer.
    const raw = await response.text();
    if (new TextEncoder().encode(raw).byteLength > MAX_RESPONSE_BYTES) return error(502, "Response exceeded service limit.", body.id);
    let safe: unknown;
    try { safe = sanitizeValue(JSON.parse(raw), ctx.service.apiKey); }
    catch { return error(502, "Invalid service response.", body.id); }
    // Restore only our own static discovery metadata, never upstream _meta.
    // MCP clients preserve this standard location even when top-level extras are stripped.
    if (body.method === "tools/list" && safe && typeof safe === "object") {
      const listed = (safe as { result?: { tools?: Array<Record<string, unknown>> } }).result?.tools;
      for (const tool of listed ?? []) {
        if (FREE_TOOLS.some((known) => known.name === tool.name)) {
          tool.securitySchemes = schemes;
          tool._meta = { securitySchemes: schemes };
        }
      }
    }
    const safeRaw = JSON.stringify(safe);
    if (new TextEncoder().encode(safeRaw).byteLength > MAX_RESPONSE_BYTES) return error(502, "Response exceeded service limit.", body.id);
    const outputHeaders = new Headers(response.headers); outputHeaders.set("cache-control", "no-store");
    outputHeaders.set("content-type", "application/json"); outputHeaders.delete("content-length");
    return new Response(safeRaw, { status: response.status, headers: outputHeaders });
  } catch { return error(500, "Protocol operation failed.", body.id); }
  finally { await server.close().catch(() => {}); }
}
