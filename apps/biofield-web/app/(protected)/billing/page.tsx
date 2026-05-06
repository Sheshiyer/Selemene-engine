"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import {
  createPortalSession,
  getBillingBalance,
} from "@/lib/api";
import {
  getStoredAuthSession,
  subscribeToAuthSession,
} from "@/lib/auth";
import type { BalanceResponse } from "@selemene/noesis-sdk-ts";

function formatPeriodEnd(iso: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  return d.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

function tierDisplayName(tier: string): string {
  const map: Record<string, string> = {
    free: "Witness Free",
    basic: "Witness Basic",
    premium: "Witness Premium",
    enterprise: "Witness Enterprise",
  };
  return map[tier?.toLowerCase()] ?? tier;
}

interface CreditsCardProps {
  balance: BalanceResponse;
}

function CreditsCard({ balance }: CreditsCardProps) {
  // Free tier: total = monthly_limit baseline. Paid: we don't currently
  // know "credits_total" without an extra Dodo call, so percentage shows
  // remaining-only as a soft signal.
  const tierBaseline: Record<string, number> = {
    free: 50,
    basic: 500,
    premium: 2500,
    enterprise: 10000,
  };
  const baseline = tierBaseline[balance.tier?.toLowerCase()] ?? 0;
  const pct =
    baseline > 0
      ? Math.max(
          0,
          Math.min(100, Math.round((balance.credits_remaining / baseline) * 100)),
        )
      : 0;

  return (
    <div
      className="biofield-panel"
      style={{ padding: "1.4rem 1.4rem", display: "grid", gap: "0.85rem" }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
        <p className="biofield-eyebrow" style={{ margin: 0 }}>
          Witness Credits
        </p>
        <span style={{ fontSize: "0.78rem", color: "var(--muted)" }}>
          Source: {balance.source.replace("_", " ")}
        </span>
      </div>
      <p style={{ margin: 0, fontSize: "2rem", fontWeight: 600, lineHeight: 1.1 }}>
        {balance.credits_remaining.toLocaleString()}
        {baseline > 0 && (
          <span
            style={{
              fontSize: "0.85rem",
              fontWeight: 400,
              opacity: 0.55,
              marginLeft: "0.45rem",
            }}
          >
            / {baseline.toLocaleString()}
          </span>
        )}
      </p>
      {baseline > 0 && (
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
              width: `${pct}%`,
              height: "100%",
              background:
                pct < 15 ? "var(--signal)" : "var(--accent)",
              transition: "width 280ms var(--ease-out-expo)",
            }}
          />
        </div>
      )}
      {balance.overage_charged && balance.overage_charged !== "0" && (
        <p style={{ margin: 0, fontSize: "0.82rem", color: "var(--text-2)" }}>
          Overage charged this period: <strong>${balance.overage_charged}</strong>
        </p>
      )}
    </div>
  );
}

