-- 1. Drop triggers
DROP TRIGGER IF EXISTS trg_tenants_insert_summary ON tenants;

DROP TRIGGER IF EXISTS trg_applications_insert_summary ON applications;

DROP TRIGGER IF EXISTS trg_users_insert_summary ON users;

DROP TRIGGER IF EXISTS trg_administrators_insert_summary ON administrators;

-- 2. Drop trigger functions
DROP FUNCTION IF EXISTS upsert_tenant_summary ();

DROP FUNCTION IF EXISTS upsert_application_summary ();

DROP FUNCTION IF EXISTS upsert_user_summary ();

DROP FUNCTION IF EXISTS upsert_administrator_summary ();

-- 3. Drop summary tables
DROP TABLE IF EXISTS platform_summary;

DROP TABLE IF EXISTS application_summary;

-- 4. Drop created_at columns
ALTER TABLE tenants
DROP COLUMN IF EXISTS created_at;

ALTER TABLE applications
DROP COLUMN IF EXISTS created_at;

ALTER TABLE users
DROP COLUMN IF EXISTS created_at;

ALTER TABLE administrators
DROP COLUMN IF EXISTS created_at;
