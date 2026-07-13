"use client";

import { useEffect, useMemo, useState, type KeyboardEvent as ReactKeyboardEvent } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { MetricSurface, SurfaceCard } from "@/components/admin-primitives";
import { BulkActionBar, useBulkSelection } from "@/components/bulk-actions";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { DrawerSurface, ModalSurface } from "@/components/overlay-surface";
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

const ROLE_OPTIONS = ["viewer", "support", "admin", "platform-admin"] as const;

type RoleOption = (typeof ROLE_OPTIONS)[number];

type PendingUserAction =
  | {
      kind: "state";
      userId: string;
      title: string;
      description: string;
      confirmLabel: string;
      nextState: "active" | "locked";
      danger?: boolean;
    }
  | {
      kind: "tier";
      userId: string;
      title: string;
      description: string;
      confirmLabel: string;
      nextTier: string;
    }
  | {
      kind: "role";
      userId: string;
      title: string;
      description: string;
      confirmLabel: string;
      nextRole: RoleOption;
    };

type BulkUserAction = {
  kind: "lock" | "unlock";
  userIds: string[];
};

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

function inferRoleFromPermissions(permissions: string[]): RoleOption {
  if (
    permissions.includes("admin:*") ||
    permissions.includes("admin:users:roles:update")
  ) {
    return "platform-admin";
  }

  if (
    permissions.includes("admin:keys:create") ||
    permissions.includes("admin:keys:rotate") ||
    permissions.includes("admin:users:tier:update")
  ) {
    return "admin";
  }

  if (
    permissions.includes("admin:users:list") ||
    permissions.includes("admin:users:read") ||
    permissions.includes("admin:history-sync:read")
  ) {
    return "support";
  }

  return "viewer";
}

async function fetchUsersData(query: string, tierFilter: string, stateFilter: string) {
  const token = getAuthToken() ?? undefined;

  const response = await getAdminUsers(token, {
    query: query || undefined,
    tier: tierFilter || undefined,
    state: stateFilter || undefined,
    limit: 100,
    offset: 0
  });

  return response;
}

function onRowKeyboardOpen(event: ReactKeyboardEvent<HTMLTableRowElement>, onOpen: () => void) {
  if (event.key !== "Enter" && event.key !== " ") {
    return;
  }

  event.preventDefault();
  onOpen();
}

