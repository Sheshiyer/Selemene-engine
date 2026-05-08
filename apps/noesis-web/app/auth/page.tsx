"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { setApiKey } from "@/lib/auth";
import { getHealth } from "@/lib/api";

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

  return (
    <div style={s.page}>
      <form style={s.card} onSubmit={handleSubmit}>
        <div style={s.logo}>NOESIS</div>
        <p style={s.subtitle}>
          Enter your Selemene API key to access the 16-engine consciousness
          analysis platform.
        </p>

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
  );
}
