import { Suspense } from "react";
import { DiscordCallbackClient } from "./discord-callback-client";

export default function DiscordCallbackPage() {
  return (
    <Suspense
      fallback={
        <main className="login-wrap">
          <section className="login-card">
            <h1>Selemene Admin</h1>
            <p>Completing Discord login...</p>
          </section>
        </main>
      }
    >
      <DiscordCallbackClient />
    </Suspense>
  );
}
