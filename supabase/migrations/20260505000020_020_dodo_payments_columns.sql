-- Migration: 020_dodo_payments_columns
-- Description: Additive Dodo Payments support — extends existing billing_subscriptions
--              (created in 014) with provider-side mapping fields, exposes a Dodo
--              customer-ID lookup on users, and lets plan_catalog map internal codes
--              to Dodo product IDs.
--
-- This migration is purely additive: no rename, no drop, no data mutation.
-- The legacy backfill rows from 014 (provider='legacy') remain untouched.

-- 1. Users gain a Dodo customer ID (one-to-one mapping)
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS dodo_customer_id VARCHAR(128);

CREATE UNIQUE INDEX IF NOT EXISTS uq_users_dodo_customer_id
    ON users(dodo_customer_id)
    WHERE dodo_customer_id IS NOT NULL;

-- 2. Plan catalog rows can carry a Dodo product ID (one per plan code)
ALTER TABLE plan_catalog
    ADD COLUMN IF NOT EXISTS dodo_product_id VARCHAR(128);

CREATE UNIQUE INDEX IF NOT EXISTS uq_plan_catalog_dodo_product_id
    ON plan_catalog(dodo_product_id)
    WHERE dodo_product_id IS NOT NULL;

-- 3. Subscriptions can carry provider-specific metadata (entitlement balance
--    snapshots, raw event refs, etc). Existing rows default to '{}'.
ALTER TABLE billing_subscriptions
    ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb;

-- 4. Webhook handlers look subscriptions up by (provider, provider_customer_id)
--    when the event payload only carries a customer ID. Non-unique because a
--    single Dodo customer can have multiple historical subscriptions (one
--    active + several canceled/expired). Uniqueness on the active row is
--    already guaranteed by uq_billing_subscriptions_active_user from 014.
CREATE INDEX IF NOT EXISTS idx_billing_subscriptions_provider_customer
    ON billing_subscriptions(provider, provider_customer_id)
    WHERE provider_customer_id IS NOT NULL;
