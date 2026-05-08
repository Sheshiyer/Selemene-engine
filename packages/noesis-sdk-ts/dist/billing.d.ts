export declare const BILLING_PROVIDER: "dodo_payments";
export type BillingProvider = typeof BILLING_PROVIDER;
export declare const PLAN_CODES: readonly ["free", "basic", "premium", "enterprise"];
export type PlanCode = (typeof PLAN_CODES)[number];
export declare const SUBSCRIPTION_STATUSES: readonly ["incomplete", "trialing", "active", "past_due", "canceled", "expired"];
export type SubscriptionStatus = (typeof SUBSCRIPTION_STATUSES)[number];
export declare const DODO_INBOUND_EVENT_TYPES: readonly ["subscription.active", "subscription.updated", "subscription.on_hold", "subscription.cancelled", "subscription.failed", "payment.succeeded", "payment.failed", "credit.added", "credit.deducted", "credit.balance_low", "credit.overage_charged"];
export type DodoInboundEventType = (typeof DODO_INBOUND_EVENT_TYPES)[number];
export interface BillingForwardRequest {
    webhook_id: string;
    webhook_timestamp: string;
    event_type: DodoInboundEventType;
    payload: Record<string, unknown>;
}
export type BillingForwardResponse = {
    status: "ok";
} | {
    status: "dedup";
};
export interface UsageEventMetadata {
    engine_id: string;
    tier: PlanCode;
    internal_user_id: string;
}
export interface UsageEvent {
    event_id: string;
    customer_id: string;
    event_name: "noesis.engine_query";
    timestamp: string;
    metadata: UsageEventMetadata;
}
export interface UsageIngestRequest {
    events: UsageEvent[];
}
export interface BalanceResponse {
    credits_remaining: number;
    /** String to preserve Dodo's decimal precision (e.g. "0", "1.50"). */
    overage_charged: string;
    /** ISO-8601 from billing_subscriptions.current_period_end, or null. */
    period_end: string | null;
    tier: PlanCode;
    cancel_at_period_end: boolean;
    source: "dodo" | "tier_default";
}
export interface CheckoutCreateRequest {
    plan_code: PlanCode;
}
export interface CheckoutCreateResponse {
    checkout_url: string;
}
export interface PortalCreateResponse {
    portal_url: string;
}
//# sourceMappingURL=billing.d.ts.map