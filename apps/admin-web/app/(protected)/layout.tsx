"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useEffect, useMemo, useState } from "react";
import { AccessDenied } from "@/components/access-denied";
import { clearAuthToken, getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminSession } from "@/lib/api";
import { hasPermission, requiredPermissionForPath } from "@/lib/permissions";
import type { AdminSession } from "@/types/admin";

interface NavItem {
  href: string;
  label: string;
  permission: string;
}

const NAV_ITEMS: NavItem[] = [
  { href: "/dashboard", label: "Dashboard", permission: "admin:analytics:read" },
  { href: "/users", label: "Users", permission: "admin:users:list" },
  { href: "/api-keys", label: "API Keys", permission: "admin:keys:list" },
  { href: "/history-sync", label: "History Sync", permission: "admin:history-sync:read" },
  { href: "/analytics", label: "Analytics", permission: "admin:analytics:read" },
  { href: "/system", label: "System", permission: "admin:system:read" },
  { href: "/audit", label: "Audit", permission: "admin:audit:list" }
];

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

  const relativePath = useMemo(() => toRelativeAdminPath(safePathname), [safePathname]);
  const requiredPermission = useMemo(
    () => requiredPermissionForPath(safePathname),
    [safePathname]
  );

  useEffect(() => {
    let cancelled = false;
    const token = getAuthToken();

    if (!token) {
      const returnTo = encodeURIComponent(relativePath);
      router.replace(`/login?redirect=${returnTo}`);
      return;
    }
    const authToken = token;

    async function loadSession() {
      try {
        const result = await getAdminSession(authToken);
        if (!cancelled) {
          setSession(result);
          setError(null);
          setLoading(false);
        }
      } catch (err) {
        if (!cancelled) {
          clearAuthToken();
          if (err instanceof ApiClientError && err.status === 401) {
            const returnTo = encodeURIComponent(relativePath);
            router.replace(`/login?redirect=${returnTo}`);
            return;
          }
          setError("Unable to load admin session.");
          setLoading(false);
        }
      }
    }

    void loadSession();

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

  if (loading) {
    return (
      <main className="content">
        <section className="panel">
          <h2>Loading admin session...</h2>
          <p className="helper">Verifying credentials and permissions.</p>
        </section>
      </main>
    );
  }

  if (error || !session) {
    return (
      <main className="content">
        <section className="panel">
          <h2>Session unavailable</h2>
          <p className="helper">{error ?? "No session found."}</p>
          <button
            type="button"
            onClick={() => {
              clearAuthToken();
              router.replace("/login");
            }}
          >
            Return to login
          </button>
        </section>
      </main>
    );
  }

  return (
    <div className="admin-shell">
      <aside className="sidebar">
        <div className="brand">Selemene · Admin</div>
        <nav className="nav-list">
          {NAV_ITEMS.map((item) => {
            const allowed = hasPermission(session.permissions, item.permission);
            const active = relativePath === item.href;
            return (
              <Link
                key={item.href}
                href={item.href}
                className={`nav-item ${active ? "active" : ""}`}
                title={allowed ? "" : `Requires ${item.permission}`}
              >
                {item.label}
                {!allowed ? " · locked" : ""}
              </Link>
            );
          })}
        </nav>
      </aside>

      <main className="content">
        <header className="topbar">
          <div>
            <h1>Admin Portal</h1>
            <p>
              {session.email} · tier: {session.tier}
            </p>
          </div>
          <button
            type="button"
            onClick={() => {
              clearAuthToken();
              router.replace("/login");
            }}
          >
            Sign out
          </button>
        </header>

        {!canViewRequiredRoute && requiredPermission ? (
          <AccessDenied permission={requiredPermission} />
        ) : (
          children
        )}
      </main>
    </div>
  );
}
