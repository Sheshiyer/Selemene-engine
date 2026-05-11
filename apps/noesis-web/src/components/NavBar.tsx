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
    // Kha Arc atmosphere: void → subtle violet tint
    background: "linear-gradient(90deg, #070B1D 0%, rgba(45,0,80,0.4) 50%, #070B1D 100%)",
    borderBottom: "1px solid var(--line-mid)",
    boxShadow: "0 1px 0 var(--line-faint), inset 0 -1px 0 rgba(11,80,251,0.06)",
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
    fontFamily: "var(--font-display)",
    fontWeight: 800,
    fontSize: "1.1rem",
    color: "var(--signal)",
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
  },
  tabs: {
    display: "flex",
    gap: "0.125rem",
  },
  tab: {
    padding: "0.3rem 0.75rem",
    borderRadius: "var(--r-pill)",
    fontSize: "0.82rem",
    fontWeight: 500,
    color: "var(--muted)",
    transition: "all 0.15s",
    fontFamily: "var(--font-body)",
    letterSpacing: "0.02em",
  },
  tabActive: {
    // Ba Arc gradient pill for active tab
    background: "linear-gradient(90deg, rgba(16,181,167,0.15) 0%, rgba(197,160,23,0.15) 100%)",
    color: "var(--signal)",
    border: "1px solid rgba(197,160,23,0.25)",
  },
  tabHover: {
    color: "var(--text)",
    background: "var(--panel-hover)",
  },
  right: {
    display: "flex",
    alignItems: "center",
    gap: "0.75rem",
  },
  // Avatar circle
  avatar: {
    width: 30,
    height: 30,
    borderRadius: "50%",
    background: "var(--surface-2)",
    border: "1.5px solid var(--signal)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    fontSize: "0.7rem",
    fontWeight: 700,
    fontFamily: "var(--font-display)",
    color: "var(--signal)",
    flexShrink: 0,
  },
  avatarAdmin: {
    border: "1.5px solid var(--c-violet)",
  },
  userInfo: {
    display: "flex",
    flexDirection: "column" as const,
    alignItems: "flex-end",
    gap: "0.1rem",
  },
  userName: {
    fontSize: "0.72rem",
    color: "var(--text)",
    fontWeight: 600,
    fontFamily: "var(--font-body)",
  },
  userTier: {
    fontSize: "0.62rem",
    color: "var(--muted)",
    fontFamily: "var(--font-mono)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
  },
  badge: {
    display: "flex",
    alignItems: "center",
    gap: "0.375rem",
    fontSize: "0.72rem",
    color: "var(--c-emerald)",
    fontFamily: "var(--font-mono)",
  },
  dot: {
    width: 6,
    height: 6,
    borderRadius: "50%",
    background: "var(--c-emerald)",
    flexShrink: 0,
  },
  biofieldDot: {
    width: 5,
    height: 5,
    borderRadius: "50%",
    background: "var(--c-emerald)",
    display: "inline-block",
    marginLeft: 4,
    boxShadow: "0 0 4px var(--c-emerald)",
    flexShrink: 0,
  },
  adminBadge: {
    fontSize: "0.6rem",
    padding: "0.12rem 0.45rem",
    borderRadius: "var(--r-xs)",
    background: "rgba(45,0,80,0.35)",
    color: "var(--c-emerald)",
    fontWeight: 700,
    border: "1px solid rgba(45,0,80,0.6)",
    textDecoration: "none",
    letterSpacing: "0.06em",
    fontFamily: "var(--font-mono)",
  },
  logoutBtn: {
    fontSize: "0.72rem",
    color: "var(--muted)",
    padding: "0.2rem 0.5rem",
    borderRadius: "var(--r-xs)",
    border: "1px solid var(--line-faint)",
    cursor: "pointer",
    background: "transparent",
    fontFamily: "var(--font-body)",
    transition: "border-color 0.15s, color 0.15s",
  },
};

const BIOFIELD_BASE_URL = "https://biofield.tryambakam.space";

function getBiofieldHref(): string {
  if (typeof window === "undefined") return BIOFIELD_BASE_URL;
  const key = localStorage.getItem("noesis_api_key");
  // Only forward JWTs (eyJ prefix). API keys (nk_ prefix) are not valid for biofield-web.
  if (key && key.startsWith("eyJ")) {
    return `${BIOFIELD_BASE_URL}/login#token=${key}`;
  }
  return BIOFIELD_BASE_URL;
}

const TABS = [
  { href: "/engines", label: "Engines", external: false },
  { href: "/readings", label: "Readings", external: false },
  { href: BIOFIELD_BASE_URL, label: "Biofield", external: true },
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
                href={getBiofieldHref()}
                target="_blank"
                rel="noreferrer"
                style={styles.tab}
              >
                {t.label}
                {/* Emerald bioluminescent dot when session exists */}
                {authed && <span style={styles.biofieldDot} aria-hidden />}
                {!authed && " ↗"}
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
                ADM
              </a>
            )}
            {/* Avatar initial circle */}
            <div
              style={{
                ...styles.avatar,
                ...(profile?.is_admin ? styles.avatarAdmin : {}),
              }}
              title={profile?.email}
            >
              {(profile?.full_name || profile?.email || "?")
                .charAt(0)
                .toUpperCase()}
            </div>
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
              border: "1px solid rgba(197,160,23,0.4)",
              color: "var(--signal)",
              letterSpacing: "0.04em",
            }}
          >
            Sign in
          </Link>
        )}
      </div>
    </nav>
  );
}
