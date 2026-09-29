// Bounded, sanitized Selemene Rust API client.
//
// Contract:
//   * URL is derived from a validated env origin plus a fixed method+path;
//     never from caller-supplied data.
//   * `X-API-Key` is the only credential ever sent (matches
//     crates/noesis-api/src/middleware.rs:209-224). We never place the key in
//     `Authorization: Bearer`.
//   * Request/response bodies are bounded; timeouts are enforced via
//     AbortSignal; redirects are refused.
//   * No retry. Calculations have persistent side effects.
//   * Errors are sanitized. Upstream response bodies are not surfaced.
//   * 401 and 403 are distinguished: 401 = credential rejected (reauth),
//     403 = access denied for authenticated user (permissions).

import type { ValidatedEnv } from "./env.js";

export const MAX_REQUEST_BYTES = 32 * 1024; // 32 KiB request body cap
export const MAX_RESPONSE_BYTES = 256 * 1024; // 256 KiB response body cap
export const DEFAULT_TIMEOUT_MS = 15_000;
export const CALCULATE_TIMEOUT_MS = 45_000;

export type SelemeneMethod = "GET" | "POST";

export type SelemenePath =
  | "/api/v1/users/me"
  | "/api/v1/engines"
  | "/api/v1/engines/capabilities"
  | "/api/v1/workflows"
  | { kind: "engine_info"; engineId: string }
  | { kind: "engine_calculate"; engineId: string }
  | { kind: "workflow_info"; workflowId: string }
  | { kind: "workflow_execute"; workflowId: string };

export interface UpstreamRequest {
  method: SelemeneMethod;
  path: SelemenePath;
  apiKey: string;
  body?: unknown;
  timeoutMs?: number;
}

export type UpstreamOutcome =
  | { kind: "ok"; status: number; data: unknown }
  | { kind: "not_found"; status: 404; sanitized: string }
  | { kind: "unauthorized"; status: 401; sanitized: string }
  | { kind: "forbidden"; status: 403; sanitized: string }
  | { kind: "rate_limited"; status: 429; sanitized: string }
  | { kind: "validation_error"; status: 400 | 422; sanitized: string }
  | { kind: "unavailable"; status: number; sanitized: string }
  | { kind: "transport_error"; sanitized: string };

const ENGINE_ID_RE = /^[a-z0-9][a-z0-9-]{0,63}$/;
const WORKFLOW_ID_RE = /^[a-z0-9][a-z0-9-]{0,63}$/;

function renderPath(path: SelemenePath): string {
  if (typeof path === "string") return path;
  switch (path.kind) {
    case "engine_info":
      requireSafeId(path.engineId, ENGINE_ID_RE, "engine_id");
      return `/api/v1/engines/${encodeURIComponent(path.engineId)}/info`;
    case "engine_calculate":
      requireSafeId(path.engineId, ENGINE_ID_RE, "engine_id");
      return `/api/v1/engines/${encodeURIComponent(path.engineId)}/calculate`;
    case "workflow_info":
      requireSafeId(path.workflowId, WORKFLOW_ID_RE, "workflow_id");
      return `/api/v1/workflows/${encodeURIComponent(path.workflowId)}/info`;
    case "workflow_execute":
      requireSafeId(path.workflowId, WORKFLOW_ID_RE, "workflow_id");
      return `/api/v1/workflows/${encodeURIComponent(path.workflowId)}/execute`;
  }
}

function requireSafeId(value: string, re: RegExp, field: string): void {
  if (!re.test(value)) {
    throw new SelemeneInputError(`invalid ${field}`);
  }
}

export class SelemeneInputError extends Error {}

export async function callSelemene(
  env: ValidatedEnv,
  req: UpstreamRequest,
): Promise<UpstreamOutcome> {
  const url = `${env.selemeneApiOrigin}${renderPath(req.path)}`;
  const controller = new AbortController();
  const timeout = req.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  const timeoutHandle = setTimeout(() => controller.abort(), timeout);

  let bodyText: string | undefined;
  if (req.body !== undefined) {
    bodyText = JSON.stringify(req.body);
    if (new TextEncoder().encode(bodyText).byteLength > MAX_REQUEST_BYTES) {
      clearTimeout(timeoutHandle);
      return {
        kind: "validation_error",
        status: 422,
        sanitized: "request body exceeds worker byte cap",
      };
    }
  }

  const headers = new Headers();
  headers.set("Accept", "application/json");
  headers.set("X-API-Key", req.apiKey);
  headers.set("User-Agent", "selemene-mcp/0.1 (+chatgpt-plugin)");
  if (bodyText !== undefined) {
    headers.set("Content-Type", "application/json");
  }

  let response: Response;
  try {
    response = await fetch(url, {
      method: req.method,
      headers,
      body: bodyText,
      signal: controller.signal,
      redirect: "error",
    });
  } catch (err) {
    clearTimeout(timeoutHandle);
    if (err instanceof Error && err.name === "AbortError") {
      return {
        kind: "transport_error",
        sanitized: "upstream request timed out",
      };
    }
    return {
      kind: "transport_error",
      sanitized: "upstream transport error",
    };
  }
  // Do NOT clear the timeout here — it must remain armed through the bounded
  // response body read below. readBounded respects the same AbortSignal.

  const status = response.status;
  // Enforce response bound before parsing. The AbortSignal from the controller
  // (which the timeout armed) is still active during this read.
  const raw = await readBounded(response, controller.signal);
  clearTimeout(timeoutHandle);

  if (raw === null) {
    return {
      kind: "unavailable",
      status,
      sanitized: "upstream response exceeded worker byte cap",
    };
  }

  if (status >= 200 && status < 300) {
    if (raw.length === 0) return { kind: "ok", status, data: null };
    try {
      const data = JSON.parse(raw);
      return { kind: "ok", status, data };
    } catch {
      return {
        kind: "unavailable",
        status,
        sanitized: "upstream returned non-JSON payload",
      };
    }
  }

  const sanitized = sanitizeStatus(status);
  if (status === 401) {
    return { kind: "unauthorized", status: 401, sanitized };
  }
  if (status === 403) {
    return { kind: "forbidden", status: 403, sanitized };
  }
  if (status === 404) {
    return { kind: "not_found", status: 404, sanitized };
  }
  if (status === 429) {
    return { kind: "rate_limited", status: 429, sanitized };
  }
  if (status === 400 || status === 422) {
    return { kind: "validation_error", status: status as 400 | 422, sanitized };
  }
  return { kind: "unavailable", status, sanitized };
}

