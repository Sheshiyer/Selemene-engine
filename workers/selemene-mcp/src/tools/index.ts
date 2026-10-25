// Tool registry for Selemene MCP — strict validation, SDK-compatible.
//
// Each tool uses per-engine Zod schemas from engine-schemas.ts.
// Same validation applies to prepare_reading and calculate.
// Upstream 401 → isError:true + _meta['mcp/www_authenticate'] challenge.
// Upstream 403 → isError:true with permission denied message.
// All tools: readOnlyHint correct, idempotentHint false for writes,
// destructiveHint false, openWorldHint false (bounded account surface).
// All require mcp:read baseline; writes additionally require mcp:calculate.

import { z } from "zod";
import { zodToJsonSchema } from "zod-to-json-schema";
import type { ValidatedEnv } from "../env.js";
import type { SelemeneAuthProps, Scope } from "../props.js";
import {
  ALLOWED_ENGINE_IDS,
  ALLOWED_WORKFLOW_IDS,
  ALLOWED_WORKFLOW_ENGINES,
  denyReason,
  isEngineAllowed,
  isWorkflowAllowed,
} from "../allowlist.js";
import {
  ENGINE_SCHEMAS,
  CALCULATION_SIDE_EFFECTS,
  validateEngineInput,
  type AllowedEngineId,
} from "../engine-schemas.js";
import {
  CALCULATE_TIMEOUT_MS,
  callSelemene,
  SelemeneInputError,
  upstreamReauthResult,
  upstreamForbiddenResult,
  type UpstreamOutcome,
} from "../upstream.js";

// ---- Types -----------------------------------------------------------------

export interface ToolAnnotations {
  title?: string;
  readOnlyHint?: boolean;
  idempotentHint?: boolean;
  destructiveHint?: boolean;
  openWorldHint?: boolean;
}

export interface ToolResult {
  content: Array<{ type: "text"; text: string }>;
  structuredContent?: Record<string, unknown>;
  isError?: boolean;
  _meta?: Record<string, unknown>;
}

export interface ToolContext {
  env: ValidatedEnv;
  props: SelemeneAuthProps;
  scope: Scope;
}

// ---- Helpers ---------------------------------------------------------------

function ok(data: Record<string, unknown>): ToolResult {
  return {
    content: [{ type: "text", text: JSON.stringify(data, null, 2) }],
    structuredContent: data,
  };
}

function fail(message: string, extra?: Record<string, unknown>): ToolResult {
  const payload = { error: message, ...(extra ?? {}) };
  return {
    isError: true,
    content: [{ type: "text", text: JSON.stringify(payload, null, 2) }],
    structuredContent: payload,
  };
}

function toolFromOutcome(
  outcome: UpstreamOutcome,
  resource: string,
): ToolResult | null {
  if (outcome.kind === "ok") return null;
  if (outcome.kind === "unauthorized")
    return upstreamReauthResult(resource, outcome);
  if (outcome.kind === "forbidden") return upstreamForbiddenResult(outcome);
  const extras: Record<string, unknown> = { kind: outcome.kind };
  if ("status" in outcome) extras.status = outcome.status;
  return fail(outcome.sanitized, extras);
}

// ---- Zod schemas for tool inputs -------------------------------------------

const ListEnginesInput = z
  .object({
    include_denied: z.boolean().optional(),
  })
  .strict();

const EngineInfoInput = z
  .object({
    engine_id: z.string().min(1).max(64),
  })
  .strict();

const PrepareReadingInput = z
  .object({
    engine_id: z.string().min(1).max(64),
    birth_data: z.record(z.unknown()).optional(),
    current_time: z.string().optional(),
    location: z.record(z.unknown()).optional(),
    parameters: z.record(z.unknown()).optional(),
    options: z.record(z.unknown()).optional(),
    question: z.string().max(2000).optional(),
  })
  .strict();

// Discriminated union for calculate: engine_id selects the strict sub-schema.
const CalculateInput = PrepareReadingInput;

const ListWorkflowsInput = z.object({}).strict();

const WorkflowInfoInput = z
  .object({
    workflow_id: z.string().min(1).max(64),
  })
  .strict();

