"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useState } from "react";
import { AccessDenied } from "@/components/access-denied";
import { ActionRail, SurfaceCard } from "@/components/admin-primitives";
import { CommandPalette, useCommandPaletteToggle } from "@/components/command-palette";
import { clearAuthToken, getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminSession } from "@/lib/api";
import { hasPermission, requiredPermissionForPath } from "@/lib/permissions";
import { trackEvent } from "@/lib/telemetry";
import type { AdminSession } from "@/types/admin";

interface NavItem {
  href: string;
  label: string;
  permission: string;
  section: "Control" | "Observability";
}

const NAV_ITEMS: NavItem[] = [
  { href: "/dashboard", label: "Dashboard", permission: "admin:analytics:read", section: "Control" },
  { href: "/users", label: "Users", permission: "admin:users:list", section: "Control" },
  { href: "/api-keys", label: "API Keys", permission: "admin:keys:list", section: "Control" },
  { href: "/billing", label: "Billing", permission: "admin:billing:read", section: "Control" },
  { href: "/history-sync", label: "History Sync", permission: "admin:history-sync:read", section: "Control" },
  { href: "/analytics", label: "Analytics", permission: "admin:analytics:read", section: "Observability" },
  { href: "/system", label: "System", permission: "admin:system:read", section: "Observability" },
  { href: "/audit", label: "Audit", permission: "admin:audit:list", section: "Observability" }
];

const ROUTE_META: Record<string, { eyebrow: string; title: string; summary: string }> = {
  "/dashboard": {
    eyebrow: "Overview",
    title: "Control Room",
    summary: "Live platform posture across traffic, failure pressure, and operating context."
  },
  "/users": {
    eyebrow: "Identity",
    title: "User Governance",
    summary: "Inspect account state, access posture, and operator-level interventions."
  },
  "/api-keys": {
    eyebrow: "Access",
    title: "Key Governance",
    summary: "Manage lifecycle, ownership, and destructive controls for machine credentials."
  },
  "/history-sync": {
    eyebrow: "Consistency",
    title: "History Sync",
    summary: "Track ingestion drift, repair posture, and device-linked sync health."
  },
  "/analytics": {
    eyebrow: "Usage",
    title: "Operational Analytics",
    summary: "Review traffic mix, usage concentration, and time-window demand behavior."
  },
  "/system": {
    eyebrow: "Infrastructure",
    title: "System Health",
    summary: "Monitor service health, cache posture, and workflow availability."
  },
  "/audit": {
    eyebrow: "Trace",
    title: "Audit Ledger",
    summary: "Inspect actor, action, target, and request lineage with operational context."
  },
  "/billing": {
    eyebrow: "Revenue",
    title: "Billing & Subscriptions",
    summary: "Subscription state, webhook ingest, plan catalog, and reconcile drift across the Dodo Payments integration."
  }
};

function toRelativeAdminPath(pathname: string): string {
  if (pathname.startsWith("/admin")) {
    const stripped = pathname.slice("/admin".length);
    return stripped === "" ? "/" : stripped;
  }
  return pathname;
}

