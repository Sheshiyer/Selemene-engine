"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import { getStoredAuthSession, subscribeToAuthSession } from "@/lib/auth";
import {
  getAdminBillingOverview,
  type AdminBillingOverview,
} from "@/lib/admin-api";

const cardStyle: React.CSSProperties = {
  padding: "1.2rem 1.4rem",
  display: "flex",
  flexDirection: "column",
  gap: "0.4rem",
};

const labelStyle: React.CSSProperties = {
  fontSize: "0.7rem",
  letterSpacing: "0.12em",
  textTransform: "uppercase",
  color: "var(--muted)",
};

const valueStyle: React.CSSProperties = {
  fontSize: "1.6rem",
  fontWeight: 600,
};

function formatUsd(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return "—";
  return `$${n.toLocaleString(undefined, { maximumFractionDigits: 0 })}`;
}

export default function AdminBillingOverviewPage() {
  const session = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [data, setData] = useState<AdminBillingOverview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!session?.token) return;
    let cancelled = false;
    setLoading(true);
    getAdminBillingOverview(session.token)
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
  }, [session?.token]);

  const counts = data?.status_counts ?? [];
  const get = (s: string) =>
    counts.find((c) => c.status === s)?.count ?? 0;

  return (
    <section className="biofield-panel" style={{ padding: "1.6rem 2rem" }}>
      <header style={{ marginBottom: "1.4rem" }}>
        <p className="biofield-eyebrow" style={{ margin: 0 }}>
          Billing · Overview
        </p>
        <h1
          style={{
            margin: "0.3rem 0 0",
            fontSize: "1.4rem",
            fontWeight: 600,
          }}
        >
          Subscription state across the fleet
        </h1>
      </header>

      {loading && <p className="biofield-copy">Loading…</p>}
      {error && (
        <p className="biofield-copy" style={{ color: "var(--accent)" }}>
          Failed to load: {error}
        </p>
      )}

      {data && (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))",
            gap: "0.8rem",
          }}
        >
          <div className="biofield-panel" style={cardStyle}>
            <span style={labelStyle}>Active</span>
            <span style={valueStyle}>{get("active").toLocaleString()}</span>
          </div>
          <div className="biofield-panel" style={cardStyle}>
            <span style={labelStyle}>Past due</span>
            <span style={valueStyle}>{get("past_due").toLocaleString()}</span>
          </div>
          <div className="biofield-panel" style={cardStyle}>
            <span style={labelStyle}>Canceled</span>
            <span style={valueStyle}>{get("canceled").toLocaleString()}</span>
          </div>
          <div className="biofield-panel" style={cardStyle}>
            <span style={labelStyle}>Trialing</span>
            <span style={valueStyle}>{get("trialing").toLocaleString()}</span>
          </div>
          <div className="biofield-panel" style={cardStyle}>
            <span style={labelStyle}>Free users</span>
            <span style={valueStyle}>
              {data.free_users.toLocaleString()}
            </span>
          </div>
          <div className="biofield-panel" style={cardStyle}>
            <span style={labelStyle}>MRR estimate</span>
            <span style={valueStyle}>{formatUsd(data.mrr_usd_estimate)}</span>
          </div>
        </div>
      )}
    </section>
  );
}
