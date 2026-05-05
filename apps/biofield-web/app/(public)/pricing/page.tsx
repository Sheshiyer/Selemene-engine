// Wave 1.3 placeholder. T13 builds the real pricing page with the 3-tier card
// grid, FAQ accordion, and upgrade CTAs that POST to /api/billing/checkout.

import type { PlanCode } from "@selemene/noesis-sdk-ts";

interface PricingTier {
  code: PlanCode;
  display: string;
  monthly: string;
  credits: string;
  overage: string;
}

const TIERS: PricingTier[] = [
  { code: "free", display: "Free", monthly: "$0", credits: "50 / mo", overage: "—" },
  { code: "basic", display: "Basic", monthly: "$9", credits: "500 / mo", overage: "$0.030 / credit" },
  { code: "premium", display: "Premium", monthly: "$29", credits: "2 500 / mo", overage: "$0.015 / credit" },
];

export default function PricingPage() {
  return (
    <main style={{ padding: "4rem 2rem", maxWidth: "960px", margin: "0 auto" }}>
      <h1>Choose your tier</h1>
      <p style={{ opacity: 0.7 }}>
        Witness Credits unlock every consciousness engine. Numbers below are
        provisional; T06 finalises them in the Dodo dashboard.
      </p>
      <ul style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "1rem", padding: 0, listStyle: "none", marginTop: "2rem" }}>
        {TIERS.map((tier) => (
          <li key={tier.code} style={{ border: "1px solid #2a2a2a", padding: "1.5rem", borderRadius: "8px" }}>
            <h2>{tier.display}</h2>
            <p style={{ fontSize: "1.5rem", fontWeight: 600 }}>{tier.monthly} <small style={{ opacity: 0.6, fontSize: "0.8rem" }}>/ month</small></p>
            <p>{tier.credits} Witness Credits</p>
            <p style={{ opacity: 0.7, fontSize: "0.9rem" }}>Overage: {tier.overage}</p>
            <p style={{ marginTop: "1rem", opacity: 0.5, fontSize: "0.8rem" }}>
              Checkout button lands in T13.
            </p>
          </li>
        ))}
      </ul>
    </main>
  );
}
