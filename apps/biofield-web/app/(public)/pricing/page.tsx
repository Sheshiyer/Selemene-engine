"use client";

import { useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { createCheckoutSession } from "@/lib/api";
import {
  getStoredAuthSession,
  subscribeToAuthSession,
} from "@/lib/auth";
import type { PlanCode } from "@selemene/noesis-sdk-ts";

interface PricingTier {
  code: PlanCode;
  name: string;
  monthly: string;
  monthlyAmount: number;
  credits: string;
  overage: string;
  highlights: string[];
  ctaLabel: string;
  highlight?: boolean;
}

const TIERS: PricingTier[] = [
  {
    code: "free",
    name: "Witness Free",
    monthly: "$0",
    monthlyAmount: 0,
    credits: "50 / month",
    overage: "Hard cap",
    highlights: [
      "Access to core engines",
      "Up to 50 engine queries / month",
      "Community support",
    ],
    ctaLabel: "Start free",
  },
  {
    code: "basic",
    name: "Witness Basic",
    monthly: "$9",
    monthlyAmount: 9,
    credits: "500 / month",
    overage: "$0.030 / credit",
    highlights: [
      "All 16 consciousness engines",
      "500 Witness Credits / month",
      "Opt-in overage at $0.030 / credit",
      "Email support",
    ],
    ctaLabel: "Choose Basic",
  },
  {
    code: "premium",
    name: "Witness Premium",
    monthly: "$29",
    monthlyAmount: 29,
    credits: "2 500 / month",
    overage: "$0.015 / credit",
    highlights: [
      "Everything in Basic",
      "2 500 Witness Credits / month",
      "Lowest overage rate ($0.015 / credit)",
      "Priority queue + bulk export",
      "Direct engineering support",
    ],
    ctaLabel: "Choose Premium",
    highlight: true,
  },
];

const FAQ: Array<{ q: string; a: string }> = [
  {
    q: "What is a Witness Credit?",
    a: "Each engine query — Panchanga, Human Design, Vimshottari, etc. — consumes one Witness Credit. Workflows that run multiple engines consume one credit per engine called.",
  },
  {
    q: "Do unused credits roll over?",
    a: "Yes. Up to 100% of unused credits roll into the next billing cycle (one cycle max), so a quiet month doesn't waste your subscription.",
  },
  {
    q: "What happens if I exceed my plan?",
    a: "Free tier is hard-capped — you'll be prompted to upgrade. Basic and Premium let you opt in to overage at the rate shown above. Paid users never hit a hard wall mid-flow.",
  },
  {
    q: "Can I cancel anytime?",
    a: "Yes. Cancellation honours the period you've already paid for; you keep access until the period ends, then drop to Free. No refunds for partial months.",
  },
  {
    q: "Which payment methods are supported?",
    a: "Cards (Visa, Mastercard, Amex), UPI (India), Apple Pay, Google Pay, and select regional methods. All processed by Dodo Payments as Merchant of Record — tax-inclusive, globally compliant.",
  },
];

export default function PricingPage() {
  const router = useRouter();
  const authSession = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [busyCode, setBusyCode] = useState<PlanCode | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function handleSelect(tier: PricingTier) {
    setError(null);
    if (!authSession) {
      // Send unauthenticated users to login first; preserve return path.
      router.push(`/login?return=${encodeURIComponent("/pricing")}`);
      return;
    }
    if (tier.code === "free") {
      // Free tier doesn't checkout — they're already on it after signup.
      router.push("/billing");
      return;
    }
    setBusyCode(tier.code);
    try {
      const { checkout_url } = await createCheckoutSession(
        authSession.token,
        tier.code,
      );
      window.location.href = checkout_url;
    } catch (e) {
      const msg =
        e instanceof Error ? e.message : "Could not start checkout. Try again.";
      setError(msg);
      setBusyCode(null);
    }
  }

  return (
    <main
      className="biofield-shell"
      style={{
        padding: "4rem 1.5rem",
        maxWidth: "1100px",
        margin: "0 auto",
      }}
    >
      <header style={{ textAlign: "center", marginBottom: "3rem" }}>
        <p className="biofield-eyebrow" style={{ marginBottom: "0.6rem" }}>
          Witness Plans
        </p>
        <h1
          style={{
            fontSize: "clamp(1.6rem, 3.5vw, 2.4rem)",
            fontWeight: 600,
            lineHeight: 1.15,
            margin: 0,
          }}
        >
          Pay for the depth you actually use.
        </h1>
        <p
          className="biofield-copy"
          style={{
            maxWidth: "560px",
            margin: "1rem auto 0",
            opacity: 0.78,
          }}
        >
          A unified credit pool across every consciousness engine. Roll over
          unused credits. Lower per-credit rates as you scale.
        </p>
      </header>

      {error && (
        <div
          role="alert"
          style={{
            border: "1px solid rgba(255, 80, 80, 0.32)",
            background: "rgba(255, 80, 80, 0.08)",
            color: "#ffaaaa",
            borderRadius: "var(--r-md)",
            padding: "0.85rem 1rem",
            marginBottom: "1.5rem",
            fontSize: "0.88rem",
          }}
        >
          {error}
        </div>
      )}

      <ul
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(260px, 1fr))",
          gap: "1.25rem",
          padding: 0,
          listStyle: "none",
          marginBottom: "4rem",
        }}
      >
        {TIERS.map((tier) => {
          const isCurrent = authSession?.tier?.toLowerCase() === tier.code;
          const isBusy = busyCode === tier.code;
          return (
            <li
              key={tier.code}
              className="biofield-panel"
              style={{
                padding: "1.6rem 1.4rem",
                display: "grid",
                gap: "1rem",
                position: "relative",
                outline: tier.highlight
                  ? "1px solid var(--accent-border)"
                  : "none",
                boxShadow: tier.highlight
                  ? "var(--inset-glow), 0 0 28px rgba(var(--accent-rgb), 0.12)"
                  : undefined,
              }}
            >
              {tier.highlight && (
                <span
                  style={{
                    position: "absolute",
                    top: "-10px",
                    left: "1.4rem",
                    fontSize: "0.65rem",
                    fontWeight: 600,
                    letterSpacing: "0.12em",
                    textTransform: "uppercase",
                    padding: "0.25rem 0.6rem",
                    borderRadius: "var(--r-pill)",
                    background: "var(--accent-dim)",
                    border: "1px solid var(--accent-border)",
                    color: "#d4d4ff",
                  }}
                >
                  Best value
                </span>
              )}

              <div>
                <p
                  className="biofield-eyebrow"
                  style={{ margin: 0, fontSize: "0.7rem" }}
                >
                  {tier.name}
                </p>
                <p
                  style={{
                    margin: "0.4rem 0 0",
                    fontSize: "2rem",
                    fontWeight: 600,
                    lineHeight: 1.1,
                  }}
                >
                  {tier.monthly}
                  <span
                    style={{
                      fontSize: "0.85rem",
                      fontWeight: 400,
                      opacity: 0.55,
                      marginLeft: "0.35rem",
                    }}
                  >
                    / month
                  </span>
                </p>
                <p
                  style={{
                    margin: "0.4rem 0 0",
                    fontSize: "0.88rem",
                    color: "var(--text-2)",
                  }}
                >
                  {tier.credits} Witness Credits
                  <br />
                  <span style={{ opacity: 0.6 }}>Overage: {tier.overage}</span>
                </p>
              </div>

              <ul
                style={{
                  listStyle: "none",
                  padding: 0,
                  margin: 0,
                  display: "grid",
                  gap: "0.4rem",
                  fontSize: "0.85rem",
                  color: "var(--text-2)",
                }}
              >
                {tier.highlights.map((h) => (
                  <li
                    key={h}
                    style={{
                      display: "flex",
                      gap: "0.55rem",
                      alignItems: "flex-start",
                    }}
                  >
                    <span
                      aria-hidden
                      style={{
                        marginTop: "0.45rem",
                        width: "4px",
                        height: "4px",
                        borderRadius: "50%",
                        background: "var(--accent)",
                        flexShrink: 0,
                      }}
                    />
                    <span>{h}</span>
                  </li>
                ))}
              </ul>

              <button
                type="button"
                onClick={() => handleSelect(tier)}
                disabled={isBusy || isCurrent}
                className={`biofield-button${
                  tier.highlight ? " biofield-button-primary" : ""
                }`}
                style={{ width: "100%" }}
              >
                {isCurrent
                  ? "Your current plan"
                  : isBusy
                    ? "Redirecting…"
                    : tier.ctaLabel}
              </button>
            </li>
          );
        })}
      </ul>

      <section
        aria-labelledby="faq-title"
        style={{ maxWidth: "720px", margin: "0 auto" }}
      >
        <h2
          id="faq-title"
          style={{
            fontSize: "1.3rem",
            fontWeight: 600,
            textAlign: "center",
            marginBottom: "1.4rem",
          }}
        >
          Common questions
        </h2>
        <div style={{ display: "grid", gap: "0.5rem" }}>
          {FAQ.map((item) => (
            <details
              key={item.q}
              className="biofield-panel"
              style={{ padding: "0.95rem 1.1rem" }}
            >
              <summary
                style={{
                  fontWeight: 500,
                  cursor: "pointer",
                  listStyle: "none",
                  outline: "none",
                  fontSize: "0.95rem",
                }}
              >
                {item.q}
              </summary>
              <p
                className="biofield-copy"
                style={{
                  marginTop: "0.7rem",
                  fontSize: "0.88rem",
                  opacity: 0.78,
                }}
              >
                {item.a}
              </p>
            </details>
          ))}
        </div>
      </section>

      <p
        style={{
          marginTop: "3rem",
          textAlign: "center",
          fontSize: "0.82rem",
          color: "var(--muted)",
        }}
      >
        All plans billed in USD. Tax inclusive. Powered by{" "}
        <Link href="https://dodopayments.com" target="_blank" rel="noreferrer">
          Dodo Payments
        </Link>{" "}
        as Merchant of Record.
      </p>
    </main>
  );
}
