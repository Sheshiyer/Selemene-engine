"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getAdminUsers,
  updateAdminUserRoles,
  updateAdminUserState,
  updateAdminUserTier
} from "@/lib/api";
import { copyToClipboard, exportCsv, exportJson } from "@/lib/export";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getStringParam } from "@/lib/url-query";
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
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const [query, setQuery] = useState(() => getStringParam(searchParams, "query"));
  const [tierFilter, setTierFilter] = useState(() => getStringParam(searchParams, "tier"));
  const [stateFilter, setStateFilter] = useState(() => getStringParam(searchParams, "state"));

  const [loading, setLoading] = useState(true);
  const [submittingId, setSubmittingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [users, setUsers] = useState<AdminUserItem[]>([]);
  const [total, setTotal] = useState(0);
  const [tierDrafts, setTierDrafts] = useState<Record<string, string>>({});
  const [roleDrafts, setRoleDrafts] = useState<Record<string, string>>({});

  useEffect(() => {
    setQuery(getStringParam(searchParams, "query"));
    setTierFilter(getStringParam(searchParams, "tier"));
    setStateFilter(getStringParam(searchParams, "state"));
  }, [searchParams]);

  useEffect(() => {
    const nextQuery = buildQueryString(searchParams, {
      query,
      tier: tierFilter,
      state: stateFilter
    });
    router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
  }, [pathname, query, router, searchParams, stateFilter, tierFilter]);

  const activeCount = useMemo(
    () => users.filter((user) => user.state === "active").length,
    [users]
  );

  const loadUsers = useCallback(async () => {
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
  }, [query, stateFilter, tierFilter]);

  useEffect(() => {
    void loadUsers();
  }, [loadUsers]);

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
      `admin-users-${new Date().toISOString().slice(0, 10)}.csv`,
      users,
      [
        { key: "id", header: "User ID" },
        { key: "email", header: "Email" },
        { key: "full_name", header: "Full Name" },
        { key: "tier", header: "Tier" },
        { key: "state", header: "State" },
        { key: "active_key_count", header: "Active Key Count" },
        { key: "last_login_at", header: "Last Login" }
      ]
    );
  }

  function handleExportJson() {
    exportJson(`admin-users-${new Date().toISOString().slice(0, 10)}.json`, users);
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
        <button type="button" onClick={handleExportCsv} disabled={users.length === 0}>
          Export CSV
        </button>
        <button type="button" onClick={handleExportJson} disabled={users.length === 0}>
          Export JSON
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

      {error ? <StateBanner variant="error" title={error} /> : null}
      {success ? <StateBanner variant="success" title={success} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading users"
          description="Resolving account inventory, tier drafts, role drafts, and filter-scoped user state."
        />
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
                    <button
                      type="button"
                      className="link-btn"
                      onClick={() => void handleCopy(user.id, "User ID")}
                    >
                      ID: {user.id}
                    </button>
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
                    <span className={statusPillClass(user.state)}>{user.state}</span>
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
                <TableEmptyStateRow
                  colSpan={6}
                  title="No users matched"
                  description="Try broadening the current search, tier, or state filters."
                />
              ) : null}
            </tbody>
          </table>
        </div>
      )}
    </PageShell>
  );
}