const RunWorkflowInput = z
  .object({
    workflow_id: z.string().min(1).max(64),
    current_time: z.string().datetime({ offset: true }).optional(),
    birth_data: z
      .object({
        name: z.string().min(1).optional(),
        date: z.string(),
        time: z.string().optional(),
        latitude: z.number().finite().min(-90).max(90),
        longitude: z.number().finite().min(-180).max(180),
        timezone: z.string().min(1),
      })
      .strict()
      .optional(),
    location: z
      .object({
        latitude: z.number().finite().min(-90).max(90),
        longitude: z.number().finite().min(-180).max(180),
      })
      .strict()
      .optional(),
    options: z.object({ question: z.string().max(2000).optional(), spread: z.enum(["single_card", "three_card", "celtic_cross"]).optional(), method: z.literal("three_coins").optional(), type: z.number().int().min(1).max(9), wing: z.number().int().min(1).max(9).optional() }).strict(),
  })
  .strict();

// ---- Tool type and registration API ----------------------------------------

export type ToolHandler = (
  ctx: ToolContext,
  args: Record<string, unknown>,
) => Promise<ToolResult>;

export interface ToolRegistration {
  name: string;
  description: string;
  inputSchema: z.ZodType;
  annotations: ToolAnnotations;
  requiredScope: "read" | "calculate";
  securitySchemes: Record<string, unknown>;
  handler: ToolHandler;
}

// ---- selemene_list_engines -------------------------------------------------

const listEngines: ToolRegistration = {
  name: "selemene_list_engines",
  description:
    "List the Selemene reflection engines exposed by this connector, with catalog metadata and readiness cues. Catalog membership is not operational readiness.",
  inputSchema: ListEnginesInput,
  annotations: {
    title: "List engines",
    readOnlyHint: true,
    idempotentHint: true,
    destructiveHint: false,
    openWorldHint: false,
  },
  requiredScope: "read",
  securitySchemes: {
    type: "oauth2",
    flows: {
      authorizationCode: {
        authorizationUrl: "{PUBLIC_ORIGIN}/authorize",
        tokenUrl: "{PUBLIC_ORIGIN}/oauth/token",
        scopes: { "mcp:read": "Read engine catalog", "mcp:calculate": "Run calculations" },
      },
    },
  },
  async handler(ctx, args) {
    const includeDenied = args.include_denied === true;

    const engines = await callSelemene(ctx.env, {
      method: "GET",
      path: "/api/v1/engines",
      apiKey: ctx.props.apiKey,
    });
    const listErr = toolFromOutcome(engines, ctx.env.mcpResource);
    if (listErr) return listErr;
    const catalog = extractEngineIds(
      engines.kind === "ok" ? engines.data : null,
    );

    let capabilityRows: Record<string, unknown> | null = null;
    let capabilitiesStatus: "ok" | "unknown" | "denied" = "unknown";
    const caps = await callSelemene(ctx.env, {
      method: "GET",
      path: "/api/v1/engines/capabilities",
      apiKey: ctx.props.apiKey,
    });
    if (caps.kind === "ok") {
      capabilityRows = extractCapabilities(caps.data);
      capabilitiesStatus = "ok";
    } else if (caps.kind === "not_found") {
      capabilitiesStatus = "unknown";
    } else if (caps.kind === "unauthorized" || caps.kind === "forbidden") {
      capabilitiesStatus = "denied";
    } else {
      capabilitiesStatus = "unknown";
    }

    const entries: Array<Record<string, unknown>> = [];
    for (const id of ALLOWED_ENGINE_IDS) {
      const schema = ENGINE_SCHEMAS[id];
      if (!schema) continue;
      const inCatalog = catalog.has(id);
      const capability = capabilityRows?.[id] ?? null;
      entries.push({
        engine_id: id,
        title: schema.title,
        summary: schema.summary,
        catalog_present: inCatalog,
        readiness_source: capabilitiesStatus,
        capability,
        side_effects: schema.side_effects,
        warnings: inCatalog
          ? []
          : ["not present in authenticated catalog for this session"],
      });
    }

    const denied: Array<Record<string, unknown>> = [];
    if (includeDenied) {
      const { DENIED_ENGINE_REASONS } = await import("../allowlist.js");
      for (const [id, reason] of Object.entries(DENIED_ENGINE_REASONS)) {
        denied.push({ engine_id: id, reason });
      }
    }

    return ok({
      engines: entries,
      capabilities_source: capabilitiesStatus,
      denied: includeDenied ? denied : undefined,
      note:
        "Catalog membership does not imply operational readiness. " +
        "Use selemene_engine_info for per-engine requirements.",
    });
  },
};

