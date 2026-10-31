import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { WebStandardStreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/webStandardStreamableHttp.js";
import { CallToolRequestSchema, ListToolsRequestSchema, ImplementationSchema } from "@modelcontextprotocol/sdk/types.js";
import { zodToJsonSchema } from "zod-to-json-schema";
import { z } from "zod";
import { ALL_TOOLS, type ToolContext, type ToolRegistration, type ToolResult } from "../../selemene-mcp/src/tools/index.js";
import { ALLOWED_ENGINE_IDS, isEngineAllowed, isWorkflowAllowed, type AllowedEngineId } from "../../selemene-mcp/src/allowlist.js";
import { ENGINE_SCHEMAS, validateEngineInput } from "../../selemene-mcp/src/engine-schemas.js";
import { callSelemene, MAX_RESPONSE_BYTES, CALCULATE_TIMEOUT_MS } from "../../selemene-mcp/src/upstream.js";
import type { FreeContext } from "./env.js";
import type { RateDecision } from "./rate-limiter.js";
import { sanitizeValue, SHARED_STORAGE_DISCLOSURE } from "./sanitize.js";
import { VEDIC_CHART_TOOL } from "./vedic-chart.js";

/** The private compatibility surface. Never add public-only operations here. */
export const FREE_TOOLS: readonly ToolRegistration[] = [...ALL_TOOLS, VEDIC_CHART_TOOL];
const MAX_BYTES = 32 * 1024;
const INTERPRETATION = "Interpretation is composed in the user's current host ChatGPT conversation. This connector makes no hosted inference calls.";
const OPERATION_DESCRIPTIONS: Readonly<Record<string, string>> = {
  selemene_list_engines: "Return the reviewed Selemene engine catalog and capability/readiness metadata only. Catalog membership is not proof of operational readiness.",
  selemene_engine_info: "Return input requirements and metadata for exactly one reviewed engine selected by engine_id. Use these requirements to supply the required birth_data, options, and parameters.",
  selemene_prepare_reading: "Validate inputs locally for the selected reviewed engine_id and report missing or invalid fields. This tool makes no upstream call and executes or saves nothing. It uses the same input validation rules as selemene_calculate.",
  selemene_calculate: "Execute exactly one calculation for the selected reviewed engine_id using its validated input.",
};
type PublishedTool = ToolRegistration & { publishedInputSchema?: Record<string, unknown> };
type McpSurface = "private" | "public";
const legacyByName = new Map(FREE_TOOLS.map((tool) => [tool.name, tool]));
const noInput = z.object({}).strict();
const minimalDate = z.string().regex(/^\d{4}-\d{2}-\d{2}$/).refine((value) => {
  const date = new Date(value + "T00:00:00Z");
  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value && Number(value.slice(0, 4)) >= 1000 && Number(value.slice(0, 4)) <= 3000;
});
const MINIMAL_PUBLIC_SCHEMAS: Partial<Record<AllowedEngineId, z.AnyZodObject>> = {
  numerology: z.object({ name: z.string().trim().min(1).max(500), birth_date: minimalDate }).strict(),
  biorhythm: z.object({ birth_date: minimalDate, current_time: z.string().datetime({ offset: true }) }).strict(),
  "vedic-clock": z.object({ current_time: z.string().datetime({ offset: true }), timezone_offset: z.number().int().min(-780).max(780) }).strict(),
};
const engineInvocation = z.object({
  birth_data: z.record(z.unknown()).optional(),
  current_time: z.string().max(64).optional(),
  location: z.record(z.unknown()).optional(),
  parameters: z.record(z.unknown()).optional(),
  options: z.record(z.unknown()).optional(),
  question: z.string().max(2000).optional(),
}).strict();

function success(data: Record<string, unknown>): ToolResult {
  return { content: [{ type: "text", text: JSON.stringify(data) }], structuredContent: data };
}

function publicEngineSchema(engineId: AllowedEngineId): Record<string, unknown> {
  const schema = zodToJsonSchema(ENGINE_SCHEMAS[engineId].zodSchema, { $refStrategy: "none" }) as Record<string, unknown>;
  const properties = schema.properties;
  if (properties && typeof properties === "object" && !Array.isArray(properties)) {
    const withoutEngine = { ...(properties as Record<string, unknown>) };
    delete withoutEngine.engine_id;
    schema.properties = withoutEngine;
  }
  if (Array.isArray(schema.required)) schema.required = schema.required.filter((key) => key !== "engine_id");
  return schema;
}

function requiredLegacy(name: string): ToolRegistration {
  const tool = legacyByName.get(name);
  if (!tool) throw new Error(`Missing legacy registration: ${name}`);
  return tool;
}

function fixedEngineTool(
  engineId: AllowedEngineId,
  operation: "info" | "prepare" | "calculate",
): PublishedTool {
  const legacyName = operation === "info"
    ? "selemene_engine_info"
    : operation === "prepare"
      ? "selemene_prepare_reading"
      : "selemene_calculate";
  const legacy = requiredLegacy(legacyName);
  const suffix = engineId.replace(/-/g, "_");
  const title = ENGINE_SCHEMAS[engineId].title;
  const name = `selemene_${suffix}_${operation}`;
  const minimal = MINIMAL_PUBLIC_SCHEMAS[engineId];
  if (minimal) {
    const schema = operation === "info" ? noInput : operation === "prepare" ? minimal.partial() : minimal;
    return {
      ...legacy, name,
      description: operation === "info" ? `Return the minimal input requirements for ${title}.`
        : operation === "prepare" ? `Check supplied ${title} inputs locally and report missing fields without calculating.`
        : `Calculate ${title} using only ${engineId === "numerology" ? "name and birth date" : engineId === "biorhythm" ? "birth date and reference time" : "reference time and explicit UTC offset in minutes"}. No birth location is collected.`,
      inputSchema: schema,
      annotations: { ...legacy.annotations, title: `${operation === "info" ? "Inspect" : operation === "prepare" ? "Prepare" : "Calculate"} ${title}` },
      handler: async (ctx, args) => {
        if (operation === "info") return success({ engine_id: engineId, input_schema: zodToJsonSchema(minimal), operations: { prepare: `selemene_${suffix}_prepare`, calculate: `selemene_${suffix}_calculate` } });
        const checked = minimal.safeParse(args);
        if (operation === "prepare") return success({ engine_id: engineId, valid: checked.success, can_calculate: checked.success,
          missing: checked.success ? [] : checked.error.issues.filter((issue) => issue.code === "invalid_type" && issue.received === "undefined").map((issue) => issue.path.join(".")),
          errors: checked.success ? [] : checked.error.issues.filter((issue) => !(issue.code === "invalid_type" && issue.received === "undefined")).map((issue) => issue.path.join(".") + ": invalid input") });
        if (!checked.success) return failed("Invalid tool input.");
        const { current_time, ...options } = checked.data;
        const outcome = await callSelemene(ctx.env, { method: "POST", path: { kind: "engine_calculate", engineId }, apiKey: ctx.props.apiKey, timeoutMs: CALCULATE_TIMEOUT_MS,
          body: { contract_version: "v1", consciousness_level: ctx.props.consciousnessLevel, parameters: {}, options,
            current_time: current_time ?? new Date().toISOString() } });
        if (outcome.kind !== "ok") return failed("The sponsored calculation failed. Do not retry automatically.");
        return success({ engine_id: engineId, persistence_status: "not_confirmed", result: outcome.data });
      },
    };
  }
  const description = operation === "info"
    ? `Return requirements and reviewed metadata for the fixed ${title} engine. No engine selector is accepted.`
    : operation === "prepare"
      ? `Validate only ${title} inputs locally and report missing or invalid fields. This tool does not calculate or save.`
      : `Run exactly one ${title} calculation with validated ${title}-specific input.`;
  return {
    ...legacy,
    name,
    description,
    inputSchema: operation === "info" ? noInput : engineInvocation,
    publishedInputSchema: operation === "info"
      ? zodToJsonSchema(noInput, { $refStrategy: "none" }) as Record<string, unknown>
      : publicEngineSchema(engineId),
    annotations: { ...legacy.annotations, title: `${operation === "info" ? "Inspect" : operation === "prepare" ? "Prepare" : "Calculate"} ${title}` },
    handler: async (ctx, args) => legacy.handler(ctx, { engine_id: engineId, ...args }),
  };
}

function catalogEngineIds(data: unknown): Set<string> {
  if (!data || typeof data !== "object") return new Set();
  const engines = (data as { engines?: unknown }).engines;
  if (!Array.isArray(engines)) return new Set();
  return new Set(engines.map((entry) => typeof entry === "string" ? entry : entry && typeof entry === "object" ? (entry as { id?: unknown }).id : null).filter((id): id is string => typeof id === "string"));
}

function capabilityValue(data: unknown, engineId: string): Record<string, unknown> | null {
  const root = data && typeof data === "object" ? data as Record<string, unknown> : null;
  if (!root) return null;
  const candidates = Array.isArray(root.capabilities) ? root.capabilities : Array.isArray(root.engines) ? root.engines : null;
  if (candidates) {
    const row = candidates.find((entry) => entry && typeof entry === "object" && ((entry as Record<string, unknown>).engine_id === engineId || (entry as Record<string, unknown>).id === engineId));
    return row && typeof row === "object" ? row as Record<string, unknown> : null;
  }
  const direct = root[engineId];
  return direct && typeof direct === "object" && !Array.isArray(direct) ? direct as Record<string, unknown> : null;
}

const engineCatalogTool: PublishedTool = {
  name: "selemene_engine_catalog",
  description: "Return the reviewed Selemene engine catalog only. Catalog membership does not prove operational readiness.",
  inputSchema: noInput,
  annotations: { title: "Engine catalog", readOnlyHint: true, idempotentHint: true, destructiveHint: false, openWorldHint: false },
  requiredScope: "read",
  securitySchemes: { type: "noauth" },
  handler: async (ctx) => {
    const outcome = await callSelemene(ctx.env, { method: "GET", path: "/api/v1/engines", apiKey: ctx.props.apiKey });
    if (outcome.kind !== "ok") return failed("The sponsored Selemene catalog is unavailable. Try again later.");
    const present = catalogEngineIds(outcome.data);
    return success({ engines: ALLOWED_ENGINE_IDS.map((engineId) => ({
      engine_id: engineId,
      title: ENGINE_SCHEMAS[engineId].title,
      summary: ENGINE_SCHEMAS[engineId].summary,
      catalog_present: present.has(engineId),
    })), note: "Catalog membership does not imply operational readiness. Use selemene_engine_capabilities for current capability metadata." });
  },
};

const engineCapabilitiesTool: PublishedTool = {
  name: "selemene_engine_capabilities",
  description: "Return current capability/readiness metadata for reviewed Selemene engines only. Readiness is distinct from catalog membership.",
  inputSchema: noInput,
  annotations: { title: "Engine capabilities", readOnlyHint: true, idempotentHint: true, destructiveHint: false, openWorldHint: false },
  requiredScope: "read",
  securitySchemes: { type: "noauth" },
  handler: async (ctx) => {
    const outcome = await callSelemene(ctx.env, { method: "GET", path: "/api/v1/engines/capabilities", apiKey: ctx.props.apiKey });
    if (outcome.kind === "not_found") return success({ capabilities_source: "unknown", engines: ALLOWED_ENGINE_IDS.map((engine_id) => ({ engine_id, capability: null })) });
    if (outcome.kind !== "ok") return failed("The sponsored Selemene capability service is unavailable. Try again later.");
    return success({ capabilities_source: "ok", engines: ALLOWED_ENGINE_IDS.map((engine_id) => ({ engine_id, capability: capabilityValue(outcome.data, engine_id) })) });
  },
};

export const PUBLIC_TOOLS: readonly PublishedTool[] = [
  engineCatalogTool,
  engineCapabilitiesTool,
  ...ALLOWED_ENGINE_IDS.flatMap((engineId) => [
    fixedEngineTool(engineId, "info"),
    fixedEngineTool(engineId, "prepare"),
    fixedEngineTool(engineId, "calculate"),
  ]),
  requiredLegacy("selemene_list_workflows"),
  requiredLegacy("selemene_workflow_info"),
  requiredLegacy("selemene_run_workflow"),
  requiredLegacy("selemene_vedic_chart"),
];
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
  if (tool.name === "selemene_prepare_reading" || tool.name.endsWith("_prepare")) return "For a later calculation only: " + SHARED_STORAGE_DISCLOSURE;
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

export async function handleMcp(request: Request, ctx: McpContext, surface: McpSurface = "private"): Promise<Response> {
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

  const tools: readonly PublishedTool[] = surface === "public" ? PUBLIC_TOOLS : FREE_TOOLS;
  const server = new Server({ name: "selemene-free-mcp", version: surface === "public" ? "0.2.5" : "0.2.0" }, { capabilities: { tools: {} } });
  const schemes = [{ type: "noauth" }];
  let rateDenied: RateDecision | undefined;
  server.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: tools.map((tool) => ({
      name: tool.name,
      description: publicDescription(tool),
      inputSchema: (tool.publishedInputSchema ?? zodToJsonSchema(tool.inputSchema, { $refStrategy: "none" })) as { type: "object"; [key: string]: unknown },
      annotations: tool.annotations, securitySchemes: schemes, _meta: { securitySchemes: schemes },
    })),
  }));
  server.setRequestHandler(CallToolRequestSchema, async ({ params: toolParams }) => {
    const tool = tools.find((item) => item.name === toolParams.name);
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
        if (tools.some((known) => known.name === tool.name)) {
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
