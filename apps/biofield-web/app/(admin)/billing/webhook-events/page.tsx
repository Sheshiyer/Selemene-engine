"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import { getStoredAuthSession, subscribeToAuthSession } from "@/lib/auth";
import {
  listAdminWebhookEvents,
  type AdminWebhookEventsResponse,
} from "@/lib/admin-api";

export default function AdminWebhookEventsPage() {
  const session = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [data, setData] = useState<AdminWebhookEventsResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!session?.token) return;
    let cancelled = false;
    setLoading(true);
    listAdminWebhookEvents(session.token, { limit: 100 })
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

  return (
    <section className="biofield-panel" style={{ padding: "1.6rem 2rem" }}>
      <header style={{ marginBottom: "1rem" }}>
        <p className="biofield-eyebrow" style={{ margin: 0 }}>
          Billing · Webhook events
        </p>
        <h1
          style={{
            margin: "0.3rem 0 0",
            fontSize: "1.4rem",
            fontWeight: 600,
          }}
        >
          Recent processed webhooks
        </h1>
        <p
          className="biofield-copy"
          style={{
            fontSize: "0.78rem",
            color: "var(--muted)",
            marginTop: "0.4rem",
          }}
        >
          Each row is one event we acknowledged via the idempotency table. If
          you expect an event you don&apos;t see, check Sentry for{" "}
          <code>signature verification failed</code> or freshness rejections.
        </p>
      </header>

      {loading && <p className="biofield-copy">Loading…</p>}
      {error && (
        <p className="biofield-copy" style={{ color: "var(--accent)" }}>
          {error}
        </p>
      )}

      {data && (
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
                <th style={{ padding: "0.5rem 0.6rem" }}>Webhook ID</th>
                <th style={{ padding: "0.5rem 0.6rem" }}>Provider</th>
                <th style={{ padding: "0.5rem 0.6rem" }}>Event type</th>
                <th style={{ padding: "0.5rem 0.6rem" }}>Processed at</th>
              </tr>
            </thead>
            <tbody>
              {data.items.map((ev) => (
                <tr
                  key={ev.webhook_id}
                  style={{
                    borderTop: "1px solid rgba(var(--signal-rgb), 0.12)",
                  }}
                >
                  <td
                    style={{
                      padding: "0.5rem 0.6rem",
                      fontFamily: "monospace",
                      fontSize: "0.76rem",
                    }}
                  >
                    {ev.webhook_id}
                  </td>
                  <td style={{ padding: "0.5rem 0.6rem" }}>{ev.provider}</td>
                  <td style={{ padding: "0.5rem 0.6rem" }}>{ev.event_type}</td>
                  <td style={{ padding: "0.5rem 0.6rem" }}>
                    {new Date(ev.processed_at).toLocaleString()}
                  </td>
                </tr>
              ))}
              {data.items.length === 0 && (
                <tr>
                  <td
                    colSpan={4}
                    style={{
                      padding: "1rem",
                      textAlign: "center",
                      color: "var(--muted)",
                    }}
                  >
                    No events recorded.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
