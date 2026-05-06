/**
 * Typed fetch helpers for the admin billing dashboard. Mirrors the shapes
 * defined in `crates/noesis-api/src/handlers/admin_billing.rs`. Direct
 * client-side fetch — no BFF proxy — matching the existing pattern in
 * `(protected)/billing/page.tsx`. Authorization comes from the stored
 * BiofieldAuthSession.
 */

import { buildApiUrl } from "@/lib/config";

// ---- Response types (mirror Rust serde shapes) ----

export interface StatusCountEntry {
  status: string;
  count: number;
}

export interface AdminBillingOverview {
  status_counts: StatusCountEntry[];
  free_users: number;
  mrr_usd_estimate: number;
}

export interface AdminSubscriptionItem {
  id: string;
  user_id: string;
  plan_id: string;
  provider: string;
  provider_customer_id: string | null;
  provider_subscription_id: string | null;
  status: string;
  cancel_at_period_end: boolean;
  current_period_start: string | null;
  current_period_end: string | null;
  canceled_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface AdminSubscriptionsResponse {
  items: AdminSubscriptionItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface AdminSubscriptionDetailResponse {
  subscription: AdminSubscriptionItem;
}

export interface AdminWebhookEventItem {
  webhook_id: string;
  provider: string;
  event_type: string;
  processed_at: string;
}

export interface AdminWebhookEventsResponse {
  items: AdminWebhookEventItem[];
  limit: number;
}

export interface AdminReconcileRun {
  id: number;
  started_at: string;
  finished_at: string | null;
  force_cancel: boolean;
  drift_json: Record<string, unknown>;
  error: string | null;
}

export interface AdminReconcileDriftResponse {
  latest: AdminReconcileRun | null;
}

export interface AdminReconcileTriggerResponse {
  command: string;
  note: string;
}

export interface AdminPlanItem {
  id: string;
  code: string;
  display_name: string;
  description: string | null;
  is_active: boolean;
  dodo_product_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface AdminPlansResponse {
  items: AdminPlanItem[];
}

// ---- Error type ----

export class AdminApiError extends Error {
  readonly status: number;
  readonly code?: string;
  constructor(status: number, message: string, code?: string) {
    super(message);
    this.name = "AdminApiError";
    this.status = status;
    this.code = code;
  }
}

// ---- Internal fetch helper ----

async function call<T>(
  path: string,
  token: string,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(buildApiUrl(path), {
    cache: "no-store",
    ...init,
    headers: {
      ...(init?.headers ?? {}),
      Authorization: `Bearer ${token}`,
    },
  });
  const text = await res.text();
  const payload = text ? JSON.parse(text) : null;
  if (!res.ok) {
    const message =
      typeof payload?.message === "string"
        ? payload.message
        : `Request failed: ${res.status}`;
    const code =
      typeof payload?.error_code === "string" ? payload.error_code : undefined;
    throw new AdminApiError(res.status, message, code);
  }
  return payload as T;
}

// ---- Public functions ----

export function getAdminBillingOverview(token: string) {
  return call<AdminBillingOverview>("/api/v1/admin/billing/overview", token);
}

export function listAdminSubscriptions(
  token: string,
  options?: { status?: string; limit?: number; offset?: number },
) {
  const qs = new URLSearchParams();
  if (options?.status) qs.set("status", options.status);
  if (options?.limit !== undefined) qs.set("limit", String(options.limit));
  if (options?.offset !== undefined) qs.set("offset", String(options.offset));
  const suffix = qs.toString() ? `?${qs.toString()}` : "";
  return call<AdminSubscriptionsResponse>(
    `/api/v1/admin/billing/subscriptions${suffix}`,
    token,
  );
}

export function getAdminSubscription(token: string, id: string) {
  return call<AdminSubscriptionDetailResponse>(
    `/api/v1/admin/billing/subscriptions/${encodeURIComponent(id)}`,
    token,
  );
}

export function cancelAdminSubscription(token: string, id: string) {
  return call<{ ok: boolean; subscription_id: string }>(
    `/api/v1/admin/billing/subscriptions/${encodeURIComponent(id)}/cancel`,
    token,
    { method: "POST" },
  );
}

export function listAdminWebhookEvents(
  token: string,
  options?: { provider?: string; limit?: number },
) {
  const qs = new URLSearchParams();
  if (options?.provider) qs.set("provider", options.provider);
  if (options?.limit !== undefined) qs.set("limit", String(options.limit));
  const suffix = qs.toString() ? `?${qs.toString()}` : "";
  return call<AdminWebhookEventsResponse>(
    `/api/v1/admin/billing/webhook-events${suffix}`,
    token,
  );
}

export function getAdminReconcileDrift(token: string) {
  return call<AdminReconcileDriftResponse>(
    "/api/v1/admin/billing/reconcile/drift",
    token,
  );
}

export function triggerAdminReconcile(token: string) {
  return call<AdminReconcileTriggerResponse>(
    "/api/v1/admin/billing/reconcile/run",
    token,
    { method: "POST" },
  );
}

export function listAdminPlans(token: string) {
  return call<AdminPlansResponse>("/api/v1/admin/billing/plans", token);
}
