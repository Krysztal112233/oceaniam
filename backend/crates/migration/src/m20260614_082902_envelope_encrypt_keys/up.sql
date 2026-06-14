UPDATE credentials SET totp = NULL;

UPDATE applications
SET configuration = configuration - 'auth' || jsonb_build_object(
    'auth',
    (configuration -> 'auth') - 'totp'
);

ALTER TABLE applications
ALTER COLUMN configuration SET DEFAULT
'{"auth":{"token":{"issuer":"OceanIAM","audience":["OceanIAM"]},"password":{"argon2":{"m_cost":12288,"t_cost":3,"p_cost":1}}},"registration":{"enabled":false}}'::jsonb;
