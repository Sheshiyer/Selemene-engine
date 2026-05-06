"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import {
  clearStoredAuthSession,
  getStoredAuthSession,
  sessionHasAnyPermission,
  setStoredAuthPermissions,
  subscribeToAuthSession,
} from "@/lib/auth";

const adminBillingNav = [
  { href: "/billing", label: "Overview" },
  { href: "/billing/subscriptions", label: "Subscriptions" },
  { href: "/billing/webhook-events", label: "Webhooks" },
  { href: "/billing/reconcile", label: "Reconcile" },
  { href: "/billing/plans", label: "Plans" },
];

const ADMIN_PERMS = [
  "admin:billing:read",
  "admin:billing:subscriptions:cancel",
  "admin:billing:reconcile:trigger",
];

/**
 * Resolves admin permissions on first mount by hitting `/api/v1/admin/session`.
 * The fetch is best-effort — failure leaves `permissions = []` and the layout
 * renders the "no access" state.
 */
async function fetchPermissionsOnce(token: string): Promise<string[]> {
  try {
    const { buildApiUrl } = await import("@/lib/config");
    const res = await fetch(buildApiUrl("/api/v1/admin/session"), {
      method: "GET",
      headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) return [];
    const body = (await res.json()) as { permissions?: string[] };
    return body.permissions ?? [];
  } catch {
    return [];
  }
}

export default function AdminLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  const router = useRouter();
  const pathname = usePathname();
  const authSession = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [resolving, setResolving] = useState(false);

  // Redirect unauthenticated users to login.
  useEffect(() => {
    if (authSession === null) router.replace("/login");
  }, [authSession, router]);

  // Lazily fetch permissions if not yet on the session.
  useEffect(() => {
    if (!authSession) return;
    if (authSession.permissions !== undefined) return;
    if (resolving) return;
    setResolving(true);
    fetchPermissionsOnce(authSession.token)
      .then((perms) => {
        setStoredAuthPermissions(perms);
      })
      .finally(() => setResolving(false));
  }, [authSession, resolving]);

  function handleLogout() {
    clearStoredAuthSession();
    router.replace("/login");
  }

  // Loading state: session present, permissions undefined.
  if (!authSession || authSession.permissions === undefined) {
    return (
      <main className="biofield-shell">
        <div className="biofield-stack">
          <section
            className="biofield-panel"
            style={{ padding: "1.8rem 2rem" }}
          >
            <p className="biofield-eyebrow" style={{ margin: 0 }}>
              Noesis · Admin
            </p>
            <p
              className="biofield-copy"
              style={{ marginTop: "0.5rem", fontSize: "0.88rem" }}
            >
              Resolving operator permissions…
            </p>
          </section>
        </div>
      </main>
    );
  }

  // Forbidden state: resolved permissions but no billing access.
  if (!sessionHasAnyPermission(authSession, ADMIN_PERMS)) {
    return (
      <main className="biofield-shell">
        <div className="biofield-stack">
          <section
            className="biofield-panel"
            style={{ padding: "1.8rem 2rem" }}
          >
            <p className="biofield-eyebrow" style={{ margin: 0 }}>
              Forbidden
            </p>
            <p
              className="biofield-copy"
              style={{ marginTop: "0.5rem", fontSize: "0.88rem" }}
            >
              Your account does not have billing-admin access. Contact a
              platform-admin to grant the <code>billing-admin</code> role.
            </p>
            <div style={{ marginTop: "1rem", display: "flex", gap: "0.5rem" }}>
              <Link
                href="/viewer"
                className="biofield-link"
                style={{ fontSize: "0.84rem", padding: "0.55rem 1rem" }}
              >
                Back to app
              </Link>
              <button
                className="biofield-link"
                onClick={handleLogout}
                style={{
                  fontSize: "0.84rem",
                  padding: "0.55rem 1rem",
                  color: "var(--muted)",
                }}
                type="button"
              >
                Sign out
              </button>
            </div>
          </section>
        </div>
      </main>
    );
  }

  return (
    <main className="biofield-shell">
      <div className="biofield-stack">
        <nav
          className="biofield-panel biofield-shell-nav"
          aria-label="Admin navigation"
        >
          <div
            style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}
          >
            <span
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: "var(--accent)",
                boxShadow: "0 0 8px rgba(var(--accent-rgb), 0.5)",
                flexShrink: 0,
              }}
            />
            <span
              style={{
                fontSize: "0.8rem",
                fontWeight: 600,
                letterSpacing: "0.12em",
                textTransform: "uppercase",
              }}
            >
              Noesis · Admin · Billing
            </span>
          </div>
          <div className="biofield-nav">
            {adminBillingNav.map((item) => (
              <Link
                key={item.href}
                className={`biofield-link${
                  pathname === item.href ? " biofield-link-active" : ""
                }`}
                href={item.href}
                style={{ fontSize: "0.84rem", padding: "0.55rem 1rem" }}
              >
                {item.label}
              </Link>
            ))}
            <Link
              className="biofield-link"
              href="/viewer"
              style={{
                fontSize: "0.84rem",
                padding: "0.55rem 1rem",
                color: "var(--muted)",
              }}
            >
              Exit admin
            </Link>
            <button
              className="biofield-link"
              onClick={handleLogout}
              style={{
                fontSize: "0.84rem",
                padding: "0.55rem 1rem",
                color: "var(--muted)",
              }}
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
