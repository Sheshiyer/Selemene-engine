"use client";

// ─── Root `/` redirect ──────────────────────────────────────────────────
// Authenticated users → /engines (existing dashboard).
// Public/anonymous users → /get-reading (the new public form). This
// makes the app's front door the reading-creation flow for new visitors.
// /auth remains accessible as an explicit sign-in route.

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { isAuthenticated } from "@/lib/auth";

export default function RootPage() {
  const router = useRouter();

  useEffect(() => {
    router.replace(isAuthenticated() ? "/engines" : "/get-reading");
  }, [router]);

  return null;
}
