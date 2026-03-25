INSERT INTO
  application_secret_bindings (secret_id, application_id)
SELECT
  id,
  application_id
FROM
  application_secrets
ON CONFLICT (secret_id, application_id) DO NOTHING
