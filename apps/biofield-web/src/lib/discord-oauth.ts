// Hosts where DISCORD_REDIRECT_URI on Railway is already configured to match
// the app's callback URL. These hosts do NOT pass an override redirect_uri.
// Only admin-web (144.tryambakam.space) belongs here because DISCORD_REDIRECT_URI
// on Railway = https://144.tryambakam.space/admin/login/discord-callback.
// biofield.tryambakam.space uses a different callback path so it must pass its own.
const STABLE_HOSTS = new Set([
  "selemene.tryambakam.space",
  "144.tryambakam.space",
]);

export function getBiofieldDiscordCallbackOverride(): string | undefined {
  if (typeof window === "undefined") return undefined;

  const { hostname, origin } = window.location;
  // On the admin host the Rust backend's configured redirect_uri already points
  // to the correct callback — no override needed.
  if (STABLE_HOSTS.has(hostname.toLowerCase())) return undefined;

  // For biofield.tryambakam.space, localhost, or any preview domain:
  // pass the full callback URL so the Rust backend overrides its configured
  // redirect_uri with the caller's own origin.
  return `${origin}/login/discord-callback`;
}
