"use client";

import { use, useEffect, useState, useSyncExternalStore } from "react";
import Link from "next/link";
import {
  getStoredAuthSession,
  sessionHasAnyPermission,
  subscribeToAuthSession,
} from "@/lib/auth";
import {
  cancelAdminSubscription,
  getAdminSubscription,
  type AdminSubscriptionItem,
} from "@/lib/admin-api";

const labelStyle: React.CSSProperties = {
  fontSize: "0.7rem",
  letterSpacing: "0.12em",
  textTransform: "uppercase",
  color: "var(--muted)",
};

const valueStyle: React.CSSProperties = {
  fontSize: "0.92rem",
  fontFamily: "monospace",
  wordBreak: "break-all",
};

function fmt(iso: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString();
}

export default function AdminSubscriptionDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = use(params);
  const session = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [sub, setSub] = useState<AdminSubscriptionItem | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [confirmInput, setConfirmInput] = useState("");
  const [actionResult, setActionResult] = useState<string | null>(null);

  const canCancel = sessionHasAnyPermission(session, [
    "admin:billing:subscriptions:cancel",
  ]);

  useEffect(() => {
    if (!session?.token) return;
    let cancelled = false;
    setLoading(true);
    getAdminSubscription(session.token, id)
      .then((d) => {
        if (!cancelled) {
          setSub(d.subscription);
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
  }, [session?.token, id]);

  async function handleCancel() {
    if (!session?.token) return;
    setSubmitting(true);
    try {
      await cancelAdminSubscription(session.token, id);
      setActionResult("Subscription canceled locally. Verify Dodo dashboard.");
      // refetch
      const d = await getAdminSubscription(session.token, id);
      setSub(d.subscription);
    } catch (e) {
      setActionResult(
        e instanceof Error ? `Cancel failed: ${e.message}` : "Cancel failed",
      );
    } finally {
      setSubmitting(false);
      setConfirmOpen(false);
      setConfirmInput("");
    }
  }

  return (
    <section className="biofield-panel" style={{ padding: "1.6rem 2rem" }}>
      <header style={{ marginBottom: "1rem" }}>
        <Link
          href="/billing/subscriptions"
          className="biofield-link"
          style={{ padding: 0, fontSize: "0.78rem" }}
        >
          ← Back to subscriptions
        </Link>
        <h1
          style={{
            margin: "0.6rem 0 0",
            fontSize: "1.3rem",
            fontWeight: 600,
          }}
        >
          Subscription detail
        </h1>
      </header>

      {loading && <p className="biofield-copy">Loading…</p>}
      {error && (
        <p className="biofield-copy" style={{ color: "var(--accent)" }}>
          {error}
        </p>
      )}

      {sub && (
        <>
          <dl
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))",
              gap: "1rem",
              margin: 0,
            }}
          >
            <div>
              <dt style={labelStyle}>ID</dt>
              <dd style={valueStyle}>{sub.id}</dd>
            </div>
            <div>
              <dt style={labelStyle}>User ID</dt>
              <dd style={valueStyle}>{sub.user_id}</dd>
            </div>
            <div>
              <dt style={labelStyle}>Status</dt>
              <dd style={valueStyle}>{sub.status}</dd>
            </div>
            <div>
              <dt style={labelStyle}>Provider sub ID</dt>
              <dd style={valueStyle}>{sub.provider_subscription_id ?? "—"}</dd>
            </div>
            <div>
              <dt style={labelStyle}>Provider customer ID</dt>
              <dd style={valueStyle}>{sub.provider_customer_id ?? "—"}</dd>
            </div>
            <div>
              <dt style={labelStyle}>Cancel at period end</dt>
              <dd style={valueStyle}>
                {sub.cancel_at_period_end ? "yes" : "no"}
              </dd>
            </div>
            <div>
              <dt style={labelStyle}>Period start</dt>
              <dd style={valueStyle}>{fmt(sub.current_period_start)}</dd>
            </div>
            <div>
              <dt style={labelStyle}>Period end</dt>
              <dd style={valueStyle}>{fmt(sub.current_period_end)}</dd>
            </div>
            <div>
              <dt style={labelStyle}>Canceled at</dt>
              <dd style={valueStyle}>{fmt(sub.canceled_at)}</dd>
            </div>
            <div>
              <dt style={labelStyle}>Created</dt>
              <dd style={valueStyle}>{fmt(sub.created_at)}</dd>
            </div>
            <div>
              <dt style={labelStyle}>Updated</dt>
              <dd style={valueStyle}>{fmt(sub.updated_at)}</dd>
            </div>
          </dl>

          <hr
            style={{
              margin: "1.4rem 0",
              border: 0,
              borderTop: "1px solid rgba(var(--signal-rgb), 0.18)",
            }}
          />

          <h2 style={{ fontSize: "0.95rem", margin: "0 0 0.6rem" }}>
            Actions
          </h2>

          {actionResult && (
            <p
              className="biofield-copy"
              style={{
                background: "rgba(var(--signal-rgb), 0.06)",
                padding: "0.6rem 0.9rem",
                borderRadius: "var(--r-pill)",
                fontSize: "0.82rem",
              }}
            >
              {actionResult}
            </p>
          )}

          {!canCancel && (
            <p
              className="biofield-copy"
              style={{ fontSize: "0.82rem", color: "var(--muted)" }}
            >
              You do not have <code>admin:billing:subscriptions:cancel</code>{" "}
              permission. Cancel disabled.
            </p>
          )}

          {canCancel && !confirmOpen && (
            <button
              type="button"
              className="biofield-link"
              onClick={() => setConfirmOpen(true)}
              disabled={
                sub.status === "canceled" || !sub.provider_subscription_id
              }
              style={{
                fontSize: "0.84rem",
                padding: "0.55rem 1.2rem",
                color: "var(--accent)",
                opacity:
                  sub.status === "canceled" || !sub.provider_subscription_id
                    ? 0.4
                    : 1,
              }}
            >
              Force-cancel locally
            </button>
          )}

          {canCancel && confirmOpen && (
            <div
              className="biofield-panel"
              style={{
                marginTop: "0.8rem",
                padding: "1rem 1.2rem",
                border: "1px solid rgba(var(--accent-rgb), 0.4)",
              }}
            >
              <p
                className="biofield-copy"
                style={{ fontSize: "0.84rem", margin: "0 0 0.6rem" }}
              >
                This sets <code>status=canceled</code> in our DB and drops the
                user&apos;s tier to <code>free</code>. It does NOT cancel in
                Dodo — do that separately in the Dodo dashboard. Type{" "}
                <strong>{sub.id.slice(0, 8)}</strong> to confirm.
              </p>
              <input
                type="text"
                value={confirmInput}
                onChange={(e) => setConfirmInput(e.target.value)}
                placeholder={sub.id.slice(0, 8)}
                style={{
                  width: "100%",
                  padding: "0.5rem 0.7rem",
                  fontSize: "0.84rem",
                  fontFamily: "monospace",
                  background: "transparent",
                  border: "1px solid rgba(var(--signal-rgb), 0.28)",
                  borderRadius: "var(--r-pill)",
                  color: "inherit",
                }}
              />
              <div
                style={{
                  marginTop: "0.6rem",
                  display: "flex",
                  gap: "0.4rem",
                }}
              >
                <button
                  type="button"
                  className="biofield-link"
                  onClick={() => {
                    setConfirmOpen(false);
                    setConfirmInput("");
                  }}
                  style={{ fontSize: "0.82rem", padding: "0.5rem 1rem" }}
                >
                  Cancel
                </button>
                <button
                  type="button"
                  className="biofield-link"
                  disabled={
                    submitting || confirmInput !== sub.id.slice(0, 8)
                  }
                  onClick={handleCancel}
                  style={{
                    fontSize: "0.82rem",
                    padding: "0.5rem 1rem",
                    color: "var(--accent)",
                    opacity:
                      submitting || confirmInput !== sub.id.slice(0, 8)
                        ? 0.4
                        : 1,
                  }}
                >
                  {submitting ? "Canceling…" : "Confirm cancel"}
                </button>
              </div>
            </div>
          )}
        </>
      )}
    </section>
  );
}
