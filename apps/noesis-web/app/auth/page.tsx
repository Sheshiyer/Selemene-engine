"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { setApiKey } from "@/lib/auth";
import { getHealth, getDiscordAuthUrl } from "@/lib/api";
import { getDiscordCallbackUri } from "@/lib/discord-oauth";

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
    gap: "1.25rem",
  },
  logo: {
    fontFamily: "'Exo 2', sans-serif",
    fontSize: "2rem",
    fontWeight: 800,
    color: "var(--gold)",
    textAlign: "center" as const,
    letterSpacing: "0.08em",
  },
  subtitle: {
    fontSize: "0.9rem",
    color: "var(--text-muted)",
    textAlign: "center" as const,
    lineHeight: 1.5,
  },
  label: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.05em",
    fontWeight: 600,
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.375rem",
  },
  input: {
    width: "100%",
    padding: "0.625rem 0.75rem",
    fontSize: "0.95rem",
  },
  button: {
    width: "100%",
    padding: "0.75rem",
    background: "var(--gold)",
    color: "#070B1D",
    fontWeight: 700,
    fontSize: "0.9rem",
    borderRadius: "var(--radius)",
    border: "none",
    cursor: "pointer",
    fontFamily: "'Space Grotesk', sans-serif",
    transition: "opacity 0.15s",
  },
  buttonDisabled: {
    opacity: 0.5,
    cursor: "not-allowed",
  },
  discordButton: {
    width: "100%",
    padding: "0.75rem",
    background: "#5865F2",
    color: "#fff",
    fontWeight: 700,
    fontSize: "0.9rem",
    borderRadius: "var(--radius)",
    border: "none",
    cursor: "pointer",
    fontFamily: "'Space Grotesk', sans-serif",
    transition: "opacity 0.15s",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    gap: "0.5rem",
  },
  divider: {
    display: "flex",
    alignItems: "center",
    gap: "0.75rem",
    color: "var(--text-muted)",
    fontSize: "0.8rem",
  },
  dividerLine: {
    flex: 1,
    height: 1,
    background: "var(--line)",
  },
  errorBox: {
    padding: "0.75rem",
    background: "rgba(239,107,115,0.1)",
    border: "1px solid var(--danger)",
    borderRadius: "var(--radius)",
    color: "var(--danger)",
    fontSize: "0.85rem",
    textAlign: "center" as const,
  },
  successBox: {
    padding: "0.75rem",
    background: "var(--emerald-soft)",
    border: "1px solid var(--emerald)",
    borderRadius: "var(--radius)",
    color: "var(--emerald)",
    fontSize: "0.85rem",
    textAlign: "center" as const,
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
      <div style={s.card}>
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
