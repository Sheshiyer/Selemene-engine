"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { ActionRail, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, TableEmptyStateRow } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminBillingSubscriptions } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import type { AdminBillingSubscriptionsResponse } from "@/types/admin";

const STATUS_OPTIONS = [
  "",
  "active",
  "past_due",
  "canceled",
  "trialing",
  "incomplete",
  "expired"
] as const;

const PAGE_SIZE = 50;

function formatDateTime(value: string | null): string {
  if (!value) return "--";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? "--" : date.toLocaleString();
}

function formatDate(value: string | null): string {
  if (!value) return "--";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? "--" : date.toLocaleDateString();
}

export default function AdminBillingSubscriptionsPage() {
  const [data, setData] = useState<AdminBillingSubscriptionsResponse | null>(null);
  const [statusFilter, setStatusFilter] = useState<string>("");
  const [offset, setOffset] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const token = getAuthToken();
    if (!token) return;
    let cancelled = false;
    getAdminBillingSubscriptions(token, {
      status: statusFilter || undefined,
      limit: PAGE_SIZE,
      offset
    })
      .then((resp) => {
        if (!cancelled) {
          setData(resp);
          setError(null);
        }
      })
      .catch((err) => {
        if (cancelled) return;
        setError(
          err instanceof ApiClientError
            ? err.message
            : err instanceof Error
              ? err.message
              : "Failed to load subscriptions"
        );
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [statusFilter, offset]);

  const items = data?.items ?? [];
  const total = data?.total ?? 0;
  const showingFrom = total === 0 ? 0 : offset + 1;
  const showingTo = Math.min(offset + items.length, total);

  return (
    <PageShell
      title="Subscriptions"
      summary="Every Dodo subscription across all users. Filter by status, click into a row to see detail and cancel locally."
    >
      {error ? (
        <StateBanner
          variant="error"
          title="Unable to load subscriptions"
          description={error}
        />
      ) : null}

      <SurfaceCard
        eyebrow="Billing"
        title="Subscriptions"
        summary={`Showing ${showingFrom}–${showingTo} of ${total.toLocaleString()}`}
        actions={
          <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
            <label htmlFor="status-filter" className="helper" style={{ margin: 0 }}>
              Status
            </label>
            <select
              id="status-filter"
              value={statusFilter}
              onChange={(e) => {
                setLoading(true);
                setOffset(0);
                setStatusFilter(e.target.value);
              }}
              className="input"
            >
              {STATUS_OPTIONS.map((s) => (
                <option key={s || "all"} value={s}>
                  {s || "all"}
                </option>
              ))}
            </select>
          </div>
        }
      >
        <div style={{ overflowX: "auto" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th>ID</th>
                <th>User</th>
                <th>Status</th>
                <th>Period end</th>
                <th>Updated</th>
              </tr>
            </thead>
            <tbody>
              {loading && items.length === 0 ? (
                <TableEmptyStateRow colSpan={5} description="Loading subscriptions…" />
              ) : items.length === 0 ? (
                <TableEmptyStateRow colSpan={5} description="No subscriptions match." />
              ) : (
                items.map((sub) => (
                  <tr key={sub.id}>
                    <td>
                      <Link
                        href={`/billing/subscriptions/${sub.id}`}
                        style={{ fontFamily: "monospace" }}
                      >
                        {sub.id.slice(0, 8)}…
                      </Link>
                    </td>
                    <td style={{ fontFamily: "monospace" }}>
                      {sub.user_id.slice(0, 8)}…
                    </td>
                    <td>
                      <span className={statusPillClass(sub.status)}>{sub.status}</span>
                    </td>
                    <td>{formatDate(sub.current_period_end)}</td>
                    <td>{formatDateTime(sub.updated_at)}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        <ActionRail>
          <button
            type="button"
            className="button button-secondary"
            disabled={offset === 0}
            onClick={() => {
              setLoading(true);
              setOffset(Math.max(0, offset - PAGE_SIZE));
            }}
          >
            ← Prev
          </button>
          <button
            type="button"
            className="button button-secondary"
            disabled={offset + items.length >= total}
            onClick={() => {
              setLoading(true);
              setOffset(offset + PAGE_SIZE);
            }}
          >
            Next →
          </button>
        </ActionRail>
      </SurfaceCard>
    </PageShell>
  );
}
