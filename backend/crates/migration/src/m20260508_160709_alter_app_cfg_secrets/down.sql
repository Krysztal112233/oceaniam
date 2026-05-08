ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"auth":{"token":{"issuer":"OceanIAM","audience":["OceanIAM"]},"password":{"argon2":{"m_cost":12288,"t_cost":3,"p_cost":1}}},"registration":{"enabled":false}}'::jsonb;

UPDATE applications
SET
  configuration = configuration #- '{auth,totp}'
WHERE
  (configuration -> 'auth' -> 'totp') IS NOT NULL;