// ---- selemene_engine_info --------------------------------------------------

const engineInfo: ToolRegistration = {
  name: "selemene_engine_info",
  description:
    "Retrieve reviewed requirements and metadata for a single engine. Use this before selemene_calculate to discover what birth_data, options, and parameters are required.",
  inputSchema: EngineInfoInput,
  annotations: {
    title: "Engine info",
    readOnlyHint: true,
    idempotentHint: true,
    destructiveHint: false,
    openWorldHint: false,
  },
  requiredScope: "read",
  securitySchemes: {
    type: "oauth2",
    flows: {
      authorizationCode: {
        authorizationUrl: "{PUBLIC_ORIGIN}/authorize",
        tokenUrl: "{PUBLIC_ORIGIN}/oauth/token",
        scopes: { "mcp:read": "Read engine catalog", "mcp:calculate": "Run calculations" },
      },
    },
  },
  async handler(ctx, args) {
    const engineId = args.engine_id as string;
    if (!isEngineAllowed(engineId)) {
      return fail(denyReason(engineId), { engine_id: engineId });
    }

    const info = await callSelemene(ctx.env, {
      method: "GET",
      path: { kind: "engine_info", engineId },
      apiKey: ctx.props.apiKey,
    });
    const infoErr = toolFromOutcome(info, ctx.env.mcpResource);
    if (infoErr) return infoErr;

    const schema = ENGINE_SCHEMAS[engineId as AllowedEngineId];
    return ok({
      engine_id: engineId,
      upstream: info.kind === "ok" ? info.data : null,
      reviewed_schema: schema
        ? {
            input_schema: zodToJsonSchema(schema.zodSchema, { $refStrategy: "none" }),
            title: schema.title,
            summary: schema.summary,
            requires_birth_data: schema.requires_birth_data,
            requires_exact_time: schema.requires_exact_time,
            requires_current_time: schema.requires_current_time,
            side_effects: schema.side_effects,
          }
        : null,
    });
  },
};

// ---- selemene_prepare_reading ----------------------------------------------

const prepareReading: ToolRegistration = {
  name: "selemene_prepare_reading",
  description:
    "Local preflight validation for a calculation. Reports what birth_data and parameters are missing or invalid, without making an upstream call. Same validation rules as selemene_calculate.",
  inputSchema: PrepareReadingInput,
  annotations: {
    title: "Prepare reading",
    readOnlyHint: true,
    idempotentHint: true,
    destructiveHint: false,
    openWorldHint: false,
  },
  requiredScope: "read",
  securitySchemes: {
    type: "oauth2",
    flows: {
      authorizationCode: {
        authorizationUrl: "{PUBLIC_ORIGIN}/authorize",
        tokenUrl: "{PUBLIC_ORIGIN}/oauth/token",
        scopes: { "mcp:read": "Read engine catalog", "mcp:calculate": "Run calculations" },
      },
    },
  },
  async handler(ctx, args) {
    const engineId = args.engine_id as string;
    if (!isEngineAllowed(engineId)) {
      return fail(denyReason(engineId), { engine_id: engineId });
    }

    // Same validation as calculate, but partial mode reports missing fields.
    const result = validateEngineInput(engineId, args, { partial: true });

    return ok({
      engine_id: engineId,
      valid: result.valid,
      errors: result.errors,
      missing: (result.parsed as Record<string, unknown>)?._prepare_missing ?? [],
      can_calculate:
        (result.parsed as Record<string, unknown>)?._prepare_can_calculate ??
        false,
      reviewed_schema: ENGINE_SCHEMAS[engineId as AllowedEngineId]
        ? {
            requires_birth_data:
              ENGINE_SCHEMAS[engineId as AllowedEngineId]!.requires_birth_data,
            requires_exact_time:
              ENGINE_SCHEMAS[engineId as AllowedEngineId]!.requires_exact_time,
            requires_current_time:
              ENGINE_SCHEMAS[engineId as AllowedEngineId]!.requires_current_time,
          }
        : null,
    });
  },
};

