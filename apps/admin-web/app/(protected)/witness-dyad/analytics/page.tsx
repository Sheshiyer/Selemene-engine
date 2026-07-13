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
import { MetricSurface } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getWitnessDyadAnalytics } from "@/lib/api";
import { buildQueryString, getNumberParam } from "@/lib/url-query";
import type { AdminWitnessDyadAnalyticsResponse } from "@/types/admin";

function formatNumber(value: number): string {
  return new Intl.NumberFormat().format(value);
}

const CHART_COLORS = ["#56d3c2", "#8ac926", "#f9c74f", "#f9844a", "#ef6b73", "#90be6d"];
const WINDOW_OPTIONS = [24, 72, 168, 720] as const;
const REFRESH_OPTIONS = [0, 15, 30, 60] as const;

export default function WitnessDyadAnalyticsPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const windowHours = getNumberParam(searchParams, "window_hours", 168, 1, 720);
  const autoRefreshSec = getNumberParam(searchParams, "refresh", 0, 0, 60);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [analytics, setAnalytics] = useState<AdminWitnessDyadAnalyticsResponse | null>(null);

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

  const loadData = useCallback(async () => {
    const token = getAuthToken() ?? undefined;
    const currentWindowHours = getNumberParam(searchParams, "window_hours", 168, 1, 720);
    return getWitnessDyadAnalytics(token, { window_hours: currentWindowHours });
  }, [searchParams]);

  useEffect(() => {
    let cancelled = false;
    async function run() {
      setLoading(true);
      setError(null);
      try {
        const result = await loadData();
        if (!cancelled) {
          setAnalytics(result);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(
            err instanceof ApiClientError
              ? err.payload?.error || err.message
              : "Failed to load dyad analytics"
          );
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void run();
    return () => {
      cancelled = true;
    };
  }, [loadData]);

  useEffect(() => {
    if (autoRefreshSec <= 0) return;
    const interval = window.setInterval(() => {
      void (async () => {
        try {
          const result = await loadData();
          setAnalytics(result);
        } catch {
          /* keep stale */
        }
      })();
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, loadData]);

  const llmPieData = useMemo(
    () =>
      (analytics?.llm_vs_rule_based ?? []).map((entry) => ({
        name: entry.llm_powered ? "LLM" : "Rule",
        value: entry.count
      })),
    [analytics]
  );

  return (
    <PageShell
      title="Witness Dyad Analytics"
      summary="LLM vs rule-based distribution, engine coverage, tier breakdown, and latency metrics."
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
            {WINDOW_OPTIONS.map((value) => (
              <option key={value} value={value}>
                Last {value}h
              </option>
            ))}
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
        <button type="button" onClick={() => window.location.reload()}>
          Refresh
        </button>
      </div>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading dyad analytics"
          description="Computing LLM rate, engine coverage, tier breakdown, and latency."
        />
      ) : analytics ? (
        <>
          <div className="grid metrics">
            <MetricSurface
              label="LLM Rate"
              value={`${analytics.llm_rate_pct.toFixed(1)}%`}
              detail="Percentage of LLM-powered executions"
            />
            <MetricSurface
              label="Avg LLM Latency"
              value={`${analytics.avg_llm_duration_ms.toLocaleString()} ms`}
              detail="Average LLM processing duration"
            />
            <MetricSurface
              label="Window"
              value={`${analytics.window_hours}h`}
              detail="Active reporting window"
            />
          </div>

          <div className="grid two-col">
            <article className="panel chart-panel">
              <h3>LLM vs Rule-based</h3>
              <div className="chart-wrap">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={llmPieData}
                      dataKey="value"
                      nameKey="name"
                      outerRadius={110}
                      label
                    >
                      {llmPieData.map((entry, index) => (
                        <Cell
                          key={entry.name}
                          fill={CHART_COLORS[index % CHART_COLORS.length]}
                        />
                      ))}
                    </Pie>
                    <Tooltip formatter={(value) => formatNumber(Number(value))} />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              </div>
            </article>

            <article className="panel chart-panel">
              <h3>Engine Coverage</h3>
              <div className="chart-wrap">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={analytics.engine_coverage ?? []}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#214247" />
                    <XAxis dataKey="label" stroke="#9cb9b6" />
                    <YAxis stroke="#9cb9b6" />
                    <Tooltip formatter={(value) => formatNumber(Number(value))} />
                    <Bar dataKey="request_count" name="Requests" fill="#56d3c2" radius={4} />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </article>
          </div>

          <article className="panel">
            <h3>Tier Breakdown</h3>
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
                  {(analytics.tier_breakdown ?? []).map((row) => (
                    <tr key={row.tier}>
                      <td>
                        <div className="table-primary">{row.tier}</div>
                      </td>
                      <td>{formatNumber(row.llm_count)}</td>
                      <td>{formatNumber(row.rule_count)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </article>
        </>
      ) : null}
    </PageShell>
  );
}