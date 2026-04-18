const DYNAMIC_DISCORD_CALLBACK_HOSTS = new Set(["localhost", "127.0.0.1", "0.0.0.0"]);
const DYNAMIC_DISCORD_CALLBACK_SUFFIXES = [".vercel.app", ".railway.app", ".tryambakam.space"];
const STABLE_DISCORD_CALLBACK_HOSTS = new Set([
  "enantiodromia-engine-dashboard.vercel.app",
  "144.tryambakam.space",
  "selemene.tryambakam.space"
]);

function normalizeHostname(hostname: string): string {
  return hostname.trim().toLowerCase();
}

export function shouldUseDynamicDiscordCallbackOverride(hostname: string): boolean {
  const normalized = normalizeHostname(hostname);
  if (STABLE_DISCORD_CALLBACK_HOSTS.has(normalized)) {
    return false;
  }

  if (DYNAMIC_DISCORD_CALLBACK_HOSTS.has(normalized)) {
    return true;
  }

  return DYNAMIC_DISCORD_CALLBACK_SUFFIXES.some((suffix) => normalized.endsWith(suffix));
}

export function buildDiscordCallbackOverride(
  origin: string,
  hostname: string,
  callbackPath: string
): string | undefined {
  if (!shouldUseDynamicDiscordCallbackOverride(hostname)) {
    return undefined;
  }

  const normalizedPath = callbackPath !== "/" ? callbackPath.replace(/\/+$/, "") : callbackPath;
  return `${origin}${normalizedPath}`;
}

export function getDiscordLoginCallbackOverride(): string | undefined {
  if (typeof window === "undefined") {
    return undefined;
  }

  return buildDiscordCallbackOverride(
    window.location.origin,
    window.location.hostname,
    "/admin/login/discord-callback"
  );
}

export function getDiscordCurrentCallbackOverride(): string | undefined {
  if (typeof window === "undefined") {
    return undefined;
  }

  return buildDiscordCallbackOverride(
    window.location.origin,
    window.location.hostname,
    window.location.pathname
  );
}
