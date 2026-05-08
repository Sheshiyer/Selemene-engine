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
  /**
   * Engine results keyed by engine_id.
   * Both `engine_outputs` and `engine_results` carry the same data;
   * prefer `engine_outputs` but fall back to `engine_results` for compatibility.
   */
  engine_outputs?: Record<string, EngineOutput>;
  engine_results?: Record<string, EngineOutput>;
  synthesis?: string;
  timestamp?: string;
  total_time_ms?: number;
  /** v3.3.0: Reading persistence fields (requires witness-agents deploy — issue #711) */
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

// ── Auth request/response types ──────────────────────────────────────────────

export interface RegisterRequest {
  email: string;
  password: string;
  full_name: string;
}

export interface RegisterResponse {
  id: string;
  message: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  user_id: string;
  email: string;
  tier: string;
}

export interface ForgotPasswordRequest {
  email: string;
}

export interface ForgotPasswordResponse {
  message: string;
}

export interface ResetPasswordRequest {
  token: string;
  new_password: string;
}

export interface ResetPasswordResponse {
  message: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

export interface ChangePasswordResponse {
  message: string;
}

// ── User profile types ────────────────────────────────────────────────────────

/** Slim profile used internally. For the full profile, use UserProfile. */
export interface UserProfile {
  id: string;
  email?: string;
  role?: string;
  created_at?: string;
}

/** Full user profile returned by GET /api/v1/users/me */
export interface UserProfileFull {
  id: string;
  email: string;
  full_name: string;
  tier: string;
  consciousness_level: number;
  experience_points: number;
  birth_date?: string | null;
  birth_time?: string | null;
  birth_location?: { lat: number; lng: number; name?: string | null } | null;
  timezone?: string | null;
  preferences: Record<string, unknown>;
}

export interface UpdateUserRequest {
  full_name?: string;
  email?: string;
  birth_date?: string;
  birth_time?: string;
  birth_location_lat?: number;
  birth_location_lng?: number;
  birth_location_name?: string;
  timezone?: string;
  preferences?: Record<string, unknown>;
}

export interface UsageSummary {
  total_calls: number;
  calls_today?: number;
  credits_used?: number;
  period_start?: string;
  period_end?: string;
}

// ── Usage analytics types ─────────────────────────────────────────────────────

export interface UserUsageWindowSummary {
  total: number;
  success: number;
  failure: number;
}

export interface UserUsageEngineEntry {
  engine_id: string;
  request_count: number;
}

export interface UserUsageResponse {
  user_id: string;
  daily: UserUsageWindowSummary;
  monthly: UserUsageWindowSummary;
  engine_breakdown: UserUsageEngineEntry[];
}

// ── Readings stats types ──────────────────────────────────────────────────────

export interface ReadingsStatsEntry {
  engine_id: string;
  count: number;
}

export interface ReadingsStatsResponse {
  stats: ReadingsStatsEntry[];
  total: number;
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
  /** JWT bearer token (Authorization: Bearer <token>) */
  authToken?: string;
  /** API key (X-API-Key: nk_...). Takes precedence over authToken when both are set. */
  apiKey?: string;
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
  private readonly apiKey?: string;
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
      this.apiKey = options.apiKey;
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
    return this.request<EngineInfo>(`/api/v1/engines/${engineId}/info`, { method: "GET" }, options);
  }

  /** Get workflow metadata by ID. */
  async getWorkflowInfo(workflowId: WorkflowId | string, options?: RequestOptions): Promise<WorkflowInfo> {
    return this.request<WorkflowInfo>(`/api/v1/workflows/${workflowId}`, { method: "GET" }, options);
  }

  // ── Auth ─────────────────────────────────────────────────────────────────────

  /** Register a new user account. Returns the new user ID. */
  async register(request: RegisterRequest, options?: RequestOptions): Promise<RegisterResponse> {
    return this.request<RegisterResponse>(
      "/api/v1/auth/register",
      { method: "POST", body: JSON.stringify(request) },
      options,
    );
  }

  /** Log in with email + password. Returns a JWT token. */
  async login(request: LoginRequest, options?: RequestOptions): Promise<LoginResponse> {
    return this.request<LoginResponse>(
      "/api/v1/auth/login",
      { method: "POST", body: JSON.stringify(request) },
      options,
    );
  }

  /** Initiate a password reset flow. */
  async forgotPassword(request: ForgotPasswordRequest, options?: RequestOptions): Promise<ForgotPasswordResponse> {
    return this.request<ForgotPasswordResponse>(
      "/api/v1/auth/forgot-password",
      { method: "POST", body: JSON.stringify(request) },
      options,
    );
  }

  /** Complete a password reset with the token received by email. */
  async resetPassword(request: ResetPasswordRequest, options?: RequestOptions): Promise<ResetPasswordResponse> {
    return this.request<ResetPasswordResponse>(
      "/api/v1/auth/reset-password",
      { method: "POST", body: JSON.stringify(request) },
      options,
    );
  }

  /** Change password for an already-authenticated user. */
  async changePassword(request: ChangePasswordRequest, options?: RequestOptions): Promise<ChangePasswordResponse> {
    return this.request<ChangePasswordResponse>(
      "/api/v1/auth/change-password",
      { method: "POST", body: JSON.stringify(request) },
      options,
    );
  }

  // ── Auth & user ─────────────────────────────────────────────────────────

  /** Get the authenticated user's full profile. */
  async getMe(options?: RequestOptions): Promise<UserProfileFull> {
    return this.request<UserProfileFull>("/api/v1/users/me", { method: "GET" }, options);
  }

  /** Update the authenticated user's profile. */
  async updateMe(request: UpdateUserRequest, options?: RequestOptions): Promise<UserProfileFull> {
    return this.request<UserProfileFull>(
      "/api/v1/users/me",
      { method: "PUT", body: JSON.stringify(request) },
      options,
    );
  }

  /** Get the authenticated user's usage analytics. */
  async getMyUsage(options?: RequestOptions): Promise<UserUsageResponse> {
    return this.request<UserUsageResponse>("/api/v1/users/me/usage", { method: "GET" }, options);
  }

  // ── Billing ──────────────────────────────────────────────────────────────

  /** Get the user's current credit balance and subscription info. */
  async getBillingBalance(options?: RequestOptions): Promise<import("./billing.js").BalanceResponse> {
    return this.request<import("./billing.js").BalanceResponse>("/api/v1/billing/balance", { method: "GET" }, options);
  }

  /** Get the user's active subscription details. */
  async getBillingSubscription(options?: RequestOptions): Promise<import("./billing.js").BalanceResponse> {
    // Alias to getBillingBalance — /billing/subscription does not exist as a separate endpoint.
    return this.getBillingBalance(options);
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

  /** Get readings count per engine for the authenticated user. */
  async getReadingsStats(options?: RequestOptions): Promise<ReadingsStatsResponse> {
    return this.request<ReadingsStatsResponse>("/api/v1/readings/stats", { method: "GET" }, options);
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

    if (this.apiKey) {
      headers.set("X-API-Key", this.apiKey);
    } else if (this.authToken) {
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
