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

export interface ReadingSummary {
  reading_id: string;
  workflow_id: string;
  birth_data: BirthData;
  created_at: string;
  engine_count: number;
}

export interface ReadingsResponse {
  readings: ReadingSummary[];
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
  const payload = text ? JSON.parse(text) : {};

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

export async function getReadings(
  apiKey: string,
): Promise<ReadingsResponse> {
  return request<ReadingsResponse>(
    "/api/v1/readings",
    { method: "GET" },
    apiKey,
  );
}

export async function getHealth(): Promise<HealthResponse> {
  return request<HealthResponse>("/health/live", { method: "GET" });
}
