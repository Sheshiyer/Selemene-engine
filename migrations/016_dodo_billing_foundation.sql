-- Migration: 016_dodo_billing_foundation
-- Description: Add Dodo-ready billing customer and webhook persistence surfaces.

CREATE TABLE IF NOT EXISTS billing_customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(32) NOT NULL,
    provider_customer_id VARCHAR(128) NOT NULL,
    email VARCHAR(255),
    mode VARCHAR(16) NOT NULL DEFAULT 'test',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT billing_customers_provider_nonempty CHECK (btrim(provider) <> ''),
    CONSTRAINT billing_customers_provider_customer_nonempty CHECK (btrim(provider_customer_id) <> ''),
    CONSTRAINT billing_customers_mode_check CHECK (mode IN ('test', 'live'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_billing_customers_provider_customer
    ON billing_customers(provider, provider_customer_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_billing_customers_user_provider
    ON billing_customers(user_id, provider);
CREATE INDEX IF NOT EXISTS idx_billing_customers_user_id
    ON billing_customers(user_id);

DROP TRIGGER IF EXISTS update_billing_customers_updated_at ON billing_customers;
CREATE TRIGGER update_billing_customers_updated_at
    BEFORE UPDATE ON billing_customers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS billing_webhook_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider VARCHAR(32) NOT NULL,
    provider_event_id VARCHAR(191) NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    signature TEXT,
    processing_state VARCHAR(16) NOT NULL DEFAULT 'pending',
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT billing_webhook_events_provider_nonempty CHECK (btrim(provider) <> ''),
    CONSTRAINT billing_webhook_events_event_id_nonempty CHECK (btrim(provider_event_id) <> ''),
    CONSTRAINT billing_webhook_events_event_type_nonempty CHECK (btrim(event_type) <> ''),
    CONSTRAINT billing_webhook_events_processing_state_check CHECK (
        processing_state IN ('pending', 'processed', 'ignored', 'failed')
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_billing_webhook_events_provider_event
    ON billing_webhook_events(provider, provider_event_id);
CREATE INDEX IF NOT EXISTS idx_billing_webhook_events_processing_state
    ON billing_webhook_events(processing_state, received_at DESC);

DROP TRIGGER IF EXISTS update_billing_webhook_events_updated_at ON billing_webhook_events;
CREATE TRIGGER update_billing_webhook_events_updated_at
    BEFORE UPDATE ON billing_webhook_events
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

ALTER TABLE billing_subscriptions
    ADD COLUMN IF NOT EXISTS provider_product_id VARCHAR(128),
    ADD COLUMN IF NOT EXISTS provider_price_id VARCHAR(128),
    ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb;

DROP INDEX IF EXISTS uq_billing_subscriptions_active_user;

ALTER TABLE billing_subscriptions
    DROP CONSTRAINT IF EXISTS billing_subscriptions_status_check;

ALTER TABLE billing_subscriptions
    ADD CONSTRAINT billing_subscriptions_status_check CHECK (
        status IN (
            'trialing',
            'active',
            'past_due',
            'on_hold',
            'canceled',
            'expired',
            'incomplete',
            'failed'
        )
    );

CREATE UNIQUE INDEX IF NOT EXISTS uq_billing_subscriptions_active_user
    ON billing_subscriptions(user_id)
    WHERE status IN ('trialing', 'active', 'past_due') AND canceled_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_billing_subscriptions_provider_customer
    ON billing_subscriptions(provider, provider_customer_id)
    WHERE provider_customer_id IS NOT NULL;
