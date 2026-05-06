-- Migration: 021_processed_webhook_events
-- Description: Atomic idempotency table for inbound provider webhooks. The
--              webhook handler INSERTs the (webhook_id) on first delivery; on
--              retries the PK conflict makes the insert a no-op and the
--              handler returns 200 dedup=true without mutating state.
--
-- Pruning: rows older than 90 days can be deleted by maintenance. Reconciliation
-- cron (T24) covers any drift before the prune threshold.

CREATE TABLE IF NOT EXISTS processed_webhook_events (
    webhook_id   VARCHAR(128) PRIMARY KEY,
    provider     VARCHAR(32)  NOT NULL,
    event_type   VARCHAR(64)  NOT NULL,
    processed_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT processed_webhook_events_provider_nonempty CHECK (btrim(provider) <> '')
);

CREATE INDEX IF NOT EXISTS idx_processed_webhook_events_processed_at
    ON processed_webhook_events(processed_at);

CREATE INDEX IF NOT EXISTS idx_processed_webhook_events_provider_type
    ON processed_webhook_events(provider, event_type);
