import { isDevPermissionBypassEnabled } from "@/lib/config";

function hasLegacyAlias(permissions: string[], required: string): boolean {
  if (permissions.includes("admin:*")) {
    return true;
  }

  if (required.startsWith("admin:users:") && permissions.includes("admin:users")) {
    return true;
  }

  if (
    (required.startsWith("admin:keys:") || required.startsWith("admin:history-sync:")) &&
    permissions.includes("admin:users")
  ) {
    return true;
  }

  if (
    (required.startsWith("admin:analytics:") ||
      required.startsWith("admin:system:") ||
      required.startsWith("admin:audit:")) &&
    permissions.includes("admin:analytics")
  ) {
    return true;
  }

  // Holding any explicit admin:billing:* perm grants the broader
  // admin:billing:read read-only check. Cancel/trigger remain explicit.
  if (
    required === "admin:billing:read" &&
    permissions.some((p) => p.startsWith("admin:billing:"))
  ) {
    return true;
  }

  return false;
}

export function hasPermission(permissions: string[], required: string): boolean {
  if (permissions.includes(required) || hasLegacyAlias(permissions, required)) {
    return true;
  }

  if (isDevPermissionBypassEnabled() && permissions.includes("basic:access")) {
    return true;
  }

  return false;
}

export function requiredPermissionForPath(pathname: string): string | null {
  const normalized = pathname.startsWith("/admin")
    ? pathname.slice("/admin".length) || "/"
    : pathname;

  if (normalized === "/" || normalized === "/dashboard") {
    return "admin:analytics:read";
  }
  if (normalized.startsWith("/users")) {
    return "admin:users:list";
  }
  if (normalized.startsWith("/api-keys")) {
    return "admin:keys:list";
  }
  if (normalized.startsWith("/history-sync")) {
    return "admin:history-sync:read";
  }
  if (normalized.startsWith("/analytics")) {
    return "admin:analytics:read";
  }
  if (normalized.startsWith("/system")) {
    return "admin:system:read";
  }
  if (normalized.startsWith("/audit")) {
    return "admin:audit:list";
  }
  if (normalized.startsWith("/billing")) {
    return "admin:billing:read";
  }
  if (normalized.startsWith("/bridge")) return "admin:system:read";
  if (normalized.startsWith("/observability")) return "admin:system:read";
  if (normalized.startsWith("/skills")) return "admin:system:read";
  if (normalized.startsWith("/witness-dyad")) return "admin:analytics:read";
  if (normalized.startsWith("/living-readings")) return "admin:analytics:read";
  if (normalized.startsWith("/readings")) return "admin:analytics:read";
  if (normalized.startsWith("/engines")) return "admin:system:read";
  if (normalized.startsWith("/biofield")) return "admin:system:read";
  if (normalized.startsWith("/workflows")) return "admin:system:read";
  if (normalized.startsWith("/sidecars")) return "admin:system:read";
  return null;
}
