UPDATE applications
SET
  configuration = jsonb_set(
    configuration,
    '{development_accounts}',
    '{"enabled":true,"default_ttl_seconds":3600,"max_ttl_seconds":86400}'::jsonb,
    true
  )
WHERE
  (configuration -> 'development_accounts') IS NULL
  OR (configuration -> 'development_accounts') = 'null'::jsonb;

ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"auth":{"token":{"issuer":"OceanIAM","audience":["OceanIAM"]},"password":{"argon2":{"m_cost":12288,"t_cost":3,"p_cost":1}},"totp":{"encryption_key":""}},"registration":{"enabled":false},"development_accounts":{"enabled":true,"default_ttl_seconds":3600,"max_ttl_seconds":86400}}'::jsonb;
