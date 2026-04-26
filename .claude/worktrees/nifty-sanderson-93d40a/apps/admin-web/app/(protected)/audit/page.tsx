"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { ActionRail, MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { EventStream, EventStreamItem } from "@/components/event-stream";
import { DrawerSurface } from "@/components/overlay-surface";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getAuditActions,
  getAuditEvent,
  getAuditEvents
} from "@/lib/api";
import { copyToClipboard, exportCsv, exportJson } from "@/lib/export";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getNumberParam, getStringParam } from "@/lib/url-query";
import type { AdminAuditEventItem } from "@/types/admin";

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

function toIso(value: string): string | undefined {
  if (!value) {
    return undefined;
  }

  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return undefined;
  }

  return parsed.toISOString();
}

const REFRESH_OPTIONS = [0, 15, 30, 60] as const;

export default function AuditPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const [actor, setActor] = useState(() => getStringParam(searchParams, "actor"));
  const [action, setAction] = useState(() => getStringParam(searchParams, "action"));
  const [result, setResult] = useState(() => getStringParam(searchParams, "result"));
  const [from, setFrom] = useState(() => getStringParam(searchParams, "from"));
  const [to, setTo] = useState(() => getStringParam(searchParams, "to"));
  const [autoRefreshSec, setAutoRefreshSec] = useState(() =>
    getNumberParam(searchParams, "refresh", 0, 0, 60)
  );

  const [loading, setLoading] = useState(true);
  const [detailLoading, setDetailLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);
  const [actions, setActions] = useState<string[]>([]);
  const [events, setEvents] = useState<AdminAuditEventItem[]>([]);
  const [total, setTotal] = useState(0);
  const [selected, setSelected] = useState<AdminAuditEventItem | null>(null);

  useEffect(() => {
    setActor(getStringParam(searchParams, "actor"));
    setAction(getStringParam(searchParams, "action"));
    setResult(getStringParam(searchParams, "result"));
    setFrom(getStringParam(searchParams, "from"));
    setTo(getStringParam(searchParams, "to"));
    setAutoRefreshSec(getNumberParam(searchParams, "refresh", 0, 0, 60));
  }, [searchParams]);

  useEffect(() => {
    const nextQuery = buildQueryString(searchParams, {
      actor,
      action,
      result,
      from,
      to,
      refresh: autoRefreshSec > 0 ? autoRefreshSec : undefined
    });
    router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
  }, [action, actor, autoRefreshSec, from, pathname, result, router, searchParams, to]);

  const loadAudit = useCallback(async () => {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const [actionsResponse, eventsResponse] = await Promise.all([
        getAuditActions(token),
        getAuditEvents(token, {
          actor: actor || undefined,
          action: action || undefined,
          result: result || undefined,
          from: toIso(from),
          to: toIso(to),
          limit: 100,
          offset: 0
        })
      ]);

      setActions(actionsResponse.actions);
      setEvents(eventsResponse.items);
      setTotal(eventsResponse.total);
      setLastUpdatedAt(new Date().toISOString());

      if (selected) {
        const updated = eventsResponse.items.find((event) => event.event_id === selected.event_id);
        setSelected(updated ?? null);
      }
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to load audit events");
      }
    } finally {
      setLoading(false);
    }
  }, [action, actor, from, result, selected, to]);

  async function openDetail(eventId: string) {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      return;
    }

    setDetailLoading(true);
    setError(null);

    try {
      const detail = await getAuditEvent(token, eventId);
      setSelected(detail.event);
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to load audit event detail");
      }
    } finally {
      setDetailLoading(false);
    }
  }

  useEffect(() => {
    void loadAudit();
  }, [loadAudit]);

  useEffect(() => {
    if (autoRefreshSec <= 0) {
      return;
    }
    const interval = window.setInterval(() => {
      void loadAudit();
    }, autoRefreshSec * 1000);
    return () => window.clearInterval(interval);
  }, [autoRefreshSec, loadAudit]);

  const failureCount = useMemo(
    () => events.filter((event) => event.result !== "success").length,
    [events]
  );

  async function handleCopy(value: string, label: string) {
    try {
      await copyToClipboard(value);
      setSuccess(`${label} copied`);
      window.setTimeout(() => setSuccess(null), 1500);
    } catch {
      setError("Failed to copy value to clipboard");
    }
  }

  function handleExportCsv() {
    exportCsv(
      `admin-audit-events-${new Date().toISOString().slice(0, 10)}.csv`,
      events,
      [
        { key: "event_id", header: "Event ID" },
        { key: "request_id", header: "Request ID" },
        { key: "occurred_at", header: "Occurred At" },
        { key: "actor_email", header: "Actor Email" },
        { key: "action", header: "Action" },
        { key: "target_type", header: "Target Type" },
        { key: "target_id", header: "Target ID" },
        { key: "result", header: "Result" },
        { key: "duration_ms", header: "Duration (ms)" }
      ]
    );
  }

  function handleExportJson() {
    exportJson(`admin-audit-events-${new Date().toISOString().slice(0, 10)}.json`, events);
  }

  return (
    <PageShell
      title="Audit Trail"
      summary="Immutable request-level event stream with actor/action filters and structured detail payloads."
      actions={
        <ActionRail label="Audit actions">
          <button type="button" onClick={() => void loadAudit()}>
            Refresh
          </button>
          <button type="button" onClick={handleExportCsv} disabled={events.length === 0}>
            Export CSV
          </button>
          <button type="button" onClick={handleExportJson} disabled={events.length === 0}>
            Export JSON
          </button>
        </ActionRail>
      }
    >
      <div className="panel-inline">
        <label>
          Actor
          <input
            value={actor}
            onChange={(event) => setActor(event.target.value)}
            placeholder="email or user UUID"
          />
        </label>
        <label>
          Action
          <select value={action} onChange={(event) => setAction(event.target.value)}>
            <option value="">All actions</option>
            {actions.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
        </label>
        <label>
          Result
          <select value={result} onChange={(event) => setResult(event.target.value)}>
            <option value="">All</option>
            <option value="success">success</option>
            <option value="failure">failure</option>
          </select>
        </label>
        <label>
          From
          <input type="datetime-local" value={from} onChange={(event) => setFrom(event.target.value)} />
        </label>
        <label>
          To
          <input type="datetime-local" value={to} onChange={(event) => setTo(event.target.value)} />
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
      </div>

      <p className="helper">Last updated: {formatDateTime(lastUpdatedAt)}</p>

      <div className="grid metrics">
        <MetricSurface
          label="Matching events"
          value={loading ? "--" : total}
          detail="Server-side count for the active filter combination."
        />
        <MetricSurface
          label="Visible rows"
          value={events.length}
          detail="Rows currently loaded into the table viewport."
        />
        <MetricSurface
          label="Failures in result"
          value={failureCount}
          detail="Non-success outcomes within the visible result set."
        />
      </div>

      {error ? <StateBanner variant="error" title={error} /> : null}
      {success ? <StateBanner variant="success" title={success} /> : null}
      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading audit events"
          description="Resolving filtered ledger rows, action metadata, and request context."
        />
      ) : null}

      <SurfaceCard
        eyebrow="Ledger"
        title="Events"
        summary="Request-scoped trace records with copyable identifiers and contextual target data."
      >
        {events.length === 0 ? (
          <div className="state-table-empty event-stream-empty">
            <div className="telemetry-caption">Empty</div>
            <div className="state-table-empty-title">No audit events matched</div>
            <div className="helper">
              Adjust the actor, action, result, or time filters to widen the ledger view.
            </div>
          </div>
        ) : (
          <EventStream label="Audit event stream">
            {events.map((event) => (
              <EventStreamItem
                key={event.event_id}
                eyebrow="Audit event"
                title={event.action}
                subtitle={`${event.actor_email} · ${event.target_type} / ${event.target_id ?? "--"}`}
                badge={<span className={statusPillClass(event.result)}>{event.result}</span>}
                metadata={[
                  { label: "Occurred", value: formatDateTime(event.occurred_at) },
                  { label: "Duration", value: `${event.duration_ms} ms` },
                  { label: "Request ID", value: event.request_id }
                ]}
                summary={`Actor ${event.actor_user_id} executed ${event.action} against ${event.target_type}.`}
                action={
                  <button type="button" onClick={() => void openDetail(event.event_id)}>
                    Open detail
                  </button>
                }
              >
                <div className="event-stream-inline-actions">
                  <div className="helper">Target ID: {event.target_id ?? "--"}</div>
                  <div className="event-stream-link-row">
                    <button
                      type="button"
                      className="link-btn"
                      onClick={() => void handleCopy(event.actor_user_id, "Actor user ID")}
                    >
                      Copy actor ID
                    </button>
                    <button
                      type="button"
                      className="link-btn"
                      onClick={() => void handleCopy(event.request_id, "Request ID")}
                    >
                      Copy request ID
                    </button>
                  </div>
                </div>
              </EventStreamItem>
            ))}
          </EventStream>
        )}
      </SurfaceCard>

      {detailLoading ? (
        <StatePanel
          variant="loading"
          title="Loading selected event"
          description="Fetching request metadata and target context for the active ledger row."
          className="audit-detail-loading"
        />
      ) : null}

      <DrawerSurface
        open={Boolean(selected)}
        onClose={() => setSelected(null)}
        eyebrow="Event Detail"
        title={selected?.action ?? "Audit event"}
        summary={
          selected
            ? `${selected.actor_email} · ${selected.target_type} / ${selected.target_id ?? "--"}`
            : undefined
        }
        footer={
          <button type="button" onClick={() => setSelected(null)}>
            Close
          </button>
        }
      >
        {selected ? (
          <div className="grid overlay-detail-grid">
            <div className="helper">
              Event ID: {selected.event_id}{" "}
              <button
                type="button"
                className="link-btn"
                onClick={() => void handleCopy(selected.event_id, "Event ID")}
              >
                copy
              </button>
            </div>
            <div className="helper">Occurred at: {formatDateTime(selected.occurred_at)}</div>
            <div className="helper">
              Actor: {selected.actor_email} ({selected.actor_user_id})
            </div>
            <div className="helper">
              Action: {selected.action} · Target: {selected.target_type} / {selected.target_id ?? "--"}
            </div>
            <div className="helper">Request ID: {selected.request_id}</div>
            <pre className="overlay-json-block">{JSON.stringify(selected.metadata, null, 2)}</pre>
          </div>
        ) : null}
      </DrawerSurface>
    </PageShell>
  );
}
