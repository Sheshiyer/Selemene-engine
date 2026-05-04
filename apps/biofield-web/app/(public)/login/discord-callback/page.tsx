"use client";

import { useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { setStoredAuthSession } from "@/lib/auth";
import { BiofieldApiError, discordCallback } from "@/lib/api";

function currentCallbackUri(): string | undefined {
  if (typeof window === "undefined") return undefined;
  return `${window.location.origin}${window.location.pathname}`;
}

export default function DiscordCallbackPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const code = searchParams.get("code");
  const state = searchParams.get("state");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!code) return;
    const authCode = code;
    let cancelled = false;

    async function exchange() {
      try {
        const session = await discordCallback(
          authCode,
          state ?? undefined,
          currentCallbackUri(),
        );
        if (cancelled) return;
        setStoredAuthSession(session);
        router.replace("/viewer");
      } catch (err) {
        if (cancelled) return;
        if (err instanceof BiofieldApiError) {
          setError(err.message);
        } else {
          setError("Discord sign-in could not be completed.");
        }
      }
    }

    void exchange();
    return () => { cancelled = true; };
  }, [code, router, state]);

  if (!code) {
    return (
      <main className="biofield-shell">
        <div className="biofield-stack">
          <section className="biofield-panel">
            <p className="biofield-eyebrow">Noesis · Biofield</p>
            <h1 className="biofield-title" style={{ fontSize: "2rem" }}>No code received</h1>
            <p className="biofield-copy">Discord did not return an authorisation code.</p>
            <div className="biofield-actions" style={{ marginTop: "1.5rem" }}>
              <a className="biofield-link" href="/login">Back to sign in</a>
            </div>
          </section>
        </div>
      </main>
    );
  }

  if (error) {
    return (
      <main className="biofield-shell">
        <div className="biofield-stack">
          <section className="biofield-panel">
            <p className="biofield-eyebrow">Noesis · Biofield</p>
            <h1 className="biofield-title" style={{ fontSize: "2rem" }}>Sign-in failed</h1>
            <p className="biofield-error">{error}</p>
            <div className="biofield-actions" style={{ marginTop: "1.5rem" }}>
              <a className="biofield-link" href="/login">Try again</a>
            </div>
          </section>
        </div>
      </main>
    );
  }

  return (
    <main className="biofield-shell">
      <div className="biofield-stack">
        <section className="biofield-panel">
          <p className="biofield-eyebrow">Noesis · Biofield</p>
          <p className="biofield-copy">Completing Discord sign-in…</p>
        </section>
      </div>
    </main>
  );
}
