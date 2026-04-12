import type {
  BiofieldCaptureResult,
  BiofieldReadingDetail,
  BiofieldSession,
  CloseBiofieldSessionRequest,
  CreateBiofieldSessionRequest,
  ListBiofieldReadingsResponse,
} from "@selemene/biofield-domain";

export interface BiofieldClientOptions {
  authToken?: string;
  fetchImpl?: typeof fetch;
}

export interface ListBiofieldReadingsParams {
  limit?: number;
  offset?: number;
}

interface ErrorPayloadShape {
  message?: string;
  error_message?: string;
  error?: string;
  detail?: string;
}

export class BiofieldClientError extends Error {
  readonly status: number;
  readonly details?: unknown;

  constructor(message: string, status: number, details?: unknown) {
    super(message);
    this.name = "BiofieldClientError";
    this.status = status;
    this.details = details;
  }
}

export class BiofieldClient {
  private readonly authToken?: string;
  private readonly fetchImpl: typeof fetch;

  constructor(
    private readonly baseUrl: string,
    options: BiofieldClientOptions = {},
  ) {
    this.authToken = options.authToken;
    this.fetchImpl = options.fetchImpl ?? fetch;
  }

  async createSession(input: CreateBiofieldSessionRequest = {}): Promise<BiofieldSession> {
    return this.request<BiofieldSession>("/api/v1/biofield/sessions", {
      method: "POST",
      body: JSON.stringify(input),
    });
  }

  async closeSession(
    sessionId: string,
    input: CloseBiofieldSessionRequest = {},
  ): Promise<BiofieldSession> {
    return this.request<BiofieldSession>(`/api/v1/biofield/sessions/${sessionId}/close`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }

  async getSession(sessionId: string): Promise<BiofieldSession> {
    return this.request<BiofieldSession>(`/api/v1/biofield/sessions/${sessionId}`, {
      method: "GET",
    });
  }

  async listReadings(
    params: ListBiofieldReadingsParams = {},
  ): Promise<ListBiofieldReadingsResponse> {
    const search = new URLSearchParams();

    if (params.limit !== undefined) {
      search.set("limit", String(params.limit));
    }

    if (params.offset !== undefined) {
      search.set("offset", String(params.offset));
    }

    const suffix = search.size > 0 ? `?${search.toString()}` : "";

    return this.request<ListBiofieldReadingsResponse>(`/api/v1/biofield/readings${suffix}`, {
      method: "GET",
    });
  }

  async getReading(readingId: string): Promise<BiofieldReadingDetail> {
    return this.request<BiofieldReadingDetail>(`/api/v1/biofield/readings/${readingId}`, {
      method: "GET",
    });
  }

  async uploadCapture(
    sessionId: string,
    payload: FormData,
  ): Promise<BiofieldCaptureResult> {
    return this.request<BiofieldCaptureResult>(
      `/api/v1/biofield/sessions/${sessionId}/captures`,
      {
        method: "POST",
        body: payload,
      },
      false,
    );
  }

  private async request<T>(
    path: string,
    init: RequestInit,
    jsonBody = true,
  ): Promise<T> {
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
    const payload = parsePayload(text);

    if (!response.ok) {
      throw new BiofieldClientError(
        resolveErrorMessage(payload, response.status),
        response.status,
        payload,
      );
    }

    return payload as T;
  }
}

function parsePayload(text: string): unknown {
  if (!text) {
    return null;
  }

  try {
    return JSON.parse(text);
  } catch {
    return { raw: text };
  }
}

function resolveErrorMessage(payload: unknown, status: number): string {
  if (payload && typeof payload === "object") {
    const candidate = payload as ErrorPayloadShape;
    if (typeof candidate.message === "string" && candidate.message.length > 0) {
      return candidate.message;
    }
    if (typeof candidate.error_message === "string" && candidate.error_message.length > 0) {
      return candidate.error_message;
    }
    if (typeof candidate.error === "string" && candidate.error.length > 0) {
      return candidate.error;
    }
    if (typeof candidate.detail === "string" && candidate.detail.length > 0) {
      return candidate.detail;
    }
  }

  return `Request failed: ${status}`;
}
