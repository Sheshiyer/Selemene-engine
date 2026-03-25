"use client";

import { useState } from "react";
import { getDiscordOauthClientId } from "@/lib/config";
import { buildDiscordOauthAuthorizeUrl, getDiscordOauthReadiness } from "@/lib/discord-oauth";

interface DiscordAuthPanelProps {
  redirectTarget: string;
}

interface DiscordButtonState {
  enabled: boolean;
  helper: string;
}

export function DiscordAuthPanel({ redirectTarget }: DiscordAuthPanelProps) {
  const hasClientId = getDiscordOauthClientId() !== null;
  const [buttonState, setButtonState] = useState<DiscordButtonState>({
    enabled: hasClientId,
    helper: hasClientId
      ? "Recommended when admin access is managed through Discord identity."
      : "Discord OAuth client ID is not configured."
  });

  function onContinue() {
    const authorizeUrl = buildDiscordOauthAuthorizeUrl(window.location.origin, redirectTarget);

    if (!authorizeUrl) {
      const readiness = getDiscordOauthReadiness(window.location.origin);
      setButtonState({
        enabled: false,
        helper: readiness.reason ?? "Discord OAuth is not available."
      });
      return;
    }

    window.location.assign(authorizeUrl);
  }

  return (
    <section className="oauth-card" aria-label="Discord OAuth sign in">
      <div className="oauth-badge">Discord OAuth</div>
      <h2>Continue with Discord</h2>
      <p className="helper">
        Route admin sign-in through Discord first, then return to Selemene to finish the session
        handshake.
      </p>
      <button
        type="button"
        className="oauth-button"
        disabled={!buttonState.enabled}
        onClick={onContinue}
      >
        Continue with Discord
      </button>
      <p className="oauth-helper">{buttonState.helper}</p>
    </section>
  );
}
