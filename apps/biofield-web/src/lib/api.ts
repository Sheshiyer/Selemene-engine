import { BiofieldClient } from "@selemene/biofield-api-client";
import { NoesisClient } from "@selemene/noesis-sdk-ts";
import { buildApiUrl, getApiBaseUrl } from "@/lib/config";
import type { BiofieldAuthSession } from "@/lib/auth";
import { emitQuotaExceeded } from "@/lib/quota";
import type { QuotaExceededDetail } from "@/components/QuotaExceededModal";

export class BiofieldApiError extends Error {
  readonly status: number;
  readonly payload?: unknown;

  constructor(message: string, status: number, payload?: unknown) {
    super(message);
    this.name = "BiofieldApiError";
    this.status = status;
    this.payload = payload;
  }
}

interface LoginResponse {
  token: string;
  user_id: string;
  email: string;
  tier: string;
}

async function apiRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(buildApiUrl(path), {
    cache: "no-store",
    ...init,
  });
  const text = await response.text();
  const payload = text ? JSON.parse(text) : null;
  if (!response.ok) {
    // Surface QUOTA_EXCEEDED to the global modal before throwing. The error
    // still propagates so callers can decide whether to fall through.
    if (
      response.status === 402 &&
      payload?.error_code === "QUOTA_EXCEEDED"
    ) {
      emitQuotaExceeded(payload?.details as QuotaExceededDetail | undefined);
    }
    throw new BiofieldApiError(
      typeof payload?.message === "string" ? payload.message : `Request failed: ${response.status}`,
      response.status,
      payload,
    );
  }
  return payload as T;
}

export async function login(email: string, password: string): Promise<BiofieldAuthSession> {
  const data = await apiRequest<LoginResponse>("/api/v1/auth/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });
  return { token: data.token, userId: data.user_id, email: data.email, tier: data.tier };
}

interface UserMeResponse {
  id: string;
  email: string;
  full_name: string | null;
  tier: string;
}

/**
 * Verify a JWT issued by the Noesis backend and return a BiofieldAuthSession.
 * Used by the token handoff flow where noesis-web passes a JWT via URL fragment.
 */
export async function verifyToken(token: string): Promise<BiofieldAuthSession> {
  const url = buildApiUrl("/api/v1/users/me");
  const response = await fetch(url, {
    headers: { Authorization: `Bearer ${token}` },
    cache: "no-store",
  });
  if (!response.ok) {
    throw new BiofieldApiError("Token verification failed", response.status);
  }
  const data = (await response.json()) as UserMeResponse;
  return { token, userId: data.id, email: data.email, tier: data.tier };
}

export async function getDiscordAuthUrl(redirectUri?: string): Promise<{ url: string }> {
  const qs = redirectUri ? `?redirect_uri=${encodeURIComponent(redirectUri)}` : "";
  return apiRequest<{ url: string }>(`/api/v1/auth/discord/authorize${qs}`);
}

export async function discordCallback(
  code: string,
  state?: string,
  redirectUri?: string,
): Promise<BiofieldAuthSession> {
  const data = await apiRequest<LoginResponse>("/api/v1/auth/discord/callback", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ code, state, redirect_uri: redirectUri }),
  });
  return { token: data.token, userId: data.user_id, email: data.email, tier: data.tier };
}

export function createBiofieldClient(token: string): BiofieldClient {
  return new BiofieldClient(getApiBaseUrl(), { authToken: token });
}

export function createNoesisClient(token: string): NoesisClient {
  return new NoesisClient(getApiBaseUrl(), { authToken: token });
}

import type {
  PlanCode,
  CheckoutCreateResponse,
  PortalCreateResponse,
  BalanceResponse,
} from "@selemene/noesis-sdk-ts";

/**
 * Creates a Dodo Payments checkout session via the Rust API.
 * Returns the hosted checkout URL the client should redirect to.
 */
export async function createCheckoutSession(
  token: string,
  planCode: PlanCode,
): Promise<CheckoutCreateResponse> {
  return apiRequest<CheckoutCreateResponse>("/api/v1/billing/checkout", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ plan_code: planCode }),
  });
}

/**
 * Creates a Dodo Payments customer-portal session for the authenticated
 * user. Returns the hosted portal URL where they manage their subscription,
 * update payment method, and download invoices.
 *
 * Returns 404 if the user has never checked out (no Dodo customer yet).
 */
export async function createPortalSession(
  token: string,
): Promise<PortalCreateResponse> {
  return apiRequest<PortalCreateResponse>("/api/v1/billing/portal", {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
  });
}

/**
 * Reads the authenticated user's billing balance: tier, period_end,
 * Witness Credits remaining. Returns tier_default for free users.
 */
export async function getBillingBalance(
  token: string,
): Promise<BalanceResponse> {
  return apiRequest<BalanceResponse>("/api/v1/billing/balance", {
    method: "GET",
    headers: { Authorization: `Bearer ${token}` },
  });
}