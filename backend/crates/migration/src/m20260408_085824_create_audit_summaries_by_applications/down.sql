DO $$
DECLARE
  scheduled_job_id bigint;
BEGIN
  SELECT jobid
  INTO scheduled_job_id
  FROM cron.job
  WHERE jobname = 'refresh_audit_summary_by_application_every_10_minutes';

  IF scheduled_job_id IS NOT NULL THEN
    PERFORM cron.unschedule(scheduled_job_id);
  END IF;
END;
$$;

DROP MATERIALIZED VIEW IF EXISTS audit_summary_by_application;
