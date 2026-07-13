"use client";

import { useCallback, useEffect, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis
} from "recharts";
import { ActionRail, MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getAdminBridgeHealth,
  getAnalyticsSummary,
  getAnalyticsTimeseries,
  getAnalyticsTopConsumers,
  getWitnessDyadAnalytics
} from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getNumberParam } from "@/lib/url-query";
import type {
  AdminAnalyticsSummaryResponse,
  AdminAnalyticsTimeseriesPoint,
  AdminAnalyticsTopConsumerItem,
  AdminBridgeHealthResponse,
  AdminWitnessDyadAnalyticsResponse
} from "@/types/admin";

const REFRESH_OPTIONS = [0, 15, 30, 60] as const;

function formatBucket(bucket: string): string {
  const date = new Date(bucket);
  if (Number.isNaN(date.getTime())) {
    return bucket;
  }
  return `${date.getHours()}:00`;
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat().format(value);
}

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

export default function DashboardPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const autoRefreshSec = getNumberParam(searchParams, "refresh", 0, 0, 60);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);

  const [summary, setSummary] = useState<AdminAnalyticsSummaryResponse | null>(null);
  const [timeseries, setTimeseries] = useState<AdminAnalyticsTimeseriesPoint[]>([]);
  const [topConsumers, setTopConsumers] = useState<AdminAnalyticsTopConsumerItem[]>([]);
  const [witnessDyadAnalytics, setWitnessDyadAnalytics] = useState<AdminWitnessDyadAnalyticsResponse | null>(null);
  const [bridgeHealth, setBridgeHealth] = useState<AdminBridgeHealthResponse | null>(null);

  const updateQuery = useCallback(
    (updates: { refresh?: number }) => {
      const nextQuery = buildQueryString(searchParams, {
        refresh: updates.refresh && updates.refresh > 0 ? updates.refresh : undefined
      });
      router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
    },
    [pathname, router, searchParams]
  );

  const loadDashboard = useCallback(async () => {
    const token = getAuthToken();
    if (!token) {
      throw new Error("Missing session token. Please sign in again.");
    }

    const [summaryResponse, timeseriesResponse, topConsumersResponse, witnessDyadResponse, bridgeHealthResponse] = await Promise.all([
      getAnalyticsSummary(token, { window_hours: 24 }),
      getAnalyticsTimeseries(token, { window_hours: 24, bucket: "hour" }),
      getAnalyticsTopConsumers(token, { window_hours: 24, limit: 5 }),
      getWitnessDyadAnalytics(token, { window_hours: 24 }),
      getAdminBridgeHealth(token)
    ]);

    return { summaryResponse, timeseriesResponse, topConsumersResponse, witnessDyadResponse, bridgeHealthResponse };
  }, []);

  const handleFetch = useCallback(
    (
      promise: Promise<{
        summaryResponse: AdminAnalyticsSummaryResponse;
        timeseriesResponse: { points: AdminAnalyticsTimeseriesPoint[] };
        topConsumersResponse: { items: AdminAnalyticsTopConsumerItem[] };
        witnessDyadResponse: AdminWitnessDyadAnalyticsResponse;
        bridgeHealthResponse: AdminBridgeHealthResponse;
      }>
    ) => {
      setLoading(true);
      setError(null);
      promise
        .then(({ summaryResponse, timeseriesResponse, topConsumersResponse, witnessDyadResponse, bridgeHealthResponse }) => {
          setSummary(summaryResponse);
          setTimeseries(timeseriesResponse.points);
          setTopConsumers(topConsumersResponse.items);
          setWitnessDyadAnalytics(witnessDyadResponse);
          setBridgeHealth(bridgeHealthResponse);
          setLastUpdatedAt(new Date().toISOString());
        })
        .catch((err) => {
          if (err instanceof ApiClientError) {
            setError(err.payload?.error || err.message);
          } else if (err instanceof Error) {
            setError(err.message);
          } else {
            setError("Failed to load dashboard");
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
      handleFetch(loadDashboard());
    });
    return () => {
      cancelled = true;
    };
  }, [handleFetch, loadDashboard]);

  useEffect(() => {
    if (autoRefreshSec <= 0) {
      return;
    }
    const interval = window.setInterval(() => {
      handleFetch(loadDashboard());
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, handleFetch, loadDashboard]);

  return (
    <PageShell
      title="Dashboard"
      summary="Live platform snapshots for active users, request volume, and error posture over the last 24h."
      actions={
        <ActionRail label="Dashboard actions">
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
          <button type="button" onClick={() => handleFetch(loadDashboard())}>
            Refresh
          </button>
        </ActionRail>
      }
    >
      <p className="helper">Last updated: {formatDateTime(lastUpdatedAt)}</p>

      {error ? <StateBanner variant="error" title={error} /> : null}

      <div className="grid metrics">
        <MetricSurface
          label="Active Users (24h)"
          value={summary ? formatNumber(summary.active_users) : "--"}
          detail="Authenticated users with activity in the current 24h window."
        />
        <MetricSurface
          label="API Requests (24h)"
          value={summary ? formatNumber(summary.requests_total) : "--"}
          detail="Aggregate request count across admin-visible engine traffic."
        />
        <MetricSurface
          label="Error Rate"
          value={summary ? `${summary.error_rate_pct.toFixed(2)}%` : "--"}
          detail="Failure ratio for the same 24h request population."
        />
        <article className="metric">
          <div className="label">Witness Dyad LLM Rate</div>
          <div className="value">{witnessDyadAnalytics ? `${witnessDyadAnalytics.llm_rate_pct.toFixed(1)}%` : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">Bridge Status</div>
          <div className="value">
            {bridgeHealth ? (
              <span className={bridgeHealth.overall_status === "healthy" ? "pill ok" : "pill warn"}>
                {bridgeHealth.overall_status}
              </span>
            ) : "--"}
          </div>
        </article>
      </div>

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading dashboard metrics"
          description="Resolving analytics summary, 24h timeseries, top-consumer telemetry, witness dyad metrics, and bridge health."
        />
      ) : (
        <>
          <SurfaceCard
            eyebrow="Telemetry"
            title="24h Request Trend"
            summary="Hourly request and failure trajectories for the active traffic window."
            className="chart-panel"
          >
            <div className="chart-wrap">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={timeseries}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#214247" />
                  <XAxis
                    dataKey="bucket_start"
                    tickFormatter={formatBucket}
                    stroke="#9cb9b6"
                    minTickGap={20}
                  />
                  <YAxis stroke="#9cb9b6" />
                  <Tooltip
                    formatter={(value) => formatNumber(Number(value))}
                    labelFormatter={(label) => formatBucket(String(label))}
                  />
                  <Line
                    type="monotone"
                    dataKey="request_count"
                    name="Requests"
                    stroke="#56d3c2"
                    strokeWidth={2}
                    dot={false}
                  />
                  <Line
                    type="monotone"
                    dataKey="failure_count"
                    name="Failures"
                    stroke="#ef6b73"
                    strokeWidth={2}
                    dot={false}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </SurfaceCard>

          <SurfaceCard
            eyebrow="Demand"
            title="Top Consumers (24h)"
            summary="Highest-traffic users ranked by volume and failure load."
          >
            <div className="table-wrap compact">
              <table>
                <thead>
                  <tr>
                    <th>User</th>
                    <th>Requests</th>
                    <th>Failures</th>
                    <th>Avg Duration (ms)</th>
                  </tr>
                </thead>
                <tbody>
                  {topConsumers.map((item) => (
                    <tr key={item.user_id}>
                      <td>
                        <div className="table-primary">{item.user_email}</div>
                        <div className="helper">{item.user_id}</div>
                      </td>
                      <td>{formatNumber(item.request_count)}</td>
                      <td>{formatNumber(item.failure_count)}</td>
                      <td>{item.avg_duration_ms.toFixed(1)}</td>
                    </tr>
                  ))}
                  {topConsumers.length === 0 ? (
                    <TableEmptyStateRow
                      colSpan={4}
                      title="No traffic data"
                      description="Top-consumer rankings will appear after request volume is recorded."
                    />
                  ) : null}
                </tbody>
              </table>
            </div>
          </SurfaceCard>

          {witnessDyadAnalytics ? (
            <SurfaceCard
              eyebrow="Consciousness"
              title="Witness Dyad Quick Stats"
              summary="LLM routing split, engine coverage, and tier distribution for the 24h traffic window."
            >
              <div className="grid metrics">
                <article className="metric">
                  <div className="label">LLM Rate</div>
                  <div className="value">{witnessDyadAnalytics.llm_rate_pct.toFixed(1)}%</div>
                </article>
                <article className="metric">
                  <div className="label">Avg LLM Duration</div>
                  <div className="value">{witnessDyadAnalytics.avg_llm_duration_ms.toFixed(1)} ms</div>
                </article>
              </div>
              {witnessDyadAnalytics.engine_coverage.length > 0 ? (
                <div className="grid metrics">
                  {witnessDyadAnalytics.engine_coverage.map((entry) => (
                    <article key={entry.label} className="metric">
                      <div className="label">{entry.label}</div>
                      <div className="value">{formatNumber(entry.request_count)}</div>
                    </article>
                  ))}
                </div>
              ) : null}
              {witnessDyadAnalytics.tier_breakdown.length > 0 ? (
                <div className="table-wrap compact">
                  <table>
                    <thead>
                      <tr>
                        <th>Tier</th>
                        <th>LLM Count</th>
                        <th>Rule Count</th>
                      </tr>
                    </thead>
                    <tbody>
                      {witnessDyadAnalytics.tier_breakdown.map((entry) => (
                        <tr key={entry.tier}>
                          <td>
                            <div className="table-primary">{entry.tier}</div>
                          </td>
                          <td>{formatNumber(entry.llm_count)}</td>
                          <td>{formatNumber(entry.rule_count)}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : null}
            </SurfaceCard>
          ) : null}

          {bridgeHealth ? (
            <SurfaceCard
              eyebrow="Connectivity"
              title="Engines Quick View"
              summary="Sidecar engine fleet health, circuit posture, and connectivity status."
            >
              <div className="grid metrics">
                <article className="metric">
                  <div className="label">Overall Status</div>
                  <div className="value">
                    <span className={statusPillClass(bridgeHealth.overall_status)}>
                      {bridgeHealth.overall_status}
                    </span>
                  </div>
                </article>
                <article className="metric">
                  <div className="label">Total Engines</div>
                  <div className="value">{bridgeHealth.total_engines}</div>
                </article>
                <article className="metric">
                  <div className="label">Healthy</div>
                  <div className="value">{bridgeHealth.healthy_engines}</div>
                </article>
                <article className="metric">
                  <div className="label">Degraded</div>
                  <div className="value">{bridgeHealth.degraded_engines}</div>
                </article>
                <article className="metric">
                  <div className="label">Sidecar Reachable</div>
                  <div className="value">
                    <span className={statusPillClass(bridgeHealth.sidecar_reachable ? "healthy" : "unavailable")}>
                      {bridgeHealth.sidecar_reachable ? "Yes" : "No"}
                    </span>
                  </div>
                </article>
              </div>
              {bridgeHealth.failed_engines.length > 0 ? (
                <div className="helper">
                  Failed engines: {bridgeHealth.failed_engines.join(", ")}
                </div>
              ) : null}
            </SurfaceCard>
          ) : null}
        </>
      )}
    </PageShell>
  );
}
