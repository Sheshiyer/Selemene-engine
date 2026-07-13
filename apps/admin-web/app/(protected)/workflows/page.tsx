"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getSystemWorkflows } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getNumberParam } from "@/lib/url-query";
import type { AdminSystemWorkflowItem } from "@/types/admin";

const REFRESH_OPTIONS = [0, 15, 30, 60] as const;

function formatDateTime(value: string | null): string {
  if (!value) {
    return "--";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "--";
  }
  return date.toLocaleString();
}

export default function WorkflowsPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const windowHours = getNumberParam(searchParams, "window_hours", 24, 1, 24 * 30);
  const autoRefreshSec = getNumberParam(searchParams, "refresh", 0, 0, 60);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [workflows, setWorkflows] = useState<AdminSystemWorkflowItem[]>([]);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);

  const updateQuery = useCallback(
    (updates: { window_hours?: number; refresh?: number }) => {
      const nextQuery = buildQueryString(searchParams, {
        window_hours: updates.window_hours,
        refresh: updates.refresh && updates.refresh > 0 ? updates.refresh : undefined
      });
      router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
    },
    [pathname, router, searchParams]
  );

  const loadWorkflows = useCallback(async () => {
    const token = getAuthToken() ?? undefined;

    const currentWindowHours = getNumberParam(searchParams, "window_hours", 24, 1, 24 * 30);
    const response = await getSystemWorkflows(token, {
      window_hours: currentWindowHours,
      limit: 100,
      offset: 0
    });
    return response;
  }, [searchParams]);

  const handleFetch = useCallback(
    (promise: Promise<{ items: AdminSystemWorkflowItem[] }>) => {
      setLoading(true);
      setError(null);
      promise
        .then((response) => {
          setWorkflows(response.items);
          setLastUpdatedAt(new Date().toISOString());
        })
        .catch((err) => {
          if (err instanceof ApiClientError) {
            setError(err.payload?.error || err.message);
          } else if (err instanceof Error) {
            setError(err.message);
          } else {
            setError("Failed to load workflow data");
          }
        })
        .finally(() => {
          setLoading(false);
        });
    },
    []
  );

  useEffect(() => {
    let cancelled = false;
    queueMicrotask(() => {
      if (cancelled) return;
      handleFetch(loadWorkflows());
    });
    return () => {
      cancelled = true;
    };
  }, [handleFetch, loadWorkflows]);

  useEffect(() => {
    if (autoRefreshSec <= 0) {
      return;
    }
    const interval = window.setInterval(() => {
      handleFetch(loadWorkflows());
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, handleFetch, loadWorkflows]);

  const degradedCount = useMemo(
    () => workflows.filter((w) => w.status === "degraded").length,
    [workflows]
  );

  const totalRuns = useMemo(
    () => workflows.reduce((sum, w) => sum + w.recent_runs, 0),
    [workflows]
  );

  const totalFailures = useMemo(
    () => workflows.reduce((sum, w) => sum + w.failure_runs, 0),
    [workflows]
  );

  return (
    <PageShell
      title="Workflow Registry"
      summary="View synthesis types, engine composition, and execution metrics for all 6 canonical workflows."
    >
      <div className="panel-inline">
        <label>
          Window
          <select
            value={windowHours}
            onChange={(event) =>
              updateQuery({ window_hours: Number.parseInt(event.target.value, 10) })
            }
          >
            <option value={24}>Last 24 hours</option>
            <option value={72}>Last 72 hours</option>
            <option value={168}>Last 7 days</option>
          </select>
        </label>
        <label>
          Auto refresh
          <select
            value={autoRefreshSec}
            onChange={(event) =>
              updateQuery({ refresh: Number.parseInt(event.target.value, 10) })
            }
          >
            {REFRESH_OPTIONS.map((value) => (
              <option key={value} value={value}>
                {value === 0 ? "Off" : `${value}s`}
              </option>
            ))}
          </select>
        </label>
        <button type="button" onClick={() => handleFetch(loadWorkflows())}>
          Refresh
        </button>
      </div>

      <p className="helper">Last updated: {formatDateTime(lastUpdatedAt)}</p>

      {error ? <StateBanner variant="error" title={error} /> : null}

      <div className="grid metrics">
        <article className="metric">
          <div className="label">Total workflows</div>
          <div className="value">{workflows.length}</div>
        </article>
        <article className="metric">
          <div className="label">Degraded</div>
          <div className="value">{loading ? "--" : degradedCount}</div>
        </article>
        <article className="metric">
          <div className="label">Total runs</div>
          <div className="value">{loading ? "--" : totalRuns}</div>
        </article>
        <article className="metric">
          <div className="label">Total failures</div>
          <div className="value">{loading ? "--" : totalFailures}</div>
        </article>
      </div>

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading workflow registry"
          description="Resolving synthesis types, engine composition, and execution metrics."
        />
      ) : (
        <div className="grid workflow-card-grid">
          {workflows.map((workflow) => (
            <SurfaceCard
              key={workflow.workflow_id}
              eyebrow="Workflow"
              title={workflow.name}
              summary={`${workflow.engine_count} engine${workflow.engine_count === 1 ? "" : "s"} · ${workflow.recent_runs} recent runs · ${workflow.failure_runs} failures`}
            >
              <div className="grid overlay-detail-grid">
                <div className="helper">
                  Status: <span className={statusPillClass(workflow.status)}>{workflow.status}</span>
                </div>
                <div className="helper">ID: {workflow.workflow_id}</div>
                {workflow.synthesis_type ? (
                  <div className="helper">
                    Synthesis: <span className="permission-chip">{workflow.synthesis_type}</span>
                  </div>
                ) : null}
                <div className="helper">Last seen: {formatDateTime(workflow.last_seen_at)}</div>
                {workflow.required_phase !== undefined ? (
                  <div className="helper">Required phase: {workflow.required_phase}</div>
                ) : null}
                {workflow.cache_hits !== undefined ? (
                  <div className="helper">Cache hits: {workflow.cache_hits}</div>
                ) : null}
                {workflow.cache_entries !== undefined ? (
                  <div className="helper">Cache entries: {workflow.cache_entries}</div>
                ) : null}
                {workflow.engine_ids && workflow.engine_ids.length > 0 ? (
                  <div className="helper">
                    <div className="table-chip-row">
                      {workflow.engine_ids.map((engineId) => (
                        <span key={engineId} className="permission-chip">
                          {engineId}
                        </span>
                      ))}
                    </div>
                  </div>
                ) : null}
              </div>
              <button
                type="button"
                className="link-btn"
                onClick={() => router.push(`/workflows/${workflow.workflow_id}`)}
              >
                View detail &rarr;
              </button>
            </SurfaceCard>
          ))}
          {workflows.length === 0 ? (
            <SurfaceCard
              eyebrow="Empty"
              title="No workflow data"
              summary="No workflow executions were observed in the selected time window."
            >
              <div className="ornament-rule" />
            </SurfaceCard>
          ) : null}
        </div>
      )}
    </PageShell>
  );
}