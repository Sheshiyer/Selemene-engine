"use client";

import { FormEvent, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { getStoredAuthSession, setStoredAuthSession } from "@/lib/auth";
import { BiofieldApiError, login } from "@/lib/api";

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState("demo@example.com");
  const [password, setPassword] = useState("DemoPass123");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (getStoredAuthSession()) {
      router.replace("/viewer");
    }
  }, [router]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);
    setErrorMessage(null);

    try {
      const session = await login(email, password);
      setStoredAuthSession(session);
      router.push("/viewer");
    } catch (error) {
      if (error instanceof BiofieldApiError) {
        setErrorMessage(error.message);
      } else {
        setErrorMessage("Unable to sign in right now.");
      }
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <main className="biofield-shell">
      <div className="biofield-stack">
        <section className="biofield-hero">
          <p className="biofield-eyebrow">BF1-04 authenticated shell</p>
          <h1 className="biofield-title">Biofield Web</h1>
          <p className="biofield-copy">
            Sign in with your Selemene account to start a real biofield session and send a capture through Noesis to the private Python analysis service.
          </p>
        </section>

        <section className="biofield-panel biofield-form-panel">
          <form className="biofield-form" onSubmit={handleSubmit}>
            <label className="biofield-field" htmlFor="biofield-email">
              <span className="biofield-kicker">Email</span>
              <input
                className="biofield-input"
                id="biofield-email"
                type="email"
                autoComplete="email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                required
              />
            </label>

            <label className="biofield-field" htmlFor="biofield-password">
              <span className="biofield-kicker">Password</span>
              <input
                className="biofield-input"
                id="biofield-password"
                type="password"
                autoComplete="current-password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                required
              />
            </label>

            {errorMessage ? <p className="biofield-error">{errorMessage}</p> : null}

            <div className="biofield-actions">
              <button className="biofield-button" disabled={isSubmitting} type="submit">
                {isSubmitting ? "Signing in…" : "Sign in"}
              </button>
            </div>
          </form>
        </section>
      </div>
    </main>
  );
}
