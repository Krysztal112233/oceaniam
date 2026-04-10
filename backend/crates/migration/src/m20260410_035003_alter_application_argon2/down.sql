ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"authentication":{"issuer":"OceanIAM","audience":["OceanIAM"]},"enable_registration":false}'::jsonb;

UPDATE applications
SET
  configuration = configuration - 'argon2'
WHERE
  configuration ? 'argon2';
