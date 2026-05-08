"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { isAuthenticated, clearApiKey } from "@/lib/auth";
import { useEffect, useState } from "react";

const styles = {
  nav: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "0 1.5rem",
    height: 56,
    background: "var(--bg-elevated)",
    borderBottom: "1px solid var(--line)",
    position: "sticky" as const,
    top: 0,
    zIndex: 100,
  },
  left: {
    display: "flex",
    alignItems: "center",
    gap: "2rem",
  },
  logo: {
    fontFamily: "'Exo 2', sans-serif",
    fontWeight: 800,
    fontSize: "1.25rem",
    color: "var(--gold)",
    letterSpacing: "0.05em",
  },
  tabs: {
    display: "flex",
    gap: "0.25rem",
  },
  tab: {
    padding: "0.375rem 0.75rem",
    borderRadius: "var(--radius)",
    fontSize: "0.875rem",
    fontWeight: 500,
    color: "var(--text-muted)",
    transition: "all 0.15s",
  },
  tabActive: {
    background: "var(--gold-soft)",
    color: "var(--gold)",
  },
  right: {
    display: "flex",
    alignItems: "center",
    gap: "0.75rem",
  },
  badge: {
    display: "flex",
    alignItems: "center",
    gap: "0.375rem",
    fontSize: "0.75rem",
    color: "var(--emerald)",
    fontFamily: "'IBM Plex Mono', monospace",
  },
  dot: {
    width: 6,
    height: 6,
    borderRadius: "50%",
    background: "var(--emerald)",
  },
  logoutBtn: {
    fontSize: "0.75rem",
    color: "var(--text-dim)",
    padding: "0.25rem 0.5rem",
    borderRadius: "var(--radius)",
    border: "1px solid var(--line)",
  },
};

const TABS = [
  { href: "/engines", label: "Engines" },
  { href: "/readings", label: "Readings" },
];

export default function NavBar() {
  const pathname = usePathname();
  const [authed, setAuthed] = useState(false);

  useEffect(() => {
    setAuthed(isAuthenticated());
  }, []);

  const handleLogout = () => {
    clearApiKey();
    window.location.href = "/";
  };

  return (
    <nav style={styles.nav}>
      <div style={styles.left}>
        <Link href="/engines" style={styles.logo}>
          NOESIS
        </Link>
        <div style={styles.tabs}>
          {TABS.map((t) => (
            <Link
              key={t.href}
              href={t.href}
              style={{
                ...styles.tab,
                ...(pathname.startsWith(t.href) ? styles.tabActive : {}),
              }}
            >
              {t.label}
            </Link>
          ))}
        </div>
      </div>
      <div style={styles.right}>
        {authed ? (
          <>
            <span style={styles.badge}>
              <span style={styles.dot} />
              API Key Active
            </span>
            <button style={styles.logoutBtn} onClick={handleLogout}>
              Clear Key
            </button>
          </>
        ) : (
          <Link
            href="/"
            style={{
              ...styles.tab,
              border: "1px solid var(--gold)",
              color: "var(--gold)",
            }}
          >
            Enter API Key
          </Link>
        )}
      </div>
    </nav>
  );
}
