"use client";

import { useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { setAuthToken } from "@/lib/auth";
import { ApiClientError, discordCallback, getAdminSession } from "@/lib/api";

const BIOFIELD_ORIGIN = "https://biofield.tryambakam.space";

function currentCallbackUri(): string | null {
  if (typeof window === "undefined") {
    return null;
  }

  return `${window.location.origin}${window.location.pathname}`;
}

/** Extract the optional client identifier from the OAuth state string.
 *  Format: "{timestamp}" or "{timestamp}:{client}".
 */
function parseClientFromState(state: string | null | undefined): string | undefined {
  if (!state) return undefined;
  const colonIdx = state.indexOf(":");
  if (colonIdx === -1) return undefined;
  const client = state.slice(colonIdx + 1).trim();
  return client || undefined;
}

export function DiscordCallbackClient() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const code = searchParams.get("code");
  const state = searchParams.get("state");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!code) {
      return;
    }
    const authCode = code;

    let cancelled = false;

    async function exchangeCode() {
      try {
        const auth = await discordCallback(
          authCode,
          state ?? undefined,
          currentCallbackUri() ?? undefined
        );
        if (cancelled) return;

        const client = parseClientFromState(state);
        if (client === "biofield") {
          // Hand the token back to biofield-web via the fragment handoff pattern.
          // The fragment never reaches the server and is immediately cleared by
          // biofield's login page after consumption.
          window.location.replace(`${BIOFIELD_ORIGIN}/login#token=${auth.token}`);
          return;
        }

        setAuthToken(auth.token);
        await getAdminSession(auth.token);
        router.replace("/dashboard");
      } catch (err) {
        if (cancelled) return;
        if (err instanceof ApiClientError) {
          setError(err.payload?.error || err.message);
        } else {
          setError("Unexpected error during Discord login.");
        }
      }
    }

    exchangeCode();
    return () => {
      cancelled = true;
    };
  }, [code, router, state]);

  if (!code) {
    return (
      <main className="login-wrap">
        <section className="login-card">
          <h1>Selemene Admin</h1>
          <div className="error">No authorization code received from Discord.</div>
          <p style={{ marginTop: "1rem" }}>
            <a href="/admin/login" style={{ color: "var(--accent)" }}>
              Back to login
            </a>
          </p>
        </section>
      </main>
    );
  }

  if (error) {
    return (
      <main className="login-wrap">
        <section className="login-card">
          <h1>Selemene Admin</h1>
          <div className="error">{error}</div>
          <p style={{ marginTop: "1rem" }}>
            <a href="/admin/login" style={{ color: "var(--accent)" }}>
              Back to login
            </a>
          </p>
        </section>
      </main>
    );
  }

  return (
    <main className="login-wrap">
      <section className="login-card">
        <h1>Selemene Admin</h1>
        <p>Completing Discord login...</p>
      </section>
    </main>
  );
}
