"use client";

import Link from "next/link";

export interface QuotaExceededDetail {
  tier?: string;
  monthly_limit?: number;
  monthly_used?: number;
  upgrade_url?: string;
}

interface Props {
  open: boolean;
  detail?: QuotaExceededDetail;
  onDismiss: () => void;
}

/**
 * Modal shown when an engine call returns HTTP 402 (QUOTA_EXCEEDED).
 * Surfaces the user's monthly usage and a CTA to /pricing.
 *
 * Triggered from the request layer — see `src/lib/api.ts` for the
 * apiRequest hook that intercepts 402 responses.
 */
export function QuotaExceededModal({ open, detail, onDismiss }: Props) {
  if (!open) return null;

  const limit = detail?.monthly_limit ?? 50;
  const used = detail?.monthly_used ?? limit;
  const tier = detail?.tier ?? "free";
  const upgradeHref = detail?.upgrade_url ?? "/pricing";

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="quota-modal-title"
      onClick={onDismiss}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 100,
        background: "rgba(9, 9, 11, 0.78)",
        backdropFilter: "blur(8px)",
        WebkitBackdropFilter: "blur(8px)",
        display: "grid",
        placeItems: "center",
        padding: "1.5rem",
        animation: "biofield-fade-in 220ms var(--ease-out-expo) both",
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="biofield-panel"
        style={{
          padding: "2rem 1.75rem",
          maxWidth: "440px",
          width: "100%",
          display: "grid",
          gap: "1.1rem",
        }}
      >
        <p className="biofield-eyebrow" style={{ margin: 0 }}>
          Witness pool exhausted
        </p>
        <h2 id="quota-modal-title" style={{ margin: 0, fontSize: "1.4rem", fontWeight: 600 }}>
          You&rsquo;ve used all {limit} engine calls this month
        </h2>
        <p className="biofield-copy" style={{ margin: 0 }}>
          Your <strong>{tier}</strong> tier resets at the start of next
          month. Upgrade now for a deeper pool, lower per-call rates, and
          access to every consciousness engine.
        </p>

        <div
          style={{
            border: "1px solid var(--line-mid)",
            borderRadius: "var(--r-md)",
            padding: "0.85rem 1rem",
            display: "grid",
            gap: "0.35rem",
            background: "var(--surface-2)",
          }}
        >
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              fontSize: "0.82rem",
              color: "var(--text-2)",
            }}
          >
            <span>This month</span>
            <span>
              {used} / {limit}
            </span>
          </div>
          <div
            style={{
              height: "6px",
              borderRadius: "3px",
              background: "var(--line-faint)",
              overflow: "hidden",
            }}
          >
            <div
              style={{
                width: `${Math.min(100, (used / limit) * 100)}%`,
                height: "100%",
                background: "var(--signal)",
              }}
            />
          </div>
        </div>

        <div style={{ display: "flex", gap: "0.6rem", marginTop: "0.4rem" }}>
          <button
            type="button"
            onClick={onDismiss}
            className="biofield-button"
            style={{ flex: 1 }}
          >
            Maybe later
          </button>
          <Link
            href={upgradeHref}
            onClick={onDismiss}
            className="biofield-button biofield-button-primary"
            style={{ flex: 1, textAlign: "center", textDecoration: "none" }}
          >
            View plans
          </Link>
        </div>
      </div>
    </div>
  );
}
