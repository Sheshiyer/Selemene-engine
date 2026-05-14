"use client";

import { FormEvent, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { getStoredAuthSession, setStoredAuthSession } from "@/lib/auth";
import { BiofieldApiError, getDiscordAuthUrl, login, verifyToken } from "@/lib/api";
import { getBiofieldDiscordCallbackOverride } from "@/lib/discord-oauth";

const DiscordIcon = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
    <path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057c.002.022.015.043.031.056a19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03z"/>
  </svg>
);

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isDiscordLoading, setIsDiscordLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (getStoredAuthSession()) {
      router.replace("/viewer");
      return;
    }

    // Token handoff: noesis-web may pass #token=<jwt> in the URL fragment.
    // The fragment never reaches the server, but we clear it immediately after
    // consuming it so it doesn't linger in the browser history.
    const hash = window.location.hash;
    const tokenMatch = hash.match(/^#token=([A-Za-z0-9\-._~+/]+=*)$/);
    if (tokenMatch) {
      // Clear fragment before any async work so it never appears in history.
      history.replaceState(null, "", window.location.pathname + window.location.search);
      const candidateToken = tokenMatch[1];
      // Only forward JWT tokens (eyJ prefix). Skip nk_ API keys.
      if (candidateToken.startsWith("eyJ")) {
        verifyToken(candidateToken)
          .then((session) => {
            setStoredAuthSession(session);
            router.replace("/viewer");
          })
          .catch(() => {
            // Invalid/expired token — stay on login, no error shown.
          });
      }
    }
  }, [router]);

  async function handleDiscordLogin() {
    setErrorMessage(null);
    setIsDiscordLoading(true);
    try {
      const { url } = await getDiscordAuthUrl(getBiofieldDiscordCallbackOverride(), "biofield");
      window.location.href = url;
    } catch (error) {
      setIsDiscordLoading(false);
      setErrorMessage(error instanceof BiofieldApiError ? error.message : "Could not initiate Discord sign-in.");
    }
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);
    setErrorMessage(null);
    try {
      const session = await login(email, password);
      setStoredAuthSession(session);
      router.push("/viewer");
    } catch (error) {
      setErrorMessage(error instanceof BiofieldApiError ? error.message : "Unable to sign in right now.");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="biofield-login-shell">
      {/* ── Left — brand context ── */}
      <div className="biofield-login-left">
        <div>
          <p className="biofield-eyebrow" style={{ marginBottom: "1.4rem" }}>Noesis · Biofield</p>
          <h1 className="biofield-title" style={{ fontSize: "clamp(2.4rem, 5vw, 4rem)", marginBottom: "1.4rem" }}>
            Your camera<br />as field instrument.
          </h1>
          <p className="biofield-copy" style={{ fontSize: "1.05rem", lineHeight: 1.7 }}>
            Live pattern data moves through sixteen Noesis engines into the private analysis layer — returning as structured readings you can compare, baseline, and track over time.
          </p>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: "0.85rem", marginTop: "0.4rem" }}>
          {[
            ["Coherence", "Live symmetry and luminance metrics from your biofield"],
            ["Sixteen engines", "Panchanga, Human Design, Gene Keys, Numerology and more"],
            ["Private analysis", "Captures processed through the dedicated Python service"],
          ].map(([label, desc]) => (
            <div key={label} style={{ display: "flex", alignItems: "flex-start", gap: "0.85rem" }}>
              <span style={{
                flexShrink: 0,
                marginTop: "0.18rem",
                width: 6,
                height: 6,
                borderRadius: "50%",
                background: "var(--accent)",
                boxShadow: "0 0 6px rgba(var(--accent-rgb), 0.5)",
              }} />
              <div>
                <p style={{ margin: 0, fontSize: "0.82rem", fontWeight: 600, letterSpacing: "0.04em", color: "var(--text)" }}>{label}</p>
                <p style={{ margin: 0, fontSize: "0.82rem", color: "var(--muted)", lineHeight: 1.5 }}>{desc}</p>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* ── Right — form card ── */}
      <div className="biofield-login-right">
        <div className="biofield-login-form-card">
          <div className="biofield-login-wordmark">
            <span className="biofield-login-wordmark-dot" />
            <span className="biofield-login-wordmark-text">Selemene · Biofield</span>
          </div>

          <h2 style={{ margin: "0 0 0.5rem", fontSize: "1.5rem", fontWeight: 700, letterSpacing: "-0.04em" }}>
            Sign in
          </h2>
          <p className="biofield-copy" style={{ fontSize: "0.88rem", marginBottom: "1.8rem" }}>
            Use your Selemene account to open a session.
          </p>

          <button
            className="biofield-button biofield-button-primary"
            disabled={isDiscordLoading || isSubmitting}
            onClick={handleDiscordLogin}
            style={{ width: "100%", padding: "0.9rem 1.2rem", marginBottom: "0.5rem" }}
            type="button"
          >
            <DiscordIcon />
            {isDiscordLoading ? "Redirecting…" : "Continue with Discord"}
          </button>

          <div className="biofield-divider">or</div>

          <form className="biofield-form" onSubmit={handleSubmit}>
            <label className="biofield-field" htmlFor="biofield-email">
              <span className="biofield-kicker" style={{ marginBottom: "0.1rem" }}>Email</span>
              <input
                className="biofield-input"
                id="biofield-email"
                type="email"
                autoComplete="email"
                placeholder="you@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
              />
            </label>

            <label className="biofield-field" htmlFor="biofield-password">
              <span className="biofield-kicker" style={{ marginBottom: "0.1rem" }}>Password</span>
              <input
                className="biofield-input"
                id="biofield-password"
                type="password"
                autoComplete="current-password"
                placeholder="••••••••"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
              />
            </label>

            {errorMessage ? <p className="biofield-error">{errorMessage}</p> : null}

            <button
              className="biofield-button"
              disabled={isSubmitting || isDiscordLoading}
              style={{ width: "100%", padding: "0.9rem 1.2rem" }}
              type="submit"
            >
              {isSubmitting ? "Signing in…" : "Sign in with email"}
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}
