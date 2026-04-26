import { Suspense } from "react";
import { LoginClient } from "./login-client";

export default function LoginPage() {
  return (
    <Suspense
      fallback={
        <main className="login-wrap">
          <section className="login-card">
            <h1>Selemene Admin</h1>
            <p>Loading login form...</p>
          </section>
        </main>
      }
    >
      <LoginClient />
    </Suspense>
  );
}
