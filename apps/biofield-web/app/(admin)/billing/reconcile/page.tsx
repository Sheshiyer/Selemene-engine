"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import {
  getStoredAuthSession,
  sessionHasAnyPermission,
  subscribeToAuthSession,
} from "@/lib/auth";
import {
  getAdminReconcileDrift,
  triggerAdminReconcile,
  type AdminReconcileDriftResponse,
} from "@/lib/admin-api";

const labelStyle: React.CSSProperties = {
  fontSize: "0.7rem",
  letterSpacing: "0.12em",
  textTransform: "uppercase",
  color: "var(--muted)",
};

function fmt(iso: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleString();
}

export default function AdminReconcilePage() {
  const session = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [data, setData] = useState<AdminReconcileDriftResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [triggerOpen, setTriggerOpen] = useState(false);
  const [triggerCmd, setTriggerCmd] = useState<string | null>(null);
  const [triggering, setTriggering] = useState(false);

  const canTrigger = sessionHasAnyPermission(session, [
    "admin:billing:reconcile:trigger",
  ]);

  useEffect(() => {
    if (!session?.token) return;
    let cancelled = false;
    setLoading(true);
    getAdminReconcileDrift(session.token)
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

  async function handleTrigger() {
    if (!session?.token) return;
    setTriggering(true);
    try {
      const r = await triggerAdminReconcile(session.token);
      setTriggerCmd(r.command);
    } catch (e) {
      setTriggerCmd(
        e instanceof Error ? `Failed: ${e.message}` : "Trigger failed",
      );
    } finally {
      setTriggering(false);
      setTriggerOpen(true);
    }
  }

  const latest = data?.latest ?? null;
  const drift = latest?.drift_json as
    | { drift?: Record<string, number>; samples?: Record<string, string[]> }
    | undefined;
  const driftCounts = drift?.drift ?? {};

  return (
    <section className="biofield-panel" style={{ padding: "1.6rem 2rem" }}>
      <header
        style={{
          display: "flex",
          alignItems: "baseline",
          justifyContent: "space-between",
          marginBottom: "1rem",
          flexWrap: "wrap",
          gap: "0.6rem",
        }}
      >
        <div>
          <p className="biofield-eyebrow" style={{ margin: 0 }}>
            Billing · Reconcile
          </p>
          <h1
            style={{
              margin: "0.3rem 0 0",
              fontSize: "1.4rem",
              fontWeight: 600,
            }}
          >
            Latest drift report
          </h1>
        </div>
        {canTrigger && (
          <button
            type="button"
            className="biofield-link"
            onClick={handleTrigger}
            disabled={triggering}
            style={{
              fontSize: "0.84rem",
              padding: "0.55rem 1.1rem",
            }}
          >
            {triggering ? "Resolving…" : "Trigger reconcile"}
          </button>
        )}
      </header>

      {loading && <p className="biofield-copy">Loading…</p>}
      {error && (
        <p className="biofield-copy" style={{ color: "var(--accent)" }}>
          {error}
        </p>
      )}

      {data && !latest && (
        <p className="biofield-copy" style={{ fontSize: "0.86rem" }}>
          No reconcile run recorded yet. Either the cron has not fired on this
          environment, or migration <code>023_reconcile_runs</code> has not
          been applied. Run the reconcile bin and refresh.
        </p>
      )}

      {latest && (
        <>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))",
              gap: "0.8rem",
            }}
          >
            <div className="biofield-panel" style={{ padding: "1rem 1.2rem" }}>
              <span style={labelStyle}>Started</span>
              <p style={{ margin: "0.4rem 0 0", fontSize: "0.86rem" }}>
                {fmt(latest.started_at)}
              </p>
            </div>
            <div className="biofield-panel" style={{ padding: "1rem 1.2rem" }}>
              <span style={labelStyle}>Finished</span>
              <p style={{ margin: "0.4rem 0 0", fontSize: "0.86rem" }}>
                {fmt(latest.finished_at)}
              </p>
            </div>
            <div className="biofield-panel" style={{ padding: "1rem 1.2rem" }}>
              <span style={labelStyle}>Force cancel</span>
              <p style={{ margin: "0.4rem 0 0", fontSize: "0.86rem" }}>
                {latest.force_cancel ? "ENABLED" : "read-only"}
              </p>
            </div>
            {Object.entries(driftCounts).map(([k, v]) => (
              <div
                key={k}
                className="biofield-panel"
                style={{ padding: "1rem 1.2rem" }}
              >
                <span style={labelStyle}>{k}</span>
                <p
                  style={{
                    margin: "0.4rem 0 0",
                    fontSize: "1.4rem",
                    fontWeight: 600,
                  }}
                >
                  {String(v)}
                </p>
              </div>
            ))}
          </div>

          {latest.error && (
            <p
              className="biofield-copy"
              style={{
                marginTop: "1rem",
                color: "var(--accent)",
                fontSize: "0.84rem",
              }}
            >
              Run error: {latest.error}
            </p>
          )}

          <details
            style={{
              marginTop: "1rem",
              fontSize: "0.78rem",
              color: "var(--muted)",
            }}
          >
            <summary
              style={{
                cursor: "pointer",
                padding: "0.4rem 0",
              }}
            >
              Raw report JSON
            </summary>
            <pre
              style={{
                background: "rgba(var(--signal-rgb), 0.06)",
                padding: "0.8rem",
                borderRadius: "var(--r-pill)",
                overflowX: "auto",
                fontSize: "0.74rem",
              }}
            >
              {JSON.stringify(latest.drift_json, null, 2)}
            </pre>
          </details>
        </>
      )}

      {triggerOpen && triggerCmd && (
        <div
          className="biofield-panel"
          style={{
            marginTop: "1rem",
            padding: "1rem 1.2rem",
            border: "1px solid rgba(var(--accent-rgb), 0.4)",
          }}
        >
          <p
            className="biofield-copy"
            style={{ margin: "0 0 0.6rem", fontSize: "0.82rem" }}
          >
            Reconcile is a separate binary. Run this command on a host with
            DB + Dodo access:
          </p>
          <pre
            style={{
              background: "rgba(var(--signal-rgb), 0.08)",
              padding: "0.7rem",
              borderRadius: "var(--r-pill)",
              fontSize: "0.78rem",
              overflowX: "auto",
            }}
          >
            {triggerCmd}
          </pre>
          <button
            type="button"
            className="biofield-link"
            onClick={() => setTriggerOpen(false)}
            style={{
              marginTop: "0.5rem",
              fontSize: "0.78rem",
              padding: "0.4rem 1rem",
            }}
          >
            Dismiss
          </button>
        </div>
      )}
    </section>
  );
}
