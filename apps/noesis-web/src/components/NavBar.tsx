"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { isAuthenticated, clearApiKey, getUserProfile, type UserProfile } from "@/lib/auth";
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
  userInfo: {
    display: "flex",
    flexDirection: "column" as const,
    alignItems: "flex-end",
    gap: "0.1rem",
  },
  userName: {
    fontSize: "0.75rem",
    color: "var(--text)",
    fontWeight: 600,
    fontFamily: "'Space Grotesk', sans-serif",
  },
  userTier: {
    fontSize: "0.65rem",
    color: "var(--text-muted)",
    fontFamily: "'IBM Plex Mono', monospace",
    textTransform: "uppercase" as const,
    letterSpacing: "0.05em",
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
  adminBadge: {
    fontSize: "0.65rem",
    padding: "0.15rem 0.5rem",
    borderRadius: 4,
    background: "rgba(99,102,241,0.15)",
    color: "#818cf8",
    fontWeight: 700,
    border: "1px solid rgba(99,102,241,0.3)",
    textDecoration: "none",
    letterSpacing: "0.04em",
  },
  logoutBtn: {
    fontSize: "0.75rem",
    color: "var(--text-dim)",
    padding: "0.25rem 0.5rem",
    borderRadius: "var(--radius)",
    border: "1px solid var(--line)",
    cursor: "pointer",
    background: "transparent",
    fontFamily: "'Space Grotesk', sans-serif",
  },
};

const TABS = [
  { href: "/engines", label: "Engines", external: false },
  { href: "/readings", label: "Readings", external: false },
  { href: "https://biofield-web.vercel.app", label: "Biofield", external: true },
];

export default function NavBar() {
  const pathname = usePathname();
  const [authed, setAuthed] = useState(false);
  const [profile, setProfile] = useState<UserProfile | null>(null);

  useEffect(() => {
    setAuthed(isAuthenticated());
    setProfile(getUserProfile());
  }, []);

  const handleLogout = () => {
    clearApiKey();
    window.location.href = "/auth";
  };

  return (
    <nav style={styles.nav}>
      <div style={styles.left}>
        <Link href="/engines" style={styles.logo}>
          NOESIS
        </Link>
        <div style={styles.tabs}>
          {TABS.map((t) =>
            t.external ? (
              <a
                key={t.href}
                href={t.href}
                target="_blank"
                rel="noreferrer"
                style={styles.tab}
              >
                {t.label} ↗
              </a>
            ) : (
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
            )
          )}
        </div>
      </div>
      <div style={styles.right}>
        {authed ? (
          <>
            {profile?.is_admin && (
              <a
                href="https://144.tryambakam.space/admin"
                target="_blank"
                rel="noreferrer"
                style={styles.adminBadge}
              >
                ⚡ ADMIN
              </a>
            )}
            {profile ? (
              <div style={styles.userInfo}>
                <span style={styles.userName}>
                  {profile.full_name || profile.email.split("@")[0]}
                </span>
                <span style={styles.userTier}>{profile.tier}</span>
              </div>
            ) : (
              <span style={styles.badge}>
                <span style={styles.dot} />
                API Key Active
              </span>
            )}
            <button style={styles.logoutBtn} onClick={handleLogout}>
              Sign out
            </button>
          </>
        ) : (
          <Link
            href="/auth"
            style={{
              ...styles.tab,
              border: "1px solid var(--gold)",
              color: "var(--gold)",
            }}
          >
            Sign in
          </Link>
        )}
      </div>
    </nav>
  );
}
