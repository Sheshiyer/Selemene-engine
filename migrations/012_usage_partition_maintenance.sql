-- Migration: 012_usage_partition_maintenance
-- Description: Add usage_logs partition maintenance function for future months.

CREATE OR REPLACE FUNCTION ensure_usage_log_partitions(months_ahead INTEGER DEFAULT 3)
RETURNS TABLE (
    partition_name TEXT,
    partition_start DATE,
    partition_end DATE,
    created BOOLEAN
)
LANGUAGE plpgsql
AS $$
DECLARE
    base_month DATE := date_trunc('month', CURRENT_DATE)::DATE;
    month_offset INTEGER;
    range_start DATE;
    range_end DATE;
    target_partition TEXT;
BEGIN
    IF months_ahead < 0 OR months_ahead > 24 THEN
        RAISE EXCEPTION 'months_ahead must be between 0 and 24';
    END IF;

    FOR month_offset IN 0..months_ahead LOOP
        range_start := (base_month + make_interval(months => month_offset))::DATE;
        range_end := (base_month + make_interval(months => month_offset + 1))::DATE;
        target_partition := format('usage_logs_%s', to_char(range_start, 'YYYY_MM'));

        IF to_regclass(target_partition) IS NULL THEN
            EXECUTE format(
                'CREATE TABLE %I PARTITION OF usage_logs FOR VALUES FROM (%L) TO (%L)',
                target_partition,
                range_start,
                range_end
            );
            created := TRUE;
        ELSE
            created := FALSE;
        END IF;

        partition_name := target_partition;
        partition_start := range_start;
        partition_end := range_end;
        RETURN NEXT;
    END LOOP;
END;
$$;
