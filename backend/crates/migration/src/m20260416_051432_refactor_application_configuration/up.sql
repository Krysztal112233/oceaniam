UPDATE applications
SET
  configuration = jsonb_build_object(
    'auth',
    jsonb_build_object(
      'token',
      jsonb_build_object(
        'issuer',
        COALESCE(
          configuration #>> '{authentication,issuer}',
          'OceanIAM'
        ),
        'audience',
        COALESCE(
          configuration #> '{authentication,audience}',
          '["OceanIAM"]'::jsonb
        )
      ),
      'password',
      jsonb_build_object(
        'argon2',
        COALESCE(
          configuration -> 'argon2',
          '{"m_cost":12288,"t_cost":3,"p_cost":1}'::jsonb
        )
      )
    ),
    'registration',
    jsonb_build_object(
      'enabled',
      COALESCE(
        (configuration ->> 'enable_registration')::boolean,
        false
      )
    )
  );

ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"auth":{"token":{"issuer":"OceanIAM","audience":["OceanIAM"]},"password":{"argon2":{"m_cost":12288,"t_cost":3,"p_cost":1}}},"registration":{"enabled":false}}'::jsonb;
