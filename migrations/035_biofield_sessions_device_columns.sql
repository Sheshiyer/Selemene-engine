-- Migration: 035_biofield_sessions_device_columns
-- Description: Restore client_device_id and viewer_version columns for admin biofield session listing.

ALTER TABLE biofield_sessions
    ADD COLUMN IF NOT EXISTS client_device_id VARCHAR(128),
    ADD COLUMN IF NOT EXISTS viewer_version VARCHAR(64);
