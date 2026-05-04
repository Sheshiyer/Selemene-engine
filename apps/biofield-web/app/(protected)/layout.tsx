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
  { href: "/viewer", label: "Viewer" },
  { href: "/history", label: "History" },
];

export default function ProtectedLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const router = useRouter();
  const pathname = usePathname();
  const authSession = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );

  useEffect(() => {
    if (!authSession) {
      router.replace("/login");
    }
  }, [authSession, router]);

  function handleLogout() {
    clearStoredAuthSession();
    router.replace("/login");
  }

  if (!authSession) {
    return (
      <main className="biofield-shell">
        <div className="biofield-stack">
          <section className="biofield-panel">
            <p className="biofield-eyebrow">Noesis · Biofield</p>
            <p className="biofield-copy">Verifying your session…</p>
          </section>
        </div>
      </main>
    );
  }

  return (
    <main className="biofield-shell">
      <div className="biofield-stack">
        <section className="biofield-panel biofield-shell-nav">
          <div>
            <h1>Selemene Biofield</h1>
            <p className="biofield-copy biofield-shell-copy">
              Signed in as <span className="biofield-mono">{authSession.email}</span> · {authSession.tier}
            </p>
          </div>
          <nav className="biofield-nav" aria-label="Biofield navigation">
            {navItems.map((item) => (
              <Link
                key={item.href}
                className={`biofield-link${pathname === item.href ? " biofield-link-active" : ""}`}
                href={item.href}
              >
                {item.label}
              </Link>
            ))}
            <button className="biofield-link" onClick={handleLogout} type="button">
              Sign out
            </button>
          </nav>
        </section>
        {children}
      </div>
    </main>
  );
}
