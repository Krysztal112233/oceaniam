-- Restore the column default exactly as the original m20260816 left it
-- (including the dead `auth.totp` key).
-- Note: the row-level removal of `auth.totp` is intentionally NOT reverted.
-- Rows that carried the key cannot be identified afterwards, and the key is
-- dead config that AuthConfiguration ignores on deserialization.

ALTER TABLE applications
ALTER COLUMN configuration
SET DEFAULT '{"auth":{"token":{"issuer":"OceanIAM","audience":["OceanIAM"]},"password":{"argon2":{"m_cost":12288,"t_cost":3,"p_cost":1}},"totp":{"encryption_key":""}},"registration":{"enabled":false},"development_accounts":{"enabled":true,"default_ttl_seconds":3600,"max_ttl_seconds":86400}}'::jsonb;
