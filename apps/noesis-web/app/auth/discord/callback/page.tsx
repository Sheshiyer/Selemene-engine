import { Suspense } from "react";
import { DiscordCallbackClient } from "./discord-callback-client";

export default function DiscordCallbackPage() {
  return (
    <Suspense>
      <DiscordCallbackClient />
    </Suspense>
  );
}