export default function ProtectedLayout({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const router = useRouter();
  const safePathname = pathname ?? "/admin/dashboard";

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [session, setSession] = useState<AdminSession | null>(null);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);

  const relativePath = useMemo(() => toRelativeAdminPath(safePathname), [safePathname]);
  const requiredPermission = useMemo(
    () => requiredPermissionForPath(safePathname),
    [safePathname]
  );
  const routeMeta = ROUTE_META[relativePath] ?? {
    eyebrow: "Operational Surface",
    title: "Admin Portal",
    summary: "Authenticated administrative controls for Selemene."
  };

  const loadSession = useCallback(
    async (authToken?: string) => {
      setLoading(true);
      try {
        const result = await getAdminSession(authToken);
        setSession(result);
        setError(null);
        setLoading(false);
      } catch (err) {
        clearAuthToken();
        if (err instanceof ApiClientError) {
          if (err.status === 401) {
            const returnTo = encodeURIComponent(relativePath);
            router.replace(`/login?redirect=${returnTo}`);
            return;
          }
          setError(err.payload?.error ?? err.message);
          setLoading(false);
          return;
        }
        setError("Unable to load admin session.");
        setLoading(false);
      }
    },
    [relativePath, router]
  );

  useEffect(() => {
    let cancelled = false;

    async function hydrateSession() {
      try {
        // Prefer the stored bearer token if present, otherwise rely on the
        // Cloudflare Access cookie sent via credentials: "include".
        const result = await getAdminSession(getAuthToken() ?? undefined);
        if (cancelled) {
          return;
        }
        setSession(result);
        setError(null);
        setLoading(false);
      } catch (err) {
        if (cancelled) {
          return;
        }
        clearAuthToken();
        if (err instanceof ApiClientError) {
          if (err.status === 401) {
            const returnTo = encodeURIComponent(relativePath);
            router.replace(`/login?redirect=${returnTo}`);
            return;
          }
          setError(err.payload?.error ?? err.message);
          setLoading(false);
          return;
        }
        setError("Unable to load admin session.");
        setLoading(false);
      }
    }

    void hydrateSession();

    return () => {
      cancelled = true;
    };
  }, [relativePath, router]);

  const canViewRequiredRoute = useMemo(() => {
    if (!requiredPermission || !session) {
      return true;
    }
    return hasPermission(session.permissions, requiredPermission);
  }, [requiredPermission, session]);

  const navSections = useMemo(
    () => [
      {
        label: "Control",
        items: NAV_ITEMS.filter((item) => item.section === "Control")
      },
      {
        label: "Observability",
        items: NAV_ITEMS.filter((item) => item.section === "Observability")
      }
    ],
    []
  );

  const accessState = canViewRequiredRoute ? "granted" : "limited";
  const permissionCount = session?.permissions.length ?? 0;

  const handleSignOut = useCallback(() => {
    trackEvent("admin_sign_out");
    clearAuthToken();
    router.replace("/login");
  }, [router]);

  const handleRefreshSession = useCallback(() => {
    void loadSession(getAuthToken() ?? undefined);
  }, [loadSession]);

  const commandPaletteItems = useMemo(() => {
    if (!session) {
      return [];
    }

    const routeItems = NAV_ITEMS.filter((item) =>
      hasPermission(session.permissions, item.permission)
    ).map((item) => ({
      id: `route-${item.href}`,
      title: `Open ${item.label}`,
      description: ROUTE_META[item.href]?.summary ?? `Navigate to ${item.label}.`,
      section: "Navigate",
      keywords: [item.label, item.href, item.section],
      onSelect: () => router.push(item.href)
    }));

    const actionItems = [
      {
        id: "action-refresh-session",
        title: "Refresh session",
        description: "Reload the current admin session and permission scope.",
        section: "Actions",
        keywords: ["session", "permissions", "reload"],
        shortcutHint: "Shell",
        onSelect: handleRefreshSession
      },
      {
        id: "action-refresh-page",
        title: "Refresh current page",
        description: "Revalidate the current route without leaving the shell.",
        section: "Actions",
        keywords: ["route", "reload", "refresh"],
        shortcutHint: "Route",
        onSelect: () => router.refresh()
      },
      {
        id: "action-sign-out",
        title: "Sign out",
        description: "Clear the admin token and return to the login surface.",
        section: "Actions",
        keywords: ["logout", "exit"],
        shortcutHint: "Auth",
        danger: true,
        onSelect: handleSignOut
      }
    ];

    return [...routeItems, ...actionItems];
  }, [handleRefreshSession, handleSignOut, router, session]);

  useCommandPaletteToggle(() => setCommandPaletteOpen(true));

  if (loading) {
    return (
      <main className="shell-state-wrap">
        <SurfaceCard
          eyebrow="Session"
          title="Loading admin session"
          summary="Verifying credentials, scope, and route permissions."
          className="shell-state-card"
        >
          <div className="ornament-rule" />
        </SurfaceCard>
      </main>
    );
  }

  if (error || !session) {
    return (
      <main className="shell-state-wrap">
        <SurfaceCard
          eyebrow="Session"
          title="Session unavailable"
          summary={error ?? "No session found."}
          actions={
            <ActionRail>
              <button
                type="button"
                className="shell-action-btn"
                onClick={handleSignOut}
              >
                Return to login
              </button>
            </ActionRail>
          }
          className="shell-state-card"
        >
          <div className="helper">The shell cannot render protected routes until the session resolves.</div>
        </SurfaceCard>
      </main>
    );
  }

  return (
    <div className="admin-shell shell-v2">
      <aside className="sidebar shell-sidebar">
        <div className="brand-lockup">
          <div className="telemetry-caption">Tryambakam Noesis</div>
          <div className="brand">Selemene Admin</div>
          <p className="helper">Maps, not prescriptions.</p>
        </div>

        <div className="ornament-rule" />

        <div className="nav-section-stack">
          {navSections.map((section) => (
            <section key={section.label} className="nav-section">
              <div className="telemetry-caption nav-section-label" aria-hidden="true">{section.label}</div>
              <nav className="nav-list" aria-label={section.label}>
                {section.items.map((item) => {
                  const allowed = hasPermission(session.permissions, item.permission);
                  const active = relativePath === item.href;
                  return (
                    <Link
                      key={item.href}
                      href={item.href}
                      className={`nav-item ${active ? "active" : ""}`}
                      title={allowed ? "" : `Requires ${item.permission}`}
                      aria-current={active ? "page" : undefined}
                      aria-disabled={!allowed || undefined}
                    >
                      <span>{item.label}</span>
                      <span className={`pill ${allowed ? "ok" : "danger"}`} aria-hidden="true">
                        {allowed ? "ready" : "locked"}
                      </span>
                    </Link>
                  );
                })}
              </nav>
            </section>
          ))}
        </div>

        <SurfaceCard
          eyebrow="Session"
          title={session.tier}
          summary={session.email}
          className="sidebar-session-card"
        >
          <div className="sidebar-session-grid">
            <div>
              <div className="telemetry-caption">Permissions</div>
              <div className="helper">{permissionCount}</div>
            </div>
            <div>
              <div className="telemetry-caption">Route access</div>
              <div className="helper">{accessState}</div>
            </div>
          </div>
        </SurfaceCard>
      </aside>

      <main className="content shell-content">
        <header className="topbar topbar-v2">
          <div>
            <div className="eyebrow">{routeMeta.eyebrow}</div>
            <h1>{routeMeta.title}</h1>
            <p>{routeMeta.summary}</p>
          </div>
          <ActionRail className="topbar-action-rail" label="Shell actions">
            <button
              type="button"
              className="shell-action-btn"
              onClick={() => setCommandPaletteOpen(true)}
            >
              Command palette
            </button>
            <button
              type="button"
              className="shell-action-btn"
              onClick={handleRefreshSession}
            >
              Refresh session
            </button>
            <Link href="/api-keys" className="shell-action-btn shell-action-link">
              Open keys
            </Link>
            <button
              type="button"
              className="shell-action-btn"
              onClick={handleSignOut}
            >
              Sign out
            </button>
          </ActionRail>
        </header>

        <div className="shell-main-grid">
          <section className="shell-primary-column">
            {!canViewRequiredRoute && requiredPermission ? (
              <AccessDenied permission={requiredPermission} />
            ) : (
              children
            )}
          </section>

          <aside className="shell-context-column">
            <SurfaceCard
              eyebrow="Route"
              title={routeMeta.title}
              summary="Shared context card for the active administrative surface."
            >
              <div className="shell-context-list">
                <div>
                  <div className="telemetry-caption">Path</div>
                  <div className="helper">{relativePath}</div>
                </div>
                <div>
                  <div className="telemetry-caption">Required permission</div>
                  <div className="helper">{requiredPermission ?? "none"}</div>
                </div>
                <div>
                  <div className="telemetry-caption">Access</div>
                  <div className="helper">{accessState}</div>
                </div>
              </div>
            </SurfaceCard>

            <SurfaceCard eyebrow="Operator" title="Current account" summary="Session-derived operator posture.">
              <div className="shell-context-list">
                <div>
                  <div className="telemetry-caption">Email</div>
                  <div className="helper">{session.email}</div>
                </div>
                <div>
                  <div className="telemetry-caption">Tier</div>
                  <div className="helper">{session.tier}</div>
                </div>
                <div>
                  <div className="telemetry-caption">Permission count</div>
                  <div className="helper">{permissionCount}</div>
                </div>
              </div>
            </SurfaceCard>
          </aside>
        </div>
      </main>

      <CommandPalette
        open={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        items={commandPaletteItems}
      />
    </div>
  );
}
