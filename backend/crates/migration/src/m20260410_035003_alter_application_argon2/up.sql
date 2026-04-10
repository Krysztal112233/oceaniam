UPDATE applications
SET
  configuration = jsonb_set(
    configuration,
    '{argon2}',
    '{"m_cost":12288,"t_cost":3,"p_cost":1}'::jsonb,
    true
  )
WHERE
  NOT (configuration ? 'argon2');

ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"authentication":{"issuer":"OceanIAM","audience":["OceanIAM"]},"enable_registration":false,"argon2":{"m_cost":12288,"t_cost":3,"p_cost":1}}'::jsonb;
