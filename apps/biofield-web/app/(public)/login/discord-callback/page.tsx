"use client";

import { useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { setStoredAuthSession } from "@/lib/auth";
import { BiofieldApiError, discordCallback } from "@/lib/api";

function currentCallbackUri(): string | undefined {
  if (typeof window === "undefined") return undefined;
  return `${window.location.origin}${window.location.pathname}`;
}

function Shell({ children }: { children: React.ReactNode }) {
  return (
    <div style={{ minHeight: "100dvh", display: "flex", alignItems: "center", justifyContent: "center", padding: "2rem" }}>
      <div style={{
        width: "100%",
        maxWidth: 400,
        border: "1px solid var(--line-mid)",
        borderRadius: "var(--r-xl)",
        padding: "2.4rem 2.2rem",
        background: "var(--panel-strong)",
        backdropFilter: "blur(24px)",
        boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06), 0 32px 72px rgba(0,0,0,0.42)",
        display: "grid",
        gap: "1rem",
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "0.5rem" }}>
          <span style={{ width: 8, height: 8, borderRadius: "50%", background: "var(--accent)", boxShadow: "0 0 8px rgba(var(--accent-rgb),0.5)" }} />
          <span style={{ fontSize: "0.72rem", fontWeight: 600, letterSpacing: "0.18em", textTransform: "uppercase", color: "var(--muted)" }}>
            Noesis · Biofield
          </span>
        </div>
        {children}
      </div>
    </div>
  );
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
        const session = await discordCallback(authCode, state ?? undefined, currentCallbackUri());
        if (cancelled) return;
        setStoredAuthSession(session);
        router.replace("/viewer");
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof BiofieldApiError ? err.message : "Discord sign-in could not be completed.");
      }
    }

    void exchange();
    return () => { cancelled = true; };
  }, [code, router, state]);

  if (!code) {
    return (
      <Shell>
        <h1 style={{ margin: 0, fontSize: "1.4rem", fontWeight: 700, letterSpacing: "-0.04em" }}>No code received</h1>
        <p className="biofield-copy" style={{ fontSize: "0.88rem" }}>Discord did not return an authorisation code.</p>
        <a className="biofield-button" href="/login" style={{ textAlign: "center", padding: "0.72rem 1.2rem" }}>Back to sign in</a>
      </Shell>
    );
  }

  if (error) {
    return (
      <Shell>
        <h1 style={{ margin: 0, fontSize: "1.4rem", fontWeight: 700, letterSpacing: "-0.04em" }}>Sign-in failed</h1>
        <p className="biofield-error">{error}</p>
        <a className="biofield-button" href="/login" style={{ textAlign: "center", padding: "0.72rem 1.2rem" }}>Try again</a>
      </Shell>
    );
  }

  return (
    <Shell>
      <h1 style={{ margin: 0, fontSize: "1.4rem", fontWeight: 700, letterSpacing: "-0.04em" }}>Completing sign-in</h1>
      <p className="biofield-copy" style={{ fontSize: "0.88rem" }}>Exchanging authorisation code with Noesis…</p>
      <div style={{ display: "flex", gap: "0.4rem", alignItems: "center" }}>
        {[0, 1, 2].map((i) => (
          <span key={i} style={{
            width: 6, height: 6, borderRadius: "50%",
            background: "var(--accent)",
            animation: "pulse-dot 1.4s ease-in-out infinite",
            animationDelay: `${i * 200}ms`,
            opacity: 0.6,
          }} />
        ))}
      </div>
    </Shell>
  );
}
