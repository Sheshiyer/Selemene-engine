-- Migration: 038_billing_mode_control
-- Description: Durable operator override for free/disabled payment behavior.
-- Dodo activation remains release-controlled by BILLING_MODE=dodo and is not
-- representable in this table.

CREATE TABLE IF NOT EXISTS billing_mode_control (
    id SMALLINT PRIMARY KEY DEFAULT 1,
    mode VARCHAR(16) NOT NULL,
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT billing_mode_control_singleton CHECK (id = 1),
    CONSTRAINT billing_mode_control_mode_check CHECK (mode IN ('free', 'disabled'))
);

COMMENT ON TABLE billing_mode_control IS
    'Durable free/disabled operator override. Release BILLING_MODE remains the Dodo safety ceiling.';

DROP TRIGGER IF EXISTS update_billing_mode_control_updated_at ON billing_mode_control;
CREATE TRIGGER update_billing_mode_control_updated_at
    BEFORE UPDATE ON billing_mode_control
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
