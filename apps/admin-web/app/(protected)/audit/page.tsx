"use client";

import { useCallback, useEffect, useMemo, useReducer, useRef } from "react";
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

type AuditState = {
  loading: boolean;
  detailLoading: boolean;
  error: string | null;
  success: string | null;
  lastUpdatedAt: string | null;
  actions: string[];
  events: AdminAuditEventItem[];
  total: number;
  selected: AdminAuditEventItem | null;
};

type AuditAction =
  | { type: "FETCH_START" }
  | {
      type: "FETCH_SUCCESS";
      actions: string[];
      events: AdminAuditEventItem[];
      total: number;
      selected: AdminAuditEventItem | null;
    }
  | { type: "FETCH_ERROR"; error: string }
  | { type: "DETAIL_START" }
  | { type: "DETAIL_SUCCESS"; event: AdminAuditEventItem }
  | { type: "DETAIL_ERROR"; error: string }
  | { type: "CLEAR_SELECTED" }
  | { type: "SHOW_SUCCESS"; message: string }
  | { type: "CLEAR_SUCCESS" }
  | { type: "SHOW_ERROR"; error: string };

const initialAuditState: AuditState = {
  loading: true,
  detailLoading: false,
  error: null,
  success: null,
  lastUpdatedAt: null,
  actions: [],
  events: [],
  total: 0,
  selected: null
};

function auditReducer(state: AuditState, action: AuditAction): AuditState {
  switch (action.type) {
    case "FETCH_START":
      return { ...state, loading: true, error: null };
    case "FETCH_SUCCESS":
      return {
        ...state,
        loading: false,
        error: null,
        actions: action.actions,
        events: action.events,
        total: action.total,
        lastUpdatedAt: new Date().toISOString(),
        selected: action.selected
      };
    case "FETCH_ERROR":
      return { ...state, loading: false, error: action.error };
    case "DETAIL_START":
      return { ...state, detailLoading: true, error: null };
    case "DETAIL_SUCCESS":
      return { ...state, detailLoading: false, selected: action.event };
    case "DETAIL_ERROR":
      return { ...state, detailLoading: false, error: action.error };
    case "CLEAR_SELECTED":
      return { ...state, selected: null };
    case "SHOW_SUCCESS":
      return { ...state, success: action.message };
    case "CLEAR_SUCCESS":
      return { ...state, success: null };
    case "SHOW_ERROR":
      return { ...state, error: action.error };
    default:
      return state;
  }
}

