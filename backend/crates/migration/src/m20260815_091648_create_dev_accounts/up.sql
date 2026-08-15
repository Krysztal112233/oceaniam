-- Development accounts: time-limited application users.
--
-- The advisory transaction lock serializes concurrent runs of this migration:
-- the integration-test harness migrates many isolated schemas against the same
-- shared database, and extension/queue creation below is database-global.
SELECT pg_advisory_xact_lock(2026072800);

CREATE EXTENSION IF NOT EXISTS pgmq;

ALTER TABLE subjects ADD COLUMN expires_at TIMESTAMPTZ NULL;

SELECT pgmq.create('dev_account_expiration');

ALTER TYPE audit_type ADD VALUE IF NOT EXISTS 'create_dev_account';
ALTER TYPE audit_type ADD VALUE IF NOT EXISTS 'dev_account_expired';
