-- Migration: 022_engine_usage_monthly
-- Description: Free-tier monthly engine-call counter. Paid tiers (basic,
--              premium) are gated server-side by Dodo's credit balance; free
--              tier has no Dodo entitlement so we count locally and hard-cap.
--
-- One row per (user, calendar month). Created lazily on first engine call.

CREATE TABLE IF NOT EXISTS engine_usage_monthly (
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    count        BIGINT NOT NULL DEFAULT 0,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, period_start),
    CONSTRAINT engine_usage_monthly_count_nonneg CHECK (count >= 0)
);

CREATE INDEX IF NOT EXISTS idx_engine_usage_monthly_period
    ON engine_usage_monthly(period_start);
