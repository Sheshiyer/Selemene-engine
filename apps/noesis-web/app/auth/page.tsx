"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { setApiKey, setUserProfile } from "@/lib/auth";
import { getHealth, getDiscordAuthUrl, getMe, checkAdminAccess } from "@/lib/api";
import { getDiscordCallbackUri } from "@/lib/discord-oauth";
import { DyadChamber } from "@selemene/dyad-ui";

const s = {
  page: {
    position: "relative" as const,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    minHeight: "100vh",
    // Full-viewport Kha Arc — the void receiving consciousness
    background: "linear-gradient(135deg, #070B1D 0%, #2D0050 55%, #0B50FB11 100%)",
    padding: "1.5rem",
    overflow: "hidden",
  },
  card: {
    background: "rgba(7,11,29,0.72)",
    border: "1px solid var(--line-mid)",
    borderRadius: "var(--r-md)",
    padding: "2.5rem 2rem",
    maxWidth: 420,
    width: "100%",
    display: "flex",
    flexDirection: "column" as const,
    gap: "1.375rem",
    backdropFilter: "blur(18px)",
    boxShadow: "var(--shadow-lg)",
  },
  sigilRing: {
    width: 56,
    height: 56,
    margin: "0 auto",
    borderRadius: "50%",
    border: "1.5px solid rgba(197,160,23,0.45)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    boxShadow: "0 0 18px rgba(197,160,23,0.12)",
  },
  logo: {
    fontFamily: "var(--font-display)",
    fontSize: "2rem",
    fontWeight: 800,
    color: "var(--signal)",
    textAlign: "center" as const,
    letterSpacing: "0.12em",
  },
  subtitle: {
    fontSize: "0.88rem",
    color: "var(--muted)",
    textAlign: "center" as const,
    lineHeight: 1.6,
    fontFamily: "var(--font-body)",
  },
  label: {
    fontSize: "0.72rem",
    color: "var(--muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    fontWeight: 600,
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.375rem",
    fontFamily: "var(--font-body)",
  },
  input: {
    width: "100%",
    padding: "0.625rem 0.75rem",
    fontSize: "0.95rem",
  },
  button: {
    width: "100%",
    padding: "0.75rem",
    // Ba Arc gradient — organic not corporate
    background: "linear-gradient(90deg, var(--c-emerald) 0%, var(--signal) 100%)",
    color: "#070B1D",
    fontWeight: 700,
    fontSize: "0.9rem",
    borderRadius: "var(--r-sm)",
    border: "none",
    cursor: "pointer",
    fontFamily: "var(--font-display)",
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    transition: "opacity 0.15s",
  },
  buttonDisabled: {
    opacity: 0.45,
    cursor: "not-allowed",
  },
  // Ba Arc tinted Discord button — Noesis aesthetic, not corporate #5865F2
  discordButton: {
    width: "100%",
    padding: "0.75rem",
    background: "linear-gradient(90deg, rgba(11,80,251,0.25) 0%, rgba(45,0,80,0.4) 100%)",
    color: "var(--text)",
    fontWeight: 600,
    fontSize: "0.9rem",
    borderRadius: "var(--r-sm)",
    border: "1px solid rgba(11,80,251,0.4)",
    cursor: "pointer",
    fontFamily: "var(--font-body)",
    transition: "opacity 0.15s, border-color 0.15s",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    gap: "0.5rem",
  },
  divider: {
    display: "flex",
    alignItems: "center",
    gap: "0.75rem",
    color: "var(--muted)",
    fontSize: "0.78rem",
    fontFamily: "var(--font-mono)",
    letterSpacing: "0.06em",
  },
  dividerLine: {
    flex: 1,
    height: 1,
    background: "var(--line-mid)",
  },
  errorBox: {
    padding: "0.75rem",
    background: "rgba(198,93,59,0.1)",
    border: "1px solid var(--error)",
    borderRadius: "var(--r-sm)",
    color: "var(--error)",
    fontSize: "0.85rem",
    textAlign: "center" as const,
    fontFamily: "var(--font-body)",
  },
  successBox: {
    padding: "0.75rem",
    background: "rgba(16,181,167,0.08)",
    border: "1px solid rgba(16,181,167,0.35)",
    borderRadius: "var(--r-sm)",
    color: "var(--c-emerald)",
    fontSize: "0.85rem",
    textAlign: "center" as const,
    fontFamily: "var(--font-body)",
  },
};