// ---- selemene_calculate ----------------------------------------------------

const calculate: ToolRegistration = {
  name: "selemene_calculate",
  description:
    "Run a Selemene engine calculation. Persistent side effects: reading inputs/results may be saved, usage and XP updated, profile may be auto-populated. Requires mcp:calculate scope.",
  inputSchema: CalculateInput,
  annotations: {
    title: "Calculate",
    readOnlyHint: false,
    idempotentHint: false,
    destructiveHint: false,
    openWorldHint: false,
  },
  requiredScope: "calculate",
  securitySchemes: {
    type: "oauth2",
    flows: {
      authorizationCode: {
        authorizationUrl: "{PUBLIC_ORIGIN}/authorize",
        tokenUrl: "{PUBLIC_ORIGIN}/oauth/token",
        scopes: { "mcp:calculate": "Run calculations" },
      },
    },
  },
  async handler(ctx, args) {
    const engineId = args.engine_id as string;
    if (!isEngineAllowed(engineId)) {
      return fail(denyReason(engineId), { engine_id: engineId });
    }

    // Strict validation — rejects before upstream call.
    const validation = validateEngineInput(engineId, args);
    if (!validation.valid) {
      return fail("input validation failed", {
        engine_id: engineId,
        errors: validation.errors,
      });
    }

    // Build upstream body. For most engines, options come from the validated
    // args directly (engine-specific Zod schema already validated them).
    const options: Record<string, unknown> = { ...(args.parameters as Record<string, unknown> ?? {}) };
    const question = args.question;
    if (typeof question === "string" && question.length > 0) {
      options.question = question;
    }
    const argsOptions = args.options;
    if (argsOptions && typeof argsOptions === "object") {
      for (const [k, v] of Object.entries(
        argsOptions as Record<string, unknown>,
      )) {
        options[k] = v;
      }
    }

    const body: Record<string, unknown> = {
      current_time:
        typeof args.current_time === "string"
          ? args.current_time
          : new Date().toISOString(),
      contract_version: "v1",
      consciousness_level: ctx.props.consciousnessLevel ?? 0,
      parameters: {},
      options,
    };

    if (args.birth_data) {
      body.birth_data = args.birth_data;
    }
    if (args.location) {
      body.location = args.location;
    }

    const outcome = await callSelemene(ctx.env, {
      method: "POST",
      path: { kind: "engine_calculate", engineId },
      apiKey: ctx.props.apiKey,
      body,
      timeoutMs: CALCULATE_TIMEOUT_MS,
    });

    if (outcome.kind !== "ok") {
      return toolFromOutcome(outcome, ctx.env.mcpResource) ?? fail("upstream request failed");
    }

    return ok({
      engine_id: engineId,
      side_effects: CALCULATION_SIDE_EFFECTS,
      persistence_status: "not_confirmed",
      disclosure:
        "Persistence is async best-effort; status is not confirmed until a subsequent query.",
      result: outcome.data,
    });
  },
};

// ---- selemene_list_workflows -----------------------------------------------

