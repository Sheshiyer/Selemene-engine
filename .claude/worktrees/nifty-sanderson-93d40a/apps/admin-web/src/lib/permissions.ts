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
  return null;
}
