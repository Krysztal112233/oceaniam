DROP TRIGGER IF EXISTS trg_audits_insert_summary ON audits;

DROP FUNCTION IF EXISTS upsert_audit_summary();

DROP TABLE IF EXISTS audit_summary_by_application;

CREATE MATERIALIZED VIEW audit_summary_by_application AS
SELECT
  (payload -> 'data' ->> 'application_id')::uuid AS application_id,
  (created_at AT TIME ZONE 'UTC')::date AS day,
  audit_type,
  COUNT(*)::bigint AS event_count
FROM
  audits
WHERE
  payload -> 'data' ->> 'application_id' IS NOT NULL
GROUP BY
  1,
  2,
  3;

CREATE UNIQUE INDEX idx_audit_summary_by_application_primary ON audit_summary_by_application (application_id, day, audit_type);
