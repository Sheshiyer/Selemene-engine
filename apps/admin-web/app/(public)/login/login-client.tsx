"use client";

import { FormEvent, useMemo, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { clearAuthToken, setAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminSession, getDiscordAuthUrl, login } from "@/lib/api";

function normalizeRedirect(rawTarget: string | null): string {
  if (!rawTarget || rawTarget.trim() === "") {
    return "/dashboard";
  }
  if (!rawTarget.startsWith("/")) {
    return "/dashboard";
  }
  if (rawTarget.startsWith("/admin")) {
    const stripped = rawTarget.slice("/admin".length);
    return stripped === "" ? "/dashboard" : stripped;
  }
  return rawTarget;
}

export function LoginClient() {
  const router = useRouter();
  const searchParams = useSearchParams();

  const redirectTarget = useMemo(
    () => normalizeRedirect(searchParams.get("redirect")),
    [searchParams]
  );

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isDiscordLoading, setIsDiscordLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setIsSubmitting(true);

    try {
      const auth = await login(email.trim(), password);
      setAuthToken(auth.token);
      await getAdminSession(auth.token);
      router.replace(redirectTarget);
    } catch (err) {
      clearAuthToken();
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Unexpected error during login.");
      }
    } finally {
      setIsSubmitting(false);
    }
  }

  async function onDiscordLogin() {
    setError(null);
    setIsDiscordLoading(true);

    try {
      const { url } = await getDiscordAuthUrl();
      window.location.href = url;
    } catch (err) {
      setIsDiscordLoading(false);
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to initiate Discord login.");
      }
    }
  }

  return (
    <main className="login-wrap">
      <section className="login-card">
        <h1>Selemene Admin</h1>
        <p>Authenticate with your API credentials to access admin operations.</p>
        {error ? <div className="error">{error}</div> : null}
        <form className="login-form" onSubmit={onSubmit}>
          <label htmlFor="email">
            Email
            <input
              id="email"
              type="email"
              autoComplete="email"
              required
              value={email}
              onChange={(event) => setEmail(event.target.value)}
            />
          </label>
          <label htmlFor="password">
            Password
            <input
              id="password"
              type="password"
              autoComplete="current-password"
              required
              value={password}
              onChange={(event) => setPassword(event.target.value)}
            />
          </label>
          <button type="submit" disabled={isSubmitting}>
            {isSubmitting ? "Signing in..." : "Sign in"}
          </button>
        </form>
        <div className="login-divider">
          <span>or</span>
        </div>
        <button
          type="button"
          className="discord-btn"
          onClick={onDiscordLogin}
          disabled={isDiscordLoading}
        >
          {isDiscordLoading ? "Redirecting..." : "Sign in with Discord"}
        </button>
      </section>
    </main>
  );
}