export default function AuthPage() {
  const router = useRouter();
  const [key, setKey] = useState("");
  const [checking, setChecking] = useState(false);
  const [discordLoading, setDiscordLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = key.trim();
    if (!trimmed) return;

    setChecking(true);
    setError(null);
    setSuccess(false);

    try {
      await getHealth();
      setApiKey(trimmed);
      // Background profile fetch — don't block redirect on this
      Promise.all([getMe(trimmed).catch(() => null), checkAdminAccess(trimmed).catch(() => false)])
        .then(([me, isAdmin]) => { if (me) setUserProfile({ id: me.id, email: me.email, full_name: me.full_name, tier: me.tier, is_admin: isAdmin as boolean }); })
        .catch(() => {});
      setSuccess(true);
      setTimeout(() => router.replace("/engines"), 800);
    } catch {
      setApiKey(trimmed);
      setSuccess(true);
      setTimeout(() => router.replace("/engines"), 800);
    }
  };

  const handleDiscordLogin = async () => {
    setDiscordLoading(true);
    setError(null);
    try {
      const callbackUri = getDiscordCallbackUri();
      const { url } = await getDiscordAuthUrl(callbackUri ?? undefined);
      window.location.href = url;
    } catch {
      setError("Failed to initiate Discord login. Please try again.");
      setDiscordLoading(false);
    }
  };

  return (
    <div style={s.page}>
      {/* Canonical DyadChamber chrome — the sign-in moment is dyadic:
          both witnesses present, joined field. The orbital ring rotates
          slowly behind the card. While Discord OAuth or API-key
          submission is in flight, both witnesses pulse together. */}
      <DyadChamber
        speaker="both"
        submitting={discordLoading || checking}
        variant="full"
      />

      <div style={{ ...s.card, position: "relative", zIndex: 1 }}>
        {/* Sacred Gold sigil ring above the wordmark */}
        <div style={s.sigilRing} aria-hidden="true">
          <svg width="28" height="28" viewBox="0 0 28 28" fill="none">
            <circle cx="14" cy="14" r="10" stroke="rgba(197,160,23,0.6)" strokeWidth="1" />
            <circle cx="14" cy="14" r="4" stroke="rgba(197,160,23,0.9)" strokeWidth="1.5" />
            <line x1="14" y1="4" x2="14" y2="24" stroke="rgba(197,160,23,0.3)" strokeWidth="0.75" />
            <line x1="4" y1="14" x2="24" y2="14" stroke="rgba(197,160,23,0.3)" strokeWidth="0.75" />
          </svg>
        </div>

        <div style={s.logo}>NOESIS</div>
        <p style={s.subtitle}>
          Sign in with Discord or enter your Selemene API key to access the
          16-engine consciousness analysis platform.
        </p>

        <button
          type="button"
          onClick={handleDiscordLogin}
          disabled={discordLoading}
          style={{
            ...s.discordButton,
            ...(discordLoading ? s.buttonDisabled : {}),
          }}
        >
          <svg width="18" height="14" viewBox="0 0 71 55" fill="none">
            <path
              d="M60.1 4.9A58.5 58.5 0 0 0 45.6.7a40.3 40.3 0 0 0-1.8 3.6 54 54 0 0 0-16.4 0A40.5 40.5 0 0 0 25.6.7 58.4 58.4 0 0 0 11 4.9C1.6 18.9-1 32.6.3 46.1a58.7 58.7 0 0 0 17.9 9 44 44 0 0 0 3.8-6.2 38.3 38.3 0 0 1-6-2.9c.5-.4 1-.7 1.5-1.1a41.8 41.8 0 0 0 35.7 0l1.5 1.1a38.2 38.2 0 0 1-6 2.9 44.5 44.5 0 0 0 3.8 6.2 58.5 58.5 0 0 0 17.9-9C72 30.4 68.1 16.8 60.1 4.9ZM23.7 38c-3.5 0-6.4-3.2-6.4-7.2s2.8-7.2 6.4-7.2c3.5 0 6.4 3.2 6.3 7.2 0 4-2.8 7.2-6.3 7.2Zm23.6 0c-3.5 0-6.4-3.2-6.4-7.2s2.8-7.2 6.4-7.2c3.5 0 6.4 3.2 6.3 7.2 0 4-2.8 7.2-6.3 7.2Z"
              fill="currentColor"
            />
          </svg>
          {discordLoading ? "Redirecting…" : "Sign in with Discord"}
        </button>

        <div style={s.divider}>
          <div style={s.dividerLine} />
          <span>or</span>
          <div style={s.dividerLine} />
        </div>

        <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
          <label style={s.label}>
            API Key
            <input
              type="password"
              value={key}
              onChange={(e) => setKey(e.target.value)}
              placeholder="nk_..."
              style={s.input}
              autoFocus
            />
          </label>

          {error && <div style={s.errorBox}>{error}</div>}
          {success && (
            <div style={s.successBox}>
              Key saved — redirecting to engines…
            </div>
          )}

          <button
            type="submit"
            disabled={checking || !key.trim()}
            style={{
              ...s.button,
              ...((checking || !key.trim()) ? s.buttonDisabled : {}),
            }}
          >
            {checking ? "Verifying…" : "Save & Continue"}
          </button>
        </form>
      </div>
    </div>
  );
}
