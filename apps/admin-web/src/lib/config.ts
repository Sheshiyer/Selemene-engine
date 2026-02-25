const FALLBACK_API_BASE = "http://localhost:8080";

export function getApiBaseUrl(): string {
  return process.env.NEXT_PUBLIC_API_BASE_URL ?? FALLBACK_API_BASE;
}

export function isDevPermissionBypassEnabled(): boolean {
  return process.env.NEXT_PUBLIC_ADMIN_DEV_MODE === "true";
}
