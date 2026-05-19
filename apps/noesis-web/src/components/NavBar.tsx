"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { isAuthenticated, clearApiKey, getUserProfile, type UserProfile } from "@/lib/auth";
import { useEffect, useState, type CSSProperties } from "react";

/* ── Tier Badge Styling ───────────────────────────────────── */

type Tier = "free" | "pro" | "architect";

const TIER_COLORS: Record<Tier, { bg: string; color: string }> = {
  free:      { bg: "rgba(255,255,255,0.1)",   color: "rgba(255,255,255,0.4)" },
  pro:       { bg: "rgba(16,181,167,0.15)",    color: "var(--c-emerald)" },
  architect: { bg: "rgba(197,160,23,0.15)",    color: "var(--c-gold)" },
};

function tierBadgeStyle(tier: string): CSSProperties {
  const t = TIER_COLORS[tier as Tier] ?? TIER_COLORS.free;
  return {
    fontSize: "0.55rem",
    fontWeight: 600,
    fontFamily: "var(--font-mono)",
    textTransform: "lowercase",
    letterSpacing: "0.04em",
    padding: "1px 6px",
    borderRadius: 10,
    background: t.bg,
    color: t.color,
    lineHeight: 1.5,
    whiteSpace: "nowrap",
  };
}

/* ── Styles ───────────────────────────────────────────────── */

const styles = {
  nav: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "0 1.25rem",
    height: 56,
    background: "var(--bg)",
    borderBottom: "1px solid var(--line-mid)",
    boxShadow: "inset 0 -1px 0 rgba(11,80,251,0.10)",
    position: "sticky" as const,
    top: 0,
    zIndex: 100,
    /* Instrument-voice hairline at the bottom edge — acts as the chrome
       indicator that the operator UI is "armed". */
  },
  left: {
    display: "flex",
    alignItems: "center",
    gap: "1.5rem",
  },
  logo: {
    fontFamily: "var(--font-display)",
    fontWeight: 800,
    fontSize: "1rem",
    color: "var(--signal)",
    letterSpacing: "0.18em",
    textTransform: "uppercase" as const,
    display: "inline-flex",
    alignItems: "center",
    gap: "0.5rem",
  },
  logoMark: {
    /* trinity-hex logomark — a small SVG would replace this in v2;
       for now use the geometric ◈ at uppercase-cap height to read
       as an Instrument-voice glyph rather than a generic word logo */
    fontSize: "0.85rem",
    color: "var(--c-emerald)",
    filter: "drop-shadow(0 0 6px rgba(16,181,167,0.45))",
  },
  tabs: {
    display: "flex",
    gap: "0.25rem",
  },
  tab: {
    padding: "0.35rem 0.85rem",
    borderRadius: "var(--r-xs)",
    fontSize: "0.74rem",
    fontWeight: 600,
    color: "var(--muted)",
    transition: "color 0.15s, border-color 0.15s, background 0.15s",
    fontFamily: "var(--font-mono)",
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    border: "1px solid transparent",
    background: "transparent",
  },
  tabActive: {
    color: "var(--signal)",
    border: "1px solid rgba(197,160,23,0.45)",
    background: "rgba(197,160,23,0.06)",
  },
  right: {
    display: "flex",
    alignItems: "center",
    gap: "0.75rem",
  },
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
  avatarGroup: {
    display: "flex",
    flexDirection: "column" as const,
    alignItems: "center",
    gap: "3px",
    position: "relative" as const,
  },
  adminTag: {
    fontSize: "0.6rem",
    padding: "1px 5px",
    borderRadius: 10,
    background: "rgba(197,160,23,0.2)",
    color: "var(--c-gold)",
    fontWeight: 600,
    fontFamily: "var(--font-mono)",
    lineHeight: 1.4,
    whiteSpace: "nowrap" as const,
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
  tooltip: {
    position: "absolute" as const,
    top: "calc(100% + 6px)",
    right: 0,
    background: "var(--surface-2)",
    border: "1px solid var(--line-mid)",
    borderRadius: 6,
    padding: "6px 10px",
    fontSize: "0.68rem",
    color: "var(--text)",
    fontFamily: "var(--font-mono)",
    whiteSpace: "nowrap" as const,
    zIndex: 200,
    boxShadow: "0 4px 12px rgba(0,0,0,0.4)",
    display: "flex",
    flexDirection: "column" as const,
    gap: "2px",
    pointerEvents: "none" as const,
  },
};

const BIOFIELD_BASE_URL = "https://biofield.tryambakam.space";

function getBiofieldHref(): string {
  if (typeof window === "undefined") return BIOFIELD_BASE_URL;
  const key = localStorage.getItem("noesis_api_key");
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

/* ── Profile Section ──────────────────────────────────────── */

function ProfileArea({ profile }: { profile: UserProfile | null }) {
  const [hovering, setHovering] = useState(false);

  if (!profile) {
    return (
      <span style={styles.badge}>
        <span style={styles.dot} />
        API Key Active
      </span>
    );
  }

  const initials = (profile.full_name || profile.email || "?").charAt(0).toUpperCase();

  return (
    <div
      style={{ display: "flex", alignItems: "center", gap: "0.6rem", position: "relative" }}
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
    >
      {/* Admin link badge */}
      {profile.is_admin && (
        <a
          href="https://144.tryambakam.space/admin"
          target="_blank"
          rel="noreferrer"
          style={styles.adminBadge}
        >
          ADM
        </a>
      )}

      {/* Avatar column: initials circle + tier badge + admin tag */}
      <div style={styles.avatarGroup}>
        {/* Admin indicator above avatar */}
        {profile.is_admin && (
          <span style={styles.adminTag}>✦ admin</span>
        )}

        {/* Avatar circle */}
        <div
          style={{
            ...styles.avatar,
            ...(profile.is_admin ? styles.avatarAdmin : {}),
          }}
        >
          {initials}
        </div>

        {/* Tier badge below avatar */}
        <span style={tierBadgeStyle(profile.tier)}>{profile.tier}</span>
      </div>

      {/* Name + tier text */}
      <div style={styles.userInfo}>
        <span style={styles.userName}>
          {profile.full_name || profile.email.split("@")[0]}
        </span>
        <span style={styles.userTier}>{profile.tier}</span>
      </div>

      {/* Hover tooltip */}
      {hovering && (
        <div style={styles.tooltip}>
          <span style={{ color: "var(--muted)" }}>{profile.email}</span>
          <span>tier: <strong style={{ color: "var(--signal)" }}>{profile.tier}</strong></span>
        </div>
      )}
    </div>
  );
}

/* ── NavBar ───────────────────────────────────────────────── */

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
        <Link href="/engines" style={styles.logo} aria-label="Noesis — home">
          <span style={styles.logoMark} aria-hidden>◈</span>
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
            <ProfileArea profile={profile} />
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
