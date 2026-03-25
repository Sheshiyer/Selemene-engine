"use client";

import Link from "next/link";
import { useMemo } from "react";
import { useSearchParams } from "next/navigation";
import { decodeDiscordOauthState } from "@/lib/discord-oauth";

function describeOauthError(error: string, description: string | null): string {
  if (description && description.trim() !== "") {
    return description;
  }

  switch (error) {
    case "access_denied":
      return "Discord access was denied before the admin session could be completed.";
    default:
      return "Discord returned an OAuth error before the admin session could be completed.";
  }
}

export function DiscordOauthCallbackClient() {
  const searchParams = useSearchParams();

  const code = searchParams.get("code");
  const error = searchParams.get("error");
  const errorDescription = searchParams.get("error_description");
  const state = useMemo(
    () => decodeDiscordOauthState(searchParams.get("state")),
    [searchParams]
  );

  const returnHref = useMemo(
    () => `/login?redirect=${encodeURIComponent(state.redirectTarget)}`,
    [state.redirectTarget]
  );

  let title = "Discord sign-in incomplete";
  let message =
    "Discord returned to the dashboard without an authorization result. Start the sign-in flow again.";

  if (error) {
    title = "Discord sign-in cancelled";
    message = describeOauthError(error, errorDescription);
  } else if (code) {
    title = "Discord authorization received";
    message =
      "Discord returned an authorization code to the dashboard. This thin slice stops at the callback UI; the backend code-exchange and admin session issuance still need to be wired.";
  }

  return (
    <main className="login-wrap">
      <section className="login-card oauth-callback-card">
        <div className="oauth-badge">Discord Callback</div>
        <h1>{title}</h1>
        <p>{message}</p>

        {code ? (
          <div className="muted-card">
            <strong>Next backend step</strong>
            <p>
              Exchange the returned Discord authorization code for an app session token, then route
              the operator back into the protected admin experience.
            </p>
          </div>
        ) : null}

        <div className="oauth-actions">
          <Link className="oauth-link" href={returnHref}>
            Return to admin login
          </Link>
        </div>
      </section>
    </main>
  );
}
