import { BiofieldClient } from "@selemene/biofield-api-client";
import { NoesisClient } from "@selemene/noesis-sdk-ts";
import { buildApiUrl, getApiBaseUrl } from "@/lib/config";
import type { BiofieldAuthSession } from "@/lib/auth";

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