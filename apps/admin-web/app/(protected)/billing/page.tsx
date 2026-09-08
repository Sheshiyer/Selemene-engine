"use client";

import { useEffect, useState } from "react";
import { MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getAdminBillingControl,
  getAdminBillingOverview,
  updateAdminBillingControl
} from "@/lib/api";
import type { AdminBillingControlResponse, AdminBillingOverviewResponse } from "@/types/admin";

function formatUsd(value: number): string {
  if (!Number.isFinite(value) || value <= 0) return "—";
  return `$${value.toLocaleString(undefined, { maximumFractionDigits: 0 })}`;
}

export default function AdminBillingOverviewPage() {
  const [data, setData] = useState<AdminBillingOverviewResponse | null>(null);
  const [control, setControl] = useState<AdminBillingControlResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [controlError, setControlError] = useState<string | null>(null);
  const [updatingControl, setUpdatingControl] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = getAuthToken() ?? undefined;
let cancelled = false;
    Promise.all([getAdminBillingOverview(token), getAdminBillingControl(token)])
      .then(([overview, billingControl]) => {
        if (!cancelled) {
          setData(overview);
          setControl(billingControl);
          setError(null);
          setControlError(null);
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

  async function setBillingMode(mode: "free" | "disabled") {
    setUpdatingControl(true);
    setControlError(null);
    try {
      const next = await updateAdminBillingControl(getAuthToken() ?? undefined, mode);
      setControl(next);
    } catch (err) {
      setControlError(
        err instanceof ApiClientError
          ? err.message
          : err instanceof Error
            ? err.message
            : "Failed to update billing control"
      );
    } finally {
      setUpdatingControl(false);
    }
  }

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
        eyebrow="Payment control"
        title="Release and admin billing mode"
        summary="Free access is safe when Dodo is unavailable. Enabling Dodo remains a release-level decision and is never available from this toggle."
      >
        {control ? (
          <div style={{ display: "grid", gap: "0.75rem" }}>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))",
                gap: "0.75rem"
              }}
            >
              <MetricSurface label="Effective mode" value={control.effective_mode} />
              <MetricSurface label="Release mode" value={control.release_mode} />
              <MetricSurface
                label="Admin override"
                value={control.override_mode ?? "none"}
              />
              <MetricSurface
                label="Dodo config"
                value={control.dodo_credentials_present ? "present" : "not configured"}
                detail="credentials only; external access still unverified"
              />
            </div>
            <p className="helper">{control.message}</p>
            {controlError ? (
              <StateBanner variant="error" title="Billing control update failed" description={controlError} />
            ) : null}
            <div style={{ display: "flex", gap: "0.75rem", flexWrap: "wrap" }}>
              <button
                type="button"
                onClick={() => void setBillingMode("free")}
                disabled={updatingControl || control.release_mode === "disabled"}
              >
                Enable free access
              </button>
              <button
                type="button"
                onClick={() => void setBillingMode("disabled")}
                disabled={updatingControl}
              >
                Disable billing
              </button>
            </div>
          </div>
        ) : loading ? (
          <p className="helper">Loading payment control…</p>
        ) : (
          <p className="helper">Payment control status is unavailable.</p>
        )}
      </SurfaceCard>

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