const listWorkflows: ToolRegistration = {
  name: "selemene_list_workflows",
  description:
    "List reviewed, safe Selemene workflows. Only decision-support is currently in the reviewed allowlist.",
  inputSchema: ListWorkflowsInput,
  annotations: {
    title: "List workflows",
    readOnlyHint: true,
    idempotentHint: true,
    destructiveHint: false,
    openWorldHint: false,
  },
  requiredScope: "read",
  securitySchemes: {
    type: "oauth2",
    flows: {
      authorizationCode: {
        authorizationUrl: "{PUBLIC_ORIGIN}/authorize",
        tokenUrl: "{PUBLIC_ORIGIN}/oauth/token",
        scopes: { "mcp:read": "Read engine catalog" },
      },
    },
  },
  async handler(ctx, _args) {
    const workflows = await callSelemene(ctx.env, {
      method: "GET",
      path: "/api/v1/workflows",
      apiKey: ctx.props.apiKey,
    });
    const listErr = toolFromOutcome(workflows, ctx.env.mcpResource);
    if (listErr) return listErr;

    const entries: Array<Record<string, unknown>> = [];
    const liveList = workflows.kind === "ok" && workflows.data && typeof workflows.data === "object" ? (workflows.data as {workflows?: {id:string}[]}).workflows : [];
    for (const id of ALLOWED_WORKFLOW_IDS) {
      if (!Array.isArray(liveList) || !liveList.some(item => item.id === id)) continue;
      const liveInfo = await callSelemene(ctx.env, {method:"GET",path:{kind:"workflow_info",workflowId:id},apiKey:ctx.props.apiKey});
      if (liveInfo.kind !== "ok") continue;
      const liveEngines = extractWorkflowEngines(liveInfo.data);
      if (liveEngines.length !== ALLOWED_WORKFLOW_ENGINES[id].length || new Set(liveEngines).size !== liveEngines.length || ALLOWED_WORKFLOW_ENGINES[id].some(engine => !liveEngines.includes(engine))) continue;
      const safeEngines = ALLOWED_WORKFLOW_ENGINES[id] ?? [];
      entries.push({
        workflow_id: id,
        safe_engine_ids: [...safeEngines],
        description:
          id === "decision-support"
            ? "Multi-engine decision support combining tarot, I Ching, human design, enneagram, and gene keys."
            : id,
      });
    }

    return ok({
      workflows: entries,
      note:
        "Only workflows whose engine composition is fully within the reviewed " +
        "non-media allowlist are exposed. Unknown or unreviewed workflows are denied.",
    });
  },
};

// ---- selemene_workflow_info ------------------------------------------------

const workflowInfo: ToolRegistration = {
  name: "selemene_workflow_info",
  description:
    "Retrieve live metadata for a workflow. Engine IDs are cross-checked against the reviewed safe set; tainted workflows are refused.",
  inputSchema: WorkflowInfoInput,
  annotations: {
    title: "Workflow info",
    readOnlyHint: true,
    idempotentHint: true,
    destructiveHint: false,
    openWorldHint: false,
  },
  requiredScope: "read",
  securitySchemes: {
    type: "oauth2",
    flows: {
      authorizationCode: {
        authorizationUrl: "{PUBLIC_ORIGIN}/authorize",
        tokenUrl: "{PUBLIC_ORIGIN}/oauth/token",
        scopes: { "mcp:read": "Read engine catalog" },
      },
    },
  },
  async handler(ctx, args) {
    const workflowId = args.workflow_id as string;
    if (!isWorkflowAllowed(workflowId)) {
      return fail("workflow is not in the reviewed allowlist", {
        workflow_id: workflowId,
      });
    }

    const info = await callSelemene(ctx.env, {
      method: "GET",
      path: { kind: "workflow_info", workflowId },
      apiKey: ctx.props.apiKey,
    });
    const infoErr = toolFromOutcome(info, ctx.env.mcpResource);
    if (infoErr) return infoErr;

    const engineIds = extractWorkflowEngines(
      info.kind === "ok" ? info.data : null,
    );
    const expected = ALLOWED_WORKFLOW_ENGINES[workflowId];
    if (!engineIds.length || new Set(engineIds).size !== engineIds.length || engineIds.length !== expected.length || expected.some((id) => !engineIds.includes(id))) return fail("Workflow composition is unavailable or changed.");
    const tainted = engineIds.filter((id) => !isEngineAllowed(id));
    if (tainted.length > 0) {
      return fail("workflow declares engines outside the reviewed allowlist", {
        workflow_id: workflowId,
        tainted_engine_ids: tainted,
      });
    }

    return ok({
      workflow_id: workflowId,
      engine_ids: engineIds,
      safe: true,
    });
  },
};

