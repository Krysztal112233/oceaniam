UPDATE administrators
SET
  role = NULL
WHERE
  name = 'root';

DELETE FROM application_roles
WHERE
  is_system = true;