export default function AuditPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const actor = getStringParam(searchParams, "actor");
  const action = getStringParam(searchParams, "action");
  const result = getStringParam(searchParams, "result");
  const from = getStringParam(searchParams, "from");
  const to = getStringParam(searchParams, "to");
  const autoRefreshSec = useMemo(
    () => getNumberParam(searchParams, "refresh", 0, 0, 60),
    [searchParams]
  );

  const [audit, dispatch] = useReducer(auditReducer, initialAuditState);
  const selectedRef = useRef(audit.selected);

  useEffect(() => {
    selectedRef.current = audit.selected;
  }, [audit.selected]);

  const updateFilters = useCallback(
    (
      updates: Partial<
        Record<
          "actor" | "action" | "result" | "from" | "to" | "refresh",
          string | number | undefined
        >
      >
    ) => {
      const refreshValue = typeof updates.refresh === "number" ? updates.refresh : autoRefreshSec;
      const nextQuery = buildQueryString(searchParams, {
        actor: updates.actor ?? actor,
        action: updates.action ?? action,
        result: updates.result ?? result,
        from: updates.from ?? from,
        to: updates.to ?? to,
        refresh: refreshValue > 0 ? refreshValue : undefined
      });
      router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
    },
    [action, actor, autoRefreshSec, from, pathname, result, router, searchParams, to]
  );

  const loadAudit = useCallback(async () => {
    const token = getAuthToken() ?? undefined;

    dispatch({ type: "FETCH_START" });

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

      const updated = selectedRef.current
        ? eventsResponse.items.find((event) => event.event_id === selectedRef.current?.event_id) ?? null
        : null;

      dispatch({
        type: "FETCH_SUCCESS",
        actions: actionsResponse.actions,
        events: eventsResponse.items,
        total: eventsResponse.total,
        selected: updated
      });
    } catch (err) {
      if (err instanceof ApiClientError) {
        dispatch({ type: "FETCH_ERROR", error: err.payload?.error || err.message });
      } else {
        dispatch({ type: "FETCH_ERROR", error: "Failed to load audit events" });
      }
    }
  }, [action, actor, from, result, to]);

  async function openDetail(eventId: string) {
    const token = getAuthToken() ?? undefined;

    dispatch({ type: "DETAIL_START" });

    try {
      const detail = await getAuditEvent(token, eventId);
      dispatch({ type: "DETAIL_SUCCESS", event: detail.event });
    } catch (err) {
      if (err instanceof ApiClientError) {
        dispatch({ type: "DETAIL_ERROR", error: err.payload?.error || err.message });
      } else {
        dispatch({ type: "DETAIL_ERROR", error: "Failed to load audit event detail" });
      }
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
    () => audit.events.filter((event) => event.result !== "success").length,
    [audit.events]
  );

  async function handleCopy(value: string, label: string) {
    try {
      await copyToClipboard(value);
      dispatch({ type: "SHOW_SUCCESS", message: `${label} copied` });
      window.setTimeout(() => dispatch({ type: "CLEAR_SUCCESS" }), 1500);
    } catch {
      dispatch({ type: "SHOW_ERROR", error: "Failed to copy value to clipboard" });
    }
  }

  function handleExportCsv() {
    exportCsv(
      `admin-audit-events-${new Date().toISOString().slice(0, 10)}.csv`,
      audit.events,
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
    exportJson(`admin-audit-events-${new Date().toISOString().slice(0, 10)}.json`, audit.events);
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
          <button type="button" onClick={handleExportCsv} disabled={audit.events.length === 0}>
            Export CSV
          </button>
          <button type="button" onClick={handleExportJson} disabled={audit.events.length === 0}>
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
            onChange={(event) => updateFilters({ actor: event.target.value })}
            placeholder="email or user UUID"
          />
        </label>
        <label>
          Action
          <select value={action} onChange={(event) => updateFilters({ action: event.target.value })}>
            <option value="">All actions</option>
            {audit.actions.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
        </label>
        <label>
          Result
          <select value={result} onChange={(event) => updateFilters({ result: event.target.value })}>
            <option value="">All</option>
            <option value="success">success</option>
            <option value="failure">failure</option>
          </select>
        </label>
        <label>
          From
          <input
            type="datetime-local"
            value={from}
            onChange={(event) => updateFilters({ from: event.target.value })}
          />
        </label>
        <label>
          To
          <input
            type="datetime-local"
            value={to}
            onChange={(event) => updateFilters({ to: event.target.value })}
          />
        </label>
        <label>
          Auto refresh
          <select
            value={autoRefreshSec}
            onChange={(event) => updateFilters({ refresh: Number.parseInt(event.target.value, 10) })}
          >
            {REFRESH_OPTIONS.map((value) => (
              <option key={value} value={value}>
                {value === 0 ? "Off" : `${value}s`}
              </option>
            ))}
          </select>
        </label>
      </div>

      <p className="helper">Last updated: {formatDateTime(audit.lastUpdatedAt)}</p>

      <div className="grid metrics">
        <MetricSurface
          label="Matching events"
          value={audit.loading ? "--" : audit.total}
          detail="Server-side count for the active filter combination."
        />
        <MetricSurface
          label="Visible rows"
          value={audit.events.length}
          detail="Rows currently loaded into the table viewport."
        />
        <MetricSurface
          label="Failures in result"
          value={failureCount}
          detail="Non-success outcomes within the visible result set."
        />
      </div>

      {audit.error ? <StateBanner variant="error" title={audit.error} /> : null}
      {audit.success ? <StateBanner variant="success" title={audit.success} /> : null}
      {audit.loading ? (
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
        {audit.events.length === 0 ? (
          <div className="state-table-empty event-stream-empty">
            <div className="telemetry-caption">Empty</div>
            <div className="state-table-empty-title">No audit events matched</div>
            <div className="helper">
              Adjust the actor, action, result, or time filters to widen the ledger view.
            </div>
          </div>
        ) : (
          <EventStream label="Audit event stream">
            {audit.events.map((event) => (
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

      {audit.detailLoading ? (
        <StatePanel
          variant="loading"
          title="Loading selected event"
          description="Fetching request metadata and target context for the active ledger row."
          className="audit-detail-loading"
        />
      ) : null}

      <DrawerSurface
        open={Boolean(audit.selected)}
        onClose={() => dispatch({ type: "CLEAR_SELECTED" })}
        eyebrow="Event Detail"
        title={audit.selected?.action ?? "Audit event"}
        summary={
          audit.selected
            ? `${audit.selected.actor_email} · ${audit.selected.target_type} / ${audit.selected.target_id ?? "--"}`
            : undefined
        }
        footer={
          <button type="button" onClick={() => dispatch({ type: "CLEAR_SELECTED" })}>
            Close
          </button>
        }
      >
        {audit.selected ? (
          <div className="grid overlay-detail-grid">
            <div className="helper">
              Event ID: {audit.selected.event_id}{" "}
              <button
                type="button"
                className="link-btn"
                onClick={() => {
                  if (!audit.selected) return;
                  void handleCopy(audit.selected.event_id, "Event ID");
                }}
              >
                copy
              </button>
            </div>
            <div className="helper">Occurred at: {formatDateTime(audit.selected.occurred_at)}</div>
            <div className="helper">
              Actor: {audit.selected.actor_email} ({audit.selected.actor_user_id})
            </div>
            <div className="helper">
              Action: {audit.selected.action} · Target: {audit.selected.target_type} / {audit.selected.target_id ?? "--"}
            </div>
            <div className="helper">Request ID: {audit.selected.request_id}</div>
            <pre className="overlay-json-block">{JSON.stringify(audit.selected.metadata, null, 2)}</pre>
          </div>
        ) : null}
      </DrawerSurface>
    </PageShell>
  );
}
