DO $cron$
BEGIN
  IF to_regprocedure('cron.unschedule(text)') IS NULL THEN
    RAISE NOTICE 'skipped cron.unschedule: pg_cron not available';
  ELSIF current_setting('transaction_read_only') = 'on' THEN
    RAISE NOTICE 'skipped cron.unschedule: read-only transaction';
  ELSE
    PERFORM cron.unschedule('refresh-audit-summary-by-application');
  END IF;
END $cron$;

DROP MATERIALIZED VIEW IF EXISTS audit_summary_by_application;
