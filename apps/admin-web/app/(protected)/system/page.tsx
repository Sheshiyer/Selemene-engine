"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
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

  const [windowHours, setWindowHours] = useState(() =>
    getNumberParam(searchParams, "window_hours", 24, 1, 24 * 30)
  );
  const [autoRefreshSec, setAutoRefreshSec] = useState(() =>
    getNumberParam(searchParams, "refresh", 0, 0, 60)
  );

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);

  const [health, setHealth] = useState<AdminSystemHealthResponse | null>(null);
  const [cache, setCache] = useState<AdminSystemCacheResponse | null>(null);
  const [services, setServices] = useState<AdminSystemServiceItem[]>([]);
  const [workflows, setWorkflows] = useState<AdminSystemWorkflowItem[]>([]);

  useEffect(() => {
    setWindowHours(getNumberParam(searchParams, "window_hours", 24, 1, 24 * 30));
    setAutoRefreshSec(getNumberParam(searchParams, "refresh", 0, 0, 60));
  }, [searchParams]);

  useEffect(() => {
    const nextQuery = buildQueryString(searchParams, {
      window_hours: windowHours,
      refresh: autoRefreshSec > 0 ? autoRefreshSec : undefined
    });
    router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
  }, [autoRefreshSec, pathname, router, searchParams, windowHours]);

  const degradedServiceCount = useMemo(
    () => services.filter((service) => service.status !== "healthy").length,
    [services]
  );

  const loadSystemViews = useCallback(async () => {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const [healthResponse, servicesResponse, workflowsResponse, cacheResponse] =
        await Promise.all([
          getSystemHealth(token),
          getSystemServices(token, { limit: 50, offset: 0 }),
          getSystemWorkflows(token, { window_hours: windowHours, limit: 100, offset: 0 }),
          getSystemCache(token)
        ]);

      setHealth(healthResponse);
      setServices(servicesResponse.items);
      setWorkflows(workflowsResponse.items);
      setCache(cacheResponse);
      setLastUpdatedAt(new Date().toISOString());
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to load system operations data");
      }
    } finally {
      setLoading(false);
    }
  }, [windowHours]);

  useEffect(() => {
    void loadSystemViews();
  }, [loadSystemViews]);

  useEffect(() => {
    if (autoRefreshSec <= 0) {
      return;
    }
    const interval = window.setInterval(() => {
      void loadSystemViews();
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, loadSystemViews]);

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
            onChange={(event) => setWindowHours(Number.parseInt(event.target.value, 10))}
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
            onChange={(event) => setAutoRefreshSec(Number.parseInt(event.target.value, 10))}
          >
            {REFRESH_OPTIONS.map((value) => (
              <option key={value} value={value}>
                {value === 0 ? "Off" : `${value}s`}
              </option>
            ))}
          </select>
        </label>
        <button type="button" onClick={() => void loadSystemViews()}>
          Refresh
        </button>
      </div>

      <p className="helper">Last updated: {formatDateTime(lastUpdatedAt)}</p>

      {error ? <div className="error">{error}</div> : null}

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

      {loading ? <p className="helper">Loading system telemetry...</p> : null}

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
                <tr>
                  <td colSpan={4}>
                    <p className="helper">No subsystem health data available.</p>
                  </td>
                </tr>
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
                <tr>
                  <td colSpan={6}>
                    <p className="helper">No service rows available.</p>
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
      </article>

      <article className="panel">
        <h3>Workflow Runtime Snapshot</h3>
        <div className="table-wrap compact">
          <table>
            <thead>
              <tr>
                <th>Workflow</th>
                <th>Status</th>
                <th>Recent Runs</th>
                <th>Failures</th>
                <th>Engines</th>
                <th>Last Seen</th>
              </tr>
            </thead>
            <tbody>
              {workflows.map((workflow) => (
                <tr key={workflow.workflow_id}>
                  <td>
                    <div className="table-primary">{workflow.name}</div>
                    <div className="helper">{workflow.workflow_id}</div>
                  </td>
                  <td>
                    <span className={statusPillClass(workflow.status)}>{workflow.status}</span>
                  </td>
                  <td>{workflow.recent_runs}</td>
                  <td>{workflow.failure_runs}</td>
                  <td>{workflow.engine_count}</td>
                  <td>{formatDateTime(workflow.last_seen_at)}</td>
                </tr>
              ))}
              {workflows.length === 0 ? (
                <tr>
                  <td colSpan={6}>
                    <p className="helper">No workflow runtime data in this window.</p>
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
      </article>
    </PageShell>
  );
}
