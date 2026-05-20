-- 1. Add created_at columns to entities that track creation time
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT now();
ALTER TABLE applications ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT now();
ALTER TABLE users ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT now();
ALTER TABLE administrators ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT now();

-- 2. Platform-level summary: tracks how many of each entity type were created per minute
CREATE TABLE IF NOT EXISTS platform_summary (
  bucket       timestamptz NOT NULL,
  entity_type  text NOT NULL,
  event_count  bigint NOT NULL DEFAULT 0,
  PRIMARY KEY (bucket, entity_type)
);

-- 3. Application-scoped summary: tracks user creation per application per minute
CREATE TABLE IF NOT EXISTS application_summary (
  application_id  uuid NOT NULL,
  bucket          timestamptz NOT NULL,
  event_count     bigint NOT NULL DEFAULT 0,
  PRIMARY KEY (application_id, bucket)
);

-- 4. Trigger: tenant INSERT → platform_summary
CREATE OR REPLACE FUNCTION upsert_tenant_summary() RETURNS trigger AS $$
BEGIN
  INSERT INTO platform_summary (bucket, entity_type, event_count)
  VALUES (date_trunc('minute', NEW.created_at AT TIME ZONE 'UTC'), 'tenant', 1)
  ON CONFLICT (bucket, entity_type)
  DO UPDATE SET event_count = platform_summary.event_count + 1;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 5. Trigger: application INSERT → platform_summary
CREATE OR REPLACE FUNCTION upsert_application_summary() RETURNS trigger AS $$
BEGIN
  INSERT INTO platform_summary (bucket, entity_type, event_count)
  VALUES (date_trunc('minute', NEW.created_at AT TIME ZONE 'UTC'), 'application', 1)
  ON CONFLICT (bucket, entity_type)
  DO UPDATE SET event_count = platform_summary.event_count + 1;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 6. Trigger: user INSERT → platform_summary + application_summary
CREATE OR REPLACE FUNCTION upsert_user_summary() RETURNS trigger AS $$
BEGIN
  INSERT INTO platform_summary (bucket, entity_type, event_count)
  VALUES (date_trunc('minute', NEW.created_at AT TIME ZONE 'UTC'), 'user', 1)
  ON CONFLICT (bucket, entity_type)
  DO UPDATE SET event_count = platform_summary.event_count + 1;

  INSERT INTO application_summary (application_id, bucket, event_count)
  VALUES (NEW.application_id, date_trunc('minute', NEW.created_at AT TIME ZONE 'UTC'), 1)
  ON CONFLICT (application_id, bucket)
  DO UPDATE SET event_count = application_summary.event_count + 1;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 7. Trigger: administrator INSERT → platform_summary
CREATE OR REPLACE FUNCTION upsert_administrator_summary() RETURNS trigger AS $$
BEGIN
  INSERT INTO platform_summary (bucket, entity_type, event_count)
  VALUES (date_trunc('minute', NEW.created_at AT TIME ZONE 'UTC'), 'administrator', 1)
  ON CONFLICT (bucket, entity_type)
  DO UPDATE SET event_count = platform_summary.event_count + 1;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 8. Wire up triggers
DROP TRIGGER IF EXISTS trg_tenants_insert_summary ON tenants;
CREATE TRIGGER trg_tenants_insert_summary
AFTER INSERT ON tenants FOR EACH ROW
EXECUTE FUNCTION upsert_tenant_summary();

DROP TRIGGER IF EXISTS trg_applications_insert_summary ON applications;
CREATE TRIGGER trg_applications_insert_summary
AFTER INSERT ON applications FOR EACH ROW
EXECUTE FUNCTION upsert_application_summary();

DROP TRIGGER IF EXISTS trg_users_insert_summary ON users;
CREATE TRIGGER trg_users_insert_summary
AFTER INSERT ON users FOR EACH ROW
EXECUTE FUNCTION upsert_user_summary();

DROP TRIGGER IF EXISTS trg_administrators_insert_summary ON administrators;
CREATE TRIGGER trg_administrators_insert_summary
AFTER INSERT ON administrators FOR EACH ROW
EXECUTE FUNCTION upsert_administrator_summary();

-- 9. Backfill existing data
INSERT INTO platform_summary (bucket, entity_type, event_count)
SELECT date_trunc('minute', created_at AT TIME ZONE 'UTC'), 'tenant', COUNT(*)::bigint
FROM tenants GROUP BY 1
ON CONFLICT (bucket, entity_type) DO NOTHING;

INSERT INTO platform_summary (bucket, entity_type, event_count)
SELECT date_trunc('minute', created_at AT TIME ZONE 'UTC'), 'application', COUNT(*)::bigint
FROM applications GROUP BY 1
ON CONFLICT (bucket, entity_type) DO NOTHING;

INSERT INTO platform_summary (bucket, entity_type, event_count)
SELECT date_trunc('minute', created_at AT TIME ZONE 'UTC'), 'user', COUNT(*)::bigint
FROM users GROUP BY 1
ON CONFLICT (bucket, entity_type) DO NOTHING;

INSERT INTO platform_summary (bucket, entity_type, event_count)
SELECT date_trunc('minute', created_at AT TIME ZONE 'UTC'), 'administrator', COUNT(*)::bigint
FROM administrators GROUP BY 1
ON CONFLICT (bucket, entity_type) DO NOTHING;

INSERT INTO application_summary (application_id, bucket, event_count)
SELECT application_id, date_trunc('minute', created_at AT TIME ZONE 'UTC'), COUNT(*)::bigint
FROM users GROUP BY application_id, 2
ON CONFLICT (application_id, bucket) DO NOTHING;
