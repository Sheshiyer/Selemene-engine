const FALLBACK_API_BASE = "http://localhost:8080";
const API_PREFIX = "/api";
const API_V1_PREFIX = "/api/v1";

function normalizePathname(pathname: string): string {
  if (pathname === "/") {
    return pathname;
  }
  return pathname.endsWith("/") ? pathname.slice(0, -1) : pathname;
}

function stripApiSuffix(pathname: string): string {
  const normalized = normalizePathname(pathname);
  if (normalized.endsWith(API_V1_PREFIX)) {
    return normalized.slice(0, -API_V1_PREFIX.length) || "/";
  }
  if (normalized.endsWith(API_PREFIX)) {
    return normalized.slice(0, -API_PREFIX.length) || "/";
  }
  return normalized;
}

export function getApiBaseUrl(): string {
  const raw = process.env.NEXT_PUBLIC_API_BASE_URL ?? FALLBACK_API_BASE;
  const resolved = new URL(raw);
  resolved.pathname = stripApiSuffix(resolved.pathname);
  return resolved.toString().replace(/\/$/, "");
}

export function buildApiUrl(path: string): string {
  const base = new URL(getApiBaseUrl());
  const request = new URL(path, "https://placeholder.internal");
  const basePath = normalizePathname(base.pathname);
  const requestPath = request.pathname;
  const combinedPath = normalizePathname(
    `${basePath === "/" ? "" : basePath}${requestPath.startsWith("/") ? requestPath : `/${requestPath}`}`,
  );

  base.pathname = combinedPath.startsWith("/") ? combinedPath : `/${combinedPath}`;
  base.search = request.search;
  return base.toString();
}
