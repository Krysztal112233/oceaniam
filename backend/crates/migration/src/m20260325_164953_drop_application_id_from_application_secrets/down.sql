UPDATE application_secrets AS s
SET
  application_id = binding.application_id
FROM
  (
    SELECT DISTINCT
      ON (secret_id) secret_id,
      application_id
    FROM
      application_secret_bindings
    ORDER BY
      secret_id,
      application_id
  ) AS binding
WHERE
  s.id = binding.secret_id;
