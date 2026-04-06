SET
  duckdb.force_execution = false;

DO $ext$ BEGIN
  CREATE EXTENSION IF NOT EXISTS pg_cron;
EXCEPTION
  WHEN insufficient_privilege THEN RAISE NOTICE 'skipped pg_cron: insufficient privilege';
  WHEN read_only_sql_transaction THEN RAISE NOTICE 'skipped pg_cron: read-only transaction';
END $ext$;

DO $ext$ BEGIN
  CREATE EXTENSION IF NOT EXISTS pg_duckdb;
EXCEPTION
  WHEN insufficient_privilege THEN RAISE NOTICE 'skipped pg_duckdb: insufficient privilege';
  WHEN read_only_sql_transaction THEN RAISE NOTICE 'skipped pg_duckdb: read-only transaction';
END $ext$;

CREATE MATERIALIZED VIEW IF NOT EXISTS audit_summary_by_application AS
SELECT
  CAST(created_at AS date) AS day,
  json_extract_string (payload, '$.data.application_id') AS application_id,
  json_extract_string (payload, '$.kind') AS audit_type,
  COUNT(*) AS event_count
FROM
  audits
WHERE
  json_extract_string (payload, '$.data.application_id') IS NOT NULL
GROUP BY
  CAST(created_at AS date),
  json_extract_string (payload, '$.data.application_id'),
  json_extract_string (payload, '$.kind')
WITH
  DATA;

CREATE UNIQUE INDEX IF NOT EXISTS idx_audit_summary_by_application_key ON audit_summary_by_application (day, application_id, audit_type);

DO $cron$
BEGIN
  IF to_regprocedure('cron.schedule(text,text,text)') IS NULL THEN
    RAISE NOTICE 'skipped cron.schedule: pg_cron not available';
  ELSIF current_setting('transaction_read_only') = 'on' THEN
    RAISE NOTICE 'skipped cron.schedule: read-only transaction';
  ELSE
    PERFORM cron.schedule(
      'refresh-audit-summary-by-application',
      '* * * * *',
      $$REFRESH MATERIALIZED VIEW CONCURRENTLY audit_summary_by_application$$
    );
  END IF;
END $cron$;
