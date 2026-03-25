const FALLBACK_API_BASE = "http://localhost:8080";
const ADMIN_BASE_PATH = "/admin";
const DISCORD_AUTHORIZE_URL = "https://discord.com/oauth2/authorize";
const DISCORD_CALLBACK_PATH = "/auth/discord/callback";
const DEFAULT_DISCORD_SCOPE = "identify email guilds";
const DEFAULT_DISCORD_PROMPT = "consent";

export function getApiBaseUrl(): string {
  return process.env.NEXT_PUBLIC_API_BASE_URL ?? FALLBACK_API_BASE;
}

export function isDevPermissionBypassEnabled(): boolean {
  return process.env.NEXT_PUBLIC_ADMIN_DEV_MODE === "true";
}

export function getAdminBasePath(): string {
  return ADMIN_BASE_PATH;
}

export function getDiscordOauthAuthorizeUrlBase(): string {
  return process.env.NEXT_PUBLIC_DISCORD_OAUTH_AUTHORIZE_URL ?? DISCORD_AUTHORIZE_URL;
}

export function getDiscordOauthCallbackPath(): string {
  return DISCORD_CALLBACK_PATH;
}

export function getDiscordOauthClientId(): string | null {
  const clientId = process.env.NEXT_PUBLIC_DISCORD_OAUTH_CLIENT_ID?.trim();
  return clientId && clientId !== "" ? clientId : null;
}

export function getDiscordOauthScope(): string {
  return process.env.NEXT_PUBLIC_DISCORD_OAUTH_SCOPE ?? DEFAULT_DISCORD_SCOPE;
}

export function getDiscordOauthPrompt(): string {
  return process.env.NEXT_PUBLIC_DISCORD_OAUTH_PROMPT ?? DEFAULT_DISCORD_PROMPT;
}

export function getDiscordOauthRedirectUri(currentOrigin?: string): string | null {
  const explicitRedirectUri = process.env.NEXT_PUBLIC_DISCORD_OAUTH_REDIRECT_URI?.trim();

  if (explicitRedirectUri && explicitRedirectUri !== "") {
    return explicitRedirectUri;
  }

  if (!currentOrigin) {
    return null;
  }

  return `${currentOrigin}${getAdminBasePath()}${getDiscordOauthCallbackPath()}`;
}
