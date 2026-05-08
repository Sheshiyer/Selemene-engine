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
    "raaga",
    "sacred-geometry",
    "sigil-forge",
    "tarot",
    "transits",
    "vedic-clock",
    "vimshottari",
];
export const WORKFLOW_IDS = [
    "birth-blueprint",
    "creative-expression",
    "daily-practice",
    "decision-support",
    "full-spectrum",
    "self-inquiry",
];
export class SelemeneError extends Error {
    status;
    details;
    constructor(message, status, details) {
        super(message);
        this.name = "SelemeneError";
        this.status = status;
        this.details = details;
    }
}
export class NoesisClient {
    baseUrl;
    authToken;
    apiKey;
    maxRetries;
    backoffMs;
    rateLimitInfo = {};
    constructor(baseUrl, options = {}) {
        this.baseUrl = baseUrl;
        if (typeof options === "string") {
            this.authToken = options;
            this.maxRetries = 0;
            this.backoffMs = 150;
        }
        else {
            this.authToken = options.authToken;
            this.apiKey = options.apiKey;
            this.maxRetries = options.maxRetries ?? 0;
            this.backoffMs = options.backoffMs ?? 150;
        }
    }
    async health(options) {
        return this.request("/health/live", { method: "GET" }, options);
    }
    async calculate(engineId, input, options) {
        return this.request(`/api/v1/engines/${engineId}/calculate`, {
            method: "POST",
            body: JSON.stringify(input),
        }, options);
    }
    async workflow(workflowId, input, options) {
        return this.request(`/api/v1/workflows/${workflowId}/execute`, {
            method: "POST",
            body: JSON.stringify(input),
        }, options);
    }
    /** List all available engines. */
    async listEngines(options) {
        return this.request("/api/v1/engines", { method: "GET" }, options);
    }
    /** List all available workflows. */
    async listWorkflows(options) {
        return this.request("/api/v1/workflows", { method: "GET" }, options);
    }
    /** Get engine metadata by ID. */
    async getEngineInfo(engineId, options) {
        return this.request(`/api/v1/engines/${engineId}/info`, { method: "GET" }, options);
    }
    /** Get workflow metadata by ID. */
    async getWorkflowInfo(workflowId, options) {
        return this.request(`/api/v1/workflows/${workflowId}`, { method: "GET" }, options);
    }
    // ── Auth ─────────────────────────────────────────────────────────────────────
    /** Register a new user account. Returns the new user ID. */
    async register(request, options) {
        return this.request("/api/v1/auth/register", { method: "POST", body: JSON.stringify(request) }, options);
    }
    /** Log in with email + password. Returns a JWT token. */
    async login(request, options) {
        return this.request("/api/v1/auth/login", { method: "POST", body: JSON.stringify(request) }, options);
    }
    /** Initiate a password reset flow. */
    async forgotPassword(request, options) {
        return this.request("/api/v1/auth/forgot-password", { method: "POST", body: JSON.stringify(request) }, options);
    }
    /** Complete a password reset with the token received by email. */
    async resetPassword(request, options) {
        return this.request("/api/v1/auth/reset-password", { method: "POST", body: JSON.stringify(request) }, options);
    }
    /** Change password for an already-authenticated user. */
    async changePassword(request, options) {
        return this.request("/api/v1/auth/change-password", { method: "POST", body: JSON.stringify(request) }, options);
    }
    // ── Auth & user ─────────────────────────────────────────────────────────
    /** Get the authenticated user's full profile. */
    async getMe(options) {
        return this.request("/api/v1/users/me", { method: "GET" }, options);
    }
    /** Update the authenticated user's profile. */
    async updateMe(request, options) {
        return this.request("/api/v1/users/me", { method: "PUT", body: JSON.stringify(request) }, options);
    }
    /** Get the authenticated user's usage analytics. */
    async getMyUsage(options) {
        return this.request("/api/v1/users/me/usage", { method: "GET" }, options);
    }
    // ── Billing ──────────────────────────────────────────────────────────────
    /** Get the user's current credit balance and subscription info. */
    async getBillingBalance(options) {
        return this.request("/api/v1/billing/balance", { method: "GET" }, options);
    }
    /** Get the user's active subscription details. */
    async getBillingSubscription(options) {
        // Alias to getBillingBalance — /billing/subscription does not exist as a separate endpoint.
        return this.getBillingBalance(options);
    }
    /** Create a Dodo checkout session for plan upgrade. */
    async createCheckout(request, options) {
        return this.request("/api/v1/billing/checkout", { method: "POST", body: JSON.stringify(request) }, options);
    }
    /** Get the Dodo billing portal URL for subscription management. */
    async getBillingPortal(options) {
        return this.request("/api/v1/billing/portal", { method: "POST", body: JSON.stringify({}) }, options);
    }
    // ── Readings ─────────────────────────────────────────────────────────────
    /** List the authenticated user's saved readings. */
    async listReadings(opts, options) {
        const params = new URLSearchParams();
        if (opts?.page)
            params.set("page", String(opts.page));
        if (opts?.per_page)
            params.set("per_page", String(opts.per_page));
        if (opts?.workflow_id)
            params.set("workflow_id", opts.workflow_id);
        const qs = params.toString();
        return this.request(`/api/v1/readings${qs ? `?${qs}` : ""}`, { method: "GET" }, options);
    }
    /** Get a single reading by ID. */
    async getReading(readingId, options) {
        return this.request(`/api/v1/readings/${readingId}`, { method: "GET" }, options);
    }
    /** Get readings count per engine for the authenticated user. */
    async getReadingsStats(options) {
        return this.request("/api/v1/readings/stats", { method: "GET" }, options);
    }
    // ── Witness ───────────────────────────────────────────────────────────────
    /** Get a witness interpretation for arbitrary text or a reading. */
    async interpretWitness(input, options) {
        return this.request("/api/v1/witness/interpret", { method: "POST", body: JSON.stringify(input) }, options);
    }
    // ── Validation ────────────────────────────────────────────────────────────
    /** Validate that an engine ID exists and is operational. Returns 200 or throws. */
    async validateEngine(engineId, options) {
        try {
            await this.getEngineInfo(engineId, options);
            return { valid: true };
        }
        catch {
            return { valid: false };
        }
    }
    async request(path, init, options) {
        const headers = new Headers(init.headers ?? {});
        headers.set("Content-Type", "application/json");
        if (this.apiKey) {
            headers.set("X-API-Key", this.apiKey);
        }
        else if (this.authToken) {
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
                return payload;
            }
            const shouldRetry = response.status >= 500 &&
                attempt < this.maxRetries &&
                !options?.signal?.aborted;
            if (!shouldRetry) {
                throw new SelemeneError(`Request failed: ${response.status}`, response.status, payload);
            }
            attempt += 1;
            const waitMs = this.backoffMs * 2 ** (attempt - 1);
            await delay(waitMs, options?.signal);
        }
    }
    captureRateLimit(headers) {
        this.rateLimitInfo = {
            limit: toNumber(headers.get("x-ratelimit-limit")),
            remaining: toNumber(headers.get("x-ratelimit-remaining")),
            reset: toNumber(headers.get("x-ratelimit-reset")),
            dailyRemaining: toNumber(headers.get("x-ratelimit-daily-remaining")),
            dailyReset: toNumber(headers.get("x-ratelimit-daily-reset")),
        };
    }
}
function toNumber(value) {
    if (!value)
        return undefined;
    const n = Number(value);
    return Number.isFinite(n) ? n : undefined;
}
async function delay(ms, signal) {
    if (!signal) {
        await new Promise((resolve) => setTimeout(resolve, ms));
        return;
    }
    await new Promise((resolve, reject) => {
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
//# sourceMappingURL=index.js.map