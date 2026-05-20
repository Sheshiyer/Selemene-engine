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
    fontFamily: "var(--font-display)",
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
        // Fetch user profile and admin status in parallel
        const [me, isAdmin] = await Promise.all([
          getMe(res.token).catch(() => null),
          checkAdminAccess(res.token).catch(() => false),
        ]);
        if (!cancelled && me) {
          setUserProfile({ id: me.id, email: me.email, full_name: me.full_name, tier: me.tier, is_admin: isAdmin });
        }

        // ─── Ephemeral-flow claim handoff ──────────────────────────────
        // If the user came here from a "Save this reading" CTA on /r/[id],
        // there's a pendingClaim in localStorage. Hit the backend claim
        // endpoint to bind the anonymous reading to this user, mark the
        // local cache as claimed, and redirect back to the reading.
        const { getPendingClaim, clearPendingClaim, markReadingClaimed } =
          await import("@/lib/integrated/readingCache");
        const { claimReading } = await import("@/lib/api");
        const pending = getPendingClaim();
        let destination = "/engines";
        if (pending?.reading_id) {
          try {
            await claimReading(pending.reading_id, res.token);
            markReadingClaimed(pending.reading_id);
          } catch {
            // Backend may not have the claim endpoint yet — degrade
            // gracefully. The reading stays in localStorage and the
            // user is still signed in.
          }
          clearPendingClaim();
          // Honor the `next` query param if it points back to the reading
          const params = new URLSearchParams(window.location.search);
          const next = params.get("next");
          destination = next && next.startsWith("/r/") ? next : `/r/${pending.reading_id}`;
        } else {
          // Generic post-auth redirect — respect `next` if provided.
          // Also check for the `noesis:pending_integrated` flag set by
          // /get-reading when an anonymous user picks "Sixteen mirrors,
          // full synthesis": that path requires identity, so the user
          // was bounced through Discord OAuth and now needs to return
          // to /get-reading so the auto-replay effect can fire the
          // integrated workflow with their fresh auth token.
          const params = new URLSearchParams(window.location.search);
          const next = params.get("next");
          let pendingIntegrated: string | null = null;
          try {
            pendingIntegrated = localStorage.getItem("noesis:pending_integrated");
          } catch { /* SSR / quota */ }
          destination =
            next ||
            (pendingIntegrated === "1" ? "/get-reading" : "/engines");
        }
        router.replace(destination);
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
