// Inlined from packages/noesis-sdk-ts — keeps biofield-web self-contained for Vercel monorepo builds.
// Update this file whenever packages/noesis-sdk-ts is updated.

// ── Billing ──────────────────────────────────────────────────────────────────

export const BILLING_PROVIDER = "dodo_payments" as const;
export type BillingProvider = typeof BILLING_PROVIDER;

export const PLAN_CODES = ["free", "basic", "premium", "enterprise"] as const;
export type PlanCode = (typeof PLAN_CODES)[number];

export const SUBSCRIPTION_STATUSES = [
  "incomplete",
  "trialing",
  "active",
  "past_due",
  "canceled",
  "expired",
] as const;
export type SubscriptionStatus = (typeof SUBSCRIPTION_STATUSES)[number];

export const DODO_INBOUND_EVENT_TYPES = [
  "subscription.active",
  "subscription.updated",
  "subscription.on_hold",
  "subscription.cancelled",
  "subscription.failed",
  "payment.succeeded",
  "payment.failed",
  "credit.added",
  "credit.deducted",
  "credit.balance_low",
  "credit.overage_charged",
] as const;
export type DodoInboundEventType = (typeof DODO_INBOUND_EVENT_TYPES)[number];

export interface BillingForwardRequest {
  webhook_id: string;
  webhook_timestamp: string;
  event_type: DodoInboundEventType;
  payload: Record<string, unknown>;
}
export type BillingForwardResponse = { status: "ok" } | { status: "dedup" };

export interface UsageEventMetadata {
  engine_id: string;
  tier: PlanCode;
  internal_user_id: string;
}
export interface UsageEvent {
  event_id: string;
  customer_id: string;
  event_name: "noesis.engine_query";
  timestamp: string;
  metadata: UsageEventMetadata;
}
export interface UsageIngestRequest {
  events: UsageEvent[];
}
export interface BalanceResponse {
  credits_remaining: number;
  overage_charged: string;
  period_end: string | null;
  tier: PlanCode;
  cancel_at_period_end: boolean;
  source: "dodo" | "tier_default";
}
export interface CheckoutCreateRequest {
  plan_code: PlanCode;
}
export interface CheckoutCreateResponse {
  checkout_url: string;
}
export interface PortalCreateResponse {
  portal_url: string;
}

// ── Engine / Workflow constants ───────────────────────────────────────────────

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
  "raaga",
  "sacred-geometry",
  "sigil-forge",
  "tarot",
  "transits",
  "vedic-clock",
  "vimshottari",
] as const;
export type EngineId = (typeof ENGINE_IDS)[number];

export const WORKFLOW_IDS = [
  "birth-blueprint",
  "creative-expression",
  "daily-practice",
  "decision-support",
  "full-spectrum",
  "self-inquiry",
] as const;
export type WorkflowId = (typeof WORKFLOW_IDS)[number];

// ── Domain types ─────────────────────────────────────────────────────────────

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
  engine_outputs?: Record<string, EngineOutput>;
  engine_results?: Record<string, EngineOutput>;
  synthesis?: string;
  timestamp?: string;
  total_time_ms?: number;
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
export interface RateLimitInfo {
  limit?: number;
  remaining?: number;
  reset?: number;
  dailyRemaining?: number;
  dailyReset?: number;
}
export interface RegisterRequest { email: string; password: string; full_name: string; }
export interface RegisterResponse { user_id: string; email: string; }
export interface LoginRequest { email: string; password: string; }
export interface LoginResponse { token: string; user_id: string; email: string; tier: string; }
export interface ForgotPasswordRequest { email: string; }
export interface ForgotPasswordResponse { message: string; }
export interface ResetPasswordRequest { token: string; new_password: string; }
export interface ResetPasswordResponse { message: string; }
export interface ChangePasswordRequest { current_password: string; new_password: string; }
export interface ChangePasswordResponse { message: string; }
export interface UserProfileFull {
  id: string;
  email: string;
  full_name?: string;
  tier: string;
  created_at?: string;
}
export interface UpdateUserRequest { full_name?: string; }
export interface UserUsageResponse { [key: string]: unknown; }
export interface ListReadingsOptions { page?: number; per_page?: number; workflow_id?: string; }
export interface Reading {
  id: string;
  workflow_id: string;
  created_at: string;
  birth_data?: BirthData;
  [key: string]: unknown;
}
export interface ReadingDetail extends Reading {
  engine_outputs?: Record<string, EngineOutput>;
}
export interface ReadingsStatsResponse { [key: string]: unknown; }
export interface WitnessInterpretation { [key: string]: unknown; }
export interface RequestOptions { signal?: AbortSignal; }
export interface NoesisClientOptions {
  authToken?: string;
  apiKey?: string;
  maxRetries?: number;
  backoffMs?: number;
}

// ── Errors ───────────────────────────────────────────────────────────────────

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

// ── NoesisClient ─────────────────────────────────────────────────────────────

export class NoesisClient {
  private readonly baseUrl: string;
  private readonly authToken?: string;
  private readonly apiKey?: string;
  private readonly maxRetries: number;
  private readonly backoffMs: number;
  rateLimitInfo: RateLimitInfo = {};

