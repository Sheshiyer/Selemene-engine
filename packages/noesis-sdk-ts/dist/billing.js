// Frozen contract types — must match .context/billing/contracts.md and the
// Rust structs in crates/noesis-api/src/billing.rs. Update all three together.
export const BILLING_PROVIDER = "dodo_payments";
export const PLAN_CODES = ["free", "basic", "premium", "enterprise"];
export const SUBSCRIPTION_STATUSES = [
    "incomplete",
    "trialing",
    "active",
    "past_due",
    "canceled",
    "expired",
];
export const DODO_INBOUND_EVENT_TYPES = [
    "subscription.active",
    "subscription.updated",
    "subscription.on_hold",
    "subscription.cancelled",
    "subscription.failed",
    "payment.succeeded",
    "payment.failed",
    "credit.added",
    "credit.deducted",
    "credit.balance_low",
    "credit.overage_charged",
];
//# sourceMappingURL=billing.js.map