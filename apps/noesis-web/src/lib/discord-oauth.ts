// Noesis-web always passes its own callback URI to the backend because the
// backend default DISCORD_REDIRECT_URI points to admin-web, not noesis-web.
const CALLBACK_PATH = "/auth/discord/callback";

export function getDiscordCallbackUri(): string | undefined {
  if (typeof window === "undefined") return undefined;
  return `${window.location.origin}${CALLBACK_PATH}`;
}
