"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  createAdminApiKey,
  deleteAdminApiKey,
  getAdminSession,
  getAdminApiKeys,
  revokeAdminApiKey,
  rotateAdminApiKey,
} from "@/lib/api";
import { copyToClipboard, exportCsv, exportJson } from "@/lib/export";
import { hasPermission } from "@/lib/permissions";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getStringParam } from "@/lib/url-query";
import type { AdminApiKeyItem, AdminSession } from "@/types/admin";

const PERMISSION_GROUPS = [
  {
    label: "Admin",
    permissions: [
      { value: "admin:*", label: "Full admin access" },
      { value: "admin:users:list", label: "List users" },
      { value: "admin:users:write", label: "Modify users" },
      { value: "admin:keys:list", label: "List API keys" },
      { value: "admin:keys:create", label: "Create API keys" },
      { value: "admin:keys:revoke", label: "Revoke API keys" },
      { value: "admin:keys:rotate", label: "Rotate API keys" },
      { value: "admin:keys:delete", label: "Delete API keys" },
      { value: "admin:analytics", label: "View analytics" },
    ],
  },
  {
    label: "API Access",
    permissions: [{ value: "basic:access", label: "Basic API access" }],
  },
  {
    label: "Engines",
    permissions: [
      { value: "engines:panchanga", label: "Panchanga" },
      { value: "engines:numerology", label: "Numerology" },
      { value: "engines:biorhythm", label: "Biorhythm" },
      { value: "engines:human-design", label: "Human Design" },
      { value: "engines:gene-keys", label: "Gene Keys" },
      { value: "engines:vimshottari", label: "Vimshottari" },
    ],
  },
];

type RecentSecretState = {
  keyId: string;
  secret: string;
  source: "created" | "rotated";
};

function formatDateTime(value: string | null): string {
  if (!value) return "--";
  const d = new Date(value);
  return Number.isNaN(d.getTime()) ? "--" : d.toLocaleString();
}

function tierPillClass(tier: string): string {
  switch (tier.toLowerCase()) {
    case "premium":
      return "pill tier-premium";
    case "enterprise":
      return "pill tier-enterprise";
    default:
      return "pill tier-free";
  }
}

function permissionsPreview(permissions: string[]): string[] {
  const fallback = permissions.length > 0 ? permissions : ["basic:access"];
  return fallback.slice(0, 2);
}

function EyeIcon({ open }: { open: boolean }) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M2 12c2.4-4.1 5.7-6.2 10-6.2S19.6 7.9 22 12c-2.4 4.1-5.7 6.2-10 6.2S4.4 16.1 2 12Z"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="12" cy="12" r="3.2" fill="none" stroke="currentColor" strokeWidth="1.6" />
      {!open ? (
        <path
          d="M4 4l16 16"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.6"
          strokeLinecap="round"
        />
      ) : null}
    </svg>
  );
}

function CopyIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <rect x="9" y="9" width="11" height="11" rx="2" fill="none" stroke="currentColor" strokeWidth="1.6" />
      <path
        d="M6 15H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v1"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
      />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M4 7h16M9 7V4h6v3m-8 0 1 12a2 2 0 0 0 2 2h4a2 2 0 0 0 2-2l1-12"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function RotateIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M20 4v6h-6M4 20v-6h6"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M7.8 9.2A6.5 6.5 0 0 1 18 10M16.2 14.8A6.5 6.5 0 0 1 6 14"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function RevokeIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M12 3 4 7v5c0 5 3.4 8 8 9 4.6-1 8-4 8-9V7l-8-4Z"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinejoin="round"
      />
      <path
        d="m8 8 8 8"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
      />
    </svg>
  );
}

