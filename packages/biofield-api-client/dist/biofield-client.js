export class BiofieldClientError extends Error {
    status;
    details;
    constructor(message, status, details) {
        super(message);
        this.name = "BiofieldClientError";
        this.status = status;
        this.details = details;
    }
}
export class BiofieldClient {
    baseUrl;
    authToken;
    fetchImpl;
    constructor(baseUrl, options = {}) {
        this.baseUrl = baseUrl;
        this.authToken = options.authToken;
        this.fetchImpl = options.fetchImpl ?? fetch;
    }
    async createSession(input = {}) {
        return this.request("/api/v1/biofield/sessions", {
            method: "POST",
            body: JSON.stringify(input),
        });
    }
    async closeSession(sessionId, input = {}) {
        return this.request(`/api/v1/biofield/sessions/${sessionId}/close`, {
            method: "POST",
            body: JSON.stringify(input),
        });
    }
    async getSession(sessionId) {
        return this.request(`/api/v1/biofield/sessions/${sessionId}`, {
            method: "GET",
        });
    }
    async listReadings(params = {}) {
        const search = new URLSearchParams();
        if (params.limit !== undefined) {
            search.set("limit", String(params.limit));
        }
        if (params.offset !== undefined) {
            search.set("offset", String(params.offset));
        }
        const suffix = search.size > 0 ? `?${search.toString()}` : "";
        return this.request(`/api/v1/biofield/readings${suffix}`, {
            method: "GET",
        });
    }
    async getReading(readingId) {
        return this.request(`/api/v1/biofield/readings/${readingId}`, {
            method: "GET",
        });
    }
    async uploadCapture(sessionId, payload) {
        return this.request(`/api/v1/biofield/sessions/${sessionId}/captures`, {
            method: "POST",
            body: payload,
        }, false);
    }
    async request(path, init, jsonBody = true) {
        const headers = new Headers(init.headers ?? {});
        if (jsonBody && !headers.has("Content-Type")) {
            headers.set("Content-Type", "application/json");
        }
        if (this.authToken) {
            headers.set("Authorization", `Bearer ${this.authToken}`);
        }
        const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
            ...init,
            headers,
        });
        const text = await response.text();
        const payload = text ? JSON.parse(text) : null;
        if (!response.ok) {
            throw new BiofieldClientError(`Request failed: ${response.status}`, response.status, payload);
        }
        return payload;
    }
}
//# sourceMappingURL=biofield-client.js.map