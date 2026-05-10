"use client";

import { useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { setApiKey, setUserProfile } from "@/lib/auth";
import { discordCallback, getMe, checkAdminAccess } from "@/lib/api";

const s = {
  page: {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    minHeight: "100vh",
    background: "var(--bg)",
    padding: "1.5rem",
  },
  card: {
    background: "var(--bg-panel)",
    border: "1px solid var(--line)",
    borderRadius: "var(--radius)",
    padding: "2rem",
    maxWidth: 440,
    width: "100%",
    display: "flex",
    flexDirection: "column" as const,
    gap: "1rem",
    textAlign: "center" as const,
  },
  logo: {
    fontFamily: "'Exo 2', sans-serif",
    fontSize: "2rem",
    fontWeight: 800,
    color: "var(--gold)",
    letterSpacing: "0.08em",
  },
  message: {
    fontSize: "0.95rem",
    color: "var(--text-muted)",
    lineHeight: 1.5,
  },
  errorBox: {
    padding: "0.75rem",
    background: "rgba(239,107,115,0.1)",
    border: "1px solid var(--danger)",
    borderRadius: "var(--radius)",
    color: "var(--danger)",
    fontSize: "0.85rem",
  },
  link: {
    color: "var(--gold)",
    textDecoration: "none",
    fontSize: "0.85rem",
  },
};

export function DiscordCallbackClient() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const code = searchParams.get("code");
  const state = searchParams.get("state");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!code) return;

    const callbackUri = `${window.location.origin}${window.location.pathname}`;
    let cancelled = false;

    async function exchangeCode() {
      try {
        const res = await discordCallback(code!, state ?? undefined, callbackUri);
        if (cancelled) return;
        // Store JWT as the session token; authHeaders() sends it as Bearer
        setApiKey(res.token);
        // Fetch user profile and admin status in parallel, then redirect
        const [me, isAdmin] = await Promise.all([
          getMe(res.token).catch(() => null),
          checkAdminAccess(res.token).catch(() => false),
        ]);
        if (!cancelled && me) {
          setUserProfile({ id: me.id, email: me.email, full_name: me.full_name, tier: me.tier, is_admin: isAdmin });
        }
        router.replace("/engines");
      } catch (err: unknown) {
        if (cancelled) return;
        const msg =
          err instanceof Error ? err.message : "Unexpected error during Discord login.";
        setError(msg);
      }
    }

    exchangeCode();
    return () => {
      cancelled = true;
    };
  }, [code, router, state]);

  if (!code) {
    return (
      <div style={s.page}>
        <div style={s.card}>
          <div style={s.logo}>NOESIS</div>
          <div style={s.errorBox}>No authorization code received from Discord.</div>
          <a href="/auth" style={s.link}>← Back to sign in</a>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div style={s.page}>
        <div style={s.card}>
          <div style={s.logo}>NOESIS</div>
          <div style={s.errorBox}>{error}</div>
          <a href="/auth" style={s.link}>← Back to sign in</a>
        </div>
      </div>
    );
  }

  return (
    <div style={s.page}>
      <div style={s.card}>
        <div style={s.logo}>NOESIS</div>
        <p style={s.message}>Completing Discord sign in…</p>
      </div>
    </div>
  );
}
