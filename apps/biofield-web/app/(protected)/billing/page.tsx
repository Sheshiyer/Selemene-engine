// Wave 1.3 placeholder. T14 builds the real billing dashboard:
//   - current tier + period_end (from /api/v1/billing/balance via Rust)
//   - Witness Credit balance with progress meter
//   - "Manage subscription" button → /api/billing/portal
//   - dunning banner (T26) when subscription.on_hold

export default function BillingPage() {
  return (
    <section className="biofield-panel" style={{ padding: "2rem" }}>
      <h1 style={{ marginTop: 0 }}>Billing</h1>
      <p className="biofield-copy">
        The billing dashboard surfaces your active tier, remaining Witness
        Credits, and a portal link for plan changes. It lands in T14.
      </p>
      <p className="biofield-copy" style={{ opacity: 0.6, fontSize: "0.85rem" }}>
        Need to upgrade? Visit <a href="/pricing">/pricing</a>.
      </p>
    </section>
  );
}
