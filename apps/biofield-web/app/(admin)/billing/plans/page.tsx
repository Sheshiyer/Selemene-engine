"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import { getStoredAuthSession, subscribeToAuthSession } from "@/lib/auth";
import { listAdminPlans, type AdminPlanItem } from "@/lib/admin-api";

export default function AdminPlansPage() {
  const session = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [items, setItems] = useState<AdminPlanItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!session?.token) return;
    let cancelled = false;
    setLoading(true);
    listAdminPlans(session.token)
      .then((d) => {
        if (!cancelled) {
          setItems(d.items);
          setError(null);
        }
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(e instanceof Error ? e.message : "Failed");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [session?.token]);

  return (
    <section className="biofield-panel" style={{ padding: "1.6rem 2rem" }}>
      <header style={{ marginBottom: "1rem" }}>
        <p className="biofield-eyebrow" style={{ margin: 0 }}>
          Billing · Plans
        </p>
        <h1
          style={{
            margin: "0.3rem 0 0",
            fontSize: "1.4rem",
            fontWeight: 600,
          }}
        >
          Plan catalog
        </h1>
        <p
          className="biofield-copy"
          style={{
            fontSize: "0.78rem",
            color: "var(--muted)",
            marginTop: "0.4rem",
          }}
        >
          Source of truth for the <code>plan_catalog</code> table. Editing is
          intentionally not exposed here — change via migration + restart.
        </p>
      </header>

      {loading && <p className="biofield-copy">Loading…</p>}
      {error && (
        <p className="biofield-copy" style={{ color: "var(--accent)" }}>
          {error}
        </p>
      )}

      <div style={{ overflowX: "auto" }}>
        <table
          style={{
            width: "100%",
            borderCollapse: "collapse",
            fontSize: "0.84rem",
          }}
        >
          <thead>
            <tr style={{ textAlign: "left", color: "var(--muted)" }}>
              <th style={{ padding: "0.5rem 0.6rem" }}>Code</th>
              <th style={{ padding: "0.5rem 0.6rem" }}>Display name</th>
              <th style={{ padding: "0.5rem 0.6rem" }}>Active</th>
              <th style={{ padding: "0.5rem 0.6rem" }}>Dodo product ID</th>
              <th style={{ padding: "0.5rem 0.6rem" }}>Updated</th>
            </tr>
          </thead>
          <tbody>
            {items.map((p) => (
              <tr
                key={p.id}
                style={{
                  borderTop: "1px solid rgba(var(--signal-rgb), 0.12)",
                }}
              >
                <td style={{ padding: "0.5rem 0.6rem" }}>{p.code}</td>
                <td style={{ padding: "0.5rem 0.6rem" }}>{p.display_name}</td>
                <td style={{ padding: "0.5rem 0.6rem" }}>
                  {p.is_active ? "yes" : "no"}
                </td>
                <td
                  style={{
                    padding: "0.5rem 0.6rem",
                    fontFamily: "monospace",
                    fontSize: "0.76rem",
                  }}
                >
                  {p.dodo_product_id ?? "—"}
                </td>
                <td style={{ padding: "0.5rem 0.6rem" }}>
                  {new Date(p.updated_at).toLocaleString()}
                </td>
              </tr>
            ))}
            {items.length === 0 && !loading && (
              <tr>
                <td
                  colSpan={5}
                  style={{
                    padding: "1rem",
                    textAlign: "center",
                    color: "var(--muted)",
                  }}
                >
                  No plans loaded. Did you run the seed migration?
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
