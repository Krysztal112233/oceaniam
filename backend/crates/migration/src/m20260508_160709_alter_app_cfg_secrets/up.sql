UPDATE applications
SET
  configuration = jsonb_set(
    configuration,
    '{auth,totp}',
    jsonb_build_object(
      'encryption_key',
      replace(
        gen_random_uuid()::text,
        '-',
        ''
      )
    ),
    true
  )
WHERE
  (configuration -> 'auth' -> 'totp') IS NULL;

ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"auth":{"token":{"issuer":"OceanIAM","audience":["OceanIAM"]},"password":{"argon2":{"m_cost":12288,"t_cost":3,"p_cost":1}},"totp":{"encryption_key":""}},"registration":{"enabled":false}}'::jsonb;
