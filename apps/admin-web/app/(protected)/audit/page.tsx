"use client";

import { useEffect, useState } from "react";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getAuditActions,
  getAuditEvent,
  getAuditEvents
} from "@/lib/api";
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

export default function AuditPage() {
  const [actor, setActor] = useState("");
  const [action, setAction] = useState("");
  const [result, setResult] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");

  const [loading, setLoading] = useState(true);
  const [detailLoading, setDetailLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [actions, setActions] = useState<string[]>([]);
  const [events, setEvents] = useState<AdminAuditEventItem[]>([]);
  const [total, setTotal] = useState(0);
  const [selected, setSelected] = useState<AdminAuditEventItem | null>(null);

  async function loadAudit() {
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
  }

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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [actor, action, result, from, to]);

  return (
    <PageShell
      title="Audit Trail"
      summary="Immutable request-level event stream with actor/action filters and structured detail payloads."
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
        <button type="button" onClick={() => void loadAudit()}>
          Refresh
        </button>
      </div>

      <div className="grid metrics">
        <article className="metric">
          <div className="label">Matching events</div>
          <div className="value">{loading ? "--" : total}</div>
        </article>
        <article className="metric">
          <div className="label">Visible rows</div>
          <div className="value">{events.length}</div>
        </article>
        <article className="metric">
          <div className="label">Failures in result</div>
          <div className="value">{events.filter((event) => event.result !== "success").length}</div>
        </article>
      </div>

      {error ? <div className="error">{error}</div> : null}
      {loading ? <p className="helper">Loading audit events...</p> : null}

      <article className="panel">
        <h3>Events</h3>
        <div className="table-wrap compact">
          <table>
            <thead>
              <tr>
                <th>Time</th>
                <th>Actor</th>
                <th>Action</th>
                <th>Target</th>
                <th>Result</th>
                <th>Duration (ms)</th>
                <th>Request ID</th>
              </tr>
            </thead>
            <tbody>
              {events.map((event) => (
                <tr
                  key={event.event_id}
                  style={{ cursor: "pointer" }}
                  onClick={() => void openDetail(event.event_id)}
                >
                  <td>{formatDateTime(event.occurred_at)}</td>
                  <td>
                    <div className="table-primary">{event.actor_email}</div>
                    <div className="helper">{event.actor_user_id}</div>
                  </td>
                  <td>{event.action}</td>
                  <td>
                    <div className="table-primary">{event.target_type}</div>
                    <div className="helper">{event.target_id ?? "--"}</div>
                  </td>
                  <td>
                    <span className={`pill ${event.result === "success" ? "ok" : "danger"}`}>
                      {event.result}
                    </span>
                  </td>
                  <td>{event.duration_ms}</td>
                  <td className="helper">{event.request_id}</td>
                </tr>
              ))}
              {events.length === 0 ? (
                <tr>
                  <td colSpan={7}>
                    <p className="helper">No audit events matched these filters.</p>
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
      </article>

      {selected ? (
        <article className="panel">
          <h3>Event Detail</h3>
          <div className="grid" style={{ gap: "0.5rem" }}>
            <div className="helper">Event ID: {selected.event_id}</div>
            <div className="helper">Occurred at: {formatDateTime(selected.occurred_at)}</div>
            <div className="helper">
              Actor: {selected.actor_email} ({selected.actor_user_id})
            </div>
            <div className="helper">
              Action: {selected.action} · Target: {selected.target_type} / {selected.target_id ?? "--"}
            </div>
            <div className="helper">Request ID: {selected.request_id}</div>
            <pre
              style={{
                margin: 0,
                border: "1px solid var(--line)",
                borderRadius: "10px",
                background: "var(--bg-elevated)",
                padding: "0.75rem",
                overflow: "auto",
                fontSize: "0.8rem"
              }}
            >
              {JSON.stringify(selected.metadata, null, 2)}
            </pre>
          </div>
        </article>
      ) : null}

      {detailLoading ? <p className="helper">Loading selected event detail...</p> : null}
    </PageShell>
  );
}
