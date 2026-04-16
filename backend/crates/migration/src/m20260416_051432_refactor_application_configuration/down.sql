ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"authentication":{"issuer":"OceanIAM","audience":["OceanIAM"]},"enable_registration":false,"argon2":{"m_cost":12288,"t_cost":3,"p_cost":1}}'::jsonb;

UPDATE applications
SET
  configuration = jsonb_build_object(
    'authentication',
    jsonb_build_object(
      'issuer',
      COALESCE(
        configuration #>> '{auth,token,issuer}',
        'OceanIAM'
      ),
      'audience',
      COALESCE(
        configuration #> '{auth,token,audience}',
        '["OceanIAM"]'::jsonb
      )
    ),
    'enable_registration',
    COALESCE(
      (configuration #>> '{registration,enabled}')::boolean,
      false
    ),
    'argon2',
    COALESCE(
      configuration #> '{auth,password,argon2}',
      '{"m_cost":12288,"t_cost":3,"p_cost":1}'::jsonb
    )
  );
