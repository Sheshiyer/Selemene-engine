-- Migration: 007_progression_logs_default_uuid
-- Description: Ensure progression_logs.id auto-generates UUID values.

ALTER TABLE IF EXISTS progression_logs
ALTER COLUMN id SET DEFAULT gen_random_uuid();
