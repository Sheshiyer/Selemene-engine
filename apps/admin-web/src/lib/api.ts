import { getApiBaseUrl } from "@/lib/config";
import type { AdminSession, ApiErrorPayload, LoginResponse } from "@/types/admin";

type HttpMethod = "GET" | "POST" | "PATCH" | "PUT" | "DELETE";

class ApiClientError extends Error {
  status: number;
  payload?: ApiErrorPayload;

  constructor(message: string, status: number, payload?: ApiErrorPayload) {
    super(message);
    this.name = "ApiClientError";
    this.status = status;
    this.payload = payload;
  }
}

async function request<T>(
  path: string,
  options: {
    method?: HttpMethod;
    token?: string;
    body?: unknown;
  } = {}
): Promise<T> {
  const baseUrl = getApiBaseUrl();
  const url = `${baseUrl}${path}`;

  const response = await fetch(url, {
    method: options.method ?? "GET",
    headers: {
      "Content-Type": "application/json",
      ...(options.token ? { Authorization: `Bearer ${options.token}` } : {})
    },
    body: options.body ? JSON.stringify(options.body) : undefined,
    cache: "no-store"
  });

  const contentType = response.headers.get("content-type");
  const hasJson = contentType?.includes("application/json");
  const payload = hasJson ? ((await response.json()) as ApiErrorPayload) : undefined;

  if (!response.ok) {
    throw new ApiClientError(
      payload?.error || `Request failed with status ${response.status}`,
      response.status,
      payload
    );
  }

  return (payload as T) ?? ({} as T);
}

export async function login(email: string, password: string): Promise<LoginResponse> {
  return request<LoginResponse>("/api/v1/auth/login", {
    method: "POST",
    body: { email, password }
  });
}

export async function getAdminSession(token: string): Promise<AdminSession> {
  return request<AdminSession>("/api/v1/admin/session", {
    method: "GET",
    token
  });
}

export { ApiClientError };
