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
    BEGIN
      CREATE EXTENSION pg_cron;
    EXCEPTION
      WHEN insufficient_privilege THEN
        RAISE WARNING 'skipped pg_cron: current user has no privilege to create the extension';
      WHEN read_only_sql_transaction THEN
        RAISE WARNING 'skipped pg_cron: current transaction is read-only';
      WHEN OTHERS THEN
        RAISE WARNING 'skipped pg_cron: %', SQLERRM;
    END;
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
    BEGIN
      CREATE EXTENSION pg_duckdb;
    EXCEPTION
      WHEN insufficient_privilege THEN
        RAISE WARNING 'skipped pg_duckdb: current user has no privilege to create the extension';
      WHEN read_only_sql_transaction THEN
        RAISE WARNING 'skipped pg_duckdb: current transaction is read-only';
      WHEN OTHERS THEN
        RAISE WARNING 'skipped pg_duckdb: %', SQLERRM;
    END;
  END IF;
END $ext$;