// ---- selemene_run_workflow -------------------------------------------------

const runWorkflow: ToolRegistration = {
  name: "selemene_run_workflow",
  description:
    "Execute a reviewed, safe Selemene workflow. Validates live metadata against the reviewed engine allowlist before POST. Persistent side effects apply. Requires mcp:calculate scope.",
  inputSchema: RunWorkflowInput,
  annotations: {
    title: "Run workflow",
    readOnlyHint: false,
    idempotentHint: false,
    destructiveHint: false,
    openWorldHint: false,
  },
  requiredScope: "calculate",
  securitySchemes: {
    type: "oauth2",
    flows: {
      authorizationCode: {
        authorizationUrl: "{PUBLIC_ORIGIN}/authorize",
        tokenUrl: "{PUBLIC_ORIGIN}/oauth/token",
        scopes: { "mcp:calculate": "Run calculations" },
      },
    },
  },
  async handler(ctx, args) {
    const workflowId = args.workflow_id as string;
    if (!isWorkflowAllowed(workflowId)) {
      return fail("workflow is not in the reviewed allowlist", {
        workflow_id: workflowId,
      });
    }

    // 1) verify live metadata BEFORE POST
    const infoOutcome = await callSelemene(ctx.env, {
      method: "GET",
      path: { kind: "workflow_info", workflowId },
      apiKey: ctx.props.apiKey,
    });
    const infoErr = toolFromOutcome(infoOutcome, ctx.env.mcpResource);
    if (infoErr) return infoErr;

    const engineIds = extractWorkflowEngines(
      infoOutcome.kind === "ok" ? infoOutcome.data : null,
    );
    const expected = ALLOWED_WORKFLOW_ENGINES[workflowId];
    if (!engineIds.length || new Set(engineIds).size !== engineIds.length || engineIds.length !== expected.length || expected.some((id) => !engineIds.includes(id))) return fail("Workflow composition is unavailable or changed.");
    const tainted = engineIds.filter((id) => !isEngineAllowed(id));
    if (tainted.length > 0) {
      return fail("workflow declares engines outside the reviewed allowlist", {
        workflow_id: workflowId,
        tainted_engine_ids: tainted,
      });
    }

    const opts = (args.options ?? {}) as Record<string, unknown>;
    for (const id of engineIds) {
      if (!isEngineAllowed(id)) return fail("Unreviewed workflow engine.");
      const candidate = id === "human-design" || id === "gene-keys"
        ? { engine_id: id, birth_data: args.birth_data }
        : { engine_id: id, options: Object.fromEntries(Object.entries(opts).filter(([key]) =>
            (id === "tarot" ? ["spread", "question"] : id === "i-ching" ? ["method", "question"] : ["type", "wing"]).includes(key))) };
      const checked = validateEngineInput(id, candidate);
      if (!checked.valid) return fail("Workflow input incomplete or invalid.", { engine_id: id, errors: checked.errors });
    }
    // 2) build body
    const body: Record<string, unknown> = {
      current_time:
        typeof args.current_time === "string"
          ? args.current_time
          : new Date().toISOString(),
    };
    if (args.birth_data) {
      const bd = args.birth_data as Record<string, unknown>;
      try {
        if (typeof bd.date !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(bd.date)) {
          throw new SelemeneInputError("birth_data.date must be YYYY-MM-DD");
        }
        if (
          typeof bd.latitude !== "number" ||
          bd.latitude < -90 ||
          bd.latitude > 90
        ) {
          throw new SelemeneInputError("latitude out of range");
        }
        if (
          typeof bd.longitude !== "number" ||
          bd.longitude < -180 ||
          bd.longitude > 180
        ) {
          throw new SelemeneInputError("longitude out of range");
        }
        if (typeof bd.timezone !== "string" || bd.timezone.length === 0) {
          throw new SelemeneInputError("birth_data.timezone required");
        }
        body.birth_data = bd;
      } catch (err) {
        return fail(err instanceof Error ? err.message : "invalid birth_data");
      }
    }
    if (args.location) {
      const loc = args.location as Record<string, unknown>;
      if (
        typeof loc.latitude !== "number" ||
        typeof loc.longitude !== "number"
      ) {
        return fail("location must have numeric latitude and longitude");
      }
      body.location = { latitude: loc.latitude, longitude: loc.longitude };
    }
    body.options = args.options ?? {};

    const outcome = await callSelemene(ctx.env, {
      method: "POST",
      path: { kind: "workflow_execute", workflowId },
      apiKey: ctx.props.apiKey,
      body,
      timeoutMs: CALCULATE_TIMEOUT_MS,
    });

    if (outcome.kind !== "ok") {
      return toolFromOutcome(outcome, ctx.env.mcpResource) ?? fail("upstream request failed");
    }

    const returned = extractReturnedEngines(outcome.data);
    const missing = engineIds.filter((id) => !returned.has(id));

    return ok({
      workflow_id: workflowId,
      persistence_status: "not_confirmed",
      disclosure:
        "Persistence is async best-effort; status is not confirmed until a subsequent query.",
      side_effects: CALCULATION_SIDE_EFFECTS,
      declared_engine_ids: engineIds,
      returned_engine_ids: [...returned],
      missing_engine_ids: missing,
      partial: missing.length > 0,
      omission_note:
        missing.length > 0
          ? "Backend does not return per-engine omission reasons; treat missing entries as unavailable without inventing a cause."
          : null,
      result: outcome.data,
    });
  },
};

