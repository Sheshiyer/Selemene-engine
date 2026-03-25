import { normalizeAdminRedirect } from "@/lib/admin-routing";
import {
  getDiscordOauthAuthorizeUrlBase,
  getDiscordOauthClientId,
  getDiscordOauthPrompt,
  getDiscordOauthRedirectUri,
  getDiscordOauthScope
} from "@/lib/config";

export interface DiscordOauthReadiness {
  clientId: string | null;
  ready: boolean;
  reason?: string;
  redirectUri: string | null;
}

export interface DiscordOauthState {
  redirectTarget: string;
}

export function getDiscordOauthReadiness(currentOrigin?: string): DiscordOauthReadiness {
  const clientId = getDiscordOauthClientId();
  const redirectUri = getDiscordOauthRedirectUri(currentOrigin);

  if (!clientId) {
    return {
      clientId: null,
      ready: false,
      reason: "Discord OAuth client ID is not configured.",
      redirectUri
    };
  }

  if (!redirectUri) {
    return {
      clientId,
      ready: false,
      reason: "Discord OAuth redirect URI is unavailable in this environment.",
      redirectUri: null
    };
  }

  return {
    clientId,
    ready: true,
    redirectUri
  };
}

export function encodeDiscordOauthState(redirectTarget: string): string {
  const params = new URLSearchParams();
  params.set("redirect", normalizeAdminRedirect(redirectTarget));
  return params.toString();
}

export function decodeDiscordOauthState(rawState: string | null): DiscordOauthState {
  const params = new URLSearchParams(rawState ?? "");
  return {
    redirectTarget: normalizeAdminRedirect(params.get("redirect"))
  };
}

export function buildDiscordOauthAuthorizeUrl(
  currentOrigin: string,
  redirectTarget: string
): string | null {
  const readiness = getDiscordOauthReadiness(currentOrigin);

  if (!readiness.ready || !readiness.clientId || !readiness.redirectUri) {
    return null;
  }

  const params = new URLSearchParams({
    client_id: readiness.clientId,
    prompt: getDiscordOauthPrompt(),
    redirect_uri: readiness.redirectUri,
    response_type: "code",
    scope: getDiscordOauthScope(),
    state: encodeDiscordOauthState(redirectTarget)
  });

  return `${getDiscordOauthAuthorizeUrlBase()}?${params.toString()}`;
}
