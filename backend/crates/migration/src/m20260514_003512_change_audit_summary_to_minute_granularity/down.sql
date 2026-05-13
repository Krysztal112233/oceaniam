-- 1. Drop trigger and function
DROP TRIGGER IF EXISTS trg_audits_insert_summary ON audits;
DROP FUNCTION IF EXISTS upsert_audit_summary();

-- 2. Rename current table to a temp name
ALTER TABLE audit_summary_by_application
  RENAME TO audit_summary_by_application_minute;

-- 3. Recreate old daily table
CREATE TABLE audit_summary_by_application (
  application_id uuid NOT NULL,
  day date NOT NULL,
  audit_type audit_type NOT NULL,
  event_count bigint NOT NULL DEFAULT 0,
  PRIMARY KEY (application_id, day, audit_type)
);

-- 4. Re-aggregate minute-level data back to daily
INSERT INTO audit_summary_by_application (application_id, day, audit_type, event_count)
SELECT
  application_id,
  (bucket AT TIME ZONE 'UTC')::date,
  audit_type,
  SUM(event_count)::bigint
FROM audit_summary_by_application_minute
GROUP BY 1, 2, 3
ON CONFLICT (application_id, day, audit_type) DO NOTHING;

-- 5. Recreate old trigger function
CREATE FUNCTION upsert_audit_summary () RETURNS trigger AS $$
BEGIN
    IF NEW.payload #>> '{data,application_id}' IS NOT NULL THEN
        INSERT INTO audit_summary_by_application (application_id, day, audit_type, event_count)
        VALUES (
            (NEW.payload #>> '{data,application_id}')::uuid,
            (NEW.created_at AT TIME ZONE 'UTC')::date,
            NEW.audit_type,
            1
        )
        ON CONFLICT (application_id, day, audit_type)
        DO UPDATE SET event_count = audit_summary_by_application.event_count + 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_audits_insert_summary
AFTER INSERT ON audits FOR EACH ROW
EXECUTE FUNCTION upsert_audit_summary ();

-- 6. Drop minute-level table
DROP TABLE IF EXISTS audit_summary_by_application_minute;
