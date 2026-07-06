"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { EventStream, EventStreamItem } from "@/components/event-stream";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getSystemCache,
  getSystemHealth,
  getSystemServices,
  getSystemWorkflows
} from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getNumberParam } from "@/lib/url-query";
import type {
  AdminSystemCacheResponse,
  AdminSystemHealthResponse,
  AdminSystemServiceItem,
  AdminSystemWorkflowItem
} from "@/types/admin";

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

function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds <= 0) {
    return "0m";
  }

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

const REFRESH_OPTIONS = [0, 15, 30, 60] as const;

export default function SystemPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const windowHours = getNumberParam(searchParams, "window_hours", 24, 1, 24 * 30);
  const autoRefreshSec = getNumberParam(searchParams, "refresh", 0, 0, 60);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);

  const [health, setHealth] = useState<AdminSystemHealthResponse | null>(null);
  const [cache, setCache] = useState<AdminSystemCacheResponse | null>(null);
  const [services, setServices] = useState<AdminSystemServiceItem[]>([]);
  const [workflows, setWorkflows] = useState<AdminSystemWorkflowItem[]>([]);

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

  const degradedServiceCount = useMemo(
    () => services.filter((service) => service.status !== "healthy").length,
    [services]
  );

  const loadSystemViews = useCallback(async () => {
    const token = getAuthToken();
    if (!token) {
      throw new Error("Missing session token. Please sign in again.");
    }

    const currentWindowHours = getNumberParam(searchParams, "window_hours", 24, 1, 24 * 30);
    const [healthResponse, servicesResponse, workflowsResponse, cacheResponse] =
      await Promise.all([
        getSystemHealth(token),
        getSystemServices(token, { limit: 50, offset: 0 }),
        getSystemWorkflows(token, { window_hours: currentWindowHours, limit: 100, offset: 0 }),
        getSystemCache(token)
      ]);

    return { healthResponse, servicesResponse, workflowsResponse, cacheResponse };
  }, [searchParams]);

  const handleFetch = useCallback(
    (
      promise: Promise<{
        healthResponse: AdminSystemHealthResponse;
        servicesResponse: { items: AdminSystemServiceItem[] };
        workflowsResponse: { items: AdminSystemWorkflowItem[] };
        cacheResponse: AdminSystemCacheResponse;
      }>
    ) => {
      setLoading(true);
      setError(null);
      promise
        .then(({ healthResponse, servicesResponse, workflowsResponse, cacheResponse }) => {
          setHealth(healthResponse);
          setServices(servicesResponse.items);
          setWorkflows(workflowsResponse.items);
          setCache(cacheResponse);
          setLastUpdatedAt(new Date().toISOString());
        })
        .catch((err) => {
          if (err instanceof ApiClientError) {
            setError(err.payload?.error || err.message);
          } else if (err instanceof Error) {
            setError(err.message);
          } else {
            setError("Failed to load system operations data");
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
      handleFetch(loadSystemViews());
    });
    return () => {
      cancelled = true;
    };
  }, [handleFetch, loadSystemViews]);

  useEffect(() => {
    if (autoRefreshSec <= 0) {
      return;
    }
    const interval = window.setInterval(() => {
      handleFetch(loadSystemViews());
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, handleFetch, loadSystemViews]);

  return (
    <PageShell
      title="System Operations"
      summary="Read-only operational status for API, data plane, orchestrator, and cache layers."
    >
      <div className="panel-inline">
        <label>
          Workflow window
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
        <button type="button" onClick={() => handleFetch(loadSystemViews())}>
          Refresh
        </button>
      </div>

      <p className="helper">Last updated: {formatDateTime(lastUpdatedAt)}</p>

      {error ? <StateBanner variant="error" title={error} /> : null}

      <div className="grid metrics">
        <article className="metric">
          <div className="label">Overall</div>
          <div className="value">
            {health ? (
              <span className={statusPillClass(health.overall_status)}>{health.overall_status}</span>
            ) : (
              "--"
            )}
          </div>
        </article>
        <article className="metric">
          <div className="label">Service Alerts</div>
          <div className="value">{loading ? "--" : degradedServiceCount}</div>
        </article>
        <article className="metric">
          <div className="label">Uptime</div>
          <div className="value">{health ? formatDuration(health.uptime_seconds) : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">Cache Hit Rate</div>
          <div className="value">{cache ? `${cache.hit_rate_pct.toFixed(2)}%` : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">Redis</div>
          <div className="value">
            {cache ? (
              <span className={statusPillClass(cache.redis_available ? "healthy" : "unavailable")}>
                {cache.redis_available ? "available" : "unavailable"}
              </span>
            ) : (
              "--"
            )}
          </div>
        </article>
        <article className="metric">
          <div className="label">L1 Entries</div>
          <div className="value">{cache ? cache.l1_entries : "--"}</div>
        </article>
      </div>

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading system telemetry"
          description="Resolving subsystem health, cache posture, service rows, and workflow runtime signals."
        />
      ) : null}

      <article className="panel">
        <h3>Subsystem Health</h3>
        <div className="table-wrap compact">
          <table>
            <thead>
              <tr>
                <th>Subsystem</th>
                <th>Status</th>
                <th>Latency (ms)</th>
                <th>Detail</th>
              </tr>
            </thead>
            <tbody>
              {health?.subsystems.map((subsystem) => (
                <tr key={subsystem.name}>
                  <td>
                    <div className="table-primary">{subsystem.name}</div>
                  </td>
                  <td>
                    <span className={statusPillClass(subsystem.status)}>{subsystem.status}</span>
                  </td>
                  <td>
                    {subsystem.latency_ms === null ? "--" : subsystem.latency_ms.toFixed(1)}
                  </td>
                  <td className="cell-wrap">{subsystem.detail}</td>
                </tr>
              ))}
              {!health || health.subsystems.length === 0 ? (
                <TableEmptyStateRow
                  colSpan={4}
                  title="No subsystem health data"
                  description="Subsystem checks have not returned any current health snapshots."
                />
              ) : null}
            </tbody>
          </table>
        </div>
      </article>

      <article className="panel">
        <h3>Service Status</h3>
        <div className="table-wrap compact">
          <table>
            <thead>
              <tr>
                <th>Service</th>
                <th>Category</th>
                <th>Status</th>
                <th>Error Rate</th>
                <th>Latency (ms)</th>
                <th>Updated</th>
              </tr>
            </thead>
            <tbody>
              {services.map((service) => (
                <tr key={service.id}>
                  <td>
                    <div className="table-primary">{service.name}</div>
                    <div className="helper">{service.detail}</div>
                  </td>
                  <td>{service.category}</td>
                  <td>
                    <span className={statusPillClass(service.status)}>{service.status}</span>
                  </td>
                  <td>
                    {service.error_rate_pct === null ? "--" : `${service.error_rate_pct.toFixed(2)}%`}
                  </td>
                  <td>{service.latency_ms === null ? "--" : service.latency_ms.toFixed(1)}</td>
                  <td>{formatDateTime(service.updated_at)}</td>
                </tr>
              ))}
              {services.length === 0 ? (
                <TableEmptyStateRow
                  colSpan={6}
                  title="No service rows"
                  description="No service-level telemetry is available for the current system view."
                />
              ) : null}
            </tbody>
          </table>
        </div>
      </article>

      <article className="panel">
        <h3>Workflow Runtime Snapshot</h3>
        {workflows.length === 0 ? (
          <div className="state-table-empty event-stream-empty">
            <div className="telemetry-caption">Empty</div>
            <div className="state-table-empty-title">No workflow runtime data</div>
            <div className="helper">
              No workflow executions were observed in the selected time window.
            </div>
          </div>
        ) : (
          <EventStream label="Workflow runtime stream">
            {workflows.map((workflow) => (
              <EventStreamItem
                key={workflow.workflow_id}
                eyebrow="Workflow"
                title={workflow.name}
                subtitle={workflow.workflow_id}
                badge={<span className={statusPillClass(workflow.status)}>{workflow.status}</span>}
                metadata={[
                  { label: "Recent runs", value: workflow.recent_runs },
                  { label: "Failures", value: workflow.failure_runs },
                  { label: "Engines", value: workflow.engine_count },
                  { label: "Last seen", value: formatDateTime(workflow.last_seen_at) }
                ]}
                summary="Operational narrative for the selected workflow window, including throughput and failure posture."
              />
            ))}
          </EventStream>
        )}
      </article>
    </PageShell>
  );
}
