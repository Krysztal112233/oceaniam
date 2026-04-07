DO $ext$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM pg_extension
    WHERE extname = 'pg_cron'
  ) THEN
    NULL;
  ELSIF NOT EXISTS (
    SELECT 1
    FROM pg_available_extensions
    WHERE name = 'pg_cron'
  ) THEN
    RAISE WARNING 'skipped pg_cron: extension is not available on this PostgreSQL instance';
  ELSE
    RAISE WARNING 'skipped pg_cron: extension is not enabled';
  END IF;
END $ext$;

DO $ext$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM pg_extension
    WHERE extname = 'pg_duckdb'
  ) THEN
    NULL;
  ELSIF NOT EXISTS (
    SELECT 1
    FROM pg_available_extensions
    WHERE name = 'pg_duckdb'
  ) THEN
    RAISE WARNING 'skipped pg_duckdb: extension is not available on this PostgreSQL instance';
  ELSE
    RAISE WARNING 'skipped pg_duckdb: extension is not enabled';
  END IF;
END $ext$;
