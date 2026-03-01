import type { ReadonlyURLSearchParams } from "next/navigation";

export function getStringParam(
  params: ReadonlyURLSearchParams,
  key: string,
  fallback = ""
): string {
  return params.get(key) ?? fallback;
}

export function getNumberParam(
  params: ReadonlyURLSearchParams,
  key: string,
  fallback: number,
  min: number,
  max: number
): number {
  const raw = params.get(key);
  if (!raw) {
    return fallback;
  }

  const parsed = Number.parseInt(raw, 10);
  if (Number.isNaN(parsed)) {
    return fallback;
  }

  return Math.min(max, Math.max(min, parsed));
}

export function buildQueryString(
  current: ReadonlyURLSearchParams,
  updates: Record<string, string | number | boolean | undefined | null>
): string {
  const next = new URLSearchParams(current.toString());

  for (const [key, value] of Object.entries(updates)) {
    if (value === undefined || value === null || value === "") {
      next.delete(key);
      continue;
    }
    next.set(key, String(value));
  }

  return next.toString();
}
