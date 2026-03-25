"use client";

import { FormEvent, useMemo, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { DiscordAuthPanel } from "@/components/discord-auth-panel";
import { normalizeAdminRedirect } from "@/lib/admin-routing";
import { clearAuthToken, setAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminSession, login } from "@/lib/api";

export function LoginClient() {
  const router = useRouter();
  const searchParams = useSearchParams();

  const redirectTarget = useMemo(
    () => normalizeAdminRedirect(searchParams.get("redirect")),
    [searchParams]
  );

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
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

  return (
    <main className="login-wrap">
      <section className="login-card">
        <h1>Selemene Admin</h1>
        <p>Choose the admin authentication path that matches the current environment.</p>
        <div className="login-stack">
          <DiscordAuthPanel redirectTarget={redirectTarget} />
          <div className="oauth-divider" role="separator" aria-label="Fallback credential login">
            <span>or continue with credentials</span>
          </div>
        </div>
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
        <p className="oauth-helper">
          Email/password remains available while Discord token exchange is still being finalized.
        </p>
      </section>
    </main>
  );
}