export default function UsersPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const query = getStringParam(searchParams, "query");
  const tierFilter = getStringParam(searchParams, "tier");
  const stateFilter = getStringParam(searchParams, "state");

  const [loading, setLoading] = useState(true);
  const [submittingId, setSubmittingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [users, setUsers] = useState<AdminUserItem[]>([]);
  const [total, setTotal] = useState(0);
  const [tierDrafts, setTierDrafts] = useState<Record<string, string>>({});
  const [roleDrafts, setRoleDrafts] = useState<Record<string, RoleOption>>({});
  const [selectedUserId, setSelectedUserId] = useState<string | null>(null);
  const [pendingAction, setPendingAction] = useState<PendingUserAction | null>(null);
  const [bulkAction, setBulkAction] = useState<BulkUserAction | null>(null);

  function updateParam(key: string, value: string) {
    const nextQuery = buildQueryString(searchParams, { [key]: value || undefined });
    router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
  }

  const activeCount = useMemo(
    () => users.filter((user) => user.state === "active").length,
    [users]
  );

  const selectedUser = useMemo(
    () => users.find((user) => user.id === selectedUserId) ?? null,
    [selectedUserId, users]
  );
  const bulkSelection = useBulkSelection(users.map((user) => user.id));
  const selectedUsers = useMemo(
    () => users.filter((user) => bulkSelection.selectedSet.has(user.id)),
    [bulkSelection.selectedSet, users]
  );
  const selectedActiveUsers = useMemo(
    () => selectedUsers.filter((user) => user.state !== "locked"),
    [selectedUsers]
  );
  const selectedLockedUsers = useMemo(
    () => selectedUsers.filter((user) => user.state === "locked"),
    [selectedUsers]
  );

  async function loadUsers() {
    setLoading(true);
    setError(null);

    try {
      const response = await fetchUsersData(query, tierFilter, stateFilter);

      setUsers(response.items);
      setTotal(response.total);

      setTierDrafts((prev) => {
        const next = { ...prev };
        for (const user of response.items) {
          next[user.id] = next[user.id] ?? user.tier;
        }
        return next;
      });

      setRoleDrafts((prev) => {
        const next = { ...prev };
        for (const user of response.items) {
          next[user.id] = next[user.id] ?? inferRoleFromPermissions(user.permissions);
        }
        return next;
      });

      if (selectedUserId && !response.items.some((user) => user.id === selectedUserId)) {
        setSelectedUserId(null);
        setPendingAction(null);
      }
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("Failed to load users");
      }
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    let cancelled = false;

    async function run() {
      setLoading(true);
      setError(null);

      try {
        const response = await fetchUsersData(query, tierFilter, stateFilter);

        if (!cancelled) {
          setUsers(response.items);
          setTotal(response.total);

          setTierDrafts((prev) => {
            const next = { ...prev };
            for (const user of response.items) {
              next[user.id] = next[user.id] ?? user.tier;
            }
            return next;
          });

          setRoleDrafts((prev) => {
            const next = { ...prev };
            for (const user of response.items) {
              next[user.id] = next[user.id] ?? inferRoleFromPermissions(user.permissions);
            }
            return next;
          });

          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          if (err instanceof ApiClientError) {
            setError(err.payload?.error || err.message);
          } else if (err instanceof Error) {
            setError(err.message);
          } else {
            setError("Failed to load users");
          }
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    void run();

    return () => {
      cancelled = true;
    };
  }, [query, stateFilter, tierFilter]);

  async function handleCopy(value: string, label: string) {
    try {
      await copyToClipboard(value);
      setSuccess(`${label} copied`);
      window.setTimeout(() => setSuccess(null), 1500);
    } catch {
      setError("Failed to copy value to clipboard");
    }
  }

  function openUserDetail(userId: string) {
    setSelectedUserId(userId);
    setPendingAction(null);
    setError(null);
    setSuccess(null);
  }

  function closeUserDetail() {
    setSelectedUserId(null);
    setPendingAction(null);
  }

  function queueBulkStateChange(kind: "lock" | "unlock") {
    const targetIds =
      kind === "lock"
        ? selectedActiveUsers.map((user) => user.id)
        : selectedLockedUsers.map((user) => user.id);

    if (targetIds.length === 0) {
      setError(
        kind === "lock"
          ? "Select at least one active user to lock."
          : "Select at least one locked user to unlock."
      );
      return;
    }

    setBulkAction({ kind, userIds: targetIds });
    setError(null);
    setSuccess(null);
  }

  function queueStateChange(user: AdminUserItem) {
    const nextState = user.state === "locked" ? "active" : "locked";
    setPendingAction({
      kind: "state",
      userId: user.id,
      title: nextState === "locked" ? `Lock ${user.email}` : `Unlock ${user.email}`,
      description:
        nextState === "locked"
          ? "This will lock the account for 60 minutes and interrupt current admin access until the lock window clears."
          : "This will restore the account to an active state immediately.",
      confirmLabel: nextState === "locked" ? "Confirm lock" : "Confirm unlock",
      nextState,
      danger: nextState === "locked"
    });
  }

  function queueTierUpdate(user: AdminUserItem) {
    const nextTier = tierDrafts[user.id]?.trim();
    if (!nextTier) {
      setError("Tier cannot be empty");
      return;
    }

    setPendingAction({
      kind: "tier",
      userId: user.id,
      title: `Update tier for ${user.email}`,
      description: `This will change the user tier from ${user.tier} to ${nextTier}.`,
      confirmLabel: "Confirm tier update",
      nextTier
    });
  }

  function queueRoleUpdate(user: AdminUserItem) {
    const nextRole = roleDrafts[user.id];
    if (!nextRole) {
      setError("Select a role first");
      return;
    }

    setPendingAction({
      kind: "role",
      userId: user.id,
      title: `Update role policy for ${user.email}`,
      description: `This will apply the ${nextRole} role policy to the selected user.`,
      confirmLabel: "Confirm role update",
      nextRole
    });
  }

  async function confirmPendingAction() {
    if (!pendingAction) {
      return;
    }

    const token = getAuthToken() ?? undefined;

    setSubmittingId(pendingAction.userId);
    setError(null);
    setSuccess(null);

    try {
      if (pendingAction.kind === "state") {
        await updateAdminUserState(token, pendingAction.userId, {
          state: pendingAction.nextState,
          lock_minutes: pendingAction.nextState === "locked" ? 60 : 0
        });
        setSuccess(
          pendingAction.nextState === "locked"
            ? "User account locked"
            : "User account unlocked"
        );
      }

      if (pendingAction.kind === "tier") {
        await updateAdminUserTier(token, pendingAction.userId, {
          tier: pendingAction.nextTier
        });
        setSuccess("Updated user tier");
      }

      if (pendingAction.kind === "role") {
        await updateAdminUserRoles(token, pendingAction.userId, {
          roles: [pendingAction.nextRole]
        });
        setSuccess("Updated user role policy");
      }

      setPendingAction(null);
      await loadUsers();
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to update user");
      }
    } finally {
      setSubmittingId(null);
    }
  }

  async function confirmBulkAction() {
    if (!bulkAction) {
      return;
    }

    const token = getAuthToken() ?? undefined;

    const activeBulkAction = bulkAction;
    setSubmittingId(`bulk-users-${activeBulkAction.kind}`);
    setError(null);
    setSuccess(null);

    try {
      const results = await Promise.allSettled(
        activeBulkAction.userIds.map((userId) =>
          updateAdminUserState(token, userId, {
            state: activeBulkAction.kind === "lock" ? "locked" : "active",
            lock_minutes: activeBulkAction.kind === "lock" ? 60 : 0
          })
        )
      );

      const failed = results.filter((result) => result.status === "rejected");
      const succeeded = results.length - failed.length;

      if (succeeded > 0) {
        setSuccess(
          activeBulkAction.kind === "lock"
            ? `Locked ${succeeded} user account${succeeded === 1 ? "" : "s"}.`
            : `Unlocked ${succeeded} user account${succeeded === 1 ? "" : "s"}.`
        );
      }

      if (failed.length > 0) {
        const firstFailure = failed[0];
        const baseMessage =
          firstFailure.reason instanceof ApiClientError
            ? firstFailure.reason.payload?.error || firstFailure.reason.message
            : "Some bulk user updates failed";
        setError(
          failed.length === results.length
            ? baseMessage
            : `${baseMessage} (${failed.length} failed)`
        );
      }

      setBulkAction(null);
      bulkSelection.clear();
      await loadUsers();
    } finally {
      setSubmittingId(null);
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
      summary="Search and inspect user accounts, then drill into a single management surface for state, tier, and role changes."
    >
      <div className="panel-inline">
        <label>
          Search
          <input
            value={query}
            onChange={(event) => updateParam("query", event.target.value)}
            placeholder="email or name"
          />
        </label>
        <label>
          Tier
          <input
            value={tierFilter}
            onChange={(event) => updateParam("tier", event.target.value)}
            placeholder="free / premium / enterprise"
          />
        </label>
        <label>
          State
          <select value={stateFilter} onChange={(event) => updateParam("state", event.target.value)}>
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
        <>
          <BulkActionBar
            itemLabel="users"
            selectedCount={bulkSelection.selectedCount}
            visibleCount={users.length}
            allVisibleSelected={bulkSelection.allVisibleSelected}
            onToggleVisible={bulkSelection.toggleVisible}
            onClear={bulkSelection.clear}
          >
            <button
              type="button"
              disabled={selectedActiveUsers.length === 0 || submittingId !== null}
              onClick={() => queueBulkStateChange("lock")}
            >
              Lock selected
            </button>
            <button
              type="button"
              disabled={selectedLockedUsers.length === 0 || submittingId !== null}
              onClick={() => queueBulkStateChange("unlock")}
            >
              Unlock selected
            </button>
          </BulkActionBar>

          <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th className="table-select-column">
                  <input
                    type="checkbox"
                    aria-label={bulkSelection.allVisibleSelected ? "Clear visible users" : "Select visible users"}
                    checked={bulkSelection.allVisibleSelected}
                    onChange={() => bulkSelection.toggleVisible()}
                    disabled={users.length === 0}
                  />
                </th>
                <th>User</th>
                <th>Tier</th>
                <th>Status</th>
                <th>Permissions</th>
                <th>Last login</th>
                <th>Manage</th>
              </tr>
            </thead>
            <tbody>
              {users.map((user) => (
                <tr
                  key={user.id}
                  className="clickable-row"
                  tabIndex={0}
                  onClick={() => openUserDetail(user.id)}
                  onKeyDown={(event) => onRowKeyboardOpen(event, () => openUserDetail(user.id))}
                >
                  <td className="table-select-cell">
                    <input
                      type="checkbox"
                      aria-label={`Select ${user.email}`}
                      checked={bulkSelection.selectedSet.has(user.id)}
                      onChange={() => bulkSelection.toggle(user.id)}
                      onClick={(event) => event.stopPropagation()}
                      onKeyDown={(event) => event.stopPropagation()}
                    />
                  </td>
                  <td>
                    <div className="table-primary">{user.email}</div>
                    <div className="helper">{user.full_name || "Unnamed"}</div>
                    <div className="table-secondary-row">
                      <button
                        type="button"
                        className="link-btn"
                        onClick={(event) => {
                          event.stopPropagation();
                          void handleCopy(user.id, "User ID");
                        }}
                      >
                        ID: {user.id}
                      </button>
                      <span className="helper">XP {user.experience_points}</span>
                      <span className="helper">Consciousness {user.consciousness_level}</span>
                    </div>
                  </td>
                  <td>
                    <div className="table-primary">{user.tier}</div>
                    <div className="helper">Draft: {tierDrafts[user.id] ?? user.tier}</div>
                  </td>
                  <td>
                    <span className={statusPillClass(user.state)}>{user.state}</span>
                    <div className="helper">Failed attempts: {user.failed_login_attempts}</div>
                  </td>
                  <td>
                    <div className="table-chip-row">
                      {user.permissions.length > 0 ? (
                        user.permissions.slice(0, 3).map((permission) => (
                          <span key={permission} className="permission-chip">
                            {permission}
                          </span>
                        ))
                      ) : (
                        <span className="permission-chip muted">basic:access</span>
                      )}
                    </div>
                    <div className="helper">
                      {user.permissions.length > 3
                        ? `${user.permissions.length} permissions total`
                        : "Permission set visible in drawer"}
                    </div>
                    <div className="helper">Active keys: {user.active_key_count}</div>
                  </td>
                  <td>{formatDateTime(user.last_login_at)}</td>
                  <td>
                    <button
                      type="button"
                      onClick={(event) => {
                        event.stopPropagation();
                        openUserDetail(user.id);
                      }}
                    >
                      Open drawer
                    </button>
                  </td>
                </tr>
              ))}
              {users.length === 0 ? (
                <TableEmptyStateRow
                  colSpan={7}
                  title="No users matched"
                  description="Try broadening the current search, tier, or state filters."
                />
              ) : null}
            </tbody>
          </table>
          </div>
        </>
      )}

      <ModalSurface
        open={Boolean(bulkAction)}
        onClose={() => setBulkAction(null)}
        eyebrow="Bulk account state"
        title={
          bulkAction?.kind === "lock"
            ? "Lock selected users"
            : "Unlock selected users"
        }
        summary={
          bulkAction
            ? `${
                bulkAction.kind === "lock" ? "Lock" : "Unlock"
              } ${bulkAction.userIds.length} selected user account${bulkAction.userIds.length === 1 ? "" : "s"} with one confirmation.`
            : undefined
        }
        footer={
          <>
            <button type="button" onClick={() => setBulkAction(null)}>
              Cancel
            </button>
            <button
              type="button"
              className={bulkAction?.kind === "lock" ? "btn-danger" : "btn-primary"}
              disabled={!bulkAction || submittingId !== null}
              onClick={() => void confirmBulkAction()}
            >
              {submittingId !== null
                ? "Applying..."
                : bulkAction?.kind === "lock"
                  ? "Confirm lock"
                  : "Confirm unlock"}
            </button>
          </>
        }
      >
        {bulkAction ? (
          <p className="helper">
            {bulkAction.kind === "lock"
              ? "Each selected account will enter a 60 minute lock window."
              : "Each selected locked account will return to active access immediately."}
          </p>
        ) : null}
      </ModalSurface>

      <DrawerSurface
        open={Boolean(selectedUser)}
        onClose={closeUserDetail}
        eyebrow="User Detail"
        title={selectedUser?.email ?? "User account"}
        summary={
          selectedUser
            ? `${selectedUser.full_name || "Unnamed"} · ${selectedUser.tier} · ${selectedUser.state}`
            : undefined
        }
        footer={
          <button type="button" onClick={closeUserDetail}>
            Close
          </button>
        }
      >
        {selectedUser ? (
          <div className="user-drawer-grid">
            <SurfaceCard
              eyebrow="Identity"
              title={selectedUser.full_name || "Unnamed user"}
              summary="Stable user context, identifiers, and account timestamps for the selected row."
              className="user-drawer-section"
            >
              <div className="grid overlay-detail-grid">
                <div className="helper">
                  User ID: {selectedUser.id}{" "}
                  <button
                    type="button"
                    className="link-btn"
                    onClick={() => void handleCopy(selectedUser.id, "User ID")}
                  >
                    copy
                  </button>
                </div>
                <div className="helper">Created at: {formatDateTime(selectedUser.created_at)}</div>
                <div className="helper">Updated at: {formatDateTime(selectedUser.updated_at)}</div>
                <div className="helper">Last login: {formatDateTime(selectedUser.last_login_at)}</div>
                <div className="helper">Locked until: {formatDateTime(selectedUser.locked_until)}</div>
              </div>
            </SurfaceCard>

            <div className="grid metrics user-drawer-metrics">
              <MetricSurface
                label="Active keys"
                value={selectedUser.active_key_count}
                detail="Current active API key count for the selected user."
              />
              <MetricSurface
                label="Failed attempts"
                value={selectedUser.failed_login_attempts}
                detail="Observed login failures in the current account state."
              />
              <MetricSurface
                label="Consciousness"
                value={selectedUser.consciousness_level}
                detail="Current profile-level consciousness score."
              />
            </div>

            {pendingAction ? (
              <SurfaceCard
                eyebrow="Confirmation"
                title={pendingAction.title}
                summary={pendingAction.description}
                className="user-drawer-section user-confirm-card"
              >
                <div className="modal-actions user-confirm-actions">
                  <button
                    type="button"
                    onClick={() => setPendingAction(null)}
                    disabled={submittingId === pendingAction.userId}
                  >
                    Cancel
                  </button>
                  <button
                    type="button"
                    className={pendingAction.kind === "state" && pendingAction.danger ? "btn-danger" : "btn-primary"}
                    onClick={() => void confirmPendingAction()}
                    disabled={submittingId === pendingAction.userId}
                  >
                    {submittingId === pendingAction.userId ? "Applying..." : pendingAction.confirmLabel}
                  </button>
                </div>
              </SurfaceCard>
            ) : (
              <div className="helper user-confirm-hint">
                Stage any action below, then confirm it in this drawer before the mutation runs.
              </div>
            )}

            <SurfaceCard
              eyebrow="Account State"
              title="Lock window and state controls"
              summary="Use the state cluster for reversible account lock and recovery actions."
              className="user-drawer-section"
            >
              <div className="action-stack user-drawer-action-stack">
                <div className="helper">
                  Current state: <span className={statusPillClass(selectedUser.state)}>{selectedUser.state}</span>
                </div>
                <div className="helper">
                  {selectedUser.state === "locked"
                    ? `This account stays locked until ${formatDateTime(selectedUser.locked_until)}.`
                    : "Locking applies a 60 minute recovery window."}
                </div>
                <button
                  type="button"
                  disabled={submittingId === selectedUser.id}
                  onClick={() => queueStateChange(selectedUser)}
                >
                  {selectedUser.state === "locked" ? "Unlock account" : "Lock account"}
                </button>
              </div>
            </SurfaceCard>

            <SurfaceCard
              eyebrow="Commercial Tier"
              title="Plan and entitlement tier"
              summary="Update the commercial tier draft, then confirm the change before it is applied."
              className="user-drawer-section"
            >
              <div className="action-stack user-drawer-action-stack">
                <label className="user-drawer-field">
                  <span>Tier draft</span>
                  <input
                    className="cell-input"
                    value={tierDrafts[selectedUser.id] ?? selectedUser.tier}
                    onChange={(event) =>
                      setTierDrafts((prev) => ({ ...prev, [selectedUser.id]: event.target.value }))
                    }
                  />
                </label>
                <div className="helper">Current tier: {selectedUser.tier}</div>
                <button
                  type="button"
                  disabled={submittingId === selectedUser.id}
                  onClick={() => queueTierUpdate(selectedUser)}
                >
                  Save tier draft
                </button>
              </div>
            </SurfaceCard>

            <SurfaceCard
              eyebrow="Role Policy"
              title="Role and permission cluster"
              summary="Review the current permission footprint, then apply a role policy from the drawer."
              className="user-drawer-section"
            >
              <div className="action-stack user-drawer-action-stack">
                <div className="table-chip-row">
                  {selectedUser.permissions.length > 0 ? (
                    selectedUser.permissions.map((permission) => (
                      <span key={permission} className="permission-chip">
                        {permission}
                      </span>
                    ))
                  ) : (
                    <span className="permission-chip muted">basic:access</span>
                  )}
                </div>
                <div className="action-row">
                  <select
                    value={roleDrafts[selectedUser.id] ?? inferRoleFromPermissions(selectedUser.permissions)}
                    onChange={(event) =>
                      setRoleDrafts((prev) => ({
                        ...prev,
                        [selectedUser.id]: event.target.value as RoleOption
                      }))
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
                    disabled={submittingId === selectedUser.id}
                    onClick={() => queueRoleUpdate(selectedUser)}
                  >
                    Apply role
                  </button>
                </div>
              </div>
            </SurfaceCard>
          </div>
        ) : null}
      </DrawerSurface>
    </PageShell>
  );
}
