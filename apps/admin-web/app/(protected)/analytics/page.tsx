"use client";

import { useEffect, useMemo, useState } from "react";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Legend,
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
  getAnalyticsBreakdown,
  getAnalyticsSummary,
  getAnalyticsTimeseries,
  getAnalyticsTopConsumers
} from "@/lib/api";
import type {
  AdminAnalyticsBreakdownEntry,
  AdminAnalyticsSummaryResponse,
  AdminAnalyticsTimeseriesPoint,
  AdminAnalyticsTopConsumerItem
} from "@/types/admin";

function formatBucket(bucket: string): string {
  const date = new Date(bucket);
  if (Number.isNaN(date.getTime())) {
    return bucket;
  }
  return `${date.getMonth() + 1}/${date.getDate()} ${date.getHours()}:00`;
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat().format(value);
}

export default function AnalyticsPage() {
  const [windowHours, setWindowHours] = useState(24);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [summary, setSummary] = useState<AdminAnalyticsSummaryResponse | null>(null);
  const [timeseries, setTimeseries] = useState<AdminAnalyticsTimeseriesPoint[]>([]);
  const [engineBreakdown, setEngineBreakdown] = useState<AdminAnalyticsBreakdownEntry[]>([]);
  const [topConsumers, setTopConsumers] = useState<AdminAnalyticsTopConsumerItem[]>([]);

  const successRate = useMemo(() => {
    if (!summary || summary.requests_total === 0) {
      return 0;
    }
    return ((summary.success_total / summary.requests_total) * 100).toFixed(2);
  }, [summary]);

  async function loadAnalytics() {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const [summaryResponse, timeseriesResponse, breakdownResponse, topConsumersResponse] =
        await Promise.all([
          getAnalyticsSummary(token, { window_hours: windowHours }),
          getAnalyticsTimeseries(token, {
            window_hours: windowHours,
            bucket: windowHours > 72 ? "day" : "hour"
          }),
          getAnalyticsBreakdown(token, { window_hours: windowHours, limit: 8 }),
          getAnalyticsTopConsumers(token, { window_hours: windowHours, limit: 8 })
        ]);

      setSummary(summaryResponse);
      setTimeseries(timeseriesResponse.points);
      setEngineBreakdown(breakdownResponse.engines);
      setTopConsumers(topConsumersResponse.items);
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to load analytics");
      }
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadAnalytics();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [windowHours]);

  return (
    <PageShell
      title="Usage Analytics"
      summary="Time-windowed usage, engine segmentation, and top-consumer attribution."
    >
      <div className="panel-inline">
        <label>
          Window
          <select
            value={windowHours}
            onChange={(event) => setWindowHours(Number.parseInt(event.target.value, 10))}
          >
            <option value={24}>Last 24 hours</option>
            <option value={72}>Last 72 hours</option>
            <option value={168}>Last 7 days</option>
            <option value={336}>Last 14 days</option>
          </select>
        </label>
        <button type="button" onClick={() => void loadAnalytics()}>
          Refresh
        </button>
      </div>

      {error ? <div className="error">{error}</div> : null}

      <div className="grid metrics">
        <article className="metric">
          <div className="label">Requests</div>
          <div className="value">{summary ? formatNumber(summary.requests_total) : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">Error Rate</div>
          <div className="value">{summary ? `${summary.error_rate_pct.toFixed(2)}%` : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">Success Rate</div>
          <div className="value">{summary ? `${successRate}%` : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">P95 (ms)</div>
          <div className="value">{summary ? summary.p95_duration_ms.toFixed(0) : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">Active Users</div>
          <div className="value">{summary ? formatNumber(summary.active_users) : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">Unique Keys</div>
          <div className="value">{summary ? formatNumber(summary.unique_keys) : "--"}</div>
        </article>
      </div>

      {loading ? (
        <p className="helper">Loading analytics...</p>
      ) : (
        <>
          <article className="panel chart-panel">
            <h3>Traffic Over Time</h3>
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
                  <Legend />
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

          <article className="panel chart-panel">
            <h3>Engine Breakdown</h3>
            <div className="chart-wrap">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={engineBreakdown}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#214247" />
                  <XAxis dataKey="label" stroke="#9cb9b6" />
                  <YAxis stroke="#9cb9b6" />
                  <Tooltip formatter={(value) => formatNumber(Number(value))} />
                  <Legend />
                  <Bar dataKey="request_count" name="Requests" fill="#56d3c2" radius={4} />
                </BarChart>
              </ResponsiveContainer>
            </div>
          </article>

          <article className="panel">
            <h3>Top Consumers</h3>
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
                        <p className="helper">No consumers in this time window.</p>
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
