export * from "./billing.js";
export declare const ENGINE_IDS: readonly ["biofield", "biorhythm", "enneagram", "face-reading", "gene-keys", "human-design", "i-ching", "nadabrahman", "numerology", "panchanga", "raaga", "sacred-geometry", "sigil-forge", "tarot", "transits", "vedic-clock", "vimshottari"];
export declare const WORKFLOW_IDS: readonly ["birth-blueprint", "creative-expression", "daily-practice", "decision-support", "full-spectrum", "self-inquiry"];
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
    birth_location?: {
        lat: number;
        lng: number;
        name?: string | null;
    } | null;
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
export declare class SelemeneError extends Error {
    readonly status: number;
    readonly details?: unknown;
    constructor(message: string, status: number, details?: unknown);
}
export declare class NoesisClient {
    private readonly baseUrl;
    private readonly authToken?;
    private readonly apiKey?;
    private readonly maxRetries;
    private readonly backoffMs;
    rateLimitInfo: RateLimitInfo;
    constructor(baseUrl: string, options?: string | NoesisClientOptions);
    health(options?: RequestOptions): Promise<HealthResponse>;
    calculate(engineId: EngineId | string, input: EngineInput, options?: RequestOptions): Promise<EngineOutput>;
    workflow(workflowId: WorkflowId | string, input: EngineInput, options?: RequestOptions): Promise<WorkflowResult>;
    /** List all available engines. */
    listEngines(options?: RequestOptions): Promise<EngineInfo[]>;
    /** List all available workflows. */
    listWorkflows(options?: RequestOptions): Promise<WorkflowInfo[]>;
    /** Get engine metadata by ID. */
    getEngineInfo(engineId: EngineId | string, options?: RequestOptions): Promise<EngineInfo>;
    /** Get workflow metadata by ID. */
    getWorkflowInfo(workflowId: WorkflowId | string, options?: RequestOptions): Promise<WorkflowInfo>;
    /** Register a new user account. Returns the new user ID. */
    register(request: RegisterRequest, options?: RequestOptions): Promise<RegisterResponse>;
    /** Log in with email + password. Returns a JWT token. */
    login(request: LoginRequest, options?: RequestOptions): Promise<LoginResponse>;
    /** Initiate a password reset flow. */
    forgotPassword(request: ForgotPasswordRequest, options?: RequestOptions): Promise<ForgotPasswordResponse>;
    /** Complete a password reset with the token received by email. */
    resetPassword(request: ResetPasswordRequest, options?: RequestOptions): Promise<ResetPasswordResponse>;
    /** Change password for an already-authenticated user. */
    changePassword(request: ChangePasswordRequest, options?: RequestOptions): Promise<ChangePasswordResponse>;
    /** Get the authenticated user's full profile. */
    getMe(options?: RequestOptions): Promise<UserProfileFull>;
    /** Update the authenticated user's profile. */
    updateMe(request: UpdateUserRequest, options?: RequestOptions): Promise<UserProfileFull>;
    /** Get the authenticated user's usage analytics. */
    getMyUsage(options?: RequestOptions): Promise<UserUsageResponse>;
    /** Get the user's current credit balance and subscription info. */
    getBillingBalance(options?: RequestOptions): Promise<import("./billing.js").BalanceResponse>;
    /** Get the user's active subscription details. */
    getBillingSubscription(options?: RequestOptions): Promise<import("./billing.js").BalanceResponse>;
    /** Create a Dodo checkout session for plan upgrade. */
    createCheckout(request: import("./billing.js").CheckoutCreateRequest, options?: RequestOptions): Promise<import("./billing.js").CheckoutCreateResponse>;
    /** Get the Dodo billing portal URL for subscription management. */
    getBillingPortal(options?: RequestOptions): Promise<import("./billing.js").PortalCreateResponse>;
    /** List the authenticated user's saved readings. */
    listReadings(opts?: ListReadingsOptions, options?: RequestOptions): Promise<Reading[]>;
    /** Get a single reading by ID. */
    getReading(readingId: string, options?: RequestOptions): Promise<ReadingDetail>;
    /** Get readings count per engine for the authenticated user. */
    getReadingsStats(options?: RequestOptions): Promise<ReadingsStatsResponse>;
    /** Get a witness interpretation for arbitrary text or a reading. */
    interpretWitness(input: {
        text?: string;
        reading_id?: string;
        context?: Record<string, unknown>;
    }, options?: RequestOptions): Promise<WitnessInterpretation>;
    /** Validate that an engine ID exists and is operational. Returns 200 or throws. */
    validateEngine(engineId: EngineId | string, options?: RequestOptions): Promise<{
        valid: boolean;
    }>;
    private request;
    private captureRateLimit;
}
//# sourceMappingURL=index.d.ts.map