async function readBounded(
  response: Response,
  signal?: AbortSignal,
): Promise<string | null> {
  if (!response.body) {
    // No streaming body — check signal, then read.
    if (signal?.aborted) return null;
    const text = await response.text();
    return text.length > MAX_RESPONSE_BYTES ? null : text;
  }
  const reader = response.body.getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;
  try {
    while (true) {
      // If the upstream timeout fired, the signal aborts and read() will reject.
      let abort: (() => void) | undefined;
      const canceled = new Promise<never>((_, reject) => {
        abort = () => reject(new Error("Upstream body interrupted"));
        if (signal?.aborted) abort(); else signal?.addEventListener("abort", abort, { once: true });
      });
      let next: ReadableStreamReadResult<Uint8Array>;
      try { next = await Promise.race([reader.read(), canceled]); }
      finally { if (abort) signal?.removeEventListener("abort", abort); }
      const { value, done } = next;
      if (done) break;
      if (value) {
        total += value.byteLength;
        if (total > MAX_RESPONSE_BYTES) {
          void reader.cancel().catch(() => {});
          return null;
        }
        chunks.push(value);
      }
    }
  } catch (err) {
    // AbortError from timeout or cancel — sanitize and return null.
    void reader.cancel().catch(() => {});
    if (err instanceof Error && err.name === "AbortError") {
      return null;
    }
    return null;
  }
  const buf = new Uint8Array(total);
  let offset = 0;
  for (const c of chunks) {
    buf.set(c, offset);
    offset += c.byteLength;
  }
  return new TextDecoder().decode(buf);
}

function sanitizeStatus(status: number): string {
  if (status === 401) return "upstream rejected credential";
  if (status === 403) return "upstream denied access for this user";
  if (status === 404) return "upstream resource not found";
  if (status === 422) return "upstream rejected input as invalid";
  if (status === 400) return "upstream rejected request";
  if (status === 429) return "upstream rate limit exceeded";
  if (status >= 500) return "upstream service unavailable";
  return "upstream request failed";
}

// ---- Upstream 401 reauth helper (used by tool handlers) --------------------

/**
 * Creates a tool-level reauth result when the upstream returns 401.
 * Returns isError:true with a www_authenticate challenge array so the
 * MCP client can trigger re-authentication. No secrets are leaked.
 */
export function upstreamReauthResult(
  resource: string,
  outcome: Extract<UpstreamOutcome, { kind: "unauthorized" }>,
): {
  content: Array<{ type: "text"; text: string }>;
  isError: boolean;
  _meta: Record<string, unknown>;
} {
  const url = new URL(resource);
  const challenge = [`Bearer resource_metadata="${url.origin}/.well-known/oauth-protected-resource${url.pathname}", error="invalid_token", error_description="Reconnect your Selemene account."`];
  return {
    isError: true,
    content: [
      {
        type: "text",
        text: JSON.stringify({
          error: "upstream_authentication_required",
          message:
            "The upstream Selemene engine rejected the credential. " +
            "Re-authenticate to obtain a fresh token.",
        }),
      },
    ],
    _meta: {
      "mcp/www_authenticate": challenge,
    },
  };
}

/**
 * Creates a tool-level forbidden result when the upstream returns 403.
 * The credential was accepted but the user lacks permission.
 */
export function upstreamForbiddenResult(
  outcome: Extract<UpstreamOutcome, { kind: "forbidden" }>,
): {
  content: Array<{ type: "text"; text: string }>;
  isError: boolean;
} {
  return {
    isError: true,
    content: [
      {
        type: "text",
        text: JSON.stringify({
          error: "upstream_permission_denied",
          message:
            "The upstream Selemene engine denied access for this user. " +
            "This may indicate insufficient tier or account status.",
        }),
      },
    ],
  };
}
