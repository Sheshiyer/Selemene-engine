const STABLE_HOSTS = new Set([
  "selemene.tryambakam.space",
  "144.tryambakam.space",
]);

export function getBiofieldDiscordCallbackOverride(): string | undefined {
  if (typeof window === "undefined") return undefined;

  const { hostname, origin } = window.location;
  // On stable production hosts the Rust backend uses its own configured redirect_uri.
  if (STABLE_HOSTS.has(hostname.toLowerCase())) return undefined;

  // On localhost (or any preview domain) pass the full local callback URL so
  // the Rust backend overrides its configured redirect_uri.
  return `${origin}/login/discord-callback`;
}