// ---- Registry ---------------------------------------------------------------

export const ALL_TOOLS: readonly ToolRegistration[] = [
  listEngines,
  engineInfo,
  prepareReading,
  calculate,
  listWorkflows,
  workflowInfo,
  runWorkflow,
];

export function findTool(name: string): ToolRegistration | undefined {
  return ALL_TOOLS.find((t) => t.name === name);
}

// ---- Extraction helpers ----------------------------------------------------

function extractEngineIds(data: unknown): Set<string> {
  const out = new Set<string>();
  if (Array.isArray(data)) {
    for (const row of data) {
      if (typeof row === "string") out.add(row);
      if (row && typeof row === "object") {
        const id = (row as Record<string, unknown>).engine_id;
        if (typeof id === "string") out.add(id);
      }
    }
  } else if (data && typeof data === "object") {
    const engines = (data as Record<string, unknown>).engines;
    if (Array.isArray(engines)) {
      for (const row of engines) {
        if (typeof row === "string") out.add(row);
      if (row && typeof row === "object") {
          const id = (row as Record<string, unknown>).engine_id;
          if (typeof id === "string") out.add(id);
        }
      }
    }
  }
  return out;
}

function extractCapabilities(data: unknown): Record<string, unknown> | null {
  if (!data || typeof data !== "object") return null;
  const rec = data as Record<string, unknown>;
  if (rec.capabilities && typeof rec.capabilities === "object") {
    return rec.capabilities as Record<string, unknown>;
  }
  return rec;
}

function extractWorkflowEngines(data: unknown): string[] {
  if (!data || typeof data !== "object") return [];
  const rec = data as Record<string, unknown>;
  const candidates: unknown[] = [];
  if (Array.isArray(rec.engine_ids)) candidates.push(...rec.engine_ids);
  if (Array.isArray(rec.engines)) candidates.push(...rec.engines);
  if (!candidates.every(v => typeof v === "string")) return [];
  return candidates as string[];
}

function extractReturnedEngines(data: unknown): Set<string> {
  const out = new Set<string>();
  if (data && typeof data === "object") {
    const rec = data as Record<string, unknown>;
    const outputs = rec.engine_outputs;
    if (outputs && typeof outputs === "object" && !Array.isArray(outputs)) {
      for (const key of Object.keys(outputs)) out.add(key);
    } else if (Array.isArray(outputs)) {
      for (const row of outputs) {
        if (
          row &&
          typeof row === "object" &&
          typeof (row as Record<string, unknown>).engine_id === "string"
        ) {
          out.add((row as Record<string, unknown>).engine_id as string);
        }
      }
    }
  }
  return out;
}
