ALTER TABLE administrators
ADD COLUMN phc VARCHAR;

UPDATE administrators
SET
  phc = c.phc
FROM
  credentials c
WHERE
  administrators.id = c.id;

TRUNCATE TABLE credentials;
