export function normalizeAdminRedirect(rawTarget: string | null): string {
  if (!rawTarget || rawTarget.trim() === "") {
    return "/dashboard";
  }

  if (!rawTarget.startsWith("/")) {
    return "/dashboard";
  }

  if (rawTarget.startsWith("/admin")) {
    const stripped = rawTarget.slice("/admin".length);
    return stripped === "" ? "/dashboard" : stripped;
  }

  return rawTarget;
}
