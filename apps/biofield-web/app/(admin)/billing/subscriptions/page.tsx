"use client";

import Link from "next/link";
import { useEffect, useState, useSyncExternalStore } from "react";
import { getStoredAuthSession, subscribeToAuthSession } from "@/lib/auth";
import {
  listAdminSubscriptions,
  type AdminSubscriptionsResponse,
} from "@/lib/admin-api";

const STATUS_OPTIONS = [
  "",
  "active",
  "past_due",
  "canceled",
  "trialing",
  "incomplete",
  "expired",
];

const PAGE_SIZE = 50;

export default function AdminSubscriptionsListPage() {
  const session = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [data, setData] = useState<AdminSubscriptionsResponse | null>(null);
  const [statusFilter, setStatusFilter] = useState("");
  const [offset, setOffset] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!session?.token) return;
    let cancelled = false;
    setLoading(true);
    listAdminSubscriptions(session.token, {
      status: statusFilter || undefined,
      limit: PAGE_SIZE,
      offset,
    })
      .then((d) => {
        if (!cancelled) {
          setData(d);
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
  }, [session?.token, statusFilter, offset]);

  const items = data?.items ?? [];
  const total = data?.total ?? 0;
  const showingFrom = total === 0 ? 0 : offset + 1;
  const showingTo = Math.min(offset + items.length, total);

  return (
    <section className="biofield-panel" style={{ padding: "1.6rem 2rem" }}>
      <header
        style={{
          display: "flex",
          alignItems: "baseline",
          justifyContent: "space-between",
          marginBottom: "1rem",
          gap: "1rem",
          flexWrap: "wrap",
        }}
      >
        <div>
          <p className="biofield-eyebrow" style={{ margin: 0 }}>
            Billing · Subscriptions
          </p>
          <h1
            style={{
              margin: "0.3rem 0 0",
              fontSize: "1.4rem",
              fontWeight: 600,
            }}
          >
            All subscriptions
          </h1>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "0.6rem" }}>
          <label
            htmlFor="status-filter"
            style={{ fontSize: "0.78rem", color: "var(--muted)" }}
          >
            Status:
          </label>
          <select
            id="status-filter"
            value={statusFilter}
            onChange={(e) => {
              setOffset(0);
              setStatusFilter(e.target.value);
            }}
            style={{
              padding: "0.45rem 0.75rem",
              fontSize: "0.84rem",
              background: "transparent",
              border: "1px solid rgba(var(--signal-rgb), 0.28)",
              borderRadius: "var(--r-pill)",
              color: "inherit",
            }}
          >
            {STATUS_OPTIONS.map((s) => (
              <option key={s || "all"} value={s}>
                {s || "all"}
              </option>
            ))}
          </select>
        </div>
      </header>

      {loading && <p className="biofield-copy">Loading…</p>}
      {error && (
        <p className="biofield-copy" style={{ color: "var(--accent)" }}>
          Failed: {error}
        </p>
      )}

      {data && (
        <>
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
                  <th style={{ padding: "0.5rem 0.6rem" }}>ID</th>
                  <th style={{ padding: "0.5rem 0.6rem" }}>User</th>
                  <th style={{ padding: "0.5rem 0.6rem" }}>Status</th>
                  <th style={{ padding: "0.5rem 0.6rem" }}>Period end</th>
                  <th style={{ padding: "0.5rem 0.6rem" }}>Updated</th>
                </tr>
              </thead>
              <tbody>
                {items.map((s) => (
                  <tr
                    key={s.id}
                    style={{
                      borderTop: "1px solid rgba(var(--signal-rgb), 0.12)",
                    }}
                  >
                    <td
                      style={{
                        padding: "0.5rem 0.6rem",
                        fontFamily: "monospace",
                      }}
                    >
                      <Link
                        href={`/billing/subscriptions/${s.id}`}
                        className="biofield-link"
                        style={{ padding: 0, fontSize: "0.8rem" }}
                      >
                        {s.id.slice(0, 8)}…
                      </Link>
                    </td>
                    <td
                      style={{
                        padding: "0.5rem 0.6rem",
                        fontFamily: "monospace",
                      }}
                    >
                      {s.user_id.slice(0, 8)}…
                    </td>
                    <td style={{ padding: "0.5rem 0.6rem" }}>{s.status}</td>
                    <td style={{ padding: "0.5rem 0.6rem" }}>
                      {s.current_period_end
                        ? new Date(s.current_period_end).toLocaleDateString()
                        : "—"}
                    </td>
                    <td style={{ padding: "0.5rem 0.6rem" }}>
                      {new Date(s.updated_at).toLocaleString()}
                    </td>
                  </tr>
                ))}
                {items.length === 0 && (
                  <tr>
                    <td
                      colSpan={5}
                      style={{
                        padding: "1rem",
                        textAlign: "center",
                        color: "var(--muted)",
                      }}
                    >
                      No subscriptions match.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>

          <footer
            style={{
              marginTop: "1rem",
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              fontSize: "0.78rem",
            }}
          >
            <span style={{ color: "var(--muted)" }}>
              Showing {showingFrom}–{showingTo} of {total}
            </span>
            <div style={{ display: "flex", gap: "0.4rem" }}>
              <button
                type="button"
                className="biofield-link"
                disabled={offset === 0}
                onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
                style={{
                  fontSize: "0.78rem",
                  padding: "0.4rem 0.9rem",
                  opacity: offset === 0 ? 0.4 : 1,
                }}
              >
                ← Prev
              </button>
              <button
                type="button"
                className="biofield-link"
                disabled={offset + items.length >= total}
                onClick={() => setOffset(offset + PAGE_SIZE)}
                style={{
                  fontSize: "0.78rem",
                  padding: "0.4rem 0.9rem",
                  opacity: offset + items.length >= total ? 0.4 : 1,
                }}
              >
                Next →
              </button>
            </div>
          </footer>
        </>
      )}
    </section>
  );
}
