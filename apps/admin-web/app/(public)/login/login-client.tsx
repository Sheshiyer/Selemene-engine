"use client";

import Link from "next/link";
import { useMemo } from "react";
import { useSearchParams } from "next/navigation";

function normalizeRedirect(rawTarget: string | null): string {
  if (!rawTarget || rawTarget.trim() === "") return "/dashboard";
  if (!rawTarget.startsWith("/")) return "/dashboard";
  if (rawTarget.startsWith("/admin")) {
    const stripped = rawTarget.slice("/admin".length);
    return stripped === "" ? "/dashboard" : stripped;
  }
  return rawTarget;
}

export function LoginClient() {
  const searchParams = useSearchParams();
  const redirectTarget = useMemo(
    () => normalizeRedirect(searchParams.get("redirect")),
    [searchParams]
  );

  return (
    <main className="login-wrap">
      <section className="login-card">
        <h1>Selemene Admin</h1>
        <p>Access is managed by Cloudflare Zero Trust.</p>
        <p>If you can see this page directly, your Access policy may not be applied to the admin route.</p>
        <Link className="discord-btn" href={redirectTarget}>Continue to dashboard</Link>
      </section>
    </main>
  );
}
