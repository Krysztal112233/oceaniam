UPDATE applications
SET
  configuration = jsonb_set(
    configuration - 'allow_registration',
    '{enable_registration}',
    COALESCE(
      configuration -> 'enable_registration',
      configuration -> 'allow_registration',
      'false'::jsonb
    ),
    true
  )
WHERE
  NOT (configuration ? 'enable_registration')
  OR configuration ? 'allow_registration';

ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"authentication":{"issuer":"OceanIAM","audience":["OceanIAM"]},"enable_registration":false}'::jsonb;
