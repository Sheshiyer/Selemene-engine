"use client";

import { useEffect, useState } from "react";
import { ActionRail, MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getAdminBillingReconcileDrift,
  getAdminSession,
  triggerAdminBillingReconcile
} from "@/lib/api";
import { hasPermission } from "@/lib/permissions";
import type {
  AdminBillingReconcileDriftResponse,
  AdminSession
} from "@/types/admin";

function formatDateTime(value: string | null): string {
  if (!value) return "--";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export default function AdminBillingReconcilePage() {
  const [data, setData] = useState<AdminBillingReconcileDriftResponse | null>(null);
  const [session, setSession] = useState<AdminSession | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [triggerCmd, setTriggerCmd] = useState<string | null>(null);
  const [triggering, setTriggering] = useState(false);

  useEffect(() => {
    const token = getAuthToken();
    if (!token) return;
    let cancelled = false;
    setLoading(true);
    Promise.all([
      getAdminBillingReconcileDrift(token),
      getAdminSession(token)
    ])
      .then(([drift, sess]) => {
        if (!cancelled) {
          setData(drift);
          setSession(sess);
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
              : "Failed to load reconcile drift"
        );
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const canTrigger =
    !!session?.permissions &&
    hasPermission(session.permissions, "admin:billing:reconcile:trigger");

  async function handleTrigger() {
    const token = getAuthToken();
    if (!token) return;
    setTriggering(true);
    try {
      const resp = await triggerAdminBillingReconcile(token);
      setTriggerCmd(resp.command);
    } catch (err) {
      setTriggerCmd(
        err instanceof Error ? `Failed: ${err.message}` : "Trigger failed"
      );
    } finally {
      setTriggering(false);
    }
  }

  const latest = data?.latest ?? null;
  const drift = latest?.drift_json as
    | { drift?: Record<string, number>; samples?: Record<string, string[]> }
    | undefined;
  const driftCounts = drift?.drift ?? {};

  return (
    <PageShell
      title="Reconcile"
      summary="The dodo_reconcile binary writes one row per execution. Drift counts compare local subscriptions vs. Dodo's active subscription set."
    >
      {error ? (
        <StateBanner
          variant="error"
          title="Unable to load reconcile drift"
          description={error}
        />
      ) : null}

      <SurfaceCard
        eyebrow="Billing"
        title="Latest reconcile run"
        summary="The dodo_reconcile binary writes one row per execution. Drift counts compare local subscriptions vs. Dodo's active subscription set."
        actions={
          canTrigger ? (
            <button
              type="button"
              className="button"
              onClick={handleTrigger}
              disabled={triggering}
            >
              {triggering ? "Resolving…" : "Trigger reconcile"}
            </button>
          ) : null
        }
      >
        {loading && !data ? (
          <p className="helper">Loading…</p>
        ) : !latest ? (
          <p className="helper">
            No reconcile run recorded yet. Either the cron has not fired on
            this environment, or migration <code>023_reconcile_runs</code> has
            not been applied. Run the bin and refresh.
          </p>
        ) : (
          <>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))",
                gap: "0.75rem"
              }}
            >
              <MetricSurface
                label="Started"
                value={formatDateTime(latest.started_at)}
              />
              <MetricSurface
                label="Finished"
                value={formatDateTime(latest.finished_at)}
              />
              <MetricSurface
                label="Force cancel"
                value={latest.force_cancel ? "ENABLED" : "read-only"}
              />
              {Object.entries(driftCounts).map(([k, v]) => (
                <MetricSurface key={k} label={k} value={String(v)} />
              ))}
            </div>

            {latest.error ? (
              <StateBanner
                variant="error"
                title="Run error"
                description={latest.error}
              />
            ) : null}

            <details className="helper" style={{ marginTop: "1rem" }}>
              <summary style={{ cursor: "pointer" }}>Raw report JSON</summary>
              <pre
                style={{
                  background: "rgba(0,0,0,0.04)",
                  padding: "0.75rem",
                  borderRadius: "8px",
                  overflowX: "auto",
                  fontSize: "0.75rem"
                }}
              >
                {JSON.stringify(latest.drift_json, null, 2)}
              </pre>
            </details>
          </>
        )}
      </SurfaceCard>

      {triggerCmd ? (
        <SurfaceCard
          eyebrow="Reconcile"
          title="Run this command"
          summary="Reconcile is a separate binary owned by cron infrastructure. Paste this on a host with DB + Dodo access:"
          actions={
            <button
              type="button"
              className="button button-secondary"
              onClick={() => setTriggerCmd(null)}
            >
              Dismiss
            </button>
          }
        >
          <pre
            style={{
              background: "rgba(0,0,0,0.04)",
              padding: "0.75rem",
              borderRadius: "8px",
              overflowX: "auto",
              fontSize: "0.82rem"
            }}
          >
            {triggerCmd}
          </pre>
        </SurfaceCard>
      ) : null}
    </PageShell>
  );
}
