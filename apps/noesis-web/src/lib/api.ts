const API_BASE =
  (typeof process !== "undefined" &&
    process.env?.NEXT_PUBLIC_API_BASE_URL) ||
  "https://selemene.tryambakam.space";

/* ── Types ─────────────────────────────────────────────── */

export interface BirthData {
  date: string;
  time?: string;
  latitude?: number;
  longitude?: number;
  timezone?: string;
  name?: string;
}

export interface EngineOutput {
  engine_id: string;
  result: Record<string, unknown>;
  witness_prompt?: string;
  witness_prompts?: string[];
  metadata?: Record<string, unknown>;
  consciousness_level?: number;
}

export interface WitnessLayer {
  title: string;
  summary: string;
  question: string;
  convergences: string[];
  frictions: string[];
  practice: string;
}

export interface WorkflowResponse {
  workflow_id: string;
  /**
   * Engine results keyed by engine_id (both fields carry same data).
   * Use `engine_outputs ?? engine_results` for compatibility.
   */
  engine_outputs?: Record<string, EngineOutput>;
  engine_results?: Record<string, EngineOutput>;
  witness_layer?: WitnessLayer;
  synthesis?: string;
  timestamp?: string;
  total_time_ms?: number;
  reading_id?: string;
}

/**
 * Matches the Rust `Reading` struct returned by GET /api/v1/readings and GET /api/v1/readings/:id.
 * Fields use the actual Rust/Serde names: `id`, `input_data`, `result_data`.
 */
export interface ReadingSummary {
  id: string;
  user_id: string;
  engine_id: string;
  workflow_id: string | null;
  /** The birth data input that was used for this reading. */
  input_data: BirthData;
  /** The workflow/engine result. For workflow readings this is a WorkflowResponse. */
  result_data: WorkflowResponse;
  witness_prompt: string | null;
  consciousness_level: number;
  calculation_time_ms: number | null;
  created_at: string;
}

export interface ReadingsResponse {
  readings: ReadingSummary[];
  total: number;
  limit: number;
  offset: number;
}

export interface HealthResponse {
  status: string;
  version: string;
  uptime_seconds: number;
  engines_loaded: number;
  workflows_loaded: number;
}

export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly details?: unknown,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

/* ── Helpers ───────────────────────────────────────────── */

function authHeaders(apiKey: string): Record<string, string> {
  if (apiKey.startsWith("nk_")) {
    return { "x-api-key": apiKey };
  }
  return { Authorization: `Bearer ${apiKey}` };
}

async function request<T>(
  path: string,
  init: RequestInit,
  apiKey?: string,
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(apiKey ? authHeaders(apiKey) : {}),
  };

  const res = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers,
  });

  const text = await res.text();
  let payload: unknown = {};
  try {
    payload = text ? JSON.parse(text) : {};
  } catch {
    // Response body is not JSON (e.g. Axum path-param rejection plain-text)
    if (!res.ok) {
      throw new ApiError(text || `Request failed: ${res.status}`, res.status);
    }
    throw new ApiError(`Unexpected non-JSON response: ${text.slice(0, 120)}`, res.status);
  }

  if (!res.ok) {
    throw new ApiError(
      `Request failed: ${res.status}`,
      res.status,
      payload,
    );
  }

  return payload as T;
}

/* ── Public API ────────────────────────────────────────── */

export interface LoginResponse {
  token: string;
  user_id: string;
  email: string;
  tier: string;
}

export async function getDiscordAuthUrl(
  redirectUri?: string,
): Promise<{ url: string }> {
  const qs = redirectUri ? `?redirect_uri=${encodeURIComponent(redirectUri)}` : "";
  return request<{ url: string }>(`/api/v1/auth/discord/authorize${qs}`, {
    method: "GET",
  });
}

export async function discordCallback(
  code: string,
  state?: string,
  redirectUri?: string,
): Promise<LoginResponse> {
  return request<LoginResponse>("/api/v1/auth/discord/callback", {
    method: "POST",
    body: JSON.stringify({ code, state, redirect_uri: redirectUri }),
  });
}

export async function executeWorkflow(
  workflowId: string,
  birthData: BirthData,
  apiKey: string,
): Promise<WorkflowResponse> {
  return request<WorkflowResponse>(
    `/api/v1/workflows/${workflowId}/execute`,
    {
      method: "POST",
      body: JSON.stringify({ birth_data: birthData }),
    },
    apiKey,
  );
}

export interface MeResponse {
  id: string;
  email: string;
  full_name: string;
  tier: string;
}

export async function getMe(apiKey: string): Promise<MeResponse> {
  return request<MeResponse>("/api/v1/users/me", { method: "GET" }, apiKey);
}

/** Returns true if the current token has admin permissions. */
export async function checkAdminAccess(apiKey: string): Promise<boolean> {
  try {
    await request<unknown>("/api/v1/admin/system/health", { method: "GET" }, apiKey);
    return true;
  } catch {
    return false;
  }
}

export async function getReadings(
  apiKey: string,
): Promise<ReadingsResponse> {
  return request<ReadingsResponse>(
    "/api/v1/readings",
    { method: "GET" },
    apiKey,
  );
}

/**
 * Claim a previously-anonymous reading and associate it with the
 * authenticated user. Called after Discord OAuth completes when
 * there's a pendingClaim in localStorage. Backend implementation note:
 * if the endpoint doesn't exist yet, this will 404 — the frontend
 * gracefully degrades (the reading remains in localStorage only).
 */
export async function claimReading(
  readingId: string,
  apiKey: string,
): Promise<{ ok: boolean; reading?: ReadingSummary }> {
  return request<{ ok: boolean; reading?: ReadingSummary }>(
    `/api/v1/readings/${encodeURIComponent(readingId)}/claim`,
    { method: "POST" },
    apiKey,
  );
}

export async function getReading(
  readingId: string,
  apiKey: string,
): Promise<ReadingSummary> {
  return request<ReadingSummary>(
    `/api/v1/readings/${readingId}`,
    { method: "GET" },
    apiKey,
  );
}

export async function getHealth(): Promise<HealthResponse> {
  return request<HealthResponse>("/health/live", { method: "GET" });
}
