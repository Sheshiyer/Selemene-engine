"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis
} from "recharts";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminUsageSummary } from "@/lib/api";
import { buildQueryString, getNumberParam } from "@/lib/url-query";
import type { AdminUsageSummaryResponse } from "@/types/admin";

const REFRESH_OPTIONS = [0, 15, 30, 60] as const;
const RANGE_DAY_OPTIONS = [7, 14, 30, 60, 90] as const;
const CHART_COLORS = ["#56d3c2", "#8ac926", "#f9c74f", "#f9844a", "#ef6b73", "#90be6d"];

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

export default function AnalyticsPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const [rangeDays, setRangeDays] = useState(() =>
    getNumberParam(searchParams, "range_days", 30, 7, 90)
  );
  const [autoRefreshSec, setAutoRefreshSec] = useState(() =>
    getNumberParam(searchParams, "refresh", 0, 0, 60)
  );

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);
  const [usage, setUsage] = useState<AdminUsageSummaryResponse | null>(null);

  useEffect(() => {
    setRangeDays(getNumberParam(searchParams, "range_days", 30, 7, 90));
    setAutoRefreshSec(getNumberParam(searchParams, "refresh", 0, 0, 60));
  }, [searchParams]);

  useEffect(() => {
    const nextQuery = buildQueryString(searchParams, {
      range_days: rangeDays,
      refresh: autoRefreshSec > 0 ? autoRefreshSec : undefined
    });
    router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
  }, [autoRefreshSec, pathname, rangeDays, router, searchParams]);

  const loadUsage = useCallback(async () => {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const response = await getAdminUsageSummary(token, {
        range_days: rangeDays,
        engine_limit: 10,
        top_users_limit: 10
      });

      setUsage(response);
      setLastUpdatedAt(new Date().toISOString());
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to load usage analytics");
      }
    } finally {
      setLoading(false);
    }
  }, [rangeDays]);

  useEffect(() => {
    void loadUsage();
  }, [loadUsage]);

  useEffect(() => {
    if (autoRefreshSec <= 0) {
      return;
    }
    const interval = window.setInterval(() => {
      void loadUsage();
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, loadUsage]);

  const chartData = useMemo(() => usage?.daily_requests ?? [], [usage]);

  return (
    <PageShell
      title="Usage Analytics"
      summary="Daily request activity, engine popularity, tier distribution, and top users."
    >
      <div className="panel-inline">
        <label>
          Date range
          <select
            value={rangeDays}
            onChange={(event) => setRangeDays(Number.parseInt(event.target.value, 10))}
          >
            {RANGE_DAY_OPTIONS.map((value) => (
              <option key={value} value={value}>
                Last {value} days
              </option>
            ))}
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
        <button type="button" onClick={() => void loadUsage()}>
          Refresh
        </button>
      </div>

      <p className="helper">Last updated: {formatDateTime(lastUpdatedAt)}</p>

      {error ? <div className="error">{error}</div> : null}

      <div className="grid metrics">
        <article className="metric">
          <div className="label">24h Requests</div>
          <div className="value">{usage ? formatNumber(usage.daily.total) : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">30d Requests</div>
          <div className="value">{usage ? formatNumber(usage.monthly.total) : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">24h Active Users</div>
          <div className="value">{usage ? formatNumber(usage.daily.active_users) : "--"}</div>
        </article>
        <article className="metric">
          <div className="label">30d Active Users</div>
          <div className="value">{usage ? formatNumber(usage.monthly.active_users) : "--"}</div>
        </article>
      </div>

      {loading ? (
        <p className="helper">Loading analytics...</p>
      ) : (
        <>
          <article className="panel chart-panel">
            <h3>Daily Requests</h3>
            <div className="chart-wrap">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={chartData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#214247" />
                  <XAxis dataKey="day" stroke="#9cb9b6" />
                  <YAxis stroke="#9cb9b6" />
                  <Tooltip formatter={(value) => formatNumber(Number(value))} />
                  <Legend />
                  <Bar dataKey="request_count" name="Requests" fill="#56d3c2" radius={4} />
                </BarChart>
              </ResponsiveContainer>
            </div>
          </article>

          <div className="grid two-col">
            <article className="panel chart-panel">
              <h3>Engine Popularity</h3>
              <div className="chart-wrap">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={usage?.engine_breakdown ?? []}
                      dataKey="request_count"
                      nameKey="engine_id"
                      outerRadius={110}
                      label
                    >
                      {(usage?.engine_breakdown ?? []).map((entry, index) => (
                        <Cell key={`${entry.engine_id}-${index}`} fill={CHART_COLORS[index % CHART_COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip formatter={(value) => formatNumber(Number(value))} />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              </div>
            </article>

            <article className="panel chart-panel">
              <h3>Tier Distribution</h3>
              <div className="chart-wrap">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={usage?.tier_distribution ?? []}
                      dataKey="request_count"
                      nameKey="tier"
                      outerRadius={110}
                      label
                    >
                      {(usage?.tier_distribution ?? []).map((entry, index) => (
                        <Cell key={`${entry.tier}-${index}`} fill={CHART_COLORS[index % CHART_COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip formatter={(value) => formatNumber(Number(value))} />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              </div>
            </article>
          </div>

          <article className="panel">
            <h3>Top 10 Users</h3>
            <div className="table-wrap compact">
              <table>
                <thead>
                  <tr>
                    <th>User</th>
                    <th>Requests</th>
                  </tr>
                </thead>
                <tbody>
                  {(usage?.top_users ?? []).map((item) => (
                    <tr key={item.user_id}>
                      <td>
                        <div className="table-primary">{item.user_email}</div>
                        <div className="helper">{item.user_id}</div>
                      </td>
                      <td>{formatNumber(item.request_count)}</td>
                    </tr>
                  ))}
                  {(usage?.top_users.length ?? 0) === 0 ? (
                    <tr>
                      <td colSpan={2}>
                        <p className="helper">No usage activity in the selected range.</p>
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
