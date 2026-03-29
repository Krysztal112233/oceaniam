ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"authentication":{"issuer":"OceanIAM","audience":["OceanIAM"]}}'::jsonb;

UPDATE applications
SET
  configuration = configuration - 'enable_registration'
WHERE
  configuration ? 'enable_registration';
