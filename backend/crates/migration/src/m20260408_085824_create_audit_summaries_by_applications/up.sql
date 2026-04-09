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
  (payload -> 'data' ->> 'application_id')::uuid,
  (created_at AT TIME ZONE 'UTC')::date,
  audit_type;

CREATE UNIQUE INDEX idx_audit_summary_by_application_primary ON audit_summary_by_application (application_id, day, audit_type);
