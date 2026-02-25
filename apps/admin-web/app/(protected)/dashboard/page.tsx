"use client";

import { useEffect, useState } from "react";
import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis
} from "recharts";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getAnalyticsSummary,
  getAnalyticsTimeseries,
  getAnalyticsTopConsumers
} from "@/lib/api";
import type {
  AdminAnalyticsSummaryResponse,
  AdminAnalyticsTimeseriesPoint,
  AdminAnalyticsTopConsumerItem
} from "@/types/admin";

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

export default function DashboardPage() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [summary, setSummary] = useState<AdminAnalyticsSummaryResponse | null>(null);
  const [timeseries, setTimeseries] = useState<AdminAnalyticsTimeseriesPoint[]>([]);
  const [topConsumers, setTopConsumers] = useState<AdminAnalyticsTopConsumerItem[]>([]);

  async function loadDashboard() {
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
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to load dashboard");
      }
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadDashboard();
  }, []);

  return (
    <PageShell
      title="Dashboard"
      summary="Live platform snapshots for active users, request volume, and error posture over the last 24h."
    >
      <div className="panel-inline">
        <button type="button" onClick={() => void loadDashboard()}>
          Refresh
        </button>
      </div>

      {error ? <div className="error">{error}</div> : null}

      <div className="grid metrics">
        <article className="metric">
          <div className="label">Active Users (24h)</div>
          <div className="value">{summary ? formatNumber(summary.active_users) : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">API Requests (24h)</div>
          <div className="value">{summary ? formatNumber(summary.requests_total) : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">Error Rate</div>
          <div className="value">{summary ? `${summary.error_rate_pct.toFixed(2)}%` : "--"}</div>
        </article>
      </div>

      {loading ? (
        <p className="helper">Loading dashboard metrics...</p>
      ) : (
        <>
          <article className="panel chart-panel">
            <h3>24h Request Trend</h3>
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
          </article>

          <article className="panel">
            <h3>Top Consumers (24h)</h3>
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
                    <tr>
                      <td colSpan={4}>
                        <p className="helper">No traffic data available.</p>
                      </td>
                    </tr>
                  ) : null}
                </tbody>
              </table>
            </div>
          </article>
        </>
      )}
    </PageShell>
  );
}
