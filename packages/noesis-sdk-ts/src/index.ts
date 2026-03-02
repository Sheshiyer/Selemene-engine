export interface BirthData {
  date: string;
  time: string;
  latitude: number;
  longitude: number;
  timezone: string;
}

export interface EngineInput {
  birth_data: BirthData;
  [key: string]: unknown;
}

export interface EngineOutput {
  engine_id: string;
  result: Record<string, unknown>;
  witness_prompts?: string[];
}

export interface WorkflowResult {
  workflow_id: string;
  engine_results: EngineOutput[];
  synthesis?: string;
}

export interface HealthResponse {
  status: string;
  version: string;
  uptime_seconds: number;
  engines_loaded: number;
  workflows_loaded: number;
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
  constructor(
    private readonly baseUrl: string,
    private readonly authToken?: string,
  ) {}

  async health(): Promise<HealthResponse> {
    return this.request<HealthResponse>("/health/live", { method: "GET" });
  }

  async calculate(engineId: string, input: EngineInput): Promise<EngineOutput> {
    return this.request<EngineOutput>(`/api/v1/engines/${engineId}/calculate`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }

  async workflow(workflowId: string, input: EngineInput): Promise<WorkflowResult> {
    return this.request<WorkflowResult>(`/api/v1/workflows/${workflowId}/execute`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }

  private async request<T>(path: string, init: RequestInit): Promise<T> {
    const headers = new Headers(init.headers ?? {});
    headers.set("Content-Type", "application/json");
    if (this.authToken) {
      headers.set("Authorization", `Bearer ${this.authToken}`);
    }

    const response = await fetch(`${this.baseUrl}${path}`, {
      ...init,
      headers,
    });

    const text = await response.text();
    const payload = text ? JSON.parse(text) : {};

    if (!response.ok) {
      throw new SelemeneError(
        `Request failed: ${response.status}`,
        response.status,
        payload,
      );
    }

    return payload as T;
  }
}
