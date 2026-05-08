export * from "./billing.js";

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

/** v3.3.0 reading-object contract fields */
export interface WitnessLayer {
  title?: string;
  summary?: string;
  convergences?: string[];
  frictions?: string[];
  practice?: string;
  question?: string;
}

export interface WorkflowResult {
  workflow_id: string;
  engine_outputs?: EngineOutput[];
  engine_results?: EngineOutput[];
  synthesis?: string;
  timestamp?: string;
  total_time_ms?: number;
  /** v3.3.0: Reading persistence fields */
  reading_id?: string;
  reading_url?: string | null;
  created_at?: string;
  subject?: string;
  evidence?: string[];
  witness_layer?: WitnessLayer;
}

export interface HealthResponse {
  status: string;
  version: string;
  uptime_seconds: number;
  engines_loaded: number;
  workflows_loaded: number;
}

export interface EngineInfo {
  id: string;
  name: string;
  description?: string;
  version?: string;
}

export interface WorkflowInfo {
  id: string;
  name: string;
  engines?: string[];
  description?: string;
}

export interface UserProfile {
  id: string;
  email?: string;
  role?: string;
  created_at?: string;
}

export interface UsageSummary {
  total_calls: number;
  calls_today?: number;
  credits_used?: number;
  period_start?: string;
  period_end?: string;
}

export interface Reading {
  id: string;
  workflow_id?: string;
  created_at: string;
  subject?: string;
  reading_url?: string | null;
  engine_count?: number;
}

export interface ReadingDetail extends Reading {
  witness_layer?: WitnessLayer;
  engine_outputs?: EngineOutput[];
  synthesis?: string;
}

export interface WitnessInterpretation {
  interpretation: string;
  context?: string;
  suggestions?: string[];
}

export interface ListReadingsOptions {
  page?: number;
  per_page?: number;
  workflow_id?: string;
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

  /** List all available engines. */
  async listEngines(options?: RequestOptions): Promise<EngineInfo[]> {
    return this.request<EngineInfo[]>("/api/v1/engines", { method: "GET" }, options);
  }

  /** List all available workflows. */
  async listWorkflows(options?: RequestOptions): Promise<WorkflowInfo[]> {
    return this.request<WorkflowInfo[]>("/api/v1/workflows", { method: "GET" }, options);
  }

  /** Get engine metadata by ID. */
  async getEngineInfo(engineId: EngineId | string, options?: RequestOptions): Promise<EngineInfo> {
    return this.request<EngineInfo>(`/api/v1/engines/${engineId}`, { method: "GET" }, options);
  }

  /** Get workflow metadata by ID. */
  async getWorkflowInfo(workflowId: WorkflowId | string, options?: RequestOptions): Promise<WorkflowInfo> {
    return this.request<WorkflowInfo>(`/api/v1/workflows/${workflowId}`, { method: "GET" }, options);
  }

  // ── Auth & user ─────────────────────────────────────────────────────────

  /** Get the authenticated user's profile. */
  async getMe(options?: RequestOptions): Promise<UserProfile> {
    return this.request<UserProfile>("/api/v1/auth/me", { method: "GET" }, options);
  }

  /** Get the authenticated user's usage summary. */
  async getMyUsage(options?: RequestOptions): Promise<UsageSummary> {
    return this.request<UsageSummary>("/api/v1/usage/me", { method: "GET" }, options);
  }

  // ── Billing ──────────────────────────────────────────────────────────────

  /** Get the user's current credit balance and subscription info. */
  async getBillingBalance(options?: RequestOptions): Promise<import("./billing.js").BalanceResponse> {
    return this.request<import("./billing.js").BalanceResponse>("/api/v1/billing/balance", { method: "GET" }, options);
  }

  /** Get the user's active subscription details. */
  async getBillingSubscription(options?: RequestOptions): Promise<Record<string, unknown>> {
    return this.request<Record<string, unknown>>("/api/v1/billing/subscription", { method: "GET" }, options);
  }

  /** Create a Dodo checkout session for plan upgrade. */
  async createCheckout(
    request: import("./billing.js").CheckoutCreateRequest,
    options?: RequestOptions,
  ): Promise<import("./billing.js").CheckoutCreateResponse> {
    return this.request<import("./billing.js").CheckoutCreateResponse>(
      "/api/v1/billing/checkout",
      { method: "POST", body: JSON.stringify(request) },
      options,
    );
  }

  /** Get the Dodo billing portal URL for subscription management. */
  async getBillingPortal(options?: RequestOptions): Promise<import("./billing.js").PortalCreateResponse> {
    return this.request<import("./billing.js").PortalCreateResponse>(
      "/api/v1/billing/portal",
      { method: "POST", body: JSON.stringify({}) },
      options,
    );
  }

  // ── Readings ─────────────────────────────────────────────────────────────

  /** List the authenticated user's saved readings. */
  async listReadings(opts?: ListReadingsOptions, options?: RequestOptions): Promise<Reading[]> {
    const params = new URLSearchParams();
    if (opts?.page) params.set("page", String(opts.page));
    if (opts?.per_page) params.set("per_page", String(opts.per_page));
    if (opts?.workflow_id) params.set("workflow_id", opts.workflow_id);
    const qs = params.toString();
    return this.request<Reading[]>(`/api/v1/readings${qs ? `?${qs}` : ""}`, { method: "GET" }, options);
  }

  /** Get a single reading by ID. */
  async getReading(readingId: string, options?: RequestOptions): Promise<ReadingDetail> {
    return this.request<ReadingDetail>(`/api/v1/readings/${readingId}`, { method: "GET" }, options);
  }

  // ── Witness ───────────────────────────────────────────────────────────────

  /** Get a witness interpretation for arbitrary text or a reading. */
  async interpretWitness(
    input: { text?: string; reading_id?: string; context?: Record<string, unknown> },
    options?: RequestOptions,
  ): Promise<WitnessInterpretation> {
    return this.request<WitnessInterpretation>(
      "/api/v1/witness/interpret",
      { method: "POST", body: JSON.stringify(input) },
      options,
    );
  }

  // ── Validation ────────────────────────────────────────────────────────────

  /** Validate that an engine ID exists and is operational. Returns 200 or throws. */
  async validateEngine(engineId: EngineId | string, options?: RequestOptions): Promise<{ valid: boolean }> {
    try {
      await this.getEngineInfo(engineId, options);
      return { valid: true };
    } catch {
      return { valid: false };
    }
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