export default function BillingPage() {
  const searchParams = useSearchParams();
  const checkoutStatus = searchParams.get("status");

  const authSession = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [balance, setBalance] = useState<BalanceResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [portalBusy, setPortalBusy] = useState(false);
  const [portalError, setPortalError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    if (!authSession?.token) {
      setLoading(false);
      return;
    }
    setLoading(true);
    setLoadError(null);
    getBillingBalance(authSession.token)
      .then((b) => {
        if (!cancelled) setBalance(b);
      })
      .catch((e) => {
        if (cancelled) return;
        const msg = e instanceof Error ? e.message : "Could not load billing details.";
        setLoadError(msg);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [authSession?.token]);

  async function handleManage() {
    if (!authSession?.token) return;
    setPortalError(null);
    setPortalBusy(true);
    try {
      const { portal_url } = await createPortalSession(authSession.token);
      window.location.href = portal_url;
    } catch (e) {
      const msg =
        e instanceof Error ? e.message : "Could not open the customer portal.";
      setPortalError(msg);
      setPortalBusy(false);
    }
  }

  const isPaid = balance && balance.tier !== "free";
  const isPastDue = false; // T26 will derive from subscription state via webhook

  return (
    <section
      className="biofield-panel"
      style={{
        padding: "1.8rem",
        display: "grid",
        gap: "1.4rem",
        maxWidth: "640px",
      }}
    >
      <header style={{ display: "grid", gap: "0.3rem" }}>
        <p className="biofield-eyebrow" style={{ margin: 0 }}>
          Billing
        </p>
        <h1 style={{ margin: 0, fontSize: "1.4rem", fontWeight: 600 }}>
          Manage your Witness plan
        </h1>
      </header>

      {checkoutStatus === "success" && (
        <div
          role="status"
          style={{
            border: "1px solid var(--accent-border)",
            background: "var(--accent-dim)",
            color: "#d4d4ff",
            borderRadius: "var(--r-md)",
            padding: "0.85rem 1rem",
            fontSize: "0.88rem",
          }}
        >
          ✓ Payment received. Your plan upgrade may take a moment to reflect
          here while the webhook lands.
        </div>
      )}

      {isPastDue && (
        <div
          role="alert"
          style={{
            border: "1px solid rgba(255, 179, 71, 0.4)",
            background: "rgba(255, 179, 71, 0.08)",
            color: "var(--signal)",
            borderRadius: "var(--r-md)",
            padding: "0.85rem 1rem",
            fontSize: "0.88rem",
          }}
        >
          ⚠ Your last payment failed. Update your payment method to keep
          your subscription active.
        </div>
      )}

      {loading && (
        <p className="biofield-copy" style={{ opacity: 0.6 }}>
          Loading…
        </p>
      )}

      {loadError && (
        <div
          role="alert"
          style={{
            border: "1px solid rgba(255, 80, 80, 0.32)",
            background: "rgba(255, 80, 80, 0.08)",
            color: "#ffaaaa",
            borderRadius: "var(--r-md)",
            padding: "0.85rem 1rem",
            fontSize: "0.88rem",
          }}
        >
          {loadError}
        </div>
      )}

      {balance && !loading && (
        <>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
              gap: "0.75rem",
            }}
          >
            <div
              className="biofield-panel"
              style={{ padding: "1rem 1.1rem" }}
            >
              <p className="biofield-eyebrow" style={{ margin: 0, fontSize: "0.65rem" }}>
                Current tier
              </p>
              <p style={{ margin: "0.35rem 0 0", fontSize: "1.1rem", fontWeight: 500 }}>
                {tierDisplayName(balance.tier)}
              </p>
            </div>
            <div
              className="biofield-panel"
              style={{ padding: "1rem 1.1rem" }}
            >
              <p className="biofield-eyebrow" style={{ margin: 0, fontSize: "0.65rem" }}>
                Renews on
              </p>
              <p style={{ margin: "0.35rem 0 0", fontSize: "1.1rem", fontWeight: 500 }}>
                {formatPeriodEnd(balance.period_end)}
              </p>
              {balance.cancel_at_period_end && (
                <p
                  style={{
                    margin: "0.3rem 0 0",
                    fontSize: "0.78rem",
                    color: "var(--signal)",
                  }}
                >
                  Cancels at period end
                </p>
              )}
            </div>
          </div>

          <CreditsCard balance={balance} />
        </>
      )}

      <div
        style={{
          display: "flex",
          gap: "0.6rem",
          flexWrap: "wrap",
          marginTop: "0.4rem",
        }}
      >
        {isPaid ? (
          <button
            type="button"
            onClick={handleManage}
            disabled={portalBusy}
            className="biofield-button biofield-button-primary"
          >
            {portalBusy ? "Opening portal…" : "Manage subscription"}
          </button>
        ) : (
          <Link
            href="/pricing"
            className="biofield-button biofield-button-primary"
            style={{ textDecoration: "none" }}
          >
            View plans
          </Link>
        )}
        <Link
          href="/pricing"
          className="biofield-button"
          style={{ textDecoration: "none" }}
        >
          See all tiers
        </Link>
      </div>

      {portalError && (
        <p
          role="alert"
          style={{
            margin: 0,
            fontSize: "0.82rem",
            color: "#ffaaaa",
          }}
        >
          {portalError}
        </p>
      )}

      <p style={{ margin: 0, fontSize: "0.78rem", color: "var(--muted)" }}>
        The Dodo customer portal handles plan changes, payment method
        updates, invoice downloads, and cancellation. Your subscription
        will be updated here automatically once the change webhook lands.
      </p>
    </section>
  );
}
