"use client";

import { useEffect, useState } from "react";
import { SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, TableEmptyStateRow } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminBillingWebhookEvents } from "@/lib/api";
import type { AdminBillingWebhookEventsResponse } from "@/types/admin";

function formatDateTime(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export default function AdminBillingWebhookEventsPage() {
  const [data, setData] = useState<AdminBillingWebhookEventsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const token = getAuthToken();
    if (!token) return;
    let cancelled = false;
    setLoading(true);
    getAdminBillingWebhookEvents(token, { limit: 100 })
      .then((resp) => {
        if (!cancelled) {
          setData(resp);
          setError(null);
        }
      })
      .catch((err) => {
        if (cancelled) return;
        setError(
          err instanceof ApiClientError
            ? err.message
            : err instanceof Error
              ? err.message
              : "Failed to load webhook events"
        );
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const items = data?.items ?? [];

  return (
    <PageShell
      title="Webhook events"
      summary="Last 100 webhooks acknowledged via the idempotency table. Missing events likely failed signature verification or freshness checks — see Sentry."
    >
      {error ? (
        <StateBanner
          variant="error"
          title="Unable to load webhook events"
          description={error}
        />
      ) : null}

      <SurfaceCard
        eyebrow="Billing"
        title="Recent processed webhooks"
        summary="One row per inbound event we acknowledged via the idempotency table. If you expect an event that's missing here, check Sentry for signature verification failures or freshness rejections."
      >
        <div style={{ overflowX: "auto" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th>Webhook ID</th>
                <th>Provider</th>
                <th>Event type</th>
                <th>Processed at</th>
              </tr>
            </thead>
            <tbody>
              {loading && items.length === 0 ? (
                <TableEmptyStateRow colSpan={4} description="Loading events…" />
              ) : items.length === 0 ? (
                <TableEmptyStateRow colSpan={4} description="No events recorded." />
              ) : (
                items.map((ev) => (
                  <tr key={ev.webhook_id}>
                    <td style={{ fontFamily: "monospace", fontSize: "0.78rem" }}>
                      {ev.webhook_id}
                    </td>
                    <td>{ev.provider}</td>
                    <td>{ev.event_type}</td>
                    <td>{formatDateTime(ev.processed_at)}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </SurfaceCard>
    </PageShell>
  );
}
