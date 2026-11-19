-- Migration: 044_biofield_baselines_reconcile_legacy_shape
-- The production Supabase-era table predates 018. CREATE TABLE IF NOT EXISTS
-- preserved it without adding notes, which baseline list/create queries need.
-- Keep legacy columns, defaults, rows, views and dependencies intact.
-- This is a no-op on the canonical 018 shape and safe to replay.
ALTER TABLE biofield_baselines ADD COLUMN IF NOT EXISTS notes TEXT;
