-- 1. Drop old trigger and function
DROP TRIGGER IF EXISTS trg_audits_insert_summary ON audits;
DROP FUNCTION IF EXISTS upsert_audit_summary();

-- 2. Rename old table to preserve data
ALTER TABLE audit_summary_by_application
  RENAME TO audit_summary_by_application_old;

-- 3. Create new table with minute-level bucket instead of day
CREATE TABLE audit_summary_by_application (
  application_id uuid NOT NULL,
  bucket timestamptz NOT NULL,
  audit_type audit_type NOT NULL,
  event_count bigint NOT NULL DEFAULT 0,
  PRIMARY KEY (application_id, bucket, audit_type)
);

-- 4. Migrate old data: day → UTC midnight of that day
INSERT INTO audit_summary_by_application (application_id, bucket, audit_type, event_count)
SELECT
  application_id,
  day AT TIME ZONE 'UTC',
  audit_type,
  event_count
FROM audit_summary_by_application_old;

-- 5. Create new trigger function with minute-level truncation
CREATE FUNCTION upsert_audit_summary () RETURNS trigger AS $$
BEGIN
    IF NEW.payload #>> '{data,application_id}' IS NOT NULL THEN
        INSERT INTO audit_summary_by_application (application_id, bucket, audit_type, event_count)
        VALUES (
            (NEW.payload #>> '{data,application_id}')::uuid,
            date_trunc('minute', NEW.created_at AT TIME ZONE 'UTC'),
            NEW.audit_type,
            1
        )
        ON CONFLICT (application_id, bucket, audit_type)
        DO UPDATE SET event_count = audit_summary_by_application.event_count + 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_audits_insert_summary
AFTER INSERT ON audits FOR EACH ROW
EXECUTE FUNCTION upsert_audit_summary ();

-- 6. Drop old table
DROP TABLE IF EXISTS audit_summary_by_application_old;
