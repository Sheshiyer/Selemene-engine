export const ENGINE_IDS = [
  "biofield",
  "biorhythm",
  "enneagram",
  "face-reading",
  "gene-keys",
  "human-design",
  "i-ching",
  "nadabrahman",
  "numerology",
  "panchanga",
  "sacred-geometry",
  "sigil-forge",
  "tarot",
  "transits",
  "vedic-clock",
  "vimshottari",
] as const;

export const WORKFLOW_IDS = [
  "birth-blueprint",
  "creative-expression",
  "daily-practice",
  "decision-support",
  "full-spectrum",
  "self-inquiry",
] as const;

export type EngineId = (typeof ENGINE_IDS)[number];
export type WorkflowId = (typeof WORKFLOW_IDS)[number];

export interface BirthData {
  date: string;
  time?: string;
  latitude?: number;
  longitude?: number;
  timezone?: string;
  name?: string;
}

export interface EngineInput {
  birth_data?: BirthData;
  current_time?: string;
  precision?: "Standard" | "High" | "Extreme";
  options?: Record<string, unknown>;
  [key: string]: unknown;
}

export interface EngineOutput {
  engine_id: string;
  result: Record<string, unknown>;
  witness_prompt?: string;
  witness_prompts?: string[];
  metadata?: Record<string, unknown>;
  consciousness_level?: number;
}

export interface WorkflowResult {
  workflow_id: string;
  engine_outputs?: EngineOutput[];
  engine_results?: EngineOutput[];
  synthesis?: string;
  timestamp?: string;
  total_time_ms?: number;
}

export interface HealthResponse {
  status: string;
  version: string;
  uptime_seconds: number;
  engines_loaded: number;
  workflows_loaded: number;
}

export interface RateLimitInfo {
  limit?: number;
  remaining?: number;
  reset?: number;
  dailyRemaining?: number;
  dailyReset?: number;
}

export interface NoesisClientOptions {
  authToken?: string;
  maxRetries?: number;
  backoffMs?: number;
}

export interface RequestOptions {
  signal?: AbortSignal;
}

export class SelemeneError extends Error {
  readonly status: number;
  readonly details?: unknown;

  constructor(message: string, status: number, details?: unknown) {
    super(message);
    this.name = "SelemeneError";
    this.status = status;
    this.details = details;
  }
}

export class NoesisClient {
  private readonly authToken?: string;
  private readonly maxRetries: number;
  private readonly backoffMs: number;
  public rateLimitInfo: RateLimitInfo = {};

  constructor(
    private readonly baseUrl: string,
    options: string | NoesisClientOptions = {},
  ) {
    if (typeof options === "string") {
      this.authToken = options;
      this.maxRetries = 0;
      this.backoffMs = 150;
    } else {
      this.authToken = options.authToken;
      this.maxRetries = options.maxRetries ?? 0;
      this.backoffMs = options.backoffMs ?? 150;
    }
  }

  async health(options?: RequestOptions): Promise<HealthResponse> {
    return this.request<HealthResponse>("/health/live", { method: "GET" }, options);
  }

  async calculate(
    engineId: EngineId | string,
    input: EngineInput,
    options?: RequestOptions,
  ): Promise<EngineOutput> {
    return this.request<EngineOutput>(
      `/api/v1/engines/${engineId}/calculate`,
      {
        method: "POST",
        body: JSON.stringify(input),
      },
      options,
    );
  }

  async workflow(
    workflowId: WorkflowId | string,
    input: EngineInput,
    options?: RequestOptions,
  ): Promise<WorkflowResult> {
    return this.request<WorkflowResult>(
      `/api/v1/workflows/${workflowId}/execute`,
      {
        method: "POST",
        body: JSON.stringify(input),
      },
      options,
    );
  }

  private async request<T>(
    path: string,
    init: RequestInit,
    options?: RequestOptions,
  ): Promise<T> {
    const headers = new Headers(init.headers ?? {});
    headers.set("Content-Type", "application/json");

    if (this.authToken) {
      headers.set("Authorization", `Bearer ${this.authToken}`);
    }

    let attempt = 0;
    while (true) {
      const response = await fetch(`${this.baseUrl}${path}`, {
        ...init,
        headers,
        signal: options?.signal,
      });

      this.captureRateLimit(response.headers);

      const text = await response.text();
      const payload = text ? JSON.parse(text) : {};

      if (response.ok) {
        return payload as T;
      }

      const shouldRetry =
        response.status >= 500 &&
        attempt < this.maxRetries &&
        !options?.signal?.aborted;

      if (!shouldRetry) {
        throw new SelemeneError(
          `Request failed: ${response.status}`,
          response.status,
          payload,
        );
      }

      attempt += 1;
      const waitMs = this.backoffMs * 2 ** (attempt - 1);
      await delay(waitMs, options?.signal);
    }
  }

  private captureRateLimit(headers: Headers): void {
    this.rateLimitInfo = {
      limit: toNumber(headers.get("x-ratelimit-limit")),
      remaining: toNumber(headers.get("x-ratelimit-remaining")),
      reset: toNumber(headers.get("x-ratelimit-reset")),
      dailyRemaining: toNumber(headers.get("x-ratelimit-daily-remaining")),
      dailyReset: toNumber(headers.get("x-ratelimit-daily-reset")),
    };
  }
}

function toNumber(value: string | null): number | undefined {
  if (!value) return undefined;
  const n = Number(value);
  return Number.isFinite(n) ? n : undefined;
}

async function delay(ms: number, signal?: AbortSignal): Promise<void> {
  if (!signal) {
    await new Promise((resolve) => setTimeout(resolve, ms));
    return;
  }

  await new Promise<void>((resolve, reject) => {
    const timer = setTimeout(() => {
      signal.removeEventListener("abort", onAbort);
      resolve();
    }, ms);

    const onAbort = () => {
      clearTimeout(timer);
      signal.removeEventListener("abort", onAbort);
      reject(new DOMException("Request aborted", "AbortError"));
    };

    signal.addEventListener("abort", onAbort, { once: true });
  });
}
