"use client";

import { useCallback, useEffect, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { ActionRail, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { EventStream, EventStreamItem } from "@/components/event-stream";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminObservabilitySummary } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getNumberParam } from "@/lib/url-query";
import type { AdminObservabilitySummary } from "@/types/admin";

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

function alertSeverityPillClass(severity: string): string {
  switch (severity.toLowerCase()) {
    case "critical":
      return "pill danger";
    case "warning":
      return "pill warn";
    case "info":
      return "pill ok";
    default:
      return "pill";
  }
}

export default function ObservabilityPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const autoRefreshSec = getNumberParam(searchParams, "refresh", 0, 0, 60);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);

  const [summary, setSummary] = useState<AdminObservabilitySummary | null>(null);

  const updateQuery = useCallback(
    (updates: { refresh?: number }) => {
      const nextQuery = buildQueryString(searchParams, {
        refresh: updates.refresh && updates.refresh > 0 ? updates.refresh : undefined
      });
      router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
    },
    [pathname, router, searchParams]
  );

  const loadSummary = useCallback(async () => {
    const token = getAuthToken();
    if (!token) {
      throw new Error("Missing session token. Please sign in again.");
    }

    return getAdminObservabilitySummary(token);
  }, []);

  const handleFetch = useCallback(
    (promise: Promise<AdminObservabilitySummary>) => {
      setLoading(true);
      setError(null);
      promise
        .then((data) => {
          setSummary(data);
          setLastUpdatedAt(new Date().toISOString());
        })
        .catch((err) => {
          if (err instanceof ApiClientError) {
            setError(err.payload?.error || err.message);
          } else if (err instanceof Error) {
            setError(err.message);
          } else {
            setError("Failed to load observability summary");
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
      handleFetch(loadSummary());
    });
    return () => {
      cancelled = true;
    };
  }, [handleFetch, loadSummary]);

  useEffect(() => {
    if (autoRefreshSec <= 0) {
      return;
    }
    const interval = window.setInterval(() => {
      handleFetch(loadSummary());
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, handleFetch, loadSummary]);

  return (
    <PageShell
      title="Observability Stack"
      summary="Prometheus, Alertmanager, Grafana, Loki, and Jaeger status with active alert feed."
      actions={
        <ActionRail label="Observability actions">
          <label>
            <span className="sr-only">Auto refresh</span>
            <select
              value={autoRefreshSec}
              onChange={(event) =>
                updateQuery({ refresh: Number.parseInt(event.target.value, 10) })
              }
            >
              {REFRESH_OPTIONS.map((value) => (
                <option key={value} value={value}>
                  {value === 0 ? "Auto refresh: Off" : `Auto refresh: ${value}s`}
                </option>
              ))}
            </select>
          </label>
          <button type="button" onClick={() => handleFetch(loadSummary())}>
            Refresh
          </button>
        </ActionRail>
      }
    >
      <p className="helper">Last updated: {formatDateTime(lastUpdatedAt)}</p>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {/* Uptime & Metrics Summary */}
      <div className="grid metrics">
        <article className="metric">
          <div className="label">Uptime</div>
          <div className="value">{summary ? formatDuration(summary.uptime_seconds) : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">Metrics Endpoint</div>
          <div className="value helper">{summary?.metrics_endpoint ?? "--"}</div>
        </article>
      </div>

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading observability stack status"
          description="Resolving Prometheus, Alertmanager, Grafana, Loki, Jaeger connectivity, and active alerts."
        />
      ) : (
        <>
          {/* Core Services Status */}
          <SurfaceCard
            eyebrow="Stack"
            title="Core Services"
            summary="Configuration, endpoint, and reachability posture for each observability service node."
          >
            <div className="table-wrap compact">
              <table>
                <thead>
                  <tr>
                    <th>Service</th>
                    <th>Configured</th>
                    <th>URL</th>
                    <th>Reachable</th>
                  </tr>
                </thead>
                <tbody>
                  {(["prometheus", "alertmanager", "grafana", "loki", "jaeger"] as const).map(
                    (name) => {
                      const service = summary?.[name];
                      return (
                        <tr key={name}>
                          <td>
                            <div className="table-primary">{name.charAt(0).toUpperCase() + name.slice(1)}</div>
                          </td>
                          <td>
                            <span
                              className={statusPillClass(
                                service?.configured ? "healthy" : "unavailable"
                              )}
                            >
                              {service?.configured ? "Yes" : "No"}
                            </span>
                          </td>
                          <td className="cell-wrap">{service?.url ?? "--"}</td>
                          <td>
                            <span
                              className={statusPillClass(
                                service?.reachable ? "healthy" : "unavailable"
                              )}
                            >
                              {service?.reachable ? "Yes" : "No"}
                            </span>
                          </td>
                        </tr>
                      );
                    }
                  )}
                </tbody>
              </table>
            </div>
          </SurfaceCard>

          {/* Grafana Dashboards */}
          {summary?.grafana_dashboards && summary.grafana_dashboards.length > 0 ? (
            <article className="panel">
              <h3>Grafana Dashboards</h3>
              <div className="table-wrap compact">
                <table>
                  <thead>
                    <tr>
                      <th>Dashboard</th>
                    </tr>
                  </thead>
                  <tbody>
                    {summary.grafana_dashboards.map((dashboard) => (
                      <tr key={dashboard}>
                        <td>
                          <div className="table-primary">{dashboard}</div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </article>
          ) : null}

          {/* Active Alerts */}
          <article className="panel">
            <h3>Active Alerts</h3>
            {summary?.active_alerts && summary.active_alerts.length > 0 ? (
              <EventStream label="Active alert stream">
                {summary.active_alerts.map((alert) => (
                  <EventStreamItem
                    key={alert.name}
                    eyebrow={alert.service}
                    title={alert.name}
                    subtitle={alert.summary}
                    badge={
                      <span className={alertSeverityPillClass(alert.severity)}>
                        {alert.severity}
                      </span>
                    }
                    metadata={[
                      { label: "Since", value: formatDateTime(alert.since) }
                    ]}
                    summary={alert.summary}
                  />
                ))}
              </EventStream>
            ) : (
              <div className="state-table-empty event-stream-empty">
                <div className="telemetry-caption">Clear</div>
                <div className="state-table-empty-title">No active alerts</div>
                <div className="helper">
                  No alerting rules are currently firing across the observability stack.
                </div>
              </div>
            )}
          </article>
        </>
      )}
    </PageShell>
  );
}