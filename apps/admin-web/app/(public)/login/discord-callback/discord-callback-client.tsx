"use client";

import { useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { setAuthToken } from "@/lib/auth";
import { ApiClientError, discordCallback, getAdminSession } from "@/lib/api";

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
        const auth = await discordCallback(authCode, state ?? undefined);
        if (cancelled) return;
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
