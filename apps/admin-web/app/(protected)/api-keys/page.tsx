"use client";

import { useEffect, useMemo, useState } from "react";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  createAdminApiKey,
  getAdminApiKeys,
  revokeAdminApiKey,
  rotateAdminApiKey
} from "@/lib/api";
import type { AdminApiKeyItem } from "@/types/admin";

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

export default function ApiKeysPage() {
  const [query, setQuery] = useState("");
  const [activeOnly, setActiveOnly] = useState(false);

  const [createUserId, setCreateUserId] = useState("");
  const [createTier, setCreateTier] = useState("premium");
  const [createPermissions, setCreatePermissions] = useState(
    "basic:access,admin:analytics:read"
  );
  const [createRateLimit, setCreateRateLimit] = useState("1000");

  const [loading, setLoading] = useState(true);
  const [submittingKeyId, setSubmittingKeyId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [latestSecret, setLatestSecret] = useState<string | null>(null);

  const [keys, setKeys] = useState<AdminApiKeyItem[]>([]);
  const [total, setTotal] = useState(0);

  const activeKeys = useMemo(() => keys.filter((key) => key.is_active).length, [keys]);

  async function loadKeys() {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const response = await getAdminApiKeys(token, {
        query: query || undefined,
        active_only: activeOnly,
        limit: 100,
        offset: 0
      });

      setKeys(response.items);
      setTotal(response.total);
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to load API keys");
      }
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadKeys();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query, activeOnly]);

  async function handleCreateKey() {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      return;
    }

    if (!createUserId.trim()) {
      setError("User ID is required to create an API key.");
      return;
    }

    setSubmittingKeyId("create");
    setError(null);
    setSuccess(null);
    setLatestSecret(null);

    const permissions = createPermissions
      .split(",")
      .map((permission) => permission.trim())
      .filter(Boolean);

    const parsedRateLimit = Number.parseInt(createRateLimit, 10);

    try {
      const response = await createAdminApiKey(token, {
        user_id: createUserId.trim(),
        tier: createTier.trim() || undefined,
        permissions: permissions.length ? permissions : undefined,
        rate_limit: Number.isNaN(parsedRateLimit) ? undefined : parsedRateLimit
      });

      setLatestSecret(response.secret_key);
      setSuccess(`Created new key ${response.key.id}`);
      await loadKeys();
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to create API key");
      }
    } finally {
      setSubmittingKeyId(null);
    }
  }

  async function handleRevokeKey(keyId: string) {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      return;
    }

    setSubmittingKeyId(keyId);
    setError(null);
    setSuccess(null);

    try {
      await revokeAdminApiKey(token, keyId);
      setSuccess(`Revoked key ${keyId}`);
      await loadKeys();
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to revoke API key");
      }
    } finally {
      setSubmittingKeyId(null);
    }
  }

  async function handleRotateKey(keyId: string) {
    const token = getAuthToken();
    if (!token) {
      setError("Missing session token. Please sign in again.");
      return;
    }

    setSubmittingKeyId(keyId);
    setError(null);
    setSuccess(null);
    setLatestSecret(null);

    try {
      const response = await rotateAdminApiKey(token, keyId);
      setLatestSecret(response.secret_key);
      setSuccess(`Rotated key ${keyId}`);
      await loadKeys();
    } catch (err) {
      if (err instanceof ApiClientError) {
        setError(err.payload?.error || err.message);
      } else {
        setError("Failed to rotate API key");
      }
    } finally {
      setSubmittingKeyId(null);
    }
  }

  return (
    <PageShell
      title="API Keys"
      summary="Manage key creation, revocation, and rotation with one-time secret reveal."
    >
      <div className="panel-inline">
        <label>
          Search
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="user email / key id"
          />
        </label>
        <label className="checkbox">
          <input
            type="checkbox"
            checked={activeOnly}
            onChange={(event) => setActiveOnly(event.target.checked)}
          />
          Active only
        </label>
        <button type="button" onClick={() => void loadKeys()}>
          Refresh
        </button>
      </div>

      <article className="panel">
        <h3>Create API Key</h3>
        <div className="panel-inline">
          <label>
            User ID
            <input
              value={createUserId}
              onChange={(event) => setCreateUserId(event.target.value)}
              placeholder="UUID"
            />
          </label>
          <label>
            Tier
            <input value={createTier} onChange={(event) => setCreateTier(event.target.value)} />
          </label>
          <label>
            Rate limit
            <input
              value={createRateLimit}
              onChange={(event) => setCreateRateLimit(event.target.value)}
              placeholder="requests/min"
            />
          </label>
        </div>
        <label>
          Permissions (comma separated)
          <input
            value={createPermissions}
            onChange={(event) => setCreatePermissions(event.target.value)}
          />
        </label>
        <button type="button" disabled={submittingKeyId === "create"} onClick={handleCreateKey}>
          {submittingKeyId === "create" ? "Creating..." : "Create key"}
        </button>
        {latestSecret ? (
          <div className="secret-card">
            <strong>New secret (copy now):</strong>
            <code>{latestSecret}</code>
          </div>
        ) : null}
      </article>

      <div className="grid metrics">
        <article className="metric">
          <div className="label">Total keys</div>
          <div className="value">{total}</div>
        </article>
        <article className="metric">
          <div className="label">Visible keys</div>
          <div className="value">{keys.length}</div>
        </article>
        <article className="metric">
          <div className="label">Active keys</div>
          <div className="value">{activeKeys}</div>
        </article>
      </div>

      {error ? <div className="error">{error}</div> : null}
      {success ? <div className="success">{success}</div> : null}

      {loading ? (
        <p className="helper">Loading API keys...</p>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Key</th>
                <th>User</th>
                <th>Tier</th>
                <th>Status</th>
                <th>Permissions</th>
                <th>Last used</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {keys.map((key) => (
                <tr key={key.id}>
                  <td>
                    <div className="table-primary">{key.id}</div>
                    <div className="helper">Created: {formatDateTime(key.created_at)}</div>
                  </td>
                  <td>
                    <div className="table-primary">{key.user_email}</div>
                    <div className="helper">{key.user_id}</div>
                  </td>
                  <td>{key.tier}</td>
                  <td>
                    <span className={`pill ${key.is_active ? "ok" : "danger"}`}>
                      {key.is_active ? "active" : "revoked"}
                    </span>
                  </td>
                  <td className="cell-wrap">{key.permissions.join(", ") || "basic:access"}</td>
                  <td>{formatDateTime(key.last_used)}</td>
                  <td>
                    <div className="action-stack">
                      <button
                        type="button"
                        disabled={submittingKeyId === key.id}
                        onClick={() => void handleRotateKey(key.id)}
                      >
                        Rotate
                      </button>
                      <button
                        type="button"
                        disabled={submittingKeyId === key.id || !key.is_active}
                        onClick={() => void handleRevokeKey(key.id)}
                      >
                        Revoke
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
              {keys.length === 0 ? (
                <tr>
                  <td colSpan={7}>
                    <p className="helper">No API keys matched this filter.</p>
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