export default function ApiKeysPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const [query, setQuery] = useState(() => getStringParam(searchParams, "query"));
  const [tierFilter, setTierFilter] = useState(() => getStringParam(searchParams, "tier", "all"));
  const [statusFilter, setStatusFilter] = useState(() => getStringParam(searchParams, "status", "all"));
  const [loading, setLoading] = useState(true);
  const [keys, setKeys] = useState<AdminApiKeyItem[]>([]);
  const [total, setTotal] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [showCreate, setShowCreate] = useState(false);
  const [createName, setCreateName] = useState("");
  const [createUserId, setCreateUserId] = useState("");
  const [createTier, setCreateTier] = useState("premium");
  const [createPerms, setCreatePerms] = useState<Set<string>>(new Set(["basic:access"]));
  const [createRateLimit, setCreateRateLimit] = useState("1000");
  const [createExpires, setCreateExpires] = useState("");
  const [creating, setCreating] = useState(false);

  const [selectedKey, setSelectedKey] = useState<AdminApiKeyItem | null>(null);
  const [recentSecret, setRecentSecret] = useState<RecentSecretState | null>(null);
  const [secretVisible, setSecretVisible] = useState(false);
  const [copiedSecret, setCopiedSecret] = useState(false);
  const [session, setSession] = useState<AdminSession | null>(null);

  const [confirmRevoke, setConfirmRevoke] = useState<AdminApiKeyItem | null>(null);
  const [confirmRotate, setConfirmRotate] = useState<AdminApiKeyItem | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<AdminApiKeyItem | null>(null);
  const [submittingId, setSubmittingId] = useState<string | null>(null);

  const activeOnly = statusFilter === "active";

  const filteredKeys = useMemo(() => {
    let result = keys;
    if (statusFilter === "revoked") result = result.filter((k) => !k.is_active);
    if (tierFilter !== "all") {
      result = result.filter((k) => k.tier.toLowerCase() === tierFilter.toLowerCase());
    }
    return result;
  }, [keys, statusFilter, tierFilter]);

  const activeCount = useMemo(() => keys.filter((k) => k.is_active).length, [keys]);
  const revokedCount = useMemo(() => keys.filter((k) => !k.is_active).length, [keys]);
  const expiringCount = useMemo(
    () =>
      keys.filter((k) => {
        if (!k.expires_at || !k.is_active) return false;
        const expiresAt = new Date(k.expires_at).getTime();
        const inSevenDays = Date.now() + 7 * 24 * 60 * 60 * 1000;
        return Number.isFinite(expiresAt) && expiresAt < inSevenDays;
      }).length,
    [keys]
  );

  const canDeleteKeys = hasPermission(session?.permissions ?? [], "admin:keys:delete");
  const canRotateKeys = hasPermission(session?.permissions ?? [], "admin:keys:rotate");
  const canRevokeKeys = hasPermission(session?.permissions ?? [], "admin:keys:revoke");
  const secretForSelectedKey =
    selectedKey && recentSecret?.keyId === selectedKey.id ? recentSecret : null;

  useEffect(() => {
    const token = getAuthToken();
    if (!token) return;

    void getAdminSession(token)
      .then(setSession)
      .catch(() => {
        // Ignore permission refresh failures and let route guards / action errors handle auth state.
      });
  }, []);

  useEffect(() => {
    setQuery(getStringParam(searchParams, "query"));
    setTierFilter(getStringParam(searchParams, "tier", "all"));
    setStatusFilter(getStringParam(searchParams, "status", "all"));
  }, [searchParams]);

  useEffect(() => {
    const nextQuery = buildQueryString(searchParams, {
      query,
      tier: tierFilter !== "all" ? tierFilter : undefined,
      status: statusFilter !== "all" ? statusFilter : undefined,
    });
    router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
  }, [pathname, query, router, searchParams, statusFilter, tierFilter]);

  useEffect(() => {
    if (!selectedKey) return;
    const fresh = keys.find((item) => item.id === selectedKey.id);
    if (fresh) {
      setSelectedKey(fresh);
    }
  }, [keys, selectedKey]);

  const loadKeys = useCallback(async () => {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      setLoading(false);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const res = await getAdminApiKeys(token, {
        query: query || undefined,
        active_only: activeOnly,
        limit: 200,
        offset: 0,
      });
      setKeys(res.items);
      setTotal(res.total);
    } catch (err) {
      setError(
        err instanceof ApiClientError ? err.payload?.error || err.message : "Failed to load API keys"
      );
    } finally {
      setLoading(false);
    }
  }, [activeOnly, query]);

  useEffect(() => {
    void loadKeys();
  }, [loadKeys]);

  function resetCreateForm() {
    setCreateName("");
    setCreateUserId("");
    setCreateTier("premium");
    setCreatePerms(new Set(["basic:access"]));
    setCreateRateLimit("1000");
    setCreateExpires("");
  }

  function openKeyModal(key: AdminApiKeyItem) {
    setSelectedKey(key);
    if (recentSecret?.keyId !== key.id) {
      setRecentSecret(null);
      setSecretVisible(false);
      setCopiedSecret(false);
    }
  }

  function closeKeyModal() {
    setSelectedKey(null);
    setRecentSecret(null);
    setSecretVisible(false);
    setCopiedSecret(false);
  }

  async function handleCreate() {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token.");
      return;
    }
    if (!createUserId.trim()) {
      setError("User ID is required.");
      return;
    }

    setCreating(true);
    setError(null);
    setSuccess(null);
    try {
      const rl = Number.parseInt(createRateLimit, 10);
      const res = await createAdminApiKey(token, {
        user_id: createUserId.trim(),
        name: createName.trim() || undefined,
        tier: createTier,
        permissions: Array.from(createPerms),
        rate_limit: Number.isNaN(rl) ? undefined : rl,
        expires_at: createExpires || undefined,
      });

      setRecentSecret({ keyId: res.key.id, secret: res.secret_key, source: "created" });
      setSecretVisible(false);
      setCopiedSecret(false);
      setSelectedKey(res.key);
      setSuccess(`Created key "${res.key.name || res.key.id}"`);
      setShowCreate(false);
      resetCreateForm();
      await loadKeys();
    } catch (err) {
      setError(
        err instanceof ApiClientError ? err.payload?.error || err.message : "Failed to create API key"
      );
    } finally {
      setCreating(false);
    }
  }

  async function handleRevoke(key: AdminApiKeyItem) {
    const token = getAuthToken();
    if (!token) return;
    setSubmittingId(key.id);
    setError(null);
    setSuccess(null);
    try {
      await revokeAdminApiKey(token, key.id);
      setSuccess(`Revoked key "${key.name || key.id}"`);
      setConfirmRevoke(null);
      await loadKeys();
    } catch (err) {
      setError(err instanceof ApiClientError ? err.payload?.error || err.message : "Failed to revoke");
    } finally {
      setSubmittingId(null);
    }
  }

  async function handleRotate(key: AdminApiKeyItem) {
    const token = getAuthToken();
    if (!token) return;
    setSubmittingId(key.id);
    setError(null);
    setSuccess(null);
    try {
      const res = await rotateAdminApiKey(token, key.id);
      setRecentSecret({ keyId: res.key.id, secret: res.secret_key, source: "rotated" });
      setSecretVisible(false);
      setCopiedSecret(false);
      setSelectedKey(res.key);
      setSuccess(`Rotated key "${key.name || key.id}"`);
      setConfirmRotate(null);
      await loadKeys();
    } catch (err) {
      setError(err instanceof ApiClientError ? err.payload?.error || err.message : "Failed to rotate");
    } finally {
      setSubmittingId(null);
    }
  }

  async function handleDelete(key: AdminApiKeyItem) {
    const token = getAuthToken();
    if (!token) return;
    setSubmittingId(key.id);
    setError(null);
    setSuccess(null);
    try {
      await deleteAdminApiKey(token, key.id);
      setSuccess(`Deleted key "${key.name || key.id}" permanently`);
      setConfirmDelete(null);
      if (selectedKey?.id === key.id) {
        closeKeyModal();
      }
      await loadKeys();
    } catch (err) {
      setError(err instanceof ApiClientError ? err.payload?.error || err.message : "Failed to delete");
    } finally {
      setSubmittingId(null);
    }
  }

  async function handleCopySecret() {
    if (!secretForSelectedKey) return;
    await copyToClipboard(secretForSelectedKey.secret);
    setCopiedSecret(true);
    setSuccess("Secret copied");
    setTimeout(() => setCopiedSecret(false), 1800);
  }

  async function handleCopyValue(value: string, label: string) {
    try {
      await copyToClipboard(value);
      setSuccess(`${label} copied`);
      setTimeout(() => setSuccess(null), 1500);
    } catch {
      setError("Failed to copy value to clipboard");
    }
  }

  function handleExportCsv() {
    exportCsv(`admin-api-keys-${new Date().toISOString().slice(0, 10)}.csv`, filteredKeys, [
      { key: "id", header: "Key ID" },
      { key: "name", header: "Name" },
      { key: "key_prefix", header: "Prefix" },
      { key: "user_id", header: "User ID" },
      { key: "user_email", header: "User Email" },
      { key: "tier", header: "Tier" },
      { key: "is_active", header: "Is Active" },
      { key: "created_at", header: "Created At" },
      { key: "last_used", header: "Last Used" },
    ]);
  }

  function handleExportJson() {
    exportJson(`admin-api-keys-${new Date().toISOString().slice(0, 10)}.json`, filteredKeys);
  }

  function togglePerm(perm: string) {
    setCreatePerms((prev) => {
      const next = new Set(prev);
      if (next.has(perm)) next.delete(perm);
      else next.add(perm);
      return next;
    });
  }

  return (
    <PageShell
      title="API Keys"
      summary="Operate API key lifecycle from a single management surface with status, ownership, and destructive controls."
    >
      <div className="panel-inline">
        <label>
          Search
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="name, email, key id"
          />
        </label>
        <label>
          Tier
          <select value={tierFilter} onChange={(e) => setTierFilter(e.target.value)}>
            <option value="all">All tiers</option>
            <option value="free">Free</option>
            <option value="premium">Premium</option>
            <option value="enterprise">Enterprise</option>
          </select>
        </label>
        <label>
          Status
          <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}>
            <option value="all">All</option>
            <option value="active">Active</option>
            <option value="revoked">Revoked</option>
          </select>
        </label>
        <button type="button" onClick={() => void loadKeys()}>
          Refresh
        </button>
        <button type="button" onClick={handleExportCsv} disabled={filteredKeys.length === 0}>
          Export CSV
        </button>
        <button type="button" onClick={handleExportJson} disabled={filteredKeys.length === 0}>
          Export JSON
        </button>
        <button type="button" onClick={() => setShowCreate(true)} style={{ minWidth: 120 }}>
          + Create key
        </button>
      </div>

      <div className="grid metrics" style={{ marginTop: "0.8rem" }}>
        <article className="metric">
          <div className="label">Total keys</div>
          <div className="value">{total}</div>
        </article>
        <article className="metric">
          <div className="label">Active</div>
          <div className="value">{activeCount}</div>
        </article>
        <article className="metric">
          <div className="label">Revoked</div>
          <div className="value">{revokedCount}</div>
        </article>
        <article className="metric">
          <div className="label">Expiring in 7d</div>
          <div className="value">{expiringCount}</div>
        </article>
      </div>

      {error ? (
        <div className="error" style={{ marginTop: "0.8rem" }} role="alert">
          {error}
        </div>
      ) : null}

      {success ? (
        <div className="success" style={{ marginTop: "0.8rem" }} role="status">
          {success}
        </div>
      ) : null}

      {loading ? (
        <p className="helper" style={{ marginTop: "1rem" }}>
          Loading API keys...
        </p>
      ) : (
        <div className="table-wrap management-table" style={{ marginTop: "0.8rem" }}>
          <table>
            <thead>
              <tr>
                <th>Key</th>
                <th>Owner</th>
                <th>Access</th>
                <th>Usage</th>
                <th>Status</th>
                <th>Manage</th>
              </tr>
            </thead>
            <tbody>
              {filteredKeys.map((key) => (
                <tr
                  key={key.id}
                  className="clickable-row"
                  tabIndex={0}
                  onClick={() => openKeyModal(key)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      openKeyModal(key);
                    }
                  }}
                >
                  <td>
                    <div className="table-primary">{key.name || "Unnamed key"}</div>
                    <div className="table-secondary-row">
                      <span className="key-prefix">{key.key_prefix || `${key.id.slice(0, 8)}...`}</span>
                      <button
                        type="button"
                        className="link-btn"
                        onClick={(event) => {
                          event.stopPropagation();
                          void handleCopyValue(key.id, "Key ID");
                        }}
                      >
                        {key.id}
                      </button>
                    </div>
                    <div className="helper">Created {formatDateTime(key.created_at)}</div>
                  </td>
                  <td>
                    <div className="table-primary">{key.user_email}</div>
                    <button
                      type="button"
                      className="link-btn"
                      onClick={(event) => {
                        event.stopPropagation();
                        void handleCopyValue(key.user_id, "User ID");
                      }}
                    >
                      {key.user_id}
                    </button>
                  </td>
                  <td>
                    <div className="table-chip-row">
                      <span className={tierPillClass(key.tier)}>{key.tier}</span>
                      {permissionsPreview(key.permissions).map((permission) => (
                        <span className="permission-chip" key={`${key.id}-${permission}`}>
                          {permission}
                        </span>
                      ))}
                      {key.permissions.length > 2 ? (
                        <span className="permission-chip muted">+{key.permissions.length - 2}</span>
                      ) : null}
                    </div>
                    <div className="helper">{key.rate_limit}/min rate limit</div>
                  </td>
                  <td>
                    <div className="table-primary">{formatDateTime(key.last_used)}</div>
                    <div className="helper">
                      {key.expires_at ? `Expires ${formatDateTime(key.expires_at)}` : "No expiration"}
                    </div>
                  </td>
                  <td>
                    <span className={statusPillClass(key.is_active ? "active" : "revoked")}>
                      {key.is_active ? "active" : "revoked"}
                    </span>
                  </td>
                  <td>
                    <button
                      type="button"
                      className="manage-trigger"
                      onClick={(event) => {
                        event.stopPropagation();
                        openKeyModal(key);
                      }}
                    >
                      Open
                    </button>
                  </td>
                </tr>
              ))}
              {filteredKeys.length === 0 ? (
                <tr>
                  <td colSpan={6}>
                    <p className="helper">No API keys matched this filter.</p>
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
      )}

      {showCreate ? (
        <div className="modal-overlay" onClick={() => setShowCreate(false)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
            <h3>Create API Key</h3>

            <div className="form-group">
              <label>Key name</label>
              <input
                value={createName}
                onChange={(e) => setCreateName(e.target.value)}
                placeholder="e.g. Production Mobile App"
              />
            </div>

            <div className="form-group">
              <label>User ID</label>
              <input
                value={createUserId}
                onChange={(e) => setCreateUserId(e.target.value)}
                placeholder="UUID of the target user"
              />
            </div>

            <div className="modal-two-column">
              <div className="form-group">
                <label>Tier</label>
                <select value={createTier} onChange={(e) => setCreateTier(e.target.value)}>
                  <option value="free">Free (60/min)</option>
                  <option value="premium">Premium (1,000/min)</option>
                  <option value="enterprise">Enterprise (10,000/min)</option>
                </select>
              </div>
              <div className="form-group">
                <label>Rate limit override</label>
                <input
                  value={createRateLimit}
                  onChange={(e) => setCreateRateLimit(e.target.value)}
                  placeholder="req/min"
                />
              </div>
            </div>

            <div className="form-group">
              <label>Expiration (optional)</label>
              <input
                type="datetime-local"
                value={createExpires}
                onChange={(e) => setCreateExpires(e.target.value)}
              />
            </div>

            <div style={{ marginBottom: "0.65rem" }}>
              <label className="form-section-label">Permissions</label>
              {PERMISSION_GROUPS.map((group) => (
                <div className="perm-group" key={group.label}>
                  <div className="perm-group-label">{group.label}</div>
                  <div className="perm-list">
                    {group.permissions.map((permission) => (
                      <label className="perm-item" key={permission.value}>
                        <input
                          type="checkbox"
                          checked={createPerms.has(permission.value)}
                          onChange={() => togglePerm(permission.value)}
                        />
                        <span>{permission.label}</span>
                      </label>
                    ))}
                  </div>
                </div>
              ))}
            </div>

            <div className="modal-actions">
              <button type="button" onClick={() => setShowCreate(false)}>
                Cancel
              </button>
              <button
                type="button"
                className="btn-primary"
                disabled={creating || !createUserId.trim()}
                onClick={handleCreate}
              >
                {creating ? "Creating..." : "Generate key"}
              </button>
            </div>
          </div>
        </div>
      ) : null}

      {selectedKey ? (
        <div className="modal-overlay" onClick={closeKeyModal}>
          <div
            className="modal-card modal-card-wide key-management-modal"
            onClick={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
          >
            <div className="key-modal-header">
              <div>
                <div className="eyebrow">API key management</div>
                <h3>{selectedKey.name || "Unnamed key"}</h3>
                <p className="helper">
                  Full lifecycle controls for a single key, including permanent deletion.
                </p>
              </div>
              <div className="key-modal-header-meta">
                <span className={statusPillClass(selectedKey.is_active ? "active" : "revoked")}>
                  {selectedKey.is_active ? "active" : "revoked"}
                </span>
                <span className={tierPillClass(selectedKey.tier)}>{selectedKey.tier}</span>
              </div>
            </div>

            <div className="key-modal-toolbar">
              <button
                type="button"
                className="icon-action"
                onClick={() => setConfirmRotate(selectedKey)}
                disabled={!selectedKey.is_active || !canRotateKeys || submittingId === selectedKey.id}
              >
                <RotateIcon />
                <span>Rotate</span>
              </button>
              <button
                type="button"
                className="icon-action"
                onClick={() => setConfirmRevoke(selectedKey)}
                disabled={!selectedKey.is_active || !canRevokeKeys || submittingId === selectedKey.id}
              >
                <RevokeIcon />
                <span>Revoke</span>
              </button>
              <button
                type="button"
                className="icon-action danger"
                onClick={() => setConfirmDelete(selectedKey)}
                disabled={!canDeleteKeys || submittingId === selectedKey.id}
              >
                <TrashIcon />
                <span>Delete</span>
              </button>
            </div>

            <div className="key-modal-grid">
              <section className="detail-card">
                <h4>Identity</h4>
                <div className="detail-list">
                  <div>
                    <span className="detail-label">Key ID</span>
                    <button
                      type="button"
                      className="link-btn"
                      onClick={() => void handleCopyValue(selectedKey.id, "Key ID")}
                    >
                      {selectedKey.id}
                    </button>
                  </div>
                  <div>
                    <span className="detail-label">Prefix</span>
                    <span className="key-prefix">{selectedKey.key_prefix || "--"}</span>
                  </div>
                  <div>
                    <span className="detail-label">User email</span>
                    <span>{selectedKey.user_email}</span>
                  </div>
                  <div>
                    <span className="detail-label">User ID</span>
                    <button
                      type="button"
                      className="link-btn"
                      onClick={() => void handleCopyValue(selectedKey.user_id, "User ID")}
                    >
                      {selectedKey.user_id}
                    </button>
                  </div>
                </div>
              </section>

              <section className="detail-card">
                <h4>Lifecycle</h4>
                <div className="detail-list">
                  <div>
                    <span className="detail-label">Created</span>
                    <span>{formatDateTime(selectedKey.created_at)}</span>
                  </div>
                  <div>
                    <span className="detail-label">Last used</span>
                    <span>{formatDateTime(selectedKey.last_used)}</span>
                  </div>
                  <div>
                    <span className="detail-label">Expires</span>
                    <span>{formatDateTime(selectedKey.expires_at)}</span>
                  </div>
                  <div>
                    <span className="detail-label">Rate limit</span>
                    <span>{selectedKey.rate_limit}/min</span>
                  </div>
                </div>
              </section>

              <section className="detail-card detail-card-span">
                <h4>Permissions</h4>
                <div className="permission-grid">
                  {(selectedKey.permissions.length > 0 ? selectedKey.permissions : ["basic:access"]).map(
                    (permission) => (
                      <span className="permission-chip" key={`${selectedKey.id}-${permission}`}>
                        {permission}
                      </span>
                    )
                  )}
                </div>
              </section>

              <section className="detail-card detail-card-span">
                <div className="detail-card-head">
                  <div>
                    <h4>Secret access</h4>
                    <p className="helper">
                      {secretForSelectedKey
                        ? `The full secret is available because this key was just ${secretForSelectedKey.source} in this session.`
                        : "Existing keys cannot reveal their full secret after issuance. Only the prefix remains visible."}
                    </p>
                  </div>
                  {secretForSelectedKey ? (
                    <div className="inline-actions">
                      <button
                        type="button"
                        className="icon-action compact"
                        onClick={() => setSecretVisible((value) => !value)}
                      >
                        <EyeIcon open={secretVisible} />
                        <span>{secretVisible ? "Hide" : "Show"}</span>
                      </button>
                      <button
                        type="button"
                        className={`icon-action compact ${copiedSecret ? "success" : ""}`}
                        onClick={() => void handleCopySecret()}
                      >
                        <CopyIcon />
                        <span>{copiedSecret ? "Copied" : "Copy"}</span>
                      </button>
                    </div>
                  ) : null}
                </div>

                <div className="secret-display">
                  <code>
                    {secretForSelectedKey
                      ? secretVisible
                        ? secretForSelectedKey.secret
                        : "nk_••••••••••••••••••••••••••••••••"
                      : `${selectedKey.key_prefix || "nk_••••"}••••••••••••••••••••`}
                  </code>
                </div>
              </section>

              <section className="detail-card detail-card-span danger-zone">
                <h4>Danger zone</h4>
                <p className="helper">
                  Permanent delete removes the key row itself. This is stronger than revoke and cannot be undone.
                </p>
                <button
                  type="button"
                  className="danger-surface-trigger"
                  onClick={() => setConfirmDelete(selectedKey)}
                  disabled={!canDeleteKeys || submittingId === selectedKey.id}
                >
                  <TrashIcon />
                  <span>Delete API key permanently</span>
                </button>
              </section>
            </div>

            <div className="modal-actions">
              <button type="button" onClick={closeKeyModal}>
                Close
              </button>
            </div>
          </div>
        </div>
      ) : null}

      {confirmRevoke ? (
        <div className="modal-overlay" onClick={() => setConfirmRevoke(null)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
            <h3>Revoke API key</h3>
            <p style={{ margin: "0 0 0.6rem" }}>
              Revoke <strong>{confirmRevoke.name || confirmRevoke.id}</strong> and stop all future usage.
            </p>
            <p className="helper" style={{ margin: "0 0 0.8rem" }}>
              User: {confirmRevoke.user_email} | Tier: {confirmRevoke.tier}
            </p>
            <div className="modal-actions">
              <button type="button" onClick={() => setConfirmRevoke(null)}>
                Cancel
              </button>
              <button
                type="button"
                className="btn-danger"
                disabled={submittingId === confirmRevoke.id}
                onClick={() => void handleRevoke(confirmRevoke)}
              >
                {submittingId === confirmRevoke.id ? "Revoking..." : "Revoke key"}
              </button>
            </div>
          </div>
        </div>
      ) : null}

      {confirmRotate ? (
        <div className="modal-overlay" onClick={() => setConfirmRotate(null)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
            <h3>Rotate API key</h3>
            <p style={{ margin: "0 0 0.6rem" }}>
              Rotating <strong>{confirmRotate.name || confirmRotate.id}</strong> deactivates the current key
              and issues a new secret.
            </p>
            <p className="helper" style={{ margin: "0 0 0.8rem" }}>
              User: {confirmRotate.user_email} | Tier: {confirmRotate.tier}
            </p>
            <div className="modal-actions">
              <button type="button" onClick={() => setConfirmRotate(null)}>
                Cancel
              </button>
              <button
                type="button"
                className="btn-primary"
                disabled={submittingId === confirmRotate.id}
                onClick={() => void handleRotate(confirmRotate)}
              >
                {submittingId === confirmRotate.id ? "Rotating..." : "Rotate key"}
              </button>
            </div>
          </div>
        </div>
      ) : null}

      {confirmDelete ? (
        <div className="modal-overlay" onClick={() => setConfirmDelete(null)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
            <h3>Delete API key permanently</h3>
            <p style={{ margin: "0 0 0.6rem" }}>
              Permanently delete <strong>{confirmDelete.name || confirmDelete.id}</strong> from the
              database.
            </p>
            <p className="helper" style={{ margin: "0 0 0.8rem" }}>
              This removes the row itself, not just active access. This action cannot be undone.
            </p>
            <div className="modal-actions">
              <button type="button" onClick={() => setConfirmDelete(null)}>
                Cancel
              </button>
              <button
                type="button"
                className="btn-danger"
                disabled={submittingId === confirmDelete.id}
                onClick={() => void handleDelete(confirmDelete)}
              >
                {submittingId === confirmDelete.id ? "Deleting..." : "Delete permanently"}
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </PageShell>
  );
}
