-- m20260816 re-introduced the dead `auth.totp` key into the column default
-- (m20260614 had already removed it, and AuthConfiguration has no such field).
-- Purge it from existing rows and from the column default.

UPDATE applications
SET
  configuration = configuration - 'auth' || jsonb_build_object(
    'auth',
    (configuration -> 'auth') - 'totp'
  )
WHERE
  (configuration -> 'auth' -> 'totp') IS NOT NULL;

ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"auth":{"token":{"issuer":"OceanIAM","audience":["OceanIAM"]},"password":{"argon2":{"m_cost":12288,"t_cost":3,"p_cost":1}}},"registration":{"enabled":false},"development_accounts":{"enabled":true,"default_ttl_seconds":3600,"max_ttl_seconds":86400}}'::jsonb;
