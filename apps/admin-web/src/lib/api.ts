import { getApiBaseUrl } from "@/lib/config";
import type {
  AdminAnalyticsBreakdownResponse,
  AdminAnalyticsSummaryResponse,
  AdminAnalyticsTimeseriesResponse,
  AdminAnalyticsTopConsumersResponse,
  AdminApiKeysResponse,
  AdminAuditActionsResponse,
  AdminAuditEventDetailResponse,
  AdminAuditEventsResponse,
  AdminBillingCancelSubscriptionResponse,
  AdminBillingOverviewResponse,
  AdminBillingPlansResponse,
  AdminBillingReconcileDriftResponse,
  AdminBillingReconcileTriggerResponse,
  AdminBillingSubscriptionDetailResponse,
  AdminBillingSubscriptionsResponse,
  AdminBillingWebhookEventsResponse,
  AdminHistorySyncDevicesResponse,
  AdminHistorySyncEventsResponse,
  AdminHistorySyncUsersResponse,
  AdminSession,
  AdminSystemCacheResponse,
  AdminSystemHealthResponse,
  AdminSystemServicesResponse,
  AdminSystemWorkflowsResponse,
  AdminUsageSummaryResponse,
  AdminUsersResponse,
  ApiErrorPayload,
  CreateApiKeyResponse,
  LoginResponse,
  RotateApiKeyResponse,
  UpdateUserRolesResponse,
  UpdateUserStateResponse,
  UpdateUserTierResponse
} from "@/types/admin";

type HttpMethod = "GET" | "POST" | "PATCH" | "PUT" | "DELETE";

const LOCALHOST_HOSTS = new Set(["localhost", "127.0.0.1", "0.0.0.0"]);
const API_PREFIX = "/api";
const API_V1_PREFIX = "/api/v1";

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

function isBrowserRuntime(): boolean {
  return typeof window !== "undefined";
}

function isLikelyProductionOrigin(hostname: string): boolean {
  return !LOCALHOST_HOSTS.has(hostname);
}

function assertApiBaseUrlRuntimeSafety(baseUrl: string): void {
  if (!isBrowserRuntime()) {
    return;
  }

  try {
    const resolved = new URL(baseUrl);
    if (isLikelyProductionOrigin(window.location.hostname) && LOCALHOST_HOSTS.has(resolved.hostname)) {
      throw new ApiClientError(
        "Admin dashboard is misconfigured: NEXT_PUBLIC_API_BASE_URL points to localhost.",
        500,
        {
          error: "Admin dashboard is misconfigured: NEXT_PUBLIC_API_BASE_URL points to localhost.",
          error_code: "ADMIN_ENV_MISCONFIG",
          details: {
            configured_api_base_url: baseUrl,
            current_origin: window.location.origin
          }
        }
      );
    }
  } catch {
    throw new ApiClientError(
      "Admin dashboard is misconfigured: NEXT_PUBLIC_API_BASE_URL is not a valid URL.",
      500,
      {
        error: "Admin dashboard is misconfigured: NEXT_PUBLIC_API_BASE_URL is not a valid URL.",
        error_code: "ADMIN_ENV_MISCONFIG",
        details: {
          configured_api_base_url: baseUrl
        }
      }
    );
  }
}

function normalizePathname(pathname: string): string {
  if (pathname === "/") {
    return pathname;
  }
  return pathname.endsWith("/") ? pathname.slice(0, -1) : pathname;
}

function hasPrefix(pathname: string, prefix: string): boolean {
  return pathname === prefix || pathname.startsWith(`${prefix}/`);
}

