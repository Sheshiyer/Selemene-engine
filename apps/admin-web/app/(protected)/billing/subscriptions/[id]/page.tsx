"use client";

import Link from "next/link";
import { use, useEffect, useState } from "react";
import { ActionRail, SurfaceCard } from "@/components/admin-primitives";
import { StateBanner } from "@/components/admin-state";
import { ModalSurface } from "@/components/overlay-surface";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  cancelAdminBillingSubscription,
  getAdminBillingSubscription,
  getAdminSession
} from "@/lib/api";
import { hasPermission } from "@/lib/permissions";
import { statusPillClass } from "@/lib/status";
import type {
  AdminBillingSubscriptionItem,
  AdminSession
} from "@/types/admin";

function formatDateTime(value: string | null): string {
  if (!value) return "--";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export default function AdminBillingSubscriptionDetailPage({
  params
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = use(params);
  const [sub, setSub] = useState<AdminBillingSubscriptionItem | null>(null);
  const [session, setSession] = useState<AdminSession | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [confirmInput, setConfirmInput] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [actionResult, setActionResult] = useState<string | null>(null);

  useEffect(() => {
    const token = getAuthToken();
    if (!token) return;
    let cancelled = false;

    // Core read data — page MUST render this even if session lookup fails.
    getAdminBillingSubscription(token, id)
      .then((detail) => {
        if (!cancelled) {
          setSub(detail.subscription);
          setError(null);
        }
      })
      .catch((err) => {
        if (cancelled) return;
        setError(
          err instanceof ApiClientError
            ? err.message
            : err instanceof Error
              ? err.message
              : "Failed to load subscription"
        );
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    // Best-effort: only gates the "Force-cancel" destructive action.
    // Failure here leaves session=null which hides the cancel button —
    // it does NOT block subscription detail visibility.
    getAdminSession(token)
      .then((sess) => {
        if (!cancelled) setSession(sess);
      })
      .catch(() => {
        // Swallow — read-only view continues to work without action gating.
      });

    return () => {
      cancelled = true;
    };
  }, [id]);

  const canCancel =
    !!session?.permissions &&
    hasPermission(session.permissions, "admin:billing:subscriptions:cancel");

  async function handleCancel() {
    const token = getAuthToken();
    if (!token || !sub) return;
    setSubmitting(true);
    try {
      await cancelAdminBillingSubscription(token, id);
      const refreshed = await getAdminBillingSubscription(token, id);
      setSub(refreshed.subscription);
      setActionResult(
        "Subscription canceled locally. Verify state in the Dodo dashboard."
      );
    } catch (err) {
      setActionResult(
        err instanceof Error ? `Cancel failed: ${err.message}` : "Cancel failed"
      );
    } finally {
      setSubmitting(false);
      setConfirmOpen(false);
      setConfirmInput("");
    }
  }

  const idPrefix = sub?.id.slice(0, 8) ?? "";

  return (
    <PageShell
      title="Subscription detail"
      summary="Inspect a single Dodo subscription. Force-cancel locally if Dodo's cancel webhook was missed and reconcile is in read-only mode."
    >
      {error ? (
        <StateBanner variant="error" title="Unable to load" description={error} />
      ) : null}

      <Link
        href="/billing/subscriptions"
        className="helper"
        style={{ marginBottom: "0.6rem", display: "inline-block" }}
      >
        ← Back to subscriptions
      </Link>

      <SurfaceCard
        eyebrow="Subscription"
        title={sub ? `${sub.id.slice(0, 8)}… (${sub.status})` : "Loading…"}
        summary={
          sub
            ? `Provider: ${sub.provider} · created ${formatDateTime(sub.created_at)}`
            : undefined
        }
      >
        {loading && !sub ? (
          <p className="helper">Loading…</p>
        ) : sub ? (
          <dl
            className="definition-grid"
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))",
              gap: "0.9rem"
            }}
          >
            <div>
              <dt className="eyebrow">ID</dt>
              <dd style={{ fontFamily: "monospace", wordBreak: "break-all" }}>
                {sub.id}
              </dd>
            </div>
            <div>
              <dt className="eyebrow">User ID</dt>
              <dd style={{ fontFamily: "monospace", wordBreak: "break-all" }}>
                {sub.user_id}
              </dd>
            </div>
            <div>
              <dt className="eyebrow">Status</dt>
              <dd>
                <span className={statusPillClass(sub.status)}>{sub.status}</span>
              </dd>
            </div>
            <div>
              <dt className="eyebrow">Provider sub ID</dt>
              <dd style={{ fontFamily: "monospace" }}>
                {sub.provider_subscription_id ?? "—"}
              </dd>
            </div>
            <div>
              <dt className="eyebrow">Provider customer ID</dt>
              <dd style={{ fontFamily: "monospace" }}>
                {sub.provider_customer_id ?? "—"}
              </dd>
            </div>
            <div>
              <dt className="eyebrow">Cancel at period end</dt>
              <dd>{sub.cancel_at_period_end ? "yes" : "no"}</dd>
            </div>
            <div>
              <dt className="eyebrow">Period start</dt>
              <dd>{formatDateTime(sub.current_period_start)}</dd>
            </div>
            <div>
              <dt className="eyebrow">Period end</dt>
              <dd>{formatDateTime(sub.current_period_end)}</dd>
            </div>
            <div>
              <dt className="eyebrow">Canceled at</dt>
              <dd>{formatDateTime(sub.canceled_at)}</dd>
            </div>
            <div>
              <dt className="eyebrow">Updated</dt>
              <dd>{formatDateTime(sub.updated_at)}</dd>
            </div>
          </dl>
        ) : null}
      </SurfaceCard>

      {actionResult ? (
        <StateBanner
          variant={actionResult.startsWith("Cancel failed") ? "error" : "success"}
          title="Cancel result"
          description={actionResult}
        />
      ) : null}

      <SurfaceCard eyebrow="Actions" title="Operator interventions">
        {!canCancel ? (
          <p className="helper">
            You do not have <code>admin:billing:subscriptions:cancel</code>.
            Cancel is disabled.
          </p>
        ) : (
          <ActionRail>
            <button
              type="button"
              className="button button-danger"
              disabled={
                !sub ||
                sub.status === "canceled" ||
                !sub.provider_subscription_id
              }
              onClick={() => setConfirmOpen(true)}
            >
              Force-cancel locally
            </button>
          </ActionRail>
        )}
      </SurfaceCard>

      <ModalSurface
        open={confirmOpen}
        onClose={() => {
          setConfirmOpen(false);
          setConfirmInput("");
        }}
        eyebrow="Destructive"
        title="Force-cancel subscription locally?"
        summary={`This sets status=canceled in our DB and drops the user's tier to free. It does NOT cancel in Dodo — do that separately. Type ${idPrefix} to confirm.`}
        footer={
          <>
            <button
              type="button"
              className="button button-secondary"
              onClick={() => {
                setConfirmOpen(false);
                setConfirmInput("");
              }}
            >
              Cancel
            </button>
            <button
              type="button"
              className="button button-danger"
              disabled={submitting || confirmInput !== idPrefix}
              onClick={handleCancel}
            >
              {submitting ? "Canceling…" : "Confirm cancel"}
            </button>
          </>
        }
      >
        <input
          type="text"
          value={confirmInput}
          onChange={(e) => setConfirmInput(e.target.value)}
          placeholder={idPrefix}
          className="input"
          style={{ fontFamily: "monospace", width: "100%" }}
        />
      </ModalSurface>
    </PageShell>
  );
}
