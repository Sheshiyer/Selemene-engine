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

export async function login(email: string, password: string): Promise<BiofieldAuthSession> {
  const response = await fetch(buildApiUrl("/api/v1/auth/login"), {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ email, password }),
    cache: "no-store",
  });

  const text = await response.text();
  const payload = text ? JSON.parse(text) : null;

  if (!response.ok) {
    throw new BiofieldApiError(
      typeof payload?.message === "string" ? payload.message : `Login failed: ${response.status}`,
      response.status,
      payload,
    );
  }

  const loginResponse = payload as LoginResponse;
  return {
    token: loginResponse.token,
    userId: loginResponse.user_id,
    email: loginResponse.email,
    tier: loginResponse.tier,
  };
}

export function createBiofieldClient(token: string): BiofieldClient {
  return new BiofieldClient(getApiBaseUrl(), {
    authToken: token,
  });
}

export function createNoesisClient(token: string): NoesisClient {
  return new NoesisClient(getApiBaseUrl(), { authToken: token });
}