  constructor(baseUrl: string, options: string | NoesisClientOptions = {}) {
    this.baseUrl = baseUrl;
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
    return this.request("/health/live", { method: "GET" }, options);
  }
  async calculate(engineId: EngineId | string, input: EngineInput, options?: RequestOptions): Promise<EngineOutput> {
    return this.request(`/api/v1/engines/${engineId}/calculate`, { method: "POST", body: JSON.stringify(input) }, options);
  }
  async workflow(workflowId: WorkflowId | string, input: EngineInput, options?: RequestOptions): Promise<WorkflowResult> {
    return this.request(`/api/v1/workflows/${workflowId}/execute`, { method: "POST", body: JSON.stringify(input) }, options);
  }
  async listEngines(options?: RequestOptions): Promise<EngineInfo[]> {
    return this.request("/api/v1/engines", { method: "GET" }, options);
  }
  async listWorkflows(options?: RequestOptions): Promise<WorkflowInfo[]> {
    return this.request("/api/v1/workflows", { method: "GET" }, options);
  }
  async getEngineInfo(engineId: EngineId | string, options?: RequestOptions): Promise<EngineInfo> {
    return this.request(`/api/v1/engines/${engineId}/info`, { method: "GET" }, options);
  }
  async getWorkflowInfo(workflowId: WorkflowId | string, options?: RequestOptions): Promise<WorkflowInfo> {
    return this.request(`/api/v1/workflows/${workflowId}`, { method: "GET" }, options);
  }
  async register(request: RegisterRequest, options?: RequestOptions): Promise<RegisterResponse> {
    return this.request("/api/v1/auth/register", { method: "POST", body: JSON.stringify(request) }, options);
  }
  async login(request: LoginRequest, options?: RequestOptions): Promise<LoginResponse> {
    return this.request("/api/v1/auth/login", { method: "POST", body: JSON.stringify(request) }, options);
  }
  async forgotPassword(request: ForgotPasswordRequest, options?: RequestOptions): Promise<ForgotPasswordResponse> {
    return this.request("/api/v1/auth/forgot-password", { method: "POST", body: JSON.stringify(request) }, options);
  }
  async resetPassword(request: ResetPasswordRequest, options?: RequestOptions): Promise<ResetPasswordResponse> {
    return this.request("/api/v1/auth/reset-password", { method: "POST", body: JSON.stringify(request) }, options);
  }
  async changePassword(request: ChangePasswordRequest, options?: RequestOptions): Promise<ChangePasswordResponse> {
    return this.request("/api/v1/auth/change-password", { method: "POST", body: JSON.stringify(request) }, options);
  }
  async getMe(options?: RequestOptions): Promise<UserProfileFull> {
    return this.request("/api/v1/users/me", { method: "GET" }, options);
  }
  async updateMe(request: UpdateUserRequest, options?: RequestOptions): Promise<UserProfileFull> {
    return this.request("/api/v1/users/me", { method: "PUT", body: JSON.stringify(request) }, options);
  }
  async getMyUsage(options?: RequestOptions): Promise<UserUsageResponse> {
    return this.request("/api/v1/users/me/usage", { method: "GET" }, options);
  }
  async getBillingBalance(options?: RequestOptions): Promise<BalanceResponse> {
    return this.request("/api/v1/billing/balance", { method: "GET" }, options);
  }
  async getBillingSubscription(options?: RequestOptions): Promise<BalanceResponse> {
    return this.getBillingBalance(options);
  }
  async createCheckout(request: CheckoutCreateRequest, options?: RequestOptions): Promise<CheckoutCreateResponse> {
    return this.request("/api/v1/billing/checkout", { method: "POST", body: JSON.stringify(request) }, options);
  }
  async getBillingPortal(options?: RequestOptions): Promise<PortalCreateResponse> {
    return this.request("/api/v1/billing/portal", { method: "POST", body: JSON.stringify({}) }, options);
  }
  async listReadings(opts?: ListReadingsOptions, options?: RequestOptions): Promise<Reading[]> {
    const params = new URLSearchParams();
    if (opts?.page) params.set("page", String(opts.page));
    if (opts?.per_page) params.set("per_page", String(opts.per_page));
    if (opts?.workflow_id) params.set("workflow_id", opts.workflow_id);
    const qs = params.toString();
    return this.request(`/api/v1/readings${qs ? `?${qs}` : ""}`, { method: "GET" }, options);
  }
  async getReading(readingId: string, options?: RequestOptions): Promise<ReadingDetail> {
    return this.request(`/api/v1/readings/${readingId}`, { method: "GET" }, options);
  }
  async getReadingsStats(options?: RequestOptions): Promise<ReadingsStatsResponse> {
    return this.request("/api/v1/readings/stats", { method: "GET" }, options);
  }
  async interpretWitness(
    input: { text?: string; reading_id?: string; context?: Record<string, unknown> },
    options?: RequestOptions,
  ): Promise<WitnessInterpretation> {
    return this.request("/api/v1/witness/interpret", { method: "POST", body: JSON.stringify(input) }, options);
  }
  async validateEngine(engineId: EngineId | string, options?: RequestOptions): Promise<{ valid: boolean }> {
    try {
      await this.getEngineInfo(engineId, options);
      return { valid: true };
    } catch {
      return { valid: false };
    }
  }

  private async request<T>(path: string, init: RequestInit, options?: RequestOptions): Promise<T> {
    const headers = new Headers(init.headers ?? {});
    headers.set("Content-Type", "application/json");
    if (this.apiKey) {
      headers.set("X-API-Key", this.apiKey);
    } else if (this.authToken) {
      headers.set("Authorization", `Bearer ${this.authToken}`);
    }

    let attempt = 0;
    while (true) {
      const response = await fetch(`${this.baseUrl}${path}`, { ...init, headers, signal: options?.signal });
      this.captureRateLimit(response.headers);
      const text = await response.text();
      const payload = text ? JSON.parse(text) : {};
      if (response.ok) return payload as T;

      const shouldRetry = response.status >= 500 && attempt < this.maxRetries && !options?.signal?.aborted;
      if (!shouldRetry) {
        throw new SelemeneError(`Request failed: ${response.status}`, response.status, payload);
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
    await new Promise<void>((resolve) => setTimeout(resolve, ms));
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
