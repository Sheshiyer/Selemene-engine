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
  getAnalyticsSummary,
  getAnalyticsTimeseries,
  getAnalyticsTopConsumers
} from "@/lib/api";
import { buildQueryString, getNumberParam } from "@/lib/url-query";
import type {
  AdminAnalyticsSummaryResponse,
  AdminAnalyticsTimeseriesPoint,
  AdminAnalyticsTopConsumerItem
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

  const [autoRefreshSec, setAutoRefreshSec] = useState(() =>
    getNumberParam(searchParams, "refresh", 0, 0, 60)
  );

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);

  const [summary, setSummary] = useState<AdminAnalyticsSummaryResponse | null>(null);
  const [timeseries, setTimeseries] = useState<AdminAnalyticsTimeseriesPoint[]>([]);
  const [topConsumers, setTopConsumers] = useState<AdminAnalyticsTopConsumerItem[]>([]);

  useEffect(() => {
    setAutoRefreshSec(getNumberParam(searchParams, "refresh", 0, 0, 60));
  }, [searchParams]);

  useEffect(() => {
    const nextQuery = buildQueryString(searchParams, {
      refresh: autoRefreshSec > 0 ? autoRefreshSec : undefined
    });
    router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
  }, [autoRefreshSec, pathname, router, searchParams]);

  const loadDashboard = useCallback(async () => {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const [summaryResponse, timeseriesResponse, topConsumersResponse] = await Promise.all([
        getAnalyticsSummary(token, { window_hours: 24 }),
        getAnalyticsTimeseries(token, { window_hours: 24, bucket: "hour" }),
        getAnalyticsTopConsumers(token, { window_hours: 24, limit: 5 })
      ]);

      setSummary(summaryResponse);
      setTimeseries(timeseriesResponse.points);
      setTopConsumers(topConsumersResponse.items);
      setLastUpdatedAt(new Date().toISOString());
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to load dashboard");
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadDashboard();
  }, [loadDashboard]);

  useEffect(() => {
    if (autoRefreshSec <= 0) {
      return;
    }
    const interval = window.setInterval(() => {
      void loadDashboard();
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, loadDashboard]);

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
              onChange={(event) => setAutoRefreshSec(Number.parseInt(event.target.value, 10))}
            >
              {REFRESH_OPTIONS.map((value) => (
                <option key={value} value={value}>
                  {value === 0 ? "Auto refresh: Off" : `Auto refresh: ${value}s`}
                </option>
              ))}
            </select>
          </label>
          <button type="button" onClick={() => void loadDashboard()}>
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
      </div>

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading dashboard metrics"
          description="Resolving analytics summary, 24h timeseries, and top-consumer telemetry."
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
        </>
      )}
    </PageShell>
  );
}
