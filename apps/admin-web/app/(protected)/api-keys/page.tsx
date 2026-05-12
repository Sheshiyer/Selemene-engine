"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { ActionRail, MetricSurface } from "@/components/admin-primitives";
import { BulkActionBar, useBulkSelection } from "@/components/bulk-actions";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { ModalSurface } from "@/components/overlay-surface";
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

const SECRET_REVEAL_WINDOW_SECONDS = 15;
const SECRET_INITIAL_REVEAL_SECONDS = 60;

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

// ─── Geometric glyph derived from key ID hash ────────────────────────────────
function KeyGlyph({ keyId }: { keyId: string }) {
  let h = 0;
  for (let i = 0; i < Math.min(keyId.length, 16); i++) {
    h = ((h << 5) - h + keyId.charCodeAt(i)) | 0;
  }
  const hue = (Math.abs(h) % 280) + 20;
  const sides = (Math.abs(h >> 8) % 3) + 4;
  const pts = Array.from({ length: sides }, (_, i) => {
    const a = (i / sides) * 2 * Math.PI - Math.PI / 2;
    return `${16 + 12 * Number(Math.cos(a).toFixed(2))},${16 + 12 * Number(Math.sin(a).toFixed(2))}`;
  }).join(" ");
  return (
    <svg viewBox="0 0 32 32" className="key-glyph-svg" aria-hidden="true">
      <polygon points={pts} fill="none" stroke={`hsl(${hue},40%,54%)`} strokeWidth="1.2" opacity="0.8" />
      <circle cx="16" cy="16" r="3.5" fill={`hsl(${hue},38%,38%)`} opacity="0.7" />
    </svg>
  );
}