function buildApiUrl(baseUrl: string, path: string): string {
  const base = new URL(baseUrl);
  const request = new URL(path, "https://placeholder.internal");

  const basePath = normalizePathname(base.pathname);
  let requestPath = request.pathname;

  // Accept both origin-only and prefixed API base URLs without duplicating path segments.
  if (basePath.endsWith(API_V1_PREFIX) && hasPrefix(requestPath, API_V1_PREFIX)) {
    requestPath = requestPath.slice(API_V1_PREFIX.length) || "/";
  } else if (basePath.endsWith(API_PREFIX) && hasPrefix(requestPath, API_PREFIX)) {
    requestPath = requestPath.slice(API_PREFIX.length) || "/";
  }

  const combinedPath = normalizePathname(
    `${basePath === "/" ? "" : basePath}${requestPath.startsWith("/") ? requestPath : `/${requestPath}`}`
  );

  base.pathname = combinedPath.startsWith("/") ? combinedPath : `/${combinedPath}`;
  base.search = request.search;

  return base.toString();
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
  assertApiBaseUrlRuntimeSafety(baseUrl);

  const url = buildApiUrl(baseUrl, path);

  let response: Response;
  try {
    response = await fetch(url, {
      method: options.method ?? "GET",
      headers: {
        "Content-Type": "application/json",
        ...(options.token ? { Authorization: `Bearer ${options.token}` } : {})
      },
      body: options.body ? JSON.stringify(options.body) : undefined,
      cache: "no-store",
      credentials: "include"
    });
  } catch {
    throw new ApiClientError(
      "Unable to reach the API service. Check NEXT_PUBLIC_API_BASE_URL and backend CORS settings.",
      503,
      {
        error: "Unable to reach the API service. Check NEXT_PUBLIC_API_BASE_URL and backend CORS settings.",
        error_code: "API_UNREACHABLE",
        details: {
          api_url: url
        }
      }
    );
  }

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

function buildQuery(params: Record<string, string | number | boolean | undefined>): string {
  const query = new URLSearchParams();

  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === "") {
      continue;
    }
    query.set(key, String(value));
  }

  const asString = query.toString();
  return asString ? `?${asString}` : "";
}

export async function login(email: string, password: string): Promise<LoginResponse> {
  return request<LoginResponse>("/api/v1/auth/login", {
    method: "POST",
    body: { email, password }
  });
}

export async function getAdminSession(token?: string): Promise<AdminSession> {
  return request<AdminSession>("/api/v1/admin/session", {
    method: "GET",
    ...(token ? { token } : {})
  });
}

export async function getAdminUsers(
  token: string,
  params: {
    query?: string;
    tier?: string;
    state?: string;
    limit?: number;
    offset?: number;
  } = {}
): Promise<AdminUsersResponse> {
  return request<AdminUsersResponse>(
    `/api/v1/admin/users${buildQuery({
      query: params.query,
      tier: params.tier,
      state: params.state,
      limit: params.limit,
      offset: params.offset
    })}`,
    { token }
  );
}

export async function updateAdminUserState(
  token: string,
  userId: string,
  payload: {
    state: "active" | "locked";
    lock_minutes?: number;
  }
): Promise<UpdateUserStateResponse> {
  return request<UpdateUserStateResponse>(`/api/v1/admin/users/${userId}/state`, {
    method: "PATCH",
    token,
    body: payload
  });
}

export async function updateAdminUserTier(
  token: string,
  userId: string,
  payload: { tier: string }
): Promise<UpdateUserTierResponse> {
  return request<UpdateUserTierResponse>(`/api/v1/admin/users/${userId}/tier`, {
    method: "PATCH",
    token,
    body: payload
  });
}

export async function updateAdminUserRoles(
  token: string,
  userId: string,
  payload: { roles: string[] }
): Promise<UpdateUserRolesResponse> {
  return request<UpdateUserRolesResponse>(`/api/v1/admin/users/${userId}/roles`, {
    method: "PUT",
    token,
    body: payload
  });
}

export async function getAdminApiKeys(
  token: string,
  params: {
    query?: string;
    user_id?: string;
    active_only?: boolean;
    limit?: number;
    offset?: number;
  } = {}
): Promise<AdminApiKeysResponse> {
  return request<AdminApiKeysResponse>(
    `/api/v1/admin/api-keys${buildQuery({
      query: params.query,
      user_id: params.user_id,
      active_only: params.active_only,
      limit: params.limit,
      offset: params.offset
    })}`,
    { token }
  );
}

export async function createAdminApiKey(
  token: string,
  payload: {
    user_id: string;
    name?: string;
    tier?: string;
    permissions?: string[];
    consciousness_level?: number;
    rate_limit?: number;
    expires_at?: string;
  }
): Promise<CreateApiKeyResponse> {
  return request<CreateApiKeyResponse>("/api/v1/admin/api-keys", {
    method: "POST",
    token,
    body: payload
  });
}

export async function revokeAdminApiKey(token: string, keyId: string): Promise<void> {
  await request<Record<string, unknown>>(`/api/v1/admin/api-keys/${keyId}/revoke`, {
    method: "POST",
    token
  });
}

