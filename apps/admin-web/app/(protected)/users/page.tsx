"use client";

import { useEffect, useMemo, useState } from "react";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getAdminUsers,
  updateAdminUserRoles,
  updateAdminUserState,
  updateAdminUserTier
} from "@/lib/api";
import type { AdminUserItem } from "@/types/admin";

const ROLE_OPTIONS = ["viewer", "support", "admin", "platform-admin"];

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

export default function UsersPage() {
  const [query, setQuery] = useState("");
  const [tierFilter, setTierFilter] = useState("");
  const [stateFilter, setStateFilter] = useState("");

  const [loading, setLoading] = useState(true);
  const [submittingId, setSubmittingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [users, setUsers] = useState<AdminUserItem[]>([]);
  const [total, setTotal] = useState(0);
  const [tierDrafts, setTierDrafts] = useState<Record<string, string>>({});
  const [roleDrafts, setRoleDrafts] = useState<Record<string, string>>({});

  const activeCount = useMemo(
    () => users.filter((user) => user.state === "active").length,
    [users]
  );

  async function loadUsers() {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const response = await getAdminUsers(token, {
        query: query || undefined,
        tier: tierFilter || undefined,
        state: stateFilter || undefined,
        limit: 100,
        offset: 0
      });

      setUsers(response.items);
      setTotal(response.total);

      setTierDrafts((prev) => {
        const next = { ...prev };
        for (const user of response.items) {
          if (!next[user.id]) {
            next[user.id] = user.tier;
          }
        }
        return next;
      });

      setRoleDrafts((prev) => {
        const next = { ...prev };
        for (const user of response.items) {
          if (!next[user.id]) {
            next[user.id] = "viewer";
          }
        }
        return next;
      });
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to load users");
      }
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadUsers();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query, tierFilter, stateFilter]);

  async function handleToggleLock(user: AdminUserItem) {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      return;
    }

    setSubmittingId(user.id);
    setError(null);
    setSuccess(null);

    try {
      await updateAdminUserState(token, user.id, {
        state: user.state === "locked" ? "active" : "locked",
        lock_minutes: 60
      });
      setSuccess(`Updated state for ${user.email}`);
      await loadUsers();
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to update user state");
      }
    } finally {
      setSubmittingId(null);
    }
  }

  async function handleTierUpdate(userId: string) {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      return;
    }

    const tier = tierDrafts[userId]?.trim();
    if (!tier) {
      setError("Tier cannot be empty");
      return;
    }

    setSubmittingId(userId);
    setError(null);
    setSuccess(null);

    try {
      await updateAdminUserTier(token, userId, { tier });
      setSuccess("Updated user tier");
      await loadUsers();
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to update user tier");
      }
    } finally {
      setSubmittingId(null);
    }
  }

  async function handleRoleUpdate(userId: string) {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      return;
    }

    const role = roleDrafts[userId];
    if (!role) {
      setError("Select a role first");
      return;
    }

    setSubmittingId(userId);
    setError(null);
    setSuccess(null);

    try {
      await updateAdminUserRoles(token, userId, { roles: [role] });
      setSuccess("Updated user role policy");
      await loadUsers();
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to update user roles");
      }
    } finally {
      setSubmittingId(null);
    }
  }

  return (
    <PageShell
      title="Users"
      summary="Search and inspect user accounts, then apply account state, tier, and role actions."
    >
      <div className="panel-inline">
        <label>
          Search
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="email or name"
          />
        </label>
        <label>
          Tier
          <input
            value={tierFilter}
            onChange={(event) => setTierFilter(event.target.value)}
            placeholder="free / premium / enterprise"
          />
        </label>
        <label>
          State
          <select value={stateFilter} onChange={(event) => setStateFilter(event.target.value)}>
            <option value="">All</option>
            <option value="active">Active</option>
            <option value="locked">Locked</option>
          </select>
        </label>
        <button type="button" onClick={() => void loadUsers()}>
          Refresh
        </button>
      </div>

      <div className="grid metrics">
        <article className="metric">
          <div className="label">Total users</div>
          <div className="value">{total}</div>
        </article>
        <article className="metric">
          <div className="label">Visible users</div>
          <div className="value">{users.length}</div>
        </article>
        <article className="metric">
          <div className="label">Active in result</div>
          <div className="value">{activeCount}</div>
        </article>
      </div>

      {error ? <div className="error">{error}</div> : null}
      {success ? <div className="success">{success}</div> : null}

      {loading ? (
        <p className="helper">Loading users...</p>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>User</th>
                <th>Tier</th>
                <th>Status</th>
                <th>Permissions</th>
                <th>Last login</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {users.map((user) => (
                <tr key={user.id}>
                  <td>
                    <div className="table-primary">{user.email}</div>
                    <div className="helper">{user.full_name || "Unnamed"}</div>
                    <div className="helper">ID: {user.id}</div>
                  </td>
                  <td>
                    <input
                      className="cell-input"
                      value={tierDrafts[user.id] ?? user.tier}
                      onChange={(event) =>
                        setTierDrafts((prev) => ({ ...prev, [user.id]: event.target.value }))
                      }
                    />
                  </td>
                  <td>
                    <span className={`pill ${user.state === "locked" ? "danger" : "ok"}`}>
                      {user.state}
                    </span>
                  </td>
                  <td>
                    <div className="helper">{user.permissions.join(", ") || "basic:access"}</div>
                    <div className="helper">Active keys: {user.active_key_count}</div>
                  </td>
                  <td>{formatDateTime(user.last_login_at)}</td>
                  <td>
                    <div className="action-stack">
                      <button
                        type="button"
                        disabled={submittingId === user.id}
                        onClick={() => void handleToggleLock(user)}
                      >
                        {user.state === "locked" ? "Unlock" : "Lock"}
                      </button>
                      <button
                        type="button"
                        disabled={submittingId === user.id}
                        onClick={() => void handleTierUpdate(user.id)}
                      >
                        Save Tier
                      </button>
                      <div className="action-row">
                        <select
                          value={roleDrafts[user.id] ?? "viewer"}
                          onChange={(event) =>
                            setRoleDrafts((prev) => ({ ...prev, [user.id]: event.target.value }))
                          }
                        >
                          {ROLE_OPTIONS.map((role) => (
                            <option key={role} value={role}>
                              {role}
                            </option>
                          ))}
                        </select>
                        <button
                          type="button"
                          disabled={submittingId === user.id}
                          onClick={() => void handleRoleUpdate(user.id)}
                        >
                          Apply Role
                        </button>
                      </div>
                    </div>
                  </td>
                </tr>
              ))}
              {users.length === 0 ? (
                <tr>
                  <td colSpan={6}>
                    <p className="helper">No users matched this filter.</p>
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
      )}
    </PageShell>
  );
}
