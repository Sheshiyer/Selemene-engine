-- Migration: 014_plan_catalog_billing_subscriptions
-- Description: Add canonical plan catalog and billing subscription schema.

CREATE TABLE IF NOT EXISTS plan_catalog (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT plan_catalog_code_nonempty CHECK (btrim(code) <> ''),
    CONSTRAINT plan_catalog_display_name_nonempty CHECK (btrim(display_name) <> ''),
    CONSTRAINT plan_catalog_code_key UNIQUE (code)
);

CREATE INDEX IF NOT EXISTS idx_plan_catalog_is_active ON plan_catalog(is_active);

DROP TRIGGER IF EXISTS update_plan_catalog_updated_at ON plan_catalog;
CREATE TRIGGER update_plan_catalog_updated_at
    BEFORE UPDATE ON plan_catalog
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS billing_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES plan_catalog(id) ON DELETE RESTRICT,
    provider VARCHAR(32) NOT NULL,
    provider_customer_id VARCHAR(128),
    provider_subscription_id VARCHAR(128),
    status VARCHAR(32) NOT NULL,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    canceled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT billing_subscriptions_provider_nonempty CHECK (btrim(provider) <> ''),
    CONSTRAINT billing_subscriptions_status_check CHECK (
        status IN ('trialing', 'active', 'past_due', 'canceled', 'expired', 'incomplete')
    )
);

CREATE INDEX IF NOT EXISTS idx_billing_subscriptions_user_id
    ON billing_subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_billing_subscriptions_plan_id
    ON billing_subscriptions(plan_id);
CREATE INDEX IF NOT EXISTS idx_billing_subscriptions_provider_status
    ON billing_subscriptions(provider, status);
CREATE UNIQUE INDEX IF NOT EXISTS uq_billing_subscriptions_provider_subscription
    ON billing_subscriptions(provider, provider_subscription_id)
    WHERE provider_subscription_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_billing_subscriptions_active_user
    ON billing_subscriptions(user_id)
    WHERE status IN ('trialing', 'active', 'past_due') AND canceled_at IS NULL;

DROP TRIGGER IF EXISTS update_billing_subscriptions_updated_at ON billing_subscriptions;
CREATE TRIGGER update_billing_subscriptions_updated_at
    BEFORE UPDATE ON billing_subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

INSERT INTO plan_catalog (code, display_name, description)
VALUES
    ('free', 'Free', 'Default free access tier'),
    ('basic', 'Basic', 'Entry paid access tier'),
    ('premium', 'Premium', 'Advanced paid access tier'),
    ('enterprise', 'Enterprise', 'Enterprise access tier')
ON CONFLICT (code) DO NOTHING;

INSERT INTO plan_catalog (code, display_name, description)
SELECT DISTINCT
    LOWER(BTRIM(u.tier)) AS code,
    INITCAP(LOWER(BTRIM(u.tier))) AS display_name,
    'Backfilled from legacy users.tier'
FROM users u
WHERE COALESCE(BTRIM(u.tier), '') <> ''
ON CONFLICT (code) DO NOTHING;

INSERT INTO billing_subscriptions (
    user_id,
    plan_id,
    provider,
    provider_customer_id,
    provider_subscription_id,
    status,
    cancel_at_period_end,
    current_period_start,
    current_period_end
)
SELECT
    u.id,
    pc.id,
    'legacy',
    NULL,
    CONCAT('legacy:', u.id::text),
    'active',
    false,
    u.created_at,
    NULL
FROM users u
JOIN plan_catalog pc
    ON pc.code = COALESCE(NULLIF(LOWER(BTRIM(u.tier)), ''), 'free')
WHERE NOT EXISTS (
    SELECT 1
    FROM billing_subscriptions bs
    WHERE bs.user_id = u.id
      AND bs.status IN ('trialing', 'active', 'past_due')
      AND bs.canceled_at IS NULL
);

CREATE OR REPLACE VIEW user_active_plan_resolutions AS
SELECT
    bs.user_id,
    pc.id AS plan_id,
    pc.code AS plan_code,
    pc.display_name AS plan_display_name,
    bs.provider,
    bs.status,
    bs.provider_customer_id,
    bs.provider_subscription_id,
    bs.current_period_start,
    bs.current_period_end,
    bs.cancel_at_period_end
FROM billing_subscriptions bs
JOIN plan_catalog pc ON pc.id = bs.plan_id
WHERE bs.status IN ('trialing', 'active', 'past_due')
  AND bs.canceled_at IS NULL;
