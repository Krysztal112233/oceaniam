UPDATE applications
SET
  comment = ''
WHERE
  comment IS NULL;

DELETE FROM application_secrets AS s
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      application_secret_bindings AS b
    WHERE
      b.secret_id = s.id
  );