export async function rotateAdminApiKey(
  token: string,
  keyId: string
): Promise<RotateApiKeyResponse> {
  return request<RotateApiKeyResponse>(`/api/v1/admin/api-keys/${keyId}/rotate`, {
    method: "POST",
    token
  });
}

export async function deleteAdminApiKey(token: string, keyId: string): Promise<void> {
  await request<Record<string, unknown>>(`/api/v1/admin/api-keys/${keyId}`, {
    method: "DELETE",
    token
  });
}

export async function getHistorySyncUsers(
  token: string,
  params: { limit?: number; offset?: number } = {}
): Promise<AdminHistorySyncUsersResponse> {
  return request<AdminHistorySyncUsersResponse>(
    `/api/v1/admin/history-sync/users${buildQuery({
      limit: params.limit,
      offset: params.offset
    })}`,
    { token }
  );
}

export async function getHistorySyncDevices(
  token: string,
  params: { limit?: number; offset?: number } = {}
): Promise<AdminHistorySyncDevicesResponse> {
  return request<AdminHistorySyncDevicesResponse>(
    `/api/v1/admin/history-sync/devices${buildQuery({
      limit: params.limit,
      offset: params.offset
    })}`,
    { token }
  );
}

export async function getHistorySyncEvents(
  token: string,
  params: { status?: string; limit?: number; offset?: number } = {}
): Promise<AdminHistorySyncEventsResponse> {
  return request<AdminHistorySyncEventsResponse>(
    `/api/v1/admin/history-sync/events${buildQuery({
      status: params.status,
      limit: params.limit,
      offset: params.offset
    })}`,
    { token }
  );
}

export async function getAdminUsageSummary(
  token: string,
  params: { engine_limit?: number; top_users_limit?: number; range_days?: number } = {}
): Promise<AdminUsageSummaryResponse> {
  return request<AdminUsageSummaryResponse>(
    `/api/v1/admin/usage/summary${buildQuery({
      engine_limit: params.engine_limit,
      top_users_limit: params.top_users_limit,
      range_days: params.range_days
    })}`,
    { token }
  );
}

export async function getAnalyticsSummary(
  token: string,
  params: { window_hours?: number } = {}
): Promise<AdminAnalyticsSummaryResponse> {
  return request<AdminAnalyticsSummaryResponse>(
    `/api/v1/admin/analytics/summary${buildQuery({
      window_hours: params.window_hours
    })}`,
    { token }
  );
}

export async function getAnalyticsTimeseries(
  token: string,
  params: { window_hours?: number; bucket?: "hour" | "day" } = {}
): Promise<AdminAnalyticsTimeseriesResponse> {
  return request<AdminAnalyticsTimeseriesResponse>(
    `/api/v1/admin/analytics/usage-timeseries${buildQuery({
      window_hours: params.window_hours,
      bucket: params.bucket
    })}`,
    { token }
  );
}

export async function getAnalyticsBreakdown(
  token: string,
  params: { window_hours?: number; limit?: number } = {}
): Promise<AdminAnalyticsBreakdownResponse> {
  return request<AdminAnalyticsBreakdownResponse>(
    `/api/v1/admin/analytics/usage-breakdown${buildQuery({
      window_hours: params.window_hours,
      limit: params.limit
    })}`,
    { token }
  );
}

export async function getAnalyticsTopConsumers(
  token: string,
  params: { window_hours?: number; limit?: number } = {}
): Promise<AdminAnalyticsTopConsumersResponse> {
  return request<AdminAnalyticsTopConsumersResponse>(
    `/api/v1/admin/analytics/top-consumers${buildQuery({
      window_hours: params.window_hours,
      limit: params.limit
    })}`,
    { token }
  );
}

export async function getSystemHealth(token: string): Promise<AdminSystemHealthResponse> {
  return request<AdminSystemHealthResponse>("/api/v1/admin/system/health", { token });
}

export async function getSystemServices(
  token: string,
  params: { limit?: number; offset?: number } = {}
): Promise<AdminSystemServicesResponse> {
  return request<AdminSystemServicesResponse>(
    `/api/v1/admin/system/services${buildQuery({
      limit: params.limit,
      offset: params.offset
    })}`,
    { token }
  );
}

