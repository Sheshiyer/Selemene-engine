"use client";

import { useEffect, useSyncExternalStore } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import {
  clearStoredAuthSession,
  getStoredAuthSession,
  subscribeToAuthSession,
} from "@/lib/auth";

const navItems = [
  { href: "/viewer", label: "Session" },
  { href: "/history", label: "Archive" },
];

function TierBadge({ tier }: { tier: string }) {
  return (
    <span style={{
      display: "inline-flex",
      alignItems: "center",
      padding: "0.2rem 0.55rem",
      borderRadius: "var(--r-pill)",
      fontSize: "0.68rem",
      fontWeight: 600,
      letterSpacing: "0.1em",
      textTransform: "uppercase",
      border: "1px solid rgba(var(--signal-rgb), 0.28)",
      background: "rgba(var(--signal-rgb), 0.1)",
      color: "var(--signal)",
    }}>
      {tier}
    </span>
  );
}

export default function ProtectedLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  const router = useRouter();
  const pathname = usePathname();
  const authSession = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );

  useEffect(() => {
    if (!authSession) router.replace("/login");
  }, [authSession, router]);

  function handleLogout() {
    clearStoredAuthSession();
    router.replace("/login");
  }

  if (!authSession) {
    return (
      <main className="biofield-shell">
        <div className="biofield-stack">
          <section className="biofield-panel" style={{ padding: "1.8rem 2rem" }}>
            <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
              <span style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--accent)", animation: "pulse-dot 1.4s ease-in-out infinite" }} />
              <p className="biofield-eyebrow" style={{ margin: 0 }}>Noesis · Biofield</p>
            </div>
            <p className="biofield-copy" style={{ marginTop: "0.5rem", fontSize: "0.88rem" }}>Verifying your session…</p>
          </section>
        </div>
      </main>
    );
  }

  return (
    <main className="biofield-shell">
      <div className="biofield-stack">
        {/* ── Dynamic island nav pill ── */}
        <nav className="biofield-panel biofield-shell-nav" aria-label="Biofield navigation">
          {/* Brand */}
          <div style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}>
            <span style={{
              width: 8, height: 8, borderRadius: "50%",
              background: "var(--accent)",
              boxShadow: "0 0 8px rgba(var(--accent-rgb), 0.5)",
              animation: "pulse-dot 2.8s ease-in-out infinite",
              flexShrink: 0,
            }} />
            <span style={{ fontSize: "0.8rem", fontWeight: 600, letterSpacing: "0.12em", textTransform: "uppercase" }}>
              Noesis · Biofield
            </span>
            <TierBadge tier={authSession.tier} />
          </div>

          {/* Links */}
          <div className="biofield-nav">
            {navItems.map((item) => (
              <Link
                key={item.href}
                className={`biofield-link${pathname === item.href ? " biofield-link-active" : ""}`}
                href={item.href}
                style={{ fontSize: "0.84rem", padding: "0.55rem 1rem" }}
              >
                {item.label}
              </Link>
            ))}
            <button
              className="biofield-link"
              onClick={handleLogout}
              style={{ fontSize: "0.84rem", padding: "0.55rem 1rem", color: "var(--muted)" }}
              type="button"
            >
              Sign out
            </button>
          </div>
        </nav>

        {children}
      </div>
    </main>
  );
}
