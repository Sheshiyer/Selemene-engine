"use client";

import { useEffect, useState } from "react";
import { MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminBillingOverview } from "@/lib/api";
import type { AdminBillingOverviewResponse } from "@/types/admin";

function formatUsd(value: number): string {
  if (!Number.isFinite(value) || value <= 0) return "—";
  return `$${value.toLocaleString(undefined, { maximumFractionDigits: 0 })}`;
}

export default function AdminBillingOverviewPage() {
  const [data, setData] = useState<AdminBillingOverviewResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = getAuthToken();
    if (!token) return;
    let cancelled = false;
    getAdminBillingOverview(token)
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
              : "Failed to load billing overview"
        );
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const counts = data?.status_counts ?? [];
  const get = (status: string) =>
    counts.find((c) => c.status === status)?.count ?? 0;

  return (
    <PageShell
      title="Billing & Subscriptions"
      summary="Subscription state, webhook ingest, plan catalog, and reconcile drift across the Dodo Payments integration."
    >
      {error ? (
        <StateBanner
          variant="error"
          title="Unable to load billing overview"
          description={error}
        />
      ) : null}

      <SurfaceCard
        eyebrow="Subscription state"
        title="Fleet posture"
        summary="Counts grouped by current subscription status across all Dodo Payments customers."
      >
        {loading && !data ? (
          <p className="helper">Loading…</p>
        ) : data ? (
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))",
              gap: "0.75rem"
            }}
          >
            <MetricSurface label="Active" value={get("active").toLocaleString()} />
            <MetricSurface
              label="Past due"
              value={get("past_due").toLocaleString()}
            />
            <MetricSurface
              label="Canceled"
              value={get("canceled").toLocaleString()}
            />
            <MetricSurface
              label="Trialing"
              value={get("trialing").toLocaleString()}
            />
            <MetricSurface
              label="Free users"
              value={data.free_users.toLocaleString()}
              detail="users on tier=free"
            />
            <MetricSurface
              label="MRR estimate"
              value={formatUsd(data.mrr_usd_estimate)}
              detail="active × avg plan price"
            />
          </div>
        ) : null}
      </SurfaceCard>
    </PageShell>
  );
}