export async function getSystemWorkflows(
  token: string,
  params: { window_hours?: number; limit?: number; offset?: number } = {}
): Promise<AdminSystemWorkflowsResponse> {
  return request<AdminSystemWorkflowsResponse>(
    `/api/v1/admin/system/workflows${buildQuery({
      window_hours: params.window_hours,
      limit: params.limit,
      offset: params.offset
    })}`,
    { token }
  );
}

export async function getSystemCache(token: string): Promise<AdminSystemCacheResponse> {
  return request<AdminSystemCacheResponse>("/api/v1/admin/system/cache", { token });
}

export async function getAuditEvents(
  token: string,
  params: {
    actor?: string;
    action?: string;
    result?: string;
    from?: string;
    to?: string;
    limit?: number;
    offset?: number;
  } = {}
): Promise<AdminAuditEventsResponse> {
  return request<AdminAuditEventsResponse>(
    `/api/v1/admin/audit-events${buildQuery({
      actor: params.actor,
      action: params.action,
      result: params.result,
      from: params.from,
      to: params.to,
      limit: params.limit,
      offset: params.offset
    })}`,
    { token }
  );
}

export async function getAuditEvent(
  token: string,
  eventId: string
): Promise<AdminAuditEventDetailResponse> {
  return request<AdminAuditEventDetailResponse>(`/api/v1/admin/audit-events/${eventId}`, { token });
}

export async function getAuditActions(token: string): Promise<AdminAuditActionsResponse> {
  return request<AdminAuditActionsResponse>("/api/v1/admin/audit-events/actions", { token });
}

// ─── Billing (Dodo Payments) ─────────────────────────────────────────────────

export async function getAdminBillingOverview(
  token: string
): Promise<AdminBillingOverviewResponse> {
  return request<AdminBillingOverviewResponse>("/api/v1/admin/billing/overview", { token });
}

export async function getAdminBillingSubscriptions(
  token: string,
  params: { status?: string; limit?: number; offset?: number } = {}
): Promise<AdminBillingSubscriptionsResponse> {
  const qs = new URLSearchParams();
  if (params.status) qs.set("status", params.status);
  if (params.limit !== undefined) qs.set("limit", String(params.limit));
  if (params.offset !== undefined) qs.set("offset", String(params.offset));
  const suffix = qs.toString() ? `?${qs.toString()}` : "";
  return request<AdminBillingSubscriptionsResponse>(
    `/api/v1/admin/billing/subscriptions${suffix}`,
    { token }
  );
}

export async function getAdminBillingSubscription(
  token: string,
  id: string
): Promise<AdminBillingSubscriptionDetailResponse> {
  return request<AdminBillingSubscriptionDetailResponse>(
    `/api/v1/admin/billing/subscriptions/${encodeURIComponent(id)}`,
    { token }
  );
}

export async function cancelAdminBillingSubscription(
  token: string,
  id: string
): Promise<AdminBillingCancelSubscriptionResponse> {
  return request<AdminBillingCancelSubscriptionResponse>(
    `/api/v1/admin/billing/subscriptions/${encodeURIComponent(id)}/cancel`,
    { token, method: "POST" }
  );
}

export async function getAdminBillingWebhookEvents(
  token: string,
  params: { provider?: string; limit?: number } = {}
): Promise<AdminBillingWebhookEventsResponse> {
  const qs = new URLSearchParams();
  if (params.provider) qs.set("provider", params.provider);
  if (params.limit !== undefined) qs.set("limit", String(params.limit));
  const suffix = qs.toString() ? `?${qs.toString()}` : "";
  return request<AdminBillingWebhookEventsResponse>(
    `/api/v1/admin/billing/webhook-events${suffix}`,
    { token }
  );
}

export async function getAdminBillingReconcileDrift(
  token: string
): Promise<AdminBillingReconcileDriftResponse> {
  return request<AdminBillingReconcileDriftResponse>(
    "/api/v1/admin/billing/reconcile/drift",
    { token }
  );
}

export async function triggerAdminBillingReconcile(
  token: string
): Promise<AdminBillingReconcileTriggerResponse> {
  return request<AdminBillingReconcileTriggerResponse>(
    "/api/v1/admin/billing/reconcile/run",
    { token, method: "POST" }
  );
}

export async function getAdminBillingPlans(
  token: string
): Promise<AdminBillingPlansResponse> {
  return request<AdminBillingPlansResponse>("/api/v1/admin/billing/plans", { token });
}

export { ApiClientError };