// ─── Dedicated secret reveal — independent of selectedKey modal state ─────────
function SecretRevealModal({
  secret,
  source,
  keyName,
  onDismiss,
}: {
  secret: string;
  source: "created" | "rotated";
  keyName: string;
  onDismiss: () => void;
}) {
  const [secondsLeft, setSecondsLeft] = useState(60);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const id = window.setInterval(() => {
      setSecondsLeft((s) => {
        if (s <= 1) { window.clearInterval(id); onDismiss(); return 0; }
        return s - 1;
      });
    }, 1000);
    return () => window.clearInterval(id);
  }, [onDismiss]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) { if (e.key === "Escape") onDismiss(); }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onDismiss]);

  async function handleCopy() {
    await copyToClipboard(secret);
    setCopied(true);
    setTimeout(() => setCopied(false), 2500);
  }

  const pct = secondsLeft / 60;
  const r = 26;
  const circ = 2 * Math.PI * r;

  return (
    <div className="secret-ceremony-overlay" role="dialog" aria-modal="true" aria-label="Secret key reveal">
      <div className="secret-ceremony-card">
        <div className="secret-ceremony-eyebrow">
          {source === "created" ? "Credential Inscribed" : "Credential Rotated"}
        </div>
        <h2 className="secret-ceremony-title">{keyName}</h2>
        <p className="secret-ceremony-warning">
          This is the only moment your secret is visible. It cannot be recovered after this window closes.
        </p>

        <div className="secret-ceremony-timer" aria-label={`${secondsLeft} seconds remaining`}>
          <svg viewBox="0 0 60 60" className="secret-ceremony-ring" aria-hidden="true">
            <circle cx="30" cy="30" r={r} className="secret-ceremony-ring-track" />
            <circle
              cx="30" cy="30" r={r}
              className="secret-ceremony-ring-fill"
              strokeDasharray={`${pct * circ} ${circ}`}
              strokeDashoffset={circ / 4}
            />
          </svg>
          <span className="secret-ceremony-timer-num">{secondsLeft}</span>
        </div>

        <div className="secret-ceremony-key-wrap">
          <code className="secret-ceremony-key">{secret}</code>
        </div>

        <div className="secret-ceremony-actions">
          <button
            type="button"
            className={`secret-ceremony-copy ${copied ? "copied" : ""}`}
            onClick={() => void handleCopy()}
          >
            <CopyIcon />
            <span>{copied ? "Copied to clipboard!" : "Copy secret key"}</span>
          </button>
          <button type="button" className="secret-ceremony-dismiss" onClick={onDismiss}>
            I&apos;ve saved it — close
          </button>
        </div>
      </div>
    </div>
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
  const [secretRevealSecondsLeft, setSecretRevealSecondsLeft] = useState(0);
  const [copiedSecret, setCopiedSecret] = useState(false);
  const [secretViewAttempted, setSecretViewAttempted] = useState(false);
  const [pendingReveal, setPendingReveal] = useState<{
    secret: string;
    source: "created" | "rotated";
    keyName: string;
  } | null>(null);
  const [session, setSession] = useState<AdminSession | null>(null);

  const [confirmRevoke, setConfirmRevoke] = useState<AdminApiKeyItem | null>(null);
  const [confirmRotate, setConfirmRotate] = useState<AdminApiKeyItem | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<AdminApiKeyItem | null>(null);
  const [confirmBulkRevokeIds, setConfirmBulkRevokeIds] = useState<string[] | null>(null);
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
  const bulkSelection = useBulkSelection(filteredKeys.map((key) => key.id));
  const selectedBulkKeys = useMemo(
    () => filteredKeys.filter((key) => bulkSelection.selectedSet.has(key.id)),
    [bulkSelection.selectedSet, filteredKeys]
  );
  const selectedRevokableKeys = useMemo(
    () => selectedBulkKeys.filter((key) => key.is_active),
    [selectedBulkKeys]
  );

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

  useEffect(() => {
    if (!secretVisible || !secretForSelectedKey) {
      return;
    }

    // Don't reset the countdown here — handlers set the initial window
    // (SECRET_INITIAL_REVEAL_SECONDS for create/rotate, SECRET_REVEAL_WINDOW_SECONDS for manual re-reveal)
    const intervalId = window.setInterval(() => {
      setSecretRevealSecondsLeft((current) => {
        if (current <= 1) {
          window.clearInterval(intervalId);
          setSecretVisible(false);
          return 0;
        }
        return current - 1;
      });
    }, 1000);

    return () => window.clearInterval(intervalId);
  }, [secretForSelectedKey, secretVisible]);

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
    setSecretViewAttempted(false);
    if (recentSecret?.keyId !== key.id) {
      setRecentSecret(null);
      setSecretVisible(false);
      setSecretRevealSecondsLeft(0);
      setCopiedSecret(false);
    }
  }

  function closeKeyModal() {
    setSelectedKey(null);
    setRecentSecret(null);
    setSecretVisible(false);
    setSecretRevealSecondsLeft(0);
    setCopiedSecret(false);
    setSecretViewAttempted(false);
  }

  function revealSecretBriefly() {
    setSecretVisible(true);
    setSecretRevealSecondsLeft(SECRET_REVEAL_WINDOW_SECONDS);
  }

  function hideSecret() {
    setSecretVisible(false);
    setSecretRevealSecondsLeft(0);
  }

  function toggleSecretReveal() {
    if (secretVisible) {
      hideSecret();
      return;
    }
    revealSecretBriefly();
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
      setSecretVisible(true);
      setSecretRevealSecondsLeft(SECRET_INITIAL_REVEAL_SECONDS);
      setCopiedSecret(false);
      setPendingReveal({ secret: res.secret_key, source: "created", keyName: res.key.name || res.key.id });
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
      setSecretVisible(true);
      setSecretRevealSecondsLeft(SECRET_INITIAL_REVEAL_SECONDS);
      setCopiedSecret(false);
      setPendingReveal({ secret: res.secret_key, source: "rotated", keyName: key.name || key.id });
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

  function queueBulkRevoke() {
    if (selectedRevokableKeys.length === 0) {
      setError("Select at least one active key to revoke.");
      return;
    }

    setConfirmBulkRevokeIds(selectedRevokableKeys.map((key) => key.id));
    setError(null);
    setSuccess(null);
  }

  async function handleBulkRevoke() {
    if (!confirmBulkRevokeIds) {
      return;
    }

    const token = getAuthToken();
    if (!token) {
      setError("Missing session token.");
      return;
    }

    const bulkIds = confirmBulkRevokeIds;
    setSubmittingId("bulk-key-revoke");
    setError(null);
    setSuccess(null);

    try {
      const results = await Promise.allSettled(
        bulkIds.map((keyId) => revokeAdminApiKey(token, keyId))
      );
      const failed = results.filter((result) => result.status === "rejected");
      const succeeded = results.length - failed.length;

      if (succeeded > 0) {
        setSuccess(`Revoked ${succeeded} API key${succeeded === 1 ? "" : "s"}.`);
      }

      if (failed.length > 0) {
        const firstFailure = failed[0];
        const baseMessage =
          firstFailure.reason instanceof ApiClientError
            ? firstFailure.reason.payload?.error || firstFailure.reason.message
            : "Some bulk key revocations failed";
        setError(
          failed.length === results.length
            ? baseMessage
            : `${baseMessage} (${failed.length} failed)`
        );
      }

      setConfirmBulkRevokeIds(null);
      bulkSelection.clear();
      await loadKeys();
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
      actions={
        <ActionRail label="API key actions">
          <button type="button" onClick={() => void loadKeys()}>
            Refresh
          </button>
          <button type="button" onClick={handleExportCsv} disabled={filteredKeys.length === 0}>
            Export CSV
          </button>
          <button type="button" onClick={handleExportJson} disabled={filteredKeys.length === 0}>
            Export JSON
          </button>
          <button type="button" onClick={() => setShowCreate(true)}>
            Create key
          </button>
        </ActionRail>
      }
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
      </div>

      <div className="grid metrics api-key-metrics">
        <MetricSurface label="Total keys" value={total} detail="All keys matching the current query scope." />
        <MetricSurface label="Active" value={activeCount} detail="Keys that can still authenticate requests." />
        <MetricSurface label="Revoked" value={revokedCount} detail="Keys retained for audit but blocked from use." />
        <MetricSurface
          label="Expiring in 7d"
          value={expiringCount}
          detail="Active keys that need renewal or rotation planning."
        />
      </div>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {success ? <StateBanner variant="success" title={success} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading API keys"
          description="Resolving key inventory, ownership, permission previews, and lifecycle state."
        />
      ) : (
        <>
          <div className="key-vault-header">
            <div className="key-vault-header-left">
              <div className="eyebrow">Key Registry</div>
              <p className="helper">
                {filteredKeys.length} credential{filteredKeys.length !== 1 ? "s" : ""} — select for bulk operations
              </p>
            </div>
            <BulkActionBar
              className="bulk-action-bar-embedded"
              itemLabel="keys"
              selectedCount={bulkSelection.selectedCount}
              visibleCount={filteredKeys.length}
              allVisibleSelected={bulkSelection.allVisibleSelected}
              onToggleVisible={bulkSelection.toggleVisible}
              onClear={bulkSelection.clear}
            >
              <button
                type="button"
                disabled={selectedRevokableKeys.length === 0 || !canRevokeKeys || submittingId !== null}
                onClick={queueBulkRevoke}
              >
                Revoke selected
              </button>
            </BulkActionBar>
          </div>

          <div className="key-vault-grid">
            {filteredKeys.map((key) => (
              <article
                key={key.id}
                className={`key-card${!key.is_active ? " key-card-revoked" : ""}${bulkSelection.selectedSet.has(key.id) ? " key-card-selected" : ""}`}
                tabIndex={0}
                role="button"
                aria-label={`Inspect key ${key.name || key.id}`}
                onClick={() => openKeyModal(key)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") { e.preventDefault(); openKeyModal(key); }
                }}
              >
                <div className="key-card-top">
                  <label className="key-card-check-wrap" onClick={(e) => e.stopPropagation()} aria-label={`Select ${key.name || key.id}`}>
                    <input
                      type="checkbox"
                      checked={bulkSelection.selectedSet.has(key.id)}
                      onChange={() => bulkSelection.toggle(key.id)}
                      onKeyDown={(e) => e.stopPropagation()}
                    />
                  </label>
                  <KeyGlyph keyId={key.id} />
                  <div className="key-card-identity">
                    <div className="key-card-name">{key.name || "Unnamed key"}</div>
                    <div className="key-card-email">{key.user_email}</div>
                  </div>
                  <div className="key-card-badges">
                    <span className={statusPillClass(key.is_active ? "active" : "revoked")}>
                      {key.is_active ? "active" : "revoked"}
                    </span>
                    <span className={tierPillClass(key.tier)}>{key.tier}</span>
                  </div>
                </div>

                <code className="key-card-prefix">{key.key_prefix || `${key.id.slice(0, 8)}…`}</code>

                <div className="key-card-stats">
                  <div>
                    <span className="key-stat-label">Last used</span>
                    <span className="key-stat-value">{formatDateTime(key.last_used)}</span>
                  </div>
                  <div>
                    <span className="key-stat-label">Rate</span>
                    <span className="key-stat-value">{key.rate_limit}/min</span>
                  </div>
                  <div>
                    <span className="key-stat-label">Expires</span>
                    <span className="key-stat-value">{key.expires_at ? formatDateTime(key.expires_at) : "Never"}</span>
                  </div>
                </div>

                <div className="key-card-perms">
                  {permissionsPreview(key.permissions).map((p) => (
                    <span className="permission-chip" key={`${key.id}-${p}`}>{p}</span>
                  ))}
                  {key.permissions.length > 2 ? (
                    <span className="permission-chip muted">+{key.permissions.length - 2}</span>
                  ) : null}
                </div>

                <div className="key-card-footer" onClick={(e) => e.stopPropagation()}>
                  <button
                    type="button"
                    className="key-card-btn"
                    title="Rotate"
                    disabled={!key.is_active || !canRotateKeys || submittingId !== null}
                    onClick={(e) => { e.stopPropagation(); setConfirmRotate(key); }}
                    aria-label={`Rotate ${key.name || key.id}`}
                  >
                    <RotateIcon />
                  </button>
                  <button
                    type="button"
                    className="key-card-btn danger"
                    title="Revoke"
                    disabled={!key.is_active || !canRevokeKeys || submittingId !== null}
                    onClick={(e) => { e.stopPropagation(); setConfirmRevoke(key); }}
                    aria-label={`Revoke ${key.name || key.id}`}
                  >
                    <RevokeIcon />
                  </button>
                  <button
                    type="button"
                    className="key-card-btn danger"
                    title="Delete record permanently"
                    disabled={!canDeleteKeys || submittingId !== null}
                    onClick={(e) => { e.stopPropagation(); setConfirmDelete(key); }}
                    aria-label={`Delete ${key.name || key.id}`}
                  >
                    <TrashIcon />
                  </button>
                  <button
                    type="button"
                    className="key-card-open"
                    onClick={(e) => { e.stopPropagation(); openKeyModal(key); }}
                  >
                    Inspect
                  </button>
                </div>
              </article>
            ))}
            {filteredKeys.length === 0 ? (
              <div className="key-vault-empty">
                <svg viewBox="0 0 48 48" className="key-vault-empty-glyph" aria-hidden="true">
                  <polygon points="24,4 44,14 44,34 24,44 4,34 4,14" fill="none" stroke="currentColor" strokeWidth="1.2" opacity="0.3" />
                  <circle cx="24" cy="24" r="6" fill="none" stroke="currentColor" strokeWidth="1.2" opacity="0.3" />
                </svg>
                <p>No credentials matched</p>
                <p className="helper">Widen the search, tier, or status filters.</p>
              </div>
            ) : null}
          </div>
        </>
      )}

      <ModalSurface
        open={showCreate}
        onClose={() => setShowCreate(false)}
        eyebrow="Create"
        title="Create API Key"
        summary="Generate a new key with explicit tier, permission, and expiration controls."
        footer={
          <>
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
          </>
        }
      >
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

        <div className="modal-section">
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
      </ModalSurface>

      <ModalSurface
        open={Boolean(selectedKey)}
        onClose={closeKeyModal}
        eyebrow="API key management"
        title={selectedKey?.name || "Unnamed key"}
        summary="Full lifecycle controls for a single key, including permanent deletion."
        className="modal-card-wide key-management-modal"
        footer={
          <button type="button" onClick={closeKeyModal}>
            Close
          </button>
        }
      >
        {selectedKey ? (
          <>
            <div className="key-modal-header">
              <div>
                <div className="eyebrow">Lifecycle</div>
                <p className="helper">Operate, rotate, revoke, and delete from a single operator surface.</p>
              </div>
              <div className="key-modal-header-meta">
                <span className={statusPillClass(selectedKey.is_active ? "active" : "revoked")}>
                  {selectedKey.is_active ? "active" : "revoked"}
                </span>
                <span className={tierPillClass(selectedKey.tier)}>{selectedKey.tier}</span>
              </div>
            </div>

            <ActionRail className="key-modal-toolbar" label="Key management actions">
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
            </ActionRail>

            {secretForSelectedKey && secretVisible ? (
              <div className="secret-reveal-alert" role="alert" aria-live="assertive">
                <div className="secret-reveal-alert-header">
                  <div className="secret-reveal-alert-copy">
                    <div className="secret-reveal-alert-eyebrow">
                      {secretForSelectedKey.source === "created" ? "KEY CREATED" : "KEY ROTATED"}
                    </div>
                    <p className="secret-reveal-alert-message">
                      Copy your secret now — this is the only time it will be visible.
                    </p>
                  </div>
                  <div className="secret-reveal-alert-timer" aria-label={`${secretRevealSecondsLeft} seconds remaining`}>
                    {secretRevealSecondsLeft}s
                  </div>
                </div>
                <div className="secret-reveal-alert-code">
                  <code>{secretForSelectedKey.secret}</code>
                </div>
                <div className="secret-reveal-alert-actions">
                  <button
                    type="button"
                    className={`secret-reveal-copy-btn ${copiedSecret ? "copied" : ""}`}
                    onClick={() => void handleCopySecret()}
                  >
                    <CopyIcon />
                    <span>{copiedSecret ? "Copied!" : "Copy secret"}</span>
                  </button>
                  <button type="button" className="secret-reveal-dismiss-btn" onClick={hideSecret}>
                    Dismiss
                  </button>
                </div>
              </div>
            ) : null}

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
                        ? `This key was just ${secretForSelectedKey.source} in this session — the secret is temporarily available.`
                        : "Only the key hash is stored. The plaintext secret can only be recovered by rotating the key."}
                    </p>
                  </div>
                  <div className="inline-actions">
                    <button
                      type="button"
                      className={`icon-action compact${secretVisible ? " active" : ""}`}
                      title={secretForSelectedKey ? (secretVisible ? "Hide secret" : "View secret") : "Secret not available — rotate to get a new one"}
                      onClick={() => {
                        if (secretForSelectedKey) {
                          toggleSecretReveal();
                        } else {
                          setSecretViewAttempted((v) => !v);
                        }
                      }}
                    >
                      <EyeIcon open={secretVisible} />
                      <span>
                        {secretForSelectedKey
                          ? secretVisible
                            ? `Hide (${secretRevealSecondsLeft}s)`
                            : "View secret"
                          : "View secret"}
                      </span>
                    </button>
                    {secretForSelectedKey && secretVisible ? (
                      <button
                        type="button"
                        className={`icon-action compact ${copiedSecret ? "success" : ""}`}
                        onClick={() => void handleCopySecret()}
                      >
                        <CopyIcon />
                        <span>{copiedSecret ? "Copied" : "Copy"}</span>
                      </button>
                    ) : null}
                  </div>
                </div>

                <div className="secret-display">
                  <code>
                    {secretForSelectedKey && secretVisible
                      ? secretForSelectedKey.secret
                      : `${selectedKey.key_prefix || "nk_••••"}••••••••••••••••••••`}
                  </code>
                </div>

                {!secretForSelectedKey && secretViewAttempted ? (
                  <div className="secret-unavailable-notice" role="alert">
                    <div className="secret-unavailable-body">
                      <div className="secret-unavailable-eyebrow">Secret unavailable</div>
                      <p>
                        The plaintext secret was only shown at creation and is no longer stored. To access a
                        new secret, rotate this key — a fresh secret will be revealed immediately.
                      </p>
                    </div>
                    <button
                      type="button"
                      className="secret-unavailable-rotate-btn"
                      disabled={!selectedKey.is_active || !canRotateKeys || submittingId === selectedKey.id}
                      onClick={() => { setSecretViewAttempted(false); setConfirmRotate(selectedKey); }}
                    >
                      Rotate to reveal
                    </button>
                  </div>
                ) : null}
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
          </>
        ) : null}
      </ModalSurface>

      <ModalSurface
        open={Boolean(confirmBulkRevokeIds)}
        onClose={() => setConfirmBulkRevokeIds(null)}
        eyebrow="Bulk revoke"
        title="Revoke selected API keys"
        summary={
          confirmBulkRevokeIds
            ? `Revoke ${confirmBulkRevokeIds.length} selected API key${confirmBulkRevokeIds.length === 1 ? "" : "s"} and retain the records for audit visibility.`
            : undefined
        }
        footer={
          <>
            <button type="button" onClick={() => setConfirmBulkRevokeIds(null)}>
              Cancel
            </button>
            <button
              type="button"
              className="btn-danger"
              disabled={!confirmBulkRevokeIds || submittingId !== null}
              onClick={() => void handleBulkRevoke()}
            >
              {submittingId !== null ? "Revoking..." : "Confirm revoke"}
            </button>
          </>
        }
      >
        {confirmBulkRevokeIds ? (
          <p className="helper">
            Revoke disables future usage while keeping each key row available for audit and operator review.
          </p>
        ) : null}
      </ModalSurface>

      <ModalSurface
        open={Boolean(confirmRevoke)}
        onClose={() => setConfirmRevoke(null)}
        eyebrow="Destructive action"
        title="Revoke API key"
        summary={
          confirmRevoke
            ? `Revoke ${confirmRevoke.name || confirmRevoke.id} and stop all future usage.`
            : undefined
        }
        footer={
          <>
            <button type="button" onClick={() => setConfirmRevoke(null)}>
              Cancel
            </button>
            <button
              type="button"
              className="btn-danger"
              disabled={!confirmRevoke || submittingId === confirmRevoke.id}
              onClick={() => confirmRevoke && void handleRevoke(confirmRevoke)}
            >
              {confirmRevoke && submittingId === confirmRevoke.id ? "Revoking..." : "Revoke key"}
            </button>
          </>
        }
      >
        {confirmRevoke ? (
          <p className="helper">User: {confirmRevoke.user_email} | Tier: {confirmRevoke.tier}</p>
        ) : null}
      </ModalSurface>

      <ModalSurface
        open={Boolean(confirmRotate)}
        onClose={() => setConfirmRotate(null)}
        eyebrow="Credential rotation"
        title="Rotate API key"
        summary={
          confirmRotate
            ? `Rotate ${confirmRotate.name || confirmRotate.id}. The current secret will be deactivated and replaced.`
            : undefined
        }
        footer={
          <>
            <button type="button" onClick={() => setConfirmRotate(null)}>
              Cancel
            </button>
            <button
              type="button"
              className="btn-primary"
              disabled={!confirmRotate || submittingId === confirmRotate.id}
              onClick={() => confirmRotate && void handleRotate(confirmRotate)}
            >
              {confirmRotate && submittingId === confirmRotate.id ? "Rotating..." : "Rotate key"}
            </button>
          </>
        }
      >
        {confirmRotate ? (
          <p className="helper">User: {confirmRotate.user_email} | Tier: {confirmRotate.tier}</p>
        ) : null}
      </ModalSurface>

      <ModalSurface
        open={Boolean(confirmDelete)}
        onClose={() => setConfirmDelete(null)}
        eyebrow="Permanent removal"
        title="Delete API key permanently"
        summary={
          confirmDelete
            ? `Delete ${confirmDelete.name || confirmDelete.id} from the database. This cannot be undone.`
            : undefined
        }
        footer={
          <>
            <button type="button" onClick={() => setConfirmDelete(null)}>
              Cancel
            </button>
            <button
              type="button"
              className="btn-danger"
              disabled={!confirmDelete || submittingId === confirmDelete.id}
              onClick={() => confirmDelete && void handleDelete(confirmDelete)}
            >
              {confirmDelete && submittingId === confirmDelete.id ? "Deleting..." : "Delete permanently"}
            </button>
          </>
        }
      >
        <p className="helper">
          This removes the stored record itself, not just active access. Prefer revoke if you need audit retention without hard deletion.
        </p>
      </ModalSurface>

      {pendingReveal ? (
        <SecretRevealModal
          secret={pendingReveal.secret}
          source={pendingReveal.source}
          keyName={pendingReveal.keyName}
          onDismiss={() => setPendingReveal(null)}
        />
      ) : null}
    </PageShell>
  );
}
