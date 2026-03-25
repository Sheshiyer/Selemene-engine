import { Suspense } from "react";
import { DiscordOauthCallbackClient } from "./callback-client";

export default function DiscordOauthCallbackPage() {
  return (
    <Suspense
      fallback={
        <main className="login-wrap">
          <section className="login-card">
            <h1>Discord sign-in</h1>
            <p>Loading Discord callback state...</p>
          </section>
        </main>
      }
    >
      <DiscordOauthCallbackClient />
    </Suspense>
  );
}
