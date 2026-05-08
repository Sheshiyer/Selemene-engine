-- Migration: 025_usage_logs_2027_partitions
-- Description: Pre-create 2027 usage_logs partitions (and cover through Jan 2028).
--              Closes G-19 / GitHub issue #692.
--
-- The ensure_usage_log_partitions() function is idempotent (skips existing partitions).
-- max allowed months_ahead = 24.  From any month in 2026, calling with 20 covers
-- well into 2028.  We also explicitly CREATE the 2027 partitions in case the function
-- cannot be invoked (e.g., during schema-only migrations without EXECUTE permission).

-- 1. Extend dynamic function coverage through Jan 2028
SELECT ensure_usage_log_partitions(20);

-- 2. Belt-and-suspenders: explicit CREATE for every 2027 month
--    Each statement is a no-op if the partition already exists.
DO $$
DECLARE
  months TEXT[] := ARRAY[
    '2027-01','2027-02','2027-03','2027-04','2027-05','2027-06',
    '2027-07','2027-08','2027-09','2027-10','2027-11','2027-12'
  ];
  m TEXT;
  part_name TEXT;
  range_start DATE;
  range_end DATE;
BEGIN
  FOREACH m IN ARRAY months LOOP
    range_start := (m || '-01')::DATE;
    range_end   := range_start + INTERVAL '1 month';
    part_name   := 'usage_logs_' || replace(m, '-', '_');

    IF to_regclass(part_name) IS NULL THEN
      EXECUTE format(
        'CREATE TABLE %I PARTITION OF usage_logs FOR VALUES FROM (%L) TO (%L)',
        part_name, range_start, range_end
      );
      RAISE NOTICE 'Created partition: %', part_name;
    ELSE
      RAISE NOTICE 'Partition already exists: %', part_name;
    END IF;
  END LOOP;
END $$;
