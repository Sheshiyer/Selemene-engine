-- G-22: Add range check constraint on readings.consciousness_level.
-- Values must be between 0 and 5 inclusive to match engine phase-gating logic.
-- Skips existing rows that already violate the range (none expected in prod).
ALTER TABLE readings
  ADD CONSTRAINT readings_consciousness_level_check
  CHECK (consciousness_level BETWEEN 0 AND 5)
  NOT VALID;

-- Validate existing rows separately (non-blocking, acquires ShareUpdateExclusiveLock).
ALTER TABLE readings
  VALIDATE CONSTRAINT readings_consciousness_level_check;
