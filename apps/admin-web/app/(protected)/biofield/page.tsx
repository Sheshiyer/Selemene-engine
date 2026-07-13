"use client";

import { useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import { ApiClientError, getAdminBiofieldSessions } from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getNumberParam, getStringParam } from "@/lib/url-query";
import type { AdminBiofieldSessionItem } from "@/types/admin";

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

export default function BiofieldPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const statusFilter = getStringParam(searchParams, "status");
  const userIdFilter = getStringParam(searchParams, "user_id");
  const limit = getNumberParam(searchParams, "limit", 50, 1, 200);
  const offset = getNumberParam(searchParams, "offset", 0, 0, 100_000);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [sessions, setSessions] = useState<AdminBiofieldSessionItem[]>([]);
  const [total, setTotal] = useState(0);

  function updateParam(key: string, value: string) {
    const nextQuery = buildQueryString(searchParams, { [key]: value || undefined, offset: key !== "offset" ? undefined : value });
    router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
  }

  function setPage(delta: number) {
    const next = Math.max(0, offset + delta * limit);
    updateParam("offset", String(next));
  }

  const activeCount = useMemo(
    () => sessions.filter((s) => s.status === "active").length,
    [sessions]
  );

  const abandonedCount = useMemo(
    () => sessions.filter((s) => s.status === "abandoned").length,
    [sessions]
  );

  useEffect(() => {
    let cancelled = false;
    const token = getAuthToken() ?? undefined;

    setError(null);
    setLoading(true);

    getAdminBiofieldSessions(token, {
      status: statusFilter || undefined,
      user_id: userIdFilter || undefined,
      limit,
      offset
    })
      .then((response) => {
        if (!cancelled) {
          setSessions(response.items);
          setTotal(response.total);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          if (err instanceof ApiClientError) {
            setError(err.payload?.error || err.message);
          } else if (err instanceof Error) {
            setError(err.message);
          } else {
            setError("Failed to load biofield sessions");
          }
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [statusFilter, userIdFilter, limit, offset]);

  return (
    <PageShell
      title="Biofield Sessions"
      summary="Browse biofield capture sessions with device tracking and artifact metadata."
    >
      <div className="panel-inline">
        <label>
          Status
          <select
            value={statusFilter}
            onChange={(event) => updateParam("status", event.target.value)}
          >
            <option value="">All</option>
            <option value="active">Active</option>
            <option value="closed">Closed</option>
            <option value="abandoned">Abandoned</option>
          </select>
        </label>
        <label>
          User ID
          <input
            value={userIdFilter}
            onChange={(event) => updateParam("user_id", event.target.value)}
            placeholder="user uuid"
          />
        </label>
        <button type="button" onClick={() => { updateParam("status", ""); updateParam("user_id", ""); }}>
          Clear
        </button>
      </div>

      <div className="grid metrics">
        <article className="metric">
          <div className="label">Total sessions</div>
          <div className="value">{total}</div>
        </article>
        <article className="metric">
          <div className="label">Visible</div>
          <div className="value">{sessions.length}</div>
        </article>
        <article className="metric">
          <div className="label">Active</div>
          <div className="value">{activeCount}</div>
        </article>
        <article className="metric">
          <div className="label">Abandoned</div>
          <div className="value">{abandonedCount}</div>
        </article>
      </div>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading biofield sessions"
          description="Resolving session inventory, device metadata, artifact and reading counts."
        />
      ) : (
        <>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>User Email</th>
                  <th>Status</th>
                  <th>Device ID</th>
                  <th>Started At</th>
                  <th>Closed At</th>
                  <th>Artifacts</th>
                  <th>Readings</th>
                  <th>Latest Reading</th>
                </tr>
              </thead>
              <tbody>
                {sessions.map((session) => (
                  <tr
                    key={session.session_id}
                    className="clickable-row"
                    tabIndex={0}
                    onClick={() => router.push(`/biofield/${session.session_id}`)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        router.push(`/biofield/${session.session_id}`);
                      }
                    }}
                  >
                    <td>
                      <div className="table-primary">{session.user_email}</div>
                      <div className="helper">ID: {session.user_id}</div>
                    </td>
                    <td>
                      <span className={statusPillClass(session.status)}>{session.status}</span>
                    </td>
                    <td>
                      <div className="helper">{session.client_device_id ?? "--"}</div>
                      <div className="helper">{session.viewer_version ?? ""}</div>
                    </td>
                    <td>{formatDateTime(session.started_at)}</td>
                    <td>{formatDateTime(session.closed_at)}</td>
                    <td>{session.artifact_count}</td>
                    <td>{session.reading_count}</td>
                    <td>{formatDateTime(session.latest_reading_at)}</td>
                  </tr>
                ))}
                {sessions.length === 0 ? (
                  <TableEmptyStateRow
                    colSpan={8}
                    title="No biofield sessions matched"
                    description="Try adjusting the status or user ID filters."
                  />
                ) : null}
              </tbody>
            </table>
          </div>

          <div className="panel-inline">
            <button
              type="button"
              disabled={offset === 0}
              onClick={() => setPage(-1)}
            >
              Previous
            </button>
            <span className="helper">
              {offset + 1}–{Math.min(offset + limit, total)} of {total}
            </span>
            <button
              type="button"
              disabled={offset + limit >= total}
              onClick={() => setPage(1)}
            >
              Next
            </button>
          </div>
        </>
      )}
    </PageShell>
  );
}