DROP MATERIALIZED VIEW IF EXISTS audit_summary_by_application CASCADE;

CREATE TABLE audit_summary_by_application (
  application_id uuid NOT NULL,
  day date NOT NULL,
  audit_type audit_type NOT NULL,
  event_count bigint NOT NULL DEFAULT 0,
  PRIMARY KEY (application_id, day, audit_type)
);

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

INSERT INTO
  audit_summary_by_application (application_id, day, audit_type, event_count)
SELECT
  (payload #>> '{data,application_id}')::uuid AS application_id,
  (created_at AT TIME ZONE 'UTC')::date AS day,
  audit_type,
  COUNT(*)::bigint AS event_count
FROM
  audits
WHERE
  payload #>> '{data,application_id}' IS NOT NULL
GROUP BY
  1,
  2,
  3
ON CONFLICT (application_id, day, audit_type) DO NOTHING;
