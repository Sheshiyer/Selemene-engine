// Frozen contract types — must match .context/billing/contracts.md and the
// Rust structs in crates/noesis-api/src/billing.rs. Update all three together.

export const BILLING_PROVIDER = "dodo_payments" as const;
export type BillingProvider = typeof BILLING_PROVIDER;

export const PLAN_CODES = ["free", "basic", "premium", "enterprise"] as const;
export type PlanCode = (typeof PLAN_CODES)[number];

export const SUBSCRIPTION_STATUSES = [
  "incomplete",
  "trialing",
  "active",
  "past_due",
  "canceled",
  "expired",
] as const;
export type SubscriptionStatus = (typeof SUBSCRIPTION_STATUSES)[number];

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
] as const;
export type DodoInboundEventType = (typeof DODO_INBOUND_EVENT_TYPES)[number];

// Forward envelope sent from biofield-web webhook route → noesis-api
// POST /internal/billing/events
export interface BillingForwardRequest {
  webhook_id: string;
  webhook_timestamp: string;
  event_type: DodoInboundEventType;
  payload: Record<string, unknown>;
}

export type BillingForwardResponse =
  | { status: "ok" }
  | { status: "dedup" };

// Outbound usage event sent from noesis-api → Dodo /usage-events/ingest
export interface UsageEventMetadata {
  engine_id: string;
  tier: PlanCode;
  internal_user_id: string;
}

export interface UsageEvent {
  event_id: string;
  customer_id: string;
  event_name: "noesis.engine_query";
  timestamp: string; // ISO-8601
  metadata: UsageEventMetadata;
}

export interface UsageIngestRequest {
  events: UsageEvent[];
}

// Customer balance proxy — biofield-web → noesis-api → Dodo
export interface BalanceResponse {
  credits_remaining: number;
  credits_total: number;
  period_end: string | null; // ISO-8601 or null for free tier
  overage_enabled: boolean;
  tier: PlanCode;
  source: "dodo" | "tier_default";
}

// Checkout creation — biofield-web App Router route
export interface CheckoutCreateRequest {
  plan_code: PlanCode;
}

export interface CheckoutCreateResponse {
  checkout_url: string;
}

// Customer portal
export interface PortalCreateResponse {
  portal_url: string;
}
