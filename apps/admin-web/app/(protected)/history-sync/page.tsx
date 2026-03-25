"use client";

import { useEffect, useState } from "react";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getHistorySyncDevices,
  getHistorySyncEvents,
  getHistorySyncUsers
} from "@/lib/api";
import { statusPillClass } from "@/lib/status";
import type {
  AdminHistorySyncDeviceItem,
  AdminHistorySyncEventItem,
  AdminHistorySyncUserItem
} from "@/types/admin";

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

export default function HistorySyncPage() {
  const [eventStatus, setEventStatus] = useState("");

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [users, setUsers] = useState<AdminHistorySyncUserItem[]>([]);
  const [devices, setDevices] = useState<AdminHistorySyncDeviceItem[]>([]);
  const [events, setEvents] = useState<AdminHistorySyncEventItem[]>([]);

  async function loadHistorySync() {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const [usersResponse, devicesResponse, eventsResponse] = await Promise.all([
        getHistorySyncUsers(token, { limit: 50, offset: 0 }),
        getHistorySyncDevices(token, { limit: 50, offset: 0 }),
        getHistorySyncEvents(token, {
          status: eventStatus || undefined,
          limit: 50,
          offset: 0
        })
      ]);

      setUsers(usersResponse.items);
      setDevices(devicesResponse.items);
      setEvents(eventsResponse.items);
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to load history sync views");
      }
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadHistorySync();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [eventStatus]);

  return (
    <PageShell
      title="History Sync"
      summary="Track drift across users, key-backed devices, and ingestion events before repair actions."
    >
      <div className="panel-inline">
        <label>
          Event status
          <select value={eventStatus} onChange={(event) => setEventStatus(event.target.value)}>
            <option value="">All</option>
            <option value="success">success</option>
            <option value="failure">failure</option>
          </select>
        </label>
        <button type="button" onClick={() => void loadHistorySync()}>
          Refresh
        </button>
      </div>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading history sync data"
          description="Resolving user drift, device posture, and recent event ingest status."
        />
      ) : null}

      <article className="panel">
        <h3>User Drift</h3>
        <div className="table-wrap compact">
          <table>
            <thead>
              <tr>
                <th>User</th>
                <th>Status</th>
                <th>Readings</th>
                <th>Usage events</th>
                <th>Drift</th>
                <th>Last event</th>
              </tr>
            </thead>
            <tbody>
              {users.map((user) => (
                <tr key={user.user_id}>
                  <td>
                    <div className="table-primary">{user.email}</div>
                    <div className="helper">{user.user_id}</div>
                  </td>
                  <td>
                    <span className={statusPillClass(user.status)}>{user.status}</span>
                  </td>
                  <td>{user.readings_count}</td>
                  <td>{user.usage_events_count}</td>
                  <td>{user.drift_count}</td>
                  <td>{formatDateTime(user.last_event_at)}</td>
                </tr>
              ))}
              {users.length === 0 ? (
                <TableEmptyStateRow
                  colSpan={6}
                  title="No user sync records"
                  description="User-level drift rows will appear once sync telemetry is ingested."
                />
              ) : null}
            </tbody>
          </table>
        </div>
      </article>

      <article className="panel">
        <h3>Device Sources</h3>
        <div className="table-wrap compact">
          <table>
            <thead>
              <tr>
                <th>Device ID</th>
                <th>User</th>
                <th>Status</th>
                <th>Tier</th>
                <th>Permissions</th>
                <th>Last seen</th>
              </tr>
            </thead>
            <tbody>
              {devices.map((device) => (
                <tr key={device.device_id}>
                  <td>
                    <div className="table-primary">{device.device_id}</div>
                    <div className="helper">Created {formatDateTime(device.created_at)}</div>
                  </td>
                  <td>
                    <div className="table-primary">{device.user_email}</div>
                    <div className="helper">{device.user_id}</div>
                  </td>
                  <td>
                    <span className={statusPillClass(device.status)}>{device.status}</span>
                  </td>
                  <td>{device.tier}</td>
                  <td>{device.permission_count}</td>
                  <td>{formatDateTime(device.last_seen_at)}</td>
                </tr>
              ))}
              {devices.length === 0 ? (
                <TableEmptyStateRow
                  colSpan={6}
                  title="No device sync records"
                  description="No key-backed device sources are currently reporting sync metadata."
                />
              ) : null}
            </tbody>
          </table>
        </div>
      </article>

      <article className="panel">
        <h3>Recent Events</h3>
        <div className="table-wrap compact">
          <table>
            <thead>
              <tr>
                <th>Time</th>
                <th>User</th>
                <th>Engine</th>
                <th>Workflow</th>
                <th>Status</th>
                <th>Duration (ms)</th>
              </tr>
            </thead>
            <tbody>
              {events.map((event) => (
                <tr key={event.event_id}>
                  <td>{formatDateTime(event.occurred_at)}</td>
                  <td>
                    <div className="table-primary">{event.user_email}</div>
                    <div className="helper">{event.user_id}</div>
                  </td>
                  <td>{event.engine_id ?? "--"}</td>
                  <td>{event.workflow_id ?? "--"}</td>
                  <td>
                    <span className={statusPillClass(event.status)}>{event.status}</span>
                  </td>
                  <td>{event.duration_ms}</td>
                </tr>
              ))}
              {events.length === 0 ? (
                <TableEmptyStateRow
                  colSpan={6}
                  title="No history sync events"
                  description="Recent sync events will appear here after ingestion or repair activity."
                />
              ) : null}
            </tbody>
          </table>
        </div>
      </article>
    </PageShell>
  );
}
