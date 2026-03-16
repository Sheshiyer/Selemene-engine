"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  createAdminApiKey,
  getAdminApiKeys,
  revokeAdminApiKey,
  rotateAdminApiKey,
} from "@/lib/api";
import { copyToClipboard, exportCsv, exportJson } from "@/lib/export";
import { statusPillClass } from "@/lib/status";
import { buildQueryString, getStringParam } from "@/lib/url-query";
import type { AdminApiKeyItem } from "@/types/admin";

/* ---------- constants ---------- */

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

/* ---------- helpers ---------- */

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

/* ---------- page ---------- */

export default function ApiKeysPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  /* --- list state --- */
  const [query, setQuery] = useState(() => getStringParam(searchParams, "query"));
  const [tierFilter, setTierFilter] = useState(() => getStringParam(searchParams, "tier", "all"));
  const [statusFilter, setStatusFilter] = useState(() => getStringParam(searchParams, "status", "all"));
  const [loading, setLoading] = useState(true);
  const [keys, setKeys] = useState<AdminApiKeyItem[]>([]);
  const [total, setTotal] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  /* --- create modal state --- */
  const [showCreate, setShowCreate] = useState(false);
  const [createName, setCreateName] = useState("");
  const [createUserId, setCreateUserId] = useState("");
  const [createTier, setCreateTier] = useState("premium");
  const [createPerms, setCreatePerms] = useState<Set<string>>(
    new Set(["basic:access"])
  );
  const [createRateLimit, setCreateRateLimit] = useState("1000");
  const [createExpires, setCreateExpires] = useState("");
  const [creating, setCreating] = useState(false);

  /* --- secret reveal modal --- */
  const [secretKey, setSecretKey] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  /* --- confirm modals --- */
  const [confirmRevoke, setConfirmRevoke] = useState<AdminApiKeyItem | null>(
    null
  );
  const [confirmRotate, setConfirmRotate] = useState<AdminApiKeyItem | null>(
    null
  );
  const [submittingId, setSubmittingId] = useState<string | null>(null);

  /* --- derived --- */
  const activeOnly = statusFilter === "active";
  const filteredKeys = useMemo(() => {
    let result = keys;
    if (statusFilter === "revoked") result = result.filter((k) => !k.is_active);
    if (tierFilter !== "all")
      result = result.filter(
        (k) => k.tier.toLowerCase() === tierFilter.toLowerCase()
      );
    return result;
  }, [keys, tierFilter, statusFilter]);

  const activeCount = useMemo(
    () => keys.filter((k) => k.is_active).length,
    [keys]
  );
  const revokedCount = useMemo(
    () => keys.filter((k) => !k.is_active).length,
    [keys]
  );

  useEffect(() => {
    setQuery(getStringParam(searchParams, "query"));
    setTierFilter(getStringParam(searchParams, "tier", "all"));
    setStatusFilter(getStringParam(searchParams, "status", "all"));
  }, [searchParams]);

  useEffect(() => {
    const nextQuery = buildQueryString(searchParams, {
      query,
      tier: tierFilter !== "all" ? tierFilter : undefined,
      status: statusFilter !== "all" ? statusFilter : undefined
    });
    router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, { scroll: false });
  }, [pathname, query, router, searchParams, statusFilter, tierFilter]);

  /* --- data loading --- */
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
        err instanceof ApiClientError
          ? err.payload?.error || err.message
          : "Failed to load API keys"
      );
    } finally {
      setLoading(false);
    }
  }, [activeOnly, query]);

  useEffect(() => {
    void loadKeys();
  }, [loadKeys]);

  /* --- create key --- */
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
      const perms = Array.from(createPerms);
      const rl = Number.parseInt(createRateLimit, 10);
      const res = await createAdminApiKey(token, {
        user_id: createUserId.trim(),
        name: createName.trim() || undefined,
        tier: createTier,
        permissions: perms.length ? perms : undefined,
        rate_limit: Number.isNaN(rl) ? undefined : rl,
        expires_at: createExpires || undefined,
      });
      setSecretKey(res.secret_key);
      setSuccess(`Created key "${res.key.name || res.key.id}"`);
      setShowCreate(false);
      resetCreateForm();
      await loadKeys();
    } catch (err) {
      setError(
        err instanceof ApiClientError
          ? err.payload?.error || err.message
          : "Failed to create API key"
      );
    } finally {
      setCreating(false);
    }
  }

  function resetCreateForm() {
    setCreateName("");
    setCreateUserId("");
    setCreateTier("premium");
    setCreatePerms(new Set(["basic:access"]));
    setCreateRateLimit("1000");
    setCreateExpires("");
  }

  /* --- revoke --- */
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
      setError(
        err instanceof ApiClientError
          ? err.payload?.error || err.message
          : "Failed to revoke"
      );
    } finally {
      setSubmittingId(null);
    }
  }

  /* --- rotate --- */
  async function handleRotate(key: AdminApiKeyItem) {
    const token = getAuthToken();
    if (!token) return;
    setSubmittingId(key.id);
    setError(null);
    setSuccess(null);
    try {
      const res = await rotateAdminApiKey(token, key.id);
      setSecretKey(res.secret_key);
      setSuccess(`Rotated key "${key.name || key.id}"`);
      setConfirmRotate(null);
      await loadKeys();
    } catch (err) {
      setError(
        err instanceof ApiClientError
          ? err.payload?.error || err.message
          : "Failed to rotate"
      );
    } finally {
      setSubmittingId(null);
    }
  }

  /* --- copy to clipboard --- */
  async function handleCopySecret() {
    if (!secretKey) return;
    await copyToClipboard(secretKey);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
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
    exportCsv(
      `admin-api-keys-${new Date().toISOString().slice(0, 10)}.csv`,
      filteredKeys,
      [
        { key: "id", header: "Key ID" },
        { key: "name", header: "Name" },
        { key: "key_prefix", header: "Prefix" },
        { key: "user_id", header: "User ID" },
        { key: "user_email", header: "User Email" },
        { key: "tier", header: "Tier" },
        { key: "is_active", header: "Is Active" },
        { key: "created_at", header: "Created At" },
        { key: "last_used", header: "Last Used" }
      ]
    );
  }

  function handleExportJson() {
    exportJson(`admin-api-keys-${new Date().toISOString().slice(0, 10)}.json`, filteredKeys);
  }

  /* --- toggle permission --- */
  function togglePerm(perm: string) {
    setCreatePerms((prev) => {
      const next = new Set(prev);
      if (next.has(perm)) next.delete(perm);
      else next.add(perm);
      return next;
    });
  }

  /* ---------- render ---------- */
  return (
    <PageShell
      title="API Keys"
      summary="Generate and manage fine-grained API keys for platform users."
    >
      {/* ---- filter bar ---- */}
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
          <select
            value={tierFilter}
            onChange={(e) => setTierFilter(e.target.value)}
          >
            <option value="all">All tiers</option>
            <option value="free">Free</option>
            <option value="premium">Premium</option>
            <option value="enterprise">Enterprise</option>
          </select>
        </label>
        <label>
          Status
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
          >
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
        <button
          type="button"
          onClick={() => setShowCreate(true)}
          style={{ minWidth: 120 }}
        >
          + Create key
        </button>
      </div>

      {/* ---- metrics ---- */}
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
      </div>

      {/* ---- banners ---- */}
      {error && (
        <div className="error" style={{ marginTop: "0.8rem" }}>
          {error}
        </div>
      )}
      {success && (
        <div className="success" style={{ marginTop: "0.8rem" }}>
          {success}
        </div>
      )}

      {/* ---- table ---- */}
      {loading ? (
        <p className="helper" style={{ marginTop: "1rem" }}>
          Loading API keys...
        </p>
      ) : (
        <div className="table-wrap" style={{ marginTop: "0.8rem" }}>
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>API Key</th>
                <th>User</th>
                <th>Tier</th>
                <th>Permissions</th>
                <th>Rate</th>
                <th>Status</th>
                <th>Last used</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredKeys.map((k) => (
                <tr key={k.id}>
                  <td>
                    <div className="table-primary">{k.name || "Unnamed"}</div>
                    <div className="helper">{formatDateTime(k.created_at)}</div>
                    <div style={{ display: "inline-flex", alignItems: "center", gap: "0.3rem" }}>
                      <span className="helper" style={{ fontSize: "0.75rem", fontFamily: "var(--font-mono), monospace" }}>
                        {k.id.slice(0, 8)}...
                      </span>
                      <button
                        type="button"
                        className="copy-icon-btn"
                        title="Copy full key ID"
                        onClick={() => void handleCopyValue(k.id, "Key ID")}
                      >
                        ⧉
                      </button>
                    </div>
                  </td>
                  <td>
                    <div className="key-display">
                      <code>
                        {k.key_prefix ? (
                          <>
                            {k.key_prefix}
                            <span className="masked">••••</span>
                          </>
                        ) : (
                          <>
                            {k.id.slice(0, 8)}...
                            <span className="legacy-tag">(legacy)</span>
                          </>
                        )}
                      </code>
                      <button
                        type="button"
                        className="copy-icon-btn"
                        title="Copy key prefix"
                        onClick={() =>
                          void handleCopyValue(
                            k.key_prefix || k.id.slice(0, 8),
                            "Key prefix"
                          )
                        }
                      >
                        ⧉
                      </button>
                    </div>
                  </td>
                  <td>
                    <div className="table-primary">{k.user_email}</div>
                    <button
                      type="button"
                      className="link-btn"
                      onClick={() => void handleCopyValue(k.user_id, "User ID")}
                    >
                      {k.user_id}
                    </button>
                  </td>
                  <td>
                    <span className={tierPillClass(k.tier)}>{k.tier}</span>
                  </td>
                  <td className="cell-wrap">
                    {k.permissions.join(", ") || "basic:access"}
                  </td>
                  <td>{k.rate_limit}/min</td>
                  <td>
                    <span className={statusPillClass(k.is_active ? "active" : "revoked")}>
                      {k.is_active ? "active" : "revoked"}
                    </span>
                  </td>
                  <td>{formatDateTime(k.last_used)}</td>
                  <td>
                    <div className="action-stack">
                      <button
                        type="button"
                        disabled={
                          submittingId === k.id || !k.is_active
                        }
                        onClick={() => setConfirmRotate(k)}
                      >
                        Rotate
                      </button>
                      <button
                        type="button"
                        disabled={
                          submittingId === k.id || !k.is_active
                        }
                        onClick={() => setConfirmRevoke(k)}
                      >
                        Revoke
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
              {filteredKeys.length === 0 && (
                <tr>
                  <td colSpan={9}>
                    <p className="helper">No API keys matched this filter.</p>
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      {/* ======== CREATE MODAL ======== */}
      {showCreate && (
        <div
          className="modal-overlay"
          onClick={() => setShowCreate(false)}
        >
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
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

            <div
              style={{
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: "0.65rem",
              }}
            >
              <div className="form-group">
                <label>Tier</label>
                <select
                  value={createTier}
                  onChange={(e) => setCreateTier(e.target.value)}
                >
                  <option value="free">Free (60/min)</option>
                  <option value="premium">Premium (1,000/min)</option>
                  <option value="enterprise">
                    Enterprise (10,000/min)
                  </option>
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
              <label
                style={{
                  fontSize: "0.86rem",
                  color: "var(--text-muted)",
                }}
              >
                Permissions
              </label>
              {PERMISSION_GROUPS.map((group) => (
                <div className="perm-group" key={group.label}>
                  <div className="perm-group-label">{group.label}</div>
                  <div className="perm-list">
                    {group.permissions.map((p) => (
                      <label className="perm-item" key={p.value}>
                        <input
                          type="checkbox"
                          checked={createPerms.has(p.value)}
                          onChange={() => togglePerm(p.value)}
                        />
                        <span>{p.label}</span>
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
      )}

      {/* ======== SECRET REVEAL MODAL ======== */}
      {secretKey && (
        <div className="modal-overlay">
          <div className="modal-card">
            <h3>Your new API key</h3>
            <p className="helper" style={{ margin: "0 0 0.8rem" }}>
              Copy this key now. It will not be shown again.
            </p>
            <div className="secret-card">
              <code>{secretKey}</code>
            </div>
            <div
              className="modal-actions"
              style={{ marginTop: "0.8rem" }}
            >
              <button
                type="button"
                className={`copy-btn ${copied ? "copied" : ""}`}
                onClick={handleCopySecret}
              >
                {copied ? "Copied!" : "Copy to clipboard"}
              </button>
              <button
                type="button"
                className="btn-primary"
                onClick={() => {
                  setSecretKey(null);
                  setCopied(false);
                }}
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ======== CONFIRM REVOKE MODAL ======== */}
      {confirmRevoke && (
        <div
          className="modal-overlay"
          onClick={() => setConfirmRevoke(null)}
        >
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <h3>Revoke API key</h3>
            <p style={{ margin: "0 0 0.6rem" }}>
              Are you sure you want to revoke{" "}
              <strong>{confirmRevoke.name || confirmRevoke.id}</strong>?
              This action cannot be undone.
            </p>
            <p className="helper" style={{ margin: "0 0 0.8rem" }}>
              User: {confirmRevoke.user_email} | Tier: {confirmRevoke.tier}
            </p>
            <div className="modal-actions">
              <button
                type="button"
                onClick={() => setConfirmRevoke(null)}
              >
                Cancel
              </button>
              <button
                type="button"
                className="btn-danger"
                disabled={submittingId === confirmRevoke.id}
                onClick={() => void handleRevoke(confirmRevoke)}
              >
                {submittingId === confirmRevoke.id
                  ? "Revoking..."
                  : "Revoke key"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ======== CONFIRM ROTATE MODAL ======== */}
      {confirmRotate && (
        <div
          className="modal-overlay"
          onClick={() => setConfirmRotate(null)}
        >
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <h3>Rotate API key</h3>
            <p style={{ margin: "0 0 0.6rem" }}>
              Rotating{" "}
              <strong>{confirmRotate.name || confirmRotate.id}</strong>{" "}
              will deactivate the current key and generate a new one.
            </p>
            <p className="helper" style={{ margin: "0 0 0.8rem" }}>
              User: {confirmRotate.user_email} | Tier: {confirmRotate.tier}
            </p>
            <div className="modal-actions">
              <button
                type="button"
                onClick={() => setConfirmRotate(null)}
              >
                Cancel
              </button>
              <button
                type="button"
                className="btn-primary"
                disabled={submittingId === confirmRotate.id}
                onClick={() => void handleRotate(confirmRotate)}
              >
                {submittingId === confirmRotate.id
                  ? "Rotating..."
                  : "Rotate key"}
              </button>
            </div>
          </div>
        </div>
      )}
    </PageShell>
  );
}